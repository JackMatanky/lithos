# Event-Driven Testing Patterns

Lithos uses a hybrid event bus (ADR 0007) and CQRS-style domain events. This guide summarizes the recommended testing patterns established in ADR 0008.

## Given-When-Then Framework (CQRS/Event Sourcing)

Use the `EventTestFramework` from `lithos-test-utils` to verify aggregate command handling (ADR 0009):

```rust,ignore
use lithos_test_utils::EventTestFramework;

let expected_events = vec![AccountOpened { id: "acct-1".into() }];
let result = EventTestFramework::given(vec![AccountCreated { id: "acct-1".into() }])
    .when(|history| handle_open_account(history))
    .then_expect_events(&expected_events);

assert!(result.is_ok());
```

## Query Handler Testing (Stubs)

Query handlers should be tested against predictable read-model stubs (ADR 0009):

```rust,ignore
#[tokio::test]
async fn test_get_user_query() {
    // Arrange
    let stub_store = Arc::new(StubUserStore::with_users(vec![
        User::new("user-1", "Alice")
    ]));
    let handler = GetUserQueryHandler::new(stub_store);

    // Act
    let query = GetUserQuery { id: "user-1".into() };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert_eq!(result.name, "Alice");
}
```

## Eventual Consistency and Timing

When testing write-to-read model propagation, control time or use retry logic for consistency windows (ADR 0009):

```rust,ignore
#[tokio::test(flavor = "multi_thread")]
async fn test_eventual_consistency() {
    // 1. Execute Command
    service.execute_command(CreateNote { id: "note-1" }).await?;

    // 2. Wait for propagation (or use a helper that retries until success/timeout)
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 3. Verify Read Model
    let note = query_service.get_note("note-1").await?;
    assert!(note.is_some());
}
```

## Observability and Tracing

Use `TestTracingSubscriber` to verify that production code is emitting the expected spans and events:

```rust,ignore
use lithos_test_utils::obs::TestTracingSubscriber;

#[tokio::test]
async fn test_observability() {
    let subscriber = TestTracingSubscriber::install();

    // Perform operation
    perform_complex_operation().await;

    // Verify span was emitted
    subscriber.assert_span_emitted("operation_started");
    subscriber.assert_event_emitted("operation_completed_successfully");
}
```

## Mock Event Bus Patterns

Use `MockEventBus` to isolate publisher and subscriber behavior across the three planes defined in ADR 0007:

- **Data Plane (MPSC)**: For reliable indexing and persistence events.
- **Control Plane (Broadcast)**: For signals like shutdown or configuration changes.
- **State Plane (Watch)**: For latest-state snapshots (e.g., UI state, LSP status).

```rust,ignore
use lithos_test_utils::MockEventBus;

let bus = MockEventBus::new(16, 16);

// Subscribe to specific planes
let mut data_rx = bus.subscribe_data();
let mut control_rx = bus.subscribe_control();
let state_rx = bus.subscribe_state();

// Publish and verify
bus.publish_data(my_event).await.unwrap();
assert_eq!(bus.recorded_data_events().await.len(), 1);
```

## Payload Verification

For domain event contracts, compare serialized payloads:

```rust,ignore
use lithos_test_utils::PayloadAssertion;

let result = PayloadAssertion::verify(&expected_event, &actual_event);
assert!(result.is_ok());
```

## Ordering and Timing Verification

Use `EventRecord` and `SequenceAssertion` to validate ordering. For timing-sensitive checks, prefer deterministic counters over wall-clock assertions.

`TimingAssertion` provides helpers to assert monotonic timestamps and bounded spans when ordering by time matters.

```rust,ignore
use lithos_test_utils::{EventRecord, TimingAssertion};
use chrono::Duration;

let records = vec![
    EventRecord::with_timestamp(1, fixed_timestamp(0), event_a),
    EventRecord::with_timestamp(2, fixed_timestamp(1), event_b),
];

TimingAssertion::verify_non_decreasing(&records)?;
TimingAssertion::verify_max_span(&records, Duration::seconds(5))?;
```

## DDD Compliance Checklist

When validating domain-driven design event workflows:

- Capture event-storming outcomes as a checklist for expected domain events.
- Add contract tests that serialize and compare event payloads.
- Include integration tests that verify event flows across multiple planes.

## Integration Event Flow Testing

Use the mock bus to validate that events traverse between planes as expected:

```rust,ignore
let bus = MockEventBus::new_with_clock(4, 4, fixed_clock());
let mut receiver = bus.subscribe_control();

bus.publish_data(event.clone()).await?;
// Relay logic should publish to control plane.
let delivered = receiver.recv().await?;
assert_eq!(delivered, event);
```

## Malformed Event Handling

Subscriber tests should explicitly handle malformed payloads without panicking:

```rust,ignore
let malformed = serde_json::json!({"account_id": 10, "version": "oops"});
let result: Result<AccountEvent, _> = serde_json::from_value(malformed);
assert!(result.is_err());
```

## Async Event Testing Guidance

- Use `#[tokio::test(flavor = "multi_thread")]` or `async_test!` to surface race conditions.
- Use `tokio::time::timeout` to prevent hanging tests.
- For concurrent publishing, prefer bounded channels and verify ordering with `SequenceAssertion`.

## Integration Test Coverage

When implementing event-driven stories, include:

- Unit tests for event utilities and mocks.
- Integration tests for publisher and subscriber flows.
- Error-path tests for malformed payloads or subscription failures.
