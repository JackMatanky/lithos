---
title: "Lithos Test Developer Guide (Master Manual)"
description: "Comprehensive reference for testing standards, patterns, and tools in the Lithos project"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Testing & Quality"
---

# Lithos Test Developer Guide (Master Manual)

This guide provides a comprehensive reference for testing standards, patterns, and tools in the Lithos project. For the full architectural strategy, see [\_bmad-output/test-design-system.md](./test-design-system.md).

## 1. Authorized Entry Points (Mise)

All testing tasks MUST be orchestrated via `mise`. This ensures the correct environment variables and toolchains are used.

| Command                       | Action                                                                            |
| :---------------------------- | :-------------------------------------------------------------------------------- |
| `mise run test`               | Run all tests (unit, integration, e2e) (alias: `t`).                              |
| `mise run test:unit`          | Run all unit tests using `nextest` (alias: `tu`).                                 |
| `mise run test:unit:core`     | Run core crate unit tests (alias: `tucore`).                                      |
| `mise run test:unit:cli`      | Run CLI crate unit tests (alias: `tucli`).                                        |
| `mise run test:unit:config`   | Run config module unit tests (alias: `tuconf`).                                   |
| `mise run test:unit:note`     | Run note module unit tests (alias: `tunote`).                                     |
| `mise run test:unit:schema`   | Run schema module unit tests (alias: `tusch`).                                    |
| `mise run test:unit:template` | Run template module unit tests (alias: `tutemp`).                                 |
| `mise run test:unit:db`       | Run db module unit tests (alias: `tudb`).                                         |
| `mise run test:unit:fs`       | Run fs module unit tests (alias: `tufs`).                                         |
| `mise run test:integration`   | Run all integration tests across the workspace (alias: `ti`).                     |
| `mise run test:e2e`           | Run end-to-end tests (alias: `te`).                                               |
| `mise run test:coverage`      | Generate code coverage reports using `tarpaulin` (alias: `tc`).                   |
| `mise run test:bench`         | Run all performance benchmarks using `criterion`.                                 |
| `mise run test:bench:core`    | Run core crate benchmarks (alias: `tbcore`).                                      |
| `mise run test:bench:cli`     | Run CLI crate benchmarks (alias: `tbcli`).                                        |
| `mise run test:watch`         | Watch mode: automatically run tests on file changes (alias: `tw`).                |
| `mise run test:burn-in`       | Run tests repeatedly to detect flaky failures (alias: `tb`).                      |
| `mise run test:changed`       | Run tests only for crates affected by changes (alias: `tc`).                      |
| `mise run verify`             | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |

## 2. Tools & Infrastructure

Lithos leverages standard Rust testing tools to ensure speed and reliability.

- **nextest**: Primary test runner for concurrent execution. Faster and more robust than raw `cargo test`.
- **tarpaulin**: Used for coverage analysis (Target: 80%+).
- **criterion**: Mandatory for NFR-critical paths like Indexing and Rendering.
- **proptest**: Property-based testing for edge case discovery and fuzz testing.
- **rstest**: Parameterized tests with named test cases for better test organization.
- **tempfile**: Temporary file/directory creation for filesystem tests (cleanup is automatic).
- **doc tests**: Mandatory for all public domain models. Use as "living documentation."

**Note:** Lithos uses **standard Rust testing patterns** only. All test utilities are inline within test modules using `#[cfg(test)]`. No external test utility crates are used.

## 3. Testing Hierarchy

Lithos follows a hexagonal testing strategy to ensure coverage across all layers of the architecture while maintaining fast execution. We distinguish between three primary sets of tests:

### Unit Tests

Tests that go in the **same module** as the tested unit. This allows visibility over private functions and parent `use` declarations.

- **Focus**: Implementation details, edge-cases, and internal logic.
- **Tools**: `cargo test`, `proptest`.
- **Rule**: KISS (Keep It Simple, Stupid). Test one state and one behavior.

### Integration Tests

