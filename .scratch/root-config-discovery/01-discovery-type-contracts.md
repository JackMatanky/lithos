---
title: 01-discovery-type-contracts
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

Define the discovery submodule's core type contracts so root resolution and config file discovery become explicit, typed seams rather than implicit helper behavior.

This slice establishes the foundational enums/structs for discovery outcomes and warning reporting, including vault root resolution source annotations and config location/source metadata.

## Acceptance criteria

- [ ] A dedicated `config/discovery/` submodule is introduced and is the home of root/config discovery contracts.
- [ ] `VaultRootResolution` exists and distinguishes explicit flag, environment variable, ascending discovery, and not-found outcomes.
- [ ] `GlobalConfigLocation`, `LocalConfigLocation`, and `ConfigLocation` exist with documented precedence semantics.
- [ ] `DiscoveredConfigFile` exists and includes `location`, `base`, `path`, and `format`.
- [ ] `ConfigDiscoveryResult` exists and includes `global`, `local`, and `warnings`.
- [ ] `DiscoveryWarning` exists as a typed warning channel and includes variants for local ambiguity, format ambiguity, and case-correction diagnostics.
- [ ] Unit tests validate the shape and semantics of these contracts (not behavior of filesystem traversal yet).

## Blocked by

None - can start immediately.
