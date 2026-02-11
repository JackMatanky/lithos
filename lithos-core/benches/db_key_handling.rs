//! Database key and UUID formatting strategy benchmarks.
//!
//! Measures performance of different approaches to database key construction
//! and UUID handling, tracking P0 optimization Tasks 1 & 2 from
//! `TODO_ALLOCATIONS.md`.
//!
//! # Benchmarks
//!
//! ## UUID Handling (Task 2)
//! - **UUID-native methods**: Direct UUID → key formatting (`get_by_uuid`,
//!   `put_by_uuid`)
//! - **String conversion**: UUID → String → key formatting (baseline)
//!
//! ## Key Formatting (Task 1)
//! - **Optimized**: Pre-allocated buffer with `write!()` macro
//! - **Baseline**: `format!()` macro allocation per operation
//!
//! # Expected Results
//!
//! UUID-native methods should show:
//! - 7-9% faster than string conversion approach
//! - 36 bytes saved per operation (UUID string allocation)
//!
//! Key formatting optimization should show:
//! - Reduced allocation overhead
//! - Better performance in high-frequency operations
//!
//! # Cross-Reference
//!
//! See `db_storage.rs` for general storage infrastructure performance.
//! See `docs/benchmarks/BASELINE.md` for detailed optimization impact analysis.
//!
//! # Safety
//! Benchmark code uses unwrap/expect for simplicity.

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::db::Database;
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================================
// UUID Handling Strategies (P0 Task 2)
// ============================================================================

/// Benchmarks UUID-native database methods vs string conversion approach.
///
/// # Optimization (Task 2)
/// Add UUID-native methods (`get_by_uuid`, `put_by_uuid`, etc.) that format
/// UUIDs directly into database keys, avoiding intermediate string allocation.
fn bench_uuid_handling(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("uuid_bench.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with test data
    let test_uuid = Uuid::now_v7();
    db.put_by_uuid("templates", test_uuid, &"test_value".to_owned())
        .expect("put");

    let mut group = c.benchmark_group("uuid_handling");
    group.throughput(Throughput::Elements(1));

    // Optimized: UUID-native get
    group.bench_function("get_by_uuid_native", |b| {
        b.iter(|| {
            db.get_by_uuid::<String, _, _>(
                "templates",
                black_box(test_uuid),
                |archived| {
                    black_box(archived);
                },
            )
            .expect("get")
        });
    });

    // Baseline: UUID → String → get
    group.bench_function("get_by_uuid_via_string", |b| {
        b.iter(|| {
            let id_str = black_box(test_uuid).to_string();
            db.get::<String, _, _>("templates", &id_str, |archived| {
                black_box(archived);
            })
            .expect("get")
        });
    });

    // Optimized: UUID-native put
    group.bench_function("put_by_uuid_native", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            db.put_by_uuid(
                "templates",
                black_box(uuid),
                &"benchmark_value".to_owned(),
            )
            .expect("put");
        });
    });

    // Baseline: UUID → String → put
    group.bench_function("put_by_uuid_via_string", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            let id_str = uuid.to_string();
            db.put("templates", &id_str, &"benchmark_value".to_owned())
                .expect("put");
        });
    });

    // Optimized: UUID-native delete
    group.bench_function("delete_by_uuid_native", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            db.put_by_uuid("templates", uuid, &"temp".to_owned())
                .expect("setup");
            let existed = db
                .delete_by_uuid("templates", black_box(uuid))
                .expect("delete");
            black_box(existed);
        });
    });

    // Baseline: UUID → String → delete
    group.bench_function("delete_by_uuid_via_string", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            let id_str = uuid.to_string();
            db.put("templates", &id_str, &"temp".to_owned()).expect("setup");
            let existed = db.delete("templates", &id_str).expect("delete");
            black_box(existed);
        });
    });

    group.finish();
}

// ============================================================================
// Key Formatting Strategies (P0 Task 1)
// ============================================================================

/// Benchmarks database key formatting strategies.
///
/// # Optimization (Task 1)
/// Replace `format!("{table}:{key}")` with pre-allocated buffer and `write!()`
/// to avoid allocation on every database operation.
fn bench_key_formatting(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("key_bench.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with some data
    for i in 0..100u32 {
        let key = format!("key-{i:04}");
        let value = format!("value-{i}");
        db.put("benchmark", &key, &value).expect("put");
    }

    let mut group = c.benchmark_group("key_formatting");
    group.throughput(Throughput::Elements(1));

    // Optimized: Current implementation uses pre-allocated buffer
    group.bench_function("get_with_string_key", |b| {
        b.iter(|| {
            db.get::<String, _, _>(
                "benchmark",
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
            db.put("benchmark", &key, black_box(&"test_value".to_owned()))
                .expect("put");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_uuid_handling, bench_key_formatting,);
criterion_main!(benches);
