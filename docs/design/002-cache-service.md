---
feature: Cache Foundation (Sync Guards)
status: Draft
author: Jack
ticket: N/A
date_created: 2026-01-30
tags: [cache, performance, refactor, redb, moka, rkyv]
---

# Tech Spec: Cache Foundation (Sync Guards)

> **Note**: See `docs/design/README.md` for usage instructions and T-Shirt sizing.

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The current cache foundation uses owned-value reads and async traits. This forces cloning, hides allocations, and blocks zero-copy optimizations available in redb/rkyv. For the concrete backends we use today (moka sync + redb), the core API can be synchronous and then wrapped for async callers via an explicit adapter that offloads blocking work.

Related design work lives in `docs/cache-foundation-design.md` (analysis and rationale). This tech spec extracts the implementable core into a coherent, minimal plan.

### 1.2 Goals & Non-Goals

**Goals**
- Zero-copy reads for redb + rkyv using guard-based access.
- Pure sync cache traits (no async in the core API).
- Validation of archived data on every access (validate-once per guard).
- Separate timestamp table (native u64) for fast staleness checks.
- Provide iterator-based `keys()` and `keys_where()` using transaction-owned iterators for redb.
- Optional async adapter that returns owned values only.

**Non-Goals**
- Redis/network cache support in the core traits.
- Async streaming APIs in core traits.
- Guaranteeing a fully zero-allocation write path in all cases (write-path optimizations are allowed but not required for the initial refactor).
- Rewriting the entire cache subsystem beyond interfaces and guard design.

### 1.3 Constraints (The Hard Limits)

- **Pure sync traits** for reader/writer operations.
- **Zero-copy reads** via guards (no `V: Clone` in read paths).
- **rkyv validation is mandatory** before access (no unchecked access).
- **redb iterators are transaction-scoped**, so streaming must own the transaction.
- **Unaligned rkyv access** uses `features = ["unaligned"]` plus field annotations.
- **Timestamp persistence** must survive restarts (use UNIX_EPOCH time).

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

Read path (sync, zero-copy):

```rust
let guard = reader.get(&key)?;
if let Some(guard) = guard {
    // The view type is backend-specific:
    // - memory guard derefs to `V`
    // - disk guard derefs to `Archived<V>`
    use_view(&*guard);
}
```

Coordinator usage must branch by variant because memory and disk guards deref to different types:

```rust
match coordinator.get(&key)? {
    None => {}
    Some(CoordinatorGuard::Memory(g)) => use_value(&*g),
    Some(CoordinatorGuard::Disk(g)) => use_archived(&*g),
}
```

Staleness checks (fast, no value materialization):

```rust
if let Some(ts) = reader.timestamp(&key)? {
    if ts.is_stale(ttl) {
        writer.delete(&key)?;
    }
}
```

Keys iteration (sync, iterator-based):

```rust
use CacheKeysExt;
use CachePrefixExt;

for key in reader.keys()? {
    let key = key?;
    handle_key(key);
}

for key in reader.keys_where(prefix)? {
    let key = key?;
    handle_key(key);
}
```

Async usage (owned values only):

```rust
let async_reader = AsyncCacheReader::new(reader);
let owned = async_reader.get_owned(&key).await?;
```

### 2.2 Mental Model

Think of the cache as a **synchronous, guard-based view** over two backends:

- Memory: `moka::sync::Cache` with cheap Arc clones.
- Disk: `redb` with mmap-backed `AccessGuard` and rkyv archived views.

The **guard** is the unit of borrowing. It guarantees that the underlying storage stays valid for the duration of use, and it exposes the appropriate view type without hidden allocation.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

- **CacheReader<K>**: sync get/has/timestamp/len. Returns guard-based views.
- **CacheWriter<K, V>**: sync put/delete/clear. Writes value and timestamp atomically.
- **Guard model**: `CacheGuard` uses `Deref<Target = View>` so Moka can deref to `V` and Redb to `Archived<V>`.
- **Keys extension traits**: `CacheKeysExt` and `CachePrefixExt` with iterator-based APIs.
- **Backfill**: event-driven observers; async work is off the read path and uses spawn_blocking for sync operations.

### 3.2 Data Models

