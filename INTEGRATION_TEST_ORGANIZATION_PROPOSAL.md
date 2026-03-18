# Integration Test Organization Proposal

## Problem

Current organization splits Loader tests across multiple files:
- `schema_resolution.rs` - Tests Loader's resolution pipeline
- `schema_incremental.rs` (proposed) - Tests Loader's incremental loading

But **resolution and incremental loading are the same concern**: both test the Loader.

## Proposed Solution

**Consolidate all Loader tests into one file: `schema_loader.rs`**

---

## Recommended File Structure

```
lithos-core/tests/
├── common/mod.rs           # Shared utilities (TestDb, builders)
├── schema_storage.rs       # Repository + redb integration (4 tests)
└── schema_loader.rs        # Loader pipeline integration (10 tests)
```

### Rationale

1. **Cohesion**: All Loader tests in one place
2. **Discoverability**: Developers know where to find Loader tests
3. **Clear separation**: Storage vs Loading concerns are distinct
4. **Scalability**: Can add more Loader test modules as file grows

---

## File: `schema_storage.rs` (KEEP AS-IS)

**Purpose**: Test Repository trait persistence with **real redb database**

**Tests** (4 total, 2 ignored):
- ✅ `property_bank_roundtrip` - Save/retrieve PropertyBank
- ✅ `schema_roundtrip` - Save/retrieve Schema by ID
- ✅ `schema_find_by_name` - Save/retrieve Schema by name
- ⚠️ `schema_list` (IGNORED) - List multiple schemas
- ⚠️ `schema_delete` (IGNORED) - Delete schema

**Scope**: Low-level persistence only
- No file I/O
- No Loader/Ingestor
- Direct Repository method calls

---

## File: `schema_loader.rs` (RENAME + CONSOLIDATE)

**Purpose**: Test Loader pipeline with **real filesystem + database**

### Module Organization (by behavior, not feature)

```rust
//! Integration tests for schema loading pipeline.
//!
//! Tests the Loader's ability to load and resolve schemas from files,
//! including:
//! - Initial loading (file → ingest → resolve → persist)
//! - Reference expansion ($ref to property_bank)
//! - Inheritance resolution (extends/excludes)
//! - Incremental loading (staleness detection)
//! - Property bank updates (incremental re-resolution)
//! - Error handling (missing refs, circular inheritance)

// ========================================================================
//                       Initial Loading Tests
// ========================================================================

/// Tests for first-time schema loading (all files are NEW)
mod initial_loading {
    /// Test: property_bank references resolve correctly
    fn resolves_property_bank_references()

    /// Test: inline properties resolve correctly
    fn resolves_inline_properties()

    /// Test: multiple schemas load in single session
    fn resolves_multiple_schemas()

    /// Test: property bank loads and persists
    fn loads_and_persists_property_bank()
}

// ========================================================================
//                       Inheritance Tests
// ========================================================================

/// Tests for schema inheritance (extends/excludes)
mod inheritance {
    /// Test: child schema inherits parent properties
    fn resolves_schema_inheritance()
}

// ========================================================================
//                       Incremental Loading Tests
// ========================================================================

/// Tests for staleness detection and incremental resolution
mod incremental_loading {
    /// Test: file change detected via mtime/hash
    fn detects_file_changes()

    /// Test: views persist across database sessions
    fn staleness_persists_across_reopens()

    /// Test: property_bank change triggers re-resolution
    fn property_bank_update_triggers_re_resolution()
}

// ========================================================================
//                       Error Handling Tests
// ========================================================================

/// Tests for error detection and propagation
mod error_handling {
    /// Test: missing property_bank reference detected
    fn detects_missing_property_bank_reference()

    /// Test: circular inheritance detected
    fn detects_circular_inheritance()
}
```

**Total**: 10 tests (7 existing from schema_resolution.rs + 3 new)

---

## Migration Plan

### Step 1: Rename File
```bash
git mv lithos-core/tests/schema_resolution.rs lithos-core/tests/schema_loader.rs
```

### Step 2: Update File Header
```rust
//! Integration tests for schema loading pipeline.
//!
//! Tests the Loader's ability to load and resolve schemas from files,
//! including:
//! - Initial loading (file → ingest → resolve → persist)
//! - Reference expansion ($ref to property_bank)
//! - Inheritance resolution (extends/excludes)
//! - Incremental loading (staleness detection)
//! - Property bank updates (incremental re-resolution)
//! - Error handling (missing refs, circular inheritance)
```

### Step 3: Organize Tests into Modules

**Before** (flat):
```rust
#[test]
fn resolves_property_bank_references() { ... }

#[test]
fn resolves_inline_properties() { ... }

// ... 7 tests
```

