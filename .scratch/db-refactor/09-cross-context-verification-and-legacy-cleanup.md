---
title: 09-cross-context-verification-and-legacy-cleanup
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-10
---

## Type

AFK

## Labels

- needs-triage

## What to build

Run cross-context verification for Schema, Note, Template, and Config after migration. Remove legacy DB adapter call paths, ensure in-memory testing adapters are aligned with new Repository seams, and complete quality-gate verification.

This slice is complete when the migrated architecture is the only active path and all context tests verify expected behavior.

## Acceptance criteria

- [ ] Legacy DB adapter paths used by migrated contexts are removed.
- [ ] Context `testing.rs` in-memory repositories are aligned with new interfaces across Schema/Note/Template/Config.
- [ ] Full project verification for this change set passes (format/lint/tests) with no regressions in repository behavior.

## Blocked by

- `06-note-storage-migration-and-testing-repo-update.md`
- `07-template-storage-migration-and-testing-repo-update.md`
- `08-config-storage-migration-and-testing-repo-update.md`
