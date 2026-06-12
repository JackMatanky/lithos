---
title: 13-context-docs-alignment
category: documentation
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

## What to build

Align context documentation with the implemented Discovery, Config, and Bootstrapper architecture.

This slice should remove stale terminology from context docs after the redesign lands, ensuring future agents see the actual boundaries and module names.

## Acceptance criteria

- [ ] `lithos-core/src/discovery/CONTEXT.md` reflects `DiscoveryService`, `DiscoveryMachine`, `FolderProbe`, `InvocationInput`, `DiscoveryResult`, and `DiscoveryReport`.
- [ ] `lithos-core/src/discovery/CONTEXT.md` documents the FS handoff: raw paths at input boundaries, `DirPath` for validated directories, and `FilePath`/`FileNode` plus metadata for discovered marker files.
- [ ] `lithos-core/src/discovery/CONTEXT.md` no longer documents `DiscoveryEngine`, `VaultDiscoveryResult`, `GlobalDiscoveryResult`, or `diagnostics.rs` as current architecture.
- [ ] `lithos-core/src/config/CONTEXT.md` reflects `Builder::from_discovery()`, `build()`, `build_global()`, and `build_vault()`.
- [ ] `lithos-core/src/config/CONTEXT.md` documents that Config consumes Discovery's FS-validated marker handoff and does not re-prove file/directory identity.
- [ ] `lithos-core/src/config/CONTEXT.md` no longer references `config/root.rs`, `ConfigDiscoveryResult`, or Config-owned discovery orchestration.
- [ ] `lithos-core/src/app` module docs mention `Bootstrapper`, `BootstrapResult`, and ADR 024.
- [ ] ADR references are added where useful: ADR 024, `docs/adr/discovery/0001-*`, and `docs/adr/config/0001-*`.
- [ ] `.scratch/root-config-discovery/discovery-redesign-decisions.md` remains as the historical design reference.

## Blocked by

- `.scratch/root-config-discovery/12-bootstrapper-orchestration-flow.md`