```rust
pub trait CacheGuard: Deref<Target = Self::Target> + Send {
    type Target: ?Sized;
    fn as_bytes(&self) -> &[u8];
}

pub trait CacheReader<K>: Send + Sync + 'static
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
{
    type Value: Send + Sync + 'static;
    type View: ?Sized;
    type Guard<'a>: CacheGuard<Target = Self::View> where Self: 'a;

    fn get(&self, key: &K) -> Result<Option<Self::Guard<'_>>, CacheError>;

    /// Returns whether the key exists.
    ///
    /// Note: this is intentionally *not* specified as a cache-read. For moka, use `contains_key`-
    /// style semantics so `has()` does not reset idle timers or update read popularity.
    fn has(&self, key: &K) -> Result<bool, CacheError>;

    fn timestamp(&self, key: &K) -> Result<Option<Timestamp>, CacheError>;

    /// Returns the number of entries.
    ///
    /// This may be an estimate for some backends (e.g., moka reports an approximate count that can
    /// be made more accurate by running pending maintenance tasks).
    fn len(&self) -> Result<usize, CacheError>;
    fn is_empty(&self) -> Result<bool, CacheError> {
        Ok(self.len()? == 0)
    }
}

pub trait CacheWriter<K, V>: Send + Sync + 'static
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    fn put(&self, key: &K, value: &V, timestamp: Timestamp) -> Result<(), CacheError>;
    fn put_now(&self, key: &K, value: &V) -> Result<(), CacheError> {
        self.put(key, value, Timestamp::now())
    }
    fn delete(&self, key: &K) -> Result<bool, CacheError>;
    fn clear(&self) -> Result<(), CacheError>;
    fn put_many(&self, entries: &[(&K, &V)]) -> Result<(), CacheError>
    where
        V: Clone,
    {
        let timestamp = Timestamp::now();
        for (k, v) in entries {
            self.put(k, v, timestamp)?;
        }
        Ok(())
    }
}
```

Timestamp model:

```rust
pub struct Timestamp {
    nanos_since_epoch: u64,
}
```

- Stored in a separate redb table as native u64.
- Derived from `SystemTime::now()` relative to `UNIX_EPOCH`.

Guard types:

```rust
pub struct MokaGuard<V> {
    inner: Arc<(Timestamp, V)>,
}

impl<V> Deref for MokaGuard<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.inner.1
    }
}

impl<V> CacheGuard for MokaGuard<V>
where
    V: Send + 'static,
{
    fn as_bytes(&self) -> &[u8] { &[] }
}

pub struct RedbGuard<'txn, V>
where
    V: Archive,
{
    _guard: AccessGuard<'txn, [u8]>,
    archived: &'txn Archived<V>,
}

impl<'txn, V> RedbGuard<'txn, V>
where
    V: Archive,
    Archived<V>: for<'a> CheckBytes<HighValidator<'a>>,
{
    pub fn new(guard: AccessGuard<'txn, [u8]>) -> Result<Self, CacheError> {
        let bytes = guard.value();
        let archived = rkyv::access::<Archived<V>, rancor::Error>(bytes)
            .map_err(|e| CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("{:?}", e),
            })?;
        Ok(Self { _guard: guard, archived })
    }
}

impl<'txn, V> Deref for RedbGuard<'txn, V>
where
    V: Archive,
{
    type Target = Archived<V>;

    fn deref(&self) -> &Self::Target {
        self.archived
    }
}

impl<'txn, V> CacheGuard for RedbGuard<'txn, V>
where
    V: Archive + Send + 'static,
    Archived<V>: Send,
{
    fn as_bytes(&self) -> &[u8] { self._guard.value() }
}
```

Coordinator guard:

```rust
pub enum CoordinatorGuard<'a, V>
where
    V: Archive,
{
    Memory(MokaGuard<V>),
    Disk(RedbGuard<'a, V>),
}

// NOTE: `CoordinatorGuard` is a tagged union. It intentionally does NOT implement `Deref` or
// `CacheGuard` because `MokaGuard<V>` derefs to `V` while `RedbGuard<'a, V>` derefs to
// `Archived<V>`.
```

Async adapter + owned conversion:

```rust
pub trait GuardToOwned {
    type Owned;
    fn to_owned(&self) -> Result<Self::Owned, CacheError>;
}

impl<V> GuardToOwned for MokaGuard<V>
where
    V: Clone,
{
    type Owned = V;
    fn to_owned(&self) -> Result<Self::Owned, CacheError> {
        Ok(self.inner.1.clone())
    }
}

impl<'txn, V> GuardToOwned for RedbGuard<'txn, V>
where
    V: Archive,
    Archived<V>: Deserialize<V, HighDeserializer>,
{
    type Owned = V;
    fn to_owned(&self) -> Result<Self::Owned, CacheError> {
        RedbGuard::to_owned(self)
    }
}

pub struct AsyncCacheReader<R> {
    reader: R,
}

impl<R, K> AsyncCacheReader<R>
where
    R: CacheReader<K> + Clone + 'static,
    K: Clone + 'static,
{
    pub async fn get_owned(&self, key: &K) -> Result<Option<R::Value>, CacheError>
    where
        for<'a> R::Guard<'a>: GuardToOwned<Owned = R::Value>,
    { /* spawn_blocking */ }
}
```

Keys and prefix iterators (extension traits):

```rust
pub trait CacheKeysExt<K> {
    type KeysIter<'a>: Iterator<Item = Result<K, CacheError>>
    where
        Self: 'a;

    fn keys(&self) -> Result<Self::KeysIter<'_>, CacheError>;
}

pub trait CachePrefixExt<K> {
    type KeysWhereIter<'a>: Iterator<Item = Result<K, CacheError>>
    where
        Self: 'a;

    fn keys_where(&self, prefix: &str) -> Result<Self::KeysWhereIter<'_>, CacheError>;
}

