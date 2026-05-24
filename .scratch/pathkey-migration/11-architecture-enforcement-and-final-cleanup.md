---
title: "Issue 11: Architecture enforcement and final cleanup for path migration"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 11: Architecture enforcement and final cleanup for path migration

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Finalize migration governance with architecture tests enforcing boundary rules, remove transitional aliases when criteria are met, and publish cleanup documentation.

## Acceptance criteria

- [ ] Architecture tests enforce: repository boundaries use `PathKey`, `Relative*Path` restricted to config, no conversion methods on `Relative*Path`.
- [ ] `NormalizedPath` transitional alias removal criteria defined and executed when ready.
- [ ] Final `RelativePath` removal criteria validated against usage audit.
- [ ] Migration cleanup notes document boundary rules and approved conversion seams.

## Blocked by

- `.scratch/pathkey-migration/08-absolutepath-removal-matrix.md`
- `.scratch/pathkey-migration/10-relativepath-deprecation-phase.md`
