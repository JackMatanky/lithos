---
title: 01-base-domain-and-deltas
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

Introduce the phase-1 domain contracts for base schema processing so downstream work can build on stable type shapes.

This slice adds the new `BaseSchema` domain type in `schema/base.rs` and the `ExtendsDelta` contract in `schema/delta.rs` using the agreed multiple-inheritance-ready shape.

## Acceptance criteria

- [ ] `BaseSchema` exists in `lithos-core/src/schema/base.rs` with fields: `id`, `name`, `properties`, `extends: Box<[SchemaName]>`, `excludes: Box<[PropertyName]>`.
- [ ] `BaseSchema` derives/implements required archive/serde traits used by schema persistence.
- [ ] `ExtendsDelta` exists in `lithos-core/src/schema/delta.rs` as aggregate delta: `added: Box<[SchemaName]>`, `removed: Box<[SchemaName]>`.
- [ ] `ExtendsDelta::is_empty()` exists and is used as unchanged semantic marker.
- [ ] `schema/mod.rs` exports `base` (and no `base_schema` legacy export name is introduced).
- [ ] Unit tests validate `BaseSchema` construction invariants and `ExtendsDelta` empty/non-empty behavior.

## Blocked by

- `.scratch/schema-processor-split/PRD.md` Phase 0 completion (`RawSchema.extends` and `SchemaVersion.extends` moved to `Vec<SchemaName>`).
