//! String and numeric formatting API benchmarks.
//!
//! Measures performance of different approaches to string construction,
//! numeric formatting, and constructor API design, tracking P0 Task 5 and
//! P1 Task 6 from `TODO_ALLOCATIONS.md`.
//!
//! # Benchmarks
//!
//! ## Numeric Formatting (P0 Task 5)
//! - **Integer formatting**: `itoa::Buffer` (zero-allocation) vs `.to_string()`
//! - **Float formatting**: `ryu::Buffer` (zero-allocation) vs `.to_string()`
//!
//! ## Constructor APIs (P1 Task 6)
//! - **`&str` parameters**: Caller controls allocation (optimized)
//! - **`String` parameters**: Forced allocation at call site (baseline)
//!
//! ## Aggregate Workflow
//! - Combined impact of all optimizations in realistic usage
//!
//! # Expected Results
//!
//! Numeric formatting should show:
//! - `itoa`: ~9.7x faster than `.to_string()` for integers
//! - `ryu`: ~17% faster than `.to_string()` for floats
//!
//! Constructor APIs should show:
//! - `&str`: 30-32% faster than forced `String` allocation
//! - Better ergonomics (caller chooses when to allocate)
//!
//! # Cross-Reference
//!
//! See `db_key_handling.rs` for database-specific string handling.
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
// Numeric Formatting (P0 Task 5)
// ============================================================================

/// Benchmarks zero-allocation numeric formatting using `itoa`/`ryu` vs
/// `.to_string()`.
///
/// # Optimization (Task 5)
/// Replace `.to_string()` with `itoa::Buffer` for integers and `ryu::Buffer`
/// for floats in query operations and display formatting.
fn bench_numeric_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_formatting");
    group.throughput(Throughput::Elements(100));

    // Optimized: Integer formatting with itoa
    group.bench_function("format_integers_itoa", |b| {
        b.iter(|| {
            let mut buffer = itoa::Buffer::new();
            for i in 0i64..100i64 {
                let s = buffer.format(black_box(i));
                black_box(s);
            }
        });
    });

    // Baseline: Integer formatting with .to_string()
    group.bench_function("format_integers_to_string", |b| {
        b.iter(|| {
            for i in 0i64..100i64 {
                let s = black_box(i).to_string();
                black_box(s);
            }
        });
    });

    // Optimized: Float formatting with ryu
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

    // Baseline: Float formatting with .to_string()
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
// Constructor APIs (P1 Task 6)
// ============================================================================

/// Benchmarks constructor APIs using `&str` vs `String` parameters.
///
/// # Optimization (Task 6)
/// Change constructors from `new(name: String)` to `new(name: &str)` to
/// avoid forcing caller allocations. Follows Rust idiom of accepting
/// borrowed parameters.
fn bench_constructor_apis(c: &mut Criterion) {
    let mut group = c.benchmark_group("constructor_apis");
    group.throughput(Throughput::Elements(1));

    // Optimized: SchemaName with &str
    group.bench_function("schema_name_from_str", |b| {
        b.iter(|| SchemaName::new(black_box("my-schema")).expect("valid name"));
    });

    // Baseline: SchemaName from String (shows forced allocation cost)
    group.bench_function("schema_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-schema").to_owned();
            SchemaName::new(&owned).expect("valid name")
        });
    });

    // Optimized: PropertyName with &str
    group.bench_function("property_name_from_str", |b| {
        b.iter(|| {
            PropertyName::new(black_box("my-property")).expect("valid name")
        });
    });

    // Baseline: PropertyName from String
    group.bench_function("property_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-property").to_owned();
            PropertyName::new(&owned).expect("valid name")
        });
    });

    // Optimized: DateSpec with &str
    group.bench_function("date_spec_from_str", |b| {
        b.iter(|| {
            DateSpec::try_new(black_box("%Y-%m-%d")).expect("valid format")
        });
    });

    // Baseline: DateSpec from String
    group.bench_function("date_spec_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("%Y-%m-%d").to_owned();
            DateSpec::try_new(&owned).expect("valid format")
        });
    });

    // Optimized: Template with &str
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

    // Baseline: Template from String
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
// Aggregate Workflow
// ============================================================================

/// Benchmarks a complete workflow that exercises multiple optimizations
/// to measure cumulative impact.
///
/// Combines:
/// - Task 5: Numeric formatting (itoa/ryu)
/// - Task 6: Constructor APIs (&str parameters)
/// - Task 2: UUID-native database operations
fn bench_aggregate_workflow(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("workflow.db");
    let db = Database::open(&db_path).expect("open database");

    let mut group = c.benchmark_group("aggregate_workflow");
    group.throughput(Throughput::Elements(1));

    group.bench_function("complete_optimized_workflow", |b| {
        b.iter(|| {
            // Task 6: Optimized constructors (&str)
            let schema_name =
                SchemaName::new("workflow-schema").expect("valid name");
            let prop_name =
                PropertyName::new("priority").expect("valid property");
            let date_spec =
                DateSpec::try_new("%Y-%m-%d").expect("valid format");

            // Task 5: Optimized numeric formatting
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
    bench_numeric_formatting,
    bench_constructor_apis,
    bench_aggregate_workflow,
);
criterion_main!(benches);
