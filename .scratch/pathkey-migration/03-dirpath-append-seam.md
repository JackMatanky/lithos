---
title: "Issue 03: Add DirPath append seam for file/dir fragments"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 03: Add DirPath append seam for file/dir fragments

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Add two generic materialization methods on `DirPath`: `append_file` and `append_dir`, with trait bounds that accept names and relative config path fragments.

## Agent Brief

**Category:** enhancement
**Summary:** Centralize relative-to-absolute materialization through generic `DirPath` append methods.

**Current behavior:**
Callers manually join strings or relative paths to construct absolute filesystem targets, bypassing structural validation and encouraging ad hoc `PathBuf` pushing.

**Desired behavior:**
Materialization of fragments onto a `DirPath` is exclusively handled by two generic traits and methods. `Relative*Path` types remain passive; the operational seam lives strictly on `DirPath`.

**Key interfaces:**

1. **Traits:**
```rust
pub trait FileFragment {
    fn as_str(&self) -> &str;
}

pub trait DirFragment {
    fn as_str(&self) -> &str;
}
```

2. **Implementations:**
- `FileName` and `RelativeFilePath` implement `FileFragment`.
- `DirName` and `RelativeDirPath` implement `DirFragment`.

3. **DirPath Methods:**
```rust
impl DirPath {
    pub fn append_file<T: FileFragment>(&self, part: &T) -> Result<FilePath, PathError> {
        // Implementation joining self with part.as_str() and validating the result
    }

    pub fn append_dir<T: DirFragment>(&self, part: &T) -> Result<DirPath, PathError> {
        // Implementation joining self with part.as_str() and validating the result
    }
}
```

**Acceptance criteria:**
- [ ] `FileFragment` and `DirFragment` traits are defined.
- [ ] Implementations are provided for `FileName`, `RelativeFilePath`, `DirName`, and `RelativeDirPath`.
- [ ] `DirPath::append_file` and `DirPath::append_dir` are implemented and return validated `FilePath`/`DirPath` instances.
- [ ] Tests cover single-segment and multi-segment append behaviors.

**Out of scope:**
- Modifying callers outside of the `fs` module to use this seam (done in slice 04+).
