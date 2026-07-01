---
title: 07-indexer-storage-consolidation
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-30
date_completed:
---

# Issue 07: Indexer storage consolidation — single `FS_ID_BY_PATH` table + kind-aware path lookup

## Why this is a foundation issue

The indexer's storage layer is foundation-owned, not integration-owned. This
refactor touches only `crates/indexer/` internals and has **zero external
consumers** (verified: nothing outside the indexer crate references
`FILE_ID_BY_PATH`, `DIR_ID_BY_PATH`, `IndexedNodes`, or `DeletedNodes`; vault has
its own separate copies over a different `FileId` type). The integration PRD
(`.scratch/filesystem-indexer/integration/PRD.md` §5) listed this as a
"precursor" but it belongs here: it closes the kind-discriminant loop that
foundation issue 02 opened (`02-domain-model.md:21,41` planned an
`FsRecordType { File, Dir }` that never landed — current `model.rs` has only
`FsParentId`).

The integration work (sink, consumers, services) depends on `IndexEvent`, which
is kind-tagged at the enum level regardless of the table shape. So this lands and
merges independently, before integration consumer work begins.

## What to build

### 1. `FsRecordKind` enum (`crates/indexer/src/model.rs`)

A new `{ File, Dir }` enum, sized for redb-friendly storage (tagged byte). This
is the discriminant `02-domain-model.md` originally called `FsRecordType`/
`FsNodeKind` but never shipped.

- Derive the rkyv set consistent with the rest of `model.rs`
  (`Archive, Serialize, Deserialize` + `Debug, Clone, Copy, PartialEq, Eq`).
- Add whatever redb-codec wrapper the value type requires. The existing
  `FsRecordId` uses `impl_redb_uuid!` (`storage/tables.rs:13`); `FsRecordKind`
  needs an equivalent byte-codec so `(FsRecordId, FsRecordKind)` can be a redb
  value. Confirm whether `traces_db` already exposes a tagged-byte/tuple value
  helper before hand-rolling one.

### 2. Path-uniqueness decision — **must be made explicit**

Today path uniqueness is **per-kind**: a file and a directory may share the same
`PathKey` because they live in two separate tables (`FILE_ID_BY_PATH`,
`DIR_ID_BY_PATH`, `tables.rs:21-25`). The shared repository contract test
deliberately exercises this — `assert_all_paths_deduplicates`
(`storage/contract.rs:67-75`) stores the **same path in both tables** and asserts
`all_paths` dedupes it to one entry.

A single `FS_ID_BY_PATH` keyed by `PathKey` alone makes uniqueness **global** and
**breaks that invariant** (a file and dir can no longer coexist at one path).

**Decision for this issue: preserve per-kind uniqueness.** Key the consolidated
table by `(PathKey, FsRecordKind)`, not by `PathKey` alone:

```
FS_ID_BY_PATH : (DbPathKey, FsRecordKind) -> FsRecordId
```

Rationale: it is a pure consolidation (one table instead of two) with **no
semantic change** to what the indexer permits, so the contract test and all
existing behaviour stay green. This deliberately differs from the integration
PRD's stated `PathKey -> (FsRecordId, FsRecordKind)` single-value shape (PRD §5
line 185), which would have forced global uniqueness. If a future requirement
genuinely wants global path uniqueness, that is a separate domain decision with
its own issue — do **not** smuggle it in here.

> ponytail: keying by `(PathKey, FsRecordKind)` is the lazy correct option — it
> keeps the existing contract intact and is one table instead of two. Global
> uniqueness is a bigger semantic change; don't take it on speculatively.

### 3. `find_id_by_path` repository method (`crates/indexer/src/repository.rs`)

Add to `ReadRepository`, matching the existing trait pattern (owned return,
`IndexerRepositoryError`):

```rust
/// Resolve a path to its record id and kind via the single FS_ID_BY_PATH
/// table, without reading the primary FILES / DIRS tables.
fn find_id_by_path(
    &self,
    path: &PathKey,
) -> Result<Option<(FsRecordId, FsRecordKind)>, IndexerRepositoryError>;
```

With the `(PathKey, FsRecordKind)` key, "look up a path of unknown kind" is a
range/prefix scan over the two kind-tagged entries for that `PathKey`, returning
the one that exists. Implement for both `RedbRepository`
(`storage/read.rs`) and the test `InMemoryRepository` (`storage/testing.rs`).

### 4. Rewrite by-path reads through `find_id_by_path` (`storage/read.rs`)

- `find_file_by_path` / `find_dir_by_path` (read.rs:74-116) chain through
  `find_id_by_path` (filtering on the expected kind), then read the primary
  table.
