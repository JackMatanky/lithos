---
title: 06-rename-fsreader-purge-methods
category: refactoring
label: ready-for-agent
status: open
---

## Parent

Ref: #40

## What to build

Finalize the decoupling architecture by renaming `FsReader` to `FileReader` globally, and deleting all scanning/traversal methods from its implementation. The reader will be strictly dedicated to reading file bytes/strings and validating paths.

## Acceptance criteria

- [ ] `FsReader` is renamed to `FileReader` across the entire codebase.
- [ ] `filter_entries`, `filter_file_entries`, `filter_dir_entries`, `filter_paths`, `filter_file_paths`, and `filter_dir_paths` methods are removed from `src/fs/reader.rs`.
- [ ] `std_metadata()`, `metadata()`, `created_at()`, and `modified_at()` are removed from `src/fs/reader.rs`.
- [ ] Any redundant unit tests in `reader.rs` testing traversal behavior are deleted.
- [ ] `mise run verify` passes with 0 errors or warnings.
- [ ] DO NOT delete the `exists(&self, path: &Path)` method from `FileReader`.

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
All of these methods are deleted. The struct `FsReader` is renamed to `FileReader`. `FsError` implementations may need minor adjustments if any variants solely existed for traversal (though ADR 017 already split `ScanError`). The `exists` method must be retained.

## TDD Implementation Plan

1. **RED**: A straightforward renaming and deletion phase.
2. **GREEN**:
   - Rename `Reader` and `FsReader` to `FileReader` in `fs/reader.rs` and `fs/mod.rs`.
   - Delete the `filter_*` methods from `FileReader`.
   - Delete `metadata()`, `std_metadata()`, `created_at()`, and `modified_at()` from `FileReader`.
   - Apply `cargo fmt` and update references in `tests/architecture.rs`.
   - Globally rename `FsReader` to `FileReader` across all Rust source files and Markdown documentation.
   - Delete corresponding unit tests in `fs/reader.rs` for the removed methods.
3. **REFACTOR**: Run `cargo check` and `mise run verify` to ensure compilation succeeds across the workspace and no dangling imports remain.
