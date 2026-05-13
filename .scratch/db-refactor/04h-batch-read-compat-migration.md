---
title: 04h-batch-read-compat-migration
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

Replace legacy batch-read coupling (`with_batch_schema_reader` and related
legacy shapes) with v2-compatible read composition while preserving efficient,
single-pass discovery behavior.

This slice is complete when batch-style schema reads needed by runtime
orchestration no longer depend on legacy storage traits.

## Acceptance Criteria

- [ ] Runtime schema flows no longer depend on legacy
      `with_batch_schema_reader` APIs.
- [ ] Path->ID and path->RawSchemaView lookup behavior remains correct for
      mixed hit/miss input sets.
- [ ] Any required v2-side helper abstraction is minimal and stays within the
      schema context boundaries.
- [ ] Integration and regression tests covering discovery/refresh behavior
      continue to pass.
- [ ] No clippy warnings and code is formatted.

## Blocked by

- `04g-schema-processor-write-path-migration.md`

## Notes

- Prioritize behavior parity and performance characteristics over API elegance.
