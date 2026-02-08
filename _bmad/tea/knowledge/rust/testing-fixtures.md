# TEA Knowledge: Test Fixture Strategies

## CONTEXT
- **Applies to**: Test data setup and helper functions
- **Purpose**: Reproducible, isolated test data
- **Location**: Inline in `#[cfg(test)]` modules

## DECISION TREE: What Fixture Strategy to Use?

```
What kind of test data do you need?
├── Simple, one-time use?
│   └── → Inline directly in test
│
├── Used in multiple tests in same module?
│   └── → Helper function in test module
│
├── Complex setup with dependencies?
│   └── → rstest fixture with #[fixture]
│
├── Expensive setup shared across tests?
│   └── → rstest #[once] fixture
│
├── Filesystem operations?
│   └── → tempfile::TempDir
│
├── Deterministic random data?
│   └── → proptest strategies with fixed seeds
│
└── Cross-test data (golden files)?
    └── → tests/fixtures/ directory
```

## VALIDATION CHECKLIST

### Inline Fixtures
- [ ] Used for simple, test-specific data
- [ ] Not extracted to helper unless reused
- [ ] Clear what the data represents

### Helper Functions
- [ ] Simple functions (not macros)
- [ ] Document purpose in doc comment
- [ ] Accept parameters for customization
- [ ] Return ready-to-use test objects

### rstest Fixtures
- [ ] Uses `#[fixture]` attribute
- [ ] Named descriptively
- [ ] Can be composed (fixtures using fixtures)
- [ ] Uses `#[once]` for expensive shared setup

### Filesystem Fixtures
- [ ] Uses `tempfile::TempDir` (not hardcoded paths)
- [ ] Creates fresh temp dir per test
- [ ] Relies on RAII cleanup (no manual deletion)

### Property Test Data
- [ ] Uses `proptest` strategies
- [ ] Has deterministic seeds
- [ ] Filters for valid ranges

### Golden/Reference Files
- [ ] Stored in `tests/fixtures/` or `docs/refs/`
- [ ] Version controlled
- [ ] Documented update process

## ANTI-PATTERNS (FLAG THESE)

### Location Issues
- ❌ **External test utility crates** → All fixtures inline
- ❌ **Shared test modules between crates** → Each crate independent
- ❌ **Test data in `src/`** → Keep test data in `tests/` or inline

### Fixture Complexity
- ❌ **Complex builder patterns** → Use simple helper functions
- ❌ **Macros for fixtures** → Use functions
- ❌ **Overly generic fixtures** → Specific fixtures per test context

### Filesystem Issues
- ❌ **Hardcoded paths** → Use `tempfile::TempDir`
- ❌ **Manual cleanup (fs::remove_file)** → Use RAII (Drop)
- ❌ **Shared temp directories** → Fresh TempDir per test
- ❌ **Tests depending on existing files** → Create in temp dir

### State Issues
- ❌ **Shared mutable state** → Independent fixtures per test
- ❌ **Static variables** → Use fixtures
- ❌ **Database not reset** → Fresh in-memory DB per test

## CORRECT EXAMPLES

### Inline Fixtures
```rust
#[test]
fn validates_single_character_name() {
    // Simple, test-specific data - inline it
    let name = "a";
    let result = validate_name(name);
    assert!(result.is_ok());
}

#[test]
fn rejects_empty_name() {
    let name = "";
    let result = validate_name(name);
    assert!(result.is_err());
}
```

### Helper Functions
```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test note with default values
    fn note_fixture(path: &str) -> Note {
        Note {
            id: Uuid::now_v7(),
            path: NotePath::new(path.to_owned()).expect("Valid test path"),
            content: "Test content".to_owned(),
            tags: vec![],
            created_at: Utc::now(),
        }
    }

    /// Creates a test note with custom content
    fn note_with_content(path: &str, content: &str) -> Note {
        Note {
            path: NotePath::new(path.to_owned()).expect("Valid test path"),
            content: content.to_owned(),
            ..note_fixture(path) // Use default for other fields
        }
    }

    #[test]
    fn validates_note_with_default_fixture() {
        let note = note_fixture("test.md");
        assert!(note.validate().is_ok());
    }

    #[test]
    fn validates_note_with_custom_content() {
        let note = note_with_content("test.md", "# Heading");
        assert!(note.validate().is_ok());
    }
}
```

### rstest Fixtures
```rust
use rstest::*;

#[fixture]
fn valid_note_id() -> Uuid {
    Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
}

#[fixture]
fn valid_note_path() -> NotePath {
    NotePath::new("notes/test.md".to_owned()).unwrap()
}

// Composition: fixtures using fixtures
#[fixture]
fn test_note(valid_note_id: Uuid, valid_note_path: NotePath) -> Note {
    Note {
        id: valid_note_id,
        path: valid_note_path,
        content: "Test content".to_owned(),
        tags: vec![],
    }
}

#[rstest]
fn validates_test_note(test_note: Note) {
    assert!(test_note.validate().is_ok());
}
```

