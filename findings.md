# Discovery & Findings

## Architecture & Semantic Alignment

The repository separates filesystem operations (`fs` context) from indexing and state tracking (`indexer` context).

1. **`fs` Context:**
   - Yields data structures containing `Path` + `Metadata`.
   - Originally used `FsFile` and `FsDir`, which imply active OS file descriptors.
   - Using `FileNode` and `DirNode` implies locations in a filesystem tree.

2. **`indexer` Context:**
   - Tracks the state of nodes over time (staleness detection, UUID assignment, `recorded_at`).
   - Originally used `FileNode` and `DirNode`.
   - Using `FileRecord` and `DirRecord` perfectly describes persisted database-like entries.

## File Pipeline
- `walkdir::DirEntry` -> Raw OS traversal
- `fs::FileNode` -> Validated, safe filesystem structure
- `indexer::FileRecord` -> Persisted, identified index state
