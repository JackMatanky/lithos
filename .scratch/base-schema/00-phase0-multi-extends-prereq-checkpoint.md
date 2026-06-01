---
title: 00-phase0-multi-extends-prereq-checkpoint
category: enhancement
label: ready-for-human
status: open
date_created: 2026-06-01
---

## Type

HITL

## Labels

- base-schema
- ready-for-human

## Parent

- `.scratch/base-schema/PRD.md`
- `.scratch/schema-processor-split/PRD.md`

## What to build

Confirm and close the cross-PRD prerequisite that Phase 0 (multi-parent extends alignment) is complete before BaseSchema Phase 1 starts.

This checkpoint is intentionally human-gated because it spans two PRDs and affects parsing, snapshots, and downstream semantics. It should produce a clear go/no-go decision for beginning the AFK BaseSchema slices.

## Acceptance criteria

- [ ] `RawSchema.extends` has been migrated from `Option<SchemaName>` to `Vec<SchemaName>`.
- [ ] `SchemaVersion.extends` has been migrated from `Option<SchemaName>` to `Vec<SchemaName>`.
- [ ] Callers that read extends from raw/snapshot types compile and preserve current single-parent behavior.
- [ ] A brief validation note is added to `.scratch/schema-processor-split/PRD.md` or linked issue comments indicating Phase 0 is complete.
- [ ] BaseSchema issues `01` through `07` are explicitly unblocked.

## Blocked by

None - can start immediately.
