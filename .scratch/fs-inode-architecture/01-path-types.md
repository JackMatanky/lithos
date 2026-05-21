---
title: 01-fs-path-types
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

Create fs/path.rs types: FilePath(RelativePath), DirPath(RelativePath), FsPath enum (File/Dir variants), ParentDir<'a> zero-copy view.

Add FilePath and DirPath that wrap RelativePath with vault-scoped validation. Add FsPath enum to represent either file or directory path. Add ParentDir enum for zero-copy parent directory extraction without allocation.

## Acceptance criteria

- [x] FilePath wraps RelativePath (vault-scoped file path)
- [x] DirPath wraps RelativePath (vault-scoped directory path)
- [x] FsPath enum with File(FilePath) and Dir(DirPath) variants
- [x] FsPath helper methods: is_file(), is_dir(), as_file(), as_dir(), as_relative()
- [x] ParentDir<'a> enum with Root and Path(&'a Path) variants
- [x] Tests for validation (no .., no absolute paths) and view extraction
- [x] Update fs/mod.rs exports

## Blocked by

None - can start immediately

## Agent Brief

**Category:** enhancement
**Summary:** Create typed filesystem path representations for vault-scoped files and directories.

**Current behavior:**
The codebase uses raw `PathBuf` or generic `RelativePath` types. There is no type-level distinction between a path that represents a file versus one that represents a directory, leading to potential runtime errors when a file path is passed where a directory is expected.

**Desired behavior:**
Introduce `FilePath` and `DirPath` newtypes that wrap `RelativePath`. Both should enforce vault-scoped validation (no `..`, no absolute paths). A unified `FsPath` enum should be provided to handle cases where an entry can be either a file or a directory. Additionally, implement a `ParentDir<'a>` enum for zero-copy parent directory extraction.

**Key interfaces:**
- `FilePath(RelativePath)` — represents a validated file path relative to the vault root
- `DirPath(RelativePath)` — represents a validated directory path relative to the vault root
- `FsPath` — enum with `File(FilePath)` and `Dir(DirPath)` variants
- `ParentDir<'a>` — enum with `Root` and `Path(&'a Path)` variants for zero-copy views

**Acceptance criteria:**
- [ ] `FilePath` and `DirPath` correctly wrap and validate `RelativePath`
- [ ] `FsPath` provides helper methods: `is_file()`, `is_dir()`, `as_file()`, `as_dir()`, and `as_relative()`
- [ ] `ParentDir<'a>` correctly identifies the vault root vs a sub-path without allocating a new `PathBuf`
- [ ] Validation logic rejects paths with `..` components or absolute paths
- [ ] Comprehensive tests cover validation and variant extraction

**Out of scope:**
- Implementing actual directory scanning logic (reserved for Issue 06)
- File format detection

## Implementation Notes

**File:** `lithos-core/src/fs/path.rs`

**Implemented Types:**
- `FilePath(RelativePath)` - Validated file path with `TryFrom<&str>` and `TryFrom<RelativePath>`
- `DirPath(RelativePath)` - Validated directory path with `TryFrom<&str>` and `TryFrom<RelativePath>`
- `FsPath` enum with `File(FilePath)` and `Dir(DirPath)` variants
- `ParentDir<'a>` enum with `Root` and `Path(&'a Path)` variants

**Key Methods:**
- `FilePath::as_path()` - Returns `&Path` view
- `FilePath::filename()` - Extracts `FileName` (from Issue 02)
- `DirPath::as_path()` - Returns `&Path` view
- `FsPath::is_file()`, `is_dir()`, `as_file()`, `as_dir()`, `as_relative()` - Variant discrimination and access
- `ParentDir` - Zero-copy parent extraction without allocation

**Validation:**
- Rejects absolute paths (via `RelativePath` validation)
- Rejects `..` parent components (via `RelativePath` validation)
- Rejects `.` current directory components (via `RelativePath` validation)
- Rejects platform-specific prefixes (via `RelativePath` validation)
- Rejects empty paths (via `RelativePath` validation)

