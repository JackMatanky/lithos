# Issue 02: Indexer domain model and result contracts

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Implement all Indexer domain types inside `lithos-core::indexer::model`. This
is the type-only layer — no application logic, no adapters. Every subsequent
issue depends on these types being stable and tested.

Types to implement:

- `FsNodeId` — newtype over `UuidV7`, following the pattern of every other
  context ID in the codebase.
- `FileNode` and `DirNode` — domain structs with `rkyv` derive for zero-copy
  persistence. Fields as decided in PRD Section 6:
  - `FileNode`: `id`, `parent_id`, `path: PathKey`, `name: FileName`,
    `format: FileFormat`, `metadata: FileMetadata`,
    `recorded_at: SystemTime` (with `rkyv::with::AsUnixTime`).
  - `DirNode`: `id`, `parent_id`, `path: PathKey`, `name: DirName`,
    `metadata: DirMetadata`, `recorded_at: SystemTime`.
- `FsNodeKind` — `File` / `Dir` enum for generic node classification.
- `IndexScope` — two-variant enum:
  `Full { filters: ScanFilters }` and `Partial { root: PathKey, filters: ScanFilters }`.
- `ScanFilters` — Indexer-owned type carrying extension/name narrowing criteria
  translatable to walkdir predicates.
- `IndexOptions` — `{ reindex: bool, dry_run: bool }`.
- `IndexStatus` — `New` / `Fresh` / `Stale`.
- `FileIndexEntry` — `{ node: FileNode, path: FilePath, status: IndexStatus }`.
- `DirIndexEntry` — `{ node: DirNode, path: DirPath, status: IndexStatus }`.
- `IndexResult` — file entries, directory entries, `Vec<FsNodeId>` deleted
  IDs, summary counts, per-node failure records.
- Indexer-specific error type(s) in `lithos-core::indexer::error`.

All fields are private by default; validated constructors where invariants
exist. No `unwrap()` or `panic!` in production code.

## Acceptance criteria

- [ ] All types above compile inside `lithos-core::indexer::model` (and
      `::error`).
- [ ] `FsNodeId` is a newtype over `UuidV7`; identity is stable, unique, and
      compatible with redb key wrappers.
- [ ] `FileNode` and `DirNode` derive `rkyv::Archive`, `rkyv::Serialize`,
      `rkyv::Deserialize` (consistent with existing node types in the
      codebase).
- [ ] Path conversion tests prove filesystem paths convert to `PathKey` only
      with an explicit Vault Root (no rootless conversion).
- [ ] Domain model construction tests reject invalid states and preserve valid
      `FsNodeId`, `FileNode`, and `DirNode` fields.
- [ ] `FsNodeId` identity tests: stable, unique, ordered where needed,
      compatible with DB key wrappers.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- issue-01-scaffolding.md
