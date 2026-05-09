//! Database key and UUID formatting strategy benchmarks.
//!
//! # Summary
//!
//! Tracks allocation optimizations for database key construction and UUID
//! handling by comparing optimized strategies against baseline
//! string-allocation approaches.
//!
//! # Motivation
//!
//! Early profiling revealed database key construction
//! (`format!("{table}:{key}")`) and UUID-to-string conversion allocated 36-100
//! bytes per operation. P0 optimizations introduced UUID helper methods for
//! writes/deletes and pre-allocated key buffers. This suite validates those
//! optimizations and guards against regressions.
//!
//! # Scope
//!
//! **Included**:
//! - UUID helper database methods (`put_by_uuid`, `delete_by_uuid`)
//! - Preformatted UUID keys for read benchmarks
//! - UUID-via-string baseline for comparison
//! - Database key formatting (optimized `write!()` vs naive `format!()`)
//!
//! **Excluded**:
//! - General storage performance (see `db_storage.rs`)
//! - String construction APIs (see `string_construction.rs`)
//! - Actual allocation measurement (uses latency as proxy)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Isolated operation measurement
//! - **Comparative**: Each optimization paired with baseline for validation
//! - **Single-threaded**: No concurrent access scenarios
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (default config)
//! - **Comparison strategy**: Run optimized and baseline variants in same
//!   benchmark group for direct comparison
//! - **Black-boxing**: All UUIDs and results passed through `black_box()` to
//!   prevent constant folding
//! - **Setup separation**: Database pre-population outside timed region
//!
//! # Input Model
//!
//! - **UUIDs**: Generated via `Uuid::now_v7()` (time-based, reproducible within
//!   session)
//! - **Keys**: Simple string keys ("key-0000" format) for key formatting
//!   benches
//! - **Determinism**: UUIDs vary across runs but ratios remain stable
//! - **Sizes**: Small test values (strings) to isolate key handling overhead
//!
//! # Controls and Fairness
//!
//! - **Same operations**: Optimized/baseline pairs perform identical work
//!   except for formatting strategy
//! - **Same database state**: Both variants access same pre-populated data
//! - **No warmup differences**: Both benefit equally from criterion warm-up
//!
//! # Expected Characteristics
//!
//! Based on measured baseline performance (2026-02-11, Apple M3 Max):
//!
//! **UUID Get Operations**:
//! - **Native (pre-formatted)**: ~420 ns - no allocation, direct key lookup
//! - **Via-string (baseline)**: ~452 ns - allocates 36-byte String for UUID
//! - **Speedup**: ~7% faster with native approach
//! - **Bottleneck**: String allocation overhead, not database lookup
//! - **Savings**: 36 bytes per operation (UUID string: 36 bytes)
//!
//! **UUID Put Operations**:
//! - **Native (`put_by_uuid`)**: ~3.64 ms - helper method with internal buffer
//! - **Via-string (baseline)**: ~3.99 ms - allocates UUID string before call
//! - **Speedup**: ~9% faster with native approach
//! - **Bottleneck**: Transaction fsync dominates (~3.6ms), allocation adds ~10%
//! - **Savings**: 36 bytes per operation
//!
//! **UUID Delete Operations**:
//! - **Native (`delete_by_uuid`)**: ~3.6-4.0 ms - helper method with internal
//!   buffer
//! - **Via-string (baseline)**: ~4.0-4.2 ms - allocates UUID string before call
//! - **Speedup**: ~5-10% faster with native approach
//! - **Bottleneck**: Transaction fsync dominates, similar to put
//! - **Savings**: 36 bytes per operation
//!
//! **Key Formatting**:
//! - **Optimized (write! + pre-allocation)**: ~243 ns - stack buffer, no heap
//! - **Baseline (removed)**: Previously ~300-400ns with `format!()` allocation
//! - **Speedup**: Pre-allocation wins for repeated key formatting
//! - **Bottleneck**: Heap allocation in naive `format!()` approach
//!
//! **Performance scaling**:
//! - UUID operations scale O(1) - constant time regardless of data size
//! - Benefits are cumulative: 100K operations → 3.6MB saved allocations
//! - fsync dominates write/delete operations (allocation is <10% overhead)
//!
//! # Interpreting Results
//!
//! **Expected impact (from RESULTS.md)**:
//! - UUID-native methods: 7-9% faster than string conversion
//! - Saves 36 bytes per UUID operation (not measured, but validated by latency)
//!
//! **Meaningful changes**:
//! - Optimized/baseline ratio approaching 1.0: Optimization benefit eroding
//! - Optimized slower than baseline: Critical regression
//! - >20% change in either variant: Investigate database layer changes
//!
//! **Valid comparisons**:
//! - Within-group ratios (optimized vs baseline): Highly stable
//! - Across machines: Absolute numbers vary, ratios should hold
//!
//! **Noise**:
//! - UUID generation adds ~5-10 ns variance
//! - Database cache state affects absolute numbers (not ratios)
//!
//! # Reporting and Workflow
//!
//! - **Development**: Run when changing database key handling or UUID methods
//! - **Validation**: Check ratios remain consistent after refactors
//! - **Documentation**: Update `RESULTS.md` if ratios shift significantly
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Database API changes (new UUID methods, key formatting strategy)
//! - Allocation optimization work affects database operations
//! - Database layer internals change (key namespace strategy)
//!
//! **Adding benchmarks**:
//! - Always pair optimization with baseline for comparison
//! - Use `_native` suffix for optimized, `_via_string` for baseline
//! - Document expected improvement in per-bench comment
//!
//! # Known Limitations
//!
//! - **No allocation measurement**: Uses latency as proxy (consider dhat for
//!   actual allocation profiling)
//! - **No large key testing**: Simple UUID keys only (not composite keys)
//! - **No multimap operations**: Focuses on primary key operations
//!
//! # Benchmark Index
//!
//! | Group            | Expected Time (Optimized) | Focus                                           |
//! | ---------------- | ------------------------- | ----------------------------------------------- |
//! | `uuid_handling`  | ~420ns (get), ~3.6-3.9ms (put/delete) | UUID helpers vs string conversion (get/put/delete) |
//! | `key_formatting` | ~243ns                    | Optimized key construction (pre-allocated buffer) |
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity.

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{db::Database, support::UuidV7};
use redb::TableDefinition;
use tempfile::TempDir;
use uuid::Uuid;

