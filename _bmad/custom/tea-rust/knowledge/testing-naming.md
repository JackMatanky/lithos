# TEA Knowledge: Test Naming Conventions

## CONTEXT
- **Applies to**: All test functions (`#[test] fn ...`)
- **Purpose**: Clear, descriptive names that explain behavior
- **Pattern**: `action_expected_condition` (Verb-First)

## THE NAMING FORMULA

```
[unit_of_work]_[expected_behavior]_[state_under_test]
```

| Component | Description | Examples |
|-----------|-------------|----------|
| **Action** (Verb) | What is being tested | `returns`, `rejects`, `validates`, `parses`, `calculates` |
| **Expected** | The expected outcome | `error`, `ok`, `true`, `false`, `value` |
| **Condition** | The state/circumstance | `when_invalid`, `with_empty_input`, `for_large_values` |

## DECISION TREE: Test Name Structure

```
Is the test...
├── Testing a specific function?
│   └── YES → Use function name as submodule: mod validate { fn rejects_empty_input() }
│
├── Testing a command (write operation)?
│   └── YES → Use action verb: `creates_`, `updates_`, `deletes_`
│
├── Testing a query (read operation)?
│   └── YES → Use query verb: `finds_`, `returns_`, `loads_`
│
├── Testing validation?
│   └── YES → Use: `rejects_invalid_`, `accepts_valid_`
│
└── Testing state transitions?
    └── YES → Use: `transitions_from_X_to_Y_when_Z`
```

## MODULE ORGANIZATION

### Decision Tree for Submodules

```
What is being tested?
├── Shared setup only?
│   └── → `fixtures` module
│
├── Property-based tests?
│   └── → `proptests` module
│
├── A command (write path)?
│   └── → Command module: `create`, `update`, `delete`
│
├── A query (read path)?
│   └── → Query module: `find_by_id`, `list`, `search`
│
├── Constructor/Builder?
│   └── → `constructor`, `builder` module
│
├── Validation?
│   └── → `validation` module
│
├── Conversions?
│   └── → `conversions` module
│
└── Specific function?
    └── → Module named after function: `process_note`, `validate_schema`
```

## VALIDATION CHECKLIST

### Test Function Names
- [ ] Follows `action_expected_condition` pattern
- [ ] Does NOT start with `test_` (redundant)
- [ ] Does NOT use generic names (`test_foo`, `test_basic`, `test_1`)
- [ ] Does NOT combine multiple behaviors with "and"
- [ ] Uses lowercase with underscores (snake_case)

### Test Module Names
- [ ] Uses singular form: `constructor` (not `constructors`)
- [ ] Uses standard module names from the canonical list
- [ ] Groups related tests logically
- [ ] Does NOT mix concerns in one module

### Submodule Names (Per Function)
- [ ] Named after the function being tested
- [ ] Improves IDE navigation
- [ ] Provides structured test output

## ANTI-PATTERNS (FLAG THESE)

### Critical Issues
- ❌ `#[test] fn test_foo()` → Use descriptive behavior name
- ❌ `#[test] fn test()` → No description at all
- ❌ `#[test] fn test_basic_functionality()` → Too vague
- ❌ `#[test] fn test_returns_ok_and_updates_state()` → Multiple behaviors (use "and")

### Naming Conventions
- ❌ `testValidInput` → Must use snake_case
- ❌ `TestValidInput` → Must be lowercase
- ❌ `VALID_INPUT_TEST` → No ALL_CAPS
- ❌ `test_1`, `test_2` → Numbered tests

### Redundancy
- ❌ `test_validate_function()` → "test" prefix redundant
- ❌ `should_test_parsing()` → "test" in name redundant
- ❌ `unit_test_for_parser()` → "unit_test" redundant

### Module Issues
- ❌ `mod tests_for_validation()` → Use `mod validation`
- ❌ `mod test_constructors()` → Use `mod constructor`
- ❌ Module mixing unit and property tests → Separate them

## CORRECT EXAMPLES

### Test Function Names
```rust
// ✅ GOOD: Clear what is being tested
#[test]
fn returns_error_when_vault_path_is_invalid() { }

#[test]
fn rejects_empty_string_as_input() { }

#[test]
fn parses_valid_markdown_frontmatter() { }

#[test]
fn maintains_event_bus_api_contract_across_boundaries() { }

#[test]
fn calculates_sum_correctly_for_positive_integers() { }

// ❌ BAD: Non-descriptive or redundant
#[test]
fn test_note_1() { }           // Too vague
#[test]
fn test_empty() { }            // Missing context
#[test]
fn test_validation() { }       // What validation?
#[test]
fn returns_ok_and_updates_db() { }  // Two behaviors
```

