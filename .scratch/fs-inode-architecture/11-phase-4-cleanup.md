---
title: 11-phase-4-old-code-cleanup
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

Phase 4 cleanup: Delete old files, methods, and rename new methods (breaking change).

ONLY after all Phase 3 subphases complete.

## Acceptance criteria

- [ ] Delete fs/file.rs (contents moved to name.rs, metadata.rs, entry.rs)
- [ ] Delete fs/types.rs (contents moved to format.rs)
- [ ] Delete old DirScanner.entries() returning Vec<FileEntry>
- [ ] Delete old FsReader.info() method
- [ ] Rename FsReader.metadata_typed() → metadata() (remove "typed" suffix)
- [ ] Update all remaining consumers after deletions
- [ ] Run `mise run verify` - no broken imports
- [ ] Tests pass

## Blocked by

- 08-phase-3a-fileinfo-to-metdata
- 09-consumer-formatkind-to-fileformat
- 10-consumer-fileentry-to-fsentry