**After** (organized):
```rust
mod initial_loading {
    use super::*;

    #[test]
    fn resolves_property_bank_references() { ... }

    #[test]
    fn resolves_inline_properties() { ... }

    #[test]
    fn resolves_multiple_schemas() { ... }

    #[test]
    fn loads_and_persists_property_bank() { ... }
}

mod inheritance {
    use super::*;

    #[test]
    fn resolves_schema_inheritance() { ... }
}

mod incremental_loading {
    use super::*;

    #[test]
    fn detects_file_changes() { ... }

    #[test]
    fn staleness_persists_across_reopens() { ... }

    #[test]
    fn property_bank_update_triggers_re_resolution() { ... }
}

mod error_handling {
    use super::*;

    #[test]
    fn detects_missing_property_bank_reference() { ... }

    #[test]
    fn detects_circular_inheritance() { ... }
}
```

### Step 4: Add 3 New Tests to `incremental_loading` Module

(Already spec'd in INTEGRATION_TEST_ACTION_PLAN.md)

### Step 5: Update Test Names (if needed)

Current test names are already good (behavior-focused), no changes needed.

---

## Alternative: `schema_loader.rs` + modules

If the file grows too large (>500 lines), split into:

```
lithos-core/tests/
├── common/mod.rs
├── schema_storage.rs
└── schema_loader/
    ├── mod.rs                  # Common setup
    ├── initial_loading.rs      # 4 tests
    ├── inheritance.rs          # 1 test
    ├── incremental_loading.rs  # 3 tests
    └── error_handling.rs       # 2 tests
```

**Recommendation**: Start with single file, split only if needed.

---

## Comparison: Current vs Proposed

### Current (Split by Feature)
```
tests/
├── schema_resolution.rs    # 7 tests (resolution features)
└── schema_incremental.rs   # 3 tests (incremental features)
```

**Problems**:
- Artificial separation (both test Loader)
- Hard to decide where new tests go
- Duplicate setup code

### Proposed (Organized by Concern)
```
tests/
├── schema_storage.rs       # 4 tests (Repository persistence)
└── schema_loader.rs        # 10 tests (Loader pipeline)
    ├── initial_loading     # 4 tests (first load)
    ├── inheritance         # 1 test (extends/excludes)
    ├── incremental_loading # 3 tests (staleness)
    └── error_handling      # 2 tests (errors)
```

**Benefits**:
- ✅ Clear concern separation (Storage vs Loader)
- ✅ Easy to find tests (all Loader tests in one place)
- ✅ Natural organization (by behavior, not feature)
- ✅ Shared setup code (no duplication)

---

## Decision Needed

**Option 1**: Single file `schema_loader.rs` with submodules (RECOMMENDED)
- Simpler
- Easier to navigate
- Can split later if needed

**Option 2**: Directory `schema_loader/` with separate files
- More scalable
- Better for very large test suites
- More overhead initially

**Recommendation**: Start with Option 1, refactor to Option 2 only if file exceeds 500 lines.

---

## Test Naming Convention

Tests follow the `unit_of_work + expected_behavior + state_under_test` pattern:

✅ **GOOD** (current names):
- `resolves_property_bank_references` (clear: what + how)
- `detects_circular_inheritance` (clear: what + error case)
- `loads_and_persists_property_bank` (clear: what + persistence)

✅ **GOOD** (new names):
- `detects_file_changes` (clear: incremental loading behavior)
- `staleness_persists_across_reopens` (clear: persistence + staleness)
- `property_bank_update_triggers_re_resolution` (clear: cause + effect)

---

## Updated INTEGRATION_TEST_ACTION_PLAN.md

**Change**: Replace references to `schema_incremental.rs` with `schema_loader.rs`

**New tests go in**: `schema_loader.rs::incremental_loading` module

**File organization**:
```rust
// lithos-core/tests/schema_loader.rs

mod initial_loading { /* 4 tests */ }
mod inheritance { /* 1 test */ }
mod incremental_loading {
    // NEW: Add these 3 tests here
    #[test]
    fn detects_file_changes() { ... }

    #[test]
    fn staleness_persists_across_reopens() { ... }

    #[test]
    fn property_bank_update_triggers_re_resolution() { ... }
}
mod error_handling { /* 2 tests */ }
```

---

## Summary

**Recommendation**: Consolidate all Loader tests into `schema_loader.rs`

**Rationale**:
- Resolution and incremental loading are the same concern (Loader behavior)
- Organizing by concern (Storage vs Loader) is clearer than by feature
- Submodules within the file provide natural organization
- Easier to maintain and extend

**Implementation**:
1. Rename `schema_resolution.rs` → `schema_loader.rs`
2. Organize existing 7 tests into submodules
3. Add 3 new tests to `incremental_loading` module
4. Update documentation

**Result**: Clean 2-file organization with clear concerns
