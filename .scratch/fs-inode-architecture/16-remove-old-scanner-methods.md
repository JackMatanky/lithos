---
title: 16-remove-old-scanner-methods
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-15
date_completed: 2026-05-25
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

---

## Review & Critical Analysis

> *This was generated by AI during triage review.*

### Impact Analysis

**Blast radius for `list_dirs` deletion:**
- Direct callers (d=1, WILL BREAK): `scan_views` in vault processor (only production consumer)
- Indirect (d=2): `discover` — calls `scan_views`
- Transitive (d=3): `process_full` — calls `discover`
- Modules affected: Vault (direct), FsReader (self)
- Risk: **LOW** — 1 direct consumer, 0 execution flows registered in the graph
- The `list_dirs` method has no test coverage of its own (only tested through processor integration tests)

**Blast radius for `list_entries` deletion:**
- Direct callers: 3 test-only consumers in `reader.rs` test module `list_entries`
- No production consumers
- Risk: **LOW** — test-only, behavior preserved via `filter_file_entries`

**Dependency chain:**
- `glob::glob()` function is called **only** in `list_dirs` (reader.rs:313)
- `glob::Pattern` is used separately in `scanner.rs:297` — the `glob` crate remains a production dependency
- The `glob` dev-dependency is used in `tests/architecture.rs` for port/context isolation checks — unrelated

### Critical Issues Found

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | **Depth sort is fallible** | HIGH | Plan says "sort uses `DirPath::as_relative().as_path().components().count()`" but `as_relative()` returns `Result`. `sort_by` closures cannot be fallible. Must pre-compute relative paths + depths before sorting, collecting errors up front. |
| 2 | **Slice 1 RED phase is passive** | MEDIUM | "Rely on existing processor tests as regression guard" skips the RED step. Existing integration tests (`process_full` in `tests/note_reader.rs`) exercise `scan_views` with real fixtures — these should be verified as passing baseline first. Additionally, no test specifically targets `scan_views` in isolation; consider whether one is needed or existing coverage suffices. |
| 3 | **Slice 2 RED/GREEN logic is inverted** | HIGH | Plan says RED = update tests → GREEN = tests pass. This is backward. `filter_file_entries` returns `Vec<FsFile>`, *not* `Vec<FsEntry>`. `FsFile` lacks `FsEntry::filename()` and `FsEntry::path_ref()` — the 3 migrated tests will **fail** on the first assertion change unless assertions are rewritten. Correct sequence: RED = migrate callers → tests FAIL (API mismatch), GREEN = rewrite assertions for `FsFile` API → tests pass, REFACTOR = delete `list_entries`. |
| 4 | **`FsFile` API differences** | MEDIUM | Test `returns_sorted_entries` uses `FsEntry::filename()` (fn on enum) and `path_ref()`. `FsFile` has `path()` → `FilePath` → `.as_path()` and `FilePath::filename()`. Test must swap `entries.first().and_then(FsEntry::filename)` for `entries.first().and_then(|e| e.path().filename())`. |
| 5 | **scanner.rs doc references `list_entries`** | LOW | `src/fs/scanner.rs:9-10` says "`Reader's convenience methods (`filter_dir` and `list_entries`)" — should be updated to "(`filter_entries` and `filter_dir_entries`)" |
| 6 | **`DirMetadata::from` → `.clone()`** | LOW | Plan says "`DirMetadata::from(&metadata)` uses unchanged adapter". Actually `entry.metadata()` returns `&DirMetadata` directly — no `from()` needed. Just `entry.metadata().clone()` since `DirMetadata` derives `Clone`. |
| 7 | **No explicit `glob` dependency removal** | LOW | The `glob` crate production dependency in `Cargo.toml` **cannot** be removed because `scanner.rs` uses `glob::Pattern`. The issue correctly scopes this to removing `glob::glob()` from reader.rs only. Confirmed: `glob::glob()` is called only in `list_dirs` (reader.rs:313). |

### Slice 1 Fallible Sort Pattern

The depth sort migration needs explicit handling for `Result`. Recommended pattern:

