# Tea-Rust Knowledge Base

## Rust Testing Best Practices

### Unit Testing Patterns

#### Test Organization
```rust
// Preferred test module organization
#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    // Group related tests together
    mod creation_tests {
        use super::*;

        #[test]
        fn test_valid_creation() {
            // Test implementation
        }

        #[test]
        fn test_invalid_input_handling() {
            // Test implementation
        }
    }

    mod validation_tests {
        use super::*;

        #[test]
        fn test_field_validation() {
            // Test implementation
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_error_variants() {
            // Test implementation
        }
    }
}
```

#### Test Data Management
```rust
// Use builder patterns for test data
pub struct NoteTestBuilder {
    title: Option<String>,
    content: Option<String>,
    timestamp: Option<Timestamp>,
}

impl NoteTestBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_title(mut self, title: &str) -> Self { /* ... */ }
    pub fn with_content(mut self, content: &str) -> Self { /* ... */ }
    pub fn build(self) -> Result<Note, ValidationError> { /* ... */ }
}

// Use fixtures for expensive data
use once_cell::sync::Lazy;

static LARGE_TEST_DATASET: Lazy<Vec<Note>> = Lazy::new(|| {
    // Generate test data once
});
```

### Property-Based Testing

#### Proptest Integration
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_note_validation_roundtrip(
        title in "[a-zA-Z0-9 ]{1,100}",
        content in "[a-zA-Z0-9 .,!?\n]{1,10000}"
    ) {
        let result = Note::new(
            NoteId::new_random(),
            NoteTitle::new(&title).unwrap(),
            NoteContent::new(&content).unwrap(),
            Timestamp::now(),
        );

        prop_assert!(result.is_ok());
        let note = result.unwrap();
        prop_assert_eq!(note.title().as_str(), title);
        prop_assert_eq!(note.content().as_str(), content);
    }
}
```

#### Custom Test Generators
```rust
use proptest::strategy::{Strategy, Just, BoxedStrategy};

impl Arbitrary for NoteTitle {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        "[a-zA-Z0-9 ]{1,100}"
            .prop_map(|s| NoteTitle::new(&s).unwrap())
            .boxed()
    }
}
```

### Mock Testing Patterns

#### Trait-Based Mocking
```rust
#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockNoteStorage {
        notes: HashMap<NoteId, Note>,
        failure_mode: Option<StorageError>,
    }

    impl MockNoteStorage {
        pub fn new() -> Self {
            Self {
                notes: HashMap::new(),
                failure_mode: None,
            }
        }

        pub fn with_failure_mode(mut self, error: StorageError) -> Self {
            self.failure_mode = Some(error);
            self
        }
    }

    impl NoteStoragePort for MockNoteStorage {
        fn get_note(&self, id: NoteId) -> Result<Option<Note>, StorageError> {
            if let Some(ref error) = self.failure_mode {
                return Err(error.clone());
            }
            Ok(self.notes.get(&id).cloned())
        }

        fn store_note(&mut self, note: &Note) -> Result<NoteId, StorageError> {
            if let Some(ref error) = self.failure_mode {
                return Err(error.clone());
            }
            let id = note.id().clone();
            self.notes.insert(id.clone(), note.clone());
            Ok(id)
        }
    }
}
```

### Integration Testing

#### Database Integration
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_integration() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut storage = RedbNoteStorage::new(&db_path).unwrap();

        // Test actual storage operations
        let note = create_test_note();
        let note_id = storage.store_note(&note).unwrap();

        let retrieved = storage.get_note(note_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title(), note.title());
    }
}
```

## Performance Testing

### Criterion Benchmarks

#### Basic Benchmark Structure
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_note_creation(c: &mut Criterion) {
    let title = "Test Note Title";
    let content = "Test note content";

    c.bench_function("note_creation", |b| {
        b.iter(|| {
            let note = Note::new(
                NoteId::new_random(),
                NoteTitle::new(black_box(title)).unwrap(),
                NoteContent::new(black_box(content)).unwrap(),
                Timestamp::now(),
            );
            black_box(note)
        })
    });
}

criterion_group!(benches, benchmark_note_creation);
criterion_main!(benches);
```

#### Comparative Benchmarks
```rust
fn benchmark_search_algorithms(c: &mut Criterion) {
    let notes: Vec<Note> = (0..1000).map(|i| create_test_note(i)).collect();
    let search_term = "test";

    c.bench_function("linear_search", |b| {
        b.iter(|| {
            notes.iter().find(|note| note.content().contains(search_term))
        })
    });

    let indexed_notes = create_indexed_notes(&notes);

    c.bench_function("indexed_search", |b| {
        b.iter(|| {
            indexed_notes.get(search_term)
        })
    });
}
```

### Memory Profiling

#### Allocation Tracking
```rust
#[test]
fn test_memory_efficiency() {
    let notes: Vec<Note> = (0..1000).map(|i| create_test_note(i)).collect();

    // Use criterion's memory profiling
    let mut group = criterion::BenchmarkGroup::new("memory_usage");
    group.measured_time = false;

    group.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let processed: Vec<ProcessedNote> = notes
                .iter()
                .map(|note| process_note(black_box(note)))
                .collect();
            black_box(processed.len())
        })
    });

    group.finish();
}
```

## Quality Gates

### Coverage Requirements

#### Meaningful Coverage Targets
```yaml
coverage_targets:
  critical_paths: "100%"
  public_apis: "100%"
  error_handling: "95%"
  business_logic: "90%"

