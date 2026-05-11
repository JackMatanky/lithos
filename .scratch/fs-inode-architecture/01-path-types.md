---
title: 01-fs-path-types
category: enhancement
label: needs-triage
status: pending
date_created: 2026-05-11
---

## Type

AFK

## Labels

- needs-triage

## What to build

Create fs/path.rs types: FilePath(RelativePath), DirPath(RelativePath), FsPath enum (File/Dir variants), ParentDir<'a> zero-copy view.

Add FilePath and DirPath that wrap RelativePath with vault-scoped validation. Add FsPath enum to represent either file or directory path. Add ParentDir enum for zero-copy parent directory extraction without allocation.

## Acceptance criteria

- [ ] FilePath wraps RelativePath (vault-scoped file path)
- [ ] DirPath wraps RelativePath (vault-scoped directory path)
- [ ] FsPath enum with File(FilePath) and Dir(DirPath) variants
- [ ] FsPath helper methods: is_file(), is_dir(), as_file(), as_dir(), as_relative()
- [ ] ParentDir<'a> enum with Root and Path(&'a Path) variants
- [ ] Tests for validation (no .., no absolute paths) and view extraction
- [ ] Update fs/mod.rs exports

## Blocked by

None - can start immediately
