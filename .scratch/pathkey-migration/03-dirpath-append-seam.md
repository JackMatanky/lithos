---
title: "Issue 03: Add DirPath append seam for file/dir fragments"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 03: Add DirPath append seam for file/dir fragments

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Add two generic materialization methods on `DirPath`: `append_file` and `append_dir`, with trait bounds that accept names and relative config path fragments.

## Acceptance criteria

- [ ] `DirPath::append_file<T: FileFragment>(...) -> Result<FilePath, PathError>` exists.
- [ ] `DirPath::append_dir<T: DirFragment>(...) -> Result<DirPath, PathError>` exists.
- [ ] `FileFragment` supports `FileName` and `RelativeFilePath`; `DirFragment` supports `DirName` and `RelativeDirPath`.
- [ ] No direct ad hoc relative-string joins in caller code outside fs seam.
- [ ] Tests cover single-segment and multi-segment append behaviors.

## Blocked by

- `.scratch/pathkey-migration/02-relative-config-types.md`
