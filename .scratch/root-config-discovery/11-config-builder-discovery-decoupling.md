---
title: 11-config-builder-discovery-decoupling
category: refactor
label: ready-for-agent
status: open
date_created: 2026-06-12
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`
- `.scratch/root-config-discovery/discovery-redesign-decisions.md`
- `docs/adr/config/0001-config-builder-decoupling.md`

## What to build

Refactor `config/Builder` so Config no longer orchestrates Discovery.

This slice should make Config consume discovery output through a narrow adapter method, then build global and vault config through Config-owned processors only.

## Acceptance criteria

- [ ] `Builder::from_discovery()` is the only Config entry point that accepts a `discovery::DiscoveryResult`.
- [ ] `Builder::from_discovery()` validates `vault_root` into `VaultRoot`, gets or creates `VaultId`, and stores winning global/vault marker state internally.
- [ ] `Builder::from_discovery()` consumes Discovery's validated FS path/metadata handoff (`DirPath`, `FilePath`/`FileNode`, `FileMetadata`) instead of re-validating plain `PathBuf` marker paths.
- [ ] `Builder::build()` orchestrates `build_global()` and `build_vault()` based on discovered marker presence.
- [ ] `Builder::build_global()` and `Builder::build_vault()` are independently testable and contain no discovery orchestration.
- [ ] `Builder` no longer stores `start_dir`.
- [ ] `config/root.rs` is deleted; `ConfigDiscoveryResult` and `DiscoveredConfigFile` are removed.
- [ ] `config/builder.rs` no longer imports `DiscoveryEngine`, `DiscoveryInput`, `GlobalDiscoveryInput`, or discovery policy types.
- [ ] `ConfigDiscoveryPipeline` keeps its name but receives already-extracted winner marker file nodes/metadata from `Builder::from_discovery()`.
- [ ] Config only reads file contents and queries cached views; file-vs-directory validation and marker metadata capture remain owned by Discovery/FS.
- [ ] Existing staleness behavior remains owned by `ConfigFileProcessor::compare()`; no `BuildMode` is introduced.
- [ ] Tests prove Config can build from vault-only, global-only, and combined discovery outputs.

## Blocked by

- `.scratch/root-config-discovery/10-discovery-service-mvp-redesign.md`
