---
title: 10-cross-context-verification-and-legacy-cleanup
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

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Final quality gate and generic renaming of repository traits.

**Current behavior:**
During migration, repository traits use context prefixes (e.g., `SchemaReadRepository`) to avoid collisions with v1 traits.

**Desired behavior:**
1. Verify all contexts (Schema, Note, Template, Config) are fully migrated and legacy v1 paths are removed.
2. Perform the "Generic Rename" step as per ADR 016:
   - `SchemaReadRepository` -> `ReadRepository`
   - `NoteReadRepository` -> `ReadRepository`
   - (and so on for Write and Unified variants)
3. Ensure all call sites use the module-qualified names (e.g., `schema::ReadRepository`).
4. Run full project verification.

**Key interfaces:**
- All repository traits across all contexts.

**Acceptance criteria:**
- [ ] Legacy DB adapter paths (v1) are completely removed from the codebase.
- [ ] All repository traits renamed to generic `ReadRepository`, `WriteRepository`, and `Repository`.
- [ ] `mise run verify` passes with no regressions.

**Revision Note (2026-05-12):**
Plan updated to include the final naming standardization step defined in ADR 016.

## Acceptance criteria

- [ ] Legacy DB adapter paths used by migrated contexts are removed.
- [ ] Context `testing.rs` in-memory repositories are aligned with new interfaces across Schema/Note/Template/Config.
- [ ] Full project verification for this change set passes (format/lint/tests) with no regressions in repository behavior.

## Blocked by

- `07-note-storage-migration-and-testing-repo-update.md`
- `08-template-storage-migration-and-testing-repo-update.md`
- `09-config-storage-migration-and-testing-repo-update.md`
