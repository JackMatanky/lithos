---
title: 10-phase-3b-formatkind-to-fileformat
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

Update all consumers to replace FormatKind with FileFormat (direct replacement, no alias).

Phase 3b: Make FileFormat public (was pub(crate)), add new format variants, update all usages.

## Acceptance criteria

- [ ] FileFormat is now public (was pub(crate))
- [ ] All FormatKind usages replaced with FileFormat
- [ ] New variants (Image, Pdf, Document, Archive) available
- [ ] All consumers updated
- [ ] Run `mise run verify` - no compile errors
- [ ] Tests pass

## Blocked by

- 03-fs-format-types
- 07-fsreader-methods
- 08-fs-error-redesign
