---
title: 06-dirscanner-methods
category: enhancement
label: ready-for-agent
status: pending
date_created: 2026-05-11
date_updated: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Add new typed methods to DirScanner: `paths_typed()` → `Vec<FsPath>` and `entries_typed()` → `Vec<FsEntry>`.

Implement `FsEntry::try_from_parts(PathBuf, Metadata)` to convert walkdir results (absolute paths) to typed entries. Add `FsPath::as_relative(base)` helper for vault-relative conversion at storage boundary.

Keep old methods (`paths()` returning `Vec<PathBuf>`, `entries()` returning `Vec<FileEntry>`) for backward compatibility during Phase 2-3 migration.

## Acceptance criteria

- [ ] Add `FsEntry::try_from_parts(path: PathBuf, metadata: std::fs::Metadata)` in `fs/entry.rs`
- [ ] Add `FsPath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [ ] Add `FilePath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [ ] Add `DirPath::as_relative(&Path) -> Result<RelativePath, ParseError>` in `fs/path.rs`
- [ ] Add `DirScanner::paths_typed(input)` returning `Vec<FsPath>` in `fs/scanner.rs`
- [ ] Add `DirScanner::entries_typed(input)` returning `Vec<FsEntry>` in `fs/scanner.rs`
- [ ] Keep existing `paths()` and `entries()` methods unchanged
- [ ] Tests for absolute → FsEntry conversion
- [ ] Tests for `as_relative()` with base path stripping
- [ ] No breaking changes to existing callers

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

### New conversion method in `fs/entry.rs`:
```rust
impl FsEntry {
    /// Convert from walkdir results (absolute path + metadata).
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
        let items = self.scan_internal(input)?;

        items.into_iter()
            .map(|(path, metadata)| FsEntry::try_from_parts(path, metadata))
            .collect()
    }

    // Keep existing methods unchanged:
    // pub fn paths(&self, input: DirScanInput) -> Result<Vec<PathBuf>, ParseError>
    // pub fn entries(&self, input: DirScanInput) -> Result<Vec<FileEntry>, ParseError>
}
```

**Important notes:**
1. **Use existing `scan_internal()`**: The private helper already returns `Vec<(PathBuf, std::fs::Metadata)>` with absolute paths. Reuse this in new methods.
2. **Absolute paths are OK**: `FilePath`/`DirPath` now wrap `PathBuf` directly (Phase 1 revision). No forced relative conversion.
3. **Base path is available**: `DirScanner` has `self.path` field containing the scan root. But DON'T use it for conversion yet - let paths remain absolute.
4. **Storage layer converts**: Vault processor will call `as_relative(vault_root)` when creating `FileView`/`DirView`.
5. **Use `_typed` suffix**: Prevents breaking existing call sites. Phase 4 will rename after migration.

**Acceptance criteria:**
- [ ] `FsEntry::try_from_parts()` correctly creates `File` or `Dir` variant from absolute path + metadata
- [ ] `FsPath::as_relative()` correctly strips base prefix and returns `RelativePath`
- [ ] Error handling: `as_relative()` returns `ParseError::NotInBasePath` if path is outside base
- [ ] `paths_typed()` returns sorted `Vec<FsPath>` (consistent with existing `paths()` behavior)
- [ ] `entries_typed()` returns sorted `Vec<FsEntry>` by path (consistent with existing `entries()` behavior)
- [ ] All existing tests for `paths()` and `entries()` continue to pass
- [ ] New tests verify absolute path handling and `as_relative()` conversion

**Out of scope:**
- Updating existing `paths()` or `entries()` methods (keep for backward compat)
- Updating `FsReader` methods (reserved for Issue 07)
- Migrating existing call sites to use new methods (Phase 3)

**Testing strategy:**
- **Unit tests for `try_from_parts()`**: Absolute file path → `FsFile`, absolute dir path → `FsDir`
- **Unit tests for `as_relative()`**: Strip base prefix, error on path outside base
- **Integration tests**: `paths_typed()` and `entries_typed()` produce correct results
- **Regression tests**: Existing `paths()` and `entries()` tests still pass

## Implementation Plan

**Phase 2a: Add conversion helpers (Issue 01/05 updates)**
1. Open `lithos-core/src/fs/path.rs`
2. Add `FsPath::as_relative(&Path) -> Result<RelativePath, ParseError>`
3. Add `FilePath::as_relative(&Path) -> Result<RelativePath, ParseError>`
4. Add `DirPath::as_relative(&Path) -> Result<RelativePath, ParseError>`
5. Add unit tests for `as_relative()` (valid prefix stripping, error on outside path)
6. Open `lithos-core/src/fs/entry.rs`
7. Add `FsEntry::try_from_parts(PathBuf, Metadata) -> Result<Self, ParseError>`
8. Add unit tests for `try_from_parts()` (absolute paths for both file and dir)

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
