---
title: "Unit Test Naming"
status: "active"
owner: "engineering"
last_updated: "2026-05-20"
scope: "Naming conventions and module organization for Rust unit tests"
---

# Unit Test Naming

## Context

- **Applies to**: all unit test functions (`#[test] fn ...`) in `lithos-core/src/**/*.rs`
- **Purpose**: clear, predictable names and module structure that make failures understandable in `nextest`
- **Core rule**: one behavior per test, one concern per module

## Quick Start (30 seconds)

1. Pick a structure:
   - Multi-unit or complex file -> **Structure A (default)**
   - Small/simple file with only 1-2 behaviors -> **Structure B**
2. Pick module name from **Canonical Module Names** (first exact match wins).
3. Name test function using the formula.
4. Use `returns_*` / `rejects_*` / `accepts_*` verb-first naming for new tests.
5. Run the checklist at the end.

## Default Convention (use this unless a rule below says otherwise)

- Use **Structure A** with submodules.
- Use module names from the canonical tables.
- Use verb-first function names (`returns_*`, `rejects_*`, `accepts_*`, `parses_*`).
- Keep one behavior per test and one concern per module.

If you are uncertain, do this:

1. Create a submodule named after the unit of work.
2. Write function names as `action_expected_condition`.
3. Prefer `lookup` for retrieval behavior and `validation` for rule checks.

## The Naming Formula

Depending on file complexity, use one of two shapes.

### Structure A: With Submodules (Preferred)

Use for files with multiple functions or multiple concern groups.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod lookup {
        use super::*;

        #[test]
        fn returns_none_when_record_is_missing() {}
    }
}
```

Formula:

- `mod [unit_of_work] { fn [action]_[expected]_[condition]() }`
- Legacy style accepted: `fn should_[action]_[expected]_[condition]()`

Combined reading:

- `lookup::returns_none_when_record_is_missing()`

### Structure B: Without Submodules (Simple Files)

Use only for small files where submodules add noise.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_none_when_record_is_missing() {}
}
```

Formula:

- `fn [unit_of_work]_[action]_[expected]_[condition]()`
- Legacy style accepted: `fn [unit_of_work]_should_[action]_[expected]_[condition]()`

Combined reading:

- `lookup_returns_none_when_record_is_missing()`

## Formula Components

| Component | Description | Examples |
| --- | --- | --- |
| Unit of Work | Method, struct, or concept being tested | `save`, `lookup`, `parse`, `validation` |
| Action (Verb) | What the code actively does | `returns`, `rejects`, `persists`, `emits` |
| Expected | The outcome or state | `error`, `ok`, `none`, `record`, `true` |
| Condition | Triggering circumstance | `when_missing`, `with_empty_input`, `if_locked` |

## Decision Tree: Choose Structure

1. Testing several independent units in one file?
   - Yes -> **Structure A**.
2. File already uses submodules?
   - Yes -> keep submodules and align naming inside them.
3. Only one or two simple behaviors in a small file?
   - Yes -> **Structure B** is acceptable.
4. Unsure?
   - Default to **Structure A** for better failure scanability.

Hard rule:

- If the file has 3+ tests or 2+ units of work, use **Structure A**.

## Module Name Selection Matrix

Use the first row that matches your test intent.

| If the tests are about... | Use this module name | Notes |
| --- | --- | --- |
| Shared setup only | `fixtures` | No assertions in this module |
| Property-based behavior only | `proptests` | Keep generators and `proptest!` here |
| Constructors and creation gates | `constructor` | Use `builder` only for builder APIs |
| Default values | `defaults` | For `Default` baseline semantics |
| Rule checks and rejection/acceptance | `validation` | Use `invariants` for cross-field always-true rules |
| Structural consistency | `integrity` | Graph/link/schema consistency |
| State transitions | `state` | Lifecycle/transition behavior |
| Read access and derived reads | `accessors` | In-memory read behavior |
| Borrowing/zero-copy views | `borrowing` | Lifetime and borrow contracts |
| Retrieval by key/path/name | `lookup` | Prefer over many `find_by_*` variants |
| Criteria matching/search | `search` | Free-form or multi-field matching |
| Subset narrowing | `filter` | Predicate-driven subset behavior |
| Windowing (limit/offset/cursor) | `pagination` | Paging mechanics |
| Collection retrieval | `list` | Default or ordered lists |
| Create/update/delete/upsert writes | `create` / `update` / `delete` / `upsert` | Pick exact operation |
| Parse input into structure | `parse` | For syntax/grammar interpretation |
| Structure to encoded output | `serialization` | Prefer over ambiguous `encoding` |
| Encoded input to structure | `deserialization` | Decoder behavior |
| Canonicalization/sanitization | `normalization` | Rewriting/cleanup behavior |
| Index build/query maintenance | `indexing` | Index data path behavior |
| Cache hit/miss/eviction | `caching` | Cache semantics |
| Atomic commit/rollback | `transactions` | Write atomicity behavior |
| Locking/concurrency control | `locking` | Acquire/release/contention behavior |
| Type conversion contracts | `conversions` | `From`/`TryFrom`/`Into` |
| Text rendering contracts | `formatting` | `Display`/`Debug` |
| Equality/ordering/hash/clone contracts | `equality` / `ordering` / `hashing` / `cloning` | Trait behavior |
| No canonical name fits precisely | exact unit/function name | Example: `parse_frontmatter`, `process_note` |

