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

## Async Event Testing Guidance

- Use `#[tokio::test(flavor = "multi_thread")]` or `async_test!` to surface race conditions.
- Use `tokio::time::timeout` to prevent hanging tests.
- For concurrent publishing, prefer bounded channels and verify ordering with `SequenceAssertion`.

## Integration Test Coverage

When implementing event-driven stories, include:

- Unit tests for event utilities and mocks.
- Integration tests for publisher and subscriber flows.
- Error-path tests for malformed payloads or subscription failures.
