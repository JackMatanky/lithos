# Findings - FS Module Refactoring

## Research & Discoveries

### Current State Analysis (2026-05-13)
- `RelativePath::validate` uses `path.to_string_lossy().split(['/', '\\'])` which allocates if the path is invalid UTF-8.
- `FsEntry::try_from(walkdir::DirEntry)` clones `path` multiple times unnecessarily.
- `FsEntry::path()` returns an owned `FsPath`, forcing a `PathBuf` clone.
- `FileName::try_from(&Path)` always creates a `Box<str>`.
- `DirScanner` helpers clone paths before the filter check.

### Proposed Interface Changes
#### `FsPathRef`
```rust
pub enum FsPathRef<'a> {
    File(&'a FilePath),
    Dir(&'a DirPath),
}

impl FsEntry {
    pub fn path_ref(&self) -> FsPathRef<'_> { ... }
}
```

### Constraints
- Must remain compatible with `rkyv` serialization requirements for stored types.
- Must not break `RelativePath` safety invariants.
