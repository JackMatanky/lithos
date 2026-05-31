---
title: 03-refactor-propertybank-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor the `PropertyBankProcessor` test suite to eliminate all calls to `FileReader::metadata()`. While the main processor implementation already avoids these calls by utilizing `FsFile` from discovery, the unit tests still trigger redundant I/O during setup and verification. These must be replaced with manual metadata construction or direct filesystem checks.

NOTE: Per user instructions, the `PropertyBankProcessor` struct itself and its typestate pipeline will NOT be refactored to store metadata separately from `FsFile`, as the current implementation already satisfies the goal of avoiding redundant I/O in the main processing flow.

## Acceptance criteria

- [ ] `PropertyBankProcessor` test fixtures no longer call `FileReader::metadata()`.
- [ ] The `run_analysis_delta_path_returns_bank_with_delta` test no longer calls `FileReader::metadata()`.
- [ ] All related unit and integration tests (`cargo test property_bank`) pass without reliance on the `metadata()` method in `FileReader`.

## Agent Brief

**Category:** refactoring
**Summary:** Remove reliance on `FileReader::metadata()` in `PropertyBankProcessor unit tests.

**Current behavior:**
`PropertyBankProcessor` unit tests (fixtures and specific cases) use `source.metadata()` to initialize or refresh `FsFile` objects.

**Desired behavior:**
Unit tests must construct `FileMetadata` manually or source it directly from the filesystem (e.g. `FsMetadata::from_path`) without using `FileReader` methods, ensuring the processor remains functional when those methods are removed from `FileReader`.

## TDD Implementation Plan

1. **RED**: Run `cargo test property_bank` to ensure current tests pass. (PASSED)
2. **GREEN**: Modify `fixtures::make_fixture` in `property_bank_processor.rs` to construct `FileMetadata` without calling `source.metadata()`. (DONE)
3. **GREEN**: Modify `run_analysis_delta_path_returns_bank_with_delta` to refresh metadata without calling `fixture.source.metadata()`. (DONE)
4. **REFACTOR**: Verify all tests pass and no `source.metadata()` calls remain in the file. (VERIFIED)

## Implementation Notes

- The refactor focused on the unit test suite where the actual `FileReader::metadata()` calls were located.
- `fixtures::make_fixture` now uses `crate::fs::metadata::FsMetadata::from_path(file_path.as_path())` instead of the reader.
- `run_analysis_delta_path_returns_bank_with_delta` now uses `crate::fs::metadata::FsMetadata::from_path(fixture.file.path().as_path())` to reload metadata after modification.
- Main processing logic in `PropertyBankProcessor` was verified to be already using discovery metadata from `FsFile` and required no structural changes to fulfill the requirement of avoiding redundant I/O.
- Verified all 45 related tests pass in the dedicated worktree.
