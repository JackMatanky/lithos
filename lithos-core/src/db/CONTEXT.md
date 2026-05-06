# DB

The DB context provides persistence infrastructure for storing and querying rebuildable domain projections.

## Language

**Projection Store**:
Persistent state derived from file-backed source truth.
_Avoid_: system of record, canonical source

**Repository Adapter**:
Infrastructure implementation that persists and loads domain entities.
_Avoid_: domain repository, business service

**Zero-Copy Read**:
A read path that accesses archived data without full materialization.
_Avoid_: cached clone, eager decode

## Invariants

- Database state is rebuildable from file-backed source data.
- Writes observe explicit transaction boundaries.
- Read APIs preserve safety guarantees for archived access.

## Not Owned Here

- Business semantics for note, schema, template, or config domains.
- Filesystem discovery and path validation policy.
- CLI command semantics and user interaction contracts.
