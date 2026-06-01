---
title: 04-phase-1-vault-root-resolution
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

Implement Phase 1 vault root resolution as a typed discovery phase with explicit override precedence, safe ascending traversal, and contextual not-found outcomes.

This slice must eliminate the circular bootstrap dependency by producing vault root context as discovery output rather than requiring `VaultRoot` as discovery input.

## Acceptance criteria

- [ ] Resolution order is explicit flag (`--vault`) then env override (`LITHOS_VAULT_PATH`) then ascending walk from canonicalized CWD.
- [ ] Explicit and env-provided paths are validated eagerly and fail fast when path is missing or not a directory.
- [ ] Ascending walk checks recognized local marker patterns and short-circuits on first match.
- [ ] Ascending walk maintains visited canonical-path set to detect symlink loops safely.
- [ ] `LITHOS_CEILING` termination is supported for boundary-limited traversal.
- [ ] Not-found is represented as typed `VaultRootResolution::NotFound` and is propagated for contextual handling.
- [ ] Unit/integration tests cover explicit, env, ascending success, ceiling termination, symlink traversal, and not-found behavior.

## Blocked by

- `.scratch/root-config-discovery/01-discovery-type-contracts.md`
