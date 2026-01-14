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
- **pretty_assertions**: Enhances `assert_eq!` and `assert_ne!` with colorful, readable diffs.
- **criterion**: Mandatory for NFR-critical paths like Indexing and Rendering.
- **doc tests**: Mandatory for all public domain models. Use as "living documentation."

## 3. Testing Hierarchy

Lithos follows a hexagonal testing strategy to ensure coverage across all layers of the architecture while maintaining fast execution. We distinguish between three primary sets of tests:

### Unit Tests
Tests that go in the **same module** as the tested unit. This allows visibility over private functions and parent `use` declarations.
- **Focus**: Implementation details, edge-cases, and internal logic.
- **Tools**: `cargo test`, `proptest`.
- **Rule**: KISS (Keep It Simple, Stupid). Test one state and one behavior.

### Integration Tests
Tests that live in the `tests/` directory. They are external to the library and can only test the **public API**.
- **Focus**: Verifying that multiple parts of the system work together correctly.
- **Tools**: `nextest`, `mockall`.
- **Note**: External states and side-effects are permitted here.

### Doc Tests
Executable examples within the source code using `///`.
- **Focus**: Happy paths and general public API usage.
- **Orchestration**: Run via `mise run test:unit`.

| Layer                            | Focus                                                     | Location                    | Tools                     |
| :------------------------------- | :-------------------------------------------------------- | :-------------------------- | :------------------------ |
| **Domain (Unit)**                | Business logic, state transitions, conversions. Zero I/O. | `crates/domain/src/**/*.rs` | `mise run test:unit`      |
| **Application (Integration)**    | Cross-module orchestration, port contracts, event flows.  | `crates/app/tests/`         | `nextest`, `mockall`      |
| **Infrastructure (Integration)** | Adapters, persistence, external APIs.                     | `tests/suite/integration/`  | `nextest`                 |
| **CLI (E2E)**                    | End-to-end user flows, binary execution.                  | `tests/suite/e2e/`          | `assert_cmd`, `TestVault` |

## 4. Safety Invariants

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

## 5. Test Authoring Standards

Tests are the first place people look to understand how your code works. They must be clear, targeted, and serve as **Living Documentation**.

### Naming Conventions & Organization

We follow a **Verb-First** and **Module-Per-Function** organization pattern.

#### The Naming Formula
A good test name reveals: `unit_of_work` + `expected_behavior` + `state_under_test`.

*   ✅ `returns_error_when_vault_path_is_invalid`
*   ✅ `maintains_event_bus_api_contract_across_boundaries`
*   ❌ `test_note_1` (non-descriptive)

#### Module Organization
For complex units, group related tests into sub-modules named after the function being tested. This improves IDE navigation and provides structured test output (e.g., `process_note::should_fail_when_limit_exceeded`).

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod process_note {
        use super::*;

        #[test]
        fn should_return_blob_when_larger_than_limit() { ... }

        #[test]
        fn should_fail_when_frontmatter_is_malformed() { ... }
    }
}
```

### Behavioral Rules

1.  **One Behavior per Test**: Describe exactly one thing the unit does. If you find yourself using `and` in a test name, consider splitting it. This makes it easier to understand why a test is failing.
2.  **Single Assertion Preference**: Ideally, use one assertion per test. If you are testing separate behaviors, make multiple tests. To avoid boilerplate for multiple inputs, use `rstest` cases with descriptive labels.
3.  **Explicit Failure Messages**: All `assert!` and `assert_eq!` calls should include formatted context. For `Ok` scenarios, always include the `Err` case in the message or use `eprintln` to aid debugging:
    *   `assert!(res.is_ok(), "Expected success, got error: {:?}", res.err());`
4.  **Matches over Equality**: When asserting on complex enums where you only care about the variant, use `matches!`. This avoids excessive boilerplate and keeps tests focused on one behavior.
    *   `assert!(matches!(err, DomainError::Validation(_)), "Expected validation error, found: {:?}", err);`

### Attributes & Metadata

- `#[ignore = "reason"]`: Use for tests that are not yet fully implemented or depend on external environment setup.
- `#[should_panic]`: Use only when panic is the *intended* and *documented* behavior of the API. Prefer returning `Result` over panicking.
- `#[cfg(test)]`: Use to wrap test-only modules or mock implementations to exclude them from the production binary.

