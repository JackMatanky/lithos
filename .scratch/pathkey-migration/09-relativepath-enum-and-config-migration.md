---
title: "Issue 09: RelativePath enum redesign and config type migration"
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-06-01
date_completed: 2026-06-01
---

# Issue 09: RelativePath enum redesign and config type migration

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## AGENT-BRIEF

Implemented the redesign of `RelativePath` as a unified enum and migrated the configuration system to use precise path types.

### Key Changes

1.  **`RelativePath` Redesign**: Converted from a `PathBuf`-backed struct to an enum:
    ```rust
    pub enum RelativePath {
        File(RelativeFilePath),
        Dir(RelativeDirPath),
    }
    ```
2.  **Config Migration**:
    - `Schema.schemas_dir`, `Template.templates_dir`, and `Cache.cache_dir` now use `RelativeDirPath`.
    - `try_new` methods for these types now accept `&std::path::Path` (best practice: avoids unnecessary pass-by-value).
    - Default implementations use `#[expect(clippy::expect_used)]` for literal-backed path construction.
3.  **Removal of Deprecated Types**:
    - `NormalizedPath` alias removed.
    - `TryFrom<RelativePath>` for `FilePath`/`DirPath` removed (replaced by direct variant construction or `DirPath::append_*`).
4.  **Verification**:
    - 1633 tests pass (`cargo test`).
    - `cargo clippy` clean (verified with `--all-targets --all-features`).
    - Rust best practices applied: preferred borrowing over cloning, proper error wrapping, and explicit lint expectations.

## Acceptance criteria

- [x] `RelativePath` is an enum with `File` and `Dir` variants backed by existing declarative types
- [x] Config types (`Schema`, `Template`, `Cache`) use `RelativeDirPath` instead of `RelativePath`
- [x] `as_relative()` methods on `FilePath`, `DirPath`, `FsPath` return the typed enum variant
- [x] `DirPath::append_file()` / `DirPath::append_dir()` are the sole conversion seam from declarative to filesystem paths
- [x] `NormalizedPath` alias is completely removed from codebase
- [x] All existing tests pass with updated types
- [x] No `#[expect(deprecated)]` tags remain for removed aliases

## Implementation Notes

- **API Improvement**: Changed `try_new(PathBuf)` to `try_new(&Path)` in `paths.rs` to satisfy `clippy::needless_pass_by_value` and improve ergonomics for callers passing string literals via `Path::new()`.
- **Rkyv Integration**: `ArchivedRelativePath` updated to handle the enum layout. `as_path()` on the archived variant returns `&Path` by delegating to the archived inner types.
- **Dependency Cleanup**: Removed unused `std::path::PathBuf` imports triggered by the API migration.
