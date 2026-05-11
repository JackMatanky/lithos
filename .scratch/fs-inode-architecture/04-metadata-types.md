---
title: 04-fs-metadata-types
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

## Agent Brief

**Category:** enhancement
**Summary:** Separate filesystem metadata into specialized types for files and directories.

**Current behavior:**
The project uses a unified `FileInfo` struct that contains timestamps and file size. However, "size" is not a meaningful or consistent property for directories across all platforms, and using the same struct for both creates ambiguity in the domain model.

**Desired behavior:**
Extract timestamp logic into a reusable `FsTimes` struct. Create distinct `FileMetadata` (which includes `size`) and `DirMetadata` (which does not) types. Unify these in an `FsMetadata` enum to allow for type-safe metadata handling.

**Key interfaces:**
- `FsTimes` — wraps `created_at` and `modified_at` (`Option<SystemTime>`)
- `FileMetadata` — contains `FsTimes`, `size` (u64), and `is_symlink`
- `DirMetadata` — contains `FsTimes` and `is_symlink`
- `FsMetadata` — enum with `File(FileMetadata)`, `Dir(DirMetadata)` variants

**Acceptance criteria:**
- [ ] `FsTimes` implements `is_match()` for staleness detection (comparing timestamps)
- [ ] `FsMetadata` provides helper methods: `is_file()`, `is_dir()`, `as_file()`, `as_dir()`
- [ ] Types are `rkyv`-compatible using `AsUnixTime` for `SystemTime` serialization
- [ ] `DirMetadata` explicitly excludes the `size` field
- [ ] Tests verify correct extraction from `std::fs::Metadata`

**Out of scope:**
- Content hashing (reserved for Vault module)
- Migrating existing `FileInfo` consumers (reserved for Issue 08)
