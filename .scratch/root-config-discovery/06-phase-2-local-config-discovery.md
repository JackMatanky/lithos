---
title: 06-phase-2-local-config-discovery
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

Implement local (vault) config discovery in Phase 2 by integrating resolved vault root context with local candidate generation and deterministic winner selection.

This slice produces local `DiscoveredConfigFile` outcomes and typed warnings for ambiguity, while preserving strict separation from config parsing and validation.

## Acceptance criteria

- [ ] Local discovery consumes Phase 1 root resolution output and supports not-found behavior without panic.
- [ ] All three local location patterns are checked with documented location precedence.
- [ ] Candidate generation and selection integrate `find_local_config_candidates` and `select_config_candidate` seams.
- [ ] Multi-location and multi-format ambiguities emit typed warnings while still producing deterministic winner.
- [ ] Local discovery records `base`, `path`, `format`, `location`, and source metadata in returned result.
- [ ] Integration tests cover happy-path local discovery, no-local-config behavior, and ambiguity-warning scenarios.

## Blocked by

- `.scratch/root-config-discovery/04-phase-1-vault-root-resolution.md`
- `.scratch/root-config-discovery/05-phase-2-environment-config-discovery.md`
