---
title: 14-vault-storage-tables
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

Implement vault/storage.rs: Repository trait with new signatures, new storage tables, multimap indexes.

Create primary inode tables, path index tables, and query optimization multimaps.

## Acceptance criteria

- [ ] Primary tables: Table<FileId, FileView>, Table<DirId, DirView>
- [ ] Path index: Table<NormalizedPath, FileId>, Table<NormalizedPath, DirId>
- [ ] Multimap indexes: basename, parent, format
- [ ] Repository trait with exact lookups (find_file_by_path, get_file, get_dir, get_entry)
- [ ] Repository trait with indexed queries (find_files_by_basename, find_files_by_parent, list_markdown_files, list_files_by_format)
- [ ] Repository trait with full scans (list_all_files, list_all_dirs)
- [ ] RedbRepository adapter implementation
- [ ] Transaction support
- [ ] Tests for CRUD, index consistency, multimap queries

## Blocked by

- 13-vault-model-types