Tests that live in `lithos-core/tests/` (when present). They are external to the library and can only test the **public API**.

- **Focus**: Verifying that multiple parts of the system work together correctly.
- **Tools**: `nextest`, `mockall`.
- **Note**: External states and side-effects are permitted here.

### Doc Tests

Executable examples within the source code using `///`.

- **Focus**: Happy paths and general public API usage.
- **Orchestration**: Run via `mise run test:unit`.

| Layer                            | Focus                                                     | Location                  | Tools                |
| :------------------------------- | :-------------------------------------------------------- | :------------------------ | :------------------- |
| **Domain (Unit)**                | Business logic, state transitions, conversions. Zero I/O. | `lithos-core/src/**/*.rs` | `mise run test:unit` |
| **Application (Integration)**    | Cross-module orchestration, port contracts, event flows.  | `lithos-core/src/**/*.rs` | `nextest`            |
| **Infrastructure (Integration)** | Adapters, persistence, external APIs.                     | `lithos-core/tests/`      | `nextest`            |
| **CLI (E2E)**                    | End-to-end user flows, binary execution.                  | `lithos-cli/src/**/*.rs`  | `assert_cmd`         |

## 4. Safety Invariants

### Sync-First Architecture

Lithos follows a **sync-first architecture**. The core domain and business logic is entirely synchronous with no async dependencies.

- **Zero async in domain**: `lithos-core` has zero async dependencies (no `tokio`, `async-trait`, etc.)
- **Synchronous tests**: All tests in `lithos-core` are standard synchronous Rust tests
- **Filesystem operations**: Use `std::fs` and `std::io` directly (no async file I/O)
- **Database operations**: `redb` and `moka` are synchronous libraries with no async overhead

### Determinism

- **Fixed Seeds**: Use deterministic seeds for any randomness or UUID generation in fixtures
- **Proptest seeds**: Use `.prop_with_config()` to set deterministic seeds for property tests
- **Temporary directories**: Use `tempfile::TempDir` which provides automatic cleanup via RAII

## 5. Test Authoring Standards

### Fixture Placement & Best Practices

- **Inline fixtures**: Define test fixtures directly within `#[cfg(test)] mod tests` blocks in the same file as the implementation
- **Helper functions**: Create simple test helper functions (not macros) when setup is repeated across multiple tests
- **Proptest strategies**: Define strategies inline using string patterns or `prop_compose!` within test modules
- **No external test crates**: All test utilities are self-contained within the module being tested
- **Avoid shared test infrastructure**: Each test module is independent and can be understood in isolation

**Why inline fixtures?**

1. **Locality**: Tests and fixtures remain close to the code they test
2. **Simplicity**: No need to navigate multiple files to understand test setup
3. **Independence**: No coupling between test modules through shared utilities
4. **Maintainability**: When code changes, related tests and fixtures are in the same file

Tests are the first place people look to understand how your code works. They must be clear, targeted, and serve as **Living Documentation**.

### Naming Conventions & Organization

We follow a **Verb-First** and **Module-Per-Function** organization pattern.

#### The Naming Formula

A good test name reveals: `unit_of_work` + `expected_behavior` + `state_under_test`.

- ✅ `returns_error_when_vault_path_is_invalid`
- ✅ `maintains_event_bus_api_contract_across_boundaries`
- ❌ `test_note_1` (non-descriptive)

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

##### Module Organization Decision Tree

1. Is this shared setup only? Use `fixtures`.
2. Is this property-based? Use `proptests`.
3. Is this a command (write path)? Use a command module name.
4. Is this a query (read path)? Use a query module name.
5. Otherwise, use the most specific core structure module name.

**Fixtures + Property-Based**

| Name        | Use when testing          |
| :---------- | :------------------------ |
| `fixtures`  | shared setup helpers only |
| `proptests` | all proptest suites       |

**Core Structure**

