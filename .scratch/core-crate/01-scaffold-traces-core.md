---
labels: ["ready-for-agent"]
---

# Scaffold traces-core and implement domain types

## Parent

PRD: `.scratch/core-crate/PRD.md`

## What to build

Create the new `traces-core` crate to hold foundational types and services.

1. Scaffold the `traces-core` crate (Cargo.toml).
2. Implement the domain types in `src/types/`:
   - `FsNode`, `FsNodeType`, `FsSize` (in `node.rs`)
   - `FsNodeId`, `FsParentId` (in `id.rs`). Note that `FsParentId` requires a dual representation: an in-memory enum (`Root` | `Id(FsNodeId)`) and a flat DB representation (where 0 is root).
   - `FsEntry`, `FsEntryType`, `EntryOutcome`, `SkippedEntry`, `SkipReason` (in `entry.rs`). Note: `FsEntryType` is just `{ File, Dir, SymFile, SymDir }` (no `Skipped` variant).
   - `FileFormat` (in `ext.rs`, moved from its current location).
3. Integrate the `typed-path` crate and implement the `ArchiveUtf8Path` wrapper with `rkyv` derives for database storage.
4. Implement the infallible `From<(FsEntry, FsParentId)> for FsNode` mapping. This must handle deriving the `name` field (stem for files, dir name for dirs) and applying backslash-to-forward-slash normalization.
5. Build a hidden-file blocking wrapper around `typed_path::join_checked()` and add the required path traversal attack tests (e.g., leading `.`, `..`, absolute paths, etc.).

## Acceptance criteria

- [ ] `traces-core` compiles successfully.
- [ ] Domain types are implemented as specified in the PRD.
- [ ] `FsNode` creation from `FsEntry` is infallible and correctly normalizes paths and extracts names.
- [ ] Hidden-file blocking wrapper is implemented and fully tested against traversal attacks.
- [ ] `ArchiveUtf8Path` is implemented and compiles with `rkyv` serialization.

## Blocked by

None - can start immediately.
