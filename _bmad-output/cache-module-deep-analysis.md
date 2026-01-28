# Cache Module Deep Gap Analysis

**Date:** January 28, 2026
**Scope:** `crates/adapters/src/spi/cache/`
**Objective:** Compare current implementation against ADR 0002 and latest crate capabilities to maximize performance and functionality.

---

## 1. `encoder.rs` vs. `rkyv` 0.8 Capabilities

### Current State

The `Codec` trait and `RkyvCodec` implementation currently rely on the high-level `rkyv::to_bytes` API, which forces allocation of an `AlignedVec`.

### Gap Analysis

| Feature            | Current Impl                                  | `rkyv` 0.8 Capability                                                | Gap Impact                                                                                                                                    |
| ------------------ | --------------------------------------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **Write Strategy** | `to_bytes` (Allocates `AlignedVec`)           | `rkyv::api::high::to_bytes_in` or custom serializer over `&mut [u8]` | **Critical**: Prevents using Redb's `insert_reserve` for zero-copy writes. We double-copy data (struct -> AlignedVec -> Redb page).           |
| **Buffer Reuse**   | `encode_key_into` (Clears `Vec` then extends) | `ScratchTracker` / reusing serializer resources                      | **Moderate**: We are reusing the output buffer, but `rkyv` itself re-allocates its internal serialization context (scratch space) every call. |
| **Validation**     | `access` with `bytecheck`                     | `access` with `bytecheck` (Correct)                                  | None. Correctly implemented.                                                                                                                  |
| **Alignment**      | Manual `align_offset` check                   | `rkyv` handles alignment requirements types                          | **Minor**: Our manual check is good defensive programming but `rkyv::access` also checks this.                                                |

### Recommendation for `encoder.rs`

1.  **Add `serialize_into` Method**:

    ```rust
    /// Serialize directly into a mutable slice.
    /// Returns the number of bytes written.
    fn serialize_into(&self, value: &V, buffer: &mut [u8]) -> Result<usize, CacheError>;
    ```

    _Why:_ Essential for Redb `insert_reserve`.

2.  **Add `serialized_size` Method**:
    ```rust
    /// Calculate required size before serialization.
    fn serialized_size(&self, value: &V) -> Result<usize, CacheError>;
    ```
    _Why:_ Redb `insert_reserve` requires knowing the size _before_ writing.

---

## 2. `redb.rs` vs. `redb` 3.1 Capabilities

### Current State

Uses standard `Table` operations (`get`, `insert`) and basic transaction management.

### Gap Analysis

| Feature              | Current Impl                       | `redb` 3.1 Capability                                                | Gap Impact                                                                                                                                                             |
| -------------------- | ---------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Zero-Copy Writes** | `table.insert` (Copies from slice) | `table.insert_reserve` (Returns `&mut [u8]` in-place)                | **High**: We adhere to zero-copy _reads_ (via AccessGuard) but fail at zero-copy _writes_.                                                                             |
| **Durability**       | Default (`Immediate`)              | `WriteTransaction::set_durability` (`None`, `Eventual`, `Immediate`) | **High**: Bulk indexing will be slow because every `put` forces an fsync. We need `Durability::None` for batch jobs.                                                   |
| **Batch Operations** | Sequential loop                    | Single transaction for multiple ops                                  | **Moderate**: `redb` commits are expensive. Doing 1000 `get`s in 1 txn is vastly faster than 1000 txns.                                                                |
| **Table Types**      | `Table` only                       | `MultimapTable` (Duplicate keys)                                     | **Future**: ADR mentions `MultimapTable` for backlinks (#tag -> [id1, id2]). Current implementation doesn't expose this, effectively blocking backlink implementation. |
| **Cache Tuning**     | Default                            | `Builder::set_cache_size`, `set_page_size`                           | **Moderate**: Defaults are conservative. For 100k+ notes, we need to explicitly set cache size (e.g. 128MB+).                                                          |

### Recommendation for `redb.rs`

1.  **Implement `insert_reserve` flow**:
    - Calculate size via `codec.serialized_size()`.
    - Call `table.insert_reserve()`.
    - Serialize directly into the returned guard.
2.  **Expose Durability**: Add to `Builder` and potentially `Writer` (or a `BatchWriter` interface).
3.  **Implement `Multimap` support**: (Optional for this pass, but architectural gap).
4.  **Tuning**: Expose `set_cache_size` in Builder.

---

## 3. `moka.rs` vs. `moka` 0.12 Capabilities

### Current State

Basic wrapper around `moka::future::Cache`.

### Gap Analysis

| Feature                 | Current Impl | `moka` 0.12 Capability                     | Gap Impact                                                                                                                                       |
| ----------------------- | ------------ | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Testing Determinism** | Implicit     | `run_pending_tasks()`                      | **Moderate**: Tests involving eviction/expiration might be flaky without explicit task running.                                                  |
| **Metrics**             | None         | `entry_count`, `weighted_size`, `hit_rate` | **Low/Moderate**: Hard to tune cache size without visibility.                                                                                    |
| **Storage Layout**      | Stores `V`   | Should store `Entry<V>`                    | **Architectural**: To support `timestamp()` API consistently across Redb and Moka, Moka must store the wrapper `Entry` containing the timestamp. |
| **Eviction Listener**   | None         | `eviction_listener`                        | **Future**: Needed if we want to clean up external resources when items drop from RAM.                                                           |

### Recommendation for `moka.rs`

1.  **Store `Entry<V>`**: Change `Cache<K, V>` to `Cache<K, Entry<V>>`.
2.  **Expose Metrics**: Add a method to retrieve cache stats.
3.  **Maintenance**: Expose `run_pending_tasks` for testing.

---

## Implementation Plan Adjustments

Based on this deep dive, the implementation tasks need to be more specific:

**Day 1: `mod.rs` (Traits)**

- Add `serialized_size` and `serialize_into` to `Codec` trait (needed for Redb zero-copy write).

**Day 2: `encoder.rs` (Optimization)**

- Implement the new trait methods using `rkyv`'s buffer serializers.

**Day 3: `moka.rs` (Feature Parity)**

- Switch to `Entry<V>`.
- Add metrics and maintenance hooks.

**Day 4: `redb.rs` (Performance)**

- Implement `put` using `insert_reserve` (requires the changes from Day 1 & 2).
- Add durability and cache size tuning to Builder.

**Day 5: `coordinator.rs`**

- Wire it all up.

This order ensures dependencies are met: `redb` needs the updated `encoder` capabilities, which need the updated `Codec` trait.
