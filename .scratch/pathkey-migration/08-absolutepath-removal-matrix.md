---
title: "Issue 08: Remove AbsolutePath from codebase"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
date_completed: null
---

# Issue 08: Remove AbsolutePath from codebase

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Remove `AbsolutePath` entirely, replacing with `Box<str>` in `TrustedVaultPath` and removing the `AbsolutePathError` variant from `PathValidationError`.

## Agent Brief

**Category:** enhancement
**Summary:** Remove `AbsolutePath` from the codebase — delete the type, remove the `AbsolutePathError` variant, and refactor `TrustedVaultPath` to wrap `Box<str>` with a `to_dir_path()` conversion boundary.

**Current behavior:**
`AbsolutePath` exists alongside `DirPath`/`FilePath`, creating overlapping semantics. `TrustedVaultPath` wraps `AbsolutePath` (a purely syntactic absolute-path wrapper), mixing config-domain path storage with filesystem path typing. `PathValidationError::AbsolutePathError` is an error variant whose name references a deleted type.

**Desired behavior:**
`AbsolutePath` is deleted entirely. `TrustedVaultPath` wraps `Box<str>` with only syntactic validation (non-empty, absolute) at config time, and provides a `to_dir_path()` method for the explicit config→filesystem boundary. `AbsolutePathError` is removed from `PathValidationError` — absolute paths are instead classified as generic validation failures. No new tracing/downgrade logic is added.

**Key interfaces:**
- `TrustedVaultPath` — inner type changes from `AbsolutePath` to `Box<str>`
- `TrustedVaultPath::to_dir_path()` — new method converting to `DirPath` at filesystem boundary
- `PathValidationError` — `AbsolutePathError` variant removed
- `fs::path::AbsolutePath` — entire struct + impls + tests deleted
- `fs::mod` — `AbsolutePath` removed from re-exports

**Acceptance criteria:**
- [ ] `TrustedVaultPath` wraps `Box<str>` with `try_new(PathBuf)` performing syntactic validation (non-empty, absolute)
- [ ] `TrustedVaultPath::to_dir_path()` returns `DirPath` at filesystem boundary
- [ ] `AbsolutePath` struct deleted from `fs/path.rs`
- [ ] `AbsolutePath` removed from `fs/mod.rs` re-exports
- [ ] `PathValidationError::AbsolutePathError` variant removed
- [ ] All existing tests pass with no regressions
- [ ] No `unwrap()`/`expect()` introduced in production code

**Out of scope:**
- Deletion of `RelativePath`
- Architecture test file (`lithos-core/tests/path_migration_architecture.rs`)
- Tracing downgrade matrix or structured tracing fields

## TDD & Implementation Plan

### Phase 1: Remove `AbsolutePathError` variant

**Behavior:** Absolute-rejection paths in `RelativePath` validation use a generic error variant instead of the `AbsolutePathError`-specific one.

- **RED:** Update tests in `fs/validator.rs` that assert `AbsolutePathError` — they should assert the replacement variant.
- **GREEN:** Remove `AbsolutePathError` from `PathValidationError`. Replace all references in `validator.rs` with `PathTraversalError` or a suitable generic variant.
- **Checklist:**
  - [ ] Test describes behavior, not implementation
  - [ ] Test uses public interface only
  - [ ] Test would survive internal refactor
  - [ ] Code is minimal for this test
  - [ ] No speculative features added
  - [ ] No `unwrap()`/`expect()` in production code

### Phase 2: Rewrite `TrustedVaultPath` as `Box<str>` wrapper

**Behavior:** `TrustedVaultPath` wraps `Box<str>` with syntactic validation (non-empty, absolute). A `to_dir_path()` method provides the explicit config→filesystem conversion boundary.

- **RED:** Update `TrustedVaultPath` tests: construction tests should assert syntactic validation only. Add a test for `to_dir_path()` returning an error for nonexistent directories.
- **GREEN:** Rewrite `TrustedVaultPath::try_new` to validate via `Path::is_absolute()` + non-empty and store as `Box<str>`. Implement `to_dir_path()` delegating to `DirPath::try_new`.
- **Checklist:**
  - [ ] Test describes behavior, not implementation
  - [ ] Test uses public interface only
  - [ ] Test would survive internal refactor
  - [ ] Code is minimal for this test
  - [ ] No speculative features added
  - [ ] No `unwrap()`/`expect()` in production code

### Phase 3: Delete `AbsolutePath` type

**Behavior:** `AbsolutePath` struct, impls, archived type, and tests are removed. Re-export in `fs/mod.rs` updated. The `#[expect(clippy::module_name_repetitions)]` with reason referencing `AbsolutePath` is removed.

- **RED:** Compilation will fail with type-not-found errors — this is the red state.
- **GREEN:** Delete the `AbsolutePath` struct definition, all impl blocks, the `ArchivedAbsolutePath` impl, the `Display` impl, and the entire `mod absolute` test module. Remove from re-export. Remove/update the clippy expect.
- **Checklist:**
  - [ ] Build succeeds
  - [ ] No remaining references to `AbsolutePath` in `*.rs` source files
  - [ ] No `unwrap()`/`expect()` in production code

### Phase 4: Cleanup

- [ ] Run `mise run fmt`
- [ ] Run `mise run lint` — no warnings
- [ ] Run `mise run test:unit` — all tests pass
- [ ] Update doc comments referencing `AbsolutePath`
- [ ] Run `gitnexus_detect_changes()` — verify only expected symbols affected