**Tests:** 22 tests covering:
- Path validation (rejection of `..`, absolute paths, empty paths)
- Variant construction and discrimination
- Parent extraction with `ParentDir`
- Conversion between types
- Edge cases (root paths, deeply nested paths)

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FilePath`, `DirPath`, `FsPath`, `RelativePath`, `AbsolutePath`
- `ParentDir` not exported (internal helper)

**Status:** ✅ Complete - All acceptance criteria met

## Revision: Phase 2 Update (2026-05-12)

### Need for `as_relative()` Method

**Issue:** Issue 06 needs to convert absolute paths from `walkdir::DirEntry` to vault-relative paths for storage layer.

**Original Design:**
- `FilePath(RelativePath)` - constrained to always be relative
- Conversion from absolute would happen at `DirScanner` construction

**Revised Design:**
- `FilePath(PathBuf)` - can be absolute or relative
- Conversion to relative happens explicitly via `as_relative(base: &Path)` method
- Storage layer calls `as_relative()` when creating `FileView`/`DirView`

**Changes Needed:**
1. Add `FilePath::as_relative(&Path) -> Result<RelativePath, ParseError>`
2. Add `DirPath::as_relative(&Path) -> Result<RelativePath, ParseError>`
3. Add `FsPath::as_relative(&Path) -> Result<RelativePath, ParseError>`
4. Update internal representation: `FilePath(PathBuf)` instead of `FilePath(RelativePath)`

**Implementation:**
```rust
impl FilePath {
    pub fn as_relative(&self, base: &Path) -> Result<RelativePath, ParseError> {
        let rel = self.0.strip_prefix(base)
            .map_err(|_| ParseError::NotInBasePath)?;
        RelativePath::new(rel)
    }
}
```

**Rationale:**
- `DirScanner` produces absolute paths (matches `walkdir` behavior)
- No need for base path in scanner constructor (simpler API)
- Explicit conversion point at storage boundary (clearer semantics)
- Flexible: infrastructure types work with any paths, not just vault-relative

**Agent Task:**
- [ ] Change `FilePath(RelativePath)` to `FilePath(PathBuf)` in implementation
- [ ] Change `DirPath(RelativePath)` to `DirPath(PathBuf)` in implementation
- [ ] Add `as_relative()` methods to `FilePath`, `DirPath`, `FsPath`
- [ ] Add tests for `as_relative()` (valid prefix strip, error on outside path)
- [ ] Update existing tests if needed (should still pass)

## Post-Completion Review: Path Type Safety Issues

**Category:** bug / enhancement
**Summary:** FilePath/DirPath have type safety violations that must be fixed

### Issue 1: Infallible `From<PathBuf>` Bypasses Validation

**Current behavior at `path.rs:473` and `path.rs:653`:**
```rust
impl From<PathBuf> for FilePath {
    fn from(path: PathBuf) -> Self {
        Self(path)  // No validation! Path might not refer to a file!
    }
}
```

Both `FilePath` and `DirPath` have infallible `From<PathBuf>` impls that bypass all validation in `FilePath::new()` / `DirPath::new()`. This violates "parse, don't validate" because code can create a `FilePath` wrapping a directory path (type lie).

**Call sites using `From<PathBuf>` (blast radius):**
- `config/paths.rs:457,499,625,647,679` — `DirPath::from(PathBuf::from("/vault"))` in doc examples and tests
- `discovery.rs:670,696` — `DirPath::from(root.path().to_path_buf())` in tests
- `path.rs:975` — `FilePath::from(temp.path().to_path_buf())` in its own test

**Additionally, `DirPath::join_file()` and `DirPath::join_dir()` (lines 610-625) construct FilePath/DirPath directly without validation:**
```rust
pub fn join_file<P>(&self, child: P) -> FilePath {
    FilePath(self.0.join(child))  // Direct construction, bypasses new()
}
```
These are used by `SchemaConfigSpec::directory()` and `SchemaConfigSpec::property_bank()` — which is safe since both operands are already validated, but they still bypass the constructor.

**Desired behavior:**
- Change to `TryFrom<PathBuf>` (falls back to validating constructor)
- OR remove the `From` impl and require explicit construction
- OR document that `From<PathBuf>` is intentionally unchecked (if that's the design choice)
- `join_file`/`join_dir` should use validated construction or be explicitly documented as unchecked

### Issue 2: Filesystem I/O in Constructors (Design Decision)

**Current behavior:** `FilePath::new()` calls `path.is_file()` (line 332), `DirPath::new()` calls `path.is_dir()` (line 513), both performing filesystem I/O.

**Analysis:**
- Requires actual file/dir existence at construction time — surprising for a constructor
- TOCTOU race condition between construction and use
- Error type is `std::io::Error` instead of `PathError` (which already has `NotAFile`/`NotADirectory` variants — see `error.rs:55-65`)
- Contradicts modularity: a lightweight path type shouldn't reach into the filesystem

**Options:**
1. **Remove I/O from constructors** — Make `FilePath`/`DirPath` purely syntactic (like `RelativePath`/`AbsolutePath`). Move filesystem existence checks to `FsReader`/`DirScanner` where I/O belongs.
2. **Keep I/O but fix error type** — Change return type to `Result<Self, PathError>` using existing `PathError::NotAFile` / `PathError::NotADirectory` variants. This is what `PathError` was designed for (`error.rs:54-65`).

**This decision cascades to Issue 1** — if I/O is removed, `From<PathBuf>` becomes less dangerous (just no type-level file/dir distinction but no I/O surprise). If I/O is kept, `From<PathBuf>` is definitely wrong.

### Issue 3: Incomplete Validation for `AbsolutePath`

**Current state:**

| Validation Check | RelativePath | AbsolutePath |
|---|---|---|
| Non-empty | ✅ | ✅ |
| `.` (curdir) | ✅ | ❌ |
| `..` (parentdir) | ✅ | ❌ |
| Platform prefix | ✅ | ❌ |
| Relative/Absolute | ✅ (must be relative) | ✅ (must be absolute) |

`AbsolutePath::validate()` only checks non-empty and absolute (lines 238-252). It should also reject `..`, `.`, and platform prefixes.

**Impact:** Medium. `AbsolutePath` is used throughout the codebase for vault roots, file paths, and schema directory paths. A path like `/vault/../etc/passwd` would pass validation but escape the intended scope.

### Key Interfaces Affected

| Symbol | File | Issue | Blast Radius |
|---|---|---|---|
| `FilePath` struct | `path.rs:317` | Issues 1, 2 | LOW — 4 symbols, Fs module only |
| `DirPath` struct | `path.rs:498` | Issues 1, 2 | LOW — 4 symbols, 2 processes affected |
| `AbsolutePath::validate()` | `path.rs:238` | Issue 3 | Fs module, config |
| `PathError` enum | `error.rs:49` | Target error type | Already has `NotAFile`/`NotADirectory` variants |
| `DirPath::join_file()` | `path.rs:610` | Issue 1 (sub) | Used by `SchemaConfigSpec::property_bank()` |
| `DirPath::join_dir()` | `path.rs:620` | Issue 1 (sub) | Used by `SchemaConfigSpec::directory()` |

**Risk assessment:** LOW (limited to Fs module, no critical execution flows)

### TDD Plan: Vertical Slices

**Slice 1: Fix `AbsolutePath::validate()`**
- RED: Test `AbsolutePath` rejects `..`, `.`, platform prefixes
- GREEN: Add checks to `AbsolutePath::validate()` matching `RelativePath::validate()`
- REFACTOR: Extract common validation into a shared helper

**Slice 2: Switch `FilePath::new()` / `DirPath::new()` error type to `PathError`**
- RED: Tests expect `PathError` variants (`NotAFile`, `NotADirectory`)
- GREEN: Change return type from `Result<Self, std::io::Error>` to `Result<Self, PathError>`
- NOTE: This is independent of the I/O-in-constructor decision

**Slice 3: Decide and fix `From<PathBuf>` on `FilePath` and `DirPath`**
- Decision needed: remove, change to `TryFrom`, or document as unchecked
- If `TryFrom`: update all call sites (config/paths.rs, discovery.rs tests)
- If unchecked: add doc comment explaining the tradeoff
- If remove: fix `join_file`/`join_dir` construction pattern

**Slice 4: Fix `DirPath::join_file()` / `DirPath::join_dir()`**
- RED: Test that `join_file`/`join_dir` validate or are documented
- GREEN: Either route through `new()` or document unchecked construction
- NOTE: Since both operands are already validated (`DirPath` + `RelativePath`), routing through `new()` would add needless I/O — so the fix depends on Slice 5

**Slice 5: Decide on I/O in constructors**
- Decision needed: keep (with PathError) or remove (pure syntactic)
- If keep: fix error types (Slice 2), document semantics
- If remove: strip `is_file()`/`is_dir()` calls, update tests, update `join_file`/`join_dir` to use validated construction

### Acceptance Criteria

- [ ] Decision documented on `From<PathBuf>` strategy (remove/change/unchecked)
- [ ] Decision documented on I/O in constructors (keep or move)
- [ ] `AbsolutePath` validation made consistent with `RelativePath` (`..`, `.`, prefix checks)
- [ ] `FilePath::new()` / `DirPath::new()` error type changed to `PathError`
- [ ] All call sites updated to use proper construction
- [ ] Tests updated for all changes
- [ ] `mise run verify` passes

---

## Maintainer Decisions (Locked)

These decisions supersede earlier open questions in this issue.

1. **Constructor invariant for `FilePath`/`DirPath`: keep filesystem I/O checks for now.**
   - Keep `is_file()` / `is_dir()` checks in constructors.
   - Keep constructor error type as `PathError` variants (`NotAFile`, `NotADirectory`, etc.).
   - Rationale: preserve current domain invariant during this migration.

2. **Remove infallible `From<PathBuf>` for `FilePath` and `DirPath`.**
   - Do not allow unchecked construction via `From<PathBuf>`.
   - Use fallible construction (`TryFrom<PathBuf>` / `new`) only.

3. **Replace `join_file`/`join_dir` with a single `join_path` API returning `FsPath`.**
   - Introduce `DirPath::join_path<P>(&self, child: P) -> FsPath`.
   - `FsPath` becomes the delegation point to `FilePath`/`DirPath` behavior.
   - Migration note: keep `join_file`/`join_dir` temporarily if needed behind deprecation, then remove after call-site migration.

4. **Do not invest in `AbsolutePath`/`RelativePath` hardening in this issue.**
   - Skip additional `AbsolutePath` / `RelativePath` validation work here.
   - These types are being replaced by `FsPath`, `FilePath`, `DirPath`, and `NormalizedPath`.

5. **Verification gate remains project-standard.**
   - Final acceptance requires `mise run verify`.

---

## Implementation Notes (Post-Completion Refactor)

### Applied Changes

1. **Removed unchecked construction paths**
   - Deleted infallible `From<PathBuf>` construction for `FilePath`/`DirPath`.
   - Added fallible conversion path via `TryFrom<PathBuf>`.

2. **Introduced explicit fallible constructors**
   - Renamed constructors to `try_new(...)` for both `FilePath` and `DirPath`.
   - All `TryFrom` implementations now route through `try_new(...)`.

3. **Centralized invariants per type (internal validation)**
   - Added internal-only `validate(&Path)` for `FilePath`.
   - Added internal-only `validate(&Path)` for `DirPath`.
   - `try_new(...)` is now the single public entry point that enforces each type's invariants.

4. **Unified path joining API**
   - Added `DirPath::join_path(...) -> FsPath`.
   - Removed `join_file(...)` and `join_dir(...)` from the API surface.

5. **Scanner conversion trait alignment**
   - Added `impl TryFrom<walkdir::DirEntry> for FsPath` with `PathError`.
   - `DirScanner` now delegates conversion to `FsPath::try_from(entry)` and maps into `ScanError`.
   - Conversion uses `DirEntry::into_path()` ownership transfer rather than borrowing/cloning.

### Call Site Migration Summary

- Migrated call sites from `FilePath::new(...)` / `DirPath::new(...)` to `try_new(...)`.
- Updated scanner and discovery paths to use trait-driven conversion and fallible path construction.
- Updated config and schema tests that used synthetic/nonexistent absolute roots so they create real temp directories/files where required by invariants.

### Verification

- `mise run lint` passes.
- `mise run verify` passes after migration.

### Notes for Next Issue (SchemaConfigSpec)

- `SchemaConfigSpec` still carries relative components plus accessor resolution logic.
- Next refactor should evaluate whether the schema-facing contract should carry execution-ready `DirPath`/`FilePath` values directly, while keeping relocatable relative intent at aggregate config level.
