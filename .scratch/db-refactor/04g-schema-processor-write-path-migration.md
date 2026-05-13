---
title: 04g-schema-processor-write-path-migration
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

Migrate schema write orchestration in `schema_processor` from legacy write calls
to the v2 write seam, including save-many, raw view persistence, topological
graph save, and delete operations used during completion.

This slice is complete when processor write paths run end-to-end on the v2 seam
with equivalent atomic behavior.

## Acceptance Criteria

- [ ] `schema_processor` no longer calls legacy `save_schemas(&[&Schema])`.
- [ ] Save-many, delete, raw-view, and topology writes used by processor are
      routed through v2 write methods.
- [ ] Atomic semantics are preserved for processor-triggered persistence
      operations.
- [ ] Existing processor tests pass and no behavior regressions are introduced.
- [ ] No clippy warnings and code is formatted.

## Blocked by

- `04f-builder-discovery-seam-migration.md`

## Notes

- Keep this slice focused on write-path cutover and behavior parity.
