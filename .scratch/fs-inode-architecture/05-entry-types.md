---
title: 05-fs-entry-types
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

Create fs/entry.rs: FsFile (path: FilePath, metadata: FileMetadata), FsDir (path: DirPath, metadata: DirMetadata), FsEntry enum (File/Dir variants).

Unified runtime entities for file system scanning results. FsEntry distinguishes files and directories at the type level.

## Acceptance criteria

- [ ] FsFile: path (FilePath), metadata (FileMetadata)
- [ ] FsDir: path (DirPath), metadata (DirMetadata)
- [ ] FsEntry enum: File(FsFile), Dir(FsDir)
- [ ] FsEntry helpers: is_file(), is_dir(), as_file(), as_dir(), path()
- [ ] rkyv archived type support
- [ ] From<DirEntry> conversions
- [ ] Tests for entry creation and path access via FsPath
- [ ] Update fs/mod.rs exports

## Blocked by

None - can start immediately (parallel with 01-04)