coverage_quality:
  branch_coverage: ">= 85%"
  condition_coverage: ">= 80%"
  mutation_score: ">= 70%"
```

#### Coverage Exclusions
```rust
// Exclude certain code from coverage if justified
#[cfg(not(test))]
pub mod production_only_code {
    // Code that cannot be meaningfully tested
}

// Or use coverage comments
#[cfg(test)]
mod coverage_tests {
    #[test]
    fn test_unreachable_code() {
        // This code path is unreachable in practice
        // and is excluded from coverage requirements
    }
}
```

### Performance Requirements

#### Performance Benchmarks
```yaml
performance_requirements:
  note_creation:
    p50_latency: "< 5ms"
    p95_latency: "< 10ms"
    p99_latency: "< 20ms"

  batch_operations:
    throughput: "> 1000 ops/sec"
    memory_efficiency: "< 2KB per note"

  search_operations:
    linear_search_1000_items: "< 50ms"
    indexed_search: "< 1ms"
```

#### Regression Detection
```rust
pub struct PerformanceRegressionDetector {
    baselines: HashMap<String, PerformanceBaseline>,
    tolerance: f64,
}

impl PerformanceRegressionDetector {
    pub fn check_regression(&self, current: &PerformanceMetrics) -> Vec<RegressionAlert> {
        // Check for performance regressions
        let mut alerts = Vec::new();

        for (metric, current_value) in current.metrics() {
            if let Some(baseline) = self.baselines.get(metric) {
                let regression = (current_value - baseline.value) / baseline.value;
                if regression > self.tolerance {
                    alerts.push(RegressionAlert::new(metric, regression));
                }
            }
        }

        alerts
    }
}
```

## Error Testing

#### Comprehensive Error Testing
```rust
#[test]
fn test_all_error_variants() {
    let test_cases = vec![
        ("", ValidationError::EmptyTitle),
        ("a".repeat(101), ValidationError::TitleTooLong),
        ("invalid", ValidationError::InvalidFormat),
    ];

    for (input, expected_error) in test_cases {
        let result = validate_title(input);
        match result {
            Err(actual_error) => {
                assert!(
                    matches!(actual_error, expected_error),
                    "Expected {:?}, got {:?}",
                    expected_error, actual_error
                );
            }
            Ok(_) => panic!("Expected error for input: {:?}", input),
        }
    }
}
```

#### Error Recovery Testing
```rust
#[test]
fn test_error_recovery_mechanisms() {
    let storage = setup_storage_with_fault_injection();

    // Test that system recovers from transient errors
    storage.inject_error(StorageError::TemporaryFailure);

    let result = store_test_note(&storage);
    assert!(result.is_err());

    // Remove error injection and retry
    storage.clear_error_injection();

    let retry_result = store_test_note(&storage);
    assert!(retry_result.is_ok());
}
```

## CI/CD Integration

### Quality Gate Configuration
```yaml
ci_quality_gates:
  test_execution:
    unit_tests: "required"
    integration_tests: "required"
    benchmarks: "required"

  quality_checks:
    code_coverage: ">= 85%"
    mutation_testing: ">= 70%"
    performance_regression: "< 5%"

  artifact_generation:
    test_reports: "required"
    coverage_reports: "required"
    benchmark_reports: "required"
```

### Automated Testing Pipeline
```bash
#!/bin/bash
# Comprehensive testing pipeline

set -euo pipefail

echo "🧪 Starting comprehensive test suite"
echo "===================================="

# 1. Fast unit tests
echo "Running unit tests..."
cargo test --lib

# 2. Integration tests
echo "Running integration tests..."
cargo test --test '*'

# 3. Coverage analysis
echo "Generating coverage reports..."
cargo tarpaulin --out html

# 4. Benchmarks
echo "Running benchmarks..."
cargo bench

# 5. Quality checks
echo "Running quality checks..."
cargo clippy -- -D warnings
cargo fmt -- --check

echo "✅ All tests passed successfully!"
```

This knowledge base provides comprehensive guidance for implementing high-quality testing practices in Rust projects, specifically tailored for the Lithos architecture and tea-rust workflow system.
