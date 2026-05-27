---
title: 03-refactor-propertybank-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor `PropertyBankProcessor` to eliminate calls to `FsReader::metadata()`. Property Bank processing must correctly retain and use the `FsMetadata` captured by the initial discovery scan rather than executing redundant filesystem I/O checks.

## Acceptance criteria

- [ ] `PropertyBankProcessor` no longer calls `FsReader::metadata()`.
- [ ] Property Bank views accurately reflect the metadata sourced from `FsEntry`.
- [ ] All related unit and integration tests (`cargo test property_bank`) pass.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `PropertyBankProcessor` does I/O to fetch metadata for staleness comparison. It must instead consume the metadata directly from the discovery payload.

**Current behavior:**
`PropertyBankProcessor::from_discovery_result` or related methods use the reader to query the property bank's file metadata.

**Desired behavior:**
The `PropertyBankDiscovery` object created in `schema/discovery.rs` already holds the `FsEntry` for the property bank. We need to extract `entry.metadata()` and pass that into the typestate pipeline, removing the need for `FsReader` in the comparison step.

## TDD Implementation Plan

1. **RED**: Run tests in `tests/property_bank_processor.rs` to ensure baseline coverage.
2. **GREEN**: Modify `PropertyBankProcessor` to extract `FileMetadata` from the `FsEntry` payload passed in from discovery. Eliminate any use of `source.metadata(...)`.
3. **REFACTOR**: Ensure the metadata passed correctly survives through to the `RawPropertyBankView`. Verify tests pass.
