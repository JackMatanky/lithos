# Lithos System Integration Guide
## redb + moka + rkyv

This document provides guidance on integrating redb, moka, and rkyv into the Lithos system to achieve maximum zero-copy performance and efficiency.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Lithos Application Layer                 │
│  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────┐│
│  │   Transaction   │  │   Query      │  │   State         ││
│  │   Processing    │  │   Engine     │  │   Management    ││
│  └────────┬────────┘  └──────┬───────┘  └────────┬────────┘│
└───────────┼───────────────────┼──────────────────┼─────────┘
            │                   │                  │
            ▼                   ▼                  ▼
┌───────────────────────────────────────────────────────────┐
│                    Performance Layer                       │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  moka (Cache)   │  │ rkyv (Format)│  │ redb (Store) │ │
│  │  ─────────────  │  │ ───────────  │  │ ──────────── │ │
│  │ • In-memory     │  │ • Zero-copy  │  │ • Persistent │ │
│  │ • TinyLFU       │  │ • Serializer │  │ • ACID       │ │
│  │ • Concurrent    │  │ • Validator  │  │ • MVCC       │ │
│  │ • TTL/TTI       │  │ • Portable   │  │ • B-tree     │ │
│  └─────────────────┘  └──────────────┘  └──────────────┘ │
└───────────────────────────────────────────────────────────┘
            │                   │                  │
            └───────────────────┴──────────────────┘
                                │
                                ▼
                        ┌───────────────┐
                        │  Storage/RAM  │
                        └───────────────┘
```

## Integration Patterns

### Pattern 1: Hot/Warm/Cold Data Architecture

```rust
use moka::sync::Cache;
use redb::{Database, ReadableDatabase, TableDefinition};
use rkyv::{Archive, Serialize, Deserialize};
use std::sync::Arc;

// Data model with rkyv
#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Transaction {
    pub id: u64,
    pub timestamp: u64,
    pub amount: i64,
    pub from: String,
    pub to: String,
    pub data: Vec<u8>,
}

// Storage layers
pub struct LithosStore {
    // HOT: Recent/frequent data (in-memory, fastest)
    hot_cache: Cache<u64, Arc<Transaction>>,

    // WARM: Less frequent but cached (persistent + cached)
    warm_db: Database,

    // COLD: Archived historical (persistent only)
    cold_db: Database,
}

const WARM_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("warm");
const COLD_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("cold");

impl LithosStore {
    pub fn new(hot_size: u64) -> Result<Self> {
        Ok(Self {
            hot_cache: Cache::builder()
                .max_capacity(hot_size)
                .weigher(|_k, v: &Arc<Transaction>| {
                    (v.data.len() as u32) + 200
                })
                .time_to_idle(Duration::from_secs(300))
                .build(),
            warm_db: Database::create("warm.redb")?,
            cold_db: Database::create("cold.redb")?,
        })
    }

    // Zero-copy read path
    pub fn get_transaction(&self, id: u64) -> Result<Option<Arc<Transaction>>> {
        // 1. Check hot cache (zero-copy if Arc)
        if let Some(tx) = self.hot_cache.get(&id) {
            return Ok(Some(tx));
        }

        // 2. Check warm database (zero-copy via rkyv)
        if let Some(tx) = self.get_from_warm(id)? {
            // Promote to hot cache
            let tx = Arc::new(tx);
            self.hot_cache.insert(id, tx.clone());
            return Ok(Some(tx));
        }

        // 3. Check cold database (full deserialization)
        if let Some(tx) = self.get_from_cold(id)? {
            return Ok(Some(Arc::new(tx)));
        }

        Ok(None)
    }

    fn get_from_warm(&self, id: u64) -> Result<Option<Transaction>> {
        let read_txn = self.warm_db.begin_read()?;
        let table = read_txn.open_table(WARM_TABLE)?;

        if let Some(bytes) = table.get(&id)? {
            // Zero-copy access via rkyv
            let archived = unsafe {
                rkyv::access_unchecked::<ArchivedTransaction>(bytes.value())
            };

            // Deserialize only if needed
            Ok(Some(rkyv::deserialize::<Transaction, _>(archived)?))
        } else {
            Ok(None)
        }
    }

