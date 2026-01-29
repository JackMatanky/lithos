# moka - Reference Documentation

**Version:** 0.12.13
**Official Docs:** https://docs.rs/moka/0.12.13/moka/
**Repository:** https://github.com/moka-rs/moka
**License:** MIT OR Apache-2.0 AND Apache-2.0

## Overview

Moka is a fast, concurrent cache library for Rust inspired by Java's Caffeine. It provides thread-safe, highly concurrent in-memory cache implementations with near-optimal hit ratios using advanced eviction algorithms.

## Core Features for High Performance

### 1. Concurrency Architecture

#### Lock-Free Concurrent Hash Table
- **Central Storage:** Lock-free concurrent hash map
- **Strong Consistency:** Immediate visibility of insertions
- **Eventually Consistent Policy:** Cache policy structures updated in batches
- **No Lock Contention:** Lock-free reads and lock-protected batch updates

**Threading Model:**
```rust
use moka::sync::Cache;

let cache = Cache::new(10_000);

// Cheap clone - creates reference-counted pointers
let cache_clone = cache.clone();

// Share across threads - no Arc needed
std::thread::spawn(move || {
    cache_clone.get(&key);
});
```

**Performance Characteristics:**
- Full concurrency for retrievals
- High expected concurrency for updates
- No dedicated maintenance thread
- User threads perform maintenance

### 2. Advanced Eviction Policies

#### TinyLFU (Default - Recommended)
**Algorithm Components:**
1. **LFU Admission Policy:** Tracks frequency of all keys (hit + missed)
2. **LRU Eviction Policy:** Evicts least recently used entries
3. **Count-Min Sketch:** Efficient frequency estimation with minimal memory

**Flow:**
```
New Entry → LFU Filter Check → Popular? → Admit to Cache → LRU Eviction
                              ↓
                              Not Popular → Reject
```

**Advantages:**
- Excellent for mixed workloads (database, search, analytics)
- Protects against one-time bulk scans
- Very low memory overhead for frequency tracking
- Near-optimal hit ratios

**When to Use:**
- General-purpose caching
- Database query caches
- Search result caching
- Analytics data caching

#### LRU (Alternative Policy)
**Algorithm:**
- Simple Least Recently Used eviction
- No admission policy

**Advantages:**
- Simpler algorithm
- Better for recency-biased workloads
- Slightly lower overhead

**When to Use:**
- Job queues
- Event streams
- Strictly recency-based access patterns

### 3. Size-Based Eviction

#### Entry Count Based
```rust
let cache = Cache::builder()
    .max_capacity(10_000)  // Maximum 10k entries
    .build();
```

**Use Case:** Simple entry-count limits

#### Weighted Size Based
```rust
use moka::sync::Cache;

let cache = Cache::builder()
    .weigher(|_key, value: &String| -> u32 {
        value.len().try_into().unwrap_or(u32::MAX)
    })
    .max_capacity(32 * 1024 * 1024)  // 32 MiB total
    .build();
```

**Features:**
- Custom size calculation per entry
- Total weighted size limit
- Useful for memory-bound caches
- Per-entry variable sizing

**Performance Notes:**
- Weigher called on insert
- Size used for eviction decisions
- Not used for admission policy

### 4. Time-Based Expiration

#### Cache-Level Policies

**Time to Live (TTL):**
```rust
use std::time::Duration;

let cache = Cache::builder()
    .time_to_live(Duration::from_secs(30 * 60))  // 30 minutes
    .build();
```

**Time to Idle (TTI):**
```rust
let cache = Cache::builder()
    .time_to_idle(Duration::from_secs(5 * 60))   // 5 minutes
    .build();
```

**Combined:**
```rust
let cache = Cache::builder()
    .time_to_live(Duration::from_secs(30 * 60))  // Max 30 min
    .time_to_idle(Duration::from_secs(5 * 60))   // Idle 5 min
    .build();
```

**Expiration Semantics:**
- TTL: Expires after duration from `insert`
- TTI: Expires after duration from last `get` or `insert`
- Combined: Whichever comes first
- Extensions: `get` resets TTI but not TTL

#### Per-Entry Expiration Policy

```rust
use moka::{sync::Cache, Expiry};
use std::time::{Duration, Instant};

pub struct MyExpiry;

impl Expiry<u32, String> for MyExpiry {
    fn expire_after_create(
        &self,
        key: &u32,
        value: &String,
        current_time: Instant,
    ) -> Option<Duration> {
        // Custom logic per entry
        if value.len() > 1000 {
            Some(Duration::from_secs(60))  // Large entries: 1 min
        } else {
            Some(Duration::from_secs(300)) // Small entries: 5 min
        }
    }

    fn expire_after_read(
        &self,
        key: &u32,
        value: &String,
        current_time: Instant,
        current_duration: Duration,
        last_modified: Instant,
    ) -> Option<Duration> {
        // Can modify expiration on read
        Some(current_duration)
    }

    fn expire_after_update(
        &self,
        key: &u32,
        value: &String,
        current_time: Instant,
        current_duration: Option<Duration>,
    ) -> Option<Duration> {
        // Can modify expiration on update
        current_duration
    }
}

let cache = Cache::builder()
    .max_capacity(100)
    .expire_after(MyExpiry)
    .build();
```

