---
title: 03-candidate-selection-format-stability
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

Implement deterministic candidate selection for config files with precedence and format-stability behavior.

This slice defines the winner-selection seam for multiple candidates at the same location, preferring previously persisted format when present, otherwise falling back to strict structured format precedence.

## Acceptance criteria

- [ ] `select_config_candidate(candidates, persisted_format)` exists and returns `None` for empty input.
- [ ] When one candidate exists, that candidate is selected.
- [ ] When multiple candidates exist and `persisted_format` matches an existing candidate, the matching format is selected regardless of precedence rank.
- [ ] When multiple candidates exist and no persisted match exists, strict precedence selects `toml > json > yaml > yml`.
- [ ] Selection emits typed warnings for multi-format ambiguity.
- [ ] Unit tests cover all selection branches, including stability and precedence fallback.

## Blocked by

- `.scratch/root-config-discovery/02-local-candidate-generation.md`
