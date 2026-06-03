---
title: 07-phase-2-environment-config-discovery
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`

## What to build

Implement Environment Config discovery in Phase 2 with deterministic source precedence, multi-format candidate checks, and non-fatal missing behavior.

This slice introduces the **`GlobalConfigProbe`** and the **`DiscoveryEngine::find_global()`** method. It covers mechanical global config source resolution (env-path, XDG, user, system), returning a **`GlobalDiscoveryResult`** containing the mechanical winner and same-tier alternatives, while keeping parsing, stability logic, and classification out of scope.

## Acceptance criteria

- [ ] **Global Discovery Implementation**:
    - [ ] Implement `GlobalConfigProbe` in `discovery/probe.rs` to find environment-level config files.
    - [ ] Add `find_global()` method to `DiscoveryEngine` in `discovery/engine.rs` returning `GlobalDiscoveryResult`.
- [ ] Environment Config source precedence is implemented using ranked **`GlobalSourceType`**: `EnvVar(0)` > `XdgConfig(1)` > `UserConfig(2)` > `SystemConfig(3)`.
- [ ] Each tier supports structured format candidates (`toml`, `json`, `yaml`, `yml`) with deterministic selection via **`discovery::selector::select_candidate()`**.
- [ ] Missing Environment Config at any tier is treated as a non-error and discovery continues to the next tier.
- [ ] Result populates `alternatives: Vec<FoundRootMarker>` for all candidates found at the winning tier.
- [ ] `--no-global-config` suppresses Environment Config lookup entirely.
- [ ] Mis-cased recognized filenames produce corrective warning diagnostics.
- [ ] Unit/integration tests cover source precedence, suppression behavior, no-config behavior, and case-correction diagnostics.

## Agent Brief

**Category:** enhancement
**Summary:** Implement Environment Config discovery with deterministic tier precedence and soft-miss behavior.

**Current behavior:**
Environment Config lookup is partially hardcoded and does not expose consistent precedence semantics across all supported locations/formats.

**Desired behavior:**
Environment Config discovery checks tiers in documented order, finds all candidates at the highest-priority tier, and picks a mechanical winner using `StructuredFileFormat::PRECEDENCE`.

**Key interfaces:**
- `discovery::engine::DiscoveryEngine::find_global()`
- `discovery::policy::GlobalSourceType` (ranked)
- `discovery::engine::GlobalDiscoveryResult` (winner + alternatives)
- `discovery::selector::select_candidate()`
- `--no-global-config` bypass path

**Boundary Note:**
Phase 2 is "Dumb". It must not import `GlobalConfigLocation` or `DiscoveredConfigFile` from the `config` context. Classification happens in the `config/discovery.rs` Consolidator after discovery returns.

**Out of scope:**
- Local (vault) config discovery
- Format stability (History-aware promotion)
- Config classification and parsing

## Blocked by

- `.scratch/root-config-discovery/02-local-candidate-generation.md`
- `.scratch/root-config-discovery/03-candidate-selection-format-stability.md`
- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`
