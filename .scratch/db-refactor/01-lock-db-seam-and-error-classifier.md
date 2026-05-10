---
title: 01-lock-db-seam-and-error-classifier
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-10
---

## Type

HITL

## Labels

- needs-triage

## What to build

Lock the DB Module seam for the Projection Store so adapters can rely on a stable Interface before migration work starts. Finalize the `Store` transaction model, wrapper-first table strategy, and error model with transparent redb wrappers plus strict `DbErrorKind` classification.

This slice is complete when one end-to-end read and one end-to-end write path can be reasoned about through the agreed seam and error modes, without requiring redb-specific matching outside the DB Module.

## Acceptance criteria

- [ ] DB Module design decisions are finalized for `Store`, `UuidTable`/`UuidMultimap`/`PathTable` wrappers, and `read.rs` + `write.rs` adapter pattern.
- [ ] `DbError` design uses transparent redb wrappers and includes strict/exhaustive `DbErrorKind` with no `Unknown` variant.
- [ ] Classifier surface is minimal and stable (`kind()`, `is_transient()`), with no deep taxonomy.

## Blocked by

None - can start immediately
