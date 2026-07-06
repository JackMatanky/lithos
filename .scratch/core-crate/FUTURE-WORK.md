# Future Work: Stripping traces-fs

The original plan for the core crate consolidation included a massive refactoring phase (originally slices 4-6) to completely strip the old domain types (`FileNode`, `DirNode`, `FsTimes`, `FileMetadata`, `PathKey`, etc.) out of `traces-fs` and migrate all downstream contexts to use the new `traces-core` types natively.

**This work is currently HELD OFF.**

## Why?

We need to complete the filesystem indexer integration (see `.scratch/filesystem-indexer/integration/PRD.md`) first. Completing that integration will allow us to safely delete the `crates/vault/` crate, which is currently a major blocker to doing a clean rip-and-replace of the filesystem types across the workspace.

## When to resume

Once the `crates/vault/` crate has been successfully deleted, we can return here to formulate the remaining issues to:
1. Migrate Nodes and Metadata (swap `FileNode`/`DirNode` for `FsNode`).
2. Migrate Storage Keys (swap `PathKey` for `Utf8UnixPathBuf`).
3. Migrate Path and Name Wrappers (swap thin wrappers for native `typed-path` methods).
