# TEA Knowledge: Test Assertion Patterns

## CONTEXT
- **Applies to**: All test assertions (`assert!`, `assert_eq!`, etc.)
- **Purpose**: Clear, explicit verification with helpful error messages
- **Goal**: Make test failures immediately diagnosable

## DECISION TREE: Which Assertion to Use?

```
What are you asserting?
├── Boolean condition?
│   └── → assert!(condition, "message with context")
│
├── Equality of values?
│   ├── Complex structs? → assert_eq!(actual, expected) with pretty_assertions
│   └── Simple values? → assert_eq!(actual, expected, "message")
│
├── Inequality?
│   └── → assert_ne!(actual, unexpected, "message")
│
├── Enum variant (not content)?
│   └── → assert!(matches!(value, Variant(_)))
│
├── Result is Ok?
│   └── → assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err())
│
├── Result is Err?
│   └── → assert!(result.is_err(), "Expected Err, got: {:?}", result.unwrap())
│
├── Specific error type?
│   └── → assert!(matches!(result, Err(Error::Specific(_))))
│
├── Option is Some?
│   └── → assert!(option.is_some(), "message")
│
├── Option is None?
│   └── → assert!(option.is_none(), "message")
│
└── Panic expected?
    └── → #[should_panic(expected = "...")] (rarely)
```

## VALIDATION CHECKLIST

### Error Messages
- [ ] All `assert!` calls include a descriptive message
- [ ] Error messages include the actual value that failed
- [ ] Error messages explain what was expected
- [ ] Uses `{:?}` debug formatting for complex types

### Assertion Types
- [ ] Uses `matches!` for enum variant checking (not equality)
- [ ] Uses `assert!(result.is_ok(), "...", result.err())` NOT `result.unwrap()`
- [ ] Uses `pretty_assertions` for struct comparisons
- [ ] Avoids `#[should_panic]` unless panic is documented behavior

### Assertion Phases
- [ ] **Arrange**: Setup can use `unwrap()` (test prerequisites)
- [ ] **Act**: NO `unwrap()` - capture the result
- [ ] **Assert**: NO `unwrap()` - use explicit assertions

## ANTI-PATTERNS (FLAG THESE)

### Critical Issues
- ❌ `result.unwrap()` in assertions → Hides error information
- ❌ `result.expect("...")` in assertions → Same issue
- ❌ `assert!(result.is_ok())` without error message → No context on failure
- ❌ `assert!(result.is_err())` without error message → Can't see what error occurred

### Message Issues
- ❌ `assert!(x > 0)` → No message at all
- ❌ `assert!(x > 0, "failed")` → Unhelpful message
- ❌ `assert!(x > 0, "x failed")` → Missing actual value
- ❌ Generic messages without context → "Expected success, got error"

### Type Issues
- ❌ `assert_eq!(result, Ok(expected))` on enums → Use `matches!`
- ❌ `assert!(error == Error::Variant)` → Use `matches!` for variants
- ❌ Complex equality checks without pretty_assertions → Hard to read diffs

### Hidden Assertions
- ❌ Assertions in helper functions → Keep assertions visible in test body
- ❌ Multiple assertions testing different behaviors → Split into separate tests

## CORRECT EXAMPLES

### Basic Assertions with Messages
```rust
// ❌ BAD: No error message
assert!(result.is_ok());

// ✅ GOOD: Includes actual error in message
assert!(
    result.is_ok(),
    "Expected successful validation, but got error: {:?}",
    result.err()
);

// ❌ BAD: No context
assert_eq!(count, 5);

// ✅ GOOD: Explains what was expected
assert_eq!(
    count, 5,
    "Expected 5 notes in vault, but found {}",
    count
);
```

### Result Assertions
```rust
// ❌ BAD: unwrap() hides the error
let value = result.unwrap();
assert_eq!(value, expected);

// ✅ GOOD: Shows error on failure
assert!(
    result.is_ok(),
    "Processing should succeed, but failed with: {:?}",
    result.err()
);
let value = result.unwrap(); // Now safe to unwrap after assertion

// ✅ GOOD: Check specific error variant
let result = validate_input("");
assert!(
    matches!(result, Err(ValidationError::EmptyInput)),
    "Expected EmptyInput error, but got: {:?}",
    result
);
```

