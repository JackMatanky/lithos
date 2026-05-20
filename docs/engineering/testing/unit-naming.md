---
title: "Unit Test Naming"
status: "active"
owner: "engineering"
last_updated: "2026-05-20"
scope: "Naming conventions and module organization for Rust unit tests"
---

# Unit Test Naming

## Context

- Make failing tests self-explanatory in `nextest` output.
- Keep naming and organization aligned with how this repository is written today.
- Provide one canonical reference for both test names and test module structure.
- Favor consistency over dogma: no CQRS-only naming model.

## Fast path

1. Choose structure:
   - Multi-unit/complex file: Structure A (submodules).
   - Small/simple file: Structure B (flat names).
2. Choose module names with the selection flowchart.
3. Name tests with the formula components.
4. Pick one style per module (`returns_*` or `should_*`) and stay consistent.
5. Run the validation checklist before finishing.

## The naming formula

Depending on file complexity, use one of two structures.

### Structure A: With submodules (preferred)

Use for files with multiple functions, multiple concern groups, or rich behavior.

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

Shape:

- `mod [unit_of_work] { fn [action]_[expected]_[condition]() }`
- `mod [unit_of_work] { fn [should_]?[action]_[expected]_[condition]() }`

Combined reading:

- `lookup::returns_none_when_record_is_missing()`

### Structure B: Without submodules (small/simple files)

