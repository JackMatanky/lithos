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

Implement mechanical Environment Config path discovery in Phase 2 with deterministic source precedence, multi-format candidate checks, and non-fatal missing behavior.

This slice implements the previously stubbed **`GlobalConfigProbe`** and **`DiscoveryEngine::find_global()`** method. It covers mechanical Environment Config path resolution (`LITHOS_CONFIG_FILE`, XDG, user, system), returning a **`GlobalDiscoveryResult`** containing the mechanical winner and non-winning same-tier alternatives, while keeping parsing, format stability, and Config-owned classification out of Discovery.

## Acceptance criteria

- [ ] **Global Discovery Implementation**:
    - [ ] Implement `GlobalConfigProbe` in `discovery/probe.rs` to find Environment Config path candidates without importing Config-owned types.
    - [ ] Replace the stubbed `DiscoveryEngine::find_global()` in `discovery/engine.rs` with an implementation returning `GlobalDiscoveryResult`.
    - [ ] Add `GLOBAL_MARKER_FILES` or an equivalent Discovery-owned marker pattern constant for `lithos.{toml,json,yaml,yml}` under generated global base directories.
- [ ] Environment Config source precedence is implemented using ranked **`GlobalSourceType`**: `EnvVar(0)` > `XdgConfig(1)` > `UserConfig(2)` > `SystemConfig(3)`. The environment override name is **`LITHOS_CONFIG_FILE`**.
- [ ] Each tier supports structured format candidates (`toml`, `json`, `yaml`, `yml`) with deterministic selection via **`discovery::selector::select_candidate()`**.
- [ ] Missing Environment Config at any tier is treated as a non-error and discovery continues to the next tier.
- [ ] Result populates `alternatives: Vec<FoundRootMarker>` with non-winning candidates found at the winning tier; the selected `marker` is not duplicated in `alternatives`.
- [ ] Core suppression input for `--no-global-config` suppresses Environment Config lookup entirely. Prefer a dedicated `GlobalDiscoveryInput<'_>` over overloading vault-specific `DiscoveryInput<'_>`.
- [ ] Mis-cased recognized filenames produce corrective Discovery-owned warning diagnostics.
- [ ] `GlobalDiscoveryResult.warnings` uses a Discovery-owned warning shape (`DiscoveryWarning` or a dedicated global warning type), not `VaultDiscoveryWarning`.
- [ ] Config-side mapping converts `GlobalDiscoveryResult` into `ConfigDiscoveryResult.global` using `ConfigLocation::Global(...)` in `config/root.rs` or another Config-owned module. Discovery must not perform this classification.
- [ ] Unit/integration tests cover source precedence, suppression behavior, no-config behavior, case-correction diagnostics, same-tier alternatives, and Config-side global mapping.

## Agent Brief

**Category:** enhancement
**Summary:** Implement Environment Config discovery with deterministic tier precedence and soft-miss behavior.

**Current behavior:**
Environment Config lookup is partially hardcoded and does not expose consistent precedence semantics across all supported locations/formats.

**Desired behavior:**
Mechanical Environment Config path discovery checks tiers in documented order, finds all candidates at the highest-priority tier, and picks a mechanical winner using `StructuredFileFormat::PRECEDENCE` through `discovery::selector::select_candidate()`.

**Key interfaces:**
- `discovery::engine::DiscoveryEngine::find_global()`
- `discovery::policy::GlobalSourceType` (ranked)
- `discovery::engine::GlobalDiscoveryResult` (winner + alternatives)
- `discovery::selector::select_candidate()`
- `GlobalDiscoveryInput<'_>` or equivalent core suppression input for `--no-global-config`
- Config-owned mapping in `config/root.rs` from `GlobalDiscoveryResult` to `ConfigDiscoveryResult.global`

**Boundary Note:**
Phase 2 is "Dumb" inside `lithos-core/src/discovery/`. Discovery may return only path/source/format metadata (`FoundRootMarker`, `GlobalDiscoveryResult`, Discovery-owned diagnostics). It must not import `GlobalConfigLocation`, `ConfigLocation`, `DiscoveredConfigFile`, `ConfigDiscoveryResult`, `ConfigWarning`, or any other Config-owned type.

Config classification happens after Discovery returns. The mapping from `GlobalDiscoveryResult` into `ConfigDiscoveryResult.global` belongs in Config-owned code, preferably `config/root.rs`; `config/discovery.rs::ConfigDiscoveryPipeline` should continue to consume already-classified `ConfigDiscoveryResult` values and read/query the selected files.

**Out of scope:**
- Local (vault) config discovery
- Format stability (History-aware promotion)
- Config parsing, validation, merging, and hashing
- CLI command wiring beyond defining the core suppression input consumed by CLI later

## Completed dependencies

- `.scratch/root-config-discovery/02-local-candidate-generation.md`
- `.scratch/root-config-discovery/03-candidate-selection-format-stability.md`
- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`

## Triage Update (2026-06-05)

> *This was generated by AI during triage.*

The prerequisite issues are completed. This issue is ready for an AFK agent after
the architecture clarifications above: Discovery remains a mechanical path/source
finder, while Config owns global location classification and selected-file
consumption.
