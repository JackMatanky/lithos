---
title: 13-vault-model-types
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

Implement vault/model.rs types: FileId(UuidV7), DirId(UuidV7), FileView, DirView, FsEntryView, NormalizedPath.

Compose fs/ primitives into domain storage entities with inode identity.

## Acceptance criteria

- [ ] FileId(UuidV7) - UUID-based file identifier
- [ ] DirId(UuidV7) - UUID-based directory identifier
- [ ] FileView: id, parent_id, name, format, metadata, content_hash
- [ ] DirView: id, parent_id, name, metadata
- [ ] FsEntryView enum: File(FileView), Dir(DirView)
- [ ] FsEntryView helpers: id_bytes(), parent_id(), name(), is_file(), is_dir()
- [ ] NormalizedPath(String) - vault-relative, forward slashes
- [ ] rkyv archived type support
- [ ] Tests for ID generation, view construction, helper methods
- [ ] Update vault/mod.rs exports

## Blocked by

- 01-fs-path-types
- 03-fs-format-types
- 04-fs-metadata-types
- 05-fs-entry-types
- 12-phase-4-cleanup
