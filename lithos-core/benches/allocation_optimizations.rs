//! Allocation optimization benchmarks tracking P0/P1 performance improvements.
//!
//! This benchmark suite specifically tracks the optimizations documented in
//! `TODO_ALLOCATIONS.md`. Each benchmark corresponds to a specific optimization
//! task and measures the impact of that optimization.
//!
//! # Benchmark Groups
//!
//! ## P0 Optimizations (Critical Hot Paths)
//! - **`database_operations`**: Tests key formatting & UUID operations (Tasks 1
//!   & 2)
//! - **`numeric_formatting`**: Tests itoa/ryu vs `.to_string()` (Task 5)
//!
//! ## P1 Optimizations (API Ergonomics)
//! - **`constructor_apis`**: Tests `&str` vs `String` constructor parameters
//!   (Task 6)
//!
//! # Expected Results
//!
//! All optimizations should show:
//! - Reduced execution time (typically 10-30% improvement)
//! - Lower memory allocations (measurable via profiling tools)
//! - Better cache locality (measurable in aggregate operations)
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
    clippy::float_arithmetic,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use std::collections::HashMap;

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{
    db::Database,
    schema::{
        aggregate::SchemaName, property::PropertyName, property_spec::DateSpec,
    },
    template::aggregate::Template,
};
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================================
// P0 Task 1 & 2: Database Operations (Key Formatting + UUID)
// ============================================================================

/// Benchmarks the optimized database operations including key formatting
/// and UUID-native methods.
///
/// # Optimizations
/// - Task 1: Replace `format!("{table}:{key}")` with pre-allocated buffers
/// - Task 2: Add UUID-native methods (`get_by_uuid`, `put_by_uuid`, etc.)
fn bench_database_operations(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("db_ops.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with some data
    for i in 0..100u32 {
        let key = format!("key-{i:04}");
        let value = format!("value-{i}");
        db.put("benchmark", &key, &value).expect("put");
    }

    let mut group = c.benchmark_group("database_operations");
    group.throughput(Throughput::Elements(1));

    // Test optimized get operation (Task 1: key formatting)
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

    // Test optimized put operation (Task 1: key formatting)
    let mut counter = 1000u32;
    group.bench_function("put_with_string_key", |b| {
        b.iter(|| {
            let key = format!("key-{counter:04}");
            counter += 1;
            db.put("benchmark", &key, black_box(&"test_value".to_owned()))
                .expect("put");
        });
    });

    // Prepare UUID-keyed data for Task 2 benchmarks
    let test_uuid = Uuid::now_v7();
    db.put_by_uuid("templates", test_uuid, &"test_template".to_owned())
        .expect("put");

    // Test UUID-native get (Task 2: optimized)
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

    // Test UUID-via-string get (Task 2: baseline for comparison)
    group.bench_function("get_by_uuid_via_string", |b| {
        b.iter(|| {
            let id_str = black_box(test_uuid).to_string();
            db.get::<String, _, _>("templates", &id_str, |archived| {
                black_box(archived);
            })
            .expect("get")
        });
    });

    // Test UUID-native put (Task 2: optimized)
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

    // Test UUID-via-string put (Task 2: baseline for comparison)
    group.bench_function("put_by_uuid_via_string", |b| {
        b.iter(|| {
            let uuid = Uuid::now_v7();
            let id_str = uuid.to_string();
            db.put("templates", &id_str, &"benchmark_value".to_owned())
                .expect("put");
        });
    });

    group.finish();
}

// ============================================================================
// P0 Task 5: Numeric Formatting
// ============================================================================

/// Benchmarks zero-allocation numeric formatting using `itoa`/`ryu` vs
/// `.to_string()`.
///
/// # Optimization
/// Replace `.to_string()` with `itoa::Buffer` for integers
/// and `ryu::Buffer` for floats in query operations.
fn bench_numeric_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_formatting");
    group.throughput(Throughput::Elements(100));

    // Benchmark optimized integer formatting (itoa) - Task 5
    group.bench_function("format_integers_itoa", |b| {
        b.iter(|| {
            let mut buffer = itoa::Buffer::new();
            for i in 0i64..100i64 {
                let s = buffer.format(black_box(i));
                black_box(s);
            }
        });
    });

    // Benchmark naive integer formatting (.to_string()) for comparison
    group.bench_function("format_integers_to_string", |b| {
        b.iter(|| {
            for i in 0i64..100i64 {
                let s = black_box(i).to_string();
                black_box(s);
            }
        });
    });

    // Benchmark optimized float formatting (ryu) - Task 5
    group.bench_function("format_floats_ryu", |b| {
        b.iter(|| {
            let mut buffer = ryu::Buffer::new();
            for i in 0u32..100u32 {
                let f = f64::from(i) + 0.5f64;
                let s = buffer.format(black_box(f));
                black_box(s);
            }
        });
    });

    // Benchmark naive float formatting (.to_string()) for comparison
    group.bench_function("format_floats_to_string", |b| {
        b.iter(|| {
            for i in 0u32..100u32 {
                let f = black_box(f64::from(i) + 0.5f64);
                let s = f.to_string();
                black_box(s);
            }
        });
    });

    group.finish();
}

