---
title: 07-fsreader-methods
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

Add new methods to FsReader: filter_paths, filter_file_paths, filter_dir_paths, filter_entries, filter_file_entries, filter_dir_entries, and metadata(path) → FsMetadata.

Delete old info() method entirely (replaced by metadata()).

## Acceptance criteria

- [ ] filter_paths(pattern) → Vec<FsPath> (files and dirs)
- [ ] filter_file_paths(pattern) → Vec<FilePath> (files only)
- [ ] filter_dir_paths(pattern) → Vec<DirPath> (dirs only)
- [ ] filter_entries(pattern) → Vec<FsEntry> (files and dirs)
- [ ] filter_file_entries(pattern) → Vec<FsFile> (files only)
- [ ] filter_dir_entries(pattern) → Vec<FsDir> (dirs only)
- [ ] metadata(path) → Result<FsMetadata, ParseError> (unified File or Dir)
- [ ] Delete info() method (replaced by metadata())
- [ ] Keep old methods during migration for backward compat
- [ ] Tests for all new methods

## Blocked by

- 01-fs-path-types
- 04-fs-metadata-types
- 05-fs-entry-types
- 06-dirscanner-methods
