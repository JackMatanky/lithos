---
title: 01-migrate-vaultprocessor
category: enhancement
label: needs-triage
status: open
---

## Parent

Ref: #40

## What to build

Update `VaultProcessor` so that directory traversal and file discovery are powered entirely by the natively injected `DirScanner`. The processor must no longer depend on `FsReader`'s `filter_entries` or any other scanning methods.

## Acceptance criteria

- [ ] `scan_views` and related Vault processor trait methods accept `scanner: &DirScanner` alongside the reader.
- [ ] Vault discovery relies solely on `DirScanner` output (`FsEntry` arrays) rather than calling `FsReader::filter_*`.
- [ ] Depth-sorting and component processing continues to work correctly.
- [ ] Integration tests in `vault` context continue to pass with identical behavior.

## Blocked by

None - can start immediately

---

## Agent Brief

**Category:** enhancement
**Summary:** We are decoupling `FsReader`'s traversal methods. `VaultProcessor`'s `scan_views` currently relies on `FsReader::filter_file_entries` and `filter_dir_entries`. These must be changed to use `scanner.entries()` or `scanner.paths()` instead.

**Current behavior:**
`VaultProcessor::scan_views` takes only `source: &FsReader`. It calls `source.filter_dir_entries("**/*")` and `source.filter_file_entries("**/*")`.

**Desired behavior:**
Inject `scanner: &DirScanner` into `scan_views`. Use `scanner.entries(DirScanInput::new().with_pattern("**/*").include_dirs(true))` to get all entries, then partition them into files and directories instead of running two separate glob queries.

## TDD Implementation Plan

1. **RED**: Review `tests/note_reader.rs` to ensure full integration tests for `process_full` exist and pass on `main`. These serve as our behavioral regression net.
2. **GREEN**:
   - Modify the signature of `scan_views` in `lithos-core/src/vault/processor.rs` to accept `scanner: &DirScanner`.
   - Update its callers (like `process_full` and `process_partial`) to also accept and pass `DirScanner`.
   - Implement the partition logic over `scanner.entries()` to split into directories and files. Ensure `as_relative` and metadata extraction continue to work natively.
3. **REFACTOR**: Ensure the new traversal perfectly replicates the depth-sorting mechanisms (pre-computing relative paths for directories and sorting by component count). Verify `cargo test` passes.