// ============================================================================
// P1 Task 6: Constructor APIs
// ============================================================================

/// Benchmarks constructor APIs using `&str` vs `String` parameters.
///
/// # Optimization
/// Change constructors from `new(name: String)` to
/// `new(name: &str)` to avoid forcing caller allocations.
fn bench_constructor_apis(c: &mut Criterion) {
    let mut group = c.benchmark_group("constructor_apis");
    group.throughput(Throughput::Elements(1));

    // Benchmark SchemaName construction (optimized with `&str`) - Task 6
    group.bench_function("schema_name_from_str", |b| {
        b.iter(|| SchemaName::new(black_box("my-schema")).expect("valid name"));
    });

    // Benchmark SchemaName from String (shows caller cost if we hadn't
    // optimized)
    group.bench_function("schema_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-schema").to_owned();
            SchemaName::new(&owned).expect("valid name")
        });
    });

    // Benchmark PropertyName construction (optimized with `&str`) - Task 6
    group.bench_function("property_name_from_str", |b| {
        b.iter(|| {
            PropertyName::new(black_box("my-property")).expect("valid name")
        });
    });

    // Benchmark PropertyName from String (shows caller cost if we hadn't
    // optimized)
    group.bench_function("property_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-property").to_owned();
            PropertyName::new(&owned).expect("valid name")
        });
    });

    // Benchmark DateSpec construction (optimized with `&str`) - Task 6
    group.bench_function("date_spec_from_str", |b| {
        b.iter(|| {
            DateSpec::try_new(black_box("%Y-%m-%d")).expect("valid format")
        });
    });

    // Benchmark DateSpec from String (shows caller cost if we hadn't optimized)
    group.bench_function("date_spec_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("%Y-%m-%d").to_owned();
            DateSpec::try_new(&owned).expect("valid format")
        });
    });

    // Benchmark Template construction (optimized with `&str`) - Task 6
    group.bench_function("template_from_str", |b| {
        b.iter(|| {
            Template::new(
                black_box("my-template"),
                "Content: {{var}}".to_owned(),
                HashMap::new(),
                None,
                lithos_core::template::aggregate::Metadata::default(),
            )
            .expect("valid template")
        });
    });

    // Benchmark Template from String (shows caller cost if we hadn't optimized)
    group.bench_function("template_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-template").to_owned();
            Template::new(
                &owned,
                "Content: {{var}}".to_owned(),
                HashMap::new(),
                None,
                lithos_core::template::aggregate::Metadata::default(),
            )
            .expect("valid template")
        });
    });

    group.finish();
}

// ============================================================================
// Aggregate Workflow Benchmark
// ============================================================================

/// Benchmarks a complete workflow that exercises multiple optimizations
/// to measure cumulative impact.
fn bench_aggregate_workflow(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("workflow.db");
    let db = Database::open(&db_path).expect("open database");

    let mut group = c.benchmark_group("aggregate_workflow");
    group.throughput(Throughput::Elements(1));

    // Workflow combining Tasks 1, 2, 5, and 6
    group.bench_function("complete_optimized_workflow", |b| {
        b.iter(|| {
            // Task 6: Optimized constructors (&str)
            let schema_name =
                SchemaName::new("workflow-schema").expect("valid name");
            let prop_name =
                PropertyName::new("priority").expect("valid property");
            let date_spec =
                DateSpec::try_new("%Y-%m-%d").expect("valid format");

            // Task 5: Optimized numeric formatting (used in queries)
            let mut int_buffer = itoa::Buffer::new();
            let priority_str = int_buffer.format(black_box(42i64));
            black_box(priority_str);

            let mut float_buffer = ryu::Buffer::new();
            let score_str = float_buffer.format(black_box(2.5f64));
            black_box(score_str);

            // Task 2: UUID-native database operations
            let template_uuid = Uuid::now_v7();
            let template = Template::new(
                "workflow-template",
                "Content".to_owned(),
                HashMap::new(),
                None,
                lithos_core::template::aggregate::Metadata::default(),
            )
            .expect("valid template");

            db.put_by_uuid("templates", template_uuid, &template).expect("put");

            let retrieved: Option<Template> =
                db.get_owned_by_uuid("templates", template_uuid).expect("get");

            black_box((schema_name, prop_name, date_spec, retrieved));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_database_operations,
    bench_numeric_formatting,
    bench_constructor_apis,
    bench_aggregate_workflow,
);
criterion_main!(benches);
