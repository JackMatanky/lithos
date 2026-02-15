# Rust Testing Anti-Patterns (Unified Source of Truth)

## CONTEXT

- **Purpose**: Comprehensive reference for detecting, scoring, and fixing anti-patterns in Rust test code.
- **Audience**: TEA agent during test review / Developers.
- **Scoring Logic**: Critical (-20), Warning (-10), Info (-5).

## CRITICAL ANTI-PATTERNS (Must Fix)

### ❌ Test Location Issues

| Anti-Pattern                    | Why It's Bad                             | Fix                           |
| ------------------------------- | ---------------------------------------- | ----------------------------- |
| Unit test in `tests/` directory | Tests private API from external location | Move to inline `#[cfg(test)]` |
| Integration test in `src/`      | Cannot test public API isolation         | Move to `tests/` directory    |
| E2E test in `lithos-core/`      | Wrong layer                              | Move to `lithos-cli/`         |

### ❌ Assertion Issues

| Anti-Pattern                                | Why It's Bad                       | Fix                                            |
| ------------------------------------------- | ---------------------------------- | ---------------------------------------------- |
| `result.unwrap()` in assertions             | Hides error information on failure | `assert!(result.is_ok(), "...", result.err())` |
| `result.expect("...")` in assertions        | Same issue                         | Use explicit assertions                        |
| `assert!(result.is_ok())` without message   | No context on failure              | Add descriptive message                        |
| `assert_eq!(result, Ok(expected))` on enums | Brittle, checks full equality      | `assert!(matches!(result, Ok(_)))`             |

### ❌ State Issues

| Anti-Pattern             | Why It's Bad                      | Fix                           |
| ------------------------ | --------------------------------- | ----------------------------- |
| Shared mutable state     | Tests interfere with each other   | Independent fixtures per test |
| Static variables         | Race conditions, non-determinism  | Use fixtures                  |
| Tests depending on order | Brittle, parallel execution fails | Make tests independent        |
| Non-deterministic tests  | Flaky failures                    | Use fixed seeds, mock time    |

### ❌ Naming Issues

| Anti-Pattern                  | Why It's Bad     | Fix                                       |
| ----------------------------- | ---------------- | ----------------------------------------- |
| `#[test] fn test_foo()`       | No description   | `#[test] fn returns_error_when_invalid()` |
| `#[test] fn test()`           | No information   | Descriptive behavior name                 |
| Multiple behaviors with "and" | Testing too much | Split into separate tests                 |
| Generic numbered tests        | No meaning       | Descriptive names                         |

## WARNING ANTI-PATTERNS (Should Fix)

### ⚠️ Fixture Issues

| Anti-Pattern                 | Why It's Bad                 | Fix                     |
| ---------------------------- | ---------------------------- | ----------------------- |
| External test utility crates | Coupling, maintenance burden | Inline fixtures         |
| Complex builder patterns     | Hard to understand           | Simple helper functions |
| Manual cleanup               | Risk of not cleaning up      | Use RAII (TempDir)      |
| Overly generic fixtures      | Unclear purpose              | Specific fixtures       |

### ⚠️ Filesystem Issues

| Anti-Pattern                | Why It's Bad                        | Fix                    |
| --------------------------- | ----------------------------------- | ---------------------- |
| Hardcoded paths             | Non-portable, test isolation issues | `tempfile::TempDir`    |
| `fs::remove_file` in tests  | May fail, not reliable              | RAII cleanup           |
| Shared temp directories     | Test isolation issues               | Fresh TempDir per test |
| Environment-dependent tests | Fail on different machines          | Mock environment       |

### ⚠️ Async Issues

| Anti-Pattern               | Why It's Bad      | Fix                               |
| -------------------------- | ----------------- | --------------------------------- |
| `sleep()` in tests         | Slow, flaky       | Use deterministic synchronization |
| Holding mutex across await | Deadlock risk     | Release before await              |
| Real time in tests         | Non-deterministic | Mock time (`tokio::time::pause`)  |

