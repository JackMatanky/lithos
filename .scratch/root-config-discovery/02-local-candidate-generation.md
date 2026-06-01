---
title: 02-local-candidate-generation
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

Implement deterministic local candidate generation for local (vault) config files across all supported location patterns and structured formats.

This slice should expose the path generation seam via `LocalConfigLocation::candidate_path(root, format)` and a discovery helper that enumerates existing candidates by iterating structured format precedence.

## Acceptance criteria

- [ ] `LocalConfigLocation::candidate_path(root, format)` exists and generates correct concrete paths for root, hidden-root, and config-directory patterns.
- [ ] `find_local_config_candidates(root, location)` exists and returns all existing candidates for that location.
- [ ] Candidate enumeration iterates `StructuredFileFormat::PRECEDENCE` and supports `toml`, `json`, `yaml`, `yml`.
- [ ] Returned candidate records include absolute path, base directory, location, and detected format.
- [ ] Unit tests cover all local location variants and all structured formats.
- [ ] Unit tests verify behavior when no candidates exist.

## Blocked by

- `.scratch/root-config-discovery/01-discovery-type-contracts.md`
