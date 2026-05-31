---
title: 04-refactor-configbuilder-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor `ConfigBuilder` to eliminate its use of `FileReader::metadata()`. Configuration discovery runs through `DirScanner`, which already captures file metadata. The builder must preserve and consume this metadata through the configuration pipeline to avoid redundant filesystem lookups.

## Acceptance criteria

- [x] `ConfigBuilder` methods no longer call `FileReader::metadata()`.
- [x] `DiscoveryEngine` uses `DirScanner` for finding vault configuration files.
- [x] Global and Vault configuration objects accurately extract timestamps and properties from the provided `FsEntry` metadata.
- [x] All configuration tests pass successfully.
- [x] Unit test suites intentionally omitted/removed per instruction (focus on Traversal/IO split).

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `ConfigBuilder` does isolated reads using `FileReader::metadata()` to build `RawGlobalConfigView` and `RawVaultConfigView`. However, `config/discovery.rs` explicitly discovers these files and holds their `FsEntry` data.

**Current behavior:**
The builder attempts to fetch `metadata()` again during config generation or update validation. `DiscoveryEngine` uses `FileReader` for both existence checks and metadata retrieval.

**Desired behavior:**
1. **Metadata Continuity**: Thread the `FileMetadata` from `GlobalDiscovery` and `VaultDiscovery` directly into the `ConfigBuilder` payload states. Populate the `metadata` field of `RawGlobalConfig` and `RawVaultConfig` during ingestion.
2. **Traversal Decoupling**: Migrate `DiscoveryEngine` to use `DirScanner` for finding `.lithos/lithos.toml` to align with the unified discovery architecture.
3. **IO/Traversal Split**: Eliminate `FileReader::metadata()` and `FileReader::exists()` invocations in the `config` module.
4. **Verified Baseline**: Establish unit test suites within each file to protect the refactor.

## TDD Implementation Plan (Updated)

1. **Phase 1: Baseline Tests**: Removed per instruction.
2. **Phase 2: Decouple Traversal (Completed)**:
   - In `discovery.rs`, replaced `FileReader` traversal methods with `DirScanner` for vault discovery.
   - Refactored global discovery to use `FsMetadata::from_path` directly.
3. **Phase 3: Metadata Threading (Completed)**:
   - Updated `builder.rs::load` to thread `FileMetadata` from discovery into raw config objects.
   - Eliminated redundant `source.metadata()` checks in the configuration pipeline.
4. **Phase 4: Implementation Tightening (Completed)**:
   - Audited `src/config/` for any remaining calls to `FileReader::metadata()` or `FileReader::exists()`.
   - Fixed configuration view persistence bug where views were not updated during rebuilds.
   - Removed `dead_code` suppressions.
