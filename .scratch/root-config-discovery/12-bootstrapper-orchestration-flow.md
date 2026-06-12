---
title: 12-bootstrapper-orchestration-flow
category: enhancement
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
- `docs/adr/024-bootstrapper-orchestration.md`

## What to build

Add the `app/Bootstrapper` orchestration flow that wires Discovery to Config without coupling either context to the other.

This slice should make `app/` the only layer that imports both `discovery/` and `config/`, returning a `BootstrapResult` for CLI and future app-level callers.

## Acceptance criteria

- [ ] `Bootstrapper` lives under `lithos-core/src/app/` and is the only component that imports both Discovery and Config.
- [ ] `Bootstrapper` constructs `DiscoveryService` through the self-builder API with platform-resolved global directories.
- [ ] `Bootstrapper` acquires per-invocation context and calls `DiscoveryService::discover(InvocationInput)`.
- [ ] `Bootstrapper` passes `DiscoveryResult` into `config::Builder::from_discovery()` and then calls `build()`.
- [ ] `BootstrapResult { config, report }` is returned to callers.
- [ ] Bootstrapper does not construct `Config` itself; config construction stays inside `config::Builder::build()`.
- [ ] Skipped ceilings and skipped overrides from `DiscoveryReport` are emitted with `tracing::warn!`.
- [ ] `DiscoveryReport` remains available in `BootstrapResult` for CLI rendering without being passed into Config.
- [ ] `discovery/` does not import `config/`; `config/` does not orchestrate `discovery/`.
- [ ] Tests cover the complete Bootstrapper happy path and report propagation.

## Blocked by

- `.scratch/root-config-discovery/11-config-builder-discovery-decoupling.md`