- Switch the primary-table read in `find_file`, `find_dir`, and the by-path
  methods from `rkyv::from_bytes` (current `deserialize_file`/`deserialize_dir`,
  read.rs:34-43) to `rkyv::access` + `rkyv::deserialize`. read.rs:9-11 already
  documents that local `rkyv::access` is available; redb yields contiguous
  slices so no `ArchivedEntity` alignment-buffering is needed.
- **`all_paths` (read.rs:217-247) must be updated.** It currently iterates BOTH
  path tables and dedupes across them. After consolidation it iterates the one
  `FS_ID_BY_PATH` table. With the `(PathKey, FsRecordKind)` key, the same
  `PathKey` can still legitimately appear twice (once per kind), so the
  cross-table dedup logic is **still required** — just over one table instead of
  two. Keep returning distinct `PathKey`s.

### 5. Rewrite the write paths (`storage/write.rs`)

All of these reference the two separate tables today and must move to the single
table keyed by `(PathKey, FsRecordKind)`:

- `remove_file_graph` / `remove_dir_graph` (write.rs:62-106) — remove the
  `(path, File)` / `(path, Dir)` entry.
- `file_path_taken_by_other` / `dir_path_taken_by_other` (write.rs:114-134) —
  the duplicate-path guard. Because the key is kind-tagged, a file save checks
  `(path, File)` and a dir save checks `(path, Dir)` — **per-kind uniqueness is
  preserved**, exactly as today.
- `save_file_in_tx` / `save_dir_in_tx` (write.rs:136-198) — insert under the
  kind-tagged key.
- `clear` (write.rs:362-392) — delete/recreate the one `FS_ID_BY_PATH` table
  instead of the two.

### 6. Deletion detection no longer reads FILES / DIRS for kind

`detect_deletions` (`service.rs:221-249`) currently does up to two by-path reads
per missing path to discover whether it's a file or a dir (lines 236-238). With
`find_id_by_path` returning the kind directly, deletion detection iterates
`FS_ID_BY_PATH`, and for each path not in `seen_paths` pushes the id into the
right bucket by kind — **no reads of `FILES` or `DIRS`**.

> Note for the integration follow-up (NOT in scope here): the integration PRD
> wants `detect_deletions` to *emit* a per-path `DeletedRecordEvent { id, path }`.
> Today it batches into a `DeletedNodes` of ids and discards the path
> (service.rs:229). This issue should leave `detect_deletions` returning
> `DeletedNodes` as-is but make the path+kind cheaply available (via
> `find_id_by_path`) so the integration emit-point rewrite is a small change
> later. Do not add the event emit here.

### 7. Out of scope (explicitly deferred)

- The `view_file_by_path` / `view_dir_by_path` zero-copy API returning
  `&ArchivedFileRecord` (integration PRD §13). No consumer needs it; the
  `rkyv::access` + `rkyv::deserialize` switch gives most of the win.
- Any `*Entry`→`*Event` / `IndexedNodes`→`*Events` renames. Those are wire-domain
  naming and belong to the integration PRD, not this storage issue.
- The `IndexEvent` sink, consumers, and `find_id_by_path`'s use by an emit path.

## Acceptance criteria

- [ ] `FsRecordKind { File, Dir }` exists in `model.rs` with rkyv derives and a
      redb value codec; a round-trip test proves it serializes/deserializes.
- [ ] A single `FS_ID_BY_PATH` table keyed by `(DbPathKey, FsRecordKind)` valued
      by `FsRecordId` replaces `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH` in
      `tables.rs`; the two old constants are gone.
- [ ] `find_id_by_path(&PathKey) -> Option<(FsRecordId, FsRecordKind)>` is added
      to `ReadRepository` and implemented for `RedbRepository` and
      `InMemoryRepository`; a test proves it returns the right id+kind from
      hand-seeded rows and `None` for an absent path.
- [ ] `find_file_by_path` / `find_dir_by_path` are rewritten to chain through
      `find_id_by_path` and still return the correct record (existing tests pass
      unchanged).
- [ ] Primary-table reads use `rkyv::access` + `rkyv::deserialize` (no
      `rkyv::from_bytes` left in `read.rs`); existing record round-trip tests
      pass.
- [ ] `all_paths` iterates the single table and still returns distinct
      `PathKey`s; `all_paths_deduplicates_across_file_and_dir_tables`
      (read.rs:704) and the contract's `assert_all_paths_deduplicates`
      (contract.rs:67) **pass unchanged** — proving per-kind uniqueness is
      preserved.