### Advanced Verification

- **Observability**: Use `TestTracingSubscriber` to verify emitted spans and events.
- **Property Testing**: Use `Proptest` for mathematical edge cases and state transition verification.
- **Error Assertions**: Use the `assert_err_kind!` macro for standardized error matching.
- **Domain Purity**: Programmatic enforcement ensures `lithos-domain` remains free of I/O dependencies.

### Snapshot Testing (insta)

Snapshots are for visual or structural correctness (CLI output, ASTs, complex JSON/YAML).

*   **YAML Snapshots**: Use the `yaml` feature for human-readable diffs in git.
*   **Named Snapshots**: Always provide a name: `assert_yaml_snapshot!("note_v1", metadata);`.
*   **Redactions**: Always redact unstable fields (UUIDs, timestamps) to ensure snapshots remain deterministic.
*   **What NOT to snapshot**:
    *   Simple types or primitives (use `assert_eq!`).
    *   Critical path logic (use precise unit tests).
    *   External resources (use mocks).

## 6. Doc-Tests (Executable Examples)

Doc-tests turn your `/// # Examples` into compiler-verified tests.

- **Living Documentation**: They show how functions are meant to be used. Targeted tests are often more helpful than reading the function body.
- **Duplication is OK**: It is acceptable to duplicate logic between doc-tests and unit tests if it improves documentation clarity.
- **Hide Boilerplate**: Use `#` at the start of a line to hide setup code (like imports) from the generated documentation while keeping it in the executable test.
- **Attributes**:
    - `no_run`: Compiles the example but doesn't execute it (ideal for side-effect heavy code).
    - `compile_fail`: Verifies that the code *cannot* compile (ideal for demonstrating incorrect API usage).
    - `should_panic`: Tells the compiler that this example block will panic.

## 7. Streamlining with `lithos-test-utils`

The `lithos-test-utils` crate is the "Core OS" for testing in Lithos.

> [!IMPORTANT]
> **Executable Source of Truth**: The examples below are simplified for quick reference. For the full, compiler-verified API documentation and advanced usage, run `mise run test:unit -p test-utils` or view the rustdoc for the relevant component in `tests/utils/src/`.

### Filesystem & Vaults
Quickly spin up realistic environments for file-based operations.
```rust
let vault = TestVault::new()
    .with_note("Work/Project.md", "# Project\nStatus: Active")
    .with_config("lithos.toml", "[vault]\nstrict = true")
    .build();

let context = IsolatedTestContext::new("my_test");
// context.temp_dir() is ready for use
```

### Async & Time Control
Streamline async setup and eliminate flakiness in time-sensitive code.
```rust
time_test!(async fn validates_cache_expiry() {
    cache.set("key", "val", Duration::from_secs(60)).await;
    advance(Duration::from_secs(61)).await;
    assert!(cache.get("key").await.is_none());
});
```

### CQRS & Event Verification
Declarative verification of complex business flows and eventual consistency.
```rust
tester.given(initial_events)
    .when(command)
    .then_expect_events(expected_events);
```

## 8. Linting & Code Quality in Tests

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

## 9. Common Pitfalls

- **Thread Starvation**: Blocking the Tokio executor with `std::fs` or `sleep`. Use `spawn_blocking_test` and `tokio::time::sleep`.
- **Race Conditions**: Using single-threaded test flavor for concurrent code. Always use `multi_thread` for integration tests.
- **Flakiness**: Relying on wall-clock time. Use `tokio::time::pause()` and the `advance()` helper for deterministic time-based tests.
- **Shared State**: Using static variables or shared files between tests. Always use fresh fixtures (e.g., `IsolatedTestContext`).

## 10. Resources & Deep-Dives

For tactical implementation details, refer to the following specs:

- [**Async Testing Pattern Spec**](./testing/async.md)
- [**CQRS Testing Pattern Spec**](./testing/cqrs.md)
- [**Event-Driven Testing Pattern Spec**](./testing/event.md)
- [**ADR 0010: Test Utilities**](./adr/0010-centralized-test-utilities.md)
