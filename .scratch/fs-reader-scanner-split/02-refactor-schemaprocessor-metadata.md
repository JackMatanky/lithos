---
title: 02-refactor-schemaprocessor-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor `SchemaProcessor` to eliminate its reliance on `FsReader::metadata()`. Instead of executing redundant filesystem I/O checks during comparison, the processor must pass down and utilize the `FsMetadata` embedded inside the `FsEntry` objects produced by `DirScanner` during discovery.

## Acceptance criteria

- [ ] `SchemaProcessor` methods no longer call `FsReader::metadata()` during standard discovery parsing.
- [ ] Internal payload structs in `SchemaProcessor` expect `metadata: FileMetadata` injected from the discovery phase.
- [ ] `refresh_metadata` behavior handles any explicit file reloading without calling `FsReader::metadata()`, or is rewritten appropriately.
- [ ] Integration tests and cache staleness checks continue to accurately process metadata timestamps.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `SchemaProcessor` uses `source.metadata(...)` extensively to check if files are stale. However, `schema/discovery.rs` already provides `FsEntry` objects which contain this metadata. We must thread the metadata from discovery all the way down instead of requesting it via I/O again.

**Current behavior:**
Functions like `from_discovery_result` and `classify_file_state` in `schema_processor.rs` currently accept `source: &FsReader` primarily so they can query metadata on individual paths.

**Desired behavior:**
`classify_file_state` and related logic should pull the metadata directly from the `FsEntry` stored in `SchemaDiscovery` (e.g. `entry.as_file().unwrap().metadata()`). The `source` parameter can be removed from these metadata-only functions.

## TDD Implementation Plan

1. **RED**: Establish baseline tests in `schema_processor::tests` ensuring staleness logic works correctly.
2. **GREEN**: Remove `source: &FsReader` from `classify_file_state` and `from_discovery_result`. Update the caller (`schema/discovery.rs` and `schema_processor.rs`) to map the `FsEntry`'s `FileMetadata` directly into `InitialScan` or `FoundScan` payloads.
3. **REFACTOR**: Verify `refresh_metadata` either takes the new metadata explicitly or is properly handled. Ensure all schema tests (`cargo test -p lithos-core -- test_schema`) pass with zero regressions.