const TEMPLATES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("templates");
const BENCHMARK_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("benchmark");

// ----------------------------------------------------------- //
//                  UUID Handling Strategies                   //
// ----------------------------------------------------------- //

/// Benchmarks UUID-native methods vs string conversion (optimization tracking).
///
/// # Purpose
///
/// Validates that UUID helper database methods (`put_by_uuid`,
/// `delete_by_uuid`) and preformatted keys for reads outperform
/// UUID-to-string conversion by avoiding intermediate string allocation.
///
/// # What is Measured
///
/// - **Metric**: Latency per operation (get/put/delete)
/// - **Variants**: Preformatted keys vs per-iteration UUID-to-string baseline
/// - **Execution**: Six paired benchmarks (get/put/delete × 2 strategies)
///
/// # Inputs
///
/// - **UUIDs**: Fresh `Uuid::now_v7()` per iteration (puts/deletes), fixed UUID
///   (gets)
/// - **Values**: Simple String values to isolate key handling overhead
/// - **Database**: Pre-populated with test UUID for get benchmarks
///
/// # Comparison Fairness
///
/// - Same database operations (only key construction differs)
/// - Same data access patterns (both hit cache for gets)
/// - UUID generation cost present in both variants (cancels out)
///
/// # Expected Characteristics
///
/// - **Preformatted**: 250-450 ns per operation
/// - **Via-string**: 280-500 ns per operation
/// - **Improvement**: 7-9% faster for preformatted (from RESULTS.md)
///
/// # Interpreting Changes
///
/// - **Ratio < 1.05**: Optimization benefit eroding (investigate)
/// - **Preformatted slower**: Critical regression in UUID handling
/// - **Both slow**: General database layer regression
/// - **Check with**: `db_storage.rs` benchmarks for broader context
///
/// # Limitations
///
/// - Does not measure actual bytes allocated (use dhat/heaptrack for that)
/// - Delete benchmarks include setup overhead (create then delete)
///
/// # Notes for Future
///
/// - If UUID format changes (v7 → v8), regenerate baseline expectations
/// - Do not inline UUID generation into timed region (adds variance)
fn bench_uuid_handling(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("uuid_bench.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with test data
    let test_uuid = Uuid::now_v7();
    let test_key = test_uuid.to_string();
    db.put_by_uuid(
        TEMPLATES_TABLE,
        UuidV7::try_from(test_uuid).expect("Uuid::now_v7 should be v7"),
        &"test_value".to_owned(),
    )
    .expect("put_by_uuid");

    let mut group = c.benchmark_group("uuid_handling");
    group.throughput(Throughput::Elements(1));

    // Optimized: preformatted key get
    group.bench_function("get_preformatted_key", |b| {
        b.iter(|| {
            let id_str = black_box(test_key.as_str());
            db.get::<String, _, _>(TEMPLATES_TABLE, id_str, |archived| {
                black_box(archived);
            })
            .expect("get")
        });
    });

    // Baseline: UUID → String → get
    group.bench_function("get_format_each_time", |b| {
        b.iter(|| {
            let id_str = black_box(test_uuid).to_string();
            db.get::<String, _, _>(TEMPLATES_TABLE, &id_str, |archived| {
                black_box(archived);
            })
            .expect("get")
        });
    });

    // Optimized: UUID-native put
    group.bench_function("put_by_uuid_native", |b| {
        b.iter(|| {
            let uuid = UuidV7::try_from(Uuid::now_v7())
                .expect("Uuid::now_v7 should be v7");
            db.put_by_uuid(
                TEMPLATES_TABLE,
                black_box(uuid),
                &"benchmark_value".to_owned(),
            )
            .expect("put_by_uuid");
        });
    });

    // Baseline: UUID → String → put
    group.bench_function("put_by_uuid_via_string", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            let id_str = uuid.to_string();
            db.put(TEMPLATES_TABLE, &id_str, &"benchmark_value".to_owned())
                .expect("put");
        });
    });

    // Optimized: UUID-native delete
    group.bench_function("delete_by_uuid_native", |b| {
        b.iter(|| {
            let uuid = UuidV7::try_from(Uuid::now_v7())
                .expect("Uuid::now_v7 should be v7");
            db.put_by_uuid(TEMPLATES_TABLE, uuid, &"temp".to_owned())
                .expect("setup");
            let existed = db
                .delete_by_uuid(TEMPLATES_TABLE, black_box(uuid))
                .expect("delete_by_uuid");
            black_box(existed);
        });
    });

    // Baseline: UUID → String → delete
    group.bench_function("delete_by_uuid_via_string", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            let id_str = uuid.to_string();
            db.put(TEMPLATES_TABLE, &id_str, &"temp".to_owned())
                .expect("setup");
            let existed = db.delete(TEMPLATES_TABLE, &id_str).expect("delete");
            black_box(existed);
        });
    });

    group.finish();
}

