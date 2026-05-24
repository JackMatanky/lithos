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
Materialization of fragments (`FileName`, `RelativeFilePath`, `DirName`, `RelativeDirPath`) onto a `DirPath` is exclusively handled by two generic traits and methods. `Relative*Path` types remain passive.

**Key interfaces:**
- `trait FileFragment` and `trait DirFragment`
- `DirPath::append_file<T: FileFragment>(&self, part: &T) -> Result<FilePath, PathError>`
- `DirPath::append_dir<T: DirFragment>(&self, part: &T) -> Result<DirPath, PathError>`

**Acceptance criteria:**
- [ ] `FileFragment` is implemented for `FileName` and `RelativeFilePath`.
- [ ] `DirFragment` is implemented for `DirName` and `RelativeDirPath`.
- [ ] `append_file` and `append_dir` successfully join fragments and return validated `FilePath`/`DirPath` instances.
- [ ] Traceable to PRD/Post-PRD User Stories: #4, #6, #19, #25.

**Out of scope:**
- Schema repository signature migration.
- Applying these methods broadly across the codebase (implemented in slice 04+).
