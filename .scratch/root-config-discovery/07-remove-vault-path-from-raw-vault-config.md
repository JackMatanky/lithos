---
title: 07-remove-vault-path-from-raw-vault-config
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

Remove `vault_path` from local raw config DTO/schema so vault root is exclusively runtime-discovered context from Phase 1.

This slice finalizes the circular-dependency break by ensuring local config no longer encodes root path data.

## Acceptance criteria

- [ ] `RawVaultConfig` no longer includes `vault_path`.
- [ ] Any schema/serde expectations tied to `vault_path` are removed or migrated.
- [ ] Builder/processor call paths no longer depend on `vault_path` for root determination.
- [ ] Deserialization tests prove local config without `vault_path` is valid.
- [ ] Regression tests verify root resolution remains owned by discovery outputs.

## Blocked by

- `.scratch/root-config-discovery/06-phase-2-local-config-discovery.md`
