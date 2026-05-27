---
title: 05-refactor-noteprocessor-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor `NoteProcessor` to completely eliminate its reliance on `FsReader::metadata()`. It must correctly pass down and utilize the `FsMetadata` embedded inside the `FsEntry` objects produced by `DirScanner` during discovery.

## Acceptance criteria

- [ ] `NoteProcessor` methods no longer call `FsReader::metadata()`.
- [ ] Internal payload structs in `NoteProcessor` are updated to expect metadata injected from the discovery phase.
- [ ] Integration tests and cache staleness checks continue to correctly process metadata timestamps and sizes.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `NoteProcessor` checks `FsReader::metadata()` for file freshness. `VaultProcessor` already generated the `FsEntry` arrays containing metadata.

**Current behavior:**
`note/processor.rs` executes `check_metadata()` by querying `FsReader::metadata()`.

**Desired behavior:**
`NoteProcessor` typestates should be initialized with the `FileMetadata` originally harvested by `VaultProcessor`/`DirScanner`. `check_metadata()` should perform pure data comparisons against the database cache without calling the filesystem.

## TDD Implementation Plan

1. **RED**: Check `tests/note_reader.rs` to ensure caching and staleness logic is thoroughly verified.
2. **GREEN**: Modify `NoteProcessor` payloads to accept `FileMetadata`. Update `check_metadata` to compare the provided `FileMetadata` against the cached `FileView`, deleting the `FsReader` call.
3. **REFACTOR**: Confirm all `note` module tests pass.
