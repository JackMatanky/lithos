## Agent Brief

**Category:** enhancement
**Summary:** Scaffold traces-core crate and define the new filesystem domain types

**Current behavior:**
The filesystem domain types are currently scattered and fragmented. `FileRecord`, `DirRecord`, `FsRecordId`, and `FsParentId` live in `crates/indexer/src/model.rs`. `FileFormat` lives in `crates/fs/src/format.rs`. `SkippedEntry` and `SkipReason` live in `crates/indexer/src/report.rs`. Furthermore, paths are wrapped in various thin newtypes (like `FileName`, `FsPath`) and we lack a unified `FsNode` type.

**Desired behavior:**
A new `traces-core` crate should exist to centralize these foundational types.
The new domain types must be implemented as outlined in the PRD, effectively replacing the scattered records and enums with a unified `FsNode` and a simplified `EntryOutcome` pipeline.

**Key interfaces:**
- `FsNode` (replaces `FileRecord`/`DirRecord`): Should have a `kind: FsNodeType` and `path: Utf8UnixPathBuf` (using the `typed-path` crate).
- `FsNodeId` and `FsParentId`: Move/rename from `FsRecordId` and `FsParentId`. `FsParentId` needs a dual representation: an in-memory enum (`Root` | `Id(FsNodeId)`) and a flat database representation where 0 is root.
- `FsEntry`, `FsEntryType`, `EntryOutcome`, `SkippedEntry`, `SkipReason`: Relocate the skip structures from `report.rs` and establish the scanner outcome types. `FsEntryType` should only contain `{ File, Dir, SymFile, SymDir }` (no `Skipped` variant).
- `FileFormat`: Relocate from `traces-fs` to `traces-core`.
- `ArchiveUtf8Path`: A newtype wrapper around `Utf8UnixPathBuf` with `rkyv` derives for database storage.
- `From<(FsEntry, FsParentId)> for FsNode`: Must be infallible. It must derive the `name` field (stem for files, dir name for dirs) and apply backslash-to-forward-slash normalization.
- Hidden-file blocking: Implement a wrapper around `typed_path::join_checked()` to prevent path traversal attacks.

**Acceptance criteria:**
- [ ] `traces-core` compiles successfully and is integrated into the workspace.
- [ ] Domain types (`FsNode`, `FsNodeType`, `FsSize`, `FsNodeId`, `FsParentId`, `FsEntry`, `FsEntryType`, `EntryOutcome`, `SkippedEntry`, `SkipReason`, `FileFormat`) are implemented in `src/types/`.
- [ ] `FsNode` creation from `FsEntry` is infallible, correctly extracts the file/dir name, and normalizes path slashes.
- [ ] Hidden-file blocking wrapper is implemented and fully tested against traversal attacks (e.g., leading `.`, `..`, absolute paths).
- [ ] `ArchiveUtf8Path` is implemented and compiles with `rkyv` serialization.

**Out of scope:**
- Modifying `traces-indexer` or `traces-settings` to actually use these types (that will happen in subsequent issues).
- Deleting the old types from `traces-fs` or `traces-indexer`.
