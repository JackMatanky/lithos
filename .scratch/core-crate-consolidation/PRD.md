---
labels: ["ready-for-agent"]
---

# PRD: Core Crate Consolidation and Filesystem Type Redesign

## Problem Statement

The workspace currently has 13 crate members. Several decisions made during the initial workspace split (`workspace-refactoring`) and the fs type redesign (`fs-inode-architecture`) have created structural issues:

1. **Every context depends on settings and indexer**, yet these live in separate crates. Downstream contexts like note, schema, and template each compile separate copies of the same foundational types and services.

2. **The fs crate defines ~20 types** (`FileName`, `BaseName`, `DirName`, `RelativeFilePath`, `RelativeDirPath`, `FileMetadata`, `DirMetadata`, `FsTimes`, `FilePath`, `DirPath`, `FsPath`, `FsPathRef`, `ParentDir`, `WriteTarget`, etc.) — most are thin newtype wrappers around `Box<str>`, `String`, or `PathBuf` with validation but no domain semantics. These decompose the filesystem into parts but never recompose into a meaningful whole, so every consumer writes its own assembly logic anyway.

3. **Identity sprawl**: 10 UUID wrappers (`FsRecordId`, `FsParentId`, `NoteId`, `SchemaId`, `PropertyId`, `TemplateId`, `FileId`, `DirId`, `VaultId`) exist across the workspace. Foundation issue 08 plans to unify context identities onto `FsRecordId`.

4. **The indexer's *Record types** (`FileRecord`, `DirRecord`) occupy the same conceptual space as the fs crate's *Node types (`FileNode`, `DirNode`) but are richer — they carry identity (`FsRecordId`), parent pointers, and index metadata. The fs crate's types carry none of this, so consumers build their own identity layer on top.

## Solution

Introduce a `traces-core` crate that consolidates foundational domain types and services every context depends on. Simultaneously, simplify the filesystem type hierarchy by replacing thin wrappers with a layered `FsEntry → FsNode` pipeline backed by `typed-path` for path handling.

### Core Crate

`traces-core` contains:

- **settings** (moved from `traces-settings`): `SettingsService`, `AppConfig`, config types, discovery, trust/track, `Builder`
- **indexer** (moved from `traces-indexer`): `IndexerService`, scan pipeline, event sink, `IndexEvent`, `IndexStatus`
- **domain types**: `FsNode`, `FsEntry`, `FsNodeType`, `FsSize`, `FsNodeId`, `FsParentId`, `FileFormat`, `FsEntryType`, `EntryOutcome`, `SkippedEntry`, `SkipReason`

Domain types live in `crates/core/src/types/` with one file per concern:

| File | Types |
|---|---|
| `types/node.rs` | `FsNode`, `FsNodeType`, `FsSize` |
| `types/id.rs` | `FsNodeId`, `FsParentId` |
| `types/ext.rs` | `FileFormat` |
| `types/entry.rs` | `FsEntry`, `FsEntryType`, `EntryOutcome`, `SkippedEntry`, `SkipReason` |

`mod.rs` re-exports. Path types (`Utf8UnixPathBuf`) come from the `typed-path` crate directly; the rkyv archive wrapper for DB storage lives in `crates/core/src/db/path.rs`.

Infrastructure crates (`traces-db`, `traces-fs`) remain separate — core depends on them normally.

### Filesystem Type Pipeline

**Layer 1 — FsEntry** (scan artifact, no identity):

```
struct FsEntry {
    path: typed_path::Utf8UnixPathBuf,
    kind: FsEntryType,
    metadata: std::fs::Metadata,
}

enum FsEntryType {
    File,
    Dir,
    SymFile,
    SymDir,
    Skipped,
}
```

This is the output of `ScannerPort`. It carries the raw OS metadata and a guaranteed-UTF-8 path. No custom path newtypes, no decomposed filename types — those are derived at the next layer.

**Layer 2 — FsNode** (post-identity domain type):

A single `FsNode` type with a `kind` discriminant. No separate `FileNode`/`DirNode` types.

```
struct FsNode {
    id: FsNodeId,
    kind: FsNodeType,
    path: Utf8UnixPathBuf,         // vault-relative, normalized
    parent_id: FsParentId,
    name: Box<str>,                  // file stem or dir name ("file.name")
    size: FsSize,
    is_symlink: bool,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    recorded_at: SystemTime,
}

enum FsNodeType {
    File(FileFormat),
    Dir,
}

enum FsParentId {
    Root,
    Id(FsNodeId),
}

enum FsSize {
    File(u64),
    Dir,
}

struct DeletedNode {
    id: FsNodeId,
    path: Utf8UnixPathBuf,
}
```

