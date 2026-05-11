---
title: 06-dirscanner-methods
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

Add new methods to DirScanner: paths(input) → Result<Vec<FsPath>, ParseError> and entries(input) → Result<Vec<FsEntry>, ParseError>.

Keep old methods (entries() returning Vec<FileEntry>, paths() returning Vec<PathBuf>) for backward compatibility during migration. New methods return typed FsPath/FsEntry enums.

## Acceptance criteria

- [ ] DirScanner.paths(input) returns Vec<FsPath> (File or Dir)
- [ ] DirScanner.entries(input) returns Vec<FsEntry> (File or Dir)
- [ ] Old methods preserved during migration (backward compat)
- [ ] FsPath/FsEntry properly constructed from DirEntry
- [ ] Tests for new methods
- [ ] No breaking changes to existing callers

## Blocked by

- 01-fs-path-types
- 05-fs-entry-types
