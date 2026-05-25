# Task: Consolidate Schema and DB Error Types

## Status
- [x] Initialize plan
- [x] Research usages and duplication
- [x] Design consolidated hierarchy
- [x] Implementation (TDD)
- [x] Refine NotFound variants with better typing
- [x] Verification
- [x] Commit

## Findings
- `SchemaStorageError` and `SchemaRepositoryError` had significant duplication.
- Many variants were unused (`Conflict`, `Corruption` on schema side).
- `SchemaStorageError` was an unnecessary level of nesting.
- `NotFound` was improved from `Box<str>` to structured types.

## Final Hierarchy
`SchemaLoaderError` -> `SchemaRepositoryError` -> `DbError`

## Implementation Notes
- COLLAPSED: `SchemaStorageError` into `SchemaRepositoryError`.
- TYPED: Lookup failures use `SchemaId`, `SchemaName`, `RelativePath`.
- STRUCTURED: Invariant failures like missing versions use `EmptyVersionHistory`.
- CLEANED: Removed ~24 redundant `.into()` calls flagged by clippy.
