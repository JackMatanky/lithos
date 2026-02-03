# Phase 9: Test Suite Cleanup & Best Practices (Tea Agent)

**Status:** PROPOSED
**Date:** 2026-02-03
**Author:** Tea Agent
**Duration:** 2-3 days
**Priority:** P1 - High (Blocks `_legacy` cleanup)
**Blocking:** Must complete before archiving test-utils and test-macros to `_legacy/`

---

## 1. Problem Statement

### Current State Issues

The test suite still depends on `lithos-test-utils` and `lithos-test-macros`, which:

1. **Violates Sync-First Principle**: test-utils has async dependencies (tokio, async-trait, tokio-test)
2. **Unnecessary Abstraction**: Many utilities are over-engineered for simple test cases
3. **Blocks Legacy Cleanup**: Cannot move test crates to `_legacy/` while core depends on them
4. **Non-Idiomatic Rust**: Uses custom test macros instead of standard Rust patterns

### Dependencies to Remove

**Current usage in `lithos-core`:**
```
lithos-core/src/note/aggregate.rs:     use lithos_test_utils::assert_err_kind;
lithos-core/src/note/aggregate.rs:     use lithos_test_utils::test_builder;
lithos-core/src/config/aggregate.rs:   use lithos_test_utils::assert_err_kind;
lithos-core/src/template/aggregate.rs: use lithos_test_utils::data::properties::valid_identifier;
lithos-core/src/schema/graph.rs:       use lithos_test_utils::assert_eq_detailed;
lithos-core/src/schema/property.rs:    use lithos_test_utils::data::properties::{...};
lithos-core/src/schema/property_spec.rs: use lithos_test_utils::assert_err_kind;
```

**Total:** 9 import sites across 6 files

---

## 2. Scope of Changes

### 2.1 Replace Custom Test Utilities

#### `assert_err_kind` → Standard Rust Pattern

**Current (lithos-test-utils):**
```rust
use lithos_test_utils::assert_err_kind;

assert_err_kind!(result, NoteError::ValidationFailed(_));
```

**Replacement (Standard Rust):**
```rust
// Option A: Pattern matching (idiomatic)
assert!(matches!(result, Err(NoteError::ValidationFailed(_))));

// Option B: More explicit (for complex cases)
match result {
    Err(NoteError::ValidationFailed(_)) => {},
    other => panic!("Expected ValidationFailed, got: {:?}", other),
}
```

**Benefits:**
- ✅ No custom macro needed
- ✅ Standard Rust pattern
- ✅ Better IDE support (jump-to-definition works)
- ✅ Easier for Rust developers to understand

#### `test_builder` Macro → Manual Builder

**Current (lithos-test-macros):**
```rust
use lithos_test_utils::test_builder;

test_builder!(NoteBuilder, Note, {
    id: Uuid = Uuid::now_v7(),
    path: NotePath = NotePath::new("default.md".to_owned()).unwrap(),
    // ... 8 more fields
});

// Usage
let note = NoteBuilder::default()
    .id(custom_id)
    .path(custom_path)
    .build();
```

**Replacement (Manual Builder or Test Fixtures):**

**Option A: Simple test fixture function:**
```rust
#[cfg(test)]
pub(crate) fn test_note(id: Uuid, path: &str) -> Note {
    Note {
        id,
        path: NotePath::new(path.to_owned()).expect("valid test path"),
        frontmatter: None,
        links: vec![],
        tags: vec![],
        headings: vec![],
        tasks: vec![],
        sections: vec![],
        pending_events: vec![],
    }
}

// Usage (simpler!)
let note = test_note(Uuid::now_v7(), "test.md");
```

**Option B: Manual builder (only if needed):**
```rust
#[cfg(test)]
pub(crate) struct NoteBuilder {
    id: Uuid,
    path: String,
    // ... only fields that vary in tests
}

impl NoteBuilder {
    pub fn new(id: Uuid, path: &str) -> Self {
        Self { id, path: path.to_owned() }
    }

    pub fn build(self) -> Note {
        Note {
            id: self.id,
            path: NotePath::new(self.path).expect("valid"),
            // ... defaults
        }
    }
}
```

