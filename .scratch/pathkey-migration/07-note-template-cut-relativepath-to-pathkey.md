---
title: "Issue 07: Note/template context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 07: Note/template context hard cut from RelativePath to PathKey

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Migrate note/template repository and storage boundaries from `RelativePath` to `PathKey`, completing canonical key usage across contexts.

## Acceptance criteria

- [ ] Note/template repository interfaces use `PathKey` only.
- [ ] Callers derive keys only at repository boundary seams.
- [ ] End-to-end tests confirm unchanged note/template behavior after migration.
- [ ] No remaining note/template repository signatures accept `RelativePath`.

## Blocked by

- `.scratch/pathkey-migration/01-pathkey-core.md`
- `.scratch/pathkey-migration/06-vault-cut-relativepath-to-pathkey.md`
