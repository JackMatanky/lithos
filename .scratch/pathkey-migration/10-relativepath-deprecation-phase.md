---
title: "Issue 10: Start RelativePath deprecation and prevent reintroduction"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 10: Start RelativePath deprecation and prevent reintroduction

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Begin staged `RelativePath` retirement by deprecating remaining approved uses and adding enforcement to prevent new references.

## Acceptance criteria

- [ ] `RelativePath` is marked deprecated with migration guidance.
- [ ] New usages are blocked by architecture checks/lints in scoped modules.
- [ ] Existing allowed legacy uses are isolated and documented with owners.
- [ ] CI fails on unauthorized new `RelativePath` references.

## Blocked by

- `.scratch/pathkey-migration/09-relativepath-usage-audit-and-mapping.md`
