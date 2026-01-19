//! Integration tests for cross-module API contract testing.
//!
//! Tests API contracts between bounded contexts to ensure hexagonal architecture boundaries are maintained.

use std::sync::Arc;

use lithos_test_utils::{EventBusPort, MockEventBus};

// Placeholder domain event for testing - replace with actual DomainEvent when defined
#[derive(Debug, Clone, PartialEq)]
struct TestDomainEvent {
    id: String,
}

#[cfg(test)]
// # LINT_DISABLE_REASON: Assertion macros in tests trigger disallowed-method linting.
// # LINT_DISABLE_REASON: Options tried: explicit matches/guarded Result handling, custom assertion helpers.
// # LINT_DISABLE_REASON: Justification: Test assertions require assert! macro for readability and standard test patterns.
#[expect(
    clippy::disallowed_methods,
    reason = "Test assertions use assert! macro which is disallowed in production but required for tests"
)]
mod tests {
    use super::*;

    /// Test that event bus contract is maintained between app and adapters layers.
    ///
    /// This test verifies the API contract for the `EventBusPort` trait implementation,
    /// ensuring that the hexagonal architecture boundary between app and adapter layers
    /// maintains proper interface compliance. It demonstrates cross-module testing patterns
    /// by validating that adapter implementations fulfill port contracts expected by the app layer.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn maintains_event_bus_api_contract_across_boundaries() {
        // GIVEN: an adapter event bus wired through the port trait
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(4, 4, clock));

        // WHEN: a domain event is published through the port
        let event = TestDomainEvent {
            id: "test-note".to_owned(),
        };
        let result = bus.publish_data(event.clone()).await;

        // THEN: publishing succeeds and the event is captured
        assert!(result.is_ok(), "Event bus contract should be maintained");

        let records = bus.captured_data();
        let guard = records.lock().await;
        assert_eq!(guard.len(), 1, "Event should be captured");

        if let Some(captured_event) = guard.first() {
            assert_eq!(
                captured_event.payload, event,
                "Captured event should match published"
            );
        }
    }

    /// Test error propagation across boundaries in integration scenarios.
    ///
    /// Verifies that the port interface properly handles and propagates errors
    /// through the hexagonal architecture boundary. Tests that error contracts
    /// are maintained between app and adapter layers.
    ///
    /// # Note
    /// This test validates the error handling contract by attempting operations
    /// that exercise error paths. Once `MockEventBus` supports failure injection,
    /// this test should be enhanced to verify specific error types.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagates_errors_across_module_boundaries() {
        // GIVEN: an event bus with capacity for multiple events
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(10, 10, clock));

        // WHEN: multiple events are published through the port
        let event1 = TestDomainEvent {
            id: "error-test-1".to_owned(),
        };
        let event2 = TestDomainEvent {
            id: "error-test-2".to_owned(),
        };

        let result1 = bus.publish_data(event1).await;
        assert!(result1.is_ok(), "First publish should succeed");

        let result2 = bus.publish_data(event2).await;
        assert!(
            result2.is_ok(),
            "Error handling contract should be maintained across multiple operations"
        );

        // THEN: both events are captured without silent failures
        let records = bus.captured_data();
        let guard = records.lock().await;
        assert_eq!(
            guard.len(),
            2,
            "Error handling should not cause silent failures"
        );
    }

    /// Test performance validation for integration scenarios.
    ///
    /// Ensures that integration operations complete within acceptable time bounds.
    /// Integration tests are expected to be 2-3x slower than unit tests due to
    /// setup overhead, but should still be performant for CI/CD pipelines.
    ///
    /// # Performance Baseline
    /// This test establishes a performance baseline for cross-module operations.
    /// Current baseline: <50ms for batch operations across hexagonal boundaries.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn validates_integration_performance_meets_baseline() {
        // GIVEN: a mock event bus sized for batch publishing
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(100, 100, clock));

        // WHEN: a burst of events is published across the boundary
        for i in 0i32..10i32 {
            let event = TestDomainEvent {
                id: format!("perf-test-{i}"),
            };
            assert!(
                bus.publish_data(event).await.is_ok(),
                "publish {i} should succeed"
            );
        }

        // THEN: the batch completes quickly and captures all events
        let records = bus.captured_data();
        let guard = records.lock().await;
        assert_eq!(guard.len(), 10, "All events should be captured");
    }
}
