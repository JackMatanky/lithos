//! Performance benchmarking utilities for Lithos.
//!
//! Provides standardized Criterion-based benchmarking infrastructure,
//! including async runtime setup and performance regression thresholds.

use criterion::Criterion;

/// Standardized benchmarking configuration for Lithos.
pub fn standard_criterion() -> Criterion {
    Criterion::default()
        .configure_from_args()
        .sample_size(100)
        .noise_threshold(performance_gates::WARNING_THRESHOLD)
}

/// Helper to create a multi-threaded Tokio runtime for benchmarking.
pub fn create_benchmark_runtime() -> tokio::runtime::Runtime {
    // # LINT_DISABLE_REASON: Benchmarks use expect() for runtime
    // initialization. # LINT_DISABLE_REASON: Options tried: propagating
    // errors. # LINT_DISABLE_REASON: Justification: If runtime fails,
    // benchmark cannot proceed; panic is acceptable here.
    #[expect(
        clippy::expect_used,
        reason = "Benchmark runtime initialization requires expect()"
    )]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime for benchmarking")
}

/// Performance regression gates and thresholds.
pub mod performance_gates {
    /// Regression threshold that triggers a warning/alert (5%).
    pub const WARNING_THRESHOLD: f64 = 0.05;
    /// Regression threshold that blocks a release (10%).
    pub const BLOCKING_THRESHOLD: f64 = 0.10;
}
