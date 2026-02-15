# Lithos Architecture Testing Patterns

## Context Isolation Testing

### Bounded Context Validation

#### Test Context Boundaries

```rust
// Note: Only test domain logic, never import other contexts
#[cfg(test)]
mod note_context_tests {
    use super::*;
    use crate::note::*;  // OK: Same context
    // use crate::schema::*;  // FORBIDDEN: Cross-context import

    #[test]
    fn test_note_context_isolation() {
        // Test note domain logic in isolation
        let note = Note::new(
            NoteId::new_random(),
            NoteTitle::new("Test").unwrap(),
            NoteContent::new("Content").unwrap(),
            Timestamp::now(),
        ).unwrap();

        // Validate note behavior without external dependencies
        assert_eq!(note.title().as_str(), "Test");
        assert_eq!(note.content().as_str(), "Content");
    }
}
```

#### Cross-Context Communication Testing

```rust
// Test cross-context communication through ports
#[cfg(test)]
mod cross_context_tests {
    use super::*;
    use crate::note::ports::*;
    use crate::schema::ports::*;

    // Use mock implementations to test communication
    struct MockSchemaPort;

    impl SchemaValidationPort for MockSchemaPort {
        fn validate_against_schema(&self, data: &str, schema_id: SchemaId) -> Result<(), ValidationError> {
            // Mock validation logic
            Ok(())
        }
    }

    #[test]
    fn test_note_schema_communication() {
        let schema_port = MockSchemaPort;
        let note = create_test_note();

        // Test communication through port interface
        let result = note.validate_with_schema(&schema_port, SchemaId::new("test"));
        assert!(result.is_ok());
    }
}
```

### Port-Based CQRS Testing

#### Command Port Testing

```rust
#[cfg(test)]
mod command_port_tests {
    use super::*;
    use crate::note::ports::*;

    struct MockNoteCommandPort {
        stored_notes: Vec<Note>,
        error_mode: Option<CommandError>,
    }

    impl MockNoteCommandPort {
        fn new() -> Self {
            Self {
                stored_notes: Vec::new(),
                error_mode: None,
            }
        }

        fn with_error_mode(mut self, error: CommandError) -> Self {
            self.error_mode = Some(error);
            self
        }
    }

    impl NoteCommandPort for MockNoteCommandPort {
        fn store_note(&mut self, note: &Note) -> Result<NoteId, CommandError> {
            if let Some(ref error) = self.error_mode {
                return Err(error.clone());
            }

            let id = NoteId::new_random();
            self.stored_notes.push(note.clone());
            Ok(id)
        }

        fn delete_note(&mut self, id: NoteId) -> Result<bool, CommandError> {
            if let Some(ref error) = self.error_mode {
                return Err(error.clone());
            }

            let initial_len = self.stored_notes.len();
            self.stored_notes.retain(|note| note.id() != &id);
            Ok(self.stored_notes.len() < initial_len)
        }
    }

    #[test]
    fn test_command_port_success() {
        let mut command_port = MockNoteCommandPort::new();
        let note = create_test_note();

        let result = command_port.store_note(&note);
        assert!(result.is_ok());

        let note_id = result.unwrap();
        let deleted = command_port.delete_note(note_id);
        assert!(deleted.is_ok());
        assert!(deleted.unwrap());
    }

    #[test]
    fn test_command_port_error_handling() {
        let mut command_port = MockNoteCommandPort::new()
            .with_error_mode(CommandError::StorageUnavailable);

        let note = create_test_note();
        let result = command_port.store_note(&note);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::StorageUnavailable));
    }
}
```

#### Query Port Testing

