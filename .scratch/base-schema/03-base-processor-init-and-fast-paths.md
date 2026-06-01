---
title: 03-base-processor-init-and-fast-paths
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- base-schema
- ready-for-agent

## Parent

- `.scratch/base-schema/PRD.md`

## What to build

Create the new base schema processor skeleton in `schema/base_processor.rs` with the same orchestration style as the current property bank processor (`Init::from_discovery(...).run(...)`).

This slice delivers runnable entry flow for missing and fresh fast paths, without yet implementing full stale-delta behavior.

## Acceptance criteria

- [ ] `BaseSchemaProcessor<P, S>` exists with `Init` and `Unknown` entry-state.
- [ ] `from_discovery(...)` exists and derives `PathKey`/identity context from discovery inputs.
- [ ] `run(...)` handles missing-view path (`None`) and present-view path (`Some(view)`) end-to-end.
- [ ] Timestamp/content match fast paths produce `Fresh` lifecycle semantics with no semantic delta payload.
- [ ] New file path constructs `BaseSchema` and persists it through repository seam.
- [ ] Unit tests cover both `run(None, ...)` and `run(Some(view), ...)` happy paths.

## Blocked by

- `.scratch/base-schema/01-base-domain-and-deltas.md`
- `.scratch/base-schema/02-base-repository-contracts-and-storage.md`