**Benefits:**
- ✅ No proc macro complexity
- ✅ Faster compile times (no macro expansion)
- ✅ Easier to debug
- ✅ More flexible (can add custom logic)

#### `assert_eq_detailed` → Standard `assert_eq!`

**Current:**
```rust
use lithos_test_utils::assert_eq_detailed;

assert_eq_detailed!(actual, expected);
```

**Replacement:**
```rust
assert_eq!(actual, expected);
// Rust already provides detailed output when T: Debug
```

**Rationale:**
- Rust's `assert_eq!` already shows detailed diffs for types implementing `Debug`
- No need for custom macro

#### `valid_identifier` / Property Test Helpers → Local Functions

**Current:**
```rust
use lithos_test_utils::data::properties::valid_identifier;

proptest! {
    #[test]
    fn test_something(id in valid_identifier()) {
        // test
    }
}
```

**Replacement:**
```rust
use proptest::prelude::*;

prop_compose! {
    fn valid_identifier()(s in "[a-z][a-z0-9_-]{0,63}") -> String {
        s
    }
}

proptest! {
    #[test]
    fn test_something(id in valid_identifier()) {
        // test
    }
}
```

**Benefits:**
- ✅ Self-contained in test module
- ✅ Easy to customize per context
- ✅ No external dependency

---

### 2.2 Rust Testing Best Practices Review

#### Apply Standard Rust Test Patterns

**Best Practice Checklist:**

1. **Co-location**: Tests in same file as implementation using `#[cfg(test)]`
   - ✅ Already implemented in lithos-core

2. **Test Module Structure**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       mod specific_feature {
           use super::*;

           #[test]
           fn should_do_something() {
               // test
           }
       }
   }
   ```
   - ✅ Already using this pattern

3. **Assertion Patterns**:
   ```rust
   // ✅ Good: Standard assertions
   assert!(condition);
   assert_eq!(actual, expected);
   assert!(matches!(result, Ok(_)));

   // ❌ Avoid: Custom assertion macros
   assert_err_kind!(result, ErrorType);
   ```

4. **Test Naming**:
   ```rust
   // ✅ Good: Descriptive, behavior-focused
   #[test]
   fn should_reject_empty_path() { }

   #[test]
   fn validates_uuid_v7_format() { }

   // ❌ Avoid: Generic names
   #[test]
   fn test1() { }
   ```
   - ✅ Already following this pattern

5. **Fixtures**:
   ```rust
   // ✅ Good: Simple helper functions
   fn test_note() -> Note { ... }

   // ❌ Avoid: Complex builder macros
   test_builder!(NoteBuilder, ...);
   ```

6. **Property Testing**:
   ```rust
   // ✅ Good: Inline strategies
   proptest! {
       #[test]
       fn prop(s in "[a-z]+") { ... }
   }

   // ❌ Avoid: External strategy dependencies
   ```

---

### 2.3 Test Organization Review

#### Current Structure
```
tests/
├── arch/          # Architecture tests (keep)
├── macros/        # Test macros (REMOVE - move to _legacy)
└── utils/         # Test utilities (REMOVE - move to _legacy)
```

#### Target Structure
```
tests/
└── arch/          # Architecture tests only

lithos-core/
└── src/
    └── note/
        └── aggregate.rs   # Tests co-located with #[cfg(test)]
```

**Rationale:**
- Tests should be co-located with implementation (already doing this)
- Only keep `tests/arch/` for cross-cutting architecture enforcement
- No need for separate test utilities crate in sync-first design

---

## 3. Implementation Plan

### 3.1 Audit Test Usage (1-2 hours)

**Tasks:**
- [x] Identify all `lithos-test-utils` imports in `lithos-core`
- [x] Identify all `lithos-test-macros` usage
- [x] Document what each utility does
- [ ] Categorize by replacement complexity

**Deliverable:** Comprehensive usage report (see Section 2.1)

### 3.2 Replace Assertion Utilities (2-3 hours)

**Files to modify:**
1. `lithos-core/src/note/aggregate.rs` (2 imports)
2. `lithos-core/src/config/aggregate.rs` (1 import)
3. `lithos-core/src/schema/property_spec.rs` (3 imports)
4. `lithos-core/src/schema/graph.rs` (1 import)

**Pattern:**
```rust
// Before
use lithos_test_utils::assert_err_kind;
assert_err_kind!(result, NoteError::ValidationFailed(_));

