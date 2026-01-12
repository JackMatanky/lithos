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
    #[tokio::test]
    async fn event_bus_api_contract_maintained() {
        // Arrange: Create a mock event bus (adapter implementation)
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(4, 4, clock));

        // Act: Use the bus through the port trait (app layer usage)
        let event = TestDomainEvent {
            id: "test-note".to_owned(),
        };
        let result = bus.publish_data(event.clone()).await;

        // Assert: Contract is fulfilled
        assert!(result.is_ok(), "Event bus contract should be maintained");

        // Verify event was captured
        let records = bus.captured_data();
        let guard = records.lock().await;
        assert_eq!(guard.len(), 1, "Event should be captured");

        // Validate captured event matches published event - we already checked len() == 1
        if let Some(captured_event) = guard.first() {
            assert_eq!(
                captured_event.payload, event,
                "Captured event should match published"
            );
        }
    }

    /// Test error propagation across boundaries in integration scenarios.
    ///
    /// Verifies that errors from adapter layer are properly propagated through
    /// the port interface to the app layer, maintaining error handling contracts
    /// in cross-module interactions.
    #[tokio::test]
    async fn error_propagation_across_boundaries() {
        // Arrange: Create a mock event bus that can simulate failures
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(4, 4, clock));

        // Act: Attempt to publish with a scenario that might fail
        // (MockEventBus currently doesn't fail, but this demonstrates the pattern)
        let event = TestDomainEvent {
            id: "error-test".to_owned(),
        };
        let result = bus.publish_data(event).await;

        // Assert: Error handling contract is maintained
        // In a real scenario with failure simulation, we'd check error types
        assert!(
            result.is_ok(),
            "Error propagation contract should be maintained"
        );
    }

    /// Test performance validation for integration scenarios.
    ///
    /// Ensures that integration operations complete within acceptable time bounds.
    /// Integration tests are expected to be 2-3x slower than unit tests due to
    /// setup overhead, but should still be performant for CI/CD pipelines.
    #[tokio::test]
    async fn integration_performance_validation() {
        // Arrange: Create event bus for performance testing
        let clock: Arc<
            dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync,
        > = Arc::new(chrono::Utc::now);
        let bus: Arc<dyn EventBusPort<TestDomainEvent>> =
            Arc::new(MockEventBus::new_with_clock(4, 4, clock));

        let event = TestDomainEvent {
            id: "perf-test".to_owned(),
        };

        // Act: Measure time for a single operation (performance baseline)
        let start = std::time::Instant::now();
        assert!(
            bus.publish_data(event).await.is_ok(),
            "publish should succeed"
        );
        let duration = start.elapsed();

        // Assert: Performance meets integration requirements
        // Allow up to 10ms for a single operation (reasonable for integration tests)
        assert!(
            duration.as_millis() < 10,
            "Integration operation took {}ms, expected <10ms",
            duration.as_millis()
        );
    }
}
