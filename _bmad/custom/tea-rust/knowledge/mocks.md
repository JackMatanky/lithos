# TEA Knowledge: Mocking Patterns

## CONTEXT

- **Applies to**: Unit and Integration tests requiring dependency isolation
- **Purpose**: Simulate external component behavior, test error paths, and verify interactions
- **Tools**: `mockall`, Trait-based manual mocks

## DECISION TREE: Mocking Strategy

```
What are you mocking?
├── Internal domain component?
│   └── → Use trait-based manual mocks or mockall
│
├── External crate dependency?
│   └── → Wrap in a local trait first, then mock the trait
│
├── Filesystem?
│   └── → Use tempfile::TempDir (not a mock)
│
└── Database/Persistence?
    └── → Use in-memory adapter or mock the storage port
```

## VALIDATION CHECKLIST

### Mock Design

- [ ] Mocks are based on traits (ports), not concrete types
- [ ] Dependencies are injected via constructor or method parameters
- [ ] Mock logic is minimal and predictable

### mockall Usage

- [ ] Uses `#[automock]` on traits where possible
- [ ] Expectations are set explicitly (`expect_*`)
- [ ] Call counts are verified (`times(1)`, etc.)

### Manual Mocks

- [ ] Implements the required trait with a struct containing state (e.g., `HashMap`)
- [ ] Provides error injection modes for testing failure paths

## ANTI-PATTERNS (FLAG THESE)

- ❌ **Over-mocking** → Mocking everything instead of using real pure logic
- ❌ **Mocking implementation details** → Mocking internal private methods
- ❌ **Fragile expectations** → Verifying exact call order when it doesn't matter
- ❌ **No verification** → Setting expectations but never checking if they were called

## CORRECT EXAMPLES

### Trait-Based Manual Mocking

```rust
#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::collections::HashMap;

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

### mockall Integration

```rust
use mockall::predicate::*;
use mockall::automock;

#[automock]
pub trait DataProvider {
    fn get_data(&self, id: u32) -> Result<String, DataError>;
}

#[test]
fn test_consumer_with_mock() {
    let mut mock = MockDataProvider::new();
    mock.expect_get_data()
        .with(eq(42))
        .times(1)
        .returning(|_| Ok("mocked data".to_string()));

    let consumer = DataConsumer::new(Box::new(mock));
    let result = consumer.process(42);
    assert_eq!(result.unwrap(), "processed: mocked data");
}
```

## RELATED MODULES

- See `test-unit.md` for unit testing context
- See `test-integration.md` for integration testing context
- See `anti-patterns.md` for comprehensive anti-patterns