// ----------------------------------------------------------- //
//                  Key Formatting Strategies                  //
// ----------------------------------------------------------- //

/// Benchmarks key formatting with pre-allocated buffer (optimization tracking).
///
/// # Purpose
///
/// Validates that current optimized key formatting (pre-allocated buffer +
/// `write!()`) performs well. Note: Baseline comparison removed as optimization
/// is now baked into database layer.
///
/// # What is Measured
///
/// - **Metric**: Latency per get/put with string key
/// - **Execution**: Current optimized implementation only
///
/// # Inputs
///
/// - **Keys**: Simple string keys ("key-0050", "key-1000", etc.)
/// - **Pre-population**: 100 keys for get benchmarks
///
/// # Expected Characteristics
///
/// - **get**: ~250 ns (dominated by database lookup, not key formatting)
/// - **put**: ~3-5 ms (dominated by transaction commit)
/// - **Key formatting**: <10 ns of total time (optimization successful)
///
/// # Interpreting Changes
///
/// - **>20% regression**: Investigate database layer (key formatting is minor
///   component)
/// - **Absolute numbers**: Compare with `db_storage.rs` zero-copy benchmarks
///
/// # Notes for Future
///
/// - Consider re-adding baseline (naive `format!()`) if investigating
///   alternative strategies
/// - Key formatting cost is small relative to transaction overhead
fn bench_key_formatting(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("key_bench.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with some data
    for i in 0..100u32 {
        let key = format!("key-{i:04}");
        let value = format!("value-{i}");
        db.put(BENCHMARK_TABLE, &key, &value).expect("put");
    }

    let mut group = c.benchmark_group("key_formatting");
    group.throughput(Throughput::Elements(1));

    // Optimized: Current implementation uses pre-allocated buffer
    group.bench_function("get_with_string_key", |b| {
        b.iter(|| {
            db.get::<String, _, _>(
                BENCHMARK_TABLE,
                black_box("key-0050"),
                |archived| {
                    black_box(archived);
                },
            )
            .expect("get")
        });
    });

    // Optimized: Current implementation uses pre-allocated buffer
    let mut counter = 1000u32;
    group.bench_function("put_with_string_key", |b| {
        b.iter(|| {
            let key = format!("key-{counter:04}");
            counter += 1;
            db.put(BENCHMARK_TABLE, &key, black_box(&"test_value".to_owned()))
                .expect("put");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_uuid_handling, bench_key_formatting,);
criterion_main!(benches);