```rust
// Pre-compute: convert FsDir entries to (RelativePath, depth, entry) tuples
// Errors bubble up before sorting
let mut dir_entries: Vec<(RelativePath, FsDir)> = source
    .filter_dir_entries("**/*")
    .map_err(|error| VaultFileError::ReadFailed { ... })?
    .into_iter()
    .map(|entry| {
        let relative = entry.path().as_relative(source.root())?;
        Ok((relative, entry))
    })
    .collect::<Result<Vec<_>, VaultFileError>>()?;

// Sort by depth (component count), then path
dir_entries.sort_by(|(rel_a, _), (rel_b, _)| {
    let depth_a = rel_a.as_path().components().count();
    let depth_b = rel_b.as_path().components().count();
    depth_a.cmp(&depth_b).then_with(|| rel_a.as_path().cmp(rel_b.as_path()))
});

// Iterate typed entries
for (relative, entry) in dir_entries {
    let path = normalized_path_from_relative(relative.as_path())?;
    let parent = parent_path(&path)?;
    let parent_id = parent.as_ref().and_then(|key| dir_ids_by_path.get(key)).copied();
    let dir = ScannedDir {
        path: path.clone(),
        view: DirView::new(
            DirId::new(),
            parent_id,
            DirName::new(last_component(relative.as_path())?),
            entry.metadata().clone(),  // Already DirMetadata, no std_metadata call needed
        ),
    };
    dir_ids_by_path.insert(path, dir.view.id());
    dirs.push(dir);
}
```

This eliminates the separate `source.std_metadata()` call (one fewer filesystem access per dir) and handles all errors before sorting.

## Revised Agent Brief

**Category:** enhancement
**Summary:** Remove `list_dirs` and `list_entries` from `FsReader`, migrating `scan_views` to `filter_dir_entries` and 3 test callers to `filter_file_entries`.

**Current behavior:**
- `Reader.list_dirs` uses `glob::glob()` directly (reader.rs:302-342), returns `Vec<PathBuf>` of relative paths, requiring a separate `std_metadata` call per directory from the consumer.
- `Reader.list_entries` wraps `DirScanner::entries()` (reader.rs:354-359), returns `Vec<FsEntry>` (file-only, no `include_dirs(true)`).
- Both methods are absent from the PRD final API spec.

**Desired behavior:**
- `scan_views` uses `filter_dir_entries("**/*")` returning `Vec<FsDir>` with bundled `DirPath` + `DirMetadata`, eliminating the separate metadata filesystem call.
- 3 test-only `list_entries` callers use `filter_file_entries` with correctly typed `FsFile` assertions.
- Both methods and the `glob::glob()` path are deleted.

**Key interfaces:**
- `Reader::filter_dir_entries(&self, pattern: &str) -> Result<Vec<FsDir>, FsError>` — replacement for `list_dirs`, returns typed dirs with `DirPath` (absolute) + `DirMetadata`
- `Reader::filter_file_entries(&self, pattern: &str) -> Result<Vec<FsFile>, FsError>` — replacement for `list_entries`, returns typed files
- `FsDir::path()` → `&DirPath`; `DirPath::as_relative(&self, base: &Path) -> Result<RelativePath, ReadError>` — convert absolute to relative
- `FsDir::metadata()` → `&DirMetadata`; derives `Clone` — replaces `source.std_metadata()` + `DirMetadata::from()`
- `FsFile::path()` → `&FilePath`; `FilePath::filename()` → `Option<FileName>` — replaces `FsEntry::filename()` call
- `scan_views(source: &FsReader) -> Result<(Vec<ScannedDir>, Vec<ScannedFile>), VaultFileError>` — sole production consumer
- `scanner.rs` doc string at `src/fs/scanner.rs:9-10` — references `list_entries`, needs update

**Acceptance criteria:**
- [ ] `scan_views` compiles and passes all tests without `list_dirs`, with depth sort correctly handling the fallible `as_relative()` call
- [ ] `filter_dir_entries` iteration uses `entry.metadata().clone()` instead of separate `source.std_metadata()` + `DirMetadata::from()`
- [ ] Depth sort computed via pre-mapped `(RelativePath, FsDir)` tuples, sorted by component count then path
- [ ] `list_entries` removed; 3 test callers migrated to `filter_file_entries` with assertions rewritten for `FsFile` API (use `FsFile::path().filename()` instead of `FsEntry::filename()`)
- [ ] No `glob::glob()` usage remains in reader.rs (note: `glob::Pattern` in scanner.rs is unrelated and stays)
- [ ] scanner.rs doc updated to remove `list_entries` reference
- [ ] No unused `glob` import artifacts in reader.rs
- [ ] `mise run verify` passes

**Out of scope:**
- Changing `filter_dir_entries` or `filter_file_entries` signatures
- Adding new scanning methods
- Removing the `glob` crate from Cargo.toml (still needed for `glob::Pattern` in scanner.rs)
- Refactoring vault processor beyond the `list_dirs` migration in `scan_views`
- Removing `filter_dir_entries` (it is in the PRD final API)