**Advanced Use Cases:**
- Variable TTL based on value properties
- Dynamic expiration adjustment
- Access-pattern-based expiration
- Content-dependent retention

### 5. High-Performance Operations

#### Entry API - Atomic Operations
```rust
use moka::sync::Cache;

let cache: Cache<String, u32> = Cache::new(100);

// Atomic insert-or-get
let entry = cache.entry("key".to_string())
    .or_insert(42);

// Atomic insert-with-condition
let entry = cache.entry("key".to_string())
    .or_insert_with_if(
        || expensive_computation(),
        |&old_value| old_value < threshold  // Replace condition
    );
```

**Benefits:**
- Reduces race conditions
- Atomic read-modify-write
- Avoids redundant computations
- Better API ergonomics

#### get_with - Coalesced Computation
```rust
use std::sync::Arc;

const TEN_MIB: usize = 10 * 1024 * 1024;

// Multiple threads calling same key
let value = cache.get_with("key1", || {
    // Only ONE thread executes this
    Arc::new(vec![0u8; TEN_MIB])
});
```

**Coalescing Guarantees:**
- Only one init closure runs per key
- Other threads wait for result
- Prevents thundering herd
- Reduces duplicate work

**Variants:**
- `get_with`: Infallible init
- `try_get_with`: Fallible init (returns `Result`)
- `optionally_get_with`: Optional init (returns `Option`)

**Performance Impact:**
- Massive savings for expensive computations
- Database connection pooling
- API rate limiting
- Large object construction

### 6. Bounded Channels & Maintenance

#### Architecture

**Two Bounded Channels:**
1. **Read Channel:** Records cache reads
2. **Write Channel:** Records cache writes

**Channel Draining:**
- Triggered when capacity reached (64 recordings)
- Or after timeout (300ms)
- Performed by user threads, not dedicated thread

**When Channels Are Full:**
- Read channel: Recordings dropped (may impact hit rate)
- Write channel: Operations block until drained

**Maintenance Tasks:**
1. Admission decision (TinyLFU check)
2. Update LFU filter and LRU queues
3. Evict entries exceeding capacity
4. Remove expired entries
5. Process invalidations
6. Call eviction listener

#### run_pending_tasks
```rust
cache.insert("key", "value");

// Stats may be stale
println!("Count: {}", cache.entry_count());  // May show 0

// Force maintenance
cache.run_pending_tasks();

// Now accurate
println!("Count: {}", cache.entry_count());  // Shows 1
```

**Use Cases:**
- Accurate stats retrieval
- Test assertions
- Forced cleanup before shutdown

### 7. Eviction Listener

```rust
let eviction_listener = |key, value, cause| {
    println!("Evicted: key={:?}, cause={:?}", key, cause);
    // Cleanup associated resources
    // Update metrics
    // Trigger background sync
};

let cache = Cache::builder()
    .max_capacity(100)
    .eviction_listener(eviction_listener)
    .build();
```

**Removal Causes:**
- `Size`: Evicted due to size constraints
- `Expired`: TTL/TTI expiration
- `Explicit`: Manual `invalidate` call
- `Replaced`: Value replaced by new insert

**Critical Requirements:**
- **Must not panic:** Panic disables listener permanently
- **Should be fast:** Runs in user thread
- **Use for cleanup:** File handles, connections, metrics

**Logging Panics:**
Enable `logging` feature and check error-level logs.

### 8. Cache Policies Trait

```rust
pub struct Policy {
    pub max_capacity(&self) -> Option<u64>;
    pub time_to_live(&self) -> Option<Duration>;
    pub time_to_idle(&self) -> Option<Duration>;
}

let policy = cache.policy();
println!("Max capacity: {:?}", policy.max_capacity());
```

**Read-Only Access:**
- Inspection of current settings
- Cannot modify after creation
- Use for monitoring/debugging

### 9. Invalidation Operations

#### Single Key Invalidation
```rust
cache.invalidate(&key);
```

#### Bulk Invalidation
```rust
cache.invalidate_all();
```

#### Conditional Invalidation
```rust
use moka::PredicateError;

cache.invalidate_entries_if(|key, value| {
    // Remove all expired items
    value.expiry_time < Instant::now()
})?;
```

**Performance Notes:**
- Async invalidation (processed in batches)
- Doesn't block immediately
- Call `run_pending_tasks()` to force
- Predicate errors stop iteration

### 10. Cache Stats & Monitoring

```rust
let cache = Cache::builder()
    .name("my-cache")  // For logging
    .max_capacity(1000)
    .build();

// Manual metrics
let count = cache.entry_count();       // May be stale
let size = cache.weighted_size();      // May be stale

cache.run_pending_tasks();             // Update stats
let accurate_count = cache.entry_count();
```

