# redb - Reference Documentation

**Version:** 3.1.0
**Official Docs:** https://docs.rs/redb/latest/redb/
**Design Doc:** https://github.com/cberner/redb/blob/master/docs/design.md
**Repository:** https://github.com/cberner/redb
**License:** MIT OR Apache-2.0

## Overview

redb (Rust Embedded DataBase) is a simple, portable, high-performance, ACID, embedded key-value store written in pure Rust. It provides zero-copy, thread-safe, BTreeMap-based API with full ACID compliance.

redb access is transaction-scoped. Tables are opened within a read or write transaction, and guards returned by `get()` are tied to that transaction's lifetime.
See https://docs.rs/redb/latest/redb/trait.ReadableDatabase.html#tymethod.begin_read and https://docs.rs/redb/latest/redb/struct.ReadTransaction.html#method.open_table.

## Core Features for Zero-Copy & High Performance

### 1. Zero-Copy Architecture

#### [Value](https://docs.rs/redb/latest/redb/trait.Value.html) Trait - Direct Memory Access

```rust
pub trait Value: Debug {
    type SelfType<'a>: Debug + 'a where Self: 'a;
    type AsBytes<'a>: AsRef<[u8]> + 'a where Self: 'a;

    fn fixed_width() -> Option<usize>;
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>;
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>;
    fn type_name() -> TypeName;
}
```

**Zero-Copy Benefits:**

- `from_bytes` returns views over data without copying
- Lifetime-based API ensures memory safety
- Direct byte slice interpretation for primitives
- No serialization overhead for reads

**Coherence Note:** You cannot implement `redb::Value` for standard library types like `String` or `Path` in a downstream crate due to Rust's [orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules). Use a local newtype wrapper if you need a custom `Value` implementation.

**Supported Types (Fixed Width - Optimal Performance):**

- All primitive integers: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- Floating point: `f32`, `f64`
- `bool`, `char`
- Arrays: `[T; N]` where `T: Value`
- Tuples up to 12 elements (if all fixed width)

**Variable Width Types:**

- `&[u8]` - byte slices (zero-copy view)
- `&str` - string slices (zero-copy view)
- `String`, `Vec<T>`, `Option<T>`

### 2. Performance-Critical Features

#### [`AccessGuard`](https://docs.rs/redb/latest/redb/struct.AccessGuard.html) - Zero-Copy Value Access

```rust
pub struct AccessGuard<'a, V: Value + 'static> {
    // Provides zero-copy access to values
    // Returns V::SelfType<'a> which is often a reference
}

impl<'a, V: Value> AccessGuard<'a, V> {
    pub fn value(&self) -> V::SelfType<'_>;
}
```

**Performance Characteristics:**

- Direct memory mapping to database pages
- No deserialization required for reads
- Lifetime-bound safety without runtime overhead
- Lock-free reads via MVCC

**Lifetime Note:** `AccessGuard` borrows from the transaction. Guards must not outlive the transaction or the table they were created from.
See https://docs.rs/redb/latest/redb/struct.AccessGuard.html and https://docs.rs/redb/latest/redb/trait.ReadableTable.html#tymethod.get.

#### [`MutInPlaceValue`](https://docs.rs/redb/latest/redb/trait.MutInPlaceValue.html) Trait - In-Place Mutations

```rust
pub trait MutInPlaceValue: Value {
    // Enables zero-allocation updates
    // Value must be safely mutable as &mut [u8]
}
```

**Use Cases:**

- Updating counters without read-modify-write cycle
- Modifying fixed-size records in place
- Avoiding allocation during updates

**Safety Note:** Only fixed-width, in-place-safe types should implement `MutInPlaceValue`.

### 3. MVCC (Multi-Version Concurrency Control)

**Concurrency Model:**

- Multiple concurrent readers without blocking
- Single writer can proceed without blocking readers
- Readers see consistent snapshot
- No lock contention for read operations
See https://github.com/cberner/redb/blob/master/docs/design.md for MVCC design details.

**Performance Implications:**

- Read throughput scales with CPU cores
- Writers don't block readers
- Readers don't block writers
- Suitable for read-heavy workloads

### 4. Copy-on-Write B-Trees

**Architecture:**

- Persistent B-tree structure
- Pages shared between transactions
- Only modified pages are copied
- Efficient bulk operations

**Performance Benefits:**

- Excellent cache locality
- Sequential disk I/O patterns
- Minimal write amplification
- Efficient range scans

### 5. Memory-Mapped I/O

**Database Backend:**

```rust
pub trait StorageBackend: Send + Sync {
    // redb uses memory mapping by default
    // Zero-copy between kernel and userspace
}
```

**Advantages:**