## Corrected TDD Plan

Use vertical-slice TDD (one behavior at a time), following red-green-refactor with public interface tests. Tests exercise behavior through `FsReader` and `VaultProcessor` public APIs — no mock-based testing.

### Slice 1: Replace `list_dirs` in `scan_views`, then delete it

1. **RED** — Establish baseline: run existing integration tests (`process_full` in `tests/note_reader.rs`) against current code to confirm dir scanning works. These tests exercise `process_full` → `discover` → `scan_views` with real filesystem fixtures. Verify all pass. *(Note: `scan_views` is a private fn; we test through the public `process_full` entry point.)*

2. **GREEN** — Swap the implementation. Key changes in `scan_views`:
   ```rust
   // Before:
   let mut dir_paths = source.list_dirs("**/*")?;
   dir_paths.sort_by(/* component count + path */);
   for relative in dir_paths {
       let path = normalized_path_from_relative(&relative)?;
       let metadata = source.std_metadata(&relative)?;
       // ... DirMetadata::from(&metadata) ...
   }

   // After:
   let mut dir_entries: Vec<(RelativePath, FsDir)> = source
       .filter_dir_entries("**/*")?
       .into_iter()
       .map(|entry| {
           let relative = entry.path().as_relative(source.root())?;
           Ok((relative, entry))
       })
       .collect::<Result<_, VaultFileError>>()?;
   dir_entries.sort_by(|(a, _), (b, _)| {
       a.as_path().components().count()
           .cmp(&b.as_path().components().count())
           .then_with(|| a.as_path().cmp(b.as_path()))
   });
   for (relative, entry) in dir_entries {
       let path = normalized_path_from_relative(relative.as_path())?;
       // No std_metadata call — entry.metadata() is already DirMetadata
       let dir = ScannedDir {
           path: path.clone(),
           view: DirView::new(
               DirId::new(),
               parent_id,
               DirName::new(last_component(relative.as_path())?),
               entry.metadata().clone(),
           ),
       };
       // ...
   }
   ```
   Verify existing integration tests pass.

3. **REFACTOR** — Delete `list_dirs` method. Remove any `glob::glob` related code from reader.rs. Update scanner.rs doc (line 9-10): `list_entries` → `filter_entries`. Verify compile with `cargo check`.

### Slice 2: Replace `list_entries` in tests, then delete it

1. **RED** — Migrate 3 test callers in `mod list_entries` to `filter_file_entries`. Tests **will fail** because:
   - `filter_file_entries` returns `Vec<FsFile>`, not `Vec<FsEntry>`
   - `FsFile` has `.path()` → `&FilePath` (use `.path().as_path()` instead of `.path_ref().as_path()`)
   - `FsFile` has `.path().filename()` instead of `FsEntry::filename()` (via `and_then`)
   - `FsFile::metadata()` returns `&FileMetadata`, not `FsMetadata` enum — `metadata().size()` stays the same on `FileMetadata`

2. **GREEN** — Rewrite test assertions for `FsFile` API:
   - `entries.first().map(|e| e.path_ref().as_path()...)` → `entries.first().map(|e| e.path().as_path()...)`
   - `entries.first().and_then(FsEntry::filename)...` → `entries.first().and_then(|e| e.path().filename())...`
   - `entries.first().is_some_and(|e| e.metadata().as_file()...)` → `entries.first().is_some_and(|e| e.metadata().size() > 0)` (direct access to FileMetadata)
   Verify all 3 tests pass with `cargo test list_entries`.

3. **REFACTOR** — Delete `list_entries` method. Rename test module from `list_entries` to `filter_file_entries` (or merge into existing `filter_file_entries` test module). Verify `cargo check`.

### Slice 3: Verify

1. `cargo test` — all tests pass
2. `cargo clippy --all-targets --all-features --locked -- -D warnings` — no warnings
3. `mise run verify` — all quality gates pass
4. Confirm no `glob::glob()` references remain in `src/` (only `tests/architecture.rs` dev-test is exempt)

### Test checklist

- [ ] Existing `process_full` integration tests in `tests/note_reader.rs` cover `scan_views` with real dir structures (regression guard)
- [ ] Migrated `filter_file_entries` tests prove sorted, file-only, non-empty results with correct metadata
- [ ] No test directly tests deleted methods (they're garbage-collected)
- [ ] All assertions rewritten for `FsFile`/`FsDir` typed API — no `FsEntry` enum access patterns
- [ ] No new test dependencies
