# Rust Testing Anti-Patterns

## Overview

This document outlines common testing anti-patterns in Rust development, specifically tailored for the Lithos project architecture. Recognizing and avoiding these anti-patterns is crucial for maintaining high-quality, effective test suites.

## High-Level Anti-Patterns

### 1. Coverage-Driven Testing

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

### 2. Implementation-Specific Testing

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

## Rust-Specific Anti-Patterns

### 1. Panic-Prone Tests

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

### 2. Ownership Testing Anti-Patterns

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

### 3. Async Testing Anti-Patterns

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

## Lithos Architecture Anti-Patterns

### 1. Context Boundary Violations

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

### 2. Port Testing Anti-Patterns

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

## Test Organization Anti-Patterns

### 1. Monolithic Test Modules

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

### 2. Test Data Management Anti-Patterns

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

## Performance Testing Anti-Patterns

### 1. Inaccurate Benchmarking

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

## CI/CD Integration Anti-Patterns

### 1. Flaky Tests in CI

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
