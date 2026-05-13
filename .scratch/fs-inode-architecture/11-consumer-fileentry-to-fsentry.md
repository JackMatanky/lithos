---
title: 11-phase-3c-fileentry-to-fsentry
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

Update all consumers to replace FileEntry with FsEntry (unified file/dir enum).

Phase 3c: More complex - FsEntry is File(FsFile) or Dir(FsDir), FileEntry was file-only. Update all consumers to handle both variants.

## Acceptance criteria

- [ ] DirScanner.entries() returns Vec<FsEntry> (not Vec<FileEntry>)
- [ ] All consumers updated to handle FsEntry::File vs FsEntry::Dir
- [ ] FsReader.list_entries() return type updated
- [ ] All other FileEntry usages replaced
- [ ] Run `mise run verify` - no compile errors
- [ ] Tests pass

## Blocked by

- 05-fs-entry-types
- 06-dirscanner-methods
- 07-fsreader-methods
- 08-fs-error-redesign