### ⚠️ Mock Issues

| Anti-Pattern               | Why It's Bad           | Fix                  |
| -------------------------- | ---------------------- | -------------------- |
| Mocks without expectations | Unclear what is tested | Set `expect_*`       |
| Over-mocking               | Testing implementation | Only mock boundaries |
| Not verifying mocks        | May not be called      | Check `times()`      |

## INFO ANTI-PATTERNS (Consider Fixing)

### ℹ️ Assertion Style

| Anti-Pattern                 | Why It's Suboptimal | Consider                     |
| ---------------------------- | ------------------- | ---------------------------- |
| `assert!(x == y)`            | Less informative    | `assert_eq!(x, y, "...")`    |
| `assert!(x != y)`            | Less informative    | `assert_ne!(x, y, "...")`    |
| `assert!(option.is_some())`  | Can't unwrap after  | `if let Some(v) = option`    |
| Hidden assertions in helpers | Test logic unclear  | Keep assertions in test body |

### ℹ️ Organization

| Anti-Pattern                 | Why It's Suboptimal | Consider                  |
| ---------------------------- | ------------------- | ------------------------- |
| Tests without submodules     | Hard to navigate    | Group by function/feature |
| Mixed concerns in one test   | Hard to diagnose    | One behavior per test     |
| Test code > 50% of prod code | Maintenance burden  | Focus on critical paths   |

### ℹ️ Documentation

| Anti-Pattern                   | Why It's Suboptimal  | Consider              |
| ------------------------------ | -------------------- | --------------------- |
| No doc tests                   | Missing API examples | Add `/// # Examples`  |
| Doc tests without hidden setup | Noisy documentation  | Use `#` to hide setup |
| Tests without comments         | Intent unclear       | Explain complex setup |

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

| Test Type   | Wrong Location | Correct Location                |
| ----------- | -------------- | ------------------------------- |
| Unit        | `tests/*.rs`   | `src/**/*.rs` in `#[cfg(test)]` |
| Integration | `src/*.rs`     | `tests/*.rs`                    |
| E2E         | `lithos-core/` | `lithos-cli/`                   |

## SCORING IMPACT

| Severity | Impact on Test Quality Score |
| -------- | ---------------------------- |
| Critical | -20 points per occurrence    |
| Warning  | -10 points per occurrence    |
| Info     | -5 points per occurrence     |

Maximum deduction: -50 points (to avoid negative scores)

## QUICK REFERENCE

| If you see...                  | Flag as... | Fix with...              |
| ------------------------------ | ---------- | ------------------------ |
| `result.unwrap()` in test body | Critical   | Explicit assertion       |
| `assert!(result.is_ok())`      | Warning    | Add error message        |
| Test in wrong directory        | Critical   | Move to correct location |
| `#[test] fn test_foo()`        | Critical   | Descriptive name         |
| Shared state                   | Critical   | Independent fixtures     |
| Hardcoded paths                | Warning    | `tempfile::TempDir`      |
| Manual cleanup                 | Warning    | RAII                     |
| `sleep()` in tests             | Critical   | Deterministic sync       |

---

## DETAILED ANALYSIS & PROSE DEEP-DIVES

### High-Level Anti-Patterns

#### 1. Coverage-Driven Testing

**Problem**: Focusing on achieving high coverage percentages rather than meaningful tests.

**Bad Example**:

```rust
#[test]
fn test_getter_coverage() {
    let note = Note::new(/*...*/).unwrap();
    assert_eq!(note.title(), "title"); // Trivial test for coverage
    assert_eq!(note.content(), "content"); // No real value
    assert!(note.id().is_some()); // Testing internal state
}
```

**Good Approach**:

```rust
#[test]
fn test_note_invariant_validation() {
    // Test actual business logic and invariants
    let result = Note::new(
        NoteId::new_random(),
        NoteTitle::new("Valid Title").unwrap(),
        NoteContent::new("Valid content").unwrap(),
        Timestamp::now(),
    );

    assert!(result.is_ok());
    let note = result.unwrap();

    // Test meaningful behavior
    assert!(note.validate_schema(&test_schema()).is_ok());
    assert!(note.title().len() <= MAX_TITLE_LENGTH);
}
```

