---
title: 01-migrate-vaultprocessor
category: enhancement
label: ready-for-agent
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
**Summary:** Simplify `VaultProcessor` by removing the unused `process_partial` optimization, and migrate full discovery to natively use `DirScanner`.

**Current behavior:**
`VaultProcessor::scan_views` relies on `FsReader::filter_dir_entries` and `FsReader::filter_file_entries`. It runs two separate glob queries. Furthermore, `VaultProcessor` maintains complex, parallel state machine logic (`discover_partial`, `compare_partial`) for sparse updates that are completely unused in production.

**Desired behavior:**
1. **Remove Premature Optimization:** Delete `process_partial` and all its associated partial-scan machinery (`discover_partial`, `compare_partial`, `complete_partial`, `ScanMode`).
2. **Migrate Full Discovery:** `VaultProcessor::scan_views` should accept a `&DirScanner` and execute a single `scanner.entries()` call (`DirScanInput::new().with_pattern("**/*").include_dirs(true)`). It must partition the yielded `FsEntry` iterator into files and directories, extracting metadata natively from the variants.

**Key interfaces:**
- `VaultProcessor::scan_views(scanner: &DirScanner)`
- `VaultProcessor::discover(self, scanner: &DirScanner)`
- `VaultProcessor::process_full` (will internally instantiate `DirScanner::new()`).

**Acceptance criteria:**
- [ ] `process_partial`, `discover_partial`, `compare_partial`, `complete_partial`, and `ScanMode` are deleted from `processor.rs`.
- [ ] `scan_views` and `discover` methods accept `scanner: &DirScanner`.
- [ ] Vault discovery uses a single `scanner.entries(...)` call instead of multiple `FsReader::filter_*` calls.
- [ ] `process_partial` tests are removed from `tests/note_reader.rs`.
- [ ] The remaining `process_full` integration tests in `tests/note_reader.rs` pass with identical behavior.

**Out of scope:**
- Removing or deprecating `FsReader::filter_*` and `FsReader::*metadata` methods themselves (that will happen in issue #06).

## TDD Implementation Plan

Following YAGNI and the project's Test-Driven Development guidelines, this migration simplifies the internal state machine and relies on existing integration tests as a regression net.

1. **RED (Baseline Verification)**:
   - Run `cargo nextest run -p lithos-core --test note_reader` to establish a passing baseline.
   - Delete `partial_scan_does_not_prune_unscanned_missing_notes` and any other partial scan tests from `tests/note_reader.rs`.

2. **GREEN (Pruning Dead Code)**:
   - In `lithos-core/src/vault/processor.rs`, delete `process_partial`, `discover_partial`, `compare_partial`, `complete_partial`, and the `ScanMode` enum.
   - Simplify `compare` and `prune` to assume a full scan (removing `if mode == ScanMode::Partial` branches).
   - *Checkpoint*: Run `cargo clippy` and `cargo test` to ensure plumbing compiles and the remaining tests pass.

3. **GREEN (Internal Refactor - Scanner Implementation)**:
   - In `processor.rs`, update `VaultProcessor::process_full` to instantiate `let scanner = DirScanner::new(config.vault_metadata().root().as_path());`.
   - Update `discover` and `scan_views` signatures to accept `scanner: &DirScanner`.
   - Inside `scan_views`, replace the calls to `source.filter_dir_entries` and `source.filter_file_entries` with a single call to `scanner.entries(DirScanInput::new().with_pattern("**/*").include_dirs(true))`.
   - Iterate over the `FsEntry` items using `.into_iter()`, matching on `FsEntry::Dir(fs_dir)` and `FsEntry::File(fs_file)`.
   - Maintain the existing logic that maps `FsDir` to `(RelativePath, FsDir)` and performs the depth-based sort.
   - *Checkpoint*: Run `cargo nextest run -p lithos-core --test note_reader`.

4. **REFACTOR (Cleanup & Quality)**:
   - Eliminate unnecessary clones in `scan_views` now that `FsEntry` directly owns its typed metadata.
   - Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any warnings.
   - Run `cargo fmt` to adhere to coding styles.
