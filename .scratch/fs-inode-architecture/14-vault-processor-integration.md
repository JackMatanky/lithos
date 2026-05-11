---
title: 14-vault-processor-integration
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

Refactor vault processor pipeline to use new types: FileId, DirId, FileView, DirView, FsEntryView, NormalizedPath.

Update save logic to populate all indexes. Delete old VaultPath, VaultFile, VaultFolder and old tables.

## Acceptance criteria

- [ ] Full vault scan produces complete FileView/DirView set
- [ ] Parent DirIds correctly linked (child.parent_id → parent DirId)
- [ ] Walkdir ordering guarantees parent-before-children
- [ ] Empty directory handling
- [ ] All indexes populated during save (path, basename, parent, format)
- [ ] Delete VaultPath, VaultFile, VaultFolder
- [ ] Remove old VAULT_FILES_BY_PATH, VAULT_FOLDERS_BY_PATH tables
- [ ] Existing tests pass
- [ ] Run `mise run verify`

## Blocked by

- 12-vault-model-types
- 13-vault-storage-tables