Every `FsNode` has a uniform, non-optional parent. `FsParentId::Root` is the vault root — no sentinel values, no `Option` footgun.

The `FsEntry → FsNode` conversion is a pure function: assign `FsNodeId`, compute `FsNode` fields from path + metadata, set `FsNodeType` via extension.

`FsSize` stays as a domain enum — a dir's size is semantically different from a file's byte count, and `FsSize::Dir` prevents treating it as `0` or `None`.

`FsTimes`, `FsMetadata`, `FileMetadata`, `DirMetadata` are all eliminated — their fields inline directly into `FsNode`.

### Path Handling with typed-path

Replace thin path newtypes with `typed_path::Utf8UnixPathBuf`:

- **`Utf8UnixPathBuf`** replaces `FileName`, `BaseName`, `DirName`, `RelativeFilePath`, `RelativeDirPath`, `RelativePath`, `FsPath` (in most uses), `FilePath`, `DirPath` (as I/O domain types).
- **`Utf8UnixPath::normalize()`** replaces `PathKey::normalize()` logic (resolves `.`/`..`, deduplicates separators).
- **`join_checked()`** replaces `WriteTarget` and `RelativePathValidator` anti-traversal validation.
- **Backslash → forward slash normalization** is a one-line `replace('\\', '/')` applied at the `FsEntry → FsNode` boundary.
- **Hidden-file blocking** remains a thin wrapper around `join_checked()` — typed-path doesn't check for hidden components.

`PathKey` is replaced by `Utf8UnixPathBuf` outright. The storage key IS the vault-relative UTF-8 path. rkyv archive support is added to `Utf8UnixPathBuf` to preserve zero-copy DB storage.

### ParentId Dual Representation

`FsParentId` distinguishes root explicitly in-memory but stores as a flat `FsNodeId` in DB:

```
// In-memory — compiler-enforced exhaustiveness
enum FsParentId { Root, Id(FsNodeId) }

// In-DB — raw FsNodeId column, ZERO sentinel for root
// Repository converts at boundary: Root → 0, Id(x) → x (and reverse)
```

Two conversion functions at the repository boundary. App code never sees ZERO, DB queries use `WHERE parent_id = ?` uniformly. This avoids both the sentinel-footgun class (in-memory) and the enum-overhead-on-query problem (storage).

### Storage Schema

The 8-table file/dir split consolidates into 5 unified tables:

| Table | Type | Key → Value | Replaces |
|---|---|---|---|
| `FS_NODES` | primary | `FsNodeId → archived FsNode` | `FILES` + `DIRS` |
| `FS_ID_BY_PATH` | unique | `Utf8UnixPathBuf → FsNodeId` | `FILE_ID_BY_PATH` + `DIR_ID_BY_PATH` |
| `FS_IDS_BY_PARENT` | multimap | `FsNodeId → [FsNodeId]` | `FILE_IDS_BY_PARENT` + `DIR_IDS_BY_PARENT` |
| `FS_IDS_BY_FORMAT` | multimap | `format_str → [FsNodeId]` | `FILE_IDS_BY_FORMAT` (dirs not indexed) |
| `FS_IDS_BY_NAME` | multimap | `name_str → [FsNodeId]` | `FILE_IDS_BY_BASENAME` (now covers dirs too) |

Repository trait collapses from 20 methods (10 file + 10 dir) to ~10 — single `find`, `find_by_path`, `list_by_parent`, `save`, `delete` instead of paired file/dir variants.

### Builder & Entry Simplification

The unified `FsNode` type eliminates the parallel file/dir branches throughout the indexer pipeline:

| Current (split) | Consolidated |
|---|---|---|
| `ScanEntry { File(FileNode), Dir(DirNode), Skipped }` | `FsEntry { kind: FsEntryType, path, metadata }` — `Skipped` is a variant of `FsEntryType` |
| `FileIndexEntry` / `DirIndexEntry` | single `IndexEntry { id, node: FsNode, status }` — `FsNodeType::File`/`Dir` on `node.kind` replaces the type-level split |
| `DeletedNodes { files, dirs }` | `DeletedNodes { ids }` |
| `CompletionKind { File, Dir, Skipped }` | `CompletionKind::Node { entry, path_key, id }` |

