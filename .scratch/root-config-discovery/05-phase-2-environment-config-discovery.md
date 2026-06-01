---
title: 05-phase-2-environment-config-discovery
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

This slice covers global config source resolution only (env-path, XDG, user, system), including suppression mode and diagnostics, while keeping parsing/validation out of scope.

## Acceptance criteria

- [ ] Environment Config source precedence is implemented as: `LITHOS_CONFIG_FILE` > XDG config home > user config home > system config.
- [ ] Each tier supports structured format candidates (`toml`, `json`, `yaml`, `yml`) with documented precedence.
- [ ] Missing Environment Config at any tier is treated as a non-error and discovery continues.
- [ ] `--no-global-config` suppresses Environment Config lookup entirely.
- [ ] Mis-cased recognized filenames produce corrective warning diagnostics.
- [ ] Unit/integration tests cover source precedence, suppression behavior, no-config behavior, and case-correction diagnostics.

## Blocked by

- `.scratch/root-config-discovery/02-local-candidate-generation.md`
- `.scratch/root-config-discovery/03-candidate-selection-format-stability.md`
