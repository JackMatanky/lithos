# TEA Knowledge: rstest Patterns

## CONTEXT

- **Tool**: `rstest` - Parameterized testing and fixtures for Rust
- **Purpose**: Reduce boilerplate, improve test clarity
- **Crates**: `rstest`, `rstest_reuse`

## DECISION TREE: When to Use rstest

```
Are you...
├── Testing same logic with multiple inputs?
│   └── YES → Use #[rstest] with #[case::name(...)]
│
├── Sharing test setup across tests?
│   └── YES → Use #[fixture]
│
├── Needing async test fixtures?
│   └── YES → Use #[fixture] + #[future]
│
├── Testing combinations of values?
│   └── YES → Use #[values(...)]
│
├── Needing expensive setup shared across tests?
│   └── YES → Use #[fixture] with #[once]
│
└── Reusing test cases across different functions?
    └── YES → Use rstest_reuse with #[template]
```

## VALIDATION CHECKLIST

### Parameterized Tests

- [ ] Uses `#[rstest]` attribute
- [ ] Uses `#[case::descriptive_name(...)]` for named cases
- [ ] Each case has a meaningful name (not just `#[case(...)]`)
- [ ] Test parameters match case arguments

### Fixtures

- [ ] Uses `#[fixture]` attribute
- [ ] Fixture functions are descriptive
- [ ] Fixtures can be composed (fixture using fixture)
- [ ] Uses `#[default(...)]` for fixture parameters

### Once Fixtures

- [ ] Uses `#[once]` for expensive shared setup
- [ ] Returns `Arc<T>` or reference for shared access
- [ ] Does not mutate shared state

### Values Combinations

- [ ] Uses `#[values(...)]` for combinatorial testing
- [ ] Understands this generates N×M test cases
- [ ] Not overused (explosion of test cases)

### Async Support

- [ ] Uses `#[future]` for async fixtures
- [ ] Properly awaits fixtures in test

## ANTI-PATTERNS (FLAG THESE)

### Case Naming

- ❌ `#[case("foo", true)]` → Use `#[case::valid_foo("foo", true)]`
- ❌ `#[case(1, 2, 3)]` → No description of what case tests
- ❌ Generic names like `case_1`, `case_2` → Use descriptive names

### Fixture Issues

- ❌ Fixtures with side effects → Should be pure
- ❌ Mutable shared fixtures → Use independent fixtures
- ❌ Overly complex fixture chains → Keep it simple
- ❌ `#[once]` fixtures that should be per-test → Use without `#[once]`

### Parameter Issues

- ❌ Too many parameters → Use struct/fixture
- ❌ Unrelated parameters in one test → Split tests
- ❌ `#[values]` explosion → 10×10 = 100 tests, be careful

### Usage Issues

- ❌ Using rstest for single test case → Standard `#[test]` is fine
- ❌ Not using `#[default(...)]` when applicable → Provides flexibility

## CORRECT EXAMPLES

### Basic Parameterized Test

```rust
use rstest::*;

#[rstest]
#[case::empty_string("", 0)]
#[case::single_char("a", 1)]
#[case::multiple_words("hello world", 11)]
fn string_length_matches_expected(
    #[case] input: &str,
    #[case] expected: usize
) {
    assert_eq!(input.len(), expected);
}
```

### Validation Testing

```rust
#[rstest]
#[case::valid_simple("hello", true)]
#[case::valid_with_underscore("hello_world", true)]
#[case::valid_with_hyphen("hello-world", true)]
#[case::valid_with_numbers("hello123", true)]
#[case::invalid_empty("", false)]
#[case::invalid_starts_with_number("123hello", false)]
#[case::invalid_contains_space("hello world", false)]
#[case::invalid_special_chars("hello@world", false)]
fn identifier_validation(
    #[case] input: &str,
    #[case] expected_valid: bool
) {
    let result = Identifier::new(input);
    assert_eq!(
        result.is_ok(),
        expected_valid,
        "Expected {} for input '{}' but got {:?}",
        if expected_valid { "Ok" } else { "Err" },
        input,
        result
    );
}
```

### Fixtures

```rust
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
fn validates_note(test_note: Note) {
    assert!(test_note.validate().is_ok());
}

#[rstest]
fn extracts_note_id(test_note: Note) {
    assert_eq!(
        test_note.id.to_string(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
}
```

### Fixtures with Defaults

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
use std::sync::Arc;

#[fixture]
#[once]
fn shared_database() -> Arc<Database> {
    // Expensive setup - runs once
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
    let result = shared_database.query("key").unwrap();
    assert!(result.is_some());
}
```

### Combinatorial Testing with #[values]

```rust
#[rstest]
fn state_transitions(
    #[values(State::Init, State::Ready, State::Processing)] current: State,
    #[values(Event::Start, Event::Stop, Event::Reset)] event: Event,
) {
    let result = current.handle(event);
    // Generates 3 × 3 = 9 test cases
    assert!(result.is_ok());
}
```

### Async Fixtures

```rust
#[fixture]
async fn async_database() -> Database {
    Database::connect("test://localhost").await.unwrap()
}

#[rstest]
#[tokio::test]
async fn test_async_insert(#[future] async_database: Database) {
    let db = async_database.await;
    db.insert("key", "value").await.unwrap();
}
```

### Template Reuse (rstest_reuse)

```rust
use rstest::rstest;
use rstest_reuse::{self, *};

#[template]
#[rstest]
#[case(2, 2, 4)]
#[case(0, 5, 5)]
#[case(-1, 1, 0)]
fn addition_cases(
    #[case] a: i32,
    #[case] b: i32,
    #[case] expected: i32
) {}

#[apply(addition_cases)]
fn test_add(a: i32, b: i32, expected: i32) {
    assert_eq!(add(a, b), expected);
}

#[apply(addition_cases)]
fn test_wrapping_add(a: i32, b: i32, expected: i32) {
    assert_eq!(wrapping_add(a, b), expected);
}
```

## QUICK REFERENCE

| Pattern             | Syntax                           |
| ------------------- | -------------------------------- |
| Basic case          | `#[case::name(args...)]`         |
| Fixture             | `#[fixture] fn name() -> T`      |
| Fixture with params | `#[default(value)] param: Type`  |
| Once fixture        | `#[fixture] #[once]`             |
| Async fixture       | `#[future]` + `.await`           |
| Values              | `#[values(a, b, c)] param: Type` |
| Template            | `#[template] #[rstest]`          |
| Apply template      | `#[apply(template_name)]`        |

## WHEN TO USE

| Scenario               | Use rstest? | Pattern              |
| ---------------------- | ----------- | -------------------- |
| Single test            | No          | `#[test]`            |
| Multiple similar tests | Yes         | `#[case::name(...)]` |
| Shared setup           | Yes         | `#[fixture]`         |
| Expensive shared setup | Yes         | `#[once]`            |
| Combinatorial testing  | Yes         | `#[values(...)]`     |
| Async setup            | Yes         | `#[future]`          |
| Template reuse         | Yes         | `rstest_reuse`       |

## RELATED MODULES

- See `testing-fixtures.md` for fixture strategies
- See `testing-unit.md` for unit testing patterns
- See `testing-anti-patterns.md` for comprehensive anti-patterns
