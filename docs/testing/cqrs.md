# CQRS Testing Guidelines

**Status**: Active
**ADR Reference**: [ADR 0009: CQRS Testing Patterns](../adr/0009-cqrs-testing-patterns.md)
**Last Updated**: 2026-01-12

---

## Table of Contents

- [Overview](#overview)
- [Command Handler Testing](#command-handler-testing)
- [Query Handler Testing](#query-handler-testing)
- [Event Sourcing Testing](#event-sourcing-testing)
- [Eventual Consistency Testing](#eventual-consistency-testing)
- [Integration Testing](#integration-testing)
- [Best Practices](#best-practices)
- [Anti-Patterns](#anti-patterns)
- [Examples](#examples)

---

## Overview

This guide provides comprehensive patterns for testing CQRS (Command Query Responsibility Segregation) architectures in Lithos. Following ADR 0009, we maintain strict separation between command-side (write) and query-side (read) testing using mocks and stubs respectively.

### Key Principles

1. **Command Testing**: Use `MockRepository` to isolate write operations
2. **Query Testing**: Use `StubQueryStore` for predictable read operations
3. **Event Verification**: Capture and verify domain events
4. **Async-First**: All tests use Tokio runtime for realistic async testing
5. **Hexagonal Architecture**: Test through ports, not adapters

---

## Command Handler Testing

### Basic Command Handler Test

Command handlers coordinate write operations and emit domain events. Test them using mock repositories and event verifiers.

```rust
use lithos_test_utils::cqrs::{MockRepository, EventVerifier};
use std::sync::Arc;

#[tokio::test]
async fn create_user_command_saves_entity_and_publishes_event() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo.clone(), event_verifier.clone());

    // Act
    let command = CreateUserCommand {
        id: "user-1".to_string(),
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };
    handler.handle(command).await.unwrap();

    // Assert
    assert_eq!(mock_repo.save_count().await, 1);
    event_verifier.assert_event_count(1).await.unwrap();
    event_verifier.assert_event_exists(&UserCreatedEvent { /* ... */ }).await.unwrap();
}
```

### Validation Testing

Always test command validation logic to ensure business rules are enforced:

```rust
#[tokio::test]
async fn create_user_command_rejects_invalid_email() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo.clone(), event_verifier.clone());

    // Act
    let command = CreateUserCommand {
        id: "user-1".to_string(),
        email: "".to_string(), // Invalid
        name: "Alice".to_string(),
    };
    let result = handler.handle(command).await;

    // Assert
    assert!(result.is_err());
    assert_eq!(mock_repo.save_count().await, 0); // Nothing saved
    event_verifier.assert_event_count(0).await.unwrap(); // No events
}
```

### Error Scenario Testing

Test failure paths using mock repository error injection:

```rust
#[tokio::test]
async fn create_user_command_handles_repository_failure() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    mock_repo.fail_next_save("Database connection failed").await;
    let event_verifier = Arc::new(EventVerifier::new());
    let handler = CreateUserHandler::new(mock_repo, event_verifier.clone());

    // Act
    let command = CreateUserCommand { /* ... */ };
    let result = handler.handle(command).await;

    // Assert
    assert!(result.is_err());
    event_verifier.assert_event_count(0).await.unwrap(); // No events on failure
}
```

### Interaction Verification

Verify the sequence and content of repository interactions using `mockall` expectations:

```rust
#[tokio::test]
async fn command_handler_interaction_sequence() {
    // Arrange
    let mut mock_repo = MockUserRepository::new();
    mock_repo.expect_save()
        .times(1)
        .returning(|_| Ok(()));

    let handler = CreateUserHandler::new(Arc::new(mock_repo), /* ... */);

    // Act
    handler.handle(CreateUserCommand { /* ... */ }).await.unwrap();

    // Assert (handled by mockall drop check)
}
```

---

## Saga and Process Manager Testing

Use `SagaTester` to verify long-running processes that span multiple aggregates:

```rust
use lithos_test_utils::cqrs::SagaTester;

#[tokio::test]
async fn order_saga_completes_successfully() {
    let mut tester = SagaTester::new();

    tester.given(vec![OrderCreated { id: 1 }])
        .when(PaymentConfirmed { order_id: 1 })
        .then_expect_events(vec![OrderReadiedForShipment { id: 1 }]);

    tester.assert_all_participants_updated().await;
}
```

---

## Query Handler Testing

### Basic Query Handler Test

Query handlers retrieve data from read models. Test them using stubbed data stores:

```rust
use lithos_test_utils::cqrs::StubQueryStore;
use std::sync::Arc;

#[tokio::test]
async fn get_user_query_returns_correct_user() {
    // Arrange
    let test_users = vec![
        UserReadModel { id: "user-1".to_string(), email: "alice@example.com".to_string(), /* ... */ },
    ];
    let stub_store = Arc::new(StubQueryStore::with_data(test_users, |u: &UserReadModel| u.id.clone()));
    let handler = GetUserHandler::new(stub_store);

    // Act
    let query = GetUserQuery { user_id: "user-1".to_string() };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert!(result.is_some());
    assert_eq!(result.unwrap().email, "alice@example.com");
}
```

### Not Found Scenarios

Test query handlers return appropriate results when data doesn't exist:

```rust
#[tokio::test]
async fn get_user_query_returns_none_for_nonexistent_user() {
    // Arrange
    let stub_store = Arc::new(StubQueryStore::with_data(vec![], |u: &UserReadModel| u.id.clone()));
    let handler = GetUserHandler::new(stub_store);

    // Act
    let query = GetUserQuery { user_id: "user-999".to_string() };
    let result = handler.handle(query).await.unwrap();

    // Assert
    assert!(result.is_none());
}
```

### Performance Validation

Verify query performance meets requirements:

```rust
#[tokio::test]
async fn query_handler_performs_within_time_bounds() {
    // Arrange
    let stub_store = Arc::new(StubQueryStore::with_data(test_users(), /* ... */));
    let handler = GetUserHandler::new(stub_store);

    // Act
    let start = std::time::Instant::now();
    handler.handle(GetUserQuery { /* ... */ }).await.unwrap();
    let duration = start.elapsed();

    // Assert
    assert!(duration.as_millis() < 10, "Query took {duration:?}, expected <10ms");
}
```

### Dynamic Data Updates

Stub stores can be updated during tests for complex scenarios:

```rust
#[tokio::test]
async fn stub_store_supports_dynamic_updates() {
    // Arrange
    let stub_store = StubQueryStore::with_data(vec![], |u: &UserReadModel| u.id.clone());

    // Act
    stub_store.add(UserReadModel { /* ... */ }).await;

    // Assert
    let all_data = stub_store.all_data().await;
    assert_eq!(all_data.len(), 1);
}
```

---

## Event Sourcing Testing

### Given-When-Then Pattern

Test aggregates using the TestFramework for readable BDD-style tests:

```rust
use lithos_test_utils::cqrs::TestFramework;

#[test]
fn account_deposit_increases_balance() {
    TestFramework::default()
        .given(vec![AccountEvent::Opened { id: 1, balance: 0 }])
        .when(DepositMoneyCommand { amount: 100 })
        .then_expect_events(vec![
            AccountEvent::MoneyDeposited { amount: 100, new_balance: 100 }
        ]);
}
```

### Event Sequence Verification

Verify aggregates emit correct event sequences:

```rust
#[test]
fn multiple_deposits_emit_correct_events() {
    let framework = TestFramework::default()
        .given(vec![AccountEvent::Opened { id: 1, balance: 0 }]);

    // Verify first deposit
    framework.clone()
        .when(DepositMoneyCommand { amount: 100 })
        .then_expect_events(vec![
            AccountEvent::MoneyDeposited { amount: 100, new_balance: 100 }
        ]);

    // Verify second deposit builds on first
    framework
        .given(vec![AccountEvent::MoneyDeposited { amount: 100, new_balance: 100 }])
        .when(DepositMoneyCommand { amount: 50 })
        .then_expect_events(vec![
            AccountEvent::MoneyDeposited { amount: 50, new_balance: 150 }
        ]);
}
```

---

## Eventual Consistency Testing

### Timing Control

Use `tokio::time` to control timing in eventual consistency tests:

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn command_eventually_updates_read_model() {
    // Arrange
    let mock_repo = Arc::new(MockRepository::new());
    let event_bus = Arc::new(TestEventBus::new());
    let read_model = Arc::new(InMemoryUserReadModel::new(event_bus.clone()));

    // Act
    let handler = CreateUserHandler::new(mock_repo, event_bus);
    handler.handle(CreateUserCommand { /* ... */ }).await.unwrap();

    // Wait for eventual consistency
    let result = timeout(
        Duration::from_millis(100),
        read_model.wait_for_user(user_id)
    ).await;

    // Assert
    assert!(result.is_ok(), "Read model not updated within timeout");
}
```

### Cross-Aggregate Verification

Test sagas that span multiple aggregates:

```rust
#[tokio::test]
async fn order_saga_updates_inventory_and_payment() {
    // Arrange
    let inventory_repo = Arc::new(MockRepository::new());
    let payment_repo = Arc::new(MockRepository::new());
    let event_bus = Arc::new(TestEventBus::new());

    // Setup saga participants
    let inventory_handler = InventoryHandler::new(inventory_repo.clone(), event_bus.clone());
    let payment_handler = PaymentHandler::new(payment_repo.clone(), event_bus.clone());

    // Act
    let order_handler = OrderHandler::new(event_bus);
    order_handler.handle(PlaceOrderCommand { /* ... */ }).await.unwrap();

    // Assert - both aggregates updated
    assert!(inventory_repo.exists(&item_id).await.unwrap());
    assert!(payment_repo.exists(&payment_id).await.unwrap());
}
```

---

## Integration Testing

### Command-to-Query Workflow

Test complete flows from command execution through query:

```rust
#[tokio::test]
async fn create_user_command_updates_query_model() {
    // Arrange
    let repo = Arc::new(InMemoryRepository::new());
    let query_store = Arc::new(InMemoryQueryStore::new());
    let event_bus = Arc::new(RealEventBus::new());

    // Wire up projections
    let projection = UserProjection::new(query_store.clone());
    event_bus.subscribe_data_plane(projection).await;

    // Act - Execute command
    let command_handler = CreateUserHandler::new(repo, event_bus);
    command_handler.handle(CreateUserCommand { /* ... */ }).await.unwrap();

    // Wait for projection
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Assert - Query returns data
    let query_handler = GetUserHandler::new(query_store);
    let result = query_handler.handle(GetUserQuery { user_id }).await.unwrap();
    assert!(result.is_some());
}
```

---

## Best Practices

### 1. Test Through Ports, Not Adapters

```rust
// ✅ GOOD: Test through port interface
let repo: Arc<dyn RepositoryPort<User>> = Arc::new(MockRepository::new());

// ❌ BAD: Test concrete adapter
let repo = RedbUserRepository::new(/* ... */);
```

### 2. Use Realistic Async Testing

```rust
// ✅ GOOD: Async test with Tokio
#[tokio::test]
async fn test_async_handler() {
    handler.handle(command).await.unwrap();
}

// ❌ BAD: Blocking test
#[test]
fn test_handler() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        handler.handle(command).await.unwrap();
    });
}
```

### 3. Isolate Tests

```rust
// ✅ GOOD: Each test gets fresh mocks
#[tokio::test]
async fn test_command_a() {
    let mock_repo = Arc::new(MockRepository::new());
    // ...
}

#[tokio::test]
async fn test_command_b() {
    let mock_repo = Arc::new(MockRepository::new()); // Fresh instance
    // ...
}

// ❌ BAD: Shared mocks between tests
static SHARED_REPO: Lazy<Arc<MockRepository<User>>> = Lazy::new(|| {
    Arc::new(MockRepository::new()) // Tests will interfere
});
```

### 4. Test Error Paths

```rust
// ✅ GOOD: Test failure scenarios
#[tokio::test]
async fn command_handles_repository_failure() {
    mock_repo.fail_next_save("Connection lost").await;
    let result = handler.handle(command).await;
    assert!(result.is_err());
}

// ❌ BAD: Only test happy paths
#[tokio::test]
async fn command_succeeds() {
    handler.handle(command).await.unwrap(); // What if it fails?
}
```

### 5. Verify Side Effects

```rust
// ✅ GOOD: Verify events were published
event_verifier.assert_event_count(1).await.unwrap();
event_verifier.assert_event_exists(&expected_event).await.unwrap();

// ❌ BAD: Only verify primary result
let result = handler.handle(command).await.unwrap();
assert!(result.is_ok()); // Did events get published?
```

---

## Anti-Patterns

### ❌ Over-Mocking

Don't mock everything - use real implementations where practical:

```rust
// ❌ BAD: Mocking simple value objects
let mock_email = MockEmail::new("alice@example.com");

// ✅ GOOD: Use real value objects
let email = Email::parse("alice@example.com").unwrap();
```

### ❌ Testing Implementation Details

Test behavior, not implementation:

```rust
// ❌ BAD: Testing internal state
assert_eq!(handler.internal_counter, 5);

// ✅ GOOD: Testing observable behavior
assert_eq!(mock_repo.save_count().await, 5);
```

### ❌ Tight Coupling to Mock Structure

Don't let mocks dictate your domain design:

```rust
// ❌ BAD: Domain designed around mocking
struct User {
    pub id: String, // Public to satisfy mock requirements
}

// ✅ GOOD: Domain encapsulated, mocks adapt
struct User {
    id: String,
}

impl Entity for User {
    type Id = String;
    fn id(&self) -> &Self::Id { &self.id }
}
```

### ❌ Ignoring Timing in Async Tests

Account for async timing properly:

```rust
// ❌ BAD: No wait for async operations
handler.handle(command).await.unwrap();
assert!(read_model.has_user(user_id)); // Might not be updated yet

// ✅ GOOD: Wait for consistency
handler.handle(command).await.unwrap();
tokio::time::timeout(
    Duration::from_millis(100),
    read_model.wait_for_user(user_id)
).await.expect("Consistency timeout");
```

---

## Examples

See comprehensive examples in:

- [`crates/test-utils/tests/cqrs_commands.rs`](../../crates/test-utils/tests/cqrs_commands.rs) - Command testing patterns
- [`crates/test-utils/tests/cqrs_queries.rs`](../../crates/test-utils/tests/cqrs_queries.rs) - Query testing patterns
- [`crates/test-utils/src/cqrs.rs`](../../crates/test-utils/src/cqrs.rs) - Framework implementation

---

## Further Reading

- [ADR 0009: CQRS Testing Patterns](../adr/0009-cqrs-testing-patterns.md) - Comprehensive decision rationale
- [ADR 0008: Event-Driven Testing Patterns](../adr/0008-event-driven-testing-patterns.md) - Event testing foundation
- [ADR 0007: Hybrid Event Orchestration](../adr/0007-event-orchestration.md) - Event bus architecture
- [Async Testing Patterns](async-testing.md) - Tokio-specific async testing
- [Integration Testing Guide](integration-testing.md) - End-to-end testing strategies

---

**Maintainers**: Jack
**Review Cycle**: Quarterly or on architectural changes
**Feedback**: Submit PRs for improvements or open discussions for clarifications