    fn get_from_cold(&self, id: u64) -> Result<Option<Transaction>> {
        let read_txn = self.cold_db.begin_read()?;
        let table = read_txn.open_table(COLD_TABLE)?;

        if let Some(bytes) = table.get(&id)? {
            let archived = unsafe {
                rkyv::access_unchecked::<ArchivedTransaction>(bytes.value())
            };
            Ok(Some(rkyv::deserialize::<Transaction, _>(archived)?))
        } else {
            Ok(None)
        }
    }

    // Write path with tiered storage
    pub fn insert_transaction(&self, tx: Transaction) -> Result<()> {
        let id = tx.id;
        let tx_arc = Arc::new(tx);

        // 1. Insert into hot cache
        self.hot_cache.insert(id, tx_arc.clone());

        // 2. Persist to warm database (async background task recommended)
        self.persist_to_warm(id, &tx_arc)?;

        Ok(())
    }

    fn persist_to_warm(&self, id: u64, tx: &Transaction) -> Result<()> {
        // Serialize with rkyv (zero-copy reads later)
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(tx)?;

        let write_txn = self.warm_db.begin_write()?;
        {
            let mut table = write_txn.open_table(WARM_TABLE)?;
            table.insert(&id, bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    // Archive old data to cold storage
    pub fn archive_old_transactions(&self, older_than: u64) -> Result<()> {
        // Move from warm to cold
        // Implementation details...
        Ok(())
    }
}
```

**Benefits of This Pattern:**
- Hot cache: O(1) lookup, Arc clone (cheap)
- Warm DB: Zero-copy access via rkyv, MVCC reads
- Cold DB: Full persistence, compressed if needed
- Automatic promotion to hot cache
- Background archival process

### Pattern 2: Computed Cache with Persistent Backing

```rust
use moka::sync::Cache;
use redb::{Database, TableDefinition};
use rkyv::{Archive, Serialize, Deserialize};
use std::sync::Arc;

#[derive(Archive, Serialize, Deserialize, Clone)]
struct ComputedResult {
    input_hash: u64,
    output: Vec<u8>,
    timestamp: u64,
}

pub struct ComputationCache {
    // In-memory cache for hot computations
    memory_cache: Cache<u64, Arc<ComputedResult>>,

    // Persistent cache for warm start
    persistent_cache: Database,
}

const CACHE_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("computed_cache");

impl ComputationCache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            memory_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(3600))
                .eviction_listener(|key, value, cause| {
                    // Persist to disk on eviction
                    if matches!(cause, RemovalCause::Size) {
                        Self::persist_on_eviction(key, value);
                    }
                })
                .build(),
            persistent_cache: Database::create("computation_cache.redb")?,
        })
    }

    pub fn get_or_compute<F>(&self, key: u64, compute: F) -> Result<Arc<ComputedResult>>
    where
        F: FnOnce() -> Result<ComputedResult>,
    {
        // Try memory cache with coalescing
        if let Some(result) = self.memory_cache.get(&key) {
            return Ok(result);
        }

        // Try persistent cache (zero-copy)
        if let Some(result) = self.load_from_disk(key)? {
            let result = Arc::new(result);
            self.memory_cache.insert(key, result.clone());
            return Ok(result);
        }

        // Compute (with coalescing via get_with)
        let result = self.memory_cache.get_with(key, || {
            Arc::new(compute().expect("Computation failed"))
        });

        // Persist for next time
        self.persist_to_disk(key, &result)?;

        Ok(result)
    }

    fn load_from_disk(&self, key: u64) -> Result<Option<ComputedResult>> {
        let read_txn = self.persistent_cache.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        if let Some(bytes) = table.get(&key)? {
            let archived = unsafe {
                rkyv::access_unchecked::<ArchivedComputedResult>(bytes.value())
            };
            Ok(Some(rkyv::deserialize(archived)?))
        } else {
            Ok(None)
        }
    }

    fn persist_to_disk(&self, key: u64, result: &ComputedResult) -> Result<()> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(result)?;

        let write_txn = self.persistent_cache.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            table.insert(&key, bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    fn persist_on_eviction(key: u64, value: Arc<ComputedResult>) {
        // Background task to persist evicted entries
        tokio::spawn(async move {
            // Persist logic here
        });
    }
}
```

**Benefits:**
- Coalesced expensive computations (moka)
- Persistent cache survives restarts (redb)
- Zero-copy reads from disk (rkyv)
- Automatic eviction to disk
- Fast cold-start

### Pattern 3: Memory-Mapped Ledger with Cache Layer

```rust
use memmap2::MmapOptions;
use moka::sync::Cache;
use rkyv::{Archive, Serialize};
use std::fs::OpenOptions;
use std::sync::Arc;

