# TEA Knowledge: Doc-Tests (Executable Examples)

## CONTEXT

- **Applies to**: Public API documentation (`/// # Examples`)
- **Purpose**: Living documentation, compiler-verified examples
- **Target**: Mandatory for all public domain models

## BEST PRACTICES

### Living Documentation
Doc-tests show how functions are meant to be used. They are often more helpful than reading the function body.

### Duplication is OK
It is acceptable to duplicate logic between doc-tests and unit tests if it improves documentation clarity.

### Setup Management
- **Hide Boilerplate**: Use `#` at the start of a line to hide setup code (like imports) from the generated documentation while keeping it in the executable test.
- **`no_run`**: Use for side-effect heavy code (I/O, network) that should compile but not execute.
- **`compile_fail`**: Verifies that incorrect API usage correctly fails to compile.
- **`should_panic`**: For demonstrating code that intentionally panics.

## VALIDATION CHECKLIST

- [ ] Public API functions have `/// # Examples`
- [ ] Examples are compilable and executable (unless `no_run`)
- [ ] Boilerplate setup is hidden using `#`
- [ ] Correct attributes used (`no_run`, `compile_fail`, `should_panic`)

## CORRECT EXAMPLES

### Basic Example with Hidden Setup
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

### Side-Effect Heavy (`no_run`)
```rust
/// Deletes a note from the vault.
///
/// # Examples
///
/// ```no_run
/// # use lithos_core::note::Repository;
/// # use std::path::Path;
/// let repo = Repository::open(Path::new("/path/to/vault")).unwrap();
/// repo.delete("notes/old.md").unwrap();
/// ```
pub fn delete(&self, path: &str) -> Result<(), NoteError> {
    // implementation
}
```

### Invalid Usage (`compile_fail`)
```rust
/// The following will **not** compile because the tag doesn't start with `#`:
///
/// ```compile_fail
/// # use lithos_core::note::Tag;
/// let tag = Tag::new("invalid").unwrap();
/// ```
pub fn new(tag: &str) -> Result<Self, TagError> {
    // implementation
}
```

## RELATED MODULES
- See `test-unit.md` for standard unit testing
- See `assertions.md` for assertion patterns