- OS manages page cache automatically
- No explicit buffer management needed
- Direct memory access to database pages
- Reduced memory copies

### 6. Table Operations - Performance APIs

#### [`ReadableTable`](https://docs.rs/redb/latest/redb/trait.ReadableTable.html) Trait

```rust
pub trait ReadableTable<K: Key + 'static, V: Value + 'static> {
    fn get<'a>(&self, key: &'a K::SelfType<'a>)
        -> Result<Option<AccessGuard<V>>>;

    fn range<'a, KR>(&self, range: KR)
        -> Result<Range<K, V>>;

    fn len(&self) -> Result<u64>;
    fn is_empty(&self) -> Result<bool>;
}
```

**Zero-Copy Read Patterns:**

```rust
// Direct access without copying
let value = table.get("key")?.unwrap().value();

// Efficient range iteration
for entry in table.range("start".."end")? {
    let (key, value) = entry?;
    // Both key and value are zero-copy views
}
```
See https://docs.rs/redb/latest/redb/trait.ReadableTable.html#tymethod.range for range iteration details.

#### Table Modifications

```rust
pub struct Table<'db, K: Key + 'static, V: Value + 'static> {
    fn insert<'a>(&mut self, key: &K::SelfType<'a>, value: &V::SelfType<'a>)
        -> Result<Option<AccessGuard<V>>>;

    fn insert_reserve<'a>(&mut self, key: &K::SelfType<'a>, size: u32)
        -> Result<AccessGuardMut<V>>;

    fn remove<'a>(&mut self, key: &K::SelfType<'a>)
        -> Result<Option<AccessGuard<V>>>;
}
```
See https://docs.rs/redb/latest/redb/struct.Table.html#method.insert, https://docs.rs/redb/latest/redb/struct.Table.html#method.remove, and https://docs.rs/redb/latest/redb/struct.Table.html#method.insert_reserve.

**`insert_reserve` for Zero-Allocation Writes:**

```rust
// Reserve space and write directly to it
let mut guard = table.insert_reserve("key", data.len() as u32)?;
guard.as_mut().copy_from_slice(data);
// No intermediate allocation needed
```

**Size Note:** `insert_reserve` takes a `u32` size. Callers should validate that serialized values fit within this limit before casting.

### 7. [Multimap Tables](https://docs.rs/redb/latest/redb/struct.MultimapTable.html)

**High-Performance Duplicate Keys:**

```rust
pub struct MultimapTable<'db, K: Key + 'static, V: Key + 'static> {
    fn insert<'a>(&mut self, key: &K::SelfType<'a>, value: &V::SelfType<'a>)
        -> Result<()>;

    fn get<'a>(&self, key: &'a K::SelfType<'a>)
        -> Result<MultimapValue<'a, K, V>>;
}
```

**Use Cases:**

- Secondary indexes
- Many-to-many relationships
- Event logs with duplicate timestamps

**Cache Use Case:** Multimap tables can model reverse indexes (e.g., tag -> file list) without full scans.

### 8. Savepoints and Rollbacks

**Transaction Management:**

```rust
pub struct WriteTransaction {
    fn set_savepoint(&mut self) -> Result<Savepoint>;
    fn rollback_to_savepoint(&mut self, savepoint: Savepoint) -> Result<()>;
    fn commit(self) -> Result<()>;
}
```

**Performance Considerations:**

- Savepoints are lightweight
- Rollback is O(1) in most cases
- Enable atomic multi-step operations
- No performance penalty if not used

**Use Case:** Savepoints are helpful when batching writes and retrying partial failures without restarting the whole transaction.

### 9. Database Configuration

#### Builder Pattern - Performance Tuning

```rust
pub struct Builder {
    fn set_cache_size(&mut self, bytes: usize) -> &mut Self;
    fn set_page_size(&mut self, size: usize) -> &mut Self;
    fn set_region_size(&mut self, size: u64) -> &mut Self;

    fn create(self, path: impl AsRef<Path>)
        -> Result<Database, DatabaseError>;
}
```
See https://docs.rs/redb/latest/redb/struct.Builder.html for builder configuration options.

**Cache Size:**

- Default: OS manages via mmap
- Explicit cache: Faster repeated access
- Trade-off: Memory usage vs. speed

**Page Size:**

- Default: 4KB (OS page size)
- Larger pages: Better for large values
- Smaller pages: Better for small values
- Must match OS page size for optimal mmap

**Guidance:** Align page size to the OS page size to maximize mmap efficiency.

### 10. Durability Modes

