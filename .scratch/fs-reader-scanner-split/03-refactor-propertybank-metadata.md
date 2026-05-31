---
title: 03-refactor-propertybank-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor the `PropertyBankProcessor` test suite to eliminate all calls to `FsReader::metadata()`. While the main processor implementation already avoids these calls by utilizing `FsFile` from discovery, the unit tests still trigger redundant I/O during setup and verification. These must be replaced with manual metadata construction or direct filesystem checks.

NOTE: Per user instructions, the `PropertyBankProcessor` struct itself and its typestate pipeline will NOT be refactored to store metadata separately from `FsFile`, as the current implementation already satisfies the goal of avoiding redundant I/O in the main processing flow.

## Acceptance criteria

- [ ] `PropertyBankProcessor` test fixtures no longer call `FsReader::metadata()`.
- [ ] The `run_analysis_delta_path_returns_bank_with_delta` test no longer calls `FsReader::metadata()`.
- [ ] All related unit and integration tests (`cargo test property_bank`) pass without reliance on the `metadata()` method in `FsReader`.

## Agent Brief

**Category:** refactoring
**Summary:** Remove reliance on `FsReader::metadata()` in `PropertyBankProcessor unit tests.

**Current behavior:**
`PropertyBankProcessor` unit tests (fixtures and specific cases) use `source.metadata()` to initialize or refresh `FsFile` objects.

**Desired behavior:**
Unit tests must construct `FileMetadata` manually or source it directly from the filesystem (e.g. `FsMetadata::from_path`) without using `FsReader` methods, ensuring the processor remains functional when those methods are removed from `FsReader`.

## TDD Implementation Plan

1. **RED**: Run `cargo test property_bank` to ensure current tests pass.
2. **GREEN**: Modify `fixtures::make_fixture` in `property_bank_processor.rs` to construct `FileMetadata` without calling `source.metadata()`.
3. **GREEN**: Modify `run_analysis_delta_path_returns_bank_with_delta` to refresh metadata without calling `fixture.source.metadata()`.
4. **REFACTOR**: Verify all tests pass and no `source.metadata()` calls remain in the file.