The builder's 5-state typestate drops from 10 state types to 5. The `Init → into_branch` match on `ScanEntry::File`/`ScanEntry::Dir` becomes a single path — `FsEntryType::Skipped` is filtered early and routes directly to `Completion`, while `File`/`Dir`/`SymFile`/`SymDir` all proceed through the same `Comparison → Persistence → Indexed → Completion` pipeline. Comparison logic (metadata vs stored record) is uniform since `FsNode` inlines all metadata fields regardless of kind.

### Dataview Implicit Field Alignment

The `FsNode` shape mirrors Obsidian Dataview's `file.*` implicit fields:

| Dataview field | FsNode mapping |
|---|---|---|
| `file.name` | `name: Box<str>` (stem for files, dir name for dirs) |
| `file.folder` | parent derived from `path.parent()` (no dedicated field) |
| `file.path` | `path: Utf8UnixPathBuf` (vault-relative full path) |
| `file.ext` | `kind: FsNodeType::File(FileFormat)` |
| `file.size` | `size: FsSize` |
| `file.ctime` / `file.cday` | `created_at: Option<SystemTime>` |
| `file.mtime` / `file.mday` | `modified_at: Option<SystemTime>` |

## User Stories

1. As a maintainer, I want `traces-core` to provide a single dependency for foundational types (settings, indexer, domain types), so that downstream crates import one crate instead of three.
2. As a developer, I want a single `FsNode` type to replace both `FileRecord`/`DirRecord` and `FileNode`/`DirNode`, so that there is one canonical representation of a scanned filesystem item.
3. As a developer, I want `FsEntry` as the scanner output type with `Utf8UnixPathBuf` and `std::fs::Metadata`, so that the scan layer has zero custom path or metadata types.
4. As a developer, I want `FsSize` as the only extracted metadata type, so that file/dir size semantics are preserved without a wrapping `FsMetadata` struct.
5. As a developer, I want `FsTimes`, `FsMetadata`, `FileMetadata`, and `DirMetadata` all eliminated with their fields inlined on `FsNode`, so that there are no intermediate grouping types without behavior.
6. As a developer, I want path anti-traversal validation via `typed_path::join_checked()`, so that security invariants come from a maintained upstream crate.
7. As a developer, I want `typed-path` as the path library for all vault-relative path handling, so that custom path newtypes are eliminated.
8. As a developer, I want `PathKey` replaced by `Utf8UnixPathBuf` with rkyv support, so that DB storage keys use the same path type as everything else.
9. As a developer, I want `FsParentId { Root, Id(FsNodeId) }` to model parent relationships, so that root is explicitly distinguished at the type level without sentinel values. DB stores a flat `FsNodeId` (ZERO for root) with conversion at the repository boundary.
10. As an architect, I want `FsNode` fields to mirror Obsidian Dataview's `file.*` implicit fields, so that the domain model aligns with the vault's primary query interface.
11. As a developer, I want the backslash→forward-slash normalization boundary at `FsEntry → FsNode`, so that normalization is applied once and guaranteed everywhere.
12. As a developer, I want the existing `traces-fs` infrastructure crate (`FileReader`, `DirScanner`, `PathValidator`, `Writer`) to stay unchanged, so that the I/O layer is not disrupted by the type consolidation.

## Implementation Decisions