| Name          | Use when testing                       |
| :------------ | :------------------------------------- |
| `constructor` | `new`, `try_new`, constructors         |
| `builders`    | builder APIs, fluent construction      |
| `defaults`    | `Default` impls and baseline config    |
| `validation`  | field/rule validation failures/success |
| `invariants`  | cross-field consistency rules          |
| `state`       | state transitions, lifecycle flags     |
| `accessors`   | getters, derived values                |
| `conversions` | `From`/`TryFrom`/`Into`                |
| `borrowing`   | zero-copy/borrowed accessors, guards   |
| `formatting`  | `Display`/`Debug` output (combined)    |
| `equality`    | `Eq`/`PartialEq` expectations          |
| `ordering`    | `Ord`/`PartialOrd` behavior            |
| `hashing`     | `Hash` behavior as map/set key         |
| `cloning`     | `Clone` behavior                       |

**Commands**

| Name             | Use when testing                |
| :--------------- | :------------------------------ |
| `create`         | create command behavior         |
| `update`         | update command behavior         |
| `delete`         | delete command behavior         |
| `upsert`         | insert/update semantics         |
| `rename`         | rename/retitle flows            |
| `link`           | link/relationship creation      |
| `unlink`         | link/relationship removal       |
| `assign`         | ownership/association addition  |
| `unassign`       | ownership/association removal   |
| `merge`          | merge command semantics         |
| `event_emission` | events emitted by commands      |
| `persistence`    | DB effects specific to commands |

**Queries**

| Name             | Use when testing             |
| :--------------- | :--------------------------- |
| `find_by_id`     | lookup by id                 |
| `find_by_name`   | lookup by name               |
| `find_by_path`   | lookup by path               |
| `find_by_tag`    | lookup by tag                |
| `list`           | list subset/default list     |
| `list_all`       | list everything              |
| `list_by_parent` | list by parent/owner         |
| `search`         | general search               |
| `search_text`    | free-text search             |
| `resolve`        | derived/linked results       |
| `indices`        | index-driven lookup behavior |
| `pagination`     | limits/offsets/cursors       |

### Behavioral Rules

1.  **One Behavior per Test**: Describe exactly one thing the unit does. If you find yourself using `and` in a test name, consider splitting it. This makes it easier to understand why a test is failing.
2.  **Single Assertion Preference**: Ideally, use one logical assertion per test to make failures easy to diagnose. If you are testing separate behaviors, create multiple tests.
3.  **Parameterized Tests (rstest)**: To avoid boilerplate when testing the same behavior with multiple inputs, use `rstest` with **Named Cases**. This ensures each input is reported as a separate, identifiable test by `nextest`.

    ```rust
    #[rstest]
    #[case::single_char("a")]
    #[case::starts_with_a("ab")]
    #[case::ends_with_a("ba")]
    fn should_accept_strings_containing_a(#[case] input: &str) {
        assert!(the_function(input).is_ok(), "Failed to accept valid input: {}", input);
    }
    ```

4.  **Explicit Failure Messages**: All `assert!` and `assert_eq!` calls should include formatted context. For `Ok` scenarios, always include the `Err` case in the message or use `eprintln` to aid debugging:
    - `assert!(res.is_ok(), "Expected success, got error: {:?}", res.err());`
5.  **Matches over Equality**: When asserting on complex enums where you only care about the variant, use `matches!`. This avoids excessive boilerplate and keeps tests focused on one behavior.
    - `assert!(matches!(err, DomainError::Validation(_)), "Expected validation error, found: {:?}", err);`

### Attributes & Metadata

- `#[ignore = "reason"]`: Use for tests that are not yet fully implemented or depend on external environment setup.
- `#[should_panic]`: Use only when panic is the _intended_ and _documented_ behavior of the API. Prefer returning `Result` over panicking.
- `#[cfg(test)]`: Use to wrap test-only modules or mock implementations to exclude them from the production binary.

### Advanced Verification