### Module Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ✅ Submodule per function
    mod process_note {
        use super::*;

        #[test]
        fn returns_blob_when_larger_than_limit() { }

        #[test]
        fn fails_when_frontmatter_is_malformed() { }
    }

    mod validate {
        use super::*;

        #[test]
        fn accepts_valid_property_names() { }

        #[test]
        fn rejects_names_starting_with_numbers() { }
    }

    // ✅ Core structure modules
    mod constructor {
        use super::*;

        #[test]
        fn creates_instance_with_valid_input() { }

        #[test]
        fn returns_error_for_invalid_path() { }
    }

    mod validation {
        use super::*;

        #[test]
        fn fails_when_required_field_is_missing() { }
    }
}
```

### Command Modules
```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod create {
        use super::*;

        #[test]
        fn persists_valid_schema_to_storage() { }

        #[test]
        fn emits_created_event() { }
    }

    mod update {
        use super::*;

        #[test]
        fn modifies_existing_schema() { }

        #[test]
        fn returns_error_when_schema_not_found() { }
    }

    mod delete {
        use super::*;

        #[test]
        fn removes_schema_from_storage() { }
    }
}
```

### Query Modules
```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod find_by_id {
        use super::*;

        #[test]
        fn returns_note_when_exists() { }

        #[test]
        fn returns_none_when_not_found() { }
    }

    mod list_all {
        use super::*;

        #[test]
        fn returns_empty_vec_when_no_notes() { }

        #[test]
        fn returns_all_notes_ordered_by_date() { }
    }

    mod search {
        use super::*;

        #[test]
        fn finds_notes_matching_query() { }
    }
}
```

## CANONICAL MODULE NAMES

### Core Structure
| Module Name | Use When Testing |
|-------------|------------------|
| `constructor` | `new`, `try_new`, constructors |
| `builder` | Builder APIs, fluent construction |
| `defaults` | `Default` impls and baseline config |
| `validation` | Field/rule validation failures/success |
| `invariants` | Cross-field consistency rules |
| `integrity` | Structural consistency checks |
| `state` | State transitions, lifecycle flags |
| `accessors` | Getters, derived values |
| `conversions` | `From`/`TryFrom`/`Into` |
| `borrowing` | Zero-copy/borrowed accessors, guards |
| `formatting` | `Display`/`Debug` output |
| `equality` | `Eq`/`PartialEq` expectations |
| `ordering` | `Ord`/`PartialOrd` behavior |
| `hashing` | `Hash` behavior as map/set key |
| `cloning` | `Clone` behavior |

### Commands
| Module Name | Use When Testing |
|-------------|------------------|
| `create` | Create command behavior |
| `update` | Update command behavior |
| `delete` | Delete command behavior |
| `upsert` | Insert/update semantics |
| `rename` | Rename/retitle flows |
| `link` | Link/relationship creation |
| `unlink` | Link/relationship removal |
| `assign` | Ownership/association addition |
| `unassign` | Ownership/association removal |
| `merge` | Merge command semantics |
| `event_emission` | Events emitted by commands |
| `persistence` | DB effects specific to commands |

### Queries
| Module Name | Use When Testing |
|-------------|------------------|
| `find_by_id` | Lookup by id |
| `find_by_name` | Lookup by name |
| `find_by_path` | Lookup by path |
| `find_by_tag` | Lookup by tag |
| `load` | Load/aggregate query results |
| `list` | List subset/default list |
| `list_all` | List everything |
| `list_by_parent` | List by parent/owner |
| `search` | General search |
| `search_text` | Free-text search |
| `resolve` | Derived/linked results |
| `indices` | Index-driven lookup behavior |
| `pagination` | Limits/offsets/cursors |

### Special
| Module Name | Use When Testing |
|-------------|------------------|
| `fixtures` | Shared setup helpers only |
| `proptests` | All proptest suites |

## QUICK REFERENCE

| Aspect | Pattern | Example |
|--------|---------|---------|
| Test name | `action_expected_condition` | `returns_error_when_invalid` |
| Module name | Singular, descriptive | `validation`, `constructor` |
| Submodule | Function name | `mod process_note` |
| Command | Action verb | `mod create`, `mod update` |
| Query | Query verb | `mod find_by_id`, `mod search` |

## RELATED MODULES
- See `testing-unit.md` for unit testing location rules
- See `testing-assertions.md` for assertion patterns
- See `testing-anti-patterns.md` for comprehensive anti-patterns