pub struct RedbKeysIter<'a, K> {
    _txn: redb::ReadTransaction,
    iter: redb::TableIterator<'a>,
    codec: &'a dyn KeyCodec<K>,
}

pub struct RedbPrefixIter<'a, K> {
    _txn: redb::ReadTransaction,
    iter: redb::TableRangeIterator<'a>,
    codec: &'a dyn KeyCodec<K>,
}
```

Codec and key encoding:

```rust
pub trait Codec<K, V>: Send + Sync {
    type ArchivedValue: ?Sized;

    fn encode_key(&self, key: &K) -> Result<Vec<u8>, CacheError>;
    fn decode_key(&self, bytes: &[u8]) -> Result<K, CacheError>;

    fn serialized_size(&self, value: &V) -> Result<usize, CacheError>;
    fn serialize_into(&self, value: &V, buf: &mut [u8]) -> Result<usize, CacheError>;
    fn access<'a>(&self, bytes: &'a [u8]) -> Result<&'a Self::ArchivedValue, CacheError>;
    fn deserialize(&self, archived: &Self::ArchivedValue) -> Result<V, CacheError>;
}

pub trait KeyCodec<K> {
    fn decode_key(&self, bytes: &[u8]) -> Result<K, CacheError>;
}
```

Backfill events and observers:

```rust
#[derive(Clone)]
pub enum CacheEvent<K, V>
where
    K: Clone,
    V: Clone,
{
    Hit { key: K, source: CacheLayer },
    Miss { key: K },
    Write { key: K, value: V },
}

#[derive(Clone, Copy, Debug)]
pub enum CacheLayer {
    Memory,
    Disk,
}

pub trait CacheObserver<K, V>: Send + Sync {
    fn on_event(&self, event: CacheEvent<K, V>);
}

pub struct BackfillObserver<K, V> {
    memory_writer: Arc<dyn CacheWriter<K, V>>,
    disk_reader: Arc<dyn CacheReader<K>>,
}
```

### 3.3 Core Logic & Algorithms

- **Redb read**: begin read txn, lookup key, validate rkyv archived data once, wrap in guard.
- **Redb write**: begin write txn, encode key, reserve space, serialize into buffer, write timestamp in same txn, commit.
- **Moka read**: store `Arc<(Timestamp, V)>` as the cache value so `moka::sync::Cache::get` returns
    a cheap clone of that `Arc` (moka `get` always returns an owned clone of the stored value).
- **Moka write**: clone key/value into `Arc<(Timestamp, V)>` and insert into sync cache.
- **keys()**: redb iterator owns read txn; moka iterates entries and collects/clones keys into a
    `Vec<K>` before returning an iterator.
- **keys_where(prefix)**: redb uses range bounds `[prefix, next_prefix)`; moka filters in-memory keys.

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Sync traits only
- **Context**: backends are sync (moka::sync, redb) and async adds overhead.
- **Choice**: pure sync traits.
- **Alternatives**: async traits with spawn_blocking wrappers (rejected: adds scheduling overhead;
    keep async at the boundary via an explicit adapter).

#### Decision: Guard-based reads
- **Context**: zero-copy is required for redb + rkyv.
- **Choice**: guards that deref to a view type.
- **Alternatives**: owned values (rejected: allocations and copies).

#### Decision: Iterator-based keys_where
- **Context**: redb iterators are transaction-scoped.
- **Choice**: transaction-owned iterators.
- **Alternatives**: async streams (rejected: invalid without owning txn, adds async overhead).

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

- Cache hit/miss tracing at debug level.
- Backfill event metrics (queued, dropped, latency).
- Validation errors surfaced with cache corruption diagnostics.

### 5.2 Migration Strategy

- Introduce new sync traits alongside existing APIs (temporary adapter layer).
- Gradually switch callers to guard-based reads.
- Deprecate old async traits after parity tests.

### 5.3 Security & Privacy

- Validate all rkyv data before access.
- No raw byte access for values outside codec layer.
- Avoid logging sensitive content in backfill or cache events.

## 6. Pre-Mortem (The "Inversion")

- **Risk**: Incorrect iterator lifetimes lead to use-after-free.
  - *Mitigation*: iterators own redb read transactions; tests for iterator validity.
- **Risk**: Unaligned data causes invalid reads.
  - *Mitigation*: enable rkyv `unaligned` + field annotations.
- **Risk**: Timestamp semantics break after restart.
  - *Mitigation*: use UNIX_EPOCH persisted timestamps; TTL tests across restarts.

## 7. Critique & Refinement Log

| Date       | Critique / Issue                             | Resolution                                              |
| :--------- | :------------------------------------------- | :------------------------------------------------------ |
| 2026-01-30 | Async traits add overhead, block hot path     | Switched to pure sync traits                            |
| 2026-01-30 | Redb streaming invalid without owning txn    | Iterator-based API owns read transaction                |
| 2026-01-30 | Guard types mismatch between backends        | Coordinator returns a tagged union; callers branch by layer |
