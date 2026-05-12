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
