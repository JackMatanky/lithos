---
title: "Issue 06: Vault context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 06: Vault context hard cut from RelativePath to PathKey

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Migrate vault repository/storage boundaries and callers from `RelativePath` to `PathKey` to align with canonical key semantics.

## Acceptance criteria

- [ ] Vault repository/storage interfaces accept `PathKey` only.
- [ ] Vault call sites derive keys at boundary seams with root-scoped conversions.
- [ ] Existing vault behavior and key matching remain intact via integration tests.
- [ ] No new `RelativePath` introduced in vault context.

## Blocked by

- `.scratch/pathkey-migration/01-pathkey-core.md`
- `.scratch/pathkey-migration/05-schema-cut-relativepath-to-pathkey.md`
