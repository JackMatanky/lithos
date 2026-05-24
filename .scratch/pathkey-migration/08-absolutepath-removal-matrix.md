---
title: "Issue 08: Remove AbsolutePath with decision matrix and tracing policy"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 08: Remove AbsolutePath with decision matrix and tracing policy

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Remove `AbsolutePath` from production flows, replacing with `DirPath`/`FilePath`, while explicitly classifying each replacement as hard error, warning+continue, or trace-only.

## Acceptance criteria

- [ ] All `AbsolutePath` call sites are inventoried and classified with rationale.
- [ ] Security and boundary-critical paths remain hard errors.
- [ ] Optional/discovery-like paths use approved warning/trace behavior where appropriate.
- [ ] Structured tracing fields added for downgraded paths (`context`, `root`, `path`, `decision`).
- [ ] No panic regressions introduced during replacement.

## Blocked by

- `.scratch/pathkey-migration/04-schema-configspec-redesign.md`
