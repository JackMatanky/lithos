---
title: 01-fs-path-types
category: enhancement
label: ready-for-agent
status: ready-for-agent
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

## Agent Brief

**Category:** enhancement
**Summary:** Create typed filesystem path representations for vault-scoped files and directories.

**Current behavior:**
The codebase uses raw `PathBuf` or generic `RelativePath` types. There is no type-level distinction between a path that represents a file versus one that represents a directory, leading to potential runtime errors when a file path is passed where a directory is expected.

**Desired behavior:**
Introduce `FilePath` and `DirPath` newtypes that wrap `RelativePath`. Both should enforce vault-scoped validation (no `..`, no absolute paths). A unified `FsPath` enum should be provided to handle cases where an entry can be either a file or a directory. Additionally, implement a `ParentDir<'a>` enum for zero-copy parent directory extraction.

**Key interfaces:**
- `FilePath(RelativePath)` — represents a validated file path relative to the vault root
- `DirPath(RelativePath)` — represents a validated directory path relative to the vault root
- `FsPath` — enum with `File(FilePath)` and `Dir(DirPath)` variants
- `ParentDir<'a>` — enum with `Root` and `Path(&'a Path)` variants for zero-copy views

**Acceptance criteria:**
- [ ] `FilePath` and `DirPath` correctly wrap and validate `RelativePath`
- [ ] `FsPath` provides helper methods: `is_file()`, `is_dir()`, `as_file()`, `as_dir()`, and `as_relative()`
- [ ] `ParentDir<'a>` correctly identifies the vault root vs a sub-path without allocating a new `PathBuf`
- [ ] Validation logic rejects paths with `..` components or absolute paths
- [ ] Comprehensive tests cover validation and variant extraction

**Out of scope:**
- Implementing actual directory scanning logic (reserved for Issue 06)
- File format detection