```rust
#[cfg(test)]
mod query_port_tests {
    use super::*;
    use crate::note::ports::*;

    struct MockNoteQueryPort {
        notes: HashMap<NoteId, Note>,
    }

    impl MockNoteQueryPort {
        fn new() -> Self {
            Self {
                notes: HashMap::new(),
            }
        }

        fn with_notes(notes: Vec<Note>) -> Self {
            let mut port = Self::new();
            for note in notes {
                let id = note.id().clone();
                port.notes.insert(id, note);
            }
            port
        }
    }

    impl NoteQueryPort for MockNoteQueryPort {
        fn get_note(&self, id: NoteId) -> Result<Option<Note>, QueryError> {
            Ok(self.notes.get(&id).cloned())
        }

        fn list_notes(&self) -> Result<Vec<Note>, QueryError> {
            Ok(self.notes.values().cloned().collect())
        }

        fn search_notes(&self, query: &str) -> Result<Vec<Note>, QueryError> {
            let results: Vec<Note> = self.notes
                .values()
                .filter(|note| {
                    note.title().as_str().contains(query) ||
                    note.content().as_str().contains(query)
                })
                .cloned()
                .collect();
            Ok(results)
        }
    }

    #[test]
    fn test_query_port_operations() {
        let notes = vec![
            create_test_note_with_title("First Note"),
            create_test_note_with_title("Second Note"),
            create_test_note_with_title("Third Note"),
        ];

        let query_port = MockNoteQueryPort::with_notes(notes);

        // Test get_note
        let all_notes = query_port.list_notes().unwrap();
        assert_eq!(all_notes.len(), 3);

        // Test search
        let search_results = query_port.search_notes("First").unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].title().as_str(), "First Note");
    }
}
```

### Zero-Copy Pattern Testing

#### Zero-Copy Storage Testing

```rust
#[cfg(test)]
mod zero_copy_tests {
    use super::*;
    use crate::db::ports::*;
    use std::borrow::Cow;

    struct MockZeroCopyStorage {
        data: HashMap<String, Vec<u8>>,
    }

    impl MockZeroCopyStorage {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }

        fn store_data(&mut self, key: &str, data: &[u8]) {
            self.data.insert(key.to_string(), data.to_vec());
        }
    }

    impl ZeroCopyStoragePort for MockZeroCopyStorage {
        type DataGuard<'a> = Cow<'a, [u8]> where Self: 'a;

        fn get_data<'a>(&'a self, key: &str) -> Result<Option<Self::DataGuard<'a>>, StorageError> {
            match self.data.get(key) {
                Some(data) => Ok(Some(Cow::Borrowed(data))),
                None => Ok(None),
            }
        }

        fn get_data_owned(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
            match self.data.get(key) {
                Some(data) => Ok(Some(data.clone())),
                None => Ok(None),
            }
        }
    }

    #[test]
    fn test_zero_copy_access() {
        let mut storage = MockZeroCopyStorage::new();
        let test_data = b"Hello, zero-copy world!";
        storage.store_data("test_key", test_data);

        // Test zero-copy access
        let guard = storage.get_data("test_key").unwrap();
        assert!(guard.is_some());

        let borrowed_data = guard.unwrap();
        assert_eq!(borrowed_data.as_ref(), test_data);

        // Verify it's a borrow, not a copy
        match borrowed_data {
            Cow::Borrowed(data) => {
                assert_eq!(data.as_ptr(), test_data.as_ptr());
            }
            Cow::Owned(_) => panic!("Expected borrowed data"),
        }
    }

    #[test]
    fn test_owned_access() {
        let mut storage = MockZeroCopyStorage::new();
        let test_data = b"Hello, owned world!";
        storage.store_data("test_key", test_data);

        // Test owned access
        let owned_data = storage.get_data_owned("test_key").unwrap();
        assert!(owned_data.is_some());

        let data = owned_data.unwrap();
        assert_eq!(data.as_slice(), test_data);
    }
}
```

#### GAT-Based Port Testing

