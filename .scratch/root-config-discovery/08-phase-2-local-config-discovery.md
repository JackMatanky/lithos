---
title: 08-phase-2-local-config-discovery
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

Implement local (vault) config discovery in Phase 2 by integrating resolved vault root context with mechanical candidate detection and deterministic winner selection.

This slice introduces the **`CandidateProbe`** which is used to find specific config file candidates within the vault root once it has been resolved. It produces a **`VaultDiscoveryResult`** containing the mechanical winner (`marker`) and all same-tier candidates (`alternatives`), while preserving strict "Dumb" separation from config-side classification and stability rules.

## Acceptance criteria

- [ ] **Candidate Discovery Implementation**:
    - [ ] Implement `CandidateProbe` in `discovery/probe.rs` to identify local config candidates (e.g., `lithos.toml`, `.lithos/config.toml`).
- [ ] Local discovery populates the `marker` and `alternatives` fields of **`VaultDiscoveryResult`**.
- [ ] All three local location patterns are checked in priority order: `RootFile` > `HiddenRootFile` > `ConfigDirFile`.
- [ ] Selection uses **`discovery::selector::select_candidate()`** to pick the mechanical winner using `StructuredFileFormat::PRECEDENCE`.
- [ ] Multi-location and multi-format ambiguities populate the `alternatives` list and emit `DiscoveryWarning` variants.
- [ ] Result records `base`, `path`, and `format` metadata for every candidate found.
- [ ] Integration tests cover happy-path local discovery, no-local-config behavior, and ambiguity-warning scenarios.

## Agent Brief

**Category:** enhancement
**Summary:** Implement Phase 2 local config discovery as mechanical detection of candidates within a resolved root.

**Current behavior:**
Local config resolution is not yet represented as a decoupled mechanical search returning a precedence winner and same-tier alternatives.

**Desired behavior:**
Local discovery finds all candidates matching the three local patterns, identifies their formats, picks a mechanical winner, and returns a `VaultDiscoveryResult` carrying the winner and all alternatives.

**Key interfaces:**
- `discovery::engine::VaultDiscoveryResult` (root + marker + alternatives)
- `discovery::probe::CandidateProbe`
- `discovery::selector::select_candidate()`
- `discovery::diagnostics::DiscoveryWarning`

**Boundary Note:**
Phase 2 is "Dumb". It must not import `LocalConfigLocation` or `DiscoveredConfigFile` from the `config` context. Stability rules (database-aware format promotion) are handled in the `config` Consolidator using discovery-provided alternatives.

**Out of scope:**
- Format stability logic (History-aware promotion)
- Global config discovery
- Config parsing and validation

## Blocked by

- `.scratch/root-config-discovery/04-phase-1-vault-root-resolution.md`
- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`
- `.scratch/root-config-discovery/07-phase-2-environment-config-discovery.md`
