# Developer Testing Guide

This guide provides a consolidated reference for testing standards, patterns, and tools in the Lithos project. It is intended for developers onboarding to the project or authoring new tests.

## 1. Testing Hierarchy

Lithos follows a hexagonal testing strategy to ensure coverage across all layers of the architecture while maintaining fast execution.

| Layer | Focus | Location | Tools |
|---|---|---|---|
| **Domain (Unit)** | Business logic, state transitions, conversions. Zero I/O. | `crates/domain/src/**/*.rs` | `cargo test` |
| **Application (Integration)** | Cross-module orchestration, port contracts, event flows. | `crates/app/tests/` | `nextest`, `mockall` |
| **Infrastructure (Integration)** | Adapters, persistence, external APIs. | `tests/integration/` | `testcontainers` (deferred), `nextest` |
| **CLI (E2E)** | End-to-end user flows, binary execution. | `tests/e2e/` | `assert_cmd`, `tempfile` |

## 2. Async Testing

Lithos is built on Tokio. All async tests must follow these safety invariants:

- **Runtime Flavor:** Use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` for integration tests to surface race conditions.
- **Blocking Limit:** NEVER block an async thread for >10ms. Use `spawn_blocking_test` for `std::fs` or heavy CPU tasks.
- **Timeouts:** Always wrap async operations with a timeout (e.g., `with_timeout(Duration::from_secs(5), ...)`).
- **Throttling:** Use `tokio::sync::Semaphore` to limit concurrent I/O during test execution.

For detailed patterns, see [Async Testing Guidelines](./async.md).

## 3. Event & CQRS Testing

Lithos uses a hybrid event bus. Testing should verify both command effects and read-model consistency.

- **Given-When-Then:** Use the `EventTestFramework` for testing aggregates.
- **Mock Event Bus:** Use `MockEventBus` to verify events published to Data, Control, or State planes.
- **Payload Verification:** Use serialized comparison for domain event contracts.
- **Eventual Consistency:** When testing read-models, account for propagation delay using controlled time or retries.

For detailed patterns, see [Event-Driven Testing](./event.md) and [ADR 0009](../adr/0009-cqrs-testing-patterns.md).

## 4. Running Tests

All test tasks are orchestrated via `mise`. This is the **primary and authorized entry point** for all tasks.

### Mise Commands
The following `mise` tasks are available for test execution and quality assurance:

| Task | Description |
|---|---|
| `mise run test` | Run all unit and integration tests (alias: `t`). |
| `mise run test:unit` | Run all unit tests across the workspace using `nextest`. |
| `mise run test:unit:<crate>` | Run unit tests for a specific crate (e.g., `test:unit:app`). |
| `mise run test:integration` | Run all integration tests across the workspace. |
| `mise run test:coverage` | Generate code coverage reports using `tarpaulin`. |
| `mise run test:bench` | Run all performance benchmarks using `criterion`. |
| `mise run test:bench:<crate>` | Run benchmarks for a specific crate. |
| `mise run test:watch` | Watch mode: automatically run tests on file changes. |
| `mise run verify` | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |
| `mise run quality` | Run formatting, linting, and ADR validation without running tests. |
| `mise run clean` | Remove build artifacts, test outputs, and coverage reports. |

### Tools
- **nextest:** Primary test runner for concurrent execution.
- **tarpaulin:** Used for coverage analysis (Target: 80%+).
- **insta:** Preferred for snapshot testing of complex structures (e.g., Markdown AST).
- **criterion:** Mandatory for NFR-critical paths (Indexing, Rendering).
- **doc tests:** Mandatory for all public domain models and utility functions. Use as "living documentation" with minimal, illustrative examples.

## 5. Test Authoring Standards

### Naming Conventions (Verb-First)
Tests must read like sentences and describe the behavior being verified. Use a **verb-first** naming convention:
- ✅ `test_note_creation_fails_with_duplicate_title`
- ✅ `maintains_event_bus_api_contract_across_boundaries`
- ✅ `detailed_equality_assertion_panics_for_unequal_values`
- ❌ `test_note_1` (non-descriptive)
- ❌ `issue_42_fix` (implementation-focused)

### Test Utilities and Fixtures
- **TestFactory:** Use the `TestFactory` proc-macro for type-safe data generation.
- **TestVault:** Use the `TestVault` utility for spinning up mock Obsidian vaults with a fluent API.
- **IsolatedTestContext:** Use for unique temp directories and database namespaces per test.
- **rstest:** Preferred for fixture injection and parameterized testing.

### Advanced Verification
- **Observability:** Use `TestTracingSubscriber` to verify emitted spans and events.
- **Property Testing:** Use `Proptest` for mathematical edge cases and state transition verification.
- **Error Assertions:** Use the `assert_err_kind!` macro for standardized error matching.
- **Domain Purity:** Programmatic enforcement ensures `lithos-domain` remains free of I/O dependencies.

### Determinism and Snapshots
- **Virtual Clock:** Use `time_test!` macro to control `tokio::time::pause()` and `advance`.
- **Snapshot Redactions:** Always redact UUIDs and Timestamps using global regex filters in `insta`.
- **Fixed Seeds:** Use deterministic seeds for any randomness or UUID generation in fixtures.

## 6. Streamlining with `lithos-test-utils`

The `lithos-test-utils` crate is the "Core OS" for testing in Lithos. It is designed to minimize boilerplate and enforce consistency. Use these utilities to accelerate your development loop.

### Quick Start: Boilerplate Reduction
Import the prelude to get access to common macros and utilities:
```rust
use lithos_test_utils::*;
```

### Async & Time Control
Streamline async setup and eliminate flakiness in time-sensitive code.
- **`async_test!`**: Replaces standard tokio test attributes with project-compliant multi-threaded defaults.
- **`time_test!`**: Automatically pauses the virtual clock. Use `tokio::time::advance` to jump through time deterministically without waiting.

```rust
time_test!(async fn validates_cache_expiry() {
    cache.set("key", "val", Duration::from_secs(60)).await;
    advance(Duration::from_secs(61)).await;
    assert!(cache.get("key").await.is_none());
});
```

### Data Factories & Builders
Avoid manual struct initialization and keep tests resilient to schema changes.
- **`test_builder!`**: Generates type-safe builders for domain entities.
- **`TestFactory`**: Proc-macro for generating randomized yet deterministic test data.

```rust
// In your test or fixture
let note = NoteBuilder::default()
    .with_title("Streamlining Guide")
    .build();
