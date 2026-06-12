# Task Plan: Rename FS Nodes and Indexer Records

## Goal
Establish a clear semantic boundary between `fs` (Filesystem Nodes) and `indexer` (Persisted Records) by renaming types and updating ubiquitous language in `CONTEXT.md`.

## Current Phase
Complete

## Phases

### Phase 1: Update Indexer to Records
- [x] Rename `FileNode` -> `FileRecord` in `indexer/model.rs`.
- [x] Rename `DirNode` -> `DirRecord` in `indexer/model.rs`.
- [x] Rename `FsNodeId` -> `FsRecordId` in `indexer/model.rs`.
- [x] Rename `FsNodeType` -> `FsRecordType` in `indexer/model.rs`.
- [x] Update `indexer/CONTEXT.md` to define `*Record` instead of `*Node`.
- [x] Apply changes across the workspace.
- **Status:** complete

### Phase 2: Update FS to Nodes
- [x] Rename `FsFile` -> `FileNode` in `fs/entry.rs`.
- [x] Rename `FsDir` -> `DirNode` in `fs/entry.rs`.
- [x] Rename `FsEntry` -> `FsNode`.
- [x] Keep `entry.rs` module path for minimal churn; type names now carry the node terminology.
- [x] Apply changes across the workspace.
- **Status:** complete

### Phase 3: Verification
- [x] Run `mise run verify` (or `fmt`, `lint`, `test`) to ensure everything compiles.
- [x] Fix any remaining type mismatch or reference errors.
- **Status:** complete

### Phase 4: Scratch Issue Documentation Sync
- [x] Update `.scratch/filesystem-indexer/02-domain-model.md` to use indexer `*Record` terminology.
- [x] Update `.scratch/filesystem-indexer/03-ports-and-adapters.md` storage/repository terminology.
- [x] Update `.scratch/filesystem-indexer/04-application-service.md` deleted ID terminology.
- [x] Update `.scratch/filesystem-indexer/PRD.md` to distinguish FS `*Node` names from indexer `*Record` names.
- [x] Verify no stale `FsEntry`, `FsFile`, `FsDir`, `FsNodeId`, or `FsNodeType` references remain in `.scratch/filesystem-indexer/`.
- **Status:** complete

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use `*Node` in `fs` | Represents structural points in the filesystem tree, distinct from open `File` handles and avoiding `Entry` collisions. |
| Use `*Record` in `indexer` | Represents the indexed, persisted state of a file in the "database" (index), matching fields like `id` and `recorded_at`. |

## Errors Encountered
| Error | Resolution |
|-------|------------|
| GitNexus `detect_changes` reported CRITICAL risk and stale old symbol IDs after rename | Verified with stale-name scans plus full `mise run verify`; recommend re-indexing GitNexus after this refactor. |
