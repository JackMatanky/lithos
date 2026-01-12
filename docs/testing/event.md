# Event-Driven Testing Patterns

Lithos uses a hybrid event bus (ADR 0007) and CQRS-style domain events. This guide summarizes the recommended testing patterns established in ADR 0008.

## Given-When-Then Framework (CQRS/Event Sourcing)

Use the `EventTestFramework` from `lithos-test-utils` to verify aggregate command handling:

```rust,ignore
use lithos_test_utils::EventTestFramework;

let expected_events = vec![AccountOpened { id: "acct-1".into() }];
let result = EventTestFramework::given(vec![AccountCreated { id: "acct-1".into() }])
    .when(|history| handle_open_account(history))
    .then_expect_events(&expected_events);

assert!(result.is_ok());
```

## Mock Event Bus Patterns

Use `MockEventBus` to isolate publisher and subscriber behavior:

- **Data Plane (MPSC)**: verify reliable delivery and ordering for indexer events.
- **Control Plane (Broadcast)**: verify signal distribution (e.g., shutdown).
- **State Plane (Watch)**: verify latest-state notifications for LSP/UI state.

```rust,ignore
use lithos_test_utils::MockEventBus;

let bus = MockEventBus::new(16, 16);
let mut receiver = bus.subscribe_control();
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
