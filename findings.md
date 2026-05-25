
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

## Proposed Hierarchy
`SchemaLoaderError` -> `SchemaRepositoryError` -> `DbError`

### Proposed `SchemaRepositoryError`
```rust
pub enum SchemaRepositoryError {
    #[error(transparent)]
    Storage(#[from] DbError),

    #[error(transparent)]
    Domain(#[from] SchemaError),

    #[error("not found: {0}")]
    NotFound(Box<str>),

    #[error("PropertyBank not found in database - initialize by loading schema files or creating properties")]
    PropertyBankNotFound,
}
```
