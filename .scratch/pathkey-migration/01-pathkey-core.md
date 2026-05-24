---
title: "Issue 01: PathKey core type and normalization pipeline"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 01: PathKey core type and normalization pipeline

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Implement `PathKey` as the canonical persistence-key type by renaming `NormalizedPath` and formalizing the `trim -> normalize -> validate` pipeline with root-scoped conversion errors.

## Acceptance criteria

- [ ] `PathKey` exists as canonical type, with short-lived deprecated `NormalizedPath` alias.
- [ ] Invariants enforced: UTF-8, relative-only, no `.`/`..`, normalized separators.
- [ ] Normalization uses zero-copy when possible and single allocation when normalization is required.
- [ ] `PathError` includes typed `OutsideRoot` and `InvalidUtf8` coverage with tests.

## Blocked by

None - can start immediately.
