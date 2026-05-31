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

## Review Findings

- **Status**: The issue is implementable but requires extending the `Note` aggregate to store metadata for comparison.
- **Current state**: `NoteProcessor::check_metadata` is a no-op that relies on a hardcoded `is_stale: true` flag from `VaultProcessor`.
- **Target state**: `Note` aggregate stores its last-known `FileMetadata`. `NoteProcessor` performs pure data comparison between injected metadata and stored metadata.

## Agent Brief

**Category:** refactoring
**Summary:** `NoteProcessor` checks `FsReader::metadata()` for file freshness. `VaultProcessor` already generated the `FsEntry` arrays containing metadata.

**Current behavior:**
`note/processor.rs` executes `check_metadata()` by simply checking an `is_stale: bool` flag passed in `NoteFileInfo`. `VaultProcessor` hardcodes this to `true` when routing.

**Desired behavior:**
`NoteProcessor` typestates should be initialized with the `FileMetadata` originally harvested by `VaultProcessor`/`DirScanner`. `check_metadata()` should perform pure data comparisons against the stored `Note` aggregate in the database cache without calling the filesystem.

**Implementation Note**: `Note` aggregate must be updated to store `FileMetadata` to allow for comparison.

## TDD Implementation Plan

1. **RED**: Add a unit test in `lithos-core/src/note/processor.rs` (under `mod tests`) that verifies `NoteProcessor::check_metadata` correctly identifies staleness by comparing injected `FileMetadata` with the `FileMetadata` stored in the `Note` aggregate.
2. **GREEN**:
   - Add `FileMetadata` field to `Note` aggregate in `lithos-core/src/note/aggregate.rs`.
   - Update `NoteFileInfo` in `lithos-core/src/note/processor.rs` to replace `is_stale: bool` with `metadata: FileMetadata`.
   - Update `NoteProcessor::check_metadata` to compare provided metadata with stored `Note.metadata`.
   - Update `VaultProcessor::route` in `lithos-core/src/vault/processor.rs` to pass `FileMetadata` from the scan.
3. **REFACTOR**: Confirm all `note` module tests pass and ensure no redundant `FsReader::metadata()` calls remain in the pipeline.
