# Indexer

The Indexer context owns filesystem node scanning and indexing, classifying nodes by type and tracking index status for downstream consumption.

## Language

**Filesystem Node**:
Any entry on disk (file or directory) discovered within an index scope.
_Avoid_: disk entry, fs object

**File Node**:
A filesystem node that is a regular file, eligible for content indexing.
_Avoid_: document, data file

**Directory Node**:
A filesystem node that is a directory, used for structural traversal.
_Avoid_: folder, container

**Index Scope**:
A configured root path and set of inclusion/exclusion patterns that define which filesystem nodes are eligible for indexing.
_Avoid_: scan area, search root

**Index Status**:
The classification of a node's index state: pending, current, stale, or deleted.
_Avoid_: freshness flag, state label

**Indexed Node**:
A filesystem node that has been scanned and is known in the index with current metadata.
_Avoid_: cached entry, processed file

**Deleted Node**:
A previously indexed node that no longer exists on disk.
_Avoid_: tombstone, removed entry

**Scanner Port**:
The interface through which the Indexer context requests filesystem traversal from the FS context.
_Avoid_: scan function, walk helper

## Invariants

- Index scope bounds are enforced by the FS context's path validation.
- Index status transitions are deterministic: pending → current, current → stale (on content change), current → deleted (on removal).

## Not Owned Here

- Filesystem path validation and traversal mechanics (delegated to FS context via Scanner Port).
- Note/schema/template business semantics and content processing.

## Interfaces

- `ScannerPort` — trait for filesystem traversal, implemented by the FS context.

## Resources