#### 2. Implementation-Specific Testing

**Problem**: Testing internal implementation details rather than behavior contracts.

**Bad Example**:

```rust
#[test]
fn test_internal_storage_format() {
    let storage = RedbStorage::new(/*...*/);
    let note = create_test_note();

    // Testing internal storage details
    let raw_data = storage.get_raw_data(&note.id()).unwrap();
    assert!(raw_data.contains("NoteV1")); // Brittle implementation test
}
```

**Good Approach**:

```rust
#[test]
fn test_storage_roundtrip() {
    let storage = RedbStorage::new(/*...*/);
    let original = create_test_note();

    // Test behavior contract: data can be stored and retrieved
    let id = storage.store_note(&original).unwrap();
    let retrieved = storage.get_note(id).unwrap();

    assert_eq!(retrieved.unwrap(), original); // Behavior, not implementation
}
```

### Rust-Specific Anti-Patterns

#### 1. Panic-Prone Tests

**Problem**: Using `unwrap()` or `expect()` in tests when testing error paths.

**Bad Example**:

```rust
#[test]
fn test_error_handling() {
    let result = Note::new(
        NoteId::new_random(),
        NoteTitle::new("").unwrap(), // Panic instead of testing error
        NoteContent::new("content").unwrap(),
        Timestamp::now(),
    );
    // Test never reaches error case
}
```

**Good Approach**:

```rust
#[test]
fn test_error_handling() {
    let result = Note::new(
        NoteId::new_random(),
        NoteTitle::new(""), // This should fail
        NoteContent::new("content").unwrap(),
        Timestamp::now(),
    );

    // Test the error case properly
    assert!(result.is_err());
    match result.unwrap_err() {
        ValidationError::EmptyTitle => {
            // Expected error variant
        }
        other => panic!("Expected EmptyTitle error, got {:?}", other),
    }
}
```

#### 2. Ownership Testing Anti-Patterns

**Problem**: Not properly testing ownership transfer and borrowing scenarios.

**Bad Example**:

```rust
#[test]
fn test_ownership_scenarios() {
    let note = create_test_note();
    let processor = NoteProcessor::new();

    // Doesn't test ownership transfer
    let result = processor.process(note);
    assert!(result.is_ok());

    // note is still accessible here - ownership wasn't actually transferred
    assert!(note.title().len() > 0); // This should be a compile error if ownership was transferred
}
```

**Good Approach**:

```rust
#[test]
fn test_ownership_transfer() {
    let note = create_test_note();
    let processor = NoteProcessor::new();

    // Test actual ownership transfer
    let result = processor.consume_note(note);
    assert!(result.is_ok());

    // note is no longer accessible - this proves ownership was transferred
    // The following line would cause a compile error if uncommented:
    // assert!(note.title().len() > 0); // compile_error: use of moved value
}

#[test]
fn test_borrowing_scenarios() {
    let note = create_test_note();
    let processor = NoteProcessor::new();

    // Test borrowing without ownership transfer
    let result = processor.process_borrowed(&note);
    assert!(result.is_ok());

    // note is still accessible - ownership was not transferred
    assert!(note.title().len() > 0); // This works correctly
}
```

#### 3. Async Testing Anti-Patterns

**Problem**: Not properly testing async code, especially with blocking operations.

**Bad Example**:

```rust
#[test]
fn test_async_storage() {
    let storage = AsyncRedbStorage::new(/*...*/);

    // Using .unwrap() on async code without proper testing
    let result = futures::executor::block_on(
        storage.store_note(&create_test_note())
    ).unwrap();

    // Doesn't test concurrent scenarios, error handling, etc.
}
```

**Good Approach**:

