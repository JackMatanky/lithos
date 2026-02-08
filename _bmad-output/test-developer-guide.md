---
title: "Lithos Test Developer Guide (Master Manual)"
description: "Comprehensive reference for testing standards, patterns, and tools in the Lithos project"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-08"
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
| `integrity`   | structural consistency checks          |
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
| `load`           | load/aggregate query results |
| `list`           | list subset/default list     |
| `list_all`       | list everything              |
| `list_by_parent` | list by parent/owner         |
| `search`         | general search               |
| `search_text`    | free-text search             |
| `resolve`        | derived/linked results       |
| `indices`        | index-driven lookup behavior |
| `pagination`     | limits/offsets/cursors       |

**Naming Notes**

- Prefer singular names: use `constructor`, `validation`, `builder` forms over plurals.
- Normalize common variants: `validate`/`validators` → `validation`, `constructors` → `constructor`.
- Keep unit-specific submodules (e.g., `property_bank`, `field_value`, `heading`, `section`) out of the canonical list.

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

### Doc Test Best Practices

#### Basic Example with Hidden Setup

```rust
/// Parses a note path from a string.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::NotePath;
/// let path = NotePath::new("notes/hello.md".to_owned()).unwrap();
/// assert_eq!(path.as_str(), "notes/hello.md");
/// ```
pub fn new(path: String) -> Result<Self, NoteError> {
    // implementation
}
```

#### `no_run` for Side-Effect Heavy Code

Use `no_run` when the code has side effects (file I/O, network calls) that shouldn't execute during doc tests:

```rust
/// Deletes a note from the vault.
///
/// # Examples
///
/// ```no_run
/// use lithos_core::note::NoteRepository;
/// use std::path::Path;
///
/// let repo = NoteRepository::open(Path::new("/path/to/vault")).unwrap();
/// repo.delete("notes/old.md").unwrap();
/// ```
pub fn delete(&self, path: &str) -> Result<(), NoteError> {
    // implementation
}
```

#### `compile_fail` for Invalid API Usage

Use `compile_fail` to document APIs that should not compile:

```rust
/// Creates a new validated tag.
///
/// Tags must start with `#` and contain only alphanumeric characters.
///
/// # Examples
///
/// ```
/// use lithos_core::note::Tag;
///
/// let tag = Tag::new("#work").unwrap();
/// assert_eq!(tag.as_str(), "#work");
/// ```
///
/// The following will **not** compile because the tag doesn't start with `#`:
///
/// ```compile_fail
/// use lithos_core::note::Tag;
///
/// // This fails to compile - tag must start with `#`
/// let tag = Tag::new("invalid").unwrap();
/// ```
pub fn new(tag: &str) -> Result<Self, TagError> {
    // implementation
}
```

#### `should_panic` for Expected Panics

Use `should_panic` when demonstrating code that intentionally panics:

```rust
/// Unwraps a value, panicking if None.
///
/// # Examples
///
/// ```
/// use lithos_core::utils::unwrap_or_panic;
///
/// let value = Some(42);
/// assert_eq!(unwrap_or_panic(value), 42);
/// ```
///
/// This will panic:
///
/// ```should_panic
/// use lithos_core::utils::unwrap_or_panic;
///
/// let value: Option<i32> = None;
/// unwrap_or_panic(value); // panics!
/// ```
pub fn unwrap_or_panic<T>(opt: Option<T>) -> T {
    opt.expect("value was None")
}
```

#### Testing Private Functions via Module Re-export

For doc tests that need access to internal types:

```rust
/// Internal utilities for testing.
///
/// # Examples
///
/// ```
/// // Access internal test helpers through the test module
/// use lithos_core::note::test_helpers::create_test_note;
///
/// let note = create_test_note("test.md");
/// assert!(!note.content().is_empty());
/// ```
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Creates a test note with default values.
    pub fn create_test_note(path: &str) -> Note {
        Note {
            // ... setup
        }
    }
}
```

#### Multiple Examples for Different Scenarios

```rust
/// Validates a property name.
///
/// Property names must:
/// - Be 1-64 characters long
/// - Contain only alphanumeric characters, underscores, and hyphens
/// - Start with a letter
///
/// # Examples
///
/// Basic valid usage:
///
/// ```
/// use lithos_core::schema::PropertyName;
///
/// let name = PropertyName::new("valid_name-123").unwrap();
/// assert_eq!(name.as_str(), "valid_name-123");
/// ```
///
/// Single character names are valid:
///
/// ```
/// use lithos_core::schema::PropertyName;
///
/// let name = PropertyName::new("a").unwrap();
/// assert_eq!(name.as_str(), "a");
/// ```
///
/// This will fail to compile (invalid start character):
///
/// ```compile_fail
/// use lithos_core::schema::PropertyName;
///
/// // Won't compile - 123 is not a valid identifier
/// let name = PropertyName::new("123invalid").unwrap();
/// ```
pub fn new(name: &str) -> Result<Self, ValidationError> {
    // implementation
}
```

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

## 10. Nextest Configuration

Nextest is Lithos's primary test runner. Configure it via `.config/nextest.toml` for consistent test execution across environments.

### Configuration File Location

Create `.config/nextest.toml` in the project root:

```toml
# .config/nextest.toml - Nextest configuration for Lithos