- **Observability**: Use `tracing-test` or a custom subscriber to verify emitted spans and events.
- **Property Testing**: Use `Proptest` for mathematical edge cases and state transition verification.
- **Error Assertions**: Use `matches!` and explicit error assertions for standardized matching.
- **Domain Purity**: Programmatic enforcement ensures `lithos-core` domain contexts remain free of I/O dependencies.

### Snapshot Testing

**Note:** Lithos currently does NOT use snapshot testing. We prefer explicit assertions for all test verification.

**Why no snapshots?**

1. **Explicit assertions**: Tests using `assert!`, `assert_eq!`, and `matches!` are clearer and more maintainable
2. **Debugging**: Explicit assertions show exactly what failed and why
3. **Review friction**: Snapshot diffs in PRs require careful review to catch regressions
4. **Determinism**: Snapshot tests can hide timing issues and non-deterministic output

**If snapshot testing is added in the future:**

- Use for structural correctness only (CLI output, complex JSON/YAML)
- Always redact unstable fields (UUIDs, timestamps)
- Prefer explicit assertions for critical path logic

## 6. Doc-Tests (Executable Examples)

Doc-tests turn your `/// # Examples` into compiler-verified tests.

- **Living Documentation**: They show how functions are meant to be used. Targeted tests are often more helpful than reading the function body.
- **Duplication is OK**: It is acceptable to duplicate logic between doc-tests and unit tests if it improves documentation clarity.
- **Hide Boilerplate**: Use `#` at the start of a line to hide setup code (like imports) from the generated documentation while keeping it in the executable test.
- **Attributes**:
  - `no_run`: Compiles the example but doesn't execute it (ideal for side-effect heavy code).
  - `compile_fail`: Verifies that the code _cannot_ compile (ideal for demonstrating incorrect API usage).
  - `should_panic`: Tells the compiler that this example block will panic.

## 7. Standard Rust Testing Patterns

Lithos uses **idiomatic Rust testing patterns** without external test utility crates. All test infrastructure is inline and self-contained.

### Simple Test Fixtures

Create helper functions within test modules for common setup:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a test note with custom fields
    fn note_fixture(id: Uuid, path: &str, tags: Vec<Tag>) -> Note {
        Note {
            id,
            path: NotePath::new(path.to_owned()).expect("Valid test path"),
            frontmatter: None,
            links: vec![],
            tags,
            headings: vec![],
            tasks: vec![],
            sections: vec![],
            pending_events: vec![],
        }
    }

    #[test]
    fn validates_note_structure() {
        let note = note_fixture(
            Uuid::now_v7(),
            "test.md",
            vec![Tag::new("#work").expect("Valid tag")]
        );
        assert!(note.validate().is_ok());
    }
}
```

### Filesystem Testing

Use `tempfile` for automatic cleanup of test directories:

```rust
use tempfile::TempDir;

#[test]
fn writes_file_successfully() -> std::io::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    std::fs::write(&file_path, "test content")?;
    let content = std::fs::read_to_string(&file_path)?;

    assert_eq!(content, "test content");
    Ok(())
    // temp_dir is automatically cleaned up when dropped
}
```

### Property-Based Testing

Define strategies inline using regex patterns or `prop_compose!`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn validates_identifier_format(
        name in "[a-zA-Z0-9_-]{1,64}"
    ) {
        let result = PropertyName::new(name);
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_invalid_characters(
        name in ".*[^a-zA-Z0-9_-].*".prop_filter(
            "valid length",
            |s: &String| !s.is_empty() && s.len() <= 64
        )
    ) {
        let result = PropertyName::new(name);
        assert!(result.is_err());
    }
}
```

## 8. Linting & Code Quality in Tests

Lithos maintains strict quality gates even for test code. While tests have more latitude than business logic, they must still be modular and readable. The goal is to **fix clippy issues properly** rather than suppress them with `#[expect(...)]` attributes.

### `#expect` vs `#allow` Guidelines

