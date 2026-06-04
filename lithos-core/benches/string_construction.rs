//! String construction and API-level formatting benchmarks.
//!
//! # Summary
//!
//! Tracks allocation optimizations for string construction and API design by
//! comparing zero-allocation numeric formatting and ergonomic constructor APIs
//! against baseline string-allocation approaches.
//!
//! # Motivation
//!
//! Hot-path profiling identified `.to_string()` calls in query formatting and
//! constructor APIs forcing unnecessary allocations at call sites. Allocation
//! optimizations introduced `itoa`/`ryu` buffers for stack-based formatting and
//! changed constructors from `String` to `&str` parameters. This suite
//! validates improvements and tracks API quality.
//!
//! # Scope
//!
//! **Included**:
//! - Integer formatting: `itoa::Buffer` vs `.to_string()`
//! - Float formatting: `ryu::Buffer` vs `.to_string()`
//! - Constructor APIs: `&str` parameters vs `String` (forced allocation)
//!
//! **Excluded**:
//! - Database key formatting (see `db_key_handling.rs`)
//! - Storage-layer operations (see `db_storage.rs`)
//! - Actual memory allocation measurement (uses latency as proxy)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Isolated formatting operations
//! - **Comparative**: Each optimization paired with baseline
//! - **Single-threaded**: No concurrent formatting scenarios
//! - **Hot-loop**: 100 iterations per benchmark to amortize setup cost
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (default configuration)
//! - **Throughput**: Reported as elements/second (100 operations per iteration)
//! - **Black-boxing**: All formatted strings and inputs passed through
//!   `black_box()`
//! - **Buffer reuse**: Realistic pattern (single buffer reused across loop)
//!
//! # Input Model
//!
//! - **Integers**: Range 0-99 (covers 1-2 digit numbers, typical for queries)
//! - **Floats**: Derived from integers + 0.5 (simple fractional values)
//! - **Constructor strings**: Fixed literals ("my-schema", "my-property", etc.)
//! - **Determinism**: Fixed inputs ensure reproducible results
//! - **Sizes**: Small values (< 20 bytes) to isolate formatting overhead
//!
//! # Controls and Fairness
//!
//! - **Same inputs**: Optimized/baseline pairs format identical values
//! - **Same work**: Constructors validate and allocate same structures
//! - **No pre-allocation**: Baseline uses `.to_string()` fresh each time
//! - **Realistic usage**: Buffer reuse pattern matches production code
//!
//! # Expected Characteristics
//!
//! Based on measured baseline performance (2026-02-11, Apple M3 Max):
//!
//! **Integer Formatting (100 items)**:
//! - **`itoa::Buffer`**: ~135 ns (742 Melem/s) - stack-based, zero-allocation
//! - **`.to_string()`**: ~1.31 µs (76 Melem/s) - heap allocation per call
//! - **Speedup**: ~9.7x faster with itoa
//! - **Bottleneck**: Heap allocation overhead in baseline, not formatting logic
//!
//! **Float Formatting (100 items)**:
//! - **`ryu::Buffer`**: ~2.76 µs (36 Melem/s) - stack-based, zero-allocation
//! - **`.to_string()`**: ~3.23 µs (31 Melem/s) - heap allocation per call
//! - **Speedup**: ~17% faster with ryu (smaller gain than integers)
//! - **Bottleneck**: Float formatting complexity reduces allocation impact
//!
//! **Constructor APIs** (per call):
//! - **`SchemaName::new(&str)`**: ~22 ns vs ~33 ns (String) → 32% faster
//! - **`PropertyName::new(&str)`**: ~25 ns vs ~36 ns (String) → 31% faster
//! - **`DateSpec::try_new(&str)`**: ~11 ns vs ~11 ns (String) → ~3% faster
//!   (validation dominates)
//! - **Trend**: Smaller constructors show larger relative benefit from `&str`

