---
title: 05-fs-entry-types
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

Create fs/entry.rs: FsFile (path: FilePath, metadata: FileMetadata), FsDir (path: DirPath, metadata: DirMetadata), FsEntry enum (File/Dir variants).

Unified runtime entities for file system scanning results. FsEntry distinguishes files and directories at the type level.

## Acceptance criteria

- [x] FsFile: path (FilePath), metadata (FileMetadata)
- [x] FsDir: path (DirPath), metadata (DirMetadata)
- [x] FsEntry enum: File(FsFile), Dir(FsDir)
- [x] FsEntry helpers: is_file(), is_dir(), as_file(), as_dir(), path()
- [x] rkyv archived type support
- [x] Tests for entry creation and path access via FsPath
- [x] Update fs/mod.rs exports
- [ ] From<DirEntry> conversions - **DEFERRED** (see implementation notes)

## Blocked by

None - can start immediately (parallel with 01-04)

## Agent Brief

**Category:** enhancement
**Summary:** Create unified `FsFile`, `FsDir`, and `FsEntry` types for filesystem scanning results.

**Current behavior:**
The `FileEntry` type is currently used for scanning results, but it is primarily focused on files. As the project moves toward a unified inode-based architecture, we need a way to represent both files and directories as first-class scanning entities with their respective paths and metadata.

**Desired behavior:**
Implement `FsFile` and `FsDir` structs that compose the new path and metadata types from Issues 01 and 04. Unify them in an `FsEntry` enum. This will serve as the primary output of the `DirScanner` and input for the Vault processor.

**Key interfaces:**
- `FsFile` — composes `FilePath` and `FileMetadata`
- `FsDir` — composes `DirPath` and `DirMetadata`
- `FsEntry` — enum with `File(FsFile)`, `Dir(FsDir)` variants

**Acceptance criteria:**
- [x] `FsEntry` provides a unified `path()` method returning `FsPath`
- [x] Helpers `is_file()`, `is_dir()`, `as_file()`, `as_dir()` are implemented
- [x] Types are `rkyv`-enabled
- [x] Tests verify correct construction and variant access
- [ ] Conversion from `std::fs::DirEntry` - **DEFERRED** (requires base path context)

**Out of scope:**
- Updating `DirScanner` or `FsReader` methods (reserved for Issues 06 and 07)

## Implementation Notes

**File:** `lithos-core/src/fs/entry.rs`

**Implemented Types:**

**FsFile:**
- `path: FilePath` - Validated file path
- `metadata: FileMetadata` - File-specific metadata including size
- Composes Issue 01 (FilePath) and Issue 04 (FileMetadata)

**FsDir:**
- `path: DirPath` - Validated directory path
- `metadata: DirMetadata` - Directory metadata (no size field)
- Composes Issue 01 (DirPath) and Issue 04 (DirMetadata)

**FsEntry enum:**
```rust
pub enum FsEntry {
    File(FsFile),
    Dir(FsDir),
}
```

**Key Methods:**
- `FsFile::new(path, metadata)` - Constructor
- `FsFile::path()` - Returns `&FilePath`
- `FsFile::metadata()` - Returns `&FileMetadata`
- `FsDir::new(path, metadata)` - Constructor
- `FsDir::path()` - Returns `&DirPath`
- `FsDir::metadata()` - Returns `&DirMetadata`
- `FsEntry::is_file()`, `is_dir()` - Variant discrimination
- `FsEntry::as_file()`, `as_dir()` - Safe variant access
- `FsEntry::path() -> FsPath` - Unified path access returning `FsPath` enum

**Design Decision - path() return type:**
- Returns `FsPath` by value (not `&FsPath`)
- `FsPath` is an enum, so returning by value requires cloning the inner path
- Alternative would be to return a reference, but that complicates lifetime management
- Trade-off: slight allocation cost for cleaner API

**Deferred: TryFrom<DirEntry>**
- `std::fs::DirEntry::path()` returns absolute paths
- Our `FilePath`/`DirPath` types require relative paths
- Proper conversion requires a base directory context to strip from absolute paths
- This context is better provided at the `DirScanner` level (Issue 06)
- **Recommendation:** Implement the conversion in Issue 06 when updating `DirScanner`

**Tests:** 7 tests covering:
- FsFile construction and field access
- FsDir construction and field access
- FsEntry variant discrimination (`is_file()`, `is_dir()`)
- Safe variant access (`as_file()`, `as_dir()`)
- Unified path access via `path()` method

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FsFile`, `FsDir`, `FsEntry`

**Status:** ✅ Complete - All core criteria met
**Deferred:** `TryFrom<DirEntry>` conversion (1 criterion) - Better addressed in Issue 06 with full context

## Revision: Phase 2 Update (2026-05-12)

### Need for `try_from_parts()` Method

**Issue:** Issue 06 needs to convert `walkdir::DirEntry` results (absolute `PathBuf` + `std::fs::Metadata`) to `FsEntry`.

**Original Plan:**
- Implement `TryFrom<walkdir::DirEntry>` directly
- Problem: Requires base path context to convert absolute → relative

**Revised Approach:**
- Don't implement `TryFrom<walkdir::DirEntry>` (too complex, requires context)
- Instead: Add `try_from_parts(PathBuf, Metadata) -> Result<FsEntry, ParseError>`
- Accepts absolute paths (matches revised `FilePath`/`DirPath` design)
- `DirScanner` extracts path + metadata from `DirEntry`, then calls `try_from_parts()`

**Changes Needed:**
1. Add `FsEntry::try_from_parts(path: PathBuf, metadata: std::fs::Metadata)`
2. Method creates `FsFile` or `FsDir` based on `metadata.is_dir()`
3. Delegates to `FileMetadata::try_from()` and `DirMetadata::try_from()`

**Implementation:**
```rust
impl FsEntry {
    /// Convert from absolute or relative path + metadata.
    pub(crate) fn try_from_parts(
        path: PathBuf,
        metadata: std::fs::Metadata,
    ) -> Result<Self, ParseError> {
        if metadata.is_dir() {
            let path = DirPath::new(path)?;
            let metadata = DirMetadata::try_from(metadata)?;
            Ok(Self::Dir(FsDir { path, metadata }))
        } else {
            let path = FilePath::new(path)?;
            let metadata = FileMetadata::try_from(metadata)?;
            Ok(Self::File(FsFile { path, metadata }))
        }
    }
}
```

**Rationale:**
- Simpler than `TryFrom<DirEntry>` (no trait implementation complexity)
- Decouples entry conversion from walkdir-specific types
- Works with absolute paths (matches revised path type design)
- `pub(crate)` visibility: internal helper for `DirScanner`

**Agent Task:**
- [ ] Add `FsEntry::try_from_parts(PathBuf, Metadata)` in `fs/entry.rs`
- [ ] Make method `pub(crate)` (internal to `fs` module)
- [ ] Add tests for absolute path → `FsFile` conversion
- [ ] Add tests for absolute path → `FsDir` conversion
- [ ] Add test for error handling (invalid paths, metadata issues)
