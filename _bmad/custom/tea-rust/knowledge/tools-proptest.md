# TEA Knowledge: Property-Based Testing (proptest)

## CONTEXT

- **Tool**: `proptest` - Property-based testing for Rust
- **Purpose**: Edge case discovery, invariant verification
- **Best for**: Mathematical properties, parsers, state machines

## DECISION TREE: When to Use proptest

```
Are you testing...
├── Mathematical invariants (round-trips, associativity)?
│   └── YES → Use proptest
│
├── Parser/tokenizer with many inputs?
│   └── YES → Use proptest
│
├── State machine transitions?
│   └── YES → Use proptest with state machine testing
│
├── Input validation edge cases?
│   └── YES → Use proptest with regex strategies
│
├── Graph/data structure consistency?
│   └── YES → Use proptest
│
├── Business logic with specific examples?
│   └── NO → Use example-based tests
│
└── Simple equality assertions?
    └── NO → Use standard assertions
```

## VALIDATION CHECKLIST

### Strategy Design

- [ ] Uses appropriate strategy for data type
- [ ] Regex strategies for string patterns
- [ ] `prop_compose!` for complex strategies
- [ ] `prop_filter` for valid ranges

### Determinism

- [ ] Uses deterministic seeds (default or explicit)
- [ ] Uses `.prop_with_config()` for config
- [ ] Tests are reproducible

### Assertions

- [ ] Uses `prop_assert!` family (not `assert!`)
- [ ] Error messages include property being tested
- [ ] Shrinking will produce minimal failing case

### Test Organization

- [ ] Property tests in `proptests` submodule
- [ ] Mixed with example-based tests (not replacing)
- [ ] Clear property description in test name

## ANTI-PATTERNS (FLAG THESE)

### Strategy Issues

- ❌ **No filtering** → Generates invalid inputs
- ❌ **Too broad strategies** → Wastes time on irrelevant cases
- ❌ **Complex strategies without `prop_compose!`** → Hard to read
- ❌ **Non-deterministic seeds** → Unreproducible failures

### Assertion Issues

- ❌ **Using `assert!` instead of `prop_assert!`** → Poor error messages
- ❌ **No property description** → What invariant is being tested?
- ❌ **Testing specific examples** → Use `#[test]` for examples

### Organization Issues

- ❌ **Only property tests** → Combine with example-based tests
- ❌ **Properties testing implementation** → Test behavior/invariants
- ❌ **Too many cases** → Slows test suite (proptest has default limits)

## CORRECT EXAMPLES

### Basic Property Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn reverse_reverse_is_identity(s: String) {
        let reversed: String = s.chars().rev().collect();
        let double_reversed: String = reversed.chars().rev().collect();
        prop_assert_eq!(s, double_reversed);
    }
}
```

### With Regex Strategy

```rust
proptest! {
    #[test]
    fn validates_identifier_format(
        name in "[a-zA-Z][a-zA-Z0-9_-]{0,63}"
    ) {
        let result = Identifier::new(name);
        prop_assert!(
            result.is_ok(),
            "Valid identifier '{}' should be accepted",
            name
        );
    }

    #[test]
    fn rejects_invalid_characters(
        name in ".*[^a-zA-Z0-9_-].*".prop_filter(
            "valid length",
            |s: &String| !s.is_empty() && s.len() <= 64
        )
    ) {
        let result = Identifier::new(name);
        prop_assert!(
            result.is_err(),
            "Invalid identifier '{}' should be rejected",
            name
        );
    }
}
```

### Composite Strategy with prop_compose

```rust
use proptest::prelude::*;

// Strategy for valid schema names
fn schema_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,63}"
}

// Strategy for note paths
fn note_path_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-/]+\\.md"
}

