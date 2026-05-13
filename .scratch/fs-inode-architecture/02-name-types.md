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

**Status:** 🔄 Refactor In Progress - Addressing Post-Review Issues
**Priority:** HIGH - Must complete before Issue 08 migration
**Complexity:** MEDIUM - API changes + test reorganization

**Current Phase:** Usage audit complete, starting TDD implementation