- **Use `#[expect(...)]`**: For intentional lint violations that are necessary for the test (e.g., using `unwrap` in a setup block or creating a complex fixture that exceeds cognitive complexity limits). This tells the compiler "I know this violates a rule, but it's intentional."
- **Use `#[allow(...)]`**: Primarily for generated code (e.g., `automock`) where the developer doesn't control the output. Avoid using `allow` for hand-written test logic.

### Common Clippy Issues and How to Fix Them

Instead of using `#[expect(...)]`, prioritize fixing the underlying issue. Here are common patterns and their solutions:

#### 1. **Cognitive Complexity Too High**

**Problem:** Functions exceed 15 cyclomatic complexity or 25 cognitive complexity.

```rust
#[test]  // clippy::cognitive_complexity flagged
fn test_complex_business_logic() {
    // 20+ conditional branches, loops, etc.
    let result = complex_function(param1, param2, param3);
    assert!(result.is_ok());
}
```

**Solutions:**

- **Extract helper functions:** Break complex tests into smaller, focused tests
- **Use table-driven tests:** Parameterize common logic
- **Split test scenarios:** One behavior per test function

```rust
// ✅ FIXED: Split into focused tests
#[test]
fn returns_ok_for_valid_inputs() {
    let result = complex_function("valid", 42, true);
    assert!(result.is_ok());
}

#[test]
fn returns_error_for_invalid_param1() {
    let result = complex_function("invalid", 42, true);
    assert!(matches!(result, Err(Error::InvalidParam(_))));
}

#[test]
fn returns_error_for_zero_param2() {
    let result = complex_function("valid", 0, true);
    assert!(matches!(result, Err(Error::ZeroValue(_))));
}

// For complex setup, extract to helper function
fn setup_complex_scenario() -> (Param1, Param2, Param3) {
    // Complex setup logic here
    ("scenario1", 100, false)
}

#[test]
fn handles_complex_scenario_correctly() {
    let (p1, p2, p3) = setup_complex_scenario();
    let result = complex_function(p1, p2, p3);
    assert!(result.is_ok());
}
```

#### 2. **Too Many Arguments**

**Problem:** Functions with more than 7 parameters.

```rust
#[test]  // clippy::too_many_arguments flagged
fn test_with_many_params() {
    let result = function_with_many_args(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
    assert!(result.is_ok());
}
```

**Solutions:**

- **Use builder pattern:** Create a test builder struct
- **Group related parameters:** Use tuples or small structs
- **Extract fixture functions:** Pre-configure common parameter combinations

```rust
// ✅ FIXED: Builder pattern for test setup
struct TestFixture {
    arg1: Type1,
    arg2: Type2,
    arg3: Type3,
    arg4: Type4,
    arg5: Type5,
    arg6: Type6,
    arg7: Type7,
    arg8: Type8,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            arg1: default_value(),
            arg2: default_value(),
            arg3: default_value(),
            arg4: default_value(),
            arg5: default_value(),
            arg6: default_value(),
            arg7: default_value(),
            arg8: default_value(),
        }
    }

    fn with_arg1(mut self, value: Type1) -> Self {
        self.arg1 = value;
        self
    }
}

#[test]
fn test_with_builder() {
    let fixture = TestFixture::new()
        .with_arg1(special_value());
    let result = function_with_many_args(
        fixture.arg1, fixture.arg2, fixture.arg3, fixture.arg4,
        fixture.arg5, fixture.arg6, fixture.arg7, fixture.arg8
    );
    assert!(result.is_ok());
}
```

#### 3. **Manual `assert!` Instead of Specific Macros**

**Problem:** Using generic `assert!` when specific assertion macros exist.

```rust
#[test]  // clippy::manual_assert flagged
fn test_result() {
    let result = some_function();
    assert!(result.is_ok());  // Too generic
}
```

**Solutions:**

- **Use Result assertions:** `assert!(result.is_ok())` → specific Result assertions
- **Use type-specific assertions:** Leverage `pretty_assertions` for better diffs

