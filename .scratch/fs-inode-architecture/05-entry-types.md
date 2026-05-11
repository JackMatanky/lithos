---
title: 05-fs-entry-types
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

## Agent Brief

**Category:** enhancement
**Summary:** Create unified `FsFile`, `FsDir`, and `FsEntry` types for filesystem scanning results.

**Current behavior:**
The `FileEntry` type is currently used for scanning results, but it is primarily focused on files. As the project moves toward a unified inode-based architecture, we need a way to represent both files and directories as first-class scanning entities with their respective paths and metadata.

**Desired behavior:**
Implement `FsFile` and `FsDir` structs that compose the new path and metadata types from Issues 01 and 04. Unify them in an `FsEntry` enum. This will serve as the primary output of the `DirScanner` and input for the Vault processor.

**Key interfaces:**
- `FsFile` — composes `FilePath` and `FileMetadata`
- `FsDir` — composes `DirPath` and `DirMetadata`
- `FsEntry` — enum with `File(FsFile)`, `Dir(FsDir)` variants

**Acceptance criteria:**
- [ ] `FsEntry` provides a unified `path()` method returning `&FsPath`
- [ ] Helpers `is_file()`, `is_dir()`, `as_file()`, `as_dir()` are implemented
- [ ] Conversion from `std::fs::DirEntry` or `walkdir::DirEntry` is provided
- [ ] Types are `rkyv`-enabled
- [ ] Tests verify correct construction and variant access

**Out of scope:**
- Updating `DirScanner` or `FsReader` methods (reserved for Issues 06 and 07)
