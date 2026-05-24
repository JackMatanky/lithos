---
title: "Issue 09: Audit all RelativePath usage and map replacement target types"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 09: Audit all RelativePath usage and map replacement target types

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Create a full inventory of remaining `RelativePath` usage and map each call site to a target type (`PathKey`, `DirPath`, `FilePath`, `RelativeDirPath`, `RelativeFilePath`) before final removal.

## Acceptance criteria

- [ ] Inventory file committed listing every `RelativePath` usage.
- [ ] Each usage mapped to a replacement target type with rationale.
- [ ] Ambiguous/unresolved cases are explicitly listed with blocker notes.
- [ ] Follow-up migration tasks are linked from inventory sections.

## Blocked by

- `.scratch/pathkey-migration/07-note-template-cut-relativepath-to-pathkey.md`
