---
title: "Issue 09: RelativePath enum redesign and config type migration"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
date_completed: null
---

# Issue 09: RelativePath enum redesign and config type migration

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Redesign `RelativePath` from a `PathBuf`-backed struct into a unified enum (`RelativePath::File(RelativeFilePath)` / `RelativePath::Dir(RelativeDirPath)`), migrate config types to use precise `RelativeDirPath`/`RelativeFilePath`, and remove the `NormalizedPath` alias.

This implements the three-tier path taxonomy:
- **Filesystem I/O** → `FsPath`, `FilePath`, `DirPath`
- **Display / OS strings** → `RelativePath` enum + `Relative*Path`
- **DB keys** → `PathKey`

### Config migration

Change `config/paths.rs` field types:

```
Schema.schemas_dir: RelativePath      → RelativeDirPath
Template.templates_dir: RelativePath  → RelativeDirPath
Cache.cache_dir: RelativePath         → RelativeDirPath
```

- Replace `default_relative_path()` helper with per-type defaults returning the correct `Relative*Path` type
- Update `property_bank_path()` to use `Path::new(rel_dir.as_str()).join(...)` instead of `.as_path().join(...)`
- Update `PropertyBank` if needed, and any test helpers that construct these types
- The `Cache`, `Template`, `Schema` struct constructors and `try_new` methods accept the new type

### RelativePath enum redesign

Replace the existing struct:

```rust
// OLD: PathBuf-backed struct
pub struct RelativePath(PathBuf);

// NEW: Enum embedding existing declarative types
pub enum RelativePath {
    File(RelativeFilePath),
    Dir(RelativeDirPath),
}
```

- Implement `AsRef<Path>`, `Display`, `rkyv` by delegating to the inner variant
- Update `FilePath::as_relative(base)` → return `RelativePath::File(RelativeFilePath)`
- Update `DirPath::as_relative(base)` → return `RelativePath::Dir(RelativeDirPath)`
- Update `FsPath::as_relative(base)` → return appropriate variant
- Remove `TryFrom<RelativePath> for FilePath` / `DirPath` — these used the old `PathBuf`-backed struct. Replace callers with `DirPath::append_file()` / `DirPath::append_dir()`
- Keep `RelativePathValidator` shared between `RelativeFilePath`, `RelativeDirPath`, `PathKey`
- Update `fs/mod.rs` re-export — same name, new type

### Cleanup

- Remove `pub type NormalizedPath = PathKey` deprecated alias
- Resolve any remaining references to `NormalizedPath`
- Remove the `#[deprecated]` on `ArchivedRelativePath` or any related aliases
- Update/add `rkyv` roundtrip tests for the new enum

### Acceptance criteria

- [ ] `RelativePath` is an enum with `File` and `Dir` variants backed by existing declarative types
- [ ] Config types (`Schema`, `Template`, `Cache`) use `RelativeDirPath` instead of `RelativePath`
- [ ] `as_relative()` methods on `FilePath`, `DirPath`, `FsPath` return the typed enum variant
- [ ] `DirPath::append_file()` / `DirPath::append_dir()` are the sole conversion seam from declarative to filesystem paths
- [ ] `NormalizedPath` alias is completely removed from codebase
- [ ] All existing tests pass with updated types
- [ ] No `#[expect(deprecated)]` tags remain for removed aliases

### Out of scope

- Writing ADR or documentation about the three-tier taxonomy (handled in issue 10)
- Enforcing taxonomy via architecture tests (documentation-driven, not CI-enforced)
- Migration of any remaining `RelativePath` usages outside config and `as_relative()` (covered by the enum — all existing consumers continue to compile under the same name)
