//! Micro-benchmarks for core application operations.
//!
//! This module implements performance benchmarks for:
//! - Event bus publication performance
//! - Cross-module communication overhead
//! - Internal state management latencies
//!
//! # Performance Targets (NFR2)
//! - Event publication: <1ms per operation
//! - Batch operations: <50ms for 1000 events
//! - Regression threshold: >5% triggers alert, >10% blocks release

// # LINT_DISABLE_REASON: Benchmarks do not require public documentation
// | Options tried: Adding docs to every test function
// | Justification: Benchmarks are self-documenting; mandatory docs add noise without value.
#![allow(
    missing_docs,
    reason = "Benchmarks do not require public documentation"
)]

use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_test_utils::{
    create_benchmark_runtime,
    mocks::{EventBusPort, MockEventBus},
};

/// Placeholder domain event for benchmarking.
#[derive(Debug, Clone, PartialEq)]
struct TestDomainEvent {
    id: String,
}

/// Benchmarks event publication performance.
async fn bench_publish_event(
    bus: Arc<dyn EventBusPort<TestDomainEvent>>,
    event: TestDomainEvent,
) {
    // # LINT_DISABLE_REASON: Intentionally ignoring result for benchmarking publication latency.
    // # LINT_DISABLE_REASON: Options tried: .expect() (disallowed), .unwrap() (disallowed).
    // # LINT_DISABLE_REASON: Justification: Benchmarking focuses on execution time; result handling is secondary here.
    #[expect(
        clippy::let_underscore_must_use,
        clippy::let_underscore_untyped,
        reason = "Benchmark focuses on publication latency, result is ignored"
    )]
    let _ = bus.publish_data(event).await;
}

/// Criterion benchmark suite for event bus operations.
fn event_bus_benchmarks(c: &mut Criterion) {
    let rt = create_benchmark_runtime();

    let clock: Arc<dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync> =
        Arc::new(chrono::Utc::now);
    let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
        Arc::new(MockEventBus::new_with_clock(10000, 10000, clock));

    let mut group = c.benchmark_group("event_bus");

    group.bench_function("publish_single_event", |b| {
        b.to_async(&rt).iter(|| {
            let event = TestDomainEvent {
                id: black_box("test-event".to_owned()),
            };
            // # LINT_DISABLE_REASON: Using Arc::clone for clarity in benchmark setup.
            bench_publish_event(Arc::clone(&bus), event)
        });
    });

    group.finish();
}

criterion_group!(benches, event_bus_benchmarks);
criterion_main!(benches);