Use only for small files or value objects where submodules add noise.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_none_when_record_is_missing() {}
}
```

Shape:

- `fn [unit_of_work]_[action]_[expected]_[condition]()`
- `fn [unit_of_work]_[should_]?[action]_[expected]_[condition]()`

Combined reading:

- `lookup_returns_none_when_record_is_missing()`

## Formula components

| Component | Description | Examples |
| --- | --- | --- |
| Unit of Work | Method, struct, or concept under test | `save`, `lookup`, `parse`, `validation` |
| Action (Verb) | What the code actively does | `returns`, `rejects`, `persists`, `emits` |
| Expected | Specific outcome/state | `error`, `ok`, `none`, `record`, `true` |
| Condition | Triggering state/circumstance | `when_missing`, `with_empty_input`, `if_locked` |

## Accepted styles in this codebase

Both styles are currently used and accepted:

- `returns_error_when_input_is_invalid`
- `should_return_error_when_input_is_invalid`

Rule: within a given module, pick one style and stay consistent.

## Decision tree: choose structure

1. Is the file testing several independent units of work?
   - Yes: use **Structure A** with submodules.
2. Are tests already organized in submodules in this file?
   - Yes: keep submodules and align naming inside them.
3. Is there only one small unit with one or two behaviors?
   - Yes: **Structure B** is acceptable.
4. Does adding submodules improve scanability in failures?
   - Yes: choose **Structure A**.

## Selection flowchart

Use this flow when choosing module names for a test suite.

1. Is this setup-only helper code?
   - Use `fixtures`.
2. Is this property-based testing only?
   - Use `proptests`.
3. Is the behavior tied to lifecycle/creation?
   - Start with `constructor`, `builder`, or `defaults`.
4. Is the behavior rule/invariant oriented?
   - Start with `validation`, `invariants`, or `integrity`.
5. Is the behavior domain operation oriented?
   - Choose from operation modules (lookup, parse, serialization, upsert, etc.).
6. Is this trait contract behavior?
   - Use `conversions`, `formatting`, `equality`, `ordering`, `hashing`, or `cloning`.
7. No canonical name fits cleanly?
   - Use the exact unit/function name (`parse_frontmatter`, `process_note`).

Tie-breakers:

- Prefer the most behavior-specific name over a generic one.
- If two names fit, choose the one that best predicts assertion type.
- Keep one concern per module; split instead of mixing.

## What to avoid

- Generic names: `test_foo`, `test_basic`, `it_works`, `test_1`.
- Mixed behaviors in one name: `returns_ok_and_updates_state`.
- Prefix noise: `test_validate...`, `unit_test_for_...`.
- Non-snake-case names: `testValidInput`, `TestValidInput`, `VALID_INPUT_TEST`.

## Module organization

### Rules

- Use singular, descriptive module names.
- Keep one concern per module.
- Keep `fixtures` for setup helpers only.
- Keep `proptests` for property-based suites only.
- Keep submodule depth shallow unless there is strong readability benefit.

### Standardized module naming convention

Use these names in this order of preference when selecting a module name.

#### 1) Lifecycle and construction

| Module name | Use when | Why this name |
| --- | --- | --- |
| `constructor` | Testing `new`, `try_new`, or canonical constructors | Signals object creation and entry invariants |
| `builder` | Testing builder APIs and fluent configuration | Distinguishes staged construction from direct constructors |
| `defaults` | Testing `Default` behavior and baseline values | Makes default-state expectations easy to find |

#### 2) Rules and correctness

| Module name | Use when | Why this name |
| --- | --- | --- |
| `validation` | Field/rule acceptance and rejection behavior | Most direct term for input/rule checks |
| `invariants` | Cross-field/domain invariants that must always hold | Highlights always-true business guarantees |
| `integrity` | Structural consistency or graph/link coherence checks | Emphasizes whole-structure correctness |

#### 3) Behavior and transitions

| Module name | Use when | Why this name |
| --- | --- | --- |
| `state` | State transitions and lifecycle behavior | Keeps transition semantics in one place |
| `accessors` | Getters and derived-read behavior | Separates read access from mutation paths |
| `borrowing` | Borrowed views, zero-copy access, lifetime-sensitive APIs | Makes ownership/borrowing contracts explicit |

#### 3a) Operation-oriented modules (cross-domain)

Use these for filesystem, database, parsing, indexing, and similar operation-heavy units.

| Module name | Use when | Why this name |
| --- | --- | --- |
| `lookup` | Keyed retrieval by id/path/name/handle | Broad read-operation name usable beyond DB |
| `search` | Query-like matching, fuzzy or multi-field retrieval | Distinguishes retrieval by criteria from direct lookup |
| `filter` | Subset selection from in-memory or query result sets | Signals predicate-based narrowing |
| `pagination` | Limit/offset/cursor windowing behavior | Standard name for page/window semantics |
| `upsert` | Insert-or-update merge behavior | Explicitly communicates dual write semantics |
| `create` | Pure create/persist-new behavior | Separates new-write behavior from updates |
| `update` | Mutating existing record/entity behavior | Separates change behavior from create/delete |
| `delete` | Removal behavior and delete constraints | Canonical destructive-operation label |
| `list` | Ordered or default collection retrieval | Clear name for collection-return operations |
| `parse` | Input-to-structured representation conversion | Canonical for syntax/format interpretation |
| `serialization` | Structured data to encoded/wire format | Distinguishes output encoding from parsing |
| `deserialization` | Encoded/wire format to structured data | Distinguishes input decoding from parsing rules |
| `normalization` | Canonicalization, sanitization, or rewrite rules | Signals shape/format cleanup behavior |
| `indexing` | Index build/update/read path behavior | Useful for search/catalog/index services |
| `caching` | Cache hit/miss/eviction/fill behavior | Keeps cache semantics grouped and testable |
| `transactions` | Atomicity, rollback, and commit semantics | Standard name for write atomicity behavior |
| `locking` | Concurrency/lock acquisition/release semantics | Makes synchronization behavior explicit |

Notes:

- Prefer `lookup` over many near-duplicates (`find_by_id`, `find_by_name`) unless method names themselves are `find_by_*` and module-per-function organization improves clarity.
- Prefer `serialization`/`deserialization` over ambiguous `encoding` unless protocol semantics are primary.
- Use `parse_*` function-specific modules when grammar/protocol has multiple independent parsers.

#### 4) Interop and representation

| Module name | Use when | Why this name |
| --- | --- | --- |
| `conversions` | `From`/`TryFrom`/`Into` transformations | Canonical name for type conversion behavior |
| `formatting` | `Display`/`Debug` or user-facing textual rendering | Centralizes presentation contracts |

#### 5) Trait contract behavior

| Module name | Use when | Why this name |
| --- | --- | --- |
| `equality` | `Eq`/`PartialEq` semantics | Clear signal for equality rules |
| `ordering` | `Ord`/`PartialOrd` behavior | Clear signal for ordering comparisons |
| `hashing` | `Hash` behavior and key stability assumptions | Groups map/set-key expectations |
| `cloning` | `Clone` behavior and copy semantics | Makes duplication behavior explicit |

#### 6) Test infrastructure modules

| Module name | Use when | Why this name |
| --- | --- | --- |
| `fixtures` | Shared setup helpers only (no assertions) | Prevents setup logic from mixing with behavior tests |
| `proptests` | Property-based tests only | Keeps generative/invariant suites discoverable |

#### 7) Unit-specific module names

When canonical names are too broad, use the exact unit/function name:

- `process_note`
- `parse_frontmatter`

Use unit-specific names when they are more precise than generic categories.

Do not force command/query module pairs unless the code itself is modeled that way.

## Validation checklist

### Test function names

- [ ] Uses Structure A or B consistently for the file.
- [ ] Follows formula shape for the chosen structure.
- [ ] Uses snake_case only.
- [ ] Does not start with `test_`.
- [ ] Does not include multiple behaviors joined by `and`.
- [ ] Is specific enough to understand failure without opening test body.

### Test module names

- [ ] Uses singular names (`constructor`, not `constructors`).
- [ ] Uses canonical module names when applicable.
- [ ] Keeps concerns separate (`validation` vs `proptests` vs `fixtures`).
- [ ] Improves scanability of test output.

## Quick checklist

- Name states one behavior.
- Name includes failure/success condition when relevant.
- No `and` in the test name unless the behavior is truly atomic.
- Module names are concise and singular where applicable.
- Nearby tests use the same naming style.

## Canonical examples

### Structure A (preferred)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod lookup {
        use super::*;

        #[test]
        fn returns_none_when_record_is_missing() {}

        #[test]
        fn returns_record_when_key_exists() {}
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_key_when_empty() {}
    }
}
```

### Structure B (small/simple)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_none_when_record_is_missing() {}

    #[test]
    fn lookup_returns_record_when_key_exists() {}
}
```

## Repository notes

- Current tests heavily use `should_*` and `returns_*`/`rejects_*` forms.
- Core organization tends toward domain concern modules (`constructor`, `validation`, `conversions`) rather than CQRS buckets.

## References

- Rust Book: unit test structure and organization.
  - https://doc.rust-lang.org/book/ch11-01-writing-tests.html
  - https://doc.rust-lang.org/book/ch11-03-test-organization.html
- Rust API Guidelines: examples and failure documentation.
  - https://rust-lang.github.io/api-guidelines/documentation.html