### rstest with Defaults
```rust
#[fixture]
fn note_with_tags(
    #[default(Uuid::now_v7())] id: Uuid,
    #[default("notes/default.md")] path: &str,
    #[default(vec!["#test".to_owned()])] tags: Vec<String>,
) -> Note {
    Note {
        id,
        path: NotePath::new(path.to_owned()).unwrap(),
        content: "Content".to_owned(),
        tags: tags.iter()
            .map(|t| Tag::new(t).unwrap())
            .collect(),
    }
}

// Use defaults
#[rstest]
fn test_with_defaults(note_with_tags: Note) {
    assert_eq!(note_with_tags.tags.len(), 1);
}

// Override defaults
#[rstest]
fn test_with_custom_tags(
    #[with(vec!["#work".to_owned(), "#urgent".to_owned()])]
    note_with_tags: Note
) {
    assert_eq!(note_with_tags.tags.len(), 2);
}
```

### Once Fixtures (Shared Setup)
```rust
#[fixture]
#[once]
fn shared_database() -> Arc<Database> {
    // Expensive setup - runs once for all tests
    let db = Database::new_in_memory();
    db.seed_test_data();
    Arc::new(db)
}

#[rstest]
fn test_insert(shared_database: &Arc<Database>) {
    shared_database.insert("key", "value").unwrap();
}

#[rstest]
fn test_query(shared_database: &Arc<Database>) {
    // Same database instance
    let result = shared_database.query("key");
    assert!(result.is_ok());
}
```

### Filesystem Fixtures
```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn writes_file_successfully() -> std::io::Result<()> {
    // Arrange
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    // Act
    fs::write(&file_path, "test content")?;
    let content = fs::read_to_string(&file_path)?;

    // Assert
    assert_eq!(content, "test content");
    Ok(())
    // temp_dir auto-cleaned when dropped
}
```

### Vault Fixture Pattern
```rust
#[fixture]
fn temp_vault() -> TempDir {
    let temp = TempDir::new().expect("Create temp dir");

    // Create vault structure
    fs::create_dir(temp.path().join(".lithos")).unwrap();
    fs::write(
        temp.path().join(".lithos/config.toml"),
        r#"version = "1.0""#
    ).unwrap();

    // Create some notes
    fs::write(
        temp.path().join("hello.md"),
        "# Hello\n\nWorld"
    ).unwrap();

    temp
}

#[rstest]
fn indexes_vault_contents(temp_vault: TempDir) {
    let vault = Vault::open(temp_vault.path()).unwrap();
    let index = vault.index().unwrap();
    assert_eq!(index.note_count(), 1);
}
```

### Property Test Data
```rust
use proptest::prelude::*;

// Strategy for valid identifiers
fn valid_id_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,63}"
}

// Strategy for note paths
fn note_path_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-/]+\.md"
}

// Composite strategy
prop_compose! {
    fn note_strategy()(
        path in note_path_strategy(),
        title in "[^\n]{1,100}"
    ) -> Note {
        Note::builder()
            .path(path)
            .title(title)
            .build()
            .unwrap()
    }
}

proptest! {
    #[test]
    fn validates_generated_notes(note in note_strategy()) {
        assert!(note.validate().is_ok());
    }
}
```

## FIXTURE SCOPE REFERENCE

| Scope | When to Use | Example |
|-------|-------------|---------|
| **Inline** | One-time use, simple data | `let name = "test"` |
| **Helper fn** | Reused in same module | `fn note_fixture() -> Note` |
| **rstest** | Complex setup, dependencies | `#[fixture] fn test_note()` |
| **rstest once** | Expensive shared setup | `#[fixture] #[once] fn db()` |
| **TempDir** | Filesystem operations | `TempDir::new().unwrap()` |
| **Golden files** | Reference data | `tests/fixtures/data.json` |

## QUICK REFERENCE

| Pattern | Usage |
|---------|-------|
| Inline fixtures | Simple, test-specific data |
| Helper functions | Reused in multiple tests |
| rstest fixtures | Complex setup with dependencies |
| Once fixtures | Expensive shared resources |
| TempDir | Filesystem isolation |
| Proptest | Deterministic random data |

## RELATED MODULES
- See `testing-unit.md` for test structure
- See `testing-tools-rstest.md` for rstest patterns
- See `testing-tools-proptest.md` for property testing
- See `testing-anti-patterns.md` for comprehensive anti-patterns
