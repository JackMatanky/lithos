---
title: 04-fs-metadata-types
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

Create fs/metadata.rs: FsTimes (created_at, modified_at), FileMetadata (times, size, is_symlink), DirMetadata (times, is_symlink), FsMetadata enum (File/Dir variants).

Split from existing FileInfo. FsMetadata provides type-level distinction between files and directories.

## Acceptance criteria

- [ ] FsTimes with created_at, modified_at (Option<SystemTime>, rkyv with AsUnixTime)
- [ ] FsTimes::is_match() for staleness detection
- [ ] FileMetadata: times (FsTimes), size (u64), is_symlink (bool)
- [ ] DirMetadata: times (FsTimes), is_symlink (bool) - no size (not meaningful for dirs)
- [ ] FsMetadata enum: File(FileMetadata), Dir(DirMetadata)
- [ ] FsMetadata helpers: is_file(), is_dir(), as_file(), as_dir()
- [ ] rkyv archived type support
- [ ] Tests for timestamp extraction, metadata construction
- [ ] Update fs/mod.rs exports

## Blocked by

None - can start immediately
