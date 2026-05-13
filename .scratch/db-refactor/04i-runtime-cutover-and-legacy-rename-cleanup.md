---
title: 04i-runtime-cutover-and-legacy-rename-cleanup
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

Complete runtime cutover to the v2 schema repository seam and then perform
module/component renaming to intended canonical names once all legacy schema
storage code paths are removed and verified absent.

This slice is complete when the runtime uses only the new seam and naming is
cleaned up so `v2` and legacy transitional naming are no longer needed.

## Acceptance Criteria

- [ ] Runtime schema loading and processing instantiate and use only the new
      schema repository seam.
- [ ] Verification confirms no active legacy schema storage call paths remain
      in production code.
- [ ] Transitional module/component names are renamed to intended canonical
      names after legacy removal is verified.
- [ ] References/imports/docs are updated consistently for renamed modules and
      components.
- [ ] Regression tests pass after renaming, with no functional behavior change.
- [ ] No clippy warnings and code is formatted.

## Blocked by

- `04h-batch-read-compat-migration.md`

## Notes

- Renaming must be done only after explicit verification that legacy code paths
  are removed, to avoid ambiguous mixed naming during transition.
