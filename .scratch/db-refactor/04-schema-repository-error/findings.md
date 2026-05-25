# Findings: Error Type Duplication and Hierarchy Analysis

## Current SchemaPersistence Hierarchy
`SchemaLoaderError` -> `SchemaRepositoryError` -> `SchemaStorageError` -> `DbError`

## Redundancies Identified
- `SchemaRepositoryError::Serialization` (String): **UNUSED**.
- `SchemaStorageError::Corruption` (Box<str>): **UNUSED**. `DbError::Corruption` is used instead.
- `SchemaStorageError::Conflict` (Vague): **UNUSED**.
- `SchemaRepositoryError::NotFound`: **UNUSED**.
- `SchemaStorageError::NotFound`: **USED** in `raw.rs` for "current version" missing.

## Consolidate Opportunity
- Merge `SchemaStorageError` into `SchemaRepositoryError`.
- Remove unused variants.
- Map `PropertyBankNotFound` to `SchemaRepositoryError` or a specific domain error.
- Use `DbError::Serialization` and `DbError::Deserialization` directly.

## Final Hierarchy
`SchemaLoaderError` -> `SchemaRepositoryError` -> `DbError`

### Final `SchemaRepositoryError` Implementation
- `Storage(#[from] DbError)`: Direct wrapping of database failures.
- `Domain(#[from] SchemaError)`: Direct wrapping of domain validation failures.
- `NotFoundById(SchemaId)`: Typed lookup failure.
- `NotFoundByName(SchemaName)`: Typed lookup failure.
- `NotFoundByPath(RelativePath)`: Typed lookup failure.
- `PropertyBankNotFound`: Specific domain invariant failure.
- `EmptyVersionHistory(RelativePath)`: Specific structural invariant failure (replacing generic NotFound for "current version").
