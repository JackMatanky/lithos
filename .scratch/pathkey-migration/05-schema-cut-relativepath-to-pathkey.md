---
title: "Issue 05: Schema context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 05: Schema context hard cut from RelativePath to PathKey

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Perform schema context hard cut so all repository/storage boundaries use `PathKey` instead of `RelativePath`.

## Acceptance criteria

- [ ] All schema repository trait signatures use `PathKey` at boundaries.
- [ ] Schema storage key types migrated from `RelativePath` to `PathKey`.
- [ ] Discovery and builder boundary calls no longer use `strip_prefix + RelativePath::try_from` chains.
- [ ] Integration tests verify schema read/write behavior remains correct after key-type migration.

## Blocked by

- `.scratch/pathkey-migration/01-pathkey-core.md`
- `.scratch/pathkey-migration/04-schema-configspec-redesign.md`
