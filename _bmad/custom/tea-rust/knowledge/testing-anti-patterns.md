# TEA Knowledge: Testing Anti-Patterns

## CONTEXT
- **Purpose**: Patterns to detect and reject in test code
- **Audience**: TEA agent during test review
- **Severity**: Critical (must fix), Warning (should fix), Info (consider)

## CRITICAL ANTI-PATTERNS (Must Fix)

### ❌ Test Location Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Unit test in `tests/` directory | Tests private API from external location | Move to inline `#[cfg(test)]` |
| Integration test in `src/` | Cannot test public API isolation | Move to `tests/` directory |
| E2E test in `lithos-core/` | Wrong layer | Move to `lithos-cli/` |

### ❌ Assertion Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| `result.unwrap()` in assertions | Hides error information on failure | `assert!(result.is_ok(), "...", result.err())` |
| `result.expect("...")` in assertions | Same issue | Use explicit assertions |
| `assert!(result.is_ok())` without message | No context on failure | Add descriptive message |
| `assert_eq!(result, Ok(expected))` on enums | Brittle, checks full equality | `assert!(matches!(result, Ok(_)))` |

### ❌ State Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Shared mutable state | Tests interfere with each other | Independent fixtures per test |
| Static variables | Race conditions, non-determinism | Use fixtures |
| Tests depending on order | Brittle, parallel execution fails | Make tests independent |
| Non-deterministic tests | Flaky failures | Use fixed seeds, mock time |

### ❌ Naming Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| `#[test] fn test_foo()` | No description | `#[test] fn returns_error_when_invalid()` |
| `#[test] fn test()` | No information | Descriptive behavior name |
| Multiple behaviors with "and" | Testing too much | Split into separate tests |
| Generic numbered tests | No meaning | Descriptive names |

## WARNING ANTI-PATTERNS (Should Fix)

### ⚠️ Fixture Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| External test utility crates | Coupling, maintenance burden | Inline fixtures |
| Complex builder patterns | Hard to understand | Simple helper functions |
| Manual cleanup | Risk of not cleaning up | Use RAII (TempDir) |
| Overly generic fixtures | Unclear purpose | Specific fixtures |

### ⚠️ Filesystem Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Hardcoded paths | Non-portable, test isolation issues | `tempfile::TempDir` |
| `fs::remove_file` in tests | May fail, not reliable | RAII cleanup |
| Shared temp directories | Test isolation issues | Fresh TempDir per test |
| Environment-dependent tests | Fail on different machines | Mock environment |

### ⚠️ Async Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| `sleep()` in tests | Slow, flaky | Use deterministic synchronization |
| Holding mutex across await | Deadlock risk | Release before await |
| Real time in tests | Non-deterministic | Mock time (`tokio::time::pause`) |

### ⚠️ Mock Issues

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Mocks without expectations | Unclear what is tested | Set `expect_*` |
| Over-mocking | Testing implementation | Only mock boundaries |
| Not verifying mocks | May not be called | Check `times()` |

## INFO ANTI-PATTERNS (Consider Fixing)

### ℹ️ Assertion Style

| Anti-Pattern | Why It's Suboptimal | Consider |
|--------------|---------------------|----------|
| `assert!(x == y)` | Less informative | `assert_eq!(x, y, "...")` |
| `assert!(x != y)` | Less informative | `assert_ne!(x, y, "...")` |
| `assert!(option.is_some())` | Can't unwrap after | `if let Some(v) = option` |
| Hidden assertions in helpers | Test logic unclear | Keep assertions in test body |

### ℹ️ Organization

| Anti-Pattern | Why It's Suboptimal | Consider |
|--------------|---------------------|----------|
| Tests without submodules | Hard to navigate | Group by function/feature |
| Mixed concerns in one test | Hard to diagnose | One behavior per test |
| Test code > 50% of prod code | Maintenance burden | Focus on critical paths |

### ℹ️ Documentation

| Anti-Pattern | Why It's Suboptimal | Consider |
|--------------|---------------------|----------|
| No doc tests | Missing API examples | Add `/// # Examples` |
| Doc tests without hidden setup | Noisy documentation | Use `#` to hide setup |
| Tests without comments | Intent unclear | Explain complex setup |

## COMPLETE EXAMPLE: Before and After

### ❌ Before (Anti-Patterns)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    static DB: Lazy<Database> = Lazy::new(|| Database::new());

    #[test]
    fn test() {
        let result = process("test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_file() {
        fs::write("/tmp/test.txt", "data").unwrap();
        let content = fs::read_to_string("/tmp/test.txt").unwrap();
        assert_eq!(content, "data");
        fs::remove_file("/tmp/test.txt").unwrap();
    }

    #[test]
    fn test_db() {
        DB.insert("key", "value").unwrap();
        let val = DB.get("key").unwrap();
        assert_eq!(val, "value");
    }
}
```

### ✅ After (Fixed)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn returns_ok_for_valid_input() {
        let result = process("test");
        assert!(
            result.is_ok(),
            "Processing should succeed, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn writes_and_reads_file() -> std::io::Result<()> {
        let temp = TempDir::new()?;
        let file_path = temp.path().join("test.txt");

        fs::write(&file_path, "data")?;
        let content = fs::read_to_string(&file_path)?;

        assert_eq!(
            content, "data",
            "File content should match what was written"
        );
        Ok(())
    }

    #[test]
    fn persists_data_to_database() {
        let db = Database::new_in_memory(); // Fresh DB per test
        db.insert("key", "value").unwrap();

        let val = db.get("key").unwrap();
        assert_eq!(
            val, "value",
            "Database should return stored value"
        );
    }
}
```

## DETECTION PATTERNS FOR TEA

When reviewing tests, check for these patterns:

### Code Patterns to Flag

```rust
// CRITICAL
result.unwrap()              // In assertions
result.expect("...")         // In assertions
static mut                   // Mutable static
std::thread::sleep           // In tests

// WARNING
cargo test                   // Should use nextest
#[test] fn test_*()          // Generic name
assert!(result.is_ok())      // Without message
fs::remove_file              // Manual cleanup

// INFO
assert!(x == y)              // Could use assert_eq
assert!(x != y)              // Could use assert_ne
```

### File Location Checks

| Test Type | Wrong Location | Correct Location |
|-----------|----------------|------------------|
| Unit | `tests/*.rs` | `src/**/*.rs` in `#[cfg(test)]` |
| Integration | `src/*.rs` | `tests/*.rs` |
| E2E | `lithos-core/` | `lithos-cli/` |

## SCORING IMPACT

| Severity | Impact on Test Quality Score |
|----------|------------------------------|
| Critical | -20 points per occurrence |
| Warning | -10 points per occurrence |
| Info | -5 points per occurrence |

Maximum deduction: -50 points (to avoid negative scores)

## QUICK REFERENCE

| If you see... | Flag as... | Fix with... |
|---------------|------------|-------------|
| `result.unwrap()` in test body | Critical | Explicit assertion |
| `assert!(result.is_ok())` | Warning | Add error message |
| Test in wrong directory | Critical | Move to correct location |
| `#[test] fn test_foo()` | Critical | Descriptive name |
| Shared state | Critical | Independent fixtures |
| Hardcoded paths | Warning | `tempfile::TempDir` |
| Manual cleanup | Warning | RAII |
| `sleep()` in tests | Critical | Deterministic sync |

## RELATED MODULES
- See `testing-unit.md` for correct unit testing patterns
- See `testing-integration.md` for integration testing
- See `testing-assertions.md` for correct assertion patterns
- See `testing-naming.md` for naming conventions
