# Indexer

The Indexer context owns filesystem node scanning and indexing, classifying nodes by type and tracking index status using persisted records for downstream consumption.

## Language

**Filesystem Node**:
Any entry on disk (file or directory) discovered within an index scope via the FS context.
_Avoid_: disk entry, fs object

**File Record**:
An indexed record of a regular file, eligible for content indexing.
_Avoid_: document, data file, file node

**Directory Record**:
An indexed record of a directory, used for structural traversal.
_Avoid_: folder, container, directory node

**Index Scope**:
A configured root path and set of inclusion/exclusion patterns that define which filesystem nodes are eligible for indexing.
_Avoid_: scan area, search root

**Index Status**:
The classification of a record's index state: pending, current, stale, or deleted.
_Avoid_: freshness flag, state label

**Index Record**:
A tracked filesystem item that has been scanned and is known in the index with current metadata.
_Avoid_: cached entry, processed file, indexed node

**Deleted Record**:
A previously indexed record whose corresponding file no longer exists on disk.
_Avoid_: tombstone, removed entry, deleted node

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
