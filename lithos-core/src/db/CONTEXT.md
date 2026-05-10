# DB

The DB context provides persistence infrastructure for storing and querying rebuildable domain projections.

## Language

**Projection Store**:
Persistent state derived from file-backed source truth.
_Avoid_: system of record, canonical source

**Store**:
The DB context handle that scopes read/write units of work and transaction lifetimes.
_Avoid_: global database singleton, persistence service

**Repository Adapter**:
Infrastructure implementation that persists and loads domain entities.
_Avoid_: domain repository, business service

**Table Wrapper**:
A typed wrapper around table definitions that encodes common key/value patterns.
_Avoid_: raw table constant, ad-hoc table alias

**Zero-Copy Read**:
A read path that accesses archived data without full materialization.
_Avoid_: cached clone, eager decode

**Error Kind**:
A stable error classification used by callers to branch without matching backend-specific error types.
_Avoid_: string parsing, backend error matching in callers

## Invariants

- Database state is rebuildable from file-backed source data.
- Writes observe explicit transaction boundaries.
- Read APIs preserve safety guarantees for archived access.
- Backend-specific errors are wrapped transparently in DB errors, while callers branch on stable Error Kind.
- Table access for common patterns uses typed wrappers rather than repeating raw table-definition shapes.

## Not Owned Here

- Business semantics for note, schema, template, or config domains.
- Filesystem discovery and path validation policy.
- CLI command semantics and user interaction contracts.
- Context-specific Repository interfaces and query semantics.
