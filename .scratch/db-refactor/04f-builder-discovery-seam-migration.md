---
title: 04f-builder-discovery-seam-migration
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-13
---

## Type

AFK

## Parent

- `04-complete-schema-adapter-migration.md`

## What to build

Migrate the runtime schema orchestration seam for `Builder` and `DiscoveryEngine`
from the legacy `schema::storage::Repository` API to the v2 segregated
repository seam (`SchemaReadRepository` + `SchemaWriteRepository`) while keeping
schema discovery behavior unchanged.

This slice is complete when discovery and builder flows read cached schema state
through the v2 seam end-to-end, without introducing behavior drift.

## Acceptance Criteria

- [ ] `schema::builder` and `schema::discovery` no longer require
      `schema::storage::Repository` for schema reads.
- [ ] Discovery still performs one coherent cached-state read pass for graph,
      property bank view, raw schema views, and schema IDs.
- [ ] Existing schema loader/discovery tests pass without changing user-visible
      behavior.
- [ ] No clippy warnings and code is formatted.

## Blocked by

- `04e-remaining-schema-operations.md`

## Notes

- Keep this issue focused on seam migration and behavior parity.
- Do not mix in module renaming or legacy module deletion here.
