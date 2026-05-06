> Legacy document: non-authoritative. Superseded by `docs/engineering/testing/README.md` and linked active testing docs.

# CQRS Testing Guidelines

Tactical specification for testing Command and Query responsibilities in Lithos.

## 1. Key Principles

### Responsibility Segregation
- **Commands (Write Side)**: Tested by side-effects. We verify that the command resulted in the correct Events being emitted and the correct State being persisted.
- **Queries (Read Side)**: Tested by return values. We setup a known state (stub) and verify the query returns the correct projection of that state.

### Port Supremacy (Hexagonal)
- **Test through Ports**: Tests should interact with `RepositoryPort`, not `RedbRepository`. This decouples the test from the implementation details (file paths, serialization formats).
- **Mocks vs Stubs**:
  - Use **Mocks** for Commands (verify `save()` was called).
  - Use **Stubs** for Queries (provide data for `find()`).

### State Verification
- **Event Sourcing**: If the aggregate is event-sourced, the test "Given" is a list of past events, "When" is the command, "Then" is the new events.
- **State Persistence**: If the aggregate is state-persisted, we verify the repository `save` argument matches the expected state.

## 2. Golden Rules

1.  **Isolate Writes**: Use `MockRepository` or `mockall::automock` to verify that handlers call the correct persistence methods with the expected parameters.
2.  **Stub Reads**: Use `StubQueryStore` to provide predictable, static data for query handler tests. Never rely on "leftover" data from previous tests.
3.  **Validation First**: Every handler test suite must include "negative cases". Validation logic (e.g., "Title cannot be empty") is domain logic and must be tested thoroughly.
4.  **No Direct DB Access**: Query tests should never touch a real `Redb` instance unless it's an integration test. Unit tests use memory-backed stubs.

## 3. Implementation Reference

### Command Handler Testing (Behavioral)
Focus on the interactions between the handler and its ports.

```rust
use crate::tests::mocks::{MockRepository, EventVerifier};

#[tokio::test]
async fn create_user_saves_to_repo_and_publishes_event() {
    // Arrange
    let mut mock_repo = MockRepository::new();
    mock_repo.expect_save()
        .withf(|u| u.id == "u1") // Verify argument content
        .times(1)
        .returning(|_| Ok(()));

    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(Arc::new(mock_repo), event_verifier.clone());

    // Act
    handler.handle(CreateUserCommand { id: "u1".into() }).await.expect("Handler failed");

    // Assert
    // Mock expectations checked on drop
    event_verifier.assert_event_exists(&UserCreated { id: "u1".into() }).await.unwrap();
}
```

### Query Handler Testing (Predictable)
Queries should be "Pure Functions" of the read model state.

```rust
use crate::tests::mocks::StubQueryStore;

#[tokio::test]
async fn get_user_query_returns_correct_stubbed_data() {
    // Arrange
    let test_users = vec![
        UserRM { id: "u1".into(), name: "Alice".into() },
        UserRM { id: "u2".into(), name: "Bob".into() }
    ];
    // StubStore allows ergonomic data setup without mocking complex traits
    let stub_store = Arc::new(StubQueryStore::with_data(test_users, |u| u.id.clone()));
    let handler = GetUserHandler::new(stub_store);

    // Act
    let result = handler.handle(GetUserQuery { id: "u1".into() }).await.unwrap();

    // Assert
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Alice");
}
```

### Saga and Process Manager Testing
Verify long-running workflows that span multiple aggregates using `SagaTester`. This is critical for testing "Eventual Consistency" logic (e.g., updating search index after note creation).

```rust
use crate::tests::mocks::SagaTester;

#[tokio::test]
async fn order_fulfillment_saga_flow() {
    let mut tester = SagaTester::new();

    // 1. Initial State (Past Events)
    tester.given(vec![
            OrderCreated { id: 1 }.into(),
            InventoryReserved { item_id: 99 }.into()
        ])
        // 2. The Trigger (New Event)
        .when(PaymentConfirmed { order_id: 1 })
        // 3. Expected Outcome (Command or Event)
        .then_expect_events(vec![
            OrderReadyForShipment { id: 1 }.into()
        ]);

    // Verify that the Process Manager actually dispatched the messages
    tester.assert_all_participants_updated().await;
}
```

