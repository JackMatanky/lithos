# TEA Knowledge: Rust Unit Testing

## CONTEXT

- **Applies to**: Tests in `#[cfg(test)] mod tests` blocks
- **Location**: Same file as implementation (`lithos-core/src/**/*.rs`)
- **Does NOT apply to**: `tests/` directory, integration tests, E2E tests
- **Purpose**: Testing business logic, state transitions, validation rules with zero I/O

## DECISION TREE: Where Should This Test Go?

```
Is the test checking...
├── Private functions or internal state?
│   └── YES → Unit test in #[cfg(test)] mod tests (same file)
│
├── Public API with no side effects (pure functions)?
│   └── YES → Unit test in #[cfg(test)] mod tests (same file)
│
├── I/O operations (filesystem, network)?
│   └── YES → Integration test in tests/ directory
│
├── External dependencies (database, services)?
│   └── YES → Integration test in tests/ directory
│
├── CLI command behavior?
│   └── YES → E2E test in lithos-cli/
│
└── Port/adapter implementations?
    └── YES → Integration test with mock ports
```

## VALIDATION CHECKLIST

### Location & Structure

- [ ] Test is in `#[cfg(test)] mod tests` block in the same file as implementation
- [ ] Test module is at the bottom of the source file (after implementation)
- [ ] Uses `use super::*;` to access private items

### Naming

- [ ] Test name follows `action_expected_condition` pattern
- [ ] No `test_` prefix (redundant with `#[test]` attribute)
- [ ] No generic names like `test_foo`, `test_1`, `test_basic`
- [ ] Uses submodule per function when testing complex units

### Assertions

- [ ] All `assert!` calls include explicit error messages with context
- [ ] Uses `matches!` for enum variant checking (not equality)
- [ ] Uses `assert!(result.is_ok(), "...")` NOT `result.unwrap()`
- [ ] Error messages include the actual error: `result.err()`

### Fixtures

- [ ] Fixtures are inline in the test module (not external crates)
- [ ] Helper functions (not macros) for repeated setup
- [ ] Uses `tempfile::TempDir` for any filesystem operations
- [ ] Deterministic fixtures with fixed seeds for randomness

### Test Isolation

- [ ] No shared mutable state between tests
- [ ] No static variables or global test state
- [ ] Each test has independent fixtures

### Phases (Arrange-Act-Assert)

- [ ] **Arrange**: `unwrap()` permitted for setup (test prerequisites)
- [ ] **Act**: NO `unwrap()` - capture the result
- [ ] **Assert**: NO `unwrap()` - use explicit assertions

## ANTI-PATTERNS (FLAG THESE)

### Critical Issues

- ❌ **Test outside `#[cfg(test)]`** → Must be in test module
- ❌ **Test in `tests/` testing private functions** → Move to inline unit test
- ❌ **`result.unwrap()` in Act/Assert phases** → Use `assert!(result.is_ok(), "...")`
- ❌ **Shared mutable state** → Use independent fixtures per test
- ❌ **Non-deterministic tests** → Use fixed seeds, avoid system time

### Naming Issues

- ❌ `#[test] fn test_foo()` → Use `returns_error_when_invalid_input()`
- ❌ `#[test] fn test_basic()` → Use specific behavior description
- ❌ Multiple behaviors in one test name (using "and") → Split into separate tests

### Assertion Issues

- ❌ `assert!(result.is_ok())` without error message
- ❌ `assert_eq!(result, Ok(expected))` on complex enums
- ❌ `result.expect("...")` in assertions
- ❌ Hidden assertions in helper functions

### Fixture Issues

- ❌ External test utility crates
- ❌ Shared test fixtures across modules
- ❌ Complex builder patterns when simple helpers suffice
- ❌ Manual cleanup instead of RAII (`tempfile::TempDir`)

## CORRECT EXAMPLES

### Basic Unit Test

```rust
// src/note/path.rs
pub fn validate(path: &str) -> Result<(), NoteError> {
    if path.is_empty() {
        return Err(NoteError::EmptyPath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_error_when_path_is_empty() {
        // Arrange
        let path = "";

        // Act
        let result = validate(path);

        // Assert
        assert!(
            matches!(result, Err(NoteError::EmptyPath)),
            "Expected EmptyPath error, got: {:?}",
            result
        );
    }

    #[test]
    fn returns_ok_when_path_is_valid() {
        let result = validate("notes/test.md");
        assert!(
            result.is_ok(),
            "Expected Ok, but got error: {:?}",
            result.err()
        );
    }
}
```

### Module Per Function Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod validate {
        use super::*;

        #[test]
        fn rejects_empty_paths() { }

        #[test]
        fn rejects_invalid_characters() { }

        #[test]
        fn accepts_valid_note_paths() { }
    }

    mod parse {
        use super::*;

        #[test]
        fn extracts_filename_correctly() { }

        #[test]
        fn handles_nested_directories() { }
    }
}
```

### With Fixtures

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a test note with default values
    fn note_fixture(path: &str) -> Note {
        Note {
            id: Uuid::now_v7(),
            path: NotePath::new(path.to_owned()).expect("Valid test path"),
            content: "Test content".to_owned(),
            tags: vec![],
        }
    }

    #[test]
    fn validates_note_with_valid_path() {
        let note = note_fixture("test.md");
        assert!(note.validate().is_ok());
    }
}
```

## QUICK REFERENCE

| Check          | Command/Pattern                             |
| -------------- | ------------------------------------------- |
| Run unit tests | `mise run test:unit` or `cargo nextest run` |
| Test location  | `src/**/*.rs` in `#[cfg(test)] mod tests`   |
| Test runner    | nextest (not cargo test)                    |
| Fixtures       | Inline helper functions                     |
| Filesystem     | `tempfile::TempDir`                         |
| Naming         | `action_expected_condition`                 |
| Assertions     | Explicit messages with context              |
| Enum checks    | `matches!` macro                            |

## RELATED MODULES

- See `testing-naming.md` for detailed naming conventions
- See `testing-assertions.md` for assertion patterns
- See `testing-fixtures.md` for fixture strategies
- See `testing-anti-patterns.md` for comprehensive anti-patterns
