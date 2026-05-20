---
title: 02-fs-name-types
category: enhancement
label: completed
status: completed
date_created: 2026-05-11
date_completed: 2026-05-12
---

## Type

AFK

## Labels

- completed

## What to build

Create fs/name.rs: owned types FileName, DirName, BaseName (Box<str>) and borrowed Ref types FileNameRef<'a>, DirNameRef<'a>, BaseNameRef<'a> (&'a OsStr).

Follow suffix pattern: no suffix = owned, Ref suffix = borrowed. BaseName follows Obsidian terminology (filename without extension).

## Acceptance criteria

- [x] FileName(Box<str>) - owned filename
- [x] DirName(Box<str>) - owned dirname
- [x] BaseName(Box<str>) - owned basename (Obsidian term)
- [x] FileNameRef<'a>(&'a OsStr) - borrowed filename view
- [x] DirNameRef<'a>(&'a OsStr) - borrowed dirname view
- [x] BaseNameRef<'a>(&'a OsStr) - borrowed basename view
- [x] Zero-copy extraction methods from path types
- [x] Conversion between owned and borrowed types
- [x] Tests for creation and extraction
- [x] Update fs/mod.rs exports

## Blocked by

None - can start immediately

## Agent Brief

**Category:** enhancement
**Summary:** Create owned and borrowed filename types with support for Obsidian-style "basename" terminology.

**Current behavior:**
Filenames are handled using generic `OsStr` or `String` types. This leads to ambiguity about whether a "name" includes an extension, and often results in unnecessary allocations when extracting just the name or the stem from a path.

**Desired behavior:**
Implement a suite of types for owned and borrowed name components following the `Ref` suffix pattern for borrowed views. Specifically, support "BaseName" which follows Obsidian terminology (the filename without its extension), as this is a core domain concept for wikilink resolution.

**Key interfaces:**
- `FileName`, `DirName`, `BaseName` — owned `Box<str>` representations
- `FileNameRef<'a>`, `DirNameRef<'a>`, `BaseNameRef<'a>` — borrowed `&'a OsStr` views
- Extraction methods on `FilePath` and `DirPath` to return these types zero-copy where possible

**Acceptance criteria:**
- [ ] All owned types use `Box<str>` for space efficiency
- [ ] All borrowed types wrap `&OsStr` for zero-copy views
- [ ] `BaseName` correctly extracts the file stem (Obsidian "basename")
- [ ] Suffix pattern is strictly followed: no suffix = owned, `Ref` suffix = borrowed
- [ ] Conversions between owned and borrowed types are implemented (`ToOwned`, `From`, etc.)
- [ ] Tests verify correct extraction from various path strings (with/without dots, hidden files, etc.)

**Out of scope:**
- Path validation (reserved for Issue 01)
- Extension-specific logic (reserved for Issue 03)

## Implementation Notes

**File:** `lithos-core/src/fs/name.rs`

**Implemented Types:**

**Owned Types (Box<str>):**
- `FileName(Box<str>)` - Full filename including extension
- `BaseName(Box<str>)` - Filename without extension (Obsidian terminology)
- `DirName(Box<str>)` - Directory name component

