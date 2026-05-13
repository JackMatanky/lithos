# Task Plan - FS Module Refactoring

Refactor `lithos-core/src/fs/` to minimize unnecessary allocations and ensure lean, performant, and idiomatic Rust code.

## Goals
- [ ] Minimize `PathBuf` and `String` allocations in hot paths.
- [ ] Implement zero-copy patterns for path and name inspection.
- [ ] Maintain 100% test coverage and behavioral parity.

## Context
- **Module**: `lithos-core/src/fs/`
- **Focus Files**: `path.rs`, `entry.rs`, `name.rs`, `scanner.rs`
- **Techniques**: `Cow`, `&Path` vs `PathBuf`, zero-copy enums (`FsPathRef`).

## Phases

### Phase 1: `RelativePath::validate` Optimization (path.rs)
- **Goal**: Remove redundant `to_string_lossy()` and string splitting.
- **Approach**:
    1. Update the `components()` loop to check for `Component::CurDir`.
    2. Remove the `path.to_string_lossy().split(...).any(...)` block.
- **Test**: `tests::relative_path::should_reject_curdir_component` must pass.
- **Status**: ✅ complete — optimized to use `path.to_str()` with fallback to `to_string_lossy()` only for non-UTF-8 paths

### Phase 2: `FsEntry::try_from` Clone Reduction (entry.rs)
- **Goal**: Eliminate redundant `path.clone()` in the success path.
- **Approach**:
    1. Consume `walkdir::DirEntry` via `into_path()`.
    2. Pass the owned `PathBuf` to `FilePath::new`/`DirPath::new`.
    3. Only clone if error reporting requires the path.
- **Test**: `tests::fs_entry::try_from::returns_file_entry_for_walkdir_file`.
- **Status**: ✅ complete — reduced from 3 clones to 1 clone (only for error path)

### Phase 3: Zero-Copy `FsPathRef` and `FsEntry::path_ref()` (entry.rs)
- **Goal**: Stop cloning `PathBuf` just to return an `FsPath`.
- **Approach**:
    1. Define `pub enum FsPathRef<'a> { File(&'a FilePath), Dir(&'a DirPath) }`.
    2. Add `impl<'a> FsPathRef<'a> { pub fn as_path(&self) -> &Path { ... } }`.
    3. Add `FsEntry::path_ref(&self) -> FsPathRef<'_>`.
    4. Update `scanner.rs` to use `path_ref()` in sort loops.
- **Test**: Create new test `tests::fs_entry::path_ref_returns_reference`.
- **Status**: pending

### Phase 4: `Scanner` Hot-Path Optimization (scanner.rs)
- **Goal**: Remove intermediate path clones in traversal helpers.
- **Approach**:
    1. Refactor `to_fs_path` to avoid cloning until the variant is determined.
    2. Update `filter_entry` to return `Option<&Path>` instead of `Option<PathBuf>`.
- **Status**: in progress

## Verification Plan
- Run `cargo test -p lithos-core --lib fs` after each phase.
- Run `mise run verify` at the end of each phase.
- Use `gitnexus_detect_changes()` to verify scope.
