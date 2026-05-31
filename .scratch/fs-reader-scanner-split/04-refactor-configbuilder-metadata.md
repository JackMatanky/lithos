---
title: 04-refactor-configbuilder-metadata
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Refactor `ConfigBuilder` to eliminate its use of `FsReader::metadata()`. Configuration discovery runs through `DirScanner`, which already captures file metadata. The builder must preserve and consume this metadata through the configuration pipeline to avoid redundant filesystem lookups.

## Acceptance criteria

- [ ] `ConfigBuilder` methods no longer call `FsReader::metadata()`.
- [ ] `DiscoveryEngine` uses `DirScanner` for finding vault configuration files.
- [ ] Global and Vault configuration objects accurately extract timestamps and properties from the provided `FsEntry` metadata.
- [ ] Comprehensive unit test suites established in `config/discovery.rs` and `config/builder.rs`.
- [ ] All configuration tests pass successfully.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `ConfigBuilder` does isolated reads using `FsReader::metadata()` to build `RawGlobalConfigView` and `RawVaultConfigView`. However, `config/discovery.rs` explicitly discovers these files and holds their `FsEntry` data.

**Current behavior:**
The builder attempts to fetch `metadata()` again during config generation or update validation. `DiscoveryEngine` uses `FsReader` for both existence checks and metadata retrieval.

**Desired behavior:**
1. **Metadata Continuity**: Thread the `FileMetadata` from `GlobalDiscovery` and `VaultDiscovery` directly into the `ConfigBuilder` payload states. Populate the `metadata` field of `RawGlobalConfig` and `RawVaultConfig` during ingestion.
2. **Traversal Decoupling**: Migrate `DiscoveryEngine` to use `DirScanner` for finding `.lithos/lithos.toml` to align with the unified discovery architecture.
3. **IO/Traversal Split**: Eliminate `FsReader::metadata()` and `FsReader::exists()` invocations in the `config` module.
4. **Verified Baseline**: Establish unit test suites within each file to protect the refactor.

## TDD Implementation Plan

1. **Phase 1: Baseline Tests (RED)**:
   - Add `mod tests` to `config/discovery.rs` covering FS/DB state combination and `DirScanner` integration.
   - Add `mod tests` to `config/builder.rs` covering `load()` with metadata threading and staleness detection.
2. **Phase 2: Decouple Traversal (GREEN)**:
   - In `discovery.rs`, replace `FsReader` traversal methods with `DirScanner` for vault discovery.
   - Refactor global discovery to use `FsMetadata::from_path` directly.
3. **Phase 4: Metadata Threading (GREEN)**:
   - Update `builder.rs` methods (like `load` and private helpers) to accept `FileMetadata` passed directly from the `DiscoveryResult`.
   - Remove `source.metadata()` checks.
4. **Phase 5: Implementation Tightening (REFACTOR)**:
   - Audit `src/config/` for any remaining calls to `FsReader::metadata()` or `FsReader::exists()`.
   - Ensure global config cache staleness behaves identically using the in-memory metadata.
   - Remove `dead_code` suppressions.
