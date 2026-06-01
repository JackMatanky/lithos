---
title: 06-lifecycle-handoff-and-deletion-semantics
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

Finalize phase-1 lifecycle handoff contract for downstream inheritance processing.

This slice ensures all processor outcomes emit deterministic `BaseSchemaChange` events (`Fresh`, `New`, `Stale`, `Deleted`) in `SchemaId` order and that deletion semantics keep persisted base state truthful.

## Acceptance criteria

- [ ] `BaseSchemaChange` variants are fully wired through processor terminal states.
- [ ] `Fresh` emits schema ID only; `New` and `Stale` carry base payload as defined in PRD.
- [ ] `Deleted` removes persisted `BaseSchema` and emits lifecycle event in same run.
- [ ] Output stream is deterministically ordered by `SchemaId`.
- [ ] No-op semantic changes do not emit `Stale`.
- [ ] Contract tests validate ordering, payload completeness, and lifecycle accuracy.

## Blocked by

- `.scratch/base-schema/05-stalereferences-targeted-reexpand-and-id-stability.md`