```rust
// ✅ FIXED: Use specific assertions
use pretty_assertions::assert_eq;

#[test]
fn test_result_success() {
    let result = some_function();
    assert!(result.is_ok(), "Expected success, got error: {:?}", result.err());
}

#[test]
fn test_struct_equality() {
    let actual = create_struct();
    let expected = Struct { field1: "value", field2: 42 };
    assert_eq!(actual, expected);  // pretty_assertions provides colorful diffs
}
```

#### 4. **Unnecessary `collect()`**

**Problem:** Using `collect()` when it's not needed.

```rust
#[test]  // clippy::needless_collect flagged
fn test_iteration() {
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<_> = data.iter().filter(|x| *x > 3).collect();
    assert_eq!(result.len(), 2);
}
```

**Solutions:**

- **Use iterator methods:** Replace `collect()` with direct iteration
- **Use `count()` for counting:** When only length matters

```rust
// ✅ FIXED: Remove unnecessary collect
#[test]
fn test_iteration_count() {
    let data = vec![1, 2, 3, 4, 5];
    let count = data.iter().filter(|x| *x > 3).count();
    assert_eq!(count, 2);
}

#[test]
fn test_iteration_values() {
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<_> = data.iter().filter(|x| *x > 3).copied().collect();
    assert_eq!(result, vec![4, 5]);
}
```

#### 5. **Shadowing Variables**

**Problem:** Shadowing variables makes code confusing.

```rust
#[test]  // clippy::shadow_unrelated flagged
fn test_shadowing() {
    let config = create_config();
    let config = modify_config(config);  // Shadows original
    assert!(config.is_modified);
}
```

**Solutions:**

- **Use different variable names:** Avoid shadowing entirely
- **Use mut:** If modification is intended

```rust
// ✅ FIXED: Use mut or different names
#[test]
fn test_modification() {
    let config = create_config();
    let modified_config = modify_config(config);
    assert!(modified_config.is_modified);
}

#[test]
fn test_in_place_modification() {
    let mut config = create_config();
    config.modify_in_place();
    assert!(config.is_modified);
}
```

#### 6. **Missing Error Documentation**

**Problem:** Functions that can fail don't document their error conditions.

```rust
#[test]  // clippy::missing_errors_doc flagged
fn test_error_case() {
    let result = function_that_can_fail();
    assert!(result.is_err());
}
```

**Solutions:**

- **Document error conditions:** Add error documentation to the tested function
- **Use proper error assertions:** Make error expectations explicit

```rust
// ✅ FIXED: Document error conditions in function under test
/// # Errors
/// Returns `Error::Validation` if input is invalid
fn function_that_can_fail() -> Result<(), Error> {
    // implementation
}

// Test becomes more specific
#[test]
fn returns_validation_error_for_invalid_input() {
    let result = function_that_can_fail();
    assert!(matches!(result, Err(Error::Validation(_))));
}
```

### Workflow for Fixing Clippy Lints

The preferred workflow for handling clippy violations in tests is:

1. **Run clippy**: `mise run lint` (or `cargo clippy --workspace --tests`)
2. **Read the diagnostic**: Clippy often provides the exact code change needed.
3. **Apply the suggestion**: Use `cargo clippy --fix` for simple lint fixes.
4. **Refactor for complexity**: If the lint flags complexity or argument counts, extract helper functions or use the builder pattern as shown above.
5. **Verify**: Run `mise run verify` to ensure all quality gates pass.

### Additional Common Fixes

#### 7. Panic in Result-returning Function

**Problem:** Using `panic!`, `unwrap`, or `expect` in a function that returns `Result`.

```rust
fn validate_input(input: &str) -> Result<(), Error> {
    if input.is_empty() {
        panic!("Input cannot be empty"); // clippy::panic_in_result_fn flagged
    }
    Ok(())
}
```

**Solution:** Return an `Err` variant instead of panicking.

```rust
// ✅ FIXED: Return Err
fn validate_input(input: &str) -> Result<(), Error> {
    if input.is_empty() {
        return Err(Error::EmptyInput);
    }
    Ok(())
}
```

