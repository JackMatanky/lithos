---
title: 06-rename-fsreader-purge-methods
category: refactoring
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Finalize the decoupling architecture by renaming `FsReader` to `FileReader` globally, and deleting all scanning/traversal methods from its implementation. The reader will be strictly dedicated to reading file bytes/strings and validating paths.

## Acceptance criteria

- [ ] `FsReader` is renamed to `FileReader` across the entire codebase.
- [ ] `filter_entries`, `filter_file_paths`, `filter_dir_paths`, etc., are removed from `src/fs/reader.rs`.
- [ ] `metadata()` is removed from `src/fs/reader.rs`.
- [ ] Any redundant unit tests in `reader.rs` testing traversal behavior are deleted.
- [ ] `mise run verify` passes with 0 errors or warnings.

## Blocked by

- 01-migrate-vaultprocessor
- 02-refactor-schemaprocessor-metadata
- 03-refactor-propertybank-metadata
- 04-refactor-configbuilder-metadata
- 05-refactor-noteprocessor-metadata

---

## Agent Brief

**Category:** refactoring
**Summary:** This is the culminating issue. Now that no processor relies on `FsReader` for scanning or fetching metadata individually, we can purge these methods and rename the struct to reflect its narrowed responsibility: reading file contents.

**Current behavior:**
`src/fs/reader.rs` contains `filter_entries`, `filter_paths`, `filter_dir_entries`, and `metadata()` along with their associated tests.

**Desired behavior:**
All of these methods are deleted. The struct `FsReader` is renamed to `FileReader`. `FsError` implementations may need minor adjustments if any variants solely existed for traversal (though ADR 017 already split `ScanError`).

## TDD Implementation Plan

1. **RED**: A straightforward renaming and deletion phase.
2. **GREEN**:
   - Rename `Reader` and `FsReader` to `FileReader` in `fs/reader.rs` and `fs/mod.rs`.
   - Delete the `filter_*` methods.
   - Delete `metadata()` and `std_metadata()`.
   - Apply `cargo fmt` and update references in `tests/architecture.rs`.
3. **REFACTOR**: Run `cargo check` and `mise run verify` to ensure the compilation succeeds across the workspace and no dangling imports remain.
