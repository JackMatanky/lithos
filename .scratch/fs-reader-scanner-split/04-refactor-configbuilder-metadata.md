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
- [ ] Global and Vault configuration objects accurately extract timestamps and properties from the provided `FsEntry` metadata.
- [ ] All configuration tests pass successfully.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** refactoring
**Summary:** `ConfigBuilder` does isolated reads using `FsReader::metadata()` to build `RawGlobalConfigView` and `RawVaultConfigView`. However, `config/discovery.rs` explicitly discovers these files and holds their `FsEntry` data.

**Current behavior:**
The builder attempts to fetch `metadata()` again during config generation or update validation.

**Desired behavior:**
Thread the `FileMetadata` from `GlobalDiscovery` and `VaultDiscovery` directly into the `ConfigBuilder` payload states. Eliminate `FsReader::metadata()` invocations in `config/builder.rs`.

## TDD Implementation Plan

1. **RED**: Verify `config_builder` tests and `ConfigLoader` integration tests pass successfully on baseline.
2. **GREEN**: Update `builder.rs` methods (like `load` and private helpers) to accept `FileMetadata` passed directly from the `DiscoveryResult`. Remove `source.metadata()` checks.
3. **REFACTOR**: Ensure global config cache staleness behaves identically using the in-memory metadata.
