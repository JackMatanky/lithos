## Agent Brief

**Category:** enhancement
**Summary:** Move traces-indexer into traces-core and refactor to use unified FsNode

**Current behavior:**
The `traces-indexer` crate has a bifurcated pipeline for files and directories. Its `ScannerPort` returns a `ScanEntry` enum that mixes skipped items with file/dir nodes. Its `EntryBuilder` state machine uses 10 typestates (`Init`, `FileComparison`, `DirComparison`, `FilePersistence`, `DirPersistence`, `FileIndexed`, `DirIndexed`, `Completion`, etc.) to handle the split. The Redb storage schema uses 8 separate tables to store files and directories independently, and its `Repository` trait has roughly 20 methods.

**Desired behavior:**
The indexer should be moved into `traces-core` as a module. It should use the new `FsNode` and `EntryOutcome` types defined in `traces-core::types`. The bifurcated pipelines should be merged into a single path. The storage schema must collapse into 5 unified tables.

**Key interfaces:**
- `traces-indexer` crate should be moved to `traces-core::indexer`.
- `ScannerPort` and `WalkdirAdapter`: Must return `EntryOutcome` instead of `ScanEntry`.
- `EntryBuilder`: Collapse the 10 builder states into 5 unified states (`Init`, `Comparison`, `Persistence`, `Indexed`, `Completion`) using a single `FsNode` payload instead of separate `FileRecord`/`DirRecord` branches.
- `DeletedNodes`: Simplify to a struct with just a list of `ids: Box<[FsNodeId]>` (representing `{ id: FsNodeId, path: Utf8UnixPathBuf }` structurally or just passing IDs around).
- `Repository` and Redb Storage: Collapse the 8 split tables into 5 unified tables: `FS_NODES`, `FS_ID_BY_PATH`, `FS_IDS_BY_PARENT`, `FS_IDS_BY_FORMAT`, `FS_IDS_BY_NAME`. Halve the `Repository` trait methods to a unified set (`find`, `find_by_path`, `list_by_parent`, `save`, `delete`).

**Acceptance criteria:**
- [ ] The `traces-indexer` crate no longer exists; its contents are in `traces-core::indexer`.
- [ ] `ScannerPort` and `WalkdirAdapter` successfully yield `EntryOutcome`.
- [ ] The database schema is consolidated to the 5 specified tables.
- [ ] The builder pipeline uses a single `FsNode` path.
- [ ] All indexer tests are updated to assert against `FsNode` and all tests pass.

**Out of scope:**
- Modifying other workspace crates (`traces-note`, `traces-schema`, etc.) beyond what's needed to make them compile (if they import anything from the indexer).
- Deleting the old types from `traces-fs`.