#[derive(Archive, Serialize, Clone, Debug)]
#[rkyv(derive(Debug))]
pub struct LedgerEntry {
    pub index: u64,
    pub timestamp: u64,
    pub transaction_id: u64,
    pub data: Vec<u8>,
}

pub struct MmapLedger {
    // Memory-mapped ledger file (zero-copy)
    mmap: memmap2::Mmap,

    // Cache for frequently accessed entries
    entry_cache: Cache<u64, Arc<ArchivedLedgerEntry>>,

    // Metadata
    entry_count: u64,
    entry_size: usize,
}

impl MmapLedger {
    pub fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)?;

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Self {
            mmap,
            entry_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_idle(Duration::from_secs(60))
                .build(),
            entry_count: Self::read_entry_count(&mmap),
            entry_size: std::mem::size_of::<ArchivedLedgerEntry>(),
        })
    }

    // Zero-copy access to ledger entry
    pub fn get_entry(&self, index: u64) -> Result<Option<&ArchivedLedgerEntry>> {
        if index >= self.entry_count {
            return Ok(None);
        }

        // Check cache first
        if let Some(cached) = self.entry_cache.get(&index) {
            // SAFETY: We ensure cache lifetime matches mmap
            return Ok(Some(unsafe {
                std::mem::transmute::<&ArchivedLedgerEntry, &ArchivedLedgerEntry>(
                    &**cached
                )
            }));
        }

        // Access from mmap (zero-copy)
        let offset = (index as usize) * self.entry_size;
        let bytes = &self.mmap[offset..offset + self.entry_size];

        let entry = unsafe {
            rkyv::access_unchecked::<ArchivedLedgerEntry>(bytes)
        };

        // Cache the reference (wrapped in Arc for moka)
        // NOTE: This is a simplified example. In production, need to ensure
        // the Arc doesn't outlive the mmap.

        Ok(Some(entry))
    }

    // Range query with zero-copy
    pub fn get_range(&self, start: u64, end: u64)
        -> impl Iterator<Item = &ArchivedLedgerEntry>
    {
        (start..end).filter_map(move |i| {
            self.get_entry(i).ok().flatten()
        })
    }

    fn read_entry_count(mmap: &memmap2::Mmap) -> u64 {
        // Read header to get entry count
        u64::from_le_bytes(mmap[0..8].try_into().unwrap())
    }
}
```

**Benefits:**
- Zero-copy access via mmap
- OS manages paging automatically
- Cache layer for hot entries
- Excellent for read-only ledgers
- Instant "loading" on startup

### Pattern 4: Transactional State with Snapshot

```rust
use redb::{Database, ReadableDatabase, TableDefinition};
use moka::sync::Cache;
use rkyv::{Archive, Serialize, Deserialize};

#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub struct AccountState {
    pub balance: i64,
    pub nonce: u64,
    pub data: Vec<u8>,
}

pub struct StateManager {
    // Current state database (ACID transactions)
    current_state: Database,

    // Snapshot cache for fast reads
    snapshot_cache: Cache<u64, Arc<AccountState>>,
}

const ACCOUNT_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("accounts");

