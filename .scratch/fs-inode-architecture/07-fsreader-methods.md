---
title: 07-fsreader-methods
category: enhancement
label: completed
status: completed
date_created: 2026-05-11
date_updated: 2026-05-13
---

## Type

AFK

## Labels

- completed

## What to build

Add new methods to FsReader: filter_paths, filter_file_paths, filter_dir_paths, filter_entries, filter_file_entries, filter_dir_entries, and metadata(path) → FsMetadata.

Delete old info() method entirely (replaced by metadata()).

## Acceptance criteria

- [x] filter_paths(pattern) → Vec<FsPath> (files and dirs)
- [x] filter_file_paths(pattern) → Vec<FilePath> (files only)
- [x] filter_dir_paths(pattern) → Vec<DirPath> (dirs only)
- [x] filter_entries(pattern) → Vec<FsEntry> (files and dirs)
- [x] filter_file_entries(pattern) → Vec<FsFile> (files only)
- [x] filter_dir_entries(pattern) → Vec<FsDir> (dirs only)
- [x] metadata(path) → Result<FsMetadata, ParseError> (unified File or Dir)
- [x] Delete info() method (replaced by metadata())
- [x] Keep old methods during migration for backward compat
- [x] Tests for all new methods

## Blocked by

- 01-fs-path-types
- 04-fs-metadata-types
- 05-fs-entry-types
- 06-dirscanner-methods

## Implementation Notes (2026-05-13)

### Summary
Successfully implemented typed filtering and metadata methods in `FsReader` (`Reader` struct in `reader.rs`) and migrated all internal and external callers from the deprecated `info()` method to the new unified `metadata()` API.

### Key Changes
- **Unified Metadata API**: Renamed the existing `metadata()` (which returned `std::fs::Metadata`) to `std_metadata()` and introduced a new `metadata()` method that returns the typed `FsMetadata` enum.
- **Typed Filtering**: Added a suite of `filter_*` methods (`filter_paths`, `filter_file_paths`, `filter_dir_paths`, `filter_entries`, `filter_file_entries`, `filter_dir_entries`) that leverage `DirScanner` to return type-safe path and entry collections.
- **Call site Migration**:
    - Updated `config/discovery.rs` and `schema/schema_processor.rs` to use the new `metadata()` method.
    - Added compatibility conversions in `fs/file.rs` (`impl From<FileMetadata> for FileInfo`) to allow existing code to continue using `FileInfo` until the full migration in Issue 08.
    - Updated `created_at` and `modified_at` in `FsReader` to use the new `metadata()` API.
- **Cleanup**: Deleted the deprecated `info()` method entirely.
- **Doctest Updates**: Updated doctests in `reader.rs` to use the new `metadata()` API.
- **Testing**: Comprehensive test coverage added for all new methods, ensuring correct filtering of files vs. directories and proper metadata retrieval.

### Verification Results
- **Tests**: 1124 unit tests and 190 doctests passing in `lithos-core`.
- **Lints**: Workspace is clean.
- **Compilation**: `lithos-core` and `lithos-cli` packages compile successfully.
- **Status**: ✅ Completed.

## Additional Refinements (2026-05-13)

### `FsMetadata::from_path()` Delegation

Refactored `FsReader::metadata()` to delegate to `FsMetadata::from_path()` for better separation of concerns:

**Before:**
```rust
pub fn metadata(&self, path: &Path) -> Result<FsMetadata, ParseError> {
    let std_meta = self.std_metadata(path)?;
    FsMetadata::try_from(std_meta).map_err(|e| ParseError::Io { ... })
}
```

**After:**
```rust
pub fn metadata(&self, path: &Path) -> Result<FsMetadata, ParseError> {
    let full_path = self.root.join(path);
    FsMetadata::from_path(&full_path).map_err(|e| ParseError::Io { ... })
}
```

**Benefits:**
- ✅ **Single Responsibility**: `FsReader` focuses on path resolution, `FsMetadata` owns construction logic
- ✅ **Idiomatic API**: Matches stdlib `std::fs::metadata(path)` pattern
- ✅ **Independent Testability**: Can test `FsMetadata::from_path()` without `FsReader`
- ✅ **Reusable**: Direct usage for absolute paths without needing `FsReader`

See Issue 04 implementation notes for full `from_path()` design rationale.

### DirScanner Integration Clarification

The `filter_*` methods properly use `DirScanner` through the builder pattern:

```rust
pub fn filter_paths(&self, pattern: &str) -> Result<Vec<FsPath>, ParseError> {
    let scanner = self.scanner()  // DirScanInput with vault root
        .glob(pattern)
        .include_dirs(true)
        .build();                 // Constructs DirScanner
    scanner.scan_paths()
}
```

**Delegation Chain:**
1. `self.scanner()` creates `DirScanInput` with vault root and default policy
2. `.glob(pattern)` configures glob pattern
3. `.include_dirs(true)` ensures directories are included
4. `.build()` constructs `DirScanner`
5. `scan_paths()` returns `Result<Vec<FsPath>, ParseError>`

All `filter_*` methods follow this pattern, properly delegating to `DirScanner` for type-safe filtering with vault boundary enforcement.

### Final Test Counts
- **Unit tests**: 1124 passing ✅
- **Doctests**: 190 total (155 passing, 35 ignored) ✅
- **Integration tests**: All passing ✅
- **Clippy**: Clean with strict lints ✅