// After
assert!(matches!(result, Err(NoteError::ValidationFailed(_))));
```

**Verification:**
- [ ] Run `cargo test --package lithos-core` after each file
- [ ] Ensure all tests still pass
- [ ] Verify error messages are still helpful

### 3.3 Replace Builder Macro (2-3 hours)

**File:** `lithos-core/src/note/aggregate.rs`

**Steps:**
1. Remove `test_builder!` macro invocation
2. Create simple test fixture function:
   ```rust
   #[cfg(test)]
   mod test_fixtures {
       use super::*;

       pub(crate) fn test_note(id: Uuid, path: &str) -> Note {
           Note {
               id,
               path: NotePath::new(path.to_owned()).expect("valid"),
               frontmatter: None,
               links: vec![],
               tags: vec![],
               headings: vec![],
               tasks: vec![],
               sections: vec![],
               pending_events: vec![],
           }
       }
   }
   ```
3. Update all test cases using `NoteBuilder::default()` to use `test_note()`
4. Run tests to verify

**Verification:**
- [ ] All note tests pass
- [ ] No performance regression
- [ ] Code is more readable

### 3.4 Replace Property Test Helpers (1-2 hours)

**Files:**
- `lithos-core/src/template/aggregate.rs`
- `lithos-core/src/schema/property.rs`

**Steps:**
1. Add `prop_compose!` macros inline in test modules
2. Replace imports from `lithos_test_utils::data::properties`
3. Run property tests to verify

**Example:**
```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    prop_compose! {
        fn valid_identifier()(s in "[a-z][a-z0-9_-]{0,63}") -> String {
            s
        }
    }

    proptest! {
        #[test]
        fn validates_identifier(id in valid_identifier()) {
            // test
        }
    }
}
```

### 3.5 Remove Test Dependencies (1 hour)

**File:** `lithos-core/Cargo.toml`

**Changes:**
```toml
[dev-dependencies]
# REMOVE these lines:
# lithos-test-utils = { workspace = true }

# Keep these (standard Rust testing):
rstest = { workspace = true }
proptest = { workspace = true }
tempfile.workspace = true
criterion = { workspace = true, features = ["html_reports"] }
```

**Verification:**
- [ ] `cargo build --package lithos-core` succeeds
- [ ] `cargo test --package lithos-core` passes (all 251 tests)
- [ ] `cargo clippy --package lithos-core` has zero warnings

### 3.6 Move Test Crates to Legacy (1 hour)

**Once all dependencies removed:**

```bash
# Move test utilities to legacy
mv tests/utils/ _legacy/tests/utils/
mv tests/macros/ _legacy/tests/macros/

# Update workspace Cargo.toml to remove them
# Remove from [workspace.members]:
# - "tests/utils"
# - "tests/macros"
```

**Final structure:**
```
tests/
└── arch/          # Only architecture tests remain

_legacy/
├── tests/
│   ├── utils/     # Old async test utilities
│   └── macros/    # Old test builder macros
└── crates/        # Old multi-crate structure
```

### 3.7 Documentation Update (30 minutes)

**Files to update:**
1. `tests/README.md` - Remove references to test-utils/test-macros
2. `docs/refs/rust/testing.md` - Add Rust testing best practices guide
3. `AGENTS.md` - Update test examples to use standard patterns

**Content:**
```markdown
# Rust Testing Best Practices

## Standard Patterns

### Assertions
Use built-in `assert!` and `matches!` macro:
```rust
assert!(matches!(result, Ok(_)));
assert!(matches!(err, Err(MyError::Specific(_))));
```

### Fixtures
Simple helper functions, not complex builders:
```rust
#[cfg(test)]
fn test_entity() -> Entity {
    Entity { /* defaults */ }
}
```

