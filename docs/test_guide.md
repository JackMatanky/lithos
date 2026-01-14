# Lithos Test Guide (Master Manual)

This guide provides a comprehensive reference for testing standards, patterns, and tools in the Lithos project. For the full architectural strategy, see [\_bmad-output/test-design-system.md](../_bmad-output/test-design-system.md).

## 1. Authorized Entry Points (Mise)

All testing tasks MUST be orchestrated via `mise`. This ensures the correct environment variables and toolchains are used.

| Command                      | Action                                                                            |
| :--------------------------- | :-------------------------------------------------------------------------------- |
| `mise run test`              | Run all unit and integration tests (alias: `t`).                                  |
| `mise run test:unit`         | Run all unit tests across the workspace using `nextest`.                          |
| `mise run test:unit:<crate>` | Run unit tests for a specific crate (e.g., `test:unit:app`).                      |
| `mise run test:integration`  | Run all integration tests across the workspace.                                   |
| `mise run test:coverage`     | Generate code coverage reports using `tarpaulin`.                                 |
| `mise run test:bench`        | Run all performance benchmarks using `criterion`.                                 |
| `mise run test:watch`        | Watch mode: automatically run tests on file changes.                              |
| `mise run verify`            | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |

## 2. Tools & Infrastructure

Lithos leverages a modern Rust testing stack to ensure speed and reliability.

- **nextest**: Primary test runner for concurrent execution. Faster and more robust than raw `cargo test`.
- **tarpaulin**: Used for coverage analysis (Target: 80%+).
- **insta**: Preferred for snapshot testing of complex structures (e.g., Markdown AST).
- **criterion**: Mandatory for NFR-critical paths like Indexing and Rendering.
- **doc tests**: Mandatory for all public domain models. Use as "living documentation."

## 3. Testing Hierarchy

Lithos follows a hexagonal testing strategy to ensure coverage across all layers of the architecture while maintaining fast execution.

| Layer                            | Focus                                                     | Location                    | Tools                     |
| :------------------------------- | :-------------------------------------------------------- | :-------------------------- | :------------------------ |
| **Domain (Unit)**                | Business logic, state transitions, conversions. Zero I/O. | `crates/domain/src/**/*.rs` | `mise run test:unit`      |
| **Application (Integration)**    | Cross-module orchestration, port contracts, event flows.  | `crates/app/tests/`         | `nextest`, `mockall`      |
| **Infrastructure (Integration)** | Adapters, persistence, external APIs.                     | `tests/suite/integration/`  | `nextest`                 |
| **CLI (E2E)**                    | End-to-end user flows, binary execution.                  | `tests/suite/e2e/`          | `assert_cmd`, `TestVault` |

## 3. Safety Invariants

### Async Testing

Lithos is built on Tokio. All async tests must follow these safety invariants:

- **Runtime Flavor**: Use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` (or `async_test!`) to surface race conditions.
- **Blocking Limit**: NEVER block an async thread for >10ms. Use `spawn_blocking_test` for `std::fs` or heavy CPU tasks.
- **Timeouts**: Always wrap async operations with a timeout (e.g., `with_timeout(Duration::from_secs(5), ...)`).
- **Throttling**: Use `tokio::sync::Semaphore` to limit concurrent I/O during test execution.

### Determinism

- **Virtual Clock**: Use `time_test!` macro to control `tokio::time::pause()` and `advance`.
- **Fixed Seeds**: Use deterministic seeds for any randomness or UUID generation in fixtures.
- **Snapshot Redactions**: Always redact UUIDs and Timestamps using global regex filters in `insta`.

## 4. Test Authoring Standards

### Naming Conventions (Verb-First)

Tests must read like sentences and describe the behavior being verified:

- ✅ `maintains_event_bus_api_contract_across_boundaries`
- ✅ `test_note_creation_fails_with_duplicate_title`
- ❌ `test_note_1` (non-descriptive)

### Advanced Verification

- **Observability**: Use `TestTracingSubscriber` to verify emitted spans and events.
- **Property Testing**: Use `Proptest` for mathematical edge cases and state transition verification.
- **Error Assertions**: Use the `assert_err_kind!` macro for standardized error matching.
- **Domain Purity**: Programmatic enforcement ensures `lithos-domain` remains free of I/O dependencies.

## 5. Streamlining with `lithos-test-utils`

The `lithos-test-utils` crate is the "Core OS" for testing in Lithos.

### Async & Time Control

```rust
time_test!(async fn validates_cache_expiry() {
    cache.set("key", "val", Duration::from_secs(60)).await;
    advance(Duration::from_secs(61)).await;
    assert!(cache.get("key").await.is_none());
});
```

### Filesystem & Vaults

```rust
let vault = TestVault::new()
    .with_note("Work/Project.md", "# Project\nStatus: Active")
    .with_config("lithos.toml", "[vault]\nstrict = true")
    .build();

let context = IsolatedTestContext::new("my_test");
// context.temp_dir() is ready for use
```

### CQRS & Event Verification

```rust
tester.given(initial_events)
    .when(command)
    .then_expect_events(expected_events);
```

## 6. Linting & Code Quality in Tests

Lithos maintains strict quality gates even for test code. While tests have more latitude than business logic, they must still be modular and readable.

### #expect vs #allow

- **Use `#[expect(...)]`**: For intentional lint violations that are necessary for the test (e.g., using `unwrap` in a setup block or creating a complex fixture that exceeds cognitive complexity limits). This tells the compiler "I know this violates a rule, but it's intentional."
- **Use `#[allow(...)]`**: Primarily for generated code (e.g., `automock`) where the developer doesn't control the output. Avoid using `allow` for hand-written test logic.

### unwrapping in Tests

- **Setup Phase**: Using `unwrap()` or `expect()` is acceptable in the _Arrange_ phase of a test. If the setup fails, the test should panic immediately as the prerequisite state wasn't met.
- **Assertion Phase**: ALWAYS use `Result` assertions or specialized macros. Never `unwrap()` a result you intend to verify; it hides the failure context and results in poor diagnostic output.

### Doc-Tests (Mandatory API Documentation)

Doc-tests are prescribed for all **public domain models** and **utility functions**.

- They serve as the "Living Documentation" for the codebase.
- High-fidelity examples of how to use `lithos-test-utils` components must be implemented as doc-tests in the source code to ensure they are verified by `mise run test:unit` (which orchestrates both nextest and doc-tests).

## 7. Common Pitfalls

- **Thread Starvation**: Blocking the Tokio executor with `std::fs` or `sleep`. Use `spawn_blocking_test` and `tokio::time::sleep`.
- **Race Conditions**: Using single-threaded test flavor for concurrent code. Always use `multi_thread` for integration tests.
- **Flakiness**: Relying on wall-clock time. Use `tokio::time::pause()` and the `advance()` helper for deterministic time-based tests.
- **Shared State**: Using static variables or shared files between tests. Always use fresh fixtures (e.g., `IsolatedTestContext`).

## 7. Resources & Deep-Dives

For tactical implementation details, refer to the following specs:

- [**Async Testing Pattern Spec**](./testing/async.md)
- [**CQRS Testing Pattern Spec**](./testing/cqrs.md)
- [**Event-Driven Testing Pattern Spec**](./testing/event.md)
- [**ADR 0010: Test Utilities**](./adr/0010-centralized-test-utilities.md)
