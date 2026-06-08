# Issue 04: Indexer application service

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Implement the Indexer application service in `lithos-core::indexer::service`
(or `::processor`). The service is the orchestration heart of the Indexer: it
accepts an `IndexScope`, `IndexOptions`, a `ScannerPort`, and a `Repository`,
and returns an `IndexResult`.

The pipeline the service must execute:

1. If `IndexOptions { reindex: true }`, discard all persisted state before
   scanning so every node is treated as new.
2. Scan via `ScannerPort` according to `IndexScope`.
3. For each scanned node, compare against persisted state to classify as
   `New`, `Fresh`, or `Stale`.
4. Persist deltas (`New` and `Stale` nodes are written; `Fresh` nodes are
   unchanged in storage).
5. Detect deletions: persisted nodes absent from the current scan are pruned
   and recorded as deleted `FsNodeId`s.
6. If `IndexOptions { dry_run: true }`, classify without persisting changes.
7. Accumulate per-node I/O failures as non-fatal records in `IndexResult`.
8. Return a complete `IndexResult` (entries, deleted IDs, summary counts,
   failures).

Hard abort conditions (return an error, do not return a partial result):
configuration errors (invalid Vault Root, missing Config specs) and repository
initialisation failures.

The service must depend only on the `ScannerPort` and `Repository` traits —
no walkdir, no redb, no concrete adapter types in the service module.

## Acceptance criteria

- [ ] Application-service tests classify missing persisted nodes as `New`.
- [ ] Application-service tests classify metadata-matching nodes as `Fresh`.
- [ ] Application-service tests classify changed metadata nodes as `Stale`.
- [ ] Application-service tests classify all nodes as `New` when
      `IndexOptions { reindex: true }` is set, regardless of stored metadata.
- [ ] Pruning tests: persisted nodes absent from the current scan are removed
      and reported as deleted `FsNodeId`s in `IndexResult`.
- [ ] Dry-run tests: classification runs without persisting any changes.
- [ ] Scope tests: `Full` and `Partial` scans use the expected scan
      boundaries.
- [ ] Per-node I/O failures accumulate in `IndexResult` without aborting the
      run.
- [ ] All application-service tests use the in-memory `ScannerPort` test
      double from issue 03 — no real disk or redb dependency.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- issue-03-ports-and-adapters.md