### Enum Variant Checking
```rust
// ❌ BAD: Checking full equality when only variant matters
assert_eq!(error, DomainError::Validation("field".to_string()));

// ✅ GOOD: Only check variant
assert!(
    matches!(error, DomainError::Validation(_)),
    "Expected Validation error, found: {:?}",
    error
);

// ✅ GOOD: Check variant with pattern matching
assert!(
    matches!(result, Err(DomainError::NotFound { id, .. }) if id == expected_id),
    "Expected NotFound error for id {}, got: {:?}",
    expected_id, result
);
```

### With pretty_assertions
```rust
use pretty_assertions::assert_eq;

#[test]
fn complex_struct_comparison() {
    let actual = generate_complex_struct();
    let expected = ComplexStruct {
        field1: "value1".to_owned(),
        field2: vec![1, 2, 3],
        nested: NestedStruct { ... },
    };

    // Shows colorful, side-by-side diff on failure
    assert_eq!(actual, expected);
}
```

### Property Test Assertions
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_serialization(data in any::<Vec<u8>>()) {
        let serialized = serialize(&data);
        let deserialized = deserialize(&serialized);

        // prop_assert! shows the failing input
        prop_assert_eq!(
            data, deserialized,
            "Serialization roundtrip failed for input: {:?}",
            data
        );
    }
}
```

### Custom Assertion Helpers
```rust
/// Assert that a result is a validation error for a specific field
#[track_caller]
fn assert_validation_error(
    result: Result<Note, NoteError>,
    expected_field: &str
) {
    match result {
        Err(NoteError::Validation { field, message }) => {
            assert_eq!(
                field, expected_field,
                "Expected validation error for field '{}', but got error for field '{}'",
                expected_field, field
            );
        }
        Ok(_) => panic!(
            "Expected validation error for field '{}', but got Ok",
            expected_field
        ),
        Err(other) => panic!(
            "Expected Validation error for field '{}', got {:?}",
            expected_field, other
        ),
    }
}

// Usage
#[test]
fn test_validation() {
    assert_validation_error(
        Note::new("", "content"),
        "path"
    );
}
```

### Option Assertions
```rust
// ✅ Check Some with context
assert!(
    option.is_some(),
    "Expected note to be found for id {}, but got None",
    note_id
);

// ✅ Check None with context
assert!(
    option.is_none(),
    "Expected no note for invalid id {}, but found {:?}",
    invalid_id, option
);

// ✅ Unwrap after assertion is safe
assert!(option.is_some(), "Note should exist");
let note = option.unwrap(); // Safe now
```

## ASSERTION PHRASES TEMPLATE

### Arrange Phase (unwrap OK)
```rust
#[test]
fn example() {
    // Arrange - unwrap permitted here
    let fixture = create_fixture().unwrap();
    let input = load_test_data().expect("Test data exists");
```

### Act Phase (NO unwrap)
```rust
    // Act - capture result, do NOT unwrap
    let result = process(input);
```

### Assert Phase (explicit assertions)
```rust
    // Assert - use explicit assertions
    assert!(
        result.is_ok(),
        "Processing failed with error: {:?}",
        result.err()
    );

    let value = result.unwrap(); // Safe after assertion
    assert_eq!(
        value.count, 5,
        "Expected count of 5, got {}",
        value.count
    );
}
```

## COMMON ASSERTION PATTERNS

| Scenario | Pattern |
|----------|---------|
| Result is Ok | `assert!(result.is_ok(), "msg: {:?}", result.err())` |
| Result is Err | `assert!(result.is_err(), "msg: {:?}", result.unwrap())` |
| Specific error | `assert!(matches!(result, Err(Error::Variant(_))))` |
| Option is Some | `assert!(opt.is_some(), "msg")` |
| Option is None | `assert!(opt.is_none(), "msg")` |
| Enum variant | `assert!(matches!(val, Variant(_)))` |
| Equality | `assert_eq!(actual, expected, "msg")` |
| Inequality | `assert_ne!(actual, unexpected, "msg")` |
| Boolean | `assert!(condition, "msg with {:?}", value)` |
| Contains | `assert!(vec.contains(&item), "msg")` |

## RELATED MODULES
- See `testing-unit.md` for test structure
- See `testing-naming.md` for naming conventions
- See `testing-anti-patterns.md` for comprehensive anti-patterns