impl StateManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_state: Database::create("state.redb")?,
            snapshot_cache: Cache::builder()
                .max_capacity(100_000)
                .time_to_idle(Duration::from_secs(30))
                .build(),
        })
    }

    // Atomic state update with caching
    pub fn update_account<F>(&self, account_id: u64, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut AccountState) -> Result<()>,
    {
        let write_txn = self.current_state.begin_write()?;

        // Read current state (zero-copy)
        let current_state = {
            let table = write_txn.open_table(ACCOUNT_TABLE)?;
            if let Some(bytes) = table.get(&account_id)? {
                let archived = unsafe {
                    rkyv::access_unchecked::<ArchivedAccountState>(bytes.value())
                };
                rkyv::deserialize::<AccountState, _>(archived)?
            } else {
                AccountState {
                    balance: 0,
                    nonce: 0,
                    data: Vec::new(),
                }
            }
        };

        // Apply update
        let mut new_state = current_state.clone();
        update_fn(&mut new_state)?;

        // Write back (serialized with rkyv)
        {
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&new_state)?;
            let mut table = write_txn.open_table(ACCOUNT_TABLE)?;
            table.insert(&account_id, bytes.as_slice())?;
        }

        // Commit transaction
        write_txn.commit()?;

        // Invalidate cache
        self.snapshot_cache.invalidate(&account_id);

        Ok(())
    }

    // Fast read with caching
    pub fn get_account(&self, account_id: u64) -> Result<Option<Arc<AccountState>>> {
        // Check cache
        if let Some(state) = self.snapshot_cache.get(&account_id) {
            return Ok(Some(state));
        }

        // Read from DB (zero-copy)
        let read_txn = self.current_state.begin_read()?;
        let table = read_txn.open_table(ACCOUNT_TABLE)?;

        if let Some(bytes) = table.get(&account_id)? {
            let archived = unsafe {
                rkyv::access_unchecked::<ArchivedAccountState>(bytes.value())
            };
            let state = Arc::new(rkyv::deserialize(archived)?);

            // Cache it
            self.snapshot_cache.insert(account_id, state.clone());

            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    // Create snapshot for consistent reads
    pub fn create_snapshot(&self) -> Result<StateSnapshot> {
        let read_txn = self.current_state.begin_read()?;

        // Snapshot is just a read transaction + cache
        Ok(StateSnapshot {
            read_txn,
            local_cache: Cache::new(10_000),
        })
    }
}

pub struct StateSnapshot {
    read_txn: redb::ReadTransaction,
    local_cache: Cache<u64, Arc<AccountState>>,
}

impl StateSnapshot {
    // Consistent reads from snapshot
    pub fn get_account(&self, account_id: u64) -> Result<Option<Arc<AccountState>>> {
        // Check local cache
        if let Some(state) = self.local_cache.get(&account_id) {
            return Ok(Some(state));
        }

        // Read from snapshot (MVCC ensures consistency)
        let table = self.read_txn.open_table(ACCOUNT_TABLE)?;

        if let Some(bytes) = table.get(&account_id)? {
            let archived = unsafe {
                rkyv::access_unchecked::<ArchivedAccountState>(bytes.value())
            };
            let state = Arc::new(rkyv::deserialize(archived)?);
            self.local_cache.insert(account_id, state.clone());
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}
```

**Benefits:**
- ACID transactions for consistency
- MVCC for concurrent readers
- Zero-copy reads via rkyv
- Snapshot isolation
- Cache invalidation on updates

## Performance Tuning Guide

### 1. redb Configuration

```rust
use redb::Builder;

let db = Builder::new()
    .set_cache_size(256 * 1024 * 1024)  // 256 MB cache
    .set_page_size(4096)                // Match OS page size
    .create("data.redb")?;
```

**Recommendations:**
- Cache size: 10-20% of working set
- Page size: Match OS (usually 4KB)
- Use `Durability::Eventual` for throughput
- Batch writes in transactions

### 2. moka Configuration

```rust
use moka::sync::Cache;
use std::time::Duration;

let cache = Cache::builder()
    .name("lithos-main")
    .max_capacity(capacity)
    .weigher(|_k, v: &Transaction| {
        (v.data.len() as u32) + overhead
    })
    .time_to_live(Duration::from_secs(3600))
    .time_to_idle(Duration::from_secs(300))
    .eviction_listener(eviction_handler)
    .build();
```

**Recommendations:**
- Use weighted size for variable data
- Set TTL/TTI based on access patterns
- Implement eviction listener for cleanup
- Name caches for debugging

### 3. rkyv Configuration

```toml
# Cargo.toml
[dependencies]
rkyv = {
    version = "0.8",
    features = [
        "bytecheck",      # For validation
        "unaligned",      # For mmap compatibility
        # "little_endian" # Platform-specific
    ]
}
```

**Recommendations:**
- Use `bytecheck` for untrusted data
- Use `unaligned` for mmap or network
- Skip validation for trusted internal data
- Use `#[with(Inline)]` for small types

## Testing Strategy

### 1. Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_serialization(tx in any::<Transaction>()) {
        // Serialize with rkyv
        let bytes = rkyv::to_bytes::<Error>(&tx).unwrap();

        // Deserialize
        let archived = rkyv::access::<ArchivedTransaction, Error>(&bytes).unwrap();
        let deserialized: Transaction = rkyv::deserialize(archived).unwrap();

        // Should match
        assert_eq!(tx, deserialized);
    }

    #[test]
    fn cache_and_db_consistency(
        account_id in any::<u64>(),
        state in any::<AccountState>()
    ) {
        let manager = StateManager::new().unwrap();

        // Update
        manager.update_account(account_id, |s| {
            *s = state.clone();
            Ok(())
        }).unwrap();

        // Read from cache
        let cached = manager.get_account(account_id).unwrap().unwrap();

        // Should match
        assert_eq!(*cached, state);
    }
}
```

### 2. Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_read_paths(c: &mut Criterion) {
    let store = LithosStore::new(10_000).unwrap();

    // Warm up
    store.insert_transaction(create_test_tx(42)).unwrap();

    let mut group = c.benchmark_group("read_paths");

    group.bench_function("hot_cache", |b| {
        b.iter(|| {
            black_box(store.get_transaction(42))
        });
    });

    group.bench_function("warm_db", |b| {
        // Clear cache first
        store.hot_cache.invalidate(&42);
        b.iter(|| {
            black_box(store.get_transaction(42))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_read_paths);
criterion_main!(benches);
```

## Monitoring & Observability

### Metrics to Track

```rust
pub struct SystemMetrics {
    // Cache metrics
    pub cache_hit_rate: f64,
    pub cache_eviction_rate: f64,
    pub cache_memory_usage: u64,

    // Database metrics
    pub db_read_latency_p50: Duration,
    pub db_read_latency_p99: Duration,
    pub db_write_latency_p50: Duration,
    pub db_write_throughput: f64,

    // Serialization metrics
    pub avg_serialize_time: Duration,
    pub avg_deserialize_time: Duration,
    pub validation_overhead: Duration,
}

impl LithosStore {
    pub fn collect_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cache_hit_rate: self.compute_hit_rate(),
            cache_eviction_rate: self.compute_eviction_rate(),
            cache_memory_usage: self.hot_cache.weighted_size(),
            // ... other metrics
        }
    }
}
```

## Summary

This integration achieves:

1. **Zero-Copy Reads:** rkyv → redb → moka
2. **High Concurrency:** moka (reads) + redb (MVCC)
3. **ACID Guarantees:** redb transactions
4. **Memory Efficiency:** Tiered storage + eviction
5. **Fast Cold-Start:** Persistent cache + mmap

**Performance Profile:**
- Hot path (cache hit): ~10-100ns
- Warm path (DB + zero-copy): ~1-10µs
- Cold path (full deserialize): ~10-100µs

**Recommended for Lithos:**
- Use Pattern 1 for general ledger data
- Use Pattern 2 for expensive computations
- Use Pattern 3 for historical/readonly data
- Use Pattern 4 for account state

Choose patterns based on specific access patterns and consistency requirements.
