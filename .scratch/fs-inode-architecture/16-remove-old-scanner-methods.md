---
title: 16-remove-old-scanner-methods
category: enhancement
label: ready-for-agent
status: ready-for-agent
date_created: 2026-05-15
---

## Type

AFK

## Labels

- enhancement
- ready-for-agent

## What to build

Remove `list_dirs` and `list_entries` from `FsReader`, migrate their callers to `filter_dir_entries` and `filter_file_entries` respectively, and eliminate the final `glob::glob` code path that bypasses `DirScanner`. Completes alignment with PRD §Reader and Scanner API Changes.

## Acceptance criteria

- [ ] `scan_views` in vault processor uses `filter_dir_entries` instead of `list_dirs`, reads metadata from `FsDir.metadata()` instead of separate `std_metadata` call
- [ ] No remaining doc, test, or production references to `list_dirs` or `list_entries`
- [ ] `DirScanner` is the sole directory scanning path — no `glob::glob` usage in reader.rs
- [ ] All existing tests pass (behavior preserved)
- [ ] `mise run verify` passes

## Blocked by

- 12-phase-4-cleanup (Phase 4b `_typed` suffix removal + `paths()` cleanup)

---

## Agent Brief

**Category:** enhancement
**Summary:** Remove `list_dirs` and `list_entries` methods from `FsReader`, migrating all callers to `filter_dir_entries` and `filter_file_entries`.

**Current behavior:**
Two `FsReader` methods bypass or duplicate the `DirScanner`/`DirScanInput` scanning architecture:
- `list_dirs` (lines 302-342) uses `glob::glob()` directly with a hand-rolled loop for directory enumeration. It returns relative `Vec<PathBuf>` requiring a separate `std_metadata` call by the consumer.
- `list_entries` (lines 354-359) wraps `DirScanner::entries()` but returns `Vec<FsEntry>` (file-only by default — no `include_dirs(true)`).
- Both methods are not present in the PRD final API specification.

**Desired behavior:**
- `scan_views` in vault processor uses `filter_dir_entries("**/*")` which returns `Vec<FsDir>` — typed `DirPath` + `DirMetadata` bundled together, eliminating the separate metadata read.
- `list_entries`'s 3 test-only callers migrate to `filter_file_entries`.
- Both methods are deleted.
- No `glob::glob` imports remain in reader.rs.

**Key interfaces:**
- `FsReader::list_dirs(&self, pattern: &str) -> Result<Vec<PathBuf>, FsError>` — delete; callers use `filter_dir_entries` instead
- `FsReader::filter_dir_entries(&self, pattern: &str) -> Result<Vec<FsDir>, FsError>` — existing, already returns typed dirs with metadata
- `FsReader::list_entries(&self, pattern: &str) -> Result<Vec<FsEntry>, FsError>` — delete; callers use `filter_file_entries` instead
- `FsReader::filter_file_entries(&self, pattern: &str) -> Result<Vec<FsFile>, FsError>` — existing, already returns typed files with metadata
- `vault/processor.rs:scan_views()` — sole production consumer of `list_dirs`. Iterates dirs, sorts by depth, converts relative paths, reads `std_metadata` separately. Must adapt to `filter_dir_entries` return type (absolute `DirPath` → `.as_relative(source.root())?` for relative; metadata from `FsDir::metadata()`).
- `DirPath::as_relative(&self, base: &Path) -> Result<RelativePath, ReadError>` — existing method, used by `filter_file_entries` consumer in same function for file path relative conversion.

**Acceptance criteria:**
- [ ] `scan_views` compiles and passes all tests without `list_dirs`
- [ ] `filter_dir_entries` iteration uses `FsDir.metadata()` instead of separate `std_metadata` per dir
- [ ] Depth sort logic preserved (uses `DirPath::as_relative().as_path().components().count()`)
- [ ] `list_entries` removed with all 3 former test callers migrated to `filter_file_entries`
- [ ] No `glob::glob` usage remains in reader.rs
- [ ] `mise run verify` passes

**Out of scope:**
- Changing `filter_dir_entries` or `filter_file_entries` signatures
- Adding new scanning methods
- Refactoring the vault processor beyond the `list_dirs` migration
- Removing `filter_dir_entries` (it is in the PRD final API)

## TDD Implementation Plan

Use vertical-slice TDD (one behavior at a time), with public-interface tests and no speculative code.

### Slice 1: Replace `list_dirs` in processor, then delete it

1. RED: Write an integration test through `scan_views` that proves dir metadata works via `filter_dir_entries` (or rely on existing processor tests as regression guard)
2. GREEN: Swap `source.list_dirs("**/*")` → `source.filter_dir_entries("**/*")` in `scan_views`:
   - Depth sort uses `entry.path().as_relative(source.root())?.as_path().components().count()`
   - Metadata reads from `entry.metadata()` instead of `source.std_metadata(&relative)`
   - `DirMetadata::from(&metadata)` uses `std::fs::Metadata` → unchanged adapter
   - Relative path for `last_component()` and `normalized_path_from_relative()` comes from `.as_relative(source.root())?`
3. REFACTOR: Delete `list_dirs` method. Remove `glob::glob` import from reader.rs. Verify compile.

### Slice 2: Replace `list_entries` in tests, then delete it

1. RED: Update 3 test functions to call `filter_file_entries` instead of `list_entries`. Assert `Vec<FsFile>` return — use `.first()`, `.get(1)`, `.len()` as before (same behavioral shape).
2. GREEN: Tests pass because both methods return sorted, file-only results with metadata.
3. REFACTOR: Delete `list_entries` method. Update any doc examples referencing it.

### Slice 3: Verify

1. Run `mise run verify` — all quality gates pass.
2. Confirm PRD alignment: final `FsReader` scanning API matches spec (`filter_paths`, `filter_file_paths`, `filter_dir_paths`, `filter_entries`, `filter_file_entries`, `filter_dir_entries` — no `list_dirs` or `list_entries`).

### Test checklist

- Behavioral equivalence: tests should prove `filter_dir_entries` returns same data shape as `list_dirs` did (sorted dirs with metadata)
- Regression: vault processor integration tests cover `scan_views` end-to-end
- No new test dependencies beyond tempfile (standard test fixture pattern)
