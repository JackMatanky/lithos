# Issue 03: ScannerPort, walkdir adapter, repository ports, and redb storage adapter

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Implement the full infrastructure boundary for the Indexer: the scanner port
and its concrete walkdir adapter, the repository port traits, and the redb
storage adapter. No application service logic yet — this issue proves that
the Indexer can read from the filesystem and read/write its own persistence
tables independently.

### Scanner

- Define `ScannerPort` trait in `lithos-core::indexer::scanner`. The trait
  yields scan records (file/dir entries) from an `IndexScope`.
- Implement the walkdir adapter as the sole concrete `ScannerPort`
  implementation. It translates `ScanFilters` into walkdir `filter_entry`
  predicates, walks the subtree, and produces Indexer scan records. The
  existing FS context `DirScanner` is **not** used here.

### Repository ports

Define in `lithos-core::indexer::repository`:

- `ReadRepository` — lookup by `FsNodeId`, lookup by `PathKey`, listing by
  kind / format / parent, loading persisted paths for deletion detection.
- `WriteRepository` — save file/dir nodes, atomic batch save, delete by ID,
  atomic batch prune.
- `Repository: ReadRepository + WriteRepository`.

### redb storage adapter

Implement in `lithos-core::indexer::storage`. Tables (all updated atomically
in one `redb::WriteTransaction`):

| Table                | Key               | Value                  |
|----------------------|-------------------|------------------------|
| `FILES`              | `FsNodeId`        | rkyv `FileNode`        |
| `DIRS`               | `FsNodeId`        | rkyv `DirNode`         |
| `FILE_ID_BY_PATH`    | `PathKey` string  | `FsNodeId`             |
| `DIR_ID_BY_PATH`     | `PathKey` string  | `FsNodeId`             |
| `FILE_IDS_BY_BASENAME` | `&str`          | `FsNodeId`             |
| `FILE_IDS_BY_PARENT` | `FsNodeId` (parent) | `FsNodeId` (child)  |
| `FILE_IDS_BY_FORMAT` | `&str`            | `FsNodeId`             |

redb primitives stay inside the adapter. The public repository port exposes
Indexer domain errors, not redb errors.

## Acceptance criteria

- [ ] `ScannerPort` trait is defined and the walkdir adapter implements it.
- [ ] Scanner adapter tests prove `ScanFilters` translate into correct walkdir
      traversal without leaking walkdir details into domain contracts.
- [ ] `ReadRepository`, `WriteRepository`, and `Repository` traits are defined
      with the operations listed above.
- [ ] redb storage adapter implements all three traits.
- [ ] Repository contract tests prove primary node records and all secondary
      indexes (`path`, `parent`, `basename`, `format`) stay consistent after
      save, batch save, delete, and prune operations.
- [ ] All seven tables are defined; all writes occur in a single atomic
      `WriteTransaction` (indexes never diverge from primary data).
- [ ] An in-memory `ScannerPort` test double is provided (used by subsequent
      issues for application-service tests without real disk access).
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- issue-02-domain-model.md
