
# Task: Consolidate Schema and DB Error Types

## Status
- [x] Initialize plan
- [x] Research usages and duplication
- [x] Design consolidated hierarchy
- [x] Implementation (TDD)
- [x] Refine NotFound variants with better typing
- [x] Verification

## Findings
- `SchemaStorageError` and `SchemaRepositoryError` had significant duplication.
- Many variants were unused (`Conflict`, `Corruption` on schema side).
- `SchemaStorageError` was an unnecessary level of nesting.
- `NotFound` was improved from `Box<str>` to structured types.

## Final Hierarchy
`SchemaLoaderError` -> `SchemaRepositoryError` -> `DbError`