// Composite strategy for complete notes
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
    fn note_validation_succeeds_for_generated_notes(note in note_strategy()) {
        prop_assert!(
            note.validate().is_ok(),
            "Generated note should be valid: {:?}",
            note
        );
    }
}
```

### Round-trip Serialization

```rust
proptest! {
    #[test]
    fn rkyv_roundtrip_preserves_data(data: Vec<u8>) {
        // Serialize
        let serialized = rkyv::to_bytes::<_, 256>(&data).unwrap();

        // Deserialize
        let deserialized: Vec<u8> = rkyv::from_bytes(&serialized).unwrap();

        // Property: round-trip preserves data
        prop_assert_eq!(
            data, deserialized,
            "Round-trip serialization failed for data: {:?}",
            data
        );
    }
}
```

### State Machine Testing

```rust
#[derive(Debug, Clone)]
enum VaultOperation {
    CreateNote(String),
    DeleteNote(Uuid),
    UpdateNote(Uuid, String),
}

fn operation_strategy() -> impl Strategy<Value = VaultOperation> {
    prop_oneof![
        "[a-zA-Z0-9_-/]+\\.md".prop_map(VaultOperation::CreateNote),
        any::<Uuid>().prop_map(VaultOperation::DeleteNote),
        (any::<Uuid>(), ".*").prop_map(|(id, content)| {
            VaultOperation::UpdateNote(id, content)
        }),
    ]
}

proptest! {
    #[test]
    fn vault_operations_maintain_consistency(
        ops in prop::collection::vec(operation_strategy(), 1..100)
    ) {
        let mut vault = Vault::empty();

        for op in ops {
            vault.apply(op.clone()).unwrap();

            // Property: vault is always consistent after each operation
            prop_assert!(
                vault.check_invariants().is_ok(),
                "Vault inconsistency after operation {:?}",
                op
            );
        }
    }
}
```

### With Deterministic Seed

```rust
use proptest::test_runner::Config;

proptest! {
    #![proptest_config(Config::with_cases(1000))]

    #[test]
    fn with_custom_config(s: String) {
        // Runs 1000 cases instead of default 256
        prop_assert!(!s.is_empty() || s.is_empty()); // Always true
    }
}
```

### Mixed with Example-Based Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Example-based tests for clarity
    #[test]
    fn rejects_empty_string() {
        assert!(Identifier::new("").is_err());
    }

    #[test]
    fn accepts_simple_name() {
        assert!(Identifier::new("valid").is_ok());
    }

    // Property-based tests for edge case discovery
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn accepts_valid_identifiers(name in "[a-zA-Z][a-zA-Z0-9_-]*") {
                prop_assert!(Identifier::new(&name).is_ok());
            }
        }
    }
}
```

## QUICK REFERENCE

| Pattern        | Syntax                                             |
| -------------- | -------------------------------------------------- |
| Basic test     | `proptest! { #[test] fn name(s: String) { ... } }` |
| Regex strategy | `"[a-z]+"`                                         |
| Filter         | `strategy.prop_filter("name", \|s\| condition)`    |
| Compose        | `prop_compose! { fn name()(...)`                   |
| Any value      | `any::<Type>()`                                    |
| Collection     | `prop::collection::vec(strategy, 1..100)`          |
| One of         | `prop_oneof![a, b, c]`                             |
| Assertion      | `prop_assert!(condition, "msg")`                   |

## CONFIGURATION

```rust
use proptest::test_runner::Config;

proptest! {
    #![proptest_config(Config {
        cases: 1000,           // Number of test cases
        max_shrink_iters: 50,  // Max shrinking iterations
        ..Config::default()
    })]

    #[test]
    fn my_test(s: String) {
        prop_assert!(...);
    }
}
```

## WHEN TO USE

| Scenario                 | Use proptest?      |
| ------------------------ | ------------------ |
| Round-trip serialization | Yes                |
| Parser/tokenizer         | Yes                |
| Input validation         | Yes                |
| State machines           | Yes                |
| Graph consistency        | Yes                |
| Mathematical properties  | Yes                |
| Business logic examples  | No (use `#[test]`) |
| Simple assertions        | No (use `#[test]`) |

## RELATED MODULES

- See `fixtures.md` for fixture strategies
- See `unit.md` for unit testing patterns
- See `anti-patterns.md` for comprehensive anti-patterns
