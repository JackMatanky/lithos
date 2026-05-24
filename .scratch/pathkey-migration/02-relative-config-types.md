---
title: "Issue 02: Add passive RelativeDirPath and RelativeFilePath config types"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 02: Add passive RelativeDirPath and RelativeFilePath config types

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Introduce `RelativeDirPath` and `RelativeFilePath` as declarative config value wrappers (string-based), with validation but no conversion/materialization behavior.

## Acceptance criteria

- [ ] `RelativeDirPath` and `RelativeFilePath` are string wrappers (not `PathBuf` wrappers).
- [ ] Types validate relative multi-segment paths and canonical separator rules.
- [ ] Types expose no conversion APIs (`to_*`, `resolve_*`, `as_*` for FS/key materialization).
- [ ] Tests cover accepted and rejected forms, including traversal and absolute-path rejection.

## Blocked by

None - can start immediately.