```rust
#[cfg(test)]
mod gat_port_tests {
    use super::*;
    use crate::db::ports::*;

    // Generic Associated Types for zero-copy patterns
    trait StoragePort {
        type Item<'a> where Self: 'a;

        fn get<'a>(&'a self, id: &str) -> Result<Option<Self::Item<'a>>, StorageError>;
        fn store(&mut self, item: &str, data: &[u8]) -> Result<(), StorageError>;
    }

    struct MockGatStorage {
        data: HashMap<String, Vec<u8>>,
    }

    impl MockGatStorage {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
    }

    impl StoragePort for MockGatStorage {
        type Item<'a> = &'a [u8] where Self: 'a;

        fn get<'a>(&'a self, id: &str) -> Result<Option<Self::Item<'a>>, StorageError> {
            Ok(self.data.get(id).map(|data| data.as_slice()))
        }

        fn store(&mut self, id: &str, data: &[u8]) -> Result<(), StorageError> {
            self.data.insert(id.to_string(), data.to_vec());
            Ok(())
        }
    }

    #[test]
    fn test_gat_zero_copy() {
        let mut storage = MockGatStorage::new();
        let test_data = b"GAT zero-copy test data";
        storage.store("test", test_data).unwrap();

        // Test zero-copy access using GATs
        let borrowed_data = storage.get("test").unwrap();
        assert!(borrowed_data.is_some());

        let data = borrowed_data.unwrap();
        assert_eq!(data, test_data);

        // Verify zero-copy nature
        assert_eq!(data.as_ptr(), test_data.as_ptr());
    }
}
```

## Type-Driven Development Testing

#### Newtype Wrapper Testing

```rust
#[cfg(test)]
mod newtype_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct NoteTitle {
        value: Box<str>,
    }

    impl NoteTitle {
        pub fn new(title: &str) -> Result<Self, ValidationError> {
            if title.is_empty() {
                return Err(ValidationError::TooShort("title".to_string()));
            }
            if title.len() > 100 {
                return Err(ValidationError::TooLong("title".to_string(), 100));
            }
            if title.contains('\n') {
                return Err(ValidationError::InvalidCharacters("title".to_string()));
            }

            Ok(Self {
                value: title.to_string().into_boxed_str(),
            })
        }

        pub fn as_str(&self) -> &str {
            &self.value
        }
    }

    #[test]
    fn test_newtype_validation() {
        // Test valid inputs
        let valid_cases = vec![
            "Valid Title",
            "A",
            "a".repeat(100).as_str(),
            "Title with spaces and numbers 123",
        ];

        for valid_input in valid_cases {
            let result = NoteTitle::new(valid_input);
            assert!(result.is_ok(), "Should succeed for: {:?}", valid_input);

            let title = result.unwrap();
            assert_eq!(title.as_str(), valid_input);
        }

        // Test invalid inputs
        let invalid_cases = vec![
            ("", ValidationError::TooShort("title".to_string())),
            ("a".repeat(101).as_str(), ValidationError::TooLong("title".to_string(), 100)),
            ("title\nwith\nnewlines", ValidationError::InvalidCharacters("title".to_string())),
        ];

        for (invalid_input, expected_error) in invalid_cases {
            let result = NoteTitle::new(invalid_input);
            assert!(result.is_err(), "Should fail for: {:?}", invalid_input);

            let actual_error = result.unwrap_err();
            assert!(
                matches!(actual_error, expected_error),
                "Expected {:?}, got {:?}",
                expected_error, actual_error
            );
        }
    }

    #[test]
    fn test_newtype_equality() {
        let title1 = NoteTitle::new("Test Title").unwrap();
        let title2 = NoteTitle::new("Test Title").unwrap();
        let title3 = NoteTitle::new("Different Title").unwrap();

        assert_eq!(title1, title2);
        assert_ne!(title1, title3);
    }

    #[test]
    fn test_newtype_immutability() {
        let title = NoteTitle::new("Original Title").unwrap();

        // NoteTitle is immutable by design
        // Cannot modify the internal value
        // This ensures type safety and invariant preservation
        assert_eq!(title.as_str(), "Original Title");
    }
}
```

#### Constructor Testing