1. **`traces-core` crate**: Contains settings, indexer, and the enriched domain types. Depends on `traces-db` and `traces-fs` as regular dependencies.
2. **`traces-settings` moves into `traces-core`**: The SettingsService, AppConfig, config types, discovery pipeline all move. No API changes.
3. **`traces-indexer` moves into `traces-core`**: The IndexerService, scan pipeline, `IndexEvent`, `IndexStatus` all move. No API changes. `FsRecordId` → `FsNodeId`.
4. **`traces-fs` loses its domain types but keeps infrastructure**: `FileReader`, `DirScanner`, `PathValidator`, `Writer` remain. The `*Node` and `*Metadata` types move to core (or are eliminated, in the case of `*Metadata`).
5. **ScannerPort returns `EntryOutcome`**: Replaces the current `ScanEntry { File(FileNode), Dir(DirNode), Skipped }` enum with `EntryOutcome { Included(FsEntry), Skipped(SkippedEntry) }`. The WalkdirAdapter converts `walkdir::DirEntry` directly into `EntryOutcome` with `Utf8UnixPathBuf` and `std::fs::Metadata` — no intermediate `FileNode`/`DirNode` construction.
6. **`FsParentId { Root, Id(FsNodeId) }`**: Dual representation — in-memory enum for compiler safety, DB stores flat `FsNodeId` (ZERO for root) with conversion at the repository boundary.
7. **`DeletedNode` is `{ id: FsNodeId, path: Utf8UnixPathBuf }`**: Deliberately minimal — no metadata, no status.
8. **`FileFormat` moves to `traces-core`**: Part of `FsNodeType::File(FileFormat)` — core already depends on `traces-fs` for infrastructure, but `FileFormat` is a domain enum, not I/O, and belongs alongside its consumer.
9. **Normalization happens at `FsEntry → FsNode` boundary**: Backslash→forward slash, `normalize()`, separator dedup — all applied once when the scan artifact becomes a domain node.
10. **rkyv archive for `Utf8UnixPathBuf`**: A newtype wrapper `ArchiveUtf8Path(Utf8UnixPathBuf)` with rkyv derives, or an orphan impl. Minimal surface — just enough for DB storage.
11. **`From<(FsEntry, FsParentId)> for FsNode` is infallible**: Because `FsEntryType` only contains `{ File, Dir, SymFile, SymDir }` (with `Skipped` handled by `EntryOutcome`), every `FsEntry` represents a valid scannable node. `FsNodeType` maps directly from `FsEntryType::File → FsNodeType::File(FileFormat)`, `FsEntryType::Dir → FsNodeType::Dir`, `SymFile/SymDir → respective FsNodeType` with `is_symlink: true`. This is the single enforcement boundary for name extraction, normalization, and field mapping.
12. **`SkippedEntry` and `SkipReason` move to `traces-core`**: Currently in `crates/indexer/src/report.rs`, but the types are scanner-level, not indexer-level. `SkippedEntry { path: Utf8UnixPathBuf, reason: SkipReason }` belongs alongside `FsEntry` in `types/entry.rs`. `SkipReason` variants remain `PermissionDenied` | `UnsupportedEntryType`. `IndexReport` stays in `traces-indexer` (aggregation is an indexer concern); it references `SkippedEntry` from core. This separates skipped items from scannable ones at the `EntryOutcome` layer instead of polluting `FsEntryType`.

## Testing Decisions

- **Integration-level testing**: The `FsEntry → FsNode` pipeline does not need isolated unit tests. Every field is extractable from path + metadata — the conversion is a pure projection that cannot fail in ways worth isolating.
- **Indexer service tests**: Existing tempdir-backed indexer tests continue with `FsNode` replacing `FileRecord`/`DirRecord`. The assertion shift is mechanical: `result.indexed().files()` → `result.indexed().nodes()`, `DeletedNodes { files, dirs }` → `DeletedNodes { ids }`.
- **typed-path wrapper tests**: The hidden-file blocking wrapper around `join_checked()` is the one piece of new validation logic. Test with known traversal-attack inputs (leading `.`, `..`, absolute paths, backslash→forward-slash conversion).
- **No regression testing needed for removed types**: `FileName`, `BaseName`, `DirName`, `RelativeFilePath`, `RelativeDirPath`, `FileMetadata`, `DirMetadata`, `FsTimes`, `FsMetadata` are deleted, not deprecated. Their callers are updated to use the replacement types directly.
- **Existing `traces-fs` infrastructure tests**: `FileReader`, `DirScanner`, `PathValidator`, `Writer` tests continue unchanged since the infrastructure API surface doesn't change.

## Out of Scope

- Moving `RedbRepository` implementations out of domain crates into an infrastructure crate. This is a separate hexagonal architecture concern.
- Adding new feature functionality. This PRD covers structural consolidation and type redesign only.
- Async runtime adoption. The crate merge doesn't change the threading model.
- File watcher integration. The event sink design from the indexer integration PRD is unchanged.

## Further Notes

- This work revises two prior decisions: (a) the `workspace-refactoring` decision to keep all contexts as separate crates, and (b) the `fs-inode-architecture` decision to create thin newtype wrappers for path components. Both were correct for their time but have shown their limits under sustained development.
- `traces-core` should not become a dumping ground. The rule: a type or service belongs in core if (a) it is depended upon by at least two downstream crates, and (b) it is not infrastructure (I/O, persistence, format-specific).
- The `FsEntry` layer exists specifically so the `ScannerPort` contract is independent of `walkdir`. This was already planned in the indexer integration PRD review (PRD review critical #4) — the typed-path path type completes that decoupling.
- The `typed-path` crate has no `no_std` constraints, supports serde via feature flag, and normalizes without I/O. It is a compatible drop-in for the path representation that `PathKey` and `RelativePath` currently serve.
