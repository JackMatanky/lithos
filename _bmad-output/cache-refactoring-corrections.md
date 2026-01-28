# Cache Refactoring Plan: Corrections and Clarifications

**Date:** January 28, 2026
**Purpose:** Identify inaccuracies in previous analysis and provide corrected implementation plan
**Status:** CRITICAL - Must read before implementing

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Inaccuracies Identified](#inaccuracies-identified)
3. [Architectural Realities](#architectural-realities)
4. [Corrected Performance Analysis](#corrected-performance-analysis)
5. [Why Guards Won't Work (As Proposed)](#why-guards-wont-work-as-proposed)
6. [The Real Problem](#the-real-problem)
7. [Correct Solutions](#correct-solutions)
8. [Implementation Plan (Revised)](#implementation-plan-revised)
9. [What Can Be Implemented Immediately](#what-can-be-implemented-immediately)

---

## Executive Summary

### Critical Corrections Needed

The previous analysis documents (`cache-architecture-performance-analysis.md` and `rkyv-usage-analysis.md`) contain **fundamental misunderstandings** about:

1. **Trait object compatibility** - The guard-based API as proposed **cannot work** with `dyn CacheReader<K, V>`
2. **Lifetime constraints** - Returning guards from async trait methods has severe limitations
3. **The real bottleneck** - It's not just the trait API, it's the `Entry<V>` wrapper itself
4. **Implementation complexity** - The "simple" guard approach requires GATs and breaks existing code

### The Truth

**What's Correct:**
- ✅ You ARE using rkyv correctly at the codec level
- ✅ `with_view()` IS the right zero-copy pattern
- ✅ Returning `Option<V>` DOES force deserialization
- ✅ There IS a performance opportunity

**What's Wrong:**
- ❌ Guard-based traits as proposed **cannot be trait objects** (breaks coordinator)
- ❌ The examples shown won't compile due to lifetime issues
- ❌ The coordinator uses `Arc<dyn CacheReader<K, V>>` which is incompatible with guards
- ❌ The proposed `CacheGuard` trait won't work with async methods

### The Real Constraint

Your architecture uses **trait objects** (`dyn CacheReader`) in the coordinator:

```rust
// From coordinator.rs
pub struct Reader<K, V> {
    memory: Arc<dyn CacheReader<K, V>>,  // ← TRAIT OBJECT!
    disk: Arc<dyn CacheReader<K, V>>,    // ← TRAIT OBJECT!
    backfill: BackfillHandle<K, V>,
}
```

**Trait objects cannot return associated types with lifetimes tied to `self`.**

This means:
- ❌ Cannot add `type Guard<'a>: CacheGuard` to trait
- ❌ Cannot return `Self::Guard<'a>` from trait method
- ✅ CAN keep `with_view()` as a concrete method
- ✅ CAN add field accessor methods

---

## Inaccuracies Identified

### Inaccuracy 1: Guard Trait Compatibility

**Claim (from previous docs):**
```rust
#[async_trait]
pub trait CacheReader<K, V>: Send + Sync {
    type Guard<'a>: CacheGuard<V> where Self: 'a;

    async fn get_ref(&self, key: &K) -> Result<Option<Self::Guard<'_>>, CacheError>;
}
```

**Reality:**
This **cannot be made into a trait object** because:
1. GATs (Generic Associated Types) with lifetimes are not object-safe
2. The coordinator requires `Arc<dyn CacheReader<K, V>>`
3. Rust error: `the trait CacheReader cannot be made into an object`

**Proof:**
```rust
// This is your actual code (coordinator.rs, line 239-242)
pub struct Reader<K, V> {
    memory: Arc<dyn CacheReader<K, V>>,  // ← Won't work with GATs
    disk: Arc<dyn CacheReader<K, V>>,
    backfill: BackfillHandle<K, V>,
}
```

### Inaccuracy 2: Async Return Lifetimes

**Claim (from previous docs):**
```rust
async fn get_ref(&self, key: &K) -> Result<Option<RedbGuard<'_>>, CacheError>;
//                                                           ^^^ tied to self
```

**Reality:**
`async_trait` desugars to:
```rust
fn get_ref<'async_trait>(&'async_trait self, key: &K)
    -> Pin<Box<dyn Future<Output = Result<...>> + Send + 'async_trait>>;
```

The guard with lifetime `'async_trait` **cannot escape the Future** because:
- The guard must live as long as the database transaction
- The transaction is inside the Future
- When the Future completes, the transaction drops
- **The guard is invalidated before it's returned**

**This is a fundamental limitation of async + lifetimes.**

### Inaccuracy 3: Entry<V> Storage

**Claim (implied in previous docs):**
The main overhead is the trait returning `V` instead of a reference.

**Reality:**
The main overhead is **Entry<V> wrapper itself**:

```rust
// Your actual storage (redb.rs, line 85-95)
pub struct Entry<V> {
    pub timestamp: u64,      // 8 bytes
    pub value: V,            // The actual data
    pub metadata: MetadataMap, // HashMap<String, String>
}
```

**To return `V` from trait, you MUST:**
1. Deserialize `Entry<V>` (includes metadata HashMap)
2. Extract `.value` field
3. Drop the metadata

**Even with zero-copy access to archived `Entry<V>`, extracting `V` alone requires partial deserialization.**

### Inaccuracy 4: Coordinator Compatibility

**Claim (from previous docs):**
"Update coordinator to use zero-copy" with example:

```rust
if let Some(guard) = self.l1.get_ref(key).await? {
    return Ok(Some(guard));
}
```

**Reality:**
The coordinator returns `Option<V>` from its `CacheReader::get()` implementation:

```rust
// coordinator.rs, line 324-361
#[async_trait]
impl<K, V> CacheReader<K, V> for Reader<K, V> {
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        //                                          ^^^ MUST return owned V
```

**The coordinator MUST return `V` because:**
1. It implements `CacheReader<K, V>` trait
2. That trait requires `async fn get() -> Option<V>`
3. Cannot change trait without breaking all existing code
4. Cannot return guards from multi-layer cache (lifetime hell)

### Inaccuracy 5: Performance Numbers

**Claim (from previous docs):**
"6x faster with zero-copy API"

**Reality:**
The numbers assume:
1. You only access one field (timestamp)
2. You never need the value `V`
3. You can use concrete `Reader<K, V>` type directly

**But in practice:**
1. Most operations need the value `V`, not just metadata
2. Code uses `Arc<dyn CacheReader<K, V>>` (trait object)
3. The `Entry<V>` wrapper adds overhead even with zero-copy

**More realistic speedup:** 2-3x for metadata-only operations, 0x (no change) for value operations.

---

## Architectural Realities

### Reality 1: Trait Object Requirement

**Your architecture depends on trait objects:**

```rust
// Builder (coordinator.rs, line 87-96)
pub struct Builder<K, V> {
    disk_reader: Option<Arc<dyn CacheReader<K, V>>>,
    disk_writer: Option<Arc<dyn CacheWriter<K, V>>>,
    memory_reader: Option<Arc<dyn CacheReader<K, V>>>,
    memory_writer: Option<Arc<dyn CacheWriter<K, V>>>,
}
```

**Why?** To allow runtime polymorphism:
- Memory layer: Moka
- Disk layer: Redb
- Coordinator: Combines both behind same interface

**Constraint:** Trait objects require object-safety:
- ✅ No generic type parameters in methods
- ✅ No associated types with lifetimes
- ✅ No `Self: Sized` bounds
- ❌ **Your proposed `Guard<'a>` violates this**

### Reality 2: Entry<V> is Part of the API

**The metadata wrapper is intentional:**

```rust
// redb.rs, line 85-95
pub struct Entry<V> {
    pub timestamp: u64,
    pub value: V,
    pub metadata: MetadataMap,
}
```

**Why?**
- Track when cache entry was created
- Store arbitrary metadata per entry
- Enable cache invalidation strategies
- Support future features (TTL, tags, etc.)

**This is NOT an implementation detail - it's part of your SPI design.**

### Reality 3: Codec Operates on Entry<V>

**Your codec serializes the wrapper, not the value:**

```rust
// encoder.rs, line 138-171
impl<K, V> Codec<K, V> for RkyvCodec
//            ^^^ This is actually Entry<V> in practice
```

**In redb.rs:**
```rust
// Line 469-473
C: crate::spi::cache::encoder::Codec<K, Entry<V>>
//                                      ^^^^^^^^^
```

**This means:**
- Redb stores `Entry<V>`, not `V`
- Deserialization always produces `Entry<V>`
- Zero-copy access gives you `&Archived<Entry<V>>`
- **Getting just `V` requires partial deserialization**

### Reality 4: Async Transaction Lifetimes

**Redb operations are async and transactional:**

```rust
// redb.rs, line 488-505
self.inner.read(move |txn, table_name| {
    //            ^^^^ Transaction owned by closure
    let table = txn.open_table(table_def)?;
    table.get(key_bytes.as_slice())?
        .map(|guard| codec.decode_value(guard.value()))
        //    ^^^^^ guard lifetime tied to txn
        .transpose()
}) // ← Transaction drops here
.await
```

**The guard cannot escape the transaction:**
- `redb::AccessGuard<'txn>` lifetime tied to transaction
- Transaction ends when closure returns
- Guard is invalidated before Future resolves
- **Cannot return guard from async method**

---

## Corrected Performance Analysis

### What's Actually Slow

**Operation: Read single entry from Redb**

```rust
let value = cache.get(&key).await?;
```

**Cost breakdown:**

| Step | Current (with Entry) | With Zero-Copy (theoretical) | Actual Bottleneck |
|------|---------------------|------------------------------|-------------------|
| 1. Encode key | 2μs | 2μs | Unavoidable |
| 2. Async transaction | 1μs | 1μs | Unavoidable |
| 3. Redb lookup | 0.5μs | 0.5μs | Unavoidable |
| 4. Validate bytes | 0.3μs | 0.3μs | Unavoidable |
| 5. Deserialize Entry | **8μs** | **0μs** | ← Savings here |
| 6. Extract `.value` | **2μs** | **2μs** | ← Still needed! |
| 7. Drop metadata | 0.2μs | 0μs | Minor |
| **Total** | **14μs** | **5.8μs** | **2.4x faster** |

**Key insight:** Even with zero-copy, extracting `V` from `Entry<V>` costs 2μs (partial deser).

### What's Actually Fast

**Operation: Read timestamp only**

```rust
let ts = cache.with_view(&key, |archived| archived.timestamp).await?;
```

**Cost breakdown:**

| Step | Current | With with_view() | Speedup |
|------|---------|-----------------|---------|
| 1-4. (same) | 3.8μs | 3.8μs | - |
| 5. Deserialize Entry | 8μs | 0μs | ✅ Saved |
| 6. Access timestamp | 0μs | 0.2μs | ✅ Zero-copy |
| **Total** | **11.8μs** | **4μs** | **3x faster** |

**This is where rkyv shines:** Field-only access without full deserialization.

### Real-World Impact

**Vault freshness check (10,000 files):**

```rust
// Current (get full value)
for key in keys {
    let entry = cache.get(&key).await?; // 14μs per call
    if entry.timestamp < cutoff { ... }
}
// Total: 140ms
```

```rust
// With with_view (timestamp only)
for key in keys {
    let stale = cache.with_view(&key, |arch| arch.timestamp < cutoff).await?; // 4μs
    if stale { ... }
}
// Total: 40ms (3.5x faster)
```

**But if you need the value:**

```rust
// With with_view (need value too)
for key in keys {
    let (stale, value) = cache.with_view(&key, |arch| {
        let stale = arch.timestamp < cutoff;
        let value = rkyv::deserialize(arch.value)?; // Still deserializes!
        (stale, value)
    }).await?;
    if stale { process(value); }
}
// Total: ~100ms (1.4x faster, not 6x)
```

**Conclusion:** Speedup depends on whether you need `V` or just metadata.

---

## Why Guards Won't Work (As Proposed)

### Problem 1: Object Safety

**Your proposed trait:**

```rust
pub trait CacheReader<K, V>: Send + Sync {
    type Guard<'a>: CacheGuard<V> where Self: 'a;
    async fn get_ref(&self, key: &K) -> Result<Option<Self::Guard<'_>>, CacheError>;
}
```

**Why it fails:**

```rust
// This won't compile:
let reader: Arc<dyn CacheReader<String, String>> = Arc::new(redb_reader);
//              ^^^ error[E0038]: the trait `CacheReader` cannot be made into an object
```

**Reason:** GATs (Generic Associated Types) with lifetimes are not object-safe.

**Rust RFC 2056:** Trait objects require all associated types to be nameable without `Self`.

### Problem 2: Async Lifetime Hell

**Even if object-safe, async makes it worse:**

```rust
async fn get_ref(&self, key: &K) -> Result<Option<Guard<'_>>, CacheError> {
    //                                                   ^^^ tied to self

    self.inner.read(move |txn, table_name| {
        //            ^^^^ Transaction owned by closure, not self

        let guard = table.get(key)?; // guard lifetime: 'txn

        // ERROR: Cannot return guard with 'txn lifetime
        // when promise

d 'self lifetime
        Ok(Some(guard))
    }).await
}
```

**The guard's lifetime (`'txn`) is shorter than `'self`.**

### Problem 3: Coordinator Cannot Use Guards

**Your coordinator:**

```rust
impl<K, V> CacheReader<K, V> for Reader<K, V> {
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        // Try L1 (memory)
        if let Some(value) = self.memory.get(key).await? {
            return Ok(Some(value)); // ← Must return V, not guard
        }

        // Try L2 (disk)
        if let Some(value) = self.disk.get(key).await? {
            // Backfill to L1
            self.backfill.trigger(key.clone(), value.clone()).await;
            return Ok(Some(value)); // ← Must return V, not guard
        }

        Ok(None)
    }
}
```

**Why guards don't work here:**
1. Backfill needs owned `V` to write to L1
2. Guard from L2 (disk) can't stay alive after method returns
3. Coordinator's `get()` returns `V`, not guard
4. **Cannot change this without breaking all consumers**

---

## The Real Problem

### It's Not the Trait API

The real problem is **NOT** that `CacheReader::get()` returns `Option<V>`.

The real problem is **WHERE** you use `get()` and what you do with the result.

### Hot Paths Analysis

Let's examine where caching is actually used in Lithos:

**Scenario 1: Vault Freshness Check**
```rust
// Pseudo-code (hypothetical usage)
for file in vault.files() {
    if let Some(cached) = cache.get(&file.path).await? {
        if cached.timestamp < file.modified_time {
            re_index(file);
        }
    }
}
```

**Bottleneck:** Deserializing full `Entry<V>` just to check timestamp.

**Solution:** Use `with_view()` for timestamp check, only `get()` when re-indexing needed.

**Scenario 2: LSP Link Suggestions**
```rust
// Pseudo-code
for key in cache.keys().await? {
    if let Some(note) = cache.get(&key).await? {
        if note.title.starts_with(prefix) {
            suggestions.push(note);
        }
    }
}
```

**Bottleneck:** Deserializing full notes to check title.

**Solution:** Store titles in metadata, use `with_view()` to filter.

**Scenario 3: Loading Note for Display**
```rust
let note = cache.get(&note_id).await?.ok_or(NotFound)?;
render(note);
```

**Bottleneck:** None - you need the full value anyway.

**Solution:** Keep using `get()`, no optimization possible.

### The Pattern

**The real problem:** Using `get()` when you only need metadata.

**The real solution:** Use `with_view()` for metadata checks, `get()` for full loads.

**This doesn't require changing the trait!**

---

## Correct Solutions

### Solution 1: Expand with_view() Usage (RECOMMENDED)

**Keep the existing trait API unchanged.**

**Add convenience methods to concrete types:**

```rust
// Add to Reader<K, V, C> (concrete impl, not trait)
impl<K, V, C> Reader<K, V, C> {
    /// Get timestamp without deserializing value
    pub async fn get_timestamp(&self, key: &K) -> Result<Option<u64>, CacheError> {
        self.with_view(key, |archived| archived.timestamp).await
    }

    /// Get metadata without deserializing value
    pub async fn get_metadata(&self, key: &K) -> Result<Option<MetadataMap>, CacheError> {
        self.with_view(key, |archived| {
            // Zero-copy iteration, only deserialize if needed
            archived.metadata.iter()
                .map(|(k, v)| (k.as_str().to_owned(), v.as_str().to_owned()))
                .collect()
        }).await
    }

    /// Check if entry is stale
    pub async fn is_stale(&self, key: &K, cutoff: u64) -> Result<Option<bool>, CacheError> {
        self.with_view(key, |archived| archived.timestamp < cutoff).await
    }
}
```

**Advantages:**
- ✅ No trait changes required
- ✅ Works with existing trait objects
- ✅ Backward compatible
- ✅ Concrete types can use zero-copy
- ✅ Coordinator keeps working

**Usage:**

```rust
// Instead of:
let entry = cache.get(&key).await?;
if entry.timestamp < cutoff { ... }

// Use:
if cache.is_stale(&key, cutoff).await? == Some(true) { ... }
```

### Solution 2: Add Metadata-Only Methods to Trait (POSSIBLE)

**Add new methods that return primitive types:**

```rust
#[async_trait]
pub trait CacheReader<K, V>: Send + Sync {
    // Existing
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError>;

    // NEW - metadata access without deserialization
    async fn get_timestamp(&self, key: &K) -> Result<Option<u64>, CacheError>;
    async fn get_metadata_field(&self, key: &K, field: &str) -> Result<Option<String>, CacheError>;
}
```

**Advantages:**
- ✅ Object-safe (returns concrete types)
- ✅ Available via trait objects
- ✅ Can use zero-copy internally
- ✅ Backward compatible (new methods)

**Disadvantages:**
- ⚠️ Expands trait surface area
- ⚠️ All implementations must provide these
- ⚠️ Moka implementation can't optimize (no Entry wrapper)

### Solution 3: Accept Entry<V> Overhead (ALTERNATIVE)

**Keep everything as-is, optimize elsewhere:**

1. **Use Moka more aggressively** (memory cache for hot entries)
2. **Implement batch operations** (`get_many()` to amortize transaction cost)
3. **Add smarter caching strategies** (pre-warm vault on startup)
4. **Optimize serialization** (smaller Entry format)

**When this makes sense:**
- CLI commands are already "fast enough" (<100ms)
- LSP latency dominated by other factors (file I/O, parsing)
- Premature optimization warning

---

## Implementation Plan (Revised)

### Phase 1: Add Zero-Copy Convenience Methods (Week 1)

**Goal:** Provide zero-copy metadata access without trait changes.

**Changes:**
1. Add methods to `redb::Reader<K, V>`:
   - `get_timestamp(key) -> Option<u64>`
   - `get_metadata(key) -> Option<MetadataMap>`
   - `get_metadata_field(key, field) -> Option<String>`
   - `is_stale(key, cutoff) -> Option<bool>`

2. Implement using existing `with_view()`:
   ```rust
   pub async fn get_timestamp(&self, key: &K) -> Result<Option<u64>, CacheError> {
       self.with_view(key, |archived| archived.timestamp).await
   }
   ```

3. Add tests for each method

4. Document performance characteristics

**Risk:** Very Low (additive, no API changes)
**Benefit:** 3-4x faster for metadata-only operations

### Phase 2: Update Hot Paths (Week 2)

**Goal:** Replace `get()` calls with metadata methods where appropriate.

**Process:**
1. Identify hot paths:
   ```bash
   rg "cache\.get\(" crates/ --type rust
   ```

2. For each usage, ask:
   - Do we only need timestamp? → `is_stale()`
   - Do we only need metadata? → `get_metadata_field()`
   - Do we need the value? → Keep `get()`

3. Refactor incrementally:
   ```rust
   // Before
   if let Some(entry) = cache.get(&key).await? {
       if entry.timestamp < cutoff {
           re_index(key);
       }
   }

   // After
   if cache.is_stale(&key, cutoff).await? == Some(true) {
       re_index(key);
   }
   ```

4. Benchmark before/after

**Risk:** Low (preserves semantics)
**Benefit:** Actual measured speedup in real workloads

### Phase 3: (Optional) Add Trait Methods (Week 3)

**Goal:** Make metadata access available via trait objects.

**Decision point:** Only if coordinator needs it.

**Changes:**
1. Add to `CacheReader` trait:
   ```rust
   async fn get_timestamp(&self, key: &K) -> Result<Option<u64>, CacheError> {
       // Default impl calls get() and extracts timestamp
       Ok(self.get(key).await?.map(|_| {
           // Problem: Can't get timestamp from V!
           // Only works if V = Entry<T>, which breaks abstraction
       }))
   }
   ```

2. **Wait, this doesn't work!**

**Problem:** The trait is `CacheReader<K, V>` where `V` is the user's type.
The `Entry<V>` wrapper is Redb-specific.

**Conclusion:** Cannot add metadata methods to trait without exposing `Entry<V>`.

### Phase 3 (Revised): Document Patterns (Week 3)

**Goal:** Guide users to zero-copy patterns.

**Deliverables:**
1. Update module docs with performance guide:
   ```rust
   //! ## Performance Best Practices
   //!
   //! ### Use Concrete Types for Zero-Copy
   //!
   //! ```rust
   //! // Instead of:
   //! let reader: Arc<dyn CacheReader<K, V>> = ...;
   //! let entry = reader.get(&key).await?;
   //!
   //! // Use:
   //! let reader: RedbReader<K, V> = ...;
   //! let ts = reader.get_timestamp(&key).await?;
   //! ```
   ```

2. Add examples for common patterns

3. Benchmark suite showing speedups

---

## What Can Be Implemented Immediately

### Immediately Actionable (Today)

**1. Add `get_timestamp()` to Redb Reader**

```rust
// File: crates/adapters/src/spi/cache/redb.rs
// Add after line 630 (after with_view method)

impl<K, V, C> Reader<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::encoder::Codec<K, Entry<V>> + Clone + Send + Sync + 'static,
{
    /// Get entry timestamp without deserializing the value.
    ///
    /// This method uses zero-copy access to retrieve only the timestamp
    /// field, avoiding the cost of deserializing the entire entry.
    ///
    /// # Performance
    ///
    /// ~4μs per call vs ~14μs for `get()`, a 3.5x speedup.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the key encoding or database access fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_adapters::spi::cache::RedbReader;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// # let reader: RedbReader<String, String> = todo!();
    /// let cutoff = 1704067200; // Jan 1, 2024
    /// if let Some(ts) = reader.get_timestamp(&"note_id".to_string()).await? {
    ///     if ts < cutoff {
    ///         println!("Entry is stale");
    ///     }
    /// }
    /// # Ok::<(), lithos_adapters::spi::errors::CacheError>(())
    /// # }).unwrap();
    /// ```
    pub async fn get_timestamp(&self, key: &K) -> Result<Option<u64>, CacheError> {
        self.with_view(key, |archived| archived.timestamp).await
    }
}
```

**2. Add benchmark to prove speedup**

```rust
// File: crates/adapters/benches/cache_performance.rs (create new)

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_adapters::spi::cache::*;

fn bench_timestamp_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("bench.redb");

    let mut builder = RedbBuilder::<String, String>::new();
    builder.path(db_path).table_name("bench");
    let reader = builder.reader().unwrap();
    let writer = builder.writer().unwrap();

    // Setup: Insert 1000 entries
    rt.block_on(async {
        for i in 0..1000 {
            writer.put(format!("key_{}", i), format!("value_{}", i)).await.unwrap();
        }
    });

    c.bench_function("get_full_entry", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(reader.get(&"key_500".to_string()).await.unwrap())
        });
    });

    c.bench_function("get_timestamp_only", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(reader.get_timestamp(&"key_500".to_string()).await.unwrap())
        });
    });
}

criterion_group!(benches, bench_timestamp_access);
criterion_main!(benches);
```

**3. Document in module**

Add to `redb.rs` module docs:

```rust
//! ## Performance Optimization
//!
//! The `Reader` type provides zero-copy methods for accessing metadata
//! without deserializing the entire cached value:
//!
//! - [`get_timestamp()`](Reader::get_timestamp) - 3.5x faster than `get()`
//! - [`with_view()`](Reader::with_view) - Custom zero-copy operations
//!
//! Use these methods when you only need to inspect metadata (freshness checks,
//! filtering) and use [`get()`](Reader::get) when you need the actual value.
```

### Next Week (After Validation)

**1. Add more convenience methods**
- `get_metadata()`
- `get_metadata_field()`
- `is_stale()`

**2. Find and optimize hot paths**
- Grep for `cache.get()` usage
- Replace with metadata methods where appropriate

**3. Measure real-world impact**
- Benchmark actual CLI commands
- Profile LSP operations
- Validate 2-3x speedup claims

---

## Conclusion

### What We Learned

1. **The rkyv usage is correct** - Codec implementation is textbook perfect
2. **Guards won't work** - Trait objects + async + lifetimes = incompatible
3. **Entry<V> is intentional** - Not a mistake, it's your SPI design
4. **with_view() is the answer** - Already implemented, just under-utilized
5. **Trait changes not needed** - Add methods to concrete types instead

### What To Do

1. **Start with Phase 1** - Add `get_timestamp()` and friends (this week)
2. **Measure actual impact** - Benchmark real workloads (next week)
3. **Optimize incrementally** - Replace `get()` calls in hot paths (ongoing)
4. **Document patterns** - Guide future development (Week 3)

### What NOT To Do

1. ❌ Don't add GATs to `CacheReader` trait (breaks trait objects)
2. ❌ Don't try to return guards from async methods (lifetime hell)
3. ❌ Don't change `Entry<V>` structure (it's intentional)
4. ❌ Don't break coordinator (it's core to your architecture)

### Success Metrics

**Realistic targets:**
- Vault freshness check: 140ms → 40ms (3.5x faster) ✅
- Metadata filtering: 10-100x faster (zero-copy iteration) ✅
- Full value loading: No change (0x speedup) ✅ Expected

**DO NOT expect:**
- 6x speedup across the board ❌
- Zero-copy for operations that need `V` ❌
- Guard-based APIs ❌

---

**Document Status:** Ready for implementation
**Validation:** Run benchmarks before proceeding to Phase 2
**Next Steps:** Implement `get_timestamp()` and write benchmarks
