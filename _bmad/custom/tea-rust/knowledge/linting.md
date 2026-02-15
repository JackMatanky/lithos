# TEA Knowledge: Linter Warning Management

## CONTEXT

- **Applies to**: All Rust code, with specific considerations for test modules
- **Purpose**: Maintain high code quality while allowing justified exceptions
- **Tools**: `cargo clippy`, `rustfmt`

## CLIPPY IN TESTS

### The Standard
All code, including tests, should be lint-clean (`cargo clippy -- -D warnings`). Warnings in tests often point to fragile assertions or inefficient setup.

### Suppressions: #[expect] vs #[allow]
Lithos follows a "Fail-Fast but Documented" suppression policy.

| Attribute | Usage | When to Use |
| :--- | :--- | :--- |
| `#[expect(lint_name, reason = "...")]` | **Preferred** | When a lint is triggered but the pattern is intentional and justified. Fails if the lint is *not* triggered (preventing stale suppressions). |
| `#[allow(lint_name)]` | **Discouraged** | Only use if `#[expect]` is not supported by the current toolchain or for temporary legacy code. |

### Justified Exceptions in Tests
Certain lints may be suppressed in tests if a clear `reason` is provided:

1.  **`clippy::unwrap_used` / `clippy::expect_used`**:
    - **Acceptable**: In the **Arrange** phase for test fixtures where failure means the test cannot proceed.
    - **Reject**: In the **Act** or **Assert** phases (use `assert!(result.is_ok())`).
2.  **`clippy::too_many_arguments`**:
    - **Acceptable**: For complex test fixtures where builder patterns are overkill.
3.  **`clippy::indexing_slicing`**:
    - **Acceptable**: When testing fixed-size data where the index is a literal.

## VALIDATION CHECKLIST

### Lint Hygiene
- [ ] `cargo clippy` runs without warnings
- [ ] `cargo fmt` has been applied
- [ ] No crate-wide or module-wide `#![allow(...)]` (prefer local suppressions)

### Suppression Quality
- [ ] Every suppression uses `#[expect]` (if stable) or `#[allow]` with a comment
- [ ] Every suppression includes a `reason = "..."` explaining why it's necessary
- [ ] Suppression scope is as tight as possible (on the specific statement or function)

## ANTI-PATTERNS (FLAG THESE)
- ❌ **Silent Failures** → Using `#[allow]` without a comment/reason
- ❌ **Global Suppressions** → `#![allow(clippy::all)]` at the top of a file
- ❌ **Stale Suppressions** → Suppressing a lint that is no longer triggered by the code
- ❌ **Assertion Unwraps** → Using `.unwrap()` in the Assert phase instead of explicit assertions

## CORRECT EXAMPLES

### Justified Suppression
```rust
#[test]
fn test_complex_flow() {
    // Arrange: unwrap is acceptable here as it's a prerequisite
    #[expect(clippy::unwrap_used, reason = "Test fixture requires valid initial state")]
    let context = setup_context().unwrap();

    // Act
    let result = process(&context);

    // Assert: Use explicit assertion for the outcome
    assert!(result.is_ok(), "Process should succeed, got {:?}", result.err());
}
```

### Formatting
```bash
# Ensure formatting is checked in CI
cargo fmt -- --check
```

## RELATED MODULES
- See `anti-patterns.md` for common code issues clippy catches
- See `test-unit.md` for Arrange-Act-Assert phase guidance
- See `ci.md` for automation gates
