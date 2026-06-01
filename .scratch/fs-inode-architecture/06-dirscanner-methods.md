---
title: 06-dirscanner-methods
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

Add new typed methods to DirScanner: `paths_typed()` → `Vec<FsPath>` and `entries_typed()` → `Vec<FsEntry>`.

Implement `FsEntry::try_from_parts(PathBuf, Metadata)` to convert walkdir results (absolute paths) to typed entries. Add `FsPath::as_relative(base)` helper for vault-relative conversion at storage boundary.

Keep old methods (`paths()` returning `Vec<PathBuf>`, `entries()` returning `Vec<FileEntry>`) for backward compatibility during Phase 2-3 migration.

## Acceptance criteria

- [x] Add `TryFrom<walkdir::DirEntry> for FsEntry` in `fs/entry.rs` (Issue 05 revision)
- [x] Add `FsPath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [x] Add `FilePath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [x] Add `DirPath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [x] Add `DirScanner::paths_typed(input)` returning `Vec<FsPath>` in `fs/scanner.rs`
- [x] Add `DirScanner::entries_typed(input)` returning `Vec<FsEntry>` in `fs/scanner.rs`
- [x] Keep existing `paths()` and `entries()` methods unchanged
- [x] Tests for walkdir::DirEntry → FsEntry conversion
- [x] Tests for `as_relative()` with base path stripping
- [x] No breaking changes to existing callers

## Blocked by

- [x] 01-fs-path-types (completed)
- [x] 05-fs-entry-types (completed)

## Agent Brief

**Category:** enhancement
**Summary:** Add typed methods to `DirScanner` that return `FsPath` and `FsEntry`, implementing conversion from walkdir's absolute paths.

**Current behavior:**
- `DirScanner::paths()` returns `Vec<PathBuf>` with relative paths
- `DirScanner::entries()` returns `Vec<FileEntry>` (file-only, legacy type)
- `DirScanner` already has `path: PathBuf` field storing the scan root
- walkdir produces absolute paths, scanner currently strips prefix to produce relative paths

**Desired behavior:**
- Add `paths_typed()` and `entries_typed()` methods using new Phase 1 types
- `FilePath`/`DirPath` accept absolute paths (wrap `PathBuf`, not `RelativePath`)
- Conversion to vault-relative paths happens at storage boundary via `as_relative(base)`
- Keep existing methods for backward compatibility (no breaking changes)

**Key interfaces:**

### New conversion trait in `fs/entry.rs`:
```rust
impl TryFrom<walkdir::DirEntry> for FsEntry {
    type Error = ParseError;

    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        let path = entry.into_path();  // Absolute PathBuf
        let metadata = entry.metadata()?;

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

### New helper methods in `fs/path.rs`:
```rust
impl FsPath {
    /// Convert to vault-relative path (requires base path).
    pub fn as_relative(&self, base: &Path) -> Result<RelativePath, ParseError> {
        let path = match self {
            FsPath::File(p) => p.as_path(),
            FsPath::Dir(p) => p.as_path(),
        };

        let rel = path.strip_prefix(base)
            .map_err(|_| ParseError::NotInBasePath)?;
        RelativePath::new(rel)
    }
}

impl FilePath {
    pub fn as_relative(&self, base: &Path) -> Result<RelativePath, ParseError> {
        // Same logic as FsPath
    }
}

impl DirPath {
    pub fn as_relative(&self, base: &Path) -> Result<RelativePath, ParseError> {
        // Same logic as FsPath
    }
}
```

### New DirScanner methods in `fs/scanner.rs`:
```rust
impl DirScanner {
    /// Scan and return typed paths (File or Dir).
    pub fn paths_typed(&self, input: DirScanInput) -> Result<Vec<FsPath>, ParseError> {
        let items = self.scan_internal(input)?;

        items.into_iter()
            .map(|(path, metadata)| {
                if metadata.is_dir() {
                    Ok(FsPath::Dir(DirPath::new(path)?))
                } else {
                    Ok(FsPath::File(FilePath::new(path)?))
                }
            })
            .collect()
    }

    /// Scan and return typed entries with metadata.
    pub fn entries_typed(&self, input: DirScanInput) -> Result<Vec<FsEntry>, ParseError> {
        let walker = self.build_walker(&input);

        walker.into_iter()
            .filter_map(Result::ok)  // Skip I/O errors
            .map(FsEntry::try_from)  // Convert via TryFrom trait
            .collect()
    }

    // Keep existing methods unchanged:
    // pub fn paths(&self, input: DirScanInput) -> Result<Vec<PathBuf>, ParseError>
    // pub fn entries(&self, input: DirScanInput) -> Result<Vec<FileEntry>, ParseError>
}
```

**Important notes:**
1. **Use `TryFrom<walkdir::DirEntry>`**: Clean trait-based conversion, no helper methods needed
2. **`walkdir::DirEntry` provides everything**: `into_path()` gives absolute PathBuf, `metadata()` gives metadata
3. **Absolute paths are OK**: `FilePath`/`DirPath` now wrap `PathBuf` directly (Phase 1 revision). No forced relative conversion.
4. **Storage layer converts**: Vault processor will call `as_relative(vault_root)` when creating `FileView`/`DirView`.
5. **Use `_typed` suffix**: Prevents breaking existing call sites. Phase 4 will rename after migration.
6. **Don't reuse `scan_internal()`**: It produces relative paths. Use walkdir iterator directly for absolute paths.

**Acceptance criteria:**
- [x] `TryFrom<walkdir::DirEntry> for FsEntry` correctly creates `File` or `Dir` variant from absolute path
- [x] `FsPath::as_relative()` correctly strips base prefix and returns `RelativePath`
- [x] Error handling: `as_relative()` returns `ParseError::NotInBasePath` if path is outside base
- [x] `paths_typed()` returns sorted `Vec<FsPath>` (consistent with existing `paths()` behavior)
- [x] `entries_typed()` returns sorted `Vec<FsEntry>` by path (consistent with existing `entries()` behavior)
- [x] All existing tests for `paths()` and `entries()` continue to pass
- [x] New tests verify absolute path handling from walkdir and `as_relative()` conversion

**Out of scope:**
- Updating existing `paths()` or `entries()` methods (keep for backward compat)
- Updating `FileReader` methods (reserved for Issue 07)
- Migrating existing call sites to use new methods (Phase 3)

**Testing strategy:**
- **Unit tests for `try_from_parts()`**: Absolute file path → `FsFile`, absolute dir path → `FsDir`
- **Unit tests for `as_relative()`**: Strip base prefix, error on path outside base
- **Integration tests**: `paths_typed()` and `entries_typed()` produce correct results
- **Regression tests**: Existing `paths()` and `entries()` tests still pass

## Implementation Plan

**Phase 2a: Add conversion helpers (Issue 01/05 updates)**
1. Open `lithos-core/src/fs/path.rs`
2. Change `FilePath(RelativePath)` to `FilePath(PathBuf)` - BREAKING CHANGE
3. Change `DirPath(RelativePath)` to `DirPath(PathBuf)` - BREAKING CHANGE
4. Update `FilePath::as_relative()` signature: `&RelativePath` → `(&Path) -> Result<RelativePath>`
5. Update `DirPath::as_relative()` signature: same as FilePath
6. Add `FsPath::as_relative(&Path) -> Result<RelativePath, ParseError>`
7. Add unit tests for new `as_relative()` (valid prefix stripping, error on outside path)
8. Open `lithos-core/src/fs/entry.rs`
9. Add `TryFrom<walkdir::DirEntry> for FsEntry` implementation
10. Add unit tests using real walkdir entries (requires tempdir)

**Phase 2b: Add DirScanner methods**
1. Open `lithos-core/src/fs/scanner.rs`
2. Add `paths_typed(&self, input) -> Result<Vec<FsPath>, ParseError>`
3. Add `entries_typed(&self, input) -> Result<Vec<FsEntry>, ParseError>`
4. Both methods use existing `scan_internal()` helper
5. Add integration tests for new methods
6. Verify existing tests still pass

**Phase 2c: Verification**
1. Run `cargo test --workspace` - all tests pass
2. Run `cargo clippy -- -D warnings` - no warnings
3. Run `mise run verify` - quality gates pass
4. Document changes in PHASE-2-SUMMARY.md

## Related Files

- `lithos-core/src/fs/path.rs` - Add `as_relative()` helpers
- `lithos-core/src/fs/entry.rs` - Add `try_from_parts()` conversion
- `lithos-core/src/fs/scanner.rs` - Add `paths_typed()` and `entries_typed()`
- `lithos-core/src/fs/error.rs` - May need `NotInBasePath` error variant

## Notes

**Design Decision: Absolute Paths in Infrastructure Layer**

Phase 1 was revised (2026-05-12) to allow `FilePath`/`DirPath` to wrap absolute paths. This decision:
- Eliminates need for base path in `DirScanner` constructor
- Matches `std::fs::DirEntry` and `walkdir::DirEntry` behavior (both return absolute paths)
- Defers relative path conversion to storage layer where vault root is naturally available
- Provides explicit conversion point via `as_relative(base)` method

**Migration Path:**
- Phase 2: Add `*_typed()` methods alongside existing methods
- Phase 3: Update all call sites to use new methods
- Phase 4: Remove old methods, rename `*_typed()` → remove suffix

**Backward Compatibility:**
Existing code using `DirScanner::paths()` or `DirScanner::entries()` is unaffected. New code can adopt `paths_typed()` and `entries_typed()` incrementally.

## Implementation Notes (2026-05-13)

### Summary
Successfully implemented typed directory scanning methods and enhanced the filesystem error hierarchy.

### Key Changes
- **DirScanner Enhancements**: Added `paths_typed()` and `entries_typed()` to `DirScanner`. These methods return the new `FsPath` and `FsEntry` types, preserving absolute paths from the underlying `walkdir` scan.
- **Unified Path Access**: Added `FsPath::as_path()` to provide a consistent way to access the underlying `std::path::Path` from both file and directory variants.
- **Error Diagnostic Improvements**: Introduced `ParseError::NotInBasePath` to explicitly handle and report paths that fall outside the expected vault root. This error was propagated through the `ConfigIngestError` and `SchemaFileError` enums to ensure consistent error reporting across contexts.
- **Refactoring for Quality**: Extracted scanning logic into private helpers (`filter_entry`, `to_fs_path`) to manage complexity and satisfy strict `clippy` nesting limits.
- **Testing**: Added a `typed_scanning` test module in `fs/scanner.rs` to verify the new methods correctly handle absolute paths and produce sorted results.

### Verification Results
- **Tests**: 1112 tests passing in `lithos-core`.
- **Lints**: Code is clean according to `cargo clippy`.
- **Status**: ✅ Completed.