[profile.default]
# Run tests with up to 8 threads (adjust based on CI/local hardware)
test-threads = "num-cpus"

# Fail fast on first failure for local development
fail-fast = true

# Mark tests as slow after 60 seconds
slow-timeout = "60s"

# Retry flaky tests up to 2 times
retries = 2

# Show test output on failure
failure-output = "immediate"

[profile.ci]
# Run all tests regardless of failure for CI
fail-fast = false

# More aggressive retries for CI environments
retries = { backoff = "exponential", count = 3, delay = "1s" }

# Mark tests as slow after 120 seconds in CI
slow-timeout = "120s"

# Store success output for CI artifact analysis
success-output = "final"

# JUnit XML report for CI integration
junit = { path = "junit.xml" }

[profile.ci.overrides]
# Network tests may need more time
filter = 'test(/\btest_network_/'
slow-timeout = "300s"
retries = 5

[[profile.ci.overrides]]
# Filesystem tests on macOS can be slower
platform = 'cfg(target_os = "macos")'
filter = 'test(/\btest_fs_/'
slow-timeout = "180s"

[profile.stress]
# For burn-in testing - run tests repeatedly
test-threads = 1
fail-fast = false
retries = 0
slow-timeout = "600s"

[profile.fast]
# Quick feedback profile - skip slow tests
default-filter = 'not test(/\bslow_/'
```

### Profile Usage

```bash
# Use default profile (automatic)
mise run test:unit

# Use CI profile (more retries, JUnit output)
cargo nextest run --profile ci

# Use stress profile for burn-in testing
cargo nextest run --profile stress --test-threads 1

# Use fast profile for quick feedback
cargo nextest run --profile fast
```

### Test Groups

For tests that cannot run in parallel (e.g., tests using a shared database):

```toml
[test-groups]
serial = { max-threads = 1 }
db = { max-threads = 4 }  # Limit database connections

[[profile.default.overrides]]
filter = 'test(/\btest_db_/'
test-group = "db"

[[profile.default.overrides]]
filter = 'test(/\btest_serial_/'
test-group = "serial"
```

### Timeout Configuration

Per-test timeout overrides:

```toml
[[profile.default.overrides]]
filter = 'test(/\btest_network_/'
slow-timeout = { period = "30s", terminate-after = 2 }
# Tests taking >30s are marked slow
# Tests taking >60s are terminated
```

## 11. rstest: Fixtures and Parameterized Tests

`rstest` provides powerful fixtures and parameterized testing capabilities that reduce boilerplate while improving test clarity.

### Basic Fixture Usage

Define reusable fixtures with the `#[fixture]` attribute:

```rust
use rstest::*;

// A simple fixture returning a fixed value
#[fixture]
fn valid_note_id() -> Uuid {
    Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
}

// A fixture that depends on another fixture
#[fixture]
fn valid_note_path() -> NotePath {
    NotePath::new("notes/test.md".to_owned()).unwrap()
}

// Fixtures can use other fixtures as parameters
#[fixture]
fn test_note(valid_note_id: Uuid, valid_note_path: NotePath) -> Note {
    Note {
        id: valid_note_id,
        path: valid_note_path,
        content: "Test content".to_owned(),
        tags: vec![],
    }
}

// Use fixtures in tests by naming them as parameters
#[rstest]
fn should_validate_note(test_note: Note) {
    assert!(test_note.validate().is_ok());
}

#[rstest]
fn should_extract_note_id(test_note: Note) {
    assert_eq!(test_note.id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}
```

### Fixture with Default Values

```rust
#[fixture]
fn note_with_tags(
    #[default(Uuid::now_v7())] id: Uuid,
    #[default("notes/default.md")] path: &str,
    #[default(vec!["#test".to_owned()])] tags: Vec<String>,
) -> Note {
    Note {
        id,
        path: NotePath::new(path.to_owned()).unwrap(),
        content: "Content".to_owned(),
        tags: tags.iter()
            .map(|t| Tag::new(t).unwrap())
            .collect(),
    }
}

// Use default values
#[rstest]
fn test_with_defaults(note_with_tags: Note) {
    assert_eq!(note_with_tags.tags.len(), 1);
}

// Override specific defaults using #[with(...)]
#[rstest]
fn test_with_custom_tags(#[with(vec!["#work".to_owned(), "#urgent".to_owned()])] note_with_tags: Note) {
    assert_eq!(note_with_tags.tags.len(), 2);
}
```

### Once Fixtures (Shared Across Tests)

For expensive setup that should run once:

```rust
use rstest::*;
use std::sync::Arc;

// This fixture runs once for all tests in the module
#[fixture]
#[once]
fn shared_database() -> Arc<Database> {
    // Expensive database initialization
    Arc::new(Database::new_test_instance())
}

#[rstest]
fn test_insert(shared_database: &Arc<Database>) {
    shared_database.insert("key", "value").unwrap();
}

#[rstest]
fn test_query(shared_database: &Arc<Database>) {
    // Same database instance as test_insert
    let result = shared_database.query("key").unwrap();
    assert!(result.is_some());
}
```

### Parameterized Tests with Named Cases

Use `#[case]` for table-driven tests with clear names:

```rust
#[rstest]
#[case::valid_simple("hello", true)]
#[case::valid_with_underscore("hello_world", true)]
#[case::valid_with_hyphen("hello-world", true)]
#[case::valid_with_numbers("hello123", true)]
#[case::invalid_empty("", false)]
#[case::invalid_starts_with_number("123hello", false)]
#[case::invalid_contains_space("hello world", false)]
#[case::invalid_special_chars("hello@world", false)]
fn test_identifier_validation(#[case] input: &str, #[case] expected: bool) {
    let result = Identifier::new(input);
    assert_eq!(result.is_ok(), expected,
        "Expected {} for input '{}' but got {:?}",
        if expected { "Ok" } else { "Err" },
        input,
        result
    );
}
```

### Combining Values with `#[values]`

Test all combinations of input values:

```rust
#[rstest]
fn test_state_transitions(
    #[values(State::Init, State::Ready, State::Processing)] current: State,
    #[values(Event::Start, Event::Stop, Event::Reset)] event: Event,
) {
    let result = current.handle(event);
    // This generates 3 * 3 = 9 test cases
    assert!(result.is_ok());
}
```

### Async Fixtures

`rstest` works seamlessly with async tests:

```rust
#[fixture]
async fn async_database() -> Database {
    Database::connect("test://localhost").await.unwrap()
}

#[rstest]
#[tokio::test]
async fn test_async_insert(#[future] async_database: Database) {
    let db = async_database.await;
    db.insert("key", "value").await.unwrap();
}
```

### Reusing Test Templates

Use `rstest_reuse` for shared test definitions:

```rust
use rstest::rstest;
use rstest_reuse::{self, *};

// Define a reusable test template
#[template]
#[rstest]
#[case(2, 2, 4)]
#[case(0, 5, 5)]
#[case(-1, 1, 0)]
fn addition_cases(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {}

// Apply the template to different implementations
#[apply(addition_cases)]
fn test_add(a: i32, b: i32, expected: i32) {
    assert_eq!(add(a, b), expected);
}

#[apply(addition_cases)]
fn test_wrapping_add(a: i32, b: i32, expected: i32) {
    assert_eq!(wrapping_add(a, b), expected);
}
```

## 12. Async Testing

While Lithos follows a sync-first architecture, documenting async testing patterns is valuable for completeness and future reference.

### Basic Async Test with Tokio

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

// Multi-threaded runtime for concurrent tests
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_operations() {
    let handles: Vec<_> = (0..10)
        .map(|i| tokio::spawn(async move { process(i).await }))
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### Testing Time-Dependent Code

Use `tokio::time::pause` for deterministic time-based tests:

```rust
#[tokio::test(start_paused = true)]
async fn test_with_mock_time() {
    use tokio::time::{sleep, Duration, Instant};

    let start = Instant::now();

    // This completes immediately due to paused time
    sleep(Duration::from_secs(60)).await;

    assert_eq!(start.elapsed(), Duration::ZERO);
}
```

### Manual Time Control

```rust
#[tokio::test]
async fn test_interval_behavior() {
    use tokio::time::{interval, Duration, Instant};

    tokio::time::pause();

    let mut interval = interval(Duration::from_millis(100));
    let start = Instant::now();

    // First tick is immediate
    interval.tick().await;

    // Advance time by 100ms
    tokio::time::advance(Duration::from_millis(100)).await;
    interval.tick().await;

    assert_eq!(start.elapsed(), Duration::from_millis(100));
}
```

### Async Assertions with tokio-test

```rust
use tokio_test::{assert_pending, assert_ready, assert_ready_ok};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[tokio::test]
async fn test_future_states() {
    let mut fut = some_async_operation();

    // Check if future is pending
    assert_pending!(fut.poll(&mut context));

    // Drive it to completion
    tokio::spawn(drive_to_completion());

    // Now it should be ready
    assert_ready_ok!(fut.poll(&mut context));
}
```

### Testing Streams

```rust
#[tokio::test]
async fn test_stream_producer() {
    use tokio_stream::StreamExt;

    let mut stream = producer_stream();

    assert_eq!(stream.next().await, Some(1));
    assert_eq!(stream.next().await, Some(2));
    assert_eq!(stream.next().await, Some(3));
    assert_eq!(stream.next().await, None);
}
```

### Timeout Testing

```rust
#[tokio::test]
async fn test_with_timeout() {
    use tokio::time::{timeout, Duration};

    let result = timeout(
        Duration::from_secs(1),
        potentially_slow_operation()
    ).await;

    assert!(result.is_ok(), "Operation timed out");
}

#[tokio::test]
async fn test_timeout_expected() {
    use tokio::time::{timeout, Duration};

    let result = timeout(
        Duration::from_millis(10),
        sleep(Duration::from_secs(1))
    ).await;

    assert!(result.is_err(), "Expected timeout");
}
```

### Async Mocking with mockall

```rust
#[automock]
#[async_trait]
trait AsyncRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, Error>;
    async fn save(&self, note: &Note) -> Result<(), Error>;
}

#[tokio::test]
async fn test_service_with_mock() {
    let mut mock = MockAsyncRepository::new();

    mock.expect_find_by_id()
        .with(eq(test_id()))
        .times(1)
        .returning(|_| Ok(Some(test_note())));

    let service = NoteService::new(mock);
    let result = service.get_note(test_id()).await;

    assert!(result.is_ok());
}
```

## 13. Coverage Analysis

Lithos targets 80%+ code coverage. We support multiple coverage tools for flexibility.

### Primary: cargo-llvm-cov (Recommended)

`cargo-llvm-cov` uses LLVM's source-based code coverage for accurate, fast results.

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Basic coverage report
cargo llvm-cov

# HTML report with line coverage
cargo llvm-cov --html --open

# Generate LCOV report for CI integration
cargo llvm-cov --lcov --output-path lcov.info

# Combine with nextest for faster execution
cargo llvm-cov nextest

# Include doctests in coverage (requires nightly)
cargo +nightly llvm-cov --doctests

# Coverage for specific package
cargo llvm-cov -p lithos-core

# Exclude certain paths
cargo llvm-cov --ignore-filename-regex 'tests/|benches/'
```

### Alternative: cargo-tarpaulin

Tarpaulin provides good cross-platform support and multiple output formats:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Basic run
cargo tarpaulin

# HTML report
cargo tarpaulin --out Html

# Generate multiple report formats
cargo tarpaulin --out Xml --out Lcov --out Html

# Use LLVM engine for better accuracy
cargo tarpaulin --engine llvm

# Exclude specific files/patterns
cargo tarpaulin --exclude-files "tests/*,benches/*"

# Run with specific features
cargo tarpaulin --features "integration-tests"

# Skip clean for faster runs (careful with stale data)
cargo tarpaulin --skip-clean
```

### Coverage Configuration in CI

Example GitHub Actions workflow:

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Generate coverage
        run: |
          cargo llvm-cov --no-report nextest
          cargo llvm-cov --no-report --doc
          cargo llvm-cov report --lcov --output-path lcov.info

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true
```

### Coverage Thresholds

Set minimum coverage requirements:

```bash
# Fail if coverage is below 80%
cargo llvm-cov --fail-under-lines 80

# For tarpaulin
cargo tarpaulin --fail-under 80
```

### Coverage Best Practices

1. **Focus on meaningful coverage**: 100% line coverage doesn't guarantee correctness
2. **Cover critical paths**: Prioritize business logic and error handling
3. **Exclude generated code**: Use `--ignore-filename-regex` for proto/generated files
4. **Use coverage to find gaps**: Not as a metric to optimize blindly
5. **Combine with property testing**: Coverage + property tests = robust validation

## 14. Advanced Assertion Patterns

### pretty_assertions

For readable diffs in test failures:

```rust
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn test_complex_struct() {
    let actual = generate_complex_struct();
    let expected = ComplexStruct {
        field1: "value1".to_owned(),
        field2: vec![1, 2, 3, 4, 5],
        field3: HashMap::from([
            ("key1", "value1"),
            ("key2", "value2"),
        ]),
    };

    // Shows colorful, side-by-side diff on failure
    assert_eq!(actual, expected);
}
```

### assert_matches

Use `assert_matches!` for ergonomic enum variant testing:

```rust
use assert_matches::assert_matches;

#[test]
fn test_specific_error_variant() {
    let result = validate_input("");

    // Cleaner than matches! macro
    assert_matches!(result, Err(DomainError::Validation(msg)) if msg.contains("empty"));
}

#[test]
fn test_nested_enum() {
    let event = process_command(cmd);

    assert_matches!(
        event,
        DomainEvent::NoteCreated { id, path } if path.as_str() == "test.md"
    );
}
```

### prop_assert! in Property Tests

Use `prop_assert!` family for better property test failures:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip_serialization(data in any::<Vec<u8>>()) {
        let serialized = serialize(&data);
        let deserialized = deserialize(&serialized);

        // Provides the failing input in the error message
        prop_assert_eq!(data, deserialized);
    }

    #[test]
    fn test_idempotent_operation(s in "[a-z]*") {
        let first = normalize(&s);
        let second = normalize(&first);

        prop_assert_eq!(
            first, second,
            "Normalization should be idempotent"
        );
    }
}
```

### Custom Assertion Helpers

Create domain-specific assertions for clarity:

```rust
// Custom assertion helpers for domain types
#[track_caller]
fn assert_valid_path(result: Result<NotePath, NoteError>) -> NotePath {
    match result {
        Ok(path) => path,
        Err(e) => panic!("Expected valid path, got error: {:?}", e),
    }
}

#[track_caller]
fn assert_validation_error(result: Result<Note, NoteError>, expected_field: &str) {
    match result {
        Err(NoteError::Validation { field, .. }) => {
            assert_eq!(field, expected_field, "Expected validation error for field '{}'", expected_field);
        }
        Ok(_) => panic!("Expected validation error, got Ok"),
        Err(other) => panic!("Expected Validation error, got {:?}", other),
    }
}

// Usage
#[test]
fn test_note_validation() {
    assert_validation_error(
        Note::new("", "content"),
        "path"
    );
}
```

## 15. Test Performance

### Fast Test Guidelines

1. **Unit tests < 10ms**: Individual unit tests should complete quickly
2. **Integration tests < 1s**: Integration test suites should be fast
3. **Avoid I/O in unit tests**: Use mocks instead of actual filesystem/network
4. **Use `tempfile`** for filesystem tests (fast, auto-cleanup)
5. **Pause time** in async tests to avoid real delays

### Parallel Execution

```rust
// Tests in the same file run in parallel by default
// Use nextest for cross-binary parallelism

// For tests that must run serially:
#[test]
#[serial]  // Requires `serial_test` crate
fn test_serial_access() {
    // This test won't run concurrently with other serial tests
}

// Or use nextest test groups (see Nextest Configuration section)
```

### Test Isolation

```rust
// Each test should be independent
// BAD: Tests share mutable state
static mut COUNTER: i32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_increment_again() {
    // May fail depending on test order!
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 2);
}

// GOOD: Each test has its own state
#[test]
fn test_increment_isolated() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

### Profiling Slow Tests

```bash
# Find slow tests with nextest
cargo nextest run --show-times slow

# Generate detailed timing report
cargo nextest run --profile ci --failure-output final

# Run only slow tests
cargo nextest run --filterset 'test(/slow/)'
```

## 16. CI/CD Integration

### GitHub Actions Example

```yaml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Install mise
        uses: jdx/mise-action@v2

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run verification
        run: mise run verify

      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: test-results-${{ matrix.os }}
          path: junit.xml

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Generate coverage
        run: |
          cargo llvm-cov --no-report nextest
          cargo llvm-cov --no-report --doc
          cargo llvm-cov report --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  check-features:
    name: Check Feature Powerset
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-hack
        uses: taiki-e/install-action@cargo-hack

      - name: Check feature powerset
        run: cargo hack check --feature-powerset --no-dev-deps
```

### Pre-commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: fmt
        name: Format
        entry: cargo fmt
        language: system
        types: [rust]
        pass_filenames: false

      - id: clippy
        name: Clippy
        entry: cargo clippy --workspace --tests -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: test
        name: Test
        entry: mise run test:unit
        language: system
        types: [rust]
        pass_filenames: false
```

### Test Matrix Strategy

```yaml
# Test across Rust versions and features
strategy:
  matrix:
    rust: [1.75.0, stable, beta, nightly]
    features: ['', '--all-features']
    exclude:
      # Don't run all-features on MSRV
      - rust: 1.75.0
        features: '--all-features'
```

## 17. Anti-Patterns to Avoid

### 1. Testing Implementation Details

```rust
// ❌ BAD: Testing private implementation
#[test]
fn test_internal_cache_state() {
    let mut service = Service::new();
    service.process("input");

    // Exposing internal state just for tests
    assert_eq!(service.internal_cache.len(), 1);
}

// ✅ GOOD: Test observable behavior
#[test]
fn test_caching_behavior() {
    let service = Service::new();

    // First call processes
    let result1 = service.process("input");
    // Second call uses cache (observable via timing or side effects)
    let result2 = service.process("input");

    assert_eq!(result1, result2);
}
```

### 2. Test Interdependence

```rust
// ❌ BAD: Tests depend on execution order
static mut SETUP_DONE: bool = false;

#[test]
fn test_setup() {
    unsafe { SETUP_DONE = true; }
}

#[test]
fn test_requires_setup() {
    assert!(unsafe { SETUP_DONE }); // Fails if run first!
}

// ✅ GOOD: Each test is self-contained
#[test]
fn test_independent() {
    setup_if_needed();
    // ... test code
}
```

### 3. Excessive Mocking

```rust
// ❌ BAD: Mocking everything
#[test]
fn test_with_too_many_mocks() {
    let mock_db = MockDb::new();
    let mock_cache = MockCache::new();
    let mock_logger = MockLogger::new();
    let mock_metrics = MockMetrics::new();
    // ... 10 more mocks

    // Test is testing the mocks, not the code
}

// ✅ GOOD: Use real implementations where practical
#[test]
fn test_with_minimal_mocking() {
    let temp_dir = TempDir::new().unwrap();
    let db = Db::new(temp_dir.path()); // Real DB
    let service = Service::new(db);

    // Test against real behavior
}
```

### 4. Ignoring Error Messages

```rust
// ❌ BAD: Generic assertion
#[test]
fn test_error() {
    let result = do_something();
    assert!(result.is_err());
}

// ✅ GOOD: Specific error assertion
#[test]
fn test_specific_error() {
    let result = do_something();
    assert!(
        matches!(result, Err(Error::InvalidInput(msg)) if msg.contains("expected format")),
        "Expected InvalidInput error with format message, got {:?}",
        result
    );
}
```

### 5. Over-Engineering Fixtures

```rust
// ❌ BAD: Complex fixture hierarchy
#[fixture]
fn db() -> Db { ... }

#[fixture]
fn connection(db: Db) -> Connection { ... }

#[fixture]
fn transaction(connection: Connection) -> Transaction { ... }

#[fixture]
fn repository(transaction: Transaction) -> Repository { ... }

#[fixture]
fn service(repository: Repository) -> Service { ... }

// ✅ GOOD: Simple, explicit setup
#[test]
fn test_service() {
    let temp_dir = TempDir::new().unwrap();
    let service = Service::new(temp_dir.path());

    // Test code
}
```

### 6. Sleeping in Tests

```rust
// ❌ BAD: Real sleep slows tests
#[test]
fn test_with_sleep() {
    do_something();
    std::thread::sleep(Duration::from_secs(1));
    assert_something();
}

// ✅ GOOD: Use synchronization primitives
#[test]
fn test_with_sync() {
    let (tx, rx) = mpsc::channel();
    do_something(tx);
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_something();
}

// Or for async tests:
#[tokio::test]
async fn test_async_with_timeout() {
    tokio::time::timeout(
        Duration::from_secs(1),
        wait_for_condition()
    ).await.unwrap();
}
```

### 7. Global Test State

```rust
// ❌ BAD: Mutable global state
static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    Mutex::new(Config::default())
});

#[test]
fn test_config_a() {
    CONFIG.lock().unwrap().value = 1;
    // ...
}

#[test]
fn test_config_b() {
    // May see value = 1 from test_config_a!
    assert_eq!(CONFIG.lock().unwrap().value, 0);
}

// ✅ GOOD: Pass configuration explicitly
#[test]
fn test_config_isolated() {
    let config = Config { value: 1 };
    let result = process_with_config(&config);
    // ...
}
```

## 18. Resources & Reference

### Official Rust Documentation

- [The Rust Book - Chapter 11: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [The rustc Book - Tests](https://doc.rust-lang.org/rustc/tests/index.html)

### Testing Tools Documentation

- [nextest Documentation](https://nexte.st/) - Fast, parallel test runner
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Code coverage
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - LLVM-based coverage
- [proptest Book](https://proptest-rs.github.io/proptest/intro.html) - Property-based testing
- [rstest Documentation](https://docs.rs/rstest/) - Parameterized tests and fixtures
- [criterion.rs](https://bheisler.github.io/criterion.rs/book/) - Benchmarking
- [mockall Documentation](https://docs.rs/mockall/) - Mocking framework
- [tokio Testing](https://tokio.rs/tokio/topics/testing) - Async testing patterns

### Lithos-Specific Documentation

- [System-Level Test Design](./test-design-system.md) - Overall testing strategy
- [AGENTS.md](../AGENTS.md) - AI agent testing guidelines