**Monitoring Integration:**
- Named caches for log correlation
- Entry count tracking
- Weighted size tracking
- Eviction listener for metrics

### 11. Hashing Algorithm

**Default: SipHash 1-3**
- HashDoS resistant
- Same as `std::HashMap`
- Good for medium-sized keys

**Custom Hasher:**
```rust
use ahash::RandomState;

let cache = Cache::builder()
    .max_capacity(10_000)
    .build_with_hasher(RandomState::new());
```

**Performance Considerations:**
- AHash: Faster for integers and small keys
- SipHash: Better security, slight overhead
- FxHash: Fastest, no HashDoS protection

## Integration with Lithos System

### Recommended Use Cases

1. **In-Memory Ledger Cache**
   - Fast access to recent transactions
   - TTL-based expiration
   - Weighted by transaction size
   - High read concurrency

2. **Computation Result Cache**
   - `get_with` for expensive operations
   - Prevent duplicate work
   - Automatic eviction
   - Size-bounded memory usage

3. **Session/State Cache**
   - TTI for idle session cleanup
   - Per-entry custom expiration
   - Eviction listener for cleanup
   - Thread-safe access

### Performance Optimization Strategies

1. **Choose Right Policy**
   - TinyLFU for general workloads
   - LRU for recency-biased workloads

2. **Tune Capacity**
   - Balance hit rate vs memory
   - Use `weigher` for variable sizes
   - Monitor via `entry_count()`

3. **Optimize Expiration**
   - Use cache-level TTL/TTI when possible
   - Per-entry expiry for complex cases
   - Avoid overly aggressive expiration

4. **Leverage Coalescing**
   - Use `get_with` for expensive ops
   - Reduces thundering herd
   - Prevents duplicate computation

5. **Handle Evictions**
   - Implement eviction listener for cleanup
   - Log evictions for monitoring
   - Use for resource management

6. **Batch Operations**
   - Group related inserts
   - Reduces maintenance overhead
   - Better channel utilization

### Benchmarking Notes

**Strengths:**
- Excellent concurrent read performance
- Near-optimal hit ratios (TinyLFU)
- Low overhead per entry
- Scalable to many cores

**Considerations:**
- Eventually consistent stats
- Maintenance in user threads
- Memory overhead for policy structures
- Channel blocking on heavy writes

## Code Examples

### Basic Usage
```rust
use moka::sync::Cache;

let cache: Cache<String, String> = Cache::new(10_000);

// Insert
cache.insert("key".to_string(), "value".to_string());

// Get (returns Option<String> - cloned)
let value = cache.get(&"key".to_string());

// Atomic get-or-insert
let value = cache.get_with("key".to_string(), || {
    expensive_database_query()
});
```

### Advanced Configuration
```rust
use moka::sync::Cache;
use std::time::Duration;

let cache = Cache::builder()
    .name("lithos-ledger")
    .max_capacity(100_000)
    .weigher(|_key, value: &Transaction| {
        (value.data.len() as u32) + 100  // Overhead estimate
    })
    .time_to_live(Duration::from_secs(3600))    // 1 hour
    .time_to_idle(Duration::from_secs(600))     // 10 minutes
    .eviction_listener(|key, value, cause| {
        metrics::increment_eviction_counter(cause);
    })
    .build();
```

### Per-Entry Expiration
```rust
use moka::{sync::Cache, Expiry};
use std::time::{Duration, Instant};

struct TransactionExpiry;

impl Expiry<TxId, Transaction> for TransactionExpiry {
    fn expire_after_create(
        &self,
        _key: &TxId,
        value: &Transaction,
        _current_time: Instant,
    ) -> Option<Duration> {
        match value.priority {
            Priority::High => Some(Duration::from_secs(3600)),  // 1 hour
            Priority::Low  => Some(Duration::from_secs(300)),   // 5 minutes
        }
    }
}

let cache = Cache::builder()
    .max_capacity(10_000)
    .expire_after(TransactionExpiry)
    .build();
```

### High-Performance Pattern
```rust
use std::sync::Arc;

// Wrap expensive-to-clone data in Arc
let cache: Cache<String, Arc<Vec<u8>>> = Cache::new(1000);

// Insert
cache.insert("key".to_string(), Arc::new(large_data));

// Get returns Arc clone (cheap)
let value: Arc<Vec<u8>> = cache.get(&"key".to_string()).unwrap();
// No data copy, just reference count increment
```

## Summary for Lithos

Moka provides exceptional performance through:
- Lock-free concurrent hash table
- Advanced eviction policies (TinyLFU)
- Coalesced computations via `get_with`
- Flexible expiration (TTL, TTI, per-entry)
- Efficient batch maintenance
- Low per-entry overhead
- Scalable multi-core performance

**Best suited for:** In-memory caching with high read concurrency, advanced eviction requirements, and flexible expiration policies.
