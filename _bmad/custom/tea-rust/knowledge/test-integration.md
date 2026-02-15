# TEA Knowledge: Rust Integration Testing

## CONTEXT

- **Applies to**: Tests in `lithos-core/tests/` directory
- **Location**: External to the library crate
- **Purpose**: Testing public API, port contracts, adapter implementations
- **Can test**: I/O operations, external dependencies, persistence layers

## DECISION TREE: Integration vs Unit Test

```
Does the test require...
├── Real I/O (filesystem, database)?
│   └── YES → Integration test in tests/
│
├── External dependencies (network, services)?
│   └── YES → Integration test in tests/
│
├── Testing port trait implementations?
│   └── YES → Integration test with real adapters
│
├── Testing persistence side effects?
│   └── YES → Integration test
│
├── Multiple component coordination?
│   └── YES → Integration test
│
└── Pure logic with no side effects?
    └── YES → Unit test (not integration)
```

## VALIDATION CHECKLIST

### Test Location

- [ ] Test is in `lithos-core/tests/` directory (not `src/`)
- [ ] Test file name follows `*_test.rs` or describes the component (e.g., `adapter_test.rs`)
- [ ] Test uses public API only (no access to private items)

### Test Structure

- [ ] Uses `use lithos_core::*;` to import the library
- [ ] Tests one port/adapter or one integration scenario per file
- [ ] Organized into modules by functionality

### I/O and Side Effects

- [ ] Uses `tempfile::TempDir` for filesystem operations
- [ ] Uses in-memory or temporary database instances (e.g., `redb` with temp file)
- [ ] Cleans up resources via RAII (Drop implementations)
- [ ] No hardcoded paths or environment-dependent resources

### Port Testing

- [ ] Tests both success and error paths for port operations
- [ ] Verifies side effects (data persisted correctly)
- [ ] Tests transaction rollback behavior where applicable
- [ ] Uses real adapters (not mocks) for integration tests

### Mock Usage

- [ ] Uses `mockall` for mocking external dependencies
- [ ] Sets clear expectations on mocks (`expect_*` methods)
- [ ] Verifies mock expectations were met (`times(1)`, etc.)

### Performance

- [ ] Integration tests complete in < 100ms median
- [ ] Uses `#[serial]` or test groups for tests that cannot run in parallel
- [ ] Avoids unnecessary I/O operations

## ANTI-PATTERNS (FLAG THESE)

### Location Issues

- ❌ **Integration test in `src/`** → Move to `tests/` directory
- ❌ **Testing private functions** → Either make public or move to unit test
- ❌ **Mixed concerns in one test file** → Split by component/adapter

### I/O Issues

- ❌ **Hardcoded file paths** → Use `tempfile::TempDir`
- ❌ **Manual cleanup (fs::remove_file)** → Use RAII (TempDir drops automatically)
- ❌ **Tests depending on specific filesystem state** → Create fresh temp dir per test
- ❌ **Network calls without mocking** → Mock external services

### Isolation Issues

- ❌ **Shared database between tests** → Use fresh in-memory DB per test
- ❌ **Tests depending on execution order** → Each test must be independent
- ❌ **Static/shared mutable state** → Use fixtures, not globals

### Mock Issues

- ❌ **Mocks without expectations set** → Always set `expect_*`
- ❌ **Over-mocking (mocking everything)** → Only mock external boundaries
- ❌ **Not verifying mock calls** → Check `times()` and call `verify()`

## CORRECT EXAMPLES

### Basic Integration Test

```rust
// Database Integration Example
#[cfg(test)]
mod database_integration_tests {
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

// lithos-core/tests/storage_adapter_test.rs
use lithos_core::*;
use tempfile::TempDir;

#[test]
fn persists_note_to_database() {
    // Arrange
    let temp_dir = TempDir::new().expect("Create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let adapter = RedbStorageAdapter::new(&db_path).expect("Create adapter");
    let note = create_test_note("test.md");

    // Act
    adapter.save(&note).expect("Save note");

    // Assert
    let retrieved = adapter.load(note.id()).expect("Load note");
    assert_eq!(retrieved.path(), note.path());
    // temp_dir is automatically cleaned up when dropped
}
```

### Port Contract Test

```rust
// lithos-core/tests/port_contract_test.rs
use lithos_core::ports::*;
use lithos_core::adapters::InMemoryStorageAdapter;

#[test]
fn storage_port_contract_honors_transactions() {
    let mut storage = InMemoryStorageAdapter::new();
    let note = create_test_note("test.md");

    // Test: Save should persist
    storage.save(&note).expect("Save succeeds");
    assert!(storage.exists(note.id()));

    // Test: Delete should remove
    storage.delete(note.id()).expect("Delete succeeds");
    assert!(!storage.exists(note.id()));
}
```

### With Mockall

```rust
use lithos_core::ports::*;
use mockall::predicate::*;
use mockall::mock;

mock! {
    ExternalService {}

    #[async_trait]
    impl ExternalServicePort for ExternalService {
        async fn fetch(&self, url: &str) -> Result<String, Error>;
    }
}

#[test]
fn service_retries_on_timeout() {
    let mut mock = MockExternalService::new();

    // Set expectation: fetch will be called twice (retry)
    mock.expect_fetch()
        .with(eq("https://api.example.com/data"))
        .times(2)
        .returning(|_| Err(Error::Timeout));

    let service = MyService::new(mock);
    let result = service.fetch_with_retry("https://api.example.com/data");

    assert!(matches!(result, Err(Error::Timeout)));
}
```

### Test Groups (Serial Execution)

```rust
// For tests that cannot run in parallel
use serial_test::serial;

#[test]
#[serial]
fn test_database_migration() {
    // This test modifies global state and must run serially
}

// Or use nextest test groups in .config/nextest.toml
```

## CQRS INTEGRATION TESTING

### Command Testing

```rust
#[test]
fn create_schema_command_persists_valid_schema() {
    let mut storage = InMemoryCommandPort::new();
    let cmd = CreateSchemaCommand::new("Task", vec![...]);

    let result = cmd.execute(&mut storage);

    assert!(result.is_ok());
    assert!(storage.contains_schema("Task"));
}
```

### Query Testing

```rust
#[test]
fn find_schema_by_id_returns_correct_schema() {
    let storage = InMemoryQueryPort::with_seed(vec![task_schema()]);
    let query = FindSchemaById::new(task_schema_id());

    let result = query.execute(&storage);

    assert_eq!(result.unwrap().name, "Task");
}
```

## RELATED MODULES

- See `test-unit.md` for unit testing patterns
- See `tools-nextest.md` for test runner configuration
- See `fixtures.md` for fixture strategies
- See `anti-patterns.md` for comprehensive anti-patterns