//! # Interpreting Results
//!
//! **Expected improvements (from RESULTS.md)**:
//! - `itoa`: 9.7x faster than `.to_string()` for integers
//! - `ryu`: 17% faster than `.to_string()` for floats
//! - `&str` constructors: 30-32% faster than `String` (forced allocation)
//!
//! **Meaningful changes**:
//! - **Ratio approaching 1.0**: Optimization benefit eroding
//! - **Optimized slower**: Critical regression in formatting library
//! - **>10% change**: Investigate compiler optimization changes
//!
//! **Valid comparisons**:
//! - Within-group ratios: Highly stable (formatting is deterministic)
//! - Across machines: Absolute numbers vary, ratios stable
//!
//! **Noise sources**:
//! - CPU frequency scaling (lock CPU frequency for precision)
//! - Background allocator activity (minimal for stack-based formatting)
//!
//! # Reporting and Workflow
//!
//! - **Development**: Run when changing numeric formatting or constructor APIs
//! - **Validation**: Check ratios after library updates (itoa, ryu upgrades)
//! - **Documentation**: Update RESULTS.md if ratios shift significantly
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Constructor signatures change (new parameters, different types)
//! - Numeric formatting strategy changes (different libraries, inlining)
//! - New domain types added requiring similar optimizations
//!
//! **Adding benchmarks**:
//! - Always pair optimization with baseline for comparison
//! - Use consistent naming: `<operation>_<strategy>` (e.g.,
//!   `format_integers_itoa`)
//! - Document expected improvement in per-bench comment
//!
//! # Known Limitations
//!
//! - **No allocation measurement**: Uses latency as proxy (consider dhat for
//!   actual measurement)
//! - **Small input range**: Does not test large numbers (>6 digits) or long
//!   floats
//! - **No Unicode**: String literals are ASCII-only
//! - **No error paths**: Constructor benchmarks use valid inputs only
//!
//! # Benchmark Index
//!
//! | Group                  | Expected Time (Optimized) | Focus                                            |
//! | ---------------------- | ------------------------- | ------------------------------------------------ |
//! | `numeric_formatting`   | ~135ns (int), ~2.76µs (float) | itoa/ryu vs .`to_string()` (integers and floats) |
//! | `constructor_apis`     | ~22-25ns                   | &str vs String parameters (domain constructors)  |
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
    clippy::float_arithmetic,
    clippy::excessive_nesting,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use lithos_core::schema::{
    identifier::SchemaName, property::PropertyName, property_spec::DateSpec,
};

// ----------------------------------------------------------- //
//                     Numeric Formatting                      //
// ----------------------------------------------------------- //

/// Benchmarks numeric formatting: itoa/ryu vs .`to_string()` (optimization
/// tracking).
///
/// # Purpose
///
/// Validates that stack-based `itoa::Buffer` and `ryu::Buffer` outperform
/// heap-allocating `.to_string()` for query parameter formatting.
///
/// # What is Measured
///
/// - **Throughput**: 100 conversions per iteration (reported as elem/sec)
/// - **Variants**: itoa (integers), ryu (floats) vs .`to_string()` baselines
/// - **Input**: Range 0-99 (1-2 digit numbers, typical for task
///   priorities/dates)
///
/// # Expected Characteristics
///
/// - **itoa**: ~740 Melem/s (9.7x faster than .`to_string()` per RESULTS.md)
/// - **ryu**: ~36 Melem/s (17% faster than .`to_string()` per RESULTS.md)
/// - **Ratio stability**: Very stable (formatting is deterministic)
///
/// # Interpreting Changes
///
/// - **itoa approaching .`to_string()`**: Compiler may be optimizing
///   .`to_string()` better
/// - **ryu slower than .`to_string()`**: Check ryu library version or compiler
///   opts
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

// ----------------------------------------------------------- //
//                      Constructor Apis                       //
// ----------------------------------------------------------- //

/// Benchmarks constructor APIs: &str vs String parameters (optimization
/// tracking).
///
/// # Purpose
///
/// Validates that `&str` parameters avoid forcing allocations at call sites,
/// improving both performance and ergonomics per Rust idioms.
///
/// # What is Measured
///
/// - **Latency**: Single constructor call per iteration
/// - **Types**: `SchemaName`, `PropertyName`, `DateSpec`
/// - **Comparison**: Optimized (&str) vs baseline (allocate then pass)
///
/// # Expected Characteristics
///
/// - **&str variants**: 10-25 ns per call (small types)
/// - **String variants**: 30-36 ns per call (adds .`to_owned()` overhead)
/// - **Improvement**: 30-32% faster for &str (from RESULTS.md)
///
/// # Notes for Future
///
/// - Small types (`SchemaName`, `PropertyName`) show clearest benefit
fn bench_constructor_apis(c: &mut Criterion) {
    let mut group = c.benchmark_group("constructor_apis");
    group.throughput(Throughput::Elements(1));

    // Optimized: SchemaName with &str
    group.bench_function("schema_name_from_str", |b| {
        b.iter(|| {
            SchemaName::try_new(black_box("my-schema")).expect("valid name")
        });
    });

    // Baseline: SchemaName from String (shows forced allocation cost)
    group.bench_function("schema_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-schema").to_owned();
            SchemaName::try_new(&owned).expect("valid name")
        });
    });

    // Optimized: PropertyName with &str
    group.bench_function("property_name_from_str", |b| {
        b.iter(|| {
            PropertyName::try_new(black_box("my-property")).expect("valid name")
        });
    });

    // Baseline: PropertyName from String
    group.bench_function("property_name_from_owned_string", |b| {
        b.iter(|| {
            let owned = black_box("my-property").to_owned();
            PropertyName::try_new(&owned).expect("valid name")
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

    group.finish();
}

criterion_group!(benches, bench_numeric_formatting, bench_constructor_apis,);
criterion_main!(benches);