### Validation Testing Invariants
Always test that the handler rejects invalid state transitions.

```rust
#[tokio::test]
async fn command_fails_on_domain_invariant_violation() {
    let handler = CreateUserHandler::new(mock_repo, mock_events);

    // Invalid Command (empty ID)
    let cmd = CreateUserCommand { id: "".into() };

    let result = handler.handle(cmd).await;

    // Use assert_err_kind macro for clean error assertion
    assert_err_kind!(result, DomainErrorKind::ValidationFailed);

    // Ensure no side effects happened
    // This protects against "partial application" bugs
    assert_eq!(mock_repo.save_count().await, 0);
}
```

## 4. Advanced Patterns

### Event Sourcing Framework
For purely event-sourced aggregates, use the explicit framework. This abstracts away the repository store/load cycle.

```rust
use crate::tests::mocks::EventTestFramework;

#[test]
fn account_withdraw_logic() {
    EventTestFramework::default()
        .given(vec![AccountDeposited { amount: 100 }]) // Hydrate state
        .when(WithdrawCommand { amount: 50 })          // Execute logic
        .then_expect_events(vec![                      // Verify output
            AccountWithdrawn { amount: 50, balance: 50 }
        ]);
}
```

### Testing Concurrency in Handlers
Use `tokio::spawn` to simulate concurrent commands hitting the same handler to verify optimistic locking or idempotency.

```rust
let handler = Arc::new(handler);
let h1 = handler.clone();
let h2 = handler.clone();

let cmd1 = UpdateNoteCommand { id: "n1".into(), ver: 1, content: "A".into() };
let cmd2 = UpdateNoteCommand { id: "n1".into(), ver: 1, content: "B".into() };

// Run both concurrently
let (r1, r2) = tokio::join!(
    h1.handle(cmd1),
    h2.handle(cmd2) // Same version! Conflict expected.
);

// One should succeed, one should fail (Optimistic Concurrency Control)
// We don't know which wins, but we know they can't BOTH win.
assert!(r1.is_ok() ^ r2.is_ok());
```

### Projection Testing (Read Models)
Test how events update the Read Models.

```rust
#[tokio::test]
async fn projection_updates_read_model_on_event() {
    let store = Arc::new(InMemoryReadModelStore::new());
    let projection = UserProjection::new(store.clone());

    // Apply event
    projection.apply(UserCreated { id: "u1".into(), name: "Alice".into() }).await.unwrap();

    // Verify Read Model State
    let user = store.get("u1").await.unwrap();
    assert_eq!(user.name, "Alice");
}
```

### Snapshot Testing
For complex aggregates, use snapshots to verify state without listing every event.

```rust
use insta::assert_debug_snapshot;

#[test]
fn aggregate_state_snapshot() {
    let agg = Aggregate::from_events(history);
    // Snapshot the final state
    assert_debug_snapshot!(agg);
}
```

## 5. Anti-Patterns (Do Not Do This)

### ❌ The "Leaky Mock"
```rust
// BAD: Mock expects strict call order unrelated to logic
mock.expect_validate().times(1);
mock.expect_save().times(1);
```
**Fix**: Use `mockall::Sequence` only when order strictly matters (e.g. validate BEFORE save). Otherwise, allow flexible ordering.

### ❌ The "Stubborn Query"
```rust
// BAD: Using a real DB for a unit test
let db = Redb::open("/tmp/test.db");
```
**Fix**: Use `StubQueryStore` or `InMemoryRepository`. Real DBs belong in `lithos-core/tests/` (when added).

### ❌ The "Silent Failure"
```rust
// BAD: Ignoring the Result
handler.handle(cmd).await;
// If it failed, we wouldn't know!
```
**Fix**: Always `.unwrap()` or `.expect()` the result in positive tests.

---
*For high-level guides, see [docs/test_guide.md](../test_guide.md)*
