# redb Performance & Optimization

`redb` is engineered for high-performance embedded workloads. This guide details how to tune its configuration and utilize its APIs for maximum throughput.

---

## 1. Memory-Mapped I/O (mmap)
`redb` uses memory-mapping to access the database file. This allows the OS to manage page caching and enables zero-copy reads.

- **Direct Access:** Calling `AccessGuard::value()` typically returns a reference directly into the mmap'd region.
- **WASI/Windows:** `redb` provides platform-specific implementations to ensure safety and performance across different environments.
> **Source:** [redb Design - File Format](https://github.com/cberner/redb/blob/master/docs/design.md#file-format)

---

## 2. Optimized Write Patterns

### `insert_reserve` (Zero-Allocation Writes)
For large values, use `insert_reserve` to avoid intermediate buffer allocations.
```rust
// ❌ SLOW: Serializes to Vec, then copies to DB
table.insert(key, &data.to_vec())?;

// ✅ FAST: Allocates directly in DB file, then serializes in-place
let mut guard = table.insert_reserve(key, data.len() as u32)?;
guard.as_mut().copy_from_slice(&data);
```

### Batching
`redb`'s overhead is largely tied to transaction commits (`fsync`).
- **Guidance:** Group multiple inserts/removes into a single `WriteTransaction`.
- **Impact:** Amortizes the cost of the B-tree re-balancing and disk synchronization.

---

## 3. Configuration & Tuning
Use the `redb::Builder` to optimize for your hardware and data access patterns.

### Cache Size (`set_cache_size`)
- **Default:** 1 GiB.
- **Impact:** Larger caches improve performance for repetitive reads and complex range scans by reducing the frequency of page faults.
> **Source:** [redb::Builder::set_cache_size](https://docs.rs/redb/latest/redb/struct.Builder.html#method.set_cache_size)

### Page Size
- **Default:** 4096 bytes (Standard OS page).
- **Small Values:** Smaller page sizes reduce write amplification for frequent updates to tiny records.
- **Large Values/Range Scans:** Larger page sizes reduce B-tree depth (fewer I/O hops) and improve range scan performance.
> **Source:** [redb Design - B-tree Pages](https://github.com/cberner/redb/blob/master/docs/design.md#b-tree-pages)

---

## 4. Performance Metrics (Benchmarks)
Benchmarks on modern NVMe hardware (Ryzen 9950X3D) show `redb`'s competitive edge in write performance.

| Operation | redb | LMDB | SQLite |
| :--- | :--- | :--- | :--- |
| **Individual Writes** | **920ms** | 1598ms | 7040ms |
| **Random Reads (1 thread)** | 1138ms | **637ms** | 4283ms |
| **Bulk Load** | 17063ms | **9232ms** | 15341ms |

*Note: LMDB leads in read-only performance due to its simpler architecture, but `redb` is often faster for mixed or write-heavy workloads.*
> **Source:** [redb GitHub README - Benchmarks](https://github.com/cberner/redb#benchmarks)

---

## 5. Storage Efficiency
- **No Compression:** `redb` does not natively compress data to maintain zero-copy speed. If storage space is critical, compress values *before* insertion.
- **Dynamic Shrinking:** `redb` can automatically truncate its file size when trailing regions become empty. See [Maintenance](maintenance.md) for more.