**Borrowed Types (&'a OsStr):**
- `FileNameRef<'a>(&'a OsStr)` - Zero-copy filename view
- `BaseNameRef<'a>(&'a OsStr)` - Zero-copy basename view
- `DirNameRef<'a>(&'a OsStr)` - Zero-copy dirname view

**Key Methods:**
- `FileName::basename()` - Extract basename (stem) without extension
- `FileName::extension()` - Extract file extension
- `FileName::as_str()` - String view
- `FileNameRef::to_owned()` - Convert to owned `FileName`
- `TryFrom<&Path>` for `FileName` - Extract from path
- `From<String>` for `FileName` - Construct from string

**Conversions:**
- Owned → Borrowed: via `as_ref()` methods
- Borrowed → Owned: via `to_owned()` and `ToOwned` trait
- String → FileName: via `From<String>`
- Path → FileName: via `TryFrom<&Path>`

**Migration:**
- Existing `FileName` in `fs/file.rs` replaced with re-export from `fs/name.rs`
- Eliminates duplication while maintaining backward compatibility

**Tests:**
- Tests integrated with `fs/path.rs` tests (22 total tests)
- Covers filename extraction, basename extraction, extension handling
- Edge cases: hidden files, no extension, multiple dots

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FileName`, `BaseName`, `DirName`, `FileNameRef`, `BaseNameRef`, `DirNameRef`
- Re-exported in `fs/file.rs` for backward compatibility

**Status:** ✅ Complete - All acceptance criteria met

---

## Post-Implementation Review (2026-05-13)

### Review Scope
Critical review of `lithos-core/src/fs/name.rs` using Apollo Rust Best Practices and TDD principles before Issue 08 migration to `FileMetadata`/`FsEntry`.

### Critical Issues Found

#### 1. API Confusion: `basename()` vs `to_basename()` (HIGH PRIORITY)

**Problem:**
Two methods for the same concept with inconsistent return types:
```rust
// Line 38: Returns borrowed &str
pub fn basename(&self) -> &str {
    Path::new(self.as_str())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("") // ❌ Returns empty string on failure
}

// Line 51: Returns owned Option<BaseName>
pub fn to_basename(&self) -> Option<BaseName> {
    BaseName::try_from(Path::new(self.as_str())).ok()
}
```

**Apollo Violations:**
- Naming violates Rust conventions: `basename()` should return `BaseName`, not `&str`
- `to_` prefix signals type conversion (Chapter 1)
- Silent failure: `basename()` returns `""` for invalid cases, hiding errors
- Redundant APIs create confusion

**Decision:** Option A - Single `basename() -> Option<BaseName>`

**Rationale:**
- `BaseName` is the domain type; method should return it
- Explicit `Option` makes failure case visible to caller
- Matches project's error handling patterns (no `unwrap_or` defaults in APIs)
- Eliminates API redundancy

#### 2. Silent Failure in `From<FileName> for BaseName` (HIGH PRIORITY)

**Problem:**
```rust
// Lines 210-221
impl From<FileName> for BaseName {
    fn from(name: FileName) -> Self {
        Path::new(name.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .map_or_else(
                || BaseName::new("".into()), // ❌ Creates empty BaseName
                |s| BaseName::new(s.into()),
            )
    }
}
```

**Apollo Chapter 4 Violation:**
- `From` trait cannot fail, forcing creation of invalid empty `BaseName`
- Hides errors from callers
- Empty basenames likely violate domain invariants

**Fix:** Change to `TryFrom<FileName> for BaseName` returning `Result`

#### 3. Potentially Dead Conversion Traits (MEDIUM PRIORITY)

**Problem:**
9+ `From`/`TryFrom` implementations may become unnecessary after FilePath/DirPath migration:

**FileName conversions (6):**
- `From<Box<str>>`
- `From<String>`
- `From<FileName> for Box<str>`
- `From<FileName> for String>`
- `TryFrom<&Path>`
- `TryFrom<PathBuf>`

**BaseName conversions (6):**
- `From<Box<str>>`
- `From<String>`
- `From<FileName>` ❌ (creates empty strings)
- `From<BaseName> for Box<str>`
- `From<BaseName> for String>`
- `TryFrom<&Path>`
- `TryFrom<PathBuf>`

**Apollo Chapter 3 (Performance) + Chapter 1 (API Design):**
- Trait implementations increase maintenance burden
- Conversions encourage unnecessary allocations
- If `FilePath`/`DirPath` become primary constructors, many are dead code

**Action:** Audit usage with `rg "FileName::try_from|FileName::from|BaseName::try_from|BaseName::from" --type rust`

#### 4. Test Suite Not Organized Per Apollo Chapter 5 (HIGH PRIORITY)

**Problem:**
Flat test module instead of submodules by type and behavior:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn should_extract_basename_correctly() { ... }

    #[test]
    fn should_convert_to_owned_basename() { ... }

    // ... 9 tests all in one flat module
}
```

**Apollo Chapter 5.1 Violation:**
> Use modules for organization. Most IDEs can run a single module of tests all together.

**Current output:**
```
tests::should_extract_basename_correctly ... ok
tests::should_convert_to_owned_basename ... ok
```

**Should be (following metadata.rs pattern):**
```
file_name::basename::extracts_stem_from_filename ... ok
base_name::try_from_path::constructs_from_file_stem ... ok
```

#### 5. Test Names Test Implementation, Not Behavior (MEDIUM PRIORITY)

**Apollo Chapter 5.1:**
> ✅ Use a name which reads like a sentence, describing the desired behavior

**Current names (implementation-focused):**
- `should_extract_basename_correctly` ❌ (vague "correctly")
- `should_convert_to_owned_basename` ❌ (tests conversion mechanism)
- `should_try_from_path_to_basename` ❌ (tests trait name)

**Should be (behavior-focused):**
- `extracts_stem_from_simple_filename` ✅
- `returns_some_for_valid_stem` ✅
- `constructs_from_file_stem` ✅

#### 6. Multiple Assertions Per Test (LOW PRIORITY)

**Example (lines 336-339):**
```rust
#[test]
fn should_return_basename_as_owned() {
    let name = FileName::from("my-note.md".to_owned());
    let base = name.to_basename();
    assert!(base.is_some());  // ❌ First assertion
    assert_eq!(base.unwrap().as_str(), "my-note");  // ❌ Second assertion
}
```

**Apollo Chapter 5.1:**
> Use very few, ideally one, assertion per test

### What's Good (Keep These Patterns)

✅ **Box<str> Storage:**
```rust
pub struct FileName(Box<str>);
pub struct BaseName(Box<str>);
pub struct DirName(Box<str>);
```
- Appropriate for unbounded filename lengths
- More efficient than `String` (saves 8 bytes per instance)
- Apollo Chapter 3.3 compliant

✅ **Inline Annotations:**
- All small methods correctly marked `#[inline]`

✅ **Borrowed Returns:**
- Accessors return `&str` correctly (no unnecessary allocations)
- Follows Apollo Chapter 1.1: "prefer `&str` over `String`"

✅ **rkyv Derives:**
- Types properly annotated for zero-copy serialization

---

## Refactor Plan

### Phase 1: API Fixes (Before Issue 08)

**Goal:** Fix critical API issues to prevent consumer confusion during `FileMetadata` migration.

#### Task 1.1: Consolidate Basename API
- [ ] Delete `FileName::basename() -> &str` (line 38-43)
- [ ] Rename `to_basename() -> Option<BaseName>` to `basename() -> Option<BaseName>` (line 51-53)
- [ ] Update all call sites (if any exist outside tests)
- [ ] Write tests using TDD for new `basename()` behavior

**Rationale:**
- Single source of truth for basename extraction
- Explicit `Option` makes failure case visible
- Domain type `BaseName` returned directly

#### Task 1.2: Fix Silent Failure in BaseName Conversion
- [ ] Change `impl From<FileName> for BaseName` to `impl TryFrom<FileName> for BaseName`
- [ ] Return `Result<BaseName, std::io::Error>` matching existing `TryFrom<&Path>` pattern
- [ ] Update error message: "Path has no stem component"
- [ ] Update call sites (search for `.into()` on `FileName` → `BaseName`)
- [ ] Write tests for both success and error cases

**Rationale:**
- No more empty `BaseName` construction
- Errors propagate to callers
- Consistent with existing error handling

#### Task 1.3: Audit Conversion Trait Usage
- [ ] Run: `rg "FileName::try_from|FileName::from" --type rust -C 2 > /tmp/filename-usage.txt`
- [ ] Run: `rg "BaseName::try_from|BaseName::from" --type rust -C 2 > /tmp/basename-usage.txt`
- [ ] Analyze which conversions are actually used
- [ ] Mark unused conversions with `// TODO(Issue-XX): Remove if still unused after FilePath migration`
- [ ] Document findings in this issue

**Decision Criteria:**
- If conversion used <3 times: candidate for removal
- If conversion only in tests: mark `#[cfg(test)]` or delete
- If conversion in production: keep but monitor during migration

### Phase 2: Test Suite Refactor (Apollo Chapter 5 Compliance)

**Goal:** Organize tests into submodules matching `metadata.rs` pattern (23 tests in 4 submodules).

#### Task 2.1: Reorganize Test Structure
Transform flat structure into nested submodules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    mod file_name {
        use super::*;

        mod basename {
            use super::*;

            #[test]
            fn extracts_stem_from_simple_filename() { ... }

            #[test]
            fn handles_double_extension() { ... }

            #[test]
            fn returns_none_for_extensionless_hidden_files() { ... }

            #[test]
            fn returns_some_for_hidden_files_with_extension() { ... }
        }

        mod extension {
            use super::*;

            #[test]
            fn extracts_extension_from_simple_filename() { ... }

            #[test]
            fn returns_none_when_no_extension() { ... }
        }

        mod as_ref {
            use super::*;

            #[test]
            fn creates_borrowed_view() { ... }
        }
    }

    mod base_name {
        use super::*;

        mod try_from_path {
            use super::*;

            #[test]
            fn constructs_from_file_stem() { ... }

            #[test]
            fn extracts_from_full_path() { ... }

            #[test]
            fn returns_error_for_path_without_stem() { ... }
        }

        mod try_from_filename {
            use super::*;

            #[test]
            fn converts_filename_with_extension() { ... }

            #[test]
            fn returns_error_for_hidden_file_without_extension() { ... }
        }
    }

    mod file_name_ref {
        use super::*;

        mod basename {
            use super::*;

            #[test]
            fn extracts_borrowed_basename_view() { ... }
        }
    }
}
```

**Test Count Target:** ~15-18 tests (current 9 tests + new edge cases)

**Submodule Organization:**
- `file_name` (6-8 tests)
  - `basename` (4 tests)
  - `extension` (2 tests)
  - `as_ref` (1 test)
- `base_name` (5-6 tests)
  - `try_from_path` (3 tests)
  - `try_from_filename` (2-3 tests)
- `file_name_ref` (2 tests)
  - `basename` (1 test)
  - `as_str` (1 test)

#### Task 2.2: Rename Tests to Behavior-Focused Names

**Mapping (old → new):**

| Old Name | New Location | New Name |
|----------|--------------|----------|
| `should_extract_basename_correctly` | `file_name::basename` | `extracts_stem_from_simple_filename` |
| `should_convert_to_owned_basename` | `file_name_ref::basename` | `extracts_borrowed_basename_view` |
| `should_return_basename_as_owned` | DELETE | Redundant with basename tests |
| `should_return_basename_for_hidden_file` | `file_name::basename` | `returns_some_for_hidden_files_with_extension` |
| `should_convert_from_filename_to_basename` | `base_name::try_from_filename` | `converts_filename_with_extension` |
| `should_try_from_path_to_basename` | `base_name::try_from_path` | `constructs_from_file_stem` |
| `should_try_from_pathbuf_to_basename` | `base_name::try_from_path` | `extracts_from_full_path` |
| `should_return_stem_for_hidden_file` | `file_name::basename` | `returns_none_for_extensionless_hidden_files` |

#### Task 2.3: Split Multi-Assertion Tests

**Example (lines 336-339):**
```rust
// OLD (2 assertions)
#[test]
fn should_return_basename_as_owned() {
    let name = FileName::from("my-note.md".to_owned());
    let base = name.to_basename();
    assert!(base.is_some());
    assert_eq!(base.unwrap().as_str(), "my-note");
}

// NEW (2 separate tests)
mod file_name {
    mod basename {
        #[test]
        fn returns_some_for_valid_filename() {
            let name = FileName::from("my-note.md".to_owned());
            assert!(name.basename().is_some());
        }

        #[test]
        fn extracts_correct_stem() {
            let name = FileName::from("my-note.md".to_owned());
            assert_eq!(name.basename().unwrap().as_str(), "my-note");
        }
    }
}
```

### Phase 3: Verification

#### Task 3.1: Run Full Test Suite
- [ ] `cargo test --lib fs::name` - verify all tests pass
- [ ] `cargo test --doc` - verify doc tests pass
- [ ] `cargo clippy --lib -- -D warnings` - verify no new warnings

#### Task 3.2: Update Documentation
- [ ] Update module-level doc comments to reflect new `basename()` API
- [ ] Add doc examples for `TryFrom<FileName> for BaseName`
- [ ] Document error cases in public API

#### Task 3.3: Final Test Count Verification
**Target:** ~15-18 tests organized in 3 top-level modules, 7 submodules

**Current:** 9 tests flat
**Expected:** 15-18 tests nested

---

## Implementation Strategy

### TDD Approach (Red-Green-Refactor)

**Vertical Slices (One test → One implementation):**

1. **RED:** Write test for new `basename() -> Option<BaseName>` API
2. **GREEN:** Implement by renaming existing `to_basename()`
3. **REFACTOR:** Delete old `basename() -> &str`, update call sites

4. **RED:** Write test for `TryFrom<FileName>` error case
5. **GREEN:** Change `From` to `TryFrom`, add error return
6. **REFACTOR:** Update call sites to handle `Result`

7. **RED:** Write test in new submodule structure (`file_name::basename::extracts_stem_from_simple_filename`)
8. **GREEN:** Move test implementation to new location
9. **REFACTOR:** Repeat for all tests, verifying each passes after move

**Never refactor while RED.** Get to GREEN first.

### Test Organization Reference

**Exemplar:** `lithos-core/src/fs/metadata.rs` test suite
- 23 tests in 4 top-level modules (`fs_times`, `file_metadata`, `dir_metadata`, `fs_metadata`)
- Behavior submodules: `is_match`, `as_file`, `as_dir`, `try_from`, `from_path`
- Behavior-first naming: `returns_true_for_identical_timestamps`
- IDE support: Runnable test groups

### Rust Best Practices Applied

**Apollo Chapter 1:**
- ✅ Prefer explicit APIs over trait magic
- ✅ Return `Option` for fallible operations
- ✅ Prefer `TryFrom` over `From` when conversion can fail

**Apollo Chapter 3:**
- ✅ Keep `Box<str>` for unbounded data
- ✅ Audit unnecessary conversions that encourage allocations

**Apollo Chapter 5:**
- ✅ Test names read like specifications
- ✅ One assertion per test
- ✅ Submodules organize related tests
- ✅ Test public behavior, not implementation

---

## Next Steps

**Order of Operations:**

1. **Audit conversion usage** (Task 1.3) - Informs what to keep/delete
2. **Fix `basename()` API** (Task 1.1) - Critical for Issue 08 consumers
3. **Fix `TryFrom<FileName>` silent failure** (Task 1.2) - Prevents invalid domain objects
4. **Reorganize test suite** (Phase 2) - Apollo compliance, better maintainability
5. **Verify & document** (Phase 3) - Ensure no regressions

**Estimated Effort:** 2-3 hours (with TDD discipline)

**Blocked by:** None - can start immediately

**Blocks:** Issue 08 (`FileInfo` → `FileMetadata` migration) should wait for clean `name.rs` API

---

## Usage Audit Results (2026-05-13)

### FileName Conversions

**Actually Used in Production:**
- `From<String>` → 2 call sites (`config/discovery.rs` lines 2)
- `TryFrom<&Path>` → 7 call sites (`file.rs`, `reader.rs`, `config/paths.rs`, `path.rs` x2, `scanner.rs`)

**Test-only:**
- `From<String>` with `.to_owned()` → 6 call sites in `name.rs` tests

**Decision:**
- ✅ **KEEP** `From<String>` (used in production)
- ✅ **KEEP** `TryFrom<&Path>` (heavily used)
- ❌ **REMOVE** `From<Box<str>>` (unused, redundant with `From<String>`)
- ❌ **REMOVE** `TryFrom<PathBuf>` (unused, `&path.as_path()` pattern preferred)
- ❌ **REMOVE** `From<FileName> for String` (unused)
- ❌ **REMOVE** `From<FileName> for Box<str>` (unused)

### BaseName Conversions

**Actually Used:**
- `TryFrom<&Path>` → 3 call sites (all in `name.rs` tests)
- `From<FileName>` → 1 call site (`name.rs` test with `.clone().into()`)

**Decision:**
- ✅ **KEEP** `TryFrom<&Path>` (used in tests, valid use case)
- ❌ **CHANGE** `From<FileName>` → `TryFrom<FileName>` (fixes silent failure bug)
- ❌ **REMOVE** `From<Box<str>>` (unused)
- ❌ **REMOVE** `From<String>` (unused, redundant with `TryFrom<&Path>`)
- ❌ **REMOVE** `TryFrom<PathBuf>` (unused)
- ❌ **REMOVE** `From<BaseName> for String` (unused)
- ❌ **REMOVE** `From<BaseName> for Box<str>` (unused)

### Basename Method Usage

**`basename() -> &str` (to be deleted):**
- `file.rs` → 3 call sites (tests)
- `schema_processor.rs` → 5 call sites (production: `.basename().to_owned().into_boxed_str()` pattern)
- `identifier.rs` → 1 call site (production: `let name = filename.basename();`)
- `path.rs` → 1 call site (production: `self.filename_ref().map(|f| f.basename())`)
- `name.rs` → 2 call sites (tests)

**Total: 12 call sites to update**

**`to_basename() -> Option<BaseName>` (to be renamed to `basename()`):**
- `name.rs` → 2 call sites (tests)
- `path.rs` → 1 call site (production: `self.filename().and_then(|f| f.to_basename())`)

**Total: 3 call sites affected by rename**

### Migration Strategy for basename() Call Sites

**Pattern 1: `filename.basename()` → `filename.basename_str()` (temporary helper)**
Used where `&str` is needed immediately:
- `identifier.rs`: `let name = filename.basename();` → Keep as-is with new helper
- `path.rs`: `self.filename_ref().map(|f| f.basename())` → Keep returning `BaseNameRef`

**Pattern 2: `.basename().to_owned().into_boxed_str()` → `.basename().map(|b| b.into())`**
Used in `schema_processor.rs` (5 call sites):
```rust
// OLD
.map(|f| f.basename().to_owned().into_boxed_str())

// NEW
.and_then(|f| f.basename()).map(|b| b.into())
// OR (if already Option)
.basename().map(|b| b.into())
```

**Pattern 3: Test assertions `assert_eq!(filename.basename(), "note")` → `assert_eq!(filename.basename_str(), "note")`**
Used in `file.rs` tests (3 call sites)

---

## Status Update

**Status:** ✅ Refactor Complete - All Phases Implemented
**Priority:** HIGH - Must complete before Issue 08 migration
**Complexity:** MEDIUM - API changes + test reorganization

**Completion Date:** 2026-05-13

## Implementation Summary

### Phase 1: API Fixes (COMPLETED)
✅ Fixed `basename()` vs `to_basename()` confusion
- Renamed `basename() -> &str` to `basename_str()`
- Renamed `to_basename()` to `basename()` (returns `Option<BaseName>`)
- Added doc examples for new API

✅ Fixed silent failure bug
- Changed `From<FileName> for BaseName` → `TryFrom<FileName>`
- Returns `Result<BaseName, io::Error>` instead of empty strings
- Added error case tests

✅ Updated 12 call sites across 5 files

**Commit:** `62b1f35a` - Phase 1 complete

### Phase 2: Test Suite Reorganization (COMPLETED)
✅ Reorganized 11 tests into 3 top-level modules, 5 submodules
- `file_name::basename` (3 tests)
- `file_name::basename_str` (2 tests)
- `file_name_ref::basename` (2 tests)
- `base_name::try_from_path` (2 tests)
- `base_name::try_from_filename` (2 tests)

✅ Renamed all tests to behavior-focused names
- Before: `should_extract_basename_correctly`
- After: `returns_some_for_simple_filename`

✅ Split multi-assertion tests into single-assertion tests

**Benefits:**
- IDE can run test groups independently
- Test output shows module hierarchy
- Follows Apollo Chapter 5 pattern (metadata.rs exemplar)

### Phase 3: Remove Dead Conversions (COMPLETED)
✅ Removed 8 unused conversion traits
- FileName: 3 removed (`From<Box<str>>`, `TryFrom<PathBuf>`, `From<FileName> for Box<str>`)
- BaseName: 5 removed (`From<Box<str>>`, `From<String>`, `TryFrom<PathBuf>`, `From<BaseName> for String`, `From<BaseName> for Box<str>`)

✅ Kept 4 necessary conversions
- FileName: `From<String>`, `From<FileName> for String`, `TryFrom<&Path>`
- BaseName: `TryFrom<&Path>`, `TryFrom<FileName>`

**Impact:**
- Net -44 lines of code (156 deleted, 112 added)
- Reduced maintenance burden
- Clearer API surface

**Commit:** `a2fc7c13` - Phase 2+3 complete

## Final Metrics

**Tests:** 1127 unit tests passing (11 fs::name tests in organized structure)
**Lints:** Clippy clean with strict lints (`-D warnings`)
**Code Reduction:** Net -44 lines
**API Clarity:** 2 primary APIs (`basename()`, `basename_str()`), 8 dead conversions removed

## Ready for Issue 08

The `name.rs` refactor is complete. All critical issues from the post-implementation review have been addressed:

1. ✅ API confusion fixed (basename methods consolidated)
2. ✅ Silent failure bug fixed (TryFrom error handling)
3. ✅ Test suite organized per Apollo Chapter 5
4. ✅ Dead conversion traits removed
5. ✅ All tests passing, Clippy clean

**Issue 08 (`FileInfo` → `FileMetadata` migration) can now proceed with a clean `name.rs` foundation.**

---

## Post-Completion Issue: FileName API Encourages Misuse

**Category:** enhancement (API design)
**Summary:** FileName provides too many convenience methods, encouraging use as primary API instead of FilePath/DirPath

### Current Behavior

**Problem:** FileName has too many convenience methods which encourage developers to use FileName directly instead of the path types (FilePath/DirPath) as the primary API:

**FileName API Surface (lithos-core/src/fs/name.rs:14-87):**
- `basename_str() -> &str` - extracts stem as string slice
- `basename() -> Option<BaseName>` - extracts stem as owned type
- `extension() -> Option<&str>` - extracts file extension
- `as_path() -> &Path` - converts to Path reference
- `as_str() -> &str` - string view (minimal API for storage)

**Production Usage Patterns:**
```rust
// identifier.rs:320 - Direct FileName method usage
let name = filename.basename_str();
Self::try_new(name)

// schema_processor.rs:1195 - FileName basename extraction
.filename(path.as_path())
.map(|f| f.basename_str().to_owned().into_boxed_str())

// storage.rs:605 - FileName basename indexing
table.insert(file.name().basename_str(), file.id())
```

**Why This Is Wrong:**
1. **Bypasses Type Safety:** Developers work directly with `FileName`: `filename.basename_str()`, `filename.extension()`. This bypasses FilePath/DirPath type safety guarantees.
2. **Inverted API Hierarchy:** PRD intended FilePath/DirPath as primary access points, name types as storage primitives only.
3. **Encourages Allocations:** Methods like `basename_str()` return `&str` slices that often get immediately allocated (`.to_owned().into_boxed_str()`), defeating zero-copy design.
4. **API Confusion:** Two basename methods (`basename_str()` and `basename()`) create confusion about which to use.

### Desired Behavior

**Primary API:** Developers primarily use path types with extraction methods:
```rust
// Preferred pattern
file_path.basename() -> &str           // Zero-copy extraction
file_path.extension() -> Option<&str>  // Zero-copy extraction
file_path.filename() -> FileName       // When storage needed
```

**Minimal FileName API:**
```rust
impl FileName {
    fn as_str(&self) -> &str           // Only method needed for storage/serialization
    // Remove: basename_str(), basename(), extension(), as_path()
}
```

**Extraction Delegation:** Path types know they have a filename component:
```rust
impl FilePath {
    fn basename(&self) -> &str                // Infallible - FilePath guarantees filename exists
    fn extension(&self) -> Option<&str>       // Zero-copy extraction
    fn filename(&self) -> FileName            // When owned type needed
}
```

**Zero-Copy Borrowing:** Use FileNameRef for borrowed views when possible:
```rust
impl FileNameRef<'_> {
    fn as_str(&self) -> &str                  // Minimal borrowed view
    // Extraction happens at path level, not name level
}
```

### Impact Analysis (GitNexus)

**FileName Usage Scope:**
- **Risk Level:** MEDIUM
- **Direct Call Sites:** 25 matches for `basename_str|\.extension\(` across 13 files
- **Affected Contexts:** `schema`, `vault`, `template`, `fs`

**Key Call Sites:**
1. **`schema/identifier.rs:320`** - Direct `basename_str()` usage
2. **`schema/schema_processor.rs:1195,1260,1298,1386,2053`** - 5× `basename_str()` pattern
3. **`vault/storage.rs:605,756`** - Basename indexing in storage layer
4. **`fs/path.rs:430,444`** - Extension checks (already on path type)

**Path Type Extension Usage (Already Correct Pattern):**
- `vault/model.rs:610` - `path.extension()`
- `vault/processor.rs:41,445,600` - `path.extension()`
- `note/paths.rs:62` - `normalized_path.extension()`
- `fs/reader.rs:522` - `path.extension()`
- `fs/scanner.rs:281` - `path.extension()`
- `fs/validator.rs:223` - `path.extension()`
- `fs/format.rs:77` - `path.extension()`

**Observation:** Extension extraction is already primarily done via path types (8 call sites), not FileName methods. This validates the proposed API direction.

### Acceptance Criteria

- [ ] **Audit FileName usage** to find direct `basename_str()` / `basename()` / `extension()` / `as_path()` calls
  - Grep results: 25 matches across 13 files
  - Concentrated in: `schema/schema_processor.rs` (5×), `vault/storage.rs` (2×), `schema/identifier.rs` (1×)
- [ ] **Migrate call sites** to use FilePath/DirPath extraction methods
  - Pattern: `filename.basename_str()` → `file_path.basename()`
  - Pattern: `filename.extension()` → `file_path.extension()` (many already correct)
- [ ] **Remove convenience methods from FileName**
  - Keep: `as_str()` (minimal storage API)
  - Remove: `basename_str()`, `basename()`, `extension()`, `as_path()`
- [ ] **Add extraction methods to FilePath/DirPath**
  - `FilePath::basename(&self) -> &str` - infallible, FilePath guarantees filename exists
  - `FilePath::extension(&self) -> Option<&str>` - zero-copy extraction
  - Leverage internal `Path` for zero-copy operations
- [ ] **Update FileNameRef API**
  - Keep minimal borrowed view: `as_str()`
  - No extraction methods (extraction happens at path level)
- [ ] **All tests updated**
  - Update test assertions using removed methods
  - Add tests for new FilePath/DirPath extraction methods
- [ ] **Documentation updated**
  - Document correct usage patterns: "Use FilePath/DirPath for extraction, FileName for storage"
  - Add examples showing preferred patterns
  - Deprecation comments on removed methods during transition

### Out of Scope

- **Error type changes** (covered in Issue 08 - `FileInfo` → `FileMetadata` migration)
- **Adding new name types** (DirName, BaseName are already complete)
- **Changing storage format** (Box<str> is correct per Apollo Ch3)

### TDD Plan

Following vertical slice approach (one test → one implementation → repeat):

#### Phase 1: Add FilePath Extraction Methods (New Capability)

**RED:**
```rust
// lithos-core/src/fs/path.rs tests
#[test]
fn file_path_basename_extracts_stem() {
    let path = FilePath::try_from("/vault/notes/my-note.md").unwrap();
    assert_eq!(path.basename(), "my-note");
}

#[test]
fn file_path_extension_extracts_ext() {
    let path = FilePath::try_from("/vault/notes/my-note.md").unwrap();
    assert_eq!(path.extension(), Some("md"));
}
```

**GREEN:**
```rust
impl FilePath {
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        self.0.file_stem()
            .and_then(|s| s.to_str())
            .expect("FilePath guarantees valid UTF-8 filename with stem")
    }

    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|s| s.to_str())
    }
}
```

**REFACTOR:** None yet - first tracer bullet complete.

#### Phase 2: Migrate Call Sites (Behavior Preservation)

**Call Site Priority (12 total):**
1. `schema/identifier.rs:320` (1 site) - Critical path
2. `schema/schema_processor.rs` (5 sites) - High frequency
3. `vault/storage.rs` (2 sites) - Storage indexing
4. Test files (4 sites) - Low risk

**RED (for each call site):**
```rust
// Example: schema/identifier.rs:320
#[test]
fn identifier_from_path_uses_filepath_basename() {
    let path = FilePath::try_from("/vault/notes/my-note.md").unwrap();
    let id = Identifier::try_from_path(&path).unwrap();
    assert_eq!(id.as_str(), "my-note");
}
```

**GREEN:**
```rust
// schema/identifier.rs:320
// OLD: let name = filename.basename_str();
// NEW: let name = path.basename();
pub fn try_from_path(path: &FilePath) -> Result<Self, IdentifierError> {
    let name = path.basename();
    Self::try_new(name)
}
```

**REFACTOR:** Run all tests, verify behavior unchanged.

**Repeat for each call site** (vertical slicing, one site at a time).

#### Phase 3: Remove FileName Convenience Methods (API Restriction)

**RED:**
```rust
// Compilation should fail after removal
#[test]
#[should_not_compile] // Using compiletest_rs or similar
fn filename_basename_str_removed() {
    let name = FileName::from("note.md".to_owned());
    let _ = name.basename_str(); // Should not compile
}
```

**GREEN:**
```rust
// lithos-core/src/fs/name.rs
impl FileName {
    // KEEP
    pub fn as_str(&self) -> &str { &self.0 }