#### 8. Large Enum Variant

**Problem:** One variant of an enum is much larger than others, causing memory inefficiency.

```rust
enum DomainEvent {
    Started,
    NoteIndexed(Note), // If Note is very large, this variant is flagged
}
```

**Solution:** Box the large variant data.

```rust
// ✅ FIXED: Box the large data
enum DomainEvent {
    Started,
    NoteIndexed(Box<Note>),
}
```

#### 9. Improper use of `unwrap()` or `expect()` in Assertions

**Problem:** Using `unwrap()` or `expect()` on a Result that is the primary target of verification. This hides the actual error variant and produces poor diagnostic output when the test fails.

```rust
#[test]
fn should_validate_successfully() {
    let result = validate_something();
    result.expect("Validation should pass"); // ❌ ANTI-PATTERN
}
```

**Solutions:**

- **Use `assert!(result.is_ok())`**: For simple success verification.
- **Include error context**: Provide the error in the assertion message for easier debugging.
- **Use `matches!` for variants**: When verifying specific error types.

```rust
// ✅ FIXED: Use explicit assertions with context
#[test]
fn should_validate_successfully() {
    let result = validate_something();
    assert!(
        result.is_ok(),
        "Validation should pass, but failed with: {:?}",
        result.err()
    );
}

#[test]
fn returns_specific_error_on_failure() {
    let result = validate_something_invalid();
    // ✅ FIXED: Use matches! for specific error verification
    assert!(
        matches!(result, Err(DomainError::InvalidInput(_))),
        "Expected InvalidInput error, but got: {:?}",
        result
    );
}
```

### unwrapping in Tests Guidelines

To maintain high quality tests, follow this rule of thumb for `unwrap()` and `expect()`:

1.  **Arrange (Setup) Phase**: `unwrap()` is **PERMITTED**. If setup fails, the test should panic immediately because the test prerequisites weren't met.
2.  **Act Phase**: `unwrap()` is **FORBIDDEN**. The result of the action should be captured.
3.  **Assert (Then) Phase**: `unwrap()` is **FORBIDDEN**. Use explicit `assert!` macros to verify outcomes.

```rust
#[test]
fn processes_valid_note() {
    // 1. ARRANGE: unwrap() is OK here
    let note = create_test_note().unwrap();

    // 2. ACT: capture the result
    let result = process_note(note);

    // 3. ASSERT: use assert! instead of unwrap()
    assert!(result.is_ok(), "Process failed: {:?}", result.err());
}
```

## 9. Common Pitfalls

- **Overly complex tests**: Tests with high cognitive complexity should be split into smaller, focused tests
- **Hidden assertions**: Avoid test helpers that hide assertions - keep assertions visible in the test body
- **Shared mutable state**: Using static variables or shared files between tests. Always use fresh fixtures per test
- **Non-deterministic tests**: Avoid relying on system time, random values without seeds, or undefined ordering
- **Unwrap in assertions**: Use explicit `assert!(result.is_ok(), "...")` instead of `result.unwrap()` for better error messages
- **Testing implementation details**: Focus on behavior and contracts, not internal implementation
- **Builder pattern overuse**: Simple helper functions are often clearer than complex test builders

## 10. Resources & Reference

### Official Rust Documentation

- [The Rust Book - Chapter 11: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [The rustc Book - Tests](https://doc.rust-lang.org/rustc/tests/index.html)

### Testing Tools Documentation

- [nextest Documentation](https://nexte.st/) - Fast, parallel test runner
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Code coverage
- [proptest Book](https://proptest-rs.github.io/proptest/intro.html) - Property-based testing
- [rstest Documentation](https://docs.rs/rstest/) - Parameterized tests
- [criterion.rs](https://bheisler.github.io/criterion.rs/book/) - Benchmarking

### Lithos-Specific Documentation

- [System-Level Test Design](./test-design-system.md) - Overall testing strategy
- [AGENTS.md](../AGENTS.md) - AI agent testing guidelines