Hard rules:

- Do not invent a new module name if a canonical name matches.
- If two names fit, choose the more behavior-specific one.
- Split modules instead of mixing concerns.

## Anti-Patterns (Flag These)

### Naming Problems

- `test_foo`, `test_basic`, `test_1`, `it_works`
- `returns_ok_and_updates_state` (multiple behaviors)
- `testValidInput`, `TestValidInput`, `VALID_INPUT_TEST`
- `test_validate_...`, `unit_test_for_...` (redundant prefixes)
- `misc`, `other`, `general`, `helpers` as behavior modules

### Module Problems

- `mod tests_for_validation` instead of `mod validation`
- Mixing fixtures, assertions, and proptests in one module
- Deeply nested modules without a clear separation benefit

## Canonical Module Names

Use these names as the default vocabulary for new tests.

### Core Structure

| Module name | Use when |
| --- | --- |
| `constructor` | `new`, `try_new`, canonical constructors |
| `builder` | builder APIs and fluent configuration |
| `defaults` | `Default` impl and baseline values |
| `validation` | field/rule acceptance and rejection |
| `invariants` | cross-field/domain invariants |
| `integrity` | structural consistency checks |
| `state` | state transitions and lifecycle behavior |
| `accessors` | getters and derived values |
| `borrowing` | zero-copy or borrowed view behavior |
| `conversions` | `From`/`TryFrom`/`Into` behavior |
| `formatting` | `Display`/`Debug` rendering |
| `equality` | `Eq`/`PartialEq` behavior |
| `ordering` | `Ord`/`PartialOrd` behavior |
| `hashing` | `Hash` behavior for keys/maps/sets |
| `cloning` | `Clone` behavior |

### Operation-Oriented (Cross-Domain)

| Module name | Use when |
| --- | --- |
| `lookup` | keyed retrieval (`id`, `path`, `name`, handle) |
| `search` | criteria-based retrieval or matching |
| `filter` | subset selection by predicate |
| `pagination` | limit/offset/cursor behavior |
| `list` | collection retrieval behavior |
| `create` | create/persist-new behavior |
| `update` | mutation behavior of existing entities |
| `delete` | removal behavior and constraints |
| `upsert` | insert-or-update behavior |
| `parse` | input-to-structure parsing |
| `serialization` | structure-to-wire/encoded output |
| `deserialization` | wire/encoded input-to-structure |
| `normalization` | canonicalization/sanitization behavior |
| `indexing` | index build/update/read behavior |
| `caching` | cache hit/miss/evict/fill behavior |
| `transactions` | atomicity/commit/rollback behavior |
| `locking` | lock/concurrency behavior |

### Infrastructure

| Module name | Use when |
| --- | --- |
| `fixtures` | shared setup helpers only (no assertions) |
| `proptests` | property-based suites only |

### Unit-Specific

When canonical names are too broad, use exact names:

- `process_note`
- `parse_frontmatter`

Notes:

- `find_by_*` module names are acceptable when matching a stable public API method family.
- Do not force command/query module pairs unless the code itself is modeled that way.

## Naming Migration Rule

- Existing `should_*` names do not need immediate renaming.
- New tests should use verb-first without `should_`.
- If you touch an existing test block significantly, normalize names in that local block for consistency.

## Validation Checklist

### Test Function Names

- [ ] Uses Structure A or B consistently for the file.
- [ ] Follows the formula for the chosen structure.
- [ ] Uses snake_case only.
- [ ] Does not start with `test_`.
- [ ] Does not combine multiple behaviors with `and`.
- [ ] Is descriptive enough to understand failure without opening test body.
- [ ] Uses verb-first naming (`returns_*`, `rejects_*`, `accepts_*`, `parses_*`) for new tests.

### Test Module Names

- [ ] Uses singular names (`constructor`, not `constructors`).
- [ ] Uses canonical module names when applicable.
- [ ] Keeps concerns separate (`validation` vs `fixtures` vs `proptests`).
- [ ] Improves scanability of test output.

## Correct Examples

### Function Names

```rust
#[test]
fn returns_error_when_vault_path_is_invalid() {}

#[test]
fn rejects_empty_string_as_input() {}

#[test]
fn parses_valid_markdown_frontmatter() {}

#[test]
fn test_note_1() {} // bad: too vague
```

### Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod process_note {
        use super::*;

        #[test]
        fn returns_blob_when_larger_than_limit() {}

        #[test]
        fn fails_when_frontmatter_is_malformed() {}
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_names_starting_with_numbers() {}
    }
}
```

## Quick Reference

| Aspect | Pattern | Example |
| --- | --- | --- |
| Structure A test | `[action]_[expected]_[condition]` | `returns_none_when_not_found` |
| Structure B test | `[unit]_[action]_[expected]_[condition]` | `lookup_returns_none_when_not_found` |
| Legacy style | `should_...` accepted for existing tests | `should_return_none_when_not_found` |
| Module name | singular + concern-focused | `validation`, `lookup`, `proptests` |

## References

- Rust Book: unit test structure and organization
  - https://doc.rust-lang.org/book/ch11-01-writing-tests.html
  - https://doc.rust-lang.org/book/ch11-03-test-organization.html
- Rust API Guidelines: examples and failure documentation
  - https://rust-lang.github.io/api-guidelines/documentation.html