    // REMOVE (comment out first to ensure tests fail)
    // pub fn basename_str(&self) -> &str { ... }
    // pub fn basename(&self) -> Option<BaseName> { ... }
    // pub fn extension(&self) -> Option<&str> { ... }
    // pub fn as_path(&self) -> &Path { ... }
}
```

**REFACTOR:**
- Remove commented-out methods permanently
- Update module documentation
- Add deprecation notes in CHANGELOG

#### Phase 4: Verification (Evidence Before Assertions)

**Verification Checklist:**
- [ ] `mise run test` - All tests pass
- [ ] `mise run lint` - Clippy clean
- [ ] `mise run fmt` - Formatting consistent
- [ ] `rg "\.basename_str\(\)" --type rust` - Zero matches
- [ ] `rg "filename\.extension\(\)" --type rust` - Zero matches (except in name.rs tests)
- [ ] `rg "filename\.as_path\(\)" --type rust` - Zero matches
- [ ] Git diff review - Only intended changes

### Migration Strategy

**Step-by-step (Risk-Mitigated):**

1. **Add new FilePath methods** (non-breaking addition)
   - Tests first (RED), implementation (GREEN), verify (REFACTOR)
   - Commit: "feat(fs): add basename/extension methods to FilePath"

2. **Migrate call sites one file at a time** (preserving behavior)
   - `schema/identifier.rs` → Commit + verify
   - `schema/schema_processor.rs` → Commit + verify
   - `vault/storage.rs` → Commit + verify
   - Test files → Commit + verify

3. **Remove FileName convenience methods** (breaking change, but no remaining usage)
   - Comment out methods, run tests (should fail if any usage remains)
   - Delete methods permanently
   - Commit: "refactor(fs): restrict FileName API to storage primitive"

4. **Update documentation** (clarity)
   - Module-level docs in `fs/name.rs`
   - Usage examples in `fs/path.rs`
   - Commit: "docs(fs): document FilePath as primary extraction API"

**Rollback Plan:** Each commit is independently revertible. If Phase 2 reveals unexpected coupling, can abort before Phase 3.

### Apollo Rust Best Practices Compliance

**Chapter 1 (API Design):**
- ✅ Prefer explicit APIs: FilePath methods are explicit about path-level operations
- ✅ Small interfaces: FileName reduced to single `as_str()` method
- ✅ Return `&str` over `String`: All extraction methods return `&str` (zero-copy)

**Chapter 3 (Performance):**
- ✅ Zero-copy extraction: `basename()` and `extension()` operate on internal `Path` without allocation
- ✅ Avoid unnecessary clones: Path methods leverage existing `Path` reference

**Chapter 4 (Error Handling):**
- ✅ `basename()` is infallible on FilePath (type guarantees filename exists)
- ✅ `extension()` returns `Option` (explicit failure case)

**Chapter 5 (Testing):**
- ✅ Test public behavior: Tests verify extraction correctness through FilePath API
- ✅ Behavior-focused names: `file_path_basename_extracts_stem`
- ✅ One assertion per test: Each test verifies one extraction behavior

### Open Questions for User

1. **FileNameRef handling:** Should `FileNameRef` also lose extraction methods, or keep them for borrowed view use cases?
   - Recommendation: Keep minimal API (`as_str()` only) for consistency

2. **DirName API:** Should DirName follow same pattern (no extraction methods)?
   - Recommendation: Yes, DirPath should be primary API for dirname operations

3. **Transition period:** Should we deprecate methods first before removing?
   - Recommendation: Direct removal is safe since this is pre-1.0 and all usage is internal

4. **BaseName preservation:** Should `FileName::basename() -> Option<BaseName>` stay for owned type conversion?
   - Recommendation: Remove it - use `FilePath::basename()` for `&str`, explicit `BaseName::try_from()` if owned needed

### Related Issues

- **Issue 08:** `FileInfo` → `FileMetadata` migration (blocked by this API cleanup)
- **Post-Implementation Review (2026-05-13):** Identified API confusion with basename methods
- **Phase 1 Refactor (Completed):** Consolidated basename API, removed dead conversions

### Status

**Status:** needs-triage
**Priority:** HIGH - Blocks Issue 08, affects API design across contexts
**Complexity:** MEDIUM - 25 call sites across 13 files, requires careful migration
**Estimated Effort:** 3-4 hours with TDD discipline (4 phases × 1 hour each)
