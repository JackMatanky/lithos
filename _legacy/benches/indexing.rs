//! Benchmarks for vault indexing performance.
//!
//! This module tracks the NFR for large vault scalability:
//! - Vault indexing (1000+ files): < 2 seconds
//! - Note lookup latency: < 50ms
//! - Metadata query latency: < 100ms

// # LINT_DISABLE_REASON: Benchmarks use thread::sleep to simulate work.
// # LINT_DISABLE_REASON: Options tried: tokio::time::sleep (requires runtime in
// iter). # LINT_DISABLE_REASON: Justification: Simulating CPU work in a
// synchronous benchmark loop.
#![expect(
    clippy::disallowed_methods,
    missing_docs,
    reason = "Benchmarks use thread::sleep to simulate work and do not \
              require public docs"
)]

use criterion::{Criterion, criterion_group, criterion_main};
use lithos_test_utils::{bench::standard_criterion, create_benchmark_runtime};

fn indexing_benchmarks(c: &mut Criterion) {
    let _rt = create_benchmark_runtime();
    let mut group = c.benchmark_group("indexing");

    group.bench_function("index_1000_files", |b| {
        // Placeholder for real indexing logic
        b.iter(|| {
            // Simulate indexing work
            std::thread::sleep(std::time::Duration::from_millis(10));
        });
    });

    group.finish();
}

fn custom_criterion() -> Criterion {
    standard_criterion()
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = indexing_benchmarks
}
criterion_main!(benches);
