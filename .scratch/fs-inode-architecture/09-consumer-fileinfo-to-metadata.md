---
title: 09-phase-3a-fileinfo-to-metadata
category: enhancement
label: needs-triage
status: pending
date_created: 2026-05-11
---

## Type

AFK

## Labels

- needs-triage

## What to build

Update all consumers across contexts to replace FileInfo with FileMetadata (direct replacement, no alias).

Phase 3a: Straightforward rename across schema/, config/, fs/ and any other contexts.

## Acceptance criteria

- [ ] All FileInfo usages replaced with FileMetadata
- [ ] schema/ context updated
- [ ] config/ context updated
- [ ] fs/ context updated (internal usages)
- [ ] Any other consumers updated
- [ ] Run `mise run verify` - no compile errors
- [ ] Tests pass

## Blocked by

- 04-fs-metadata-types
- 07-fsreader-methods
- 08-fs-error-redesign