### Property Testing
Inline strategies with `prop_compose!`:
```rust
prop_compose! {
    fn valid_string()(s in "[a-z]+") -> String { s }
}
```
```

---

## 4. Acceptance Criteria

### Test Quality
- [ ] All 251 lithos-core tests pass
- [ ] No `lithos-test-utils` imports in `lithos-core`
- [ ] No `lithos-test-macros` usage in `lithos-core`
- [ ] Test code uses standard Rust patterns only

### Code Quality
- [ ] `cargo clippy --package lithos-core` has zero warnings
- [ ] No custom test macros (use standard Rust)
- [ ] Test code is more readable than before

### Dependency Cleanup
- [ ] `tests/utils/` moved to `_legacy/tests/utils/`
- [ ] `tests/macros/` moved to `_legacy/tests/macros/`
- [ ] Workspace no longer includes test utility crates
- [ ] `cargo build --workspace` succeeds

### Documentation
- [ ] Testing best practices documented
- [ ] Examples use standard Rust patterns
- [ ] No references to removed test utilities

---

## 5. Risk Assessment

### Low Risk Items
- ✅ Replacing `assert_err_kind` with `matches!` (straightforward)
- ✅ Removing `assert_eq_detailed` (Rust has this built-in)
- ✅ Moving test crates to `_legacy` (once dependencies removed)

### Medium Risk Items
- ⚠️ Replacing `test_builder` macro (requires updating multiple test cases)
- ⚠️ Replacing property test helpers (need to replicate logic inline)

**Mitigation:** Run tests after each change, verify individually

### High Risk Items
- ❌ None - all changes are local to test code

---

## 6. Success Metrics

| Metric                           | Current | Target | Verification               |
| -------------------------------- | ------- | ------ | -------------------------- |
| lithos-test-utils imports        | 9       | 0      | `grep -r` check              |
| Custom test macros               | 1       | 0      | `grep -r test_builder`       |
| Test utility crates in workspace | 2       | 0      | `Cargo.toml` review          |
| Test pass rate                   | 100%    | 100%   | `cargo test`                 |
| Async test dependencies          | Yes     | No     | `Cargo.toml` audit           |
| Standard Rust patterns           | 50%     | 100%   | Manual review              |

---

## 7. Timeline

**Estimated Duration:** 2-3 days

| Task                         | Duration | Status    |
| ---------------------------- | -------- | --------- |
| 3.1 Audit test usage         | 1-2h     | COMPLETE  |
| 3.2 Replace assertion utils  | 2-3h     | PENDING   |
| 3.3 Replace builder macro    | 2-3h     | PENDING   |
| 3.4 Replace property helpers | 1-2h     | PENDING   |
| 3.5 Remove dependencies      | 1h       | PENDING   |
| 3.6 Move to legacy           | 1h       | PENDING   |
| 3.7 Update documentation     | 30m      | PENDING   |
| **Total**                        | **8-12h**    | **0% done** |

---

## 8. Rollback Strategy

**If issues arise:**

1. **Individual file level**: Git revert specific commits
2. **Full rollback**: Revert to pre-Phase-9 commit
3. **Partial completion**: Keep completed replacements, defer others

**Safety:** All changes are in test code only - no production code affected.

---

## 9. Next Steps After Phase 9

Once Phase 9 completes:

1. ✅ Archive `_legacy/` directory entirely (no longer needed)
2. ✅ Clean workspace is ready for feature development
3. ✅ New contributors see only idiomatic Rust patterns
4. ✅ Faster compile times (no test macro expansion)

---

## 10. Agent Assignment

**Primary:** Tea Agent (Testing & Quality Assurance)
**Support:** Dev Agent (for complex refactoring if needed)

**Tea responsibilities:**
- Review all test code for best practices
- Replace custom utilities with standard patterns
- Verify test quality and coverage
- Document Rust testing patterns

---

**Status:** Ready for execution
**Approval Required:** Yes (impacts test infrastructure)
**Blocking:** Must complete before final `_legacy/` cleanup