- [ ] All write paths (`remove_*_graph`, `*_path_taken_by_other`,
      `save_*_in_tx`, `clear`) operate on the single table; the duplicate-path
      contract checks (contract.rs:77-93, write.rs duplicate_path tests) pass
      unchanged.
- [ ] `detect_deletions` discovers kind via `find_id_by_path` with **no reads of
      `FILES` / `DIRS`**; all `detect_deletions` tests (service.rs) pass.
- [ ] `assert_repository_contract` (contract.rs) passes for both
      `RedbRepository` and `InMemoryRepository` with no changes to the contract
      assertions themselves.
- [ ] No `view_*` API, no `*Event` renames, no event-emit logic introduced.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`); formatted (`mise run fmt`).

## Blocked by

- None. This builds on the merged foundation (issues 01-06) and is independent
  of the integration PRD. It should land **before** integration consumer work
  begins.

## Notes for the re-grilling sessions (context, not scope)

- This issue removes the integration PRD's Critical #4 (storage consolidation
  breaks the contract test) from the integration critical path. Once merged, the
  integration PRD's §5 collapses to a one-line "depends on foundation issue 07"
  and drops the table-shape detail.
- The integration PRD's stated value shape `PathKey -> (FsRecordId,
  FsRecordKind)` (PRD §5 line 185) should be corrected to the **key** shape
  `(PathKey, FsRecordKind) -> FsRecordId` decided here, or the discrepancy
  re-grilled if global uniqueness turns out to be wanted.

## TDD Plan

### 1. Domain Modeling: `FsRecordKind`
- **RED:** Write a test in `model.rs` asserting `FsRecordKind { File, Dir }` can be serialized/deserialized with `rkyv`, and has a valid `redb::Key` and `redb::Value` implementation (a 1-byte codec).
- **GREEN:** Define `FsRecordKind` with the required derives. Implement the `redb` codecs.
- **REFACTOR:** Ensure it conforms to zero-copy patterns (small `Copy` type passed by value).

### 2. Storage Setup: The New Table Definition
- **RED:** In `storage/testing.rs`, add a test for `InMemoryRepository::find_id_by_path` asserting it correctly resolves a hand-seeded path to its ID and Kind.
- **GREEN:**
  - Add `FS_ID_BY_PATH` to `tables.rs` keyed by `(DbPathKey, FsRecordKind)` and valued by `FsRecordId`.
  - Add `find_id_by_path` to the `ReadRepository` trait.
  - Implement it for `InMemoryRepository` and `MockRepository`.
- **REFACTOR:** Check visibility and naming.

### 3. Database Read Path Integration
- **RED:** Add an integration test in `storage/read.rs` ensuring `RedbRepository::find_id_by_path` returns correct data for both files and directories, handling the prefix/range scan over the `(PathKey, FsRecordKind)` tuple.
- **GREEN:** Implement `find_id_by_path` in `RedbRepository`.

### 4. Read Methods Migration (Refactoring under green tests)
- **REFACTOR:** Update `find_file_by_path` and `find_dir_by_path` in `read.rs` to internally call `find_id_by_path` instead of reading the old path tables.
- **REFACTOR:** Switch primary table reads in `read.rs` from `rkyv::from_bytes` to `rkyv::access` + `rkyv::deserialize`.
- **VERIFY:** Existing repository contract tests and integration tests must stay GREEN.

### 5. Write Methods Migration (Refactoring under green tests)
- **REFACTOR:** Update `save_file_in_tx`, `save_dir_in_tx`, `remove_file_graph`, and `remove_dir_graph` in `write.rs` to write/delete from the new `FS_ID_BY_PATH` table instead of the old separate tables.
- **REFACTOR:** Update duplication guards (`file_path_taken_by_other`, `dir_path_taken_by_other`) to check the new kind-tagged table.
- **REFACTOR:** Drop old `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH` tables from `tables.rs` and the `clear` function.
- **VERIFY:** `assert_repository_contract` and all duplicate-path tests must pass perfectly, proving per-kind uniqueness is preserved.

### 6. `all_paths` Iteration Migration
- **REFACTOR:** Update `all_paths` to iterate the single `FS_ID_BY_PATH` table while maintaining cross-kind deduplication.
- **VERIFY:** `all_paths_deduplicates_across_file_and_dir_tables` and the contract tests remain GREEN.

### 7. Service Layer Optimization: `detect_deletions`
- **RED:** (Covered by existing `detect_deletions` tests in `service.rs`). Run them to ensure they are GREEN before the change.
- **REFACTOR:** Modify `detect_deletions` to iterate `FS_ID_BY_PATH` and extract the kind directly from the key, removing the need to fetch the full record from the primary `FILES` / `DIRS` tables.
- **VERIFY:** Existing tests pass.