```rust
pub enum Durability {
    None,      // Fastest, no crash safety
    Eventual,  // Background fsync
    Immediate, // Immediate fsync (slowest, safest)
}

impl WriteTransaction {
    fn set_durability(&mut self, durability: Durability)
        -> Result<(), SetDurabilityError>;
}
```
See https://docs.rs/redb/latest/redb/enum.Durability.html for durability definitions.

**Performance Trade-offs:**

- `None`: Maximum write throughput, data may be lost on crash
- `Eventual`: Good throughput, minimal data loss window
- `Immediate`: ACID guarantees, lower write throughput

**Cache Guidance:** For cache data that can be rebuilt, `None` or `Eventual` are usually appropriate.

## Integration with Lithos System

### Recommended Use Cases

1. **Persistent State Storage**
   - Zero-copy access to ledger entries
   - Efficient range queries for history
   - MVCC for concurrent reads

2. **Indexing Layer**
   - Multimap tables for secondary indexes
   - Fast lookups with zero-copy values
   - Atomic index updates via transactions

3. **Cache Backend**
   - Persistent cache with zero-copy reads
   - TTL via custom metadata
   - MVCC for lock-free cache access

### Performance Optimization Strategies

1. **Use Fixed-Width Types When Possible**
   - Store lengths separately if needed
   - Custom encoding for variable data
   - Batch variable-width inserts

2. **Leverage In-Place Updates**
   - Implement `MutInPlaceValue` for counters
   - Update flags/status without rewrites
   - Atomic increment operations

3. **Batch Operations**
   - Group inserts in single transaction
   - Amortize fsync costs
   - Reduce lock acquisition overhead

4. **Read Optimization**
   - Use `get` for single keys
   - Use `range` for sequential access
   - Consider `ReadableTable` for read-only views

5. **Memory Management**
   - Tune cache size based on working set
   - Use appropriate page size for data
   - Monitor via `CacheStats` and `DatabaseStats`

### Benchmarking Notes

**Strengths:**

- Excellent read performance (zero-copy)
- Very fast range scans (B-tree locality)
- Good write performance with batching
- Low memory overhead

**Considerations:**

- Single writer limitation
- Mmap overhead on small databases
- No compression (pure zero-copy)
- File size growth (vacuuming needed)

**Compaction Note:** [`Database::compact()`](https://docs.rs/redb/latest/redb/struct.Database.html#method.compact) performs a full rewrite and is typically a blocking operation. Plan for maintenance windows if using it on large datasets.
See https://github.com/cberner/redb/blob/master/docs/design.md for implementation details.

## Code Examples

### Basic Zero-Copy Usage

```rust
use redb::{Database, ReadableDatabase, TableDefinition};

const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("data");

// Create database
let db = Database::create("data.redb")?;

// Write
let write_txn = db.begin_write()?;
{
    let mut table = write_txn.open_table(TABLE)?;
    table.insert("key", b"value")?;
}
write_txn.commit()?;

// Zero-copy read
let read_txn = db.begin_read()?;
let table = read_txn.open_table(TABLE)?;
let value = table.get("key")?.unwrap();
let bytes: &[u8] = value.value(); // No copy!
```

### High-Performance Batch Insert

```rust
let write_txn = db.begin_write()?;
{
    let mut table = write_txn.open_table(TABLE)?;

    // Batch insert in single transaction
    for i in 0..10000 {
        let key = format!("key_{}", i);
        table.insert(key.as_str(), &i.to_le_bytes())?;
    }
}
write_txn.commit()?; // Single fsync
```

### In-Place Update Example

```rust
// Define custom type supporting in-place mutation
#[derive(Debug)]
struct Counter(u64);

impl Value for Counter {
    type SelfType<'a> = u64;
    type AsBytes<'a> = [u8; 8];

    fn fixed_width() -> Option<usize> { Some(8) }
    fn from_bytes<'a>(data: &'a [u8]) -> u64 {
        u64::from_le_bytes(data.try_into().unwrap())
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a u64) -> [u8; 8] {
        value.to_le_bytes()
    }
    fn type_name() -> TypeName {
        TypeName::new("Counter")
    }
}

impl MutInPlaceValue for Counter {}

// Use insert_reserve for in-place update
let mut guard = table.insert_reserve("counter", 8)?;
let mut bytes = guard.as_mut();
let current = u64::from_le_bytes(bytes.try_into().unwrap());
bytes.copy_from_slice(&(current + 1).to_le_bytes());
```

## Summary for Lithos

redb provides exceptional zero-copy performance through:

- Direct memory mapping and value access
- MVCC for lock-free concurrent reads
- Copy-on-write B-trees with excellent locality
- Fixed-width type optimizations
- In-place mutation support
- Minimal serialization overhead

**Best suited for:** Persistent storage with heavy read workloads, range queries, and atomic transactions where zero-copy access is critical.