```rust
#[tokio::test]
async fn test_async_storage_concurrent() {
    let storage = Arc::new(AsyncRedbStorage::new(/*...*/).await);
    let note = create_test_note();

    // Test concurrent access scenarios
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let storage = Arc::clone(&storage);
            let note = note.clone();
            tokio::spawn(async move {
                storage.store_note(&note).await
            })
        })
        .collect();

    // All concurrent operations should succeed
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[test]
fn test_async_error_handling() {
    let storage = create_mock_storage_with_failure();

    // Test async error scenarios
    let result = futures::executor::block_on(
        storage.store_note(&create_invalid_note())
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        StorageError::Validation(_) => {
            // Expected error case
        }
        other => panic!("Expected validation error, got {:?}", other),
    }
}
```

### Lithos Architecture Anti-Patterns

#### 1. Context Boundary Violations

**Problem**: Tests that create inappropriate dependencies between bounded contexts.

**Bad Example**:

```rust
#[test]
fn test_cross_context_violation() {
    let note_storage = NoteStorage::new();
    let schema_storage = SchemaStorage::new();

    // Direct coupling between contexts - anti-pattern
    let note = note_storage.get_note(id).unwrap();
    let schema = schema_storage.get_schema(note.schema_id()).unwrap();

    // Testing implementation details across context boundaries
    assert!(schema.validation_rules().len() > 0);
}
```

**Good Approach**:

```rust
#[test]
fn test_context_boundary_respect() {
    // Test through proper domain services
    let note_service = NoteService::new(/*...*/);
    let schema_service = SchemaService::new(/*...*/);

    // Each context tested independently
    let schema_result = schema_service.get_schema(schema_id);
    assert!(schema_result.is_ok());

    // Cross-context interaction through proper interfaces
    let note_result = note_service.create_note_with_schema(&note_data, schema_id);
    assert!(note_result.is_ok());

    // Test behavior, not implementation
    let created_note = note_result.unwrap();
    assert!(created_note.is_valid_against_schema(&schema_service));
}
```

#### 2. Port Testing Anti-Patterns

**Problem**: Testing port implementations with concrete dependencies instead of mocks.

**Bad Example**:

```rust
#[test]
fn test_port_with_real_database() {
    // Uses real database - integration test, not unit test
    let storage = RedbStorage::new(Path::new("/tmp/test.db")).unwrap();
    let port = NoteStoragePort::new(storage);

    let note = create_test_note();
    let result = port.store_note(&note);
    assert!(result.is_ok());
}
```

**Good Approach**:

```rust
#[test]
fn test_port_with_mock() {
    let mut mock = MockNoteStoragePort::new();

    // Configure mock behavior
    mock.expect_store_note()
        .returning(|note| Ok(note.id().clone()));

    mock.expect_get_note()
        .returning(|id| Some(create_test_note_with_id(id)));

    // Test port behavior with isolated dependencies
    let service = NoteService::new(Box::new(mock));
    let note = create_test_note();
    let result = service.create_note(&note);

    assert!(result.is_ok());
}
```

### Test Organization Anti-Patterns

#### 1. Monolithic Test Modules

**Problem**: All tests in one large, unorganized module.

**Bad Example**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_creation_1() { /*...*/ }
    #[test] fn test_creation_2() { /*...*/ }
    #[test] fn test_validation_1() { /*...*/ }
    #[test] fn test_validation_2() { /*...*/ }
    #[test] fn test_storage_1() { /*...*/ }
    #[test] fn test_storage_2() { /*...*/ }
    // 50 more tests...
}
```

**Good Approach**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    mod creation_tests {
        use super::*;

        #[test]
        fn test_valid_creation() { /*...*/ }

        #[test]
        fn test_invalid_title_creation() { /*...*/ }

        #[test]
        fn test_boundary_values() { /*...*/ }
    }

    mod validation_tests {
        use super::*;

        #[test]
        fn test_schema_validation() { /*...*/ }

        #[test]
        fn test_business_rule_validation() { /*...*/ }
    }

    mod storage_tests {
        use super::*;

        #[test]
        fn test_persistence() { /*...*/ }

        #[test]
        fn test_retrieval() { /*...*/ }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow() { /*...*/ }
    }
}
```

