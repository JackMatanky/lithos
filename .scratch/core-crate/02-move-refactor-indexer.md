---
labels: ["ready-for-agent"]
---

# Move and refactor Indexer in traces-core

## Parent

PRD: `.scratch/core-crate/PRD.md`

## What to build

Migrate the indexer context into `traces-core` and refactor it to use the new domain types.

1. Move the `traces-indexer` crate contents into `traces-core::indexer`.
2. Refactor `ScannerPort` and `WalkdirAdapter` to yield `EntryOutcome` instead of the old `ScanEntry` enum.
3. Replace the split `FileRecord`/`DirRecord` builder pipeline with a single `FsNode` pipeline. This should reduce the indexer builder's typestates from 10 states to 5.
4. Update `DeletedNode` to the minimal `{ id, path }` shape.
5. Consolidate the Redb storage schema in the indexer:
   - Collapse the 8 split tables into the 5 unified tables defined in the PRD (`FS_NODES`, `FS_ID_BY_PATH`, `FS_IDS_BY_PARENT`, `FS_IDS_BY_FORMAT`, `FS_IDS_BY_NAME`).
   - Collapse the `Repository` trait methods from ~20 to ~10.
6. Update all indexer tests to assert against the new `FsNode` domain type rather than the old separate file/dir records.

## Acceptance criteria

- [ ] Indexer module exists entirely within `traces-core`.
- [ ] `ScannerPort` and `WalkdirAdapter` yield `EntryOutcome`.
- [ ] Storage schema is consolidated to 5 tables and the Repository trait is halved in size.
- [ ] Builder pipeline uses a single `FsNode` path (5 typestates).
- [ ] All tests pass.

## Blocked by

- 01-scaffold-traces-core.md
