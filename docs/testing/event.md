# Event-Driven Testing Guidelines

Tactical specification for testing the Lithos hybrid event bus and domain events.

## 1. Key Principles

### Hybrid Plane Awareness
Lithos uses a "Three Plane" event architecture (ADR 0007). Tests must be explicit about which plane they are verifying.
- **Data Plane (MPSC)**: Guaranteed delivery, strict ordering. Used for Indexing, Persistence. Test for *completeness* and *order*.
- **Control Plane (Broadcast)**: Fire-and-forget, ephemeral. Used for Shutdown, Cache Invalidation. Test for *reachability* and *responsiveness*.
- **State Plane (Watch)**: Last-value-wins. Used for UI updates, Status indicators. Test for *eventual correctness* (intermediate states may be skipped).

### Contract Integrity
Events are the public API of the domain.
- **Serialization**: Verify that events serialize/deserialize correctly (JSON/rkyv).
- **Schema Evolution**: Verify that adding a field doesn't break existing consumers (backward compatibility). This is critical for long-lived stores.

### Eventual Consistency
- **No Sleep**: Never use `sleep(50ms)` to wait for a read model update. It's flaky.
- **Polling**: Use `EventualConsistencyTester` which polls with a timeout.
- **Causality**: Verify that Event A *causes* Event B, not just that they both happened.

## 2. Golden Rules

1.  **Mock the Bus**: Use `MockEventBus` to isolate publishers from subscribers. Never rely on the real system bus for unit tests; it introduces unnecessary concurrency and setup overhead.
2.  **Verify Ordering**: For sequential processes (like indexing), use `SequenceAssertion` to verify causal event chains. A followed by B is different from B followed by A.
3.  **Failure Atomicity**: Verify that failed commands do NOT leak events to the bus. This is a common bug (publishing before persisting).
4.  **Redaction**: When using snapshots (`insta`), always redact volatile fields (UUIDs, Timestamps) to prevent false negatives.

## 3. Implementation Reference

### Plane-Specific Verification
Distinguish between the reliability of the Data Plane and the transience of the Control Plane.

```rust
use lithos_test_utils::MockEventBus;

#[tokio::test]
async fn dual_plane_emission() {
    let bus = MockEventBus::new();
    let mut data_rx = bus.subscribe_data();
    let mut control_rx = bus.subscribe_control();

    // Execute business logic that emits to multiple planes...
    bus.publish_data(my_event).await.unwrap();
    bus.publish_control(signal).await.unwrap();

    // Verify Data Plane persistence (must exist and be ordered)
    assert_eq!(bus.recorded_data_events().await.len(), 1);

    // Verify Control Plane signal (must be received by active subscriber)
    assert!(control_rx.try_recv().is_ok());
}
```

### Eventual Consistency & Timeouts
Avoid flakiness in read-model tests by using the polling helper.

```rust
use lithos_test_utils::EventualConsistencyTester;

#[tokio::test]
async fn read_model_eventually_syncs() {
    let tester = EventualConsistencyTester::new();

    // The tester polls the condition every 10ms until true or timeout (default 5s)
    tester.wait_until(|| async {
        query_service.get_note("note-1").await.is_some()
    }).await.expect("Read model failed to synchronize within timeout");
}
```

### Contract & Payload Verification
Verify that the domain events fulfill their serialization contracts using `PayloadAssertion`.

```rust
use lithos_test_utils::PayloadAssertion;

#[test]
fn event_contract_is_stable() {
    let event = NoteCreated { id: "n1".into() };

    // Define the expected JSON shape explicitly
    let expected_json = serde_json::json!({
        "type": "NoteCreated",
        "id": "n1"
    });

    // Verifies bidirectional serialization (Struct -> JSON -> Struct)
    // Ensures no data is lost and keys match snake_case/camelCase rules
    PayloadAssertion::verify(&expected_json, &event).expect("Event contract broken");
}
```

### Ordering and Timing Verification
Use `EventRecord` and `SequenceAssertion` to validate causal chains.

```rust
use lithos_test_utils::{EventRecord, TimingAssertion, SequenceAssertion};

#[test]
fn events_occur_in_causal_order() {
    let records = vec![
        EventRecord::with_timestamp(1, t0, "Started"),
        EventRecord::with_timestamp(2, t1, "Processing"),
        EventRecord::with_timestamp(3, t2, "Completed"),
    ];

    // Verify logical order (A before B before C)
    SequenceAssertion::verify_order(&records, vec!["Started", "Processing", "Completed"]);

    // Verify temporal order (timestamps are monotonic)
    TimingAssertion::verify_non_decreasing(&records).unwrap();

    // Verify performance constraint (Total duration < 1s)
    TimingAssertion::verify_max_span(&records, Duration::seconds(1)).unwrap();
}
```

## 4. Advanced Patterns

### Malformed Event Handling (Poison Pill)
Subscribers must be robust. Test that they don't crash the bus on bad data.

```rust
#[tokio::test]
async fn subscriber_survives_malformed_payload() {
    let bus = MockEventBus::new();
    let subscriber = MySubscriber::new(bus.clone());

    // Inject poison pill (raw JSON that doesn't match struct)
    let malformed = serde_json::json!({ "id": 123 }); // Expecting String, got Int
    bus.inject_raw_data(malformed).await;

    // Inject valid event afterwards
    bus.publish_data(valid_event).await.unwrap();

    // Verify subscriber processed the valid event despite the error
    assert_eq!(subscriber.processed_count().await, 1);
    assert_eq!(subscriber.error_count().await, 1);
}
```

### Schema Evolution (Backwards Compatibility)
Verify that old events can still be read by new code.

```rust
#[test]
fn can_read_legacy_event_version() {
    // JSON from Version 1.0 (missing "author" field)
    let v1_event = serde_json::json!({ "id": "n1", "content": "foo" });

    // Attempt to deserialize into Version 2.0 Struct
    let v2_event: NoteCreatedV2 = serde_json::from_value(v1_event).unwrap();

    // Verify default value was applied
    assert_eq!(v2_event.author, "Anonymous");
}
```

### Trace Verification (Observability)
Use `TestTracingSubscriber` to verify that high-level operations emit correct observability spans. This ensures we can debug production issues.

```rust
use lithos_test_utils::obs::TestTracingSubscriber;

#[tokio::test]
async fn operation_emits_traces() {
    let subscriber = TestTracingSubscriber::install();

    // Perform complex operation
    my_complex_operation().await;

    // Verify Span hierarchy
    subscriber.assert_span_exists("indexing_batch");

    // Verify specific log event within span
    subscriber.assert_event_emitted("file_processed");
}
```

## 5. Anti-Patterns (Do Not Do This)

### ❌ The "Ghost Listener"
```rust
// BAD: Subscribing AFTER publishing
bus.publish_control(msg).await;
let mut rx = bus.subscribe_control(); // Message is already gone!
```
**Fix**: Always subscribe *before* triggering the action for Broadcast channels.

### ❌ The "Unchecked Spawn"
```rust
// BAD: Spawning a subscriber without tracking it
tokio::spawn(async move { subscriber.run().await });
// If it panics, the test might pass but logs will be dirty
```
**Fix**: Use `JoinSet` or return a `JoinHandle` and `.await` it at the end of the test.

---
*For high-level guides, see [docs/test_guide.md](../test_guide.md)*