#### 2. Test Data Management Anti-Patterns

**Problem**: Hardcoded test data or inefficient data generation.

**Bad Example**:

```rust
#[test]
fn test_with_hardcoded_data() {
    let note = Note::new(
        NoteId::from_u64(12345), // Magic number
        NoteTitle::new("A specific title").unwrap(), // Hardcoded
        NoteContent::new("Specific content").unwrap(), // Brittle
        Timestamp::from_i64(1234567890), // Magic timestamp
    );
    // Test logic...
}
```

**Good Approach**:

```rust
// Use builder pattern for flexible test data
pub struct NoteTestBuilder {
    title: Option<String>,
    content: Option<String>,
    timestamp: Option<Timestamp>,
    schema_id: Option<SchemaId>,
}

impl NoteTestBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            content: None,
            timestamp: Some(Timestamp::now()),
            schema_id: Some(SchemaId::new_random()),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn build(self) -> Result<Note, ValidationError> {
        let title = self.title.unwrap_or_else(|| "Test Title".to_string());
        let content = self.content.unwrap_or_else(|| "Test Content".to_string());

        Note::new(
            NoteId::new_random(),
            NoteTitle::new(&title)?,
            NoteContent::new(&content)?,
            self.timestamp.unwrap_or_else(Timestamp::now),
        )
    }
}

#[test]
fn test_with_builder() {
    let note = NoteTestBuilder::new()
        .with_title("Custom Title")
        .with_content("Custom content")
        .build()
        .unwrap();

    // Test logic...
}
```

### Performance Testing Anti-Patterns

#### 1. Inaccurate Benchmarking

**Problem**: Not accounting for warmup, JIT compilation, or measurement error.

**Bad Example**:

```rust
#[test]
fn test_performance_naive() {
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let note = create_test_note();
        process_note(note); // First iteration includes compilation overhead
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100); // Not meaningful measurement
}
```

**Good Approach**:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_note_processing(c: &mut Criterion) {
    let notes: Vec<_> = (0..1000)
        .map(|i| create_test_note_with_id(i))
        .collect();

    c.bench_function("note_processing", |b| {
        b.iter(|| {
            for note in &notes {
                process_note(black_box(note.clone()));
            }
        })
    });
}

criterion_group!(benches, benchmark_note_processing);
criterion_main!(benches);
```

### CI/CD Integration Anti-Patterns

#### 1. Flaky Tests in CI

**Problem**: Tests that pass locally but fail intermittently in CI.

**Bad Example**:

```rust
#[test]
fn test_with_timing_dependency() {
    let note = create_test_note();
    let start = std::time::Instant::now();

    process_note_async(note);

    // Race condition: assumes processing completes within 100ms
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(start.elapsed().as_millis() >= 100); // Flaky in CI
}
```

**Good Approach**:

```rust
#[tokio::test]
async fn test_async_properly() {
    let note = create_test_note();

    // Wait for actual completion, not arbitrary timeout
    let result = process_note_async(note).await;
    assert!(result.is_ok());
}
```

## Detection and Prevention

### Automated Detection

- Use clippy lints to catch common anti-patterns
- Implement custom test quality metrics
- Monitor for brittle tests that break frequently
- Track test execution times for performance regressions

### Code Review Checklist

- [ ] Tests focus on behavior, not implementation
- [ ] Error cases are properly tested
- [ ] No `unwrap()` calls in error path tests
- [ ] Test data is generated, not hardcoded
- [ ] Tests are organized into logical modules
- [ ] Context boundaries are respected
- [ ] Ports are tested with mocks, not real implementations
- [ ] Async tests use proper async/await patterns
- [ ] Performance tests use proper benchmarking tools

By avoiding these anti-patterns, the Lithos project can maintain high-quality, effective test suites that provide real confidence in code correctness and reliability.