```

### Filesystem & Vaults
Quickly spin up realistic environments for file-based operations.
- **`TestVault`**: A fluent API to create complex Obsidian-style vault structures in a temporary directory.
- **`IsolatedTestContext`**: Provides a unique, isolated workspace per test, including temp paths and database namespaces.

```rust
let vault = TestVault::new()
    .with_note("Work/Project.md", "# Project\nStatus: Active")
    .with_config("lithos.toml", "[vault]\nstrict = true")
    .build();

let context = IsolatedTestContext::new("my_test");
// context.temp_dir() is ready for use
```

### CQRS & Event Verification
Declarative verification of complex business flows and eventual consistency.
- **`EventTestFramework`**: Given-When-Then pattern for aggregate command handling.
- **`SagaTester`**: Coordinates and verifies interactions across multiple aggregates and read models.
- **`EventualConsistencyTester`**: Polling utility that waits for read-models to sync with a configurable timeout.

```rust
tester.given(initial_events)
    .when(command)
    .then_expect_events(expected_events);
```

### Mocks & Ports
Standardized mocks for hexagonal boundaries.
- **`MockEventBus`**: Full implementation of the three-plane event bus (Data, Control, State) for subscriber testing.
- **`MockRepositoryPort`**: Pre-configured `mockall` traits for common persistence operations.

### Observability
Test your instrumentation as a first-class citizen.
- **`TestTracingSubscriber`**: Installs a local subscriber to verify that specific spans or log events were emitted by the code under test.

```rust
let subscriber = TestTracingSubscriber::install();
operation().await;
subscriber.assert_span_emitted("note_indexed");
```

## 7. Common Pitfalls

- **Thread Starvation:** Blocking the Tokio executor with `std::fs` or `sleep`. Use `spawn_blocking` and `tokio::time::sleep`.
- **Race Conditions:** Using single-threaded test flavor for concurrent code. Always use `multi_thread` for integration tests.
- **Flakiness:** Relying on wall-clock time. Use `tokio::time::pause()` for deterministic time-based tests.
- **Shared State:** Using static variables or shared files between tests. Always use fresh fixtures.

## 8. Resources
- [Async Testing](./async.md)
- [Event-Driven Testing](./event.md)
- [CQRS Testing](./cqrs.md)
- [ADR 0010: Test Utilities](../adr/0010-centralized-test-utilities.md)
- [ADR 0011: Integration Testing](../adr/0011-integration-testing-patterns.md)
- [ADR 0012: Benchmarking](../adr/0012-benchmarking-infrastructure.md)