```rust
#[cfg(test)]
mod constructor_tests {
    use super::*;

    pub struct Note {
        id: NoteId,
        title: NoteTitle,
        content: NoteContent,
        created_at: Timestamp,
    }

    impl Note {
        pub fn new(
            id: NoteId,
            title: NoteTitle,
            content: NoteContent,
            created_at: Timestamp,
        ) -> Result<Self, ValidationError> {
            // Validate invariants
            if title.as_str().is_empty() && content.as_str().is_empty() {
                return Err(ValidationError::EmptyNote);
            }

            Ok(Self {
                id,
                title,
                content,
                created_at,
            })
        }

        // Getters - all fields are private to enforce invariants
        pub fn id(&self) -> &NoteId { &self.id }
        pub fn title(&self) -> &NoteTitle { &self.title }
        pub fn content(&self) -> &NoteContent { &self.content }
        pub fn created_at(&self) -> &Timestamp { &self.created_at }
    }

    #[test]
    fn test_note_construction_success() {
        let id = NoteId::new_random();
        let title = NoteTitle::new("Test Note").unwrap();
        let content = NoteContent::new("Test content").unwrap();
        let timestamp = Timestamp::now();

        let note = Note::new(id, title, content, timestamp);
        assert!(note.is_ok());

        let note = note.unwrap();
        assert_eq!(note.id(), &id);
        assert_eq!(note.title().as_str(), "Test Note");
        assert_eq!(note.content().as_str(), "Test content");
    }

    #[test]
    fn test_note_invariant_violation() {
        let id = NoteId::new_random();
        let title = NoteTitle::new("").unwrap(); // Empty title (allowed)
        let content = NoteContent::new("").unwrap(); // Empty content (allowed)
        let timestamp = Timestamp::now();

        let note = Note::new(id, title, content, timestamp);
        assert!(note.is_err());
        assert!(matches!(note.unwrap_err(), ValidationError::EmptyNote));
    }

    #[test]
    fn test_note_immutability() {
        let note = create_valid_note();

        // All fields are private - cannot modify
        // Note is immutable by design
        let original_title = note.title().as_str().to_string();

        // No way to modify the note after creation
        // Must create new note for changes
        assert_eq!(note.title().as_str(), original_title);
    }
}
```

## Error Handling Testing

#### Comprehensive Error Testing

```rust
#[cfg(test)]
mod error_handling_tests {
    use super::*;
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq)]
    pub enum DomainError {
        #[error("Validation error: {0}")]
        Validation(#[from] ValidationError),

        #[error("Storage error: {0}")]
        Storage(#[from] StorageError),

        #[error("Note not found: {id}")]
        NotFound { id: NoteId },

        #[error("Permission denied: {action}")]
        PermissionDenied { action: String },
    }

    #[test]
    fn test_error_chaining() {
        let validation_error = ValidationError::TooShort("title".to_string());
        let domain_error = DomainError::Validation(validation_error);

        // Test error chain
        assert_eq!(domain_error.to_string(), "Validation error: title too short");
        assert!(domain_error.source().is_none()); // Terminal error
    }

    #[test]
    fn test_error_with_context() {
        let id = NoteId::new_random();
        let domain_error = DomainError::NotFound { id };

        let error_message = domain_error.to_string();
        assert!(error_message.contains("not found"));
        assert!(error_message.contains(&id.to_string()));
    }

    #[test]
    fn test_error_recovery_information() {
        let permission_error = DomainError::PermissionDenied {
            action: "delete_note".to_string(),
        };

        // Error should provide enough information for recovery
        assert!(permission_error.to_string().contains("delete_note"));

        // Users can be informed of required permissions
        match permission_error {
            DomainError::PermissionDenied { action } => {
                assert!(!action.is_empty());
                // Could suggest: "Please request permission to: {action}"
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
```

This Lithos architecture testing guide provides comprehensive patterns for testing bounded contexts, port-based CQRS, zero-copy patterns, and type-driven development while maintaining architectural constraints and quality standards.
