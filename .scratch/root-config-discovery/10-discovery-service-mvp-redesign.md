---
title: 10-discovery-service-mvp-redesign
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
- `docs/adr/discovery/0001-discovery-service-redesign.md`

## What to build

Refactor `lithos-core/src/discovery` into the MVP `DiscoveryService` design from the discovery redesign decisions.

This slice should deliver one complete discovery execution path through the new public API: construct `DiscoveryService`, call `discover(InvocationInput)`, and receive `(DiscoveryResult, DiscoveryReport)` without involving Config or CLI code.

## Acceptance criteria

- [ ] `DiscoveryService` is the public discovery entry point and uses the self-builder pattern (`default()...build()?`).
- [ ] `InvocationInput` carries only per-call values: `flag_path`, `env_path`, `cwd`, and `ceiling_dirs_raw`.
- [ ] A private `DiscoveryMachine` owns the internal typestate pipeline for the 6-phase discovery process.
- [ ] `DiscoveryResult` replaces the old vault/global split and contains `vault_root`, ordered `vault` markers, and ordered `global` markers.
- [ ] `DiscoveryReport` carries non-fatal process metadata: skipped override, skipped ceilings, and traversal stop reason.
- [ ] `FolderProbe` replaces `VaultRootProbe` and `GlobalRootProbe` by accepting marker patterns at construction.
- [ ] Marker pattern lists and project boundary marker constants live in `policy.rs`; no separate `DiscoveryPolicy` struct remains.
- [ ] Project boundary markers use probe-then-stop semantics.
- [ ] Inaccessible global directories are skipped with `tracing::warn!`; they do not add a `DiscoveryReport` field.
- [ ] `engine.rs` and `diagnostics.rs` responsibilities are removed or migrated.
- [ ] Unit tests cover explicit override, env override, ascending traversal, global fallback, ceiling diagnostics, and boundary marker stopping.

## Blocked by

None - can start immediately.
