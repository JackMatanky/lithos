# Schema CQRS Error Recovery Guide

**Status**: Active
**Audience**: Developers, Operations
**Last Updated**: 2026-03-02

## Overview

This document provides operational guidance for recovering from errors in the Lithos schema CQRS system. It covers detection, diagnosis, and recovery strategies for common failure scenarios.

## Error Categories

### 1. Transient Errors

**Characteristics**:
- Temporary failures (database locked, I/O errors)
- Automatically retried by the system
- Usually resolve without intervention

**Examples**:
- `DbError::Locked` - Database is temporarily locked
- `DbError::Busy` - Database is busy with another operation
- `DbError::Io` - Temporary I/O failure

**Detection**:
```rust
if error.is_transient() {
    // System will retry automatically
}
```

**Automatic Recovery**:
The system uses exponential backoff retry with default configuration:
- **Max attempts**: 3
- **Initial delay**: 10ms
- **Max delay**: 1000ms
- **Backoff multiplier**: 2x

**Manual Recovery**:
Usually not needed. If persistent:
1. Check system resources (disk space, file handles)
2. Check for long-running transactions
3. Restart the application if necessary

### 2. Data Corruption

**Characteristics**:
- Invalid rkyv-serialized data in database
- Orphaned records (schema without metadata)
- Index inconsistencies

**Examples**:
- `DbError::Corruption` - Detected inconsistency (schema exists but metadata missing)
- `DbError::Deserialization` - Cannot deserialize stored data
- `SchemaQueryError::Storage` - General storage error during query

**Detection**:
```rust
// Corruption detected during query
match query.find_by_id(id) {
    Err(e) if e.to_string().contains("corruption") => {
        // Database corruption detected
    }
    _ => {}
}
```

**Prevention**:
- Always use `save_batch_with_metadata()` for schema saves
- Never manually modify database tables
- Use transactions for multi-step operations

**Recovery Strategies**:

#### Strategy 1: Re-ingest from Source Files
**Best for**: Corrupted schema data with intact source files

```rust
// 1. Delete corrupted schema
command.delete(corrupted_schema_id)?;

// 2. Re-ingest from file
let ingestor = Ingestor::new(fs_reader, &config);
let raw_schemas = ingestor.scan_raw_schemas()?;

// 3. Process through pipeline
let service = SchemaService::new(/* ... */);
service.load(&ingestor)?;
```

**Steps**:
1. Identify corrupted schema ID from error message
2. Delete corrupted record using command API
3. Re-run schema ingestion pipeline
4. Verify schema is valid: `query.find_by_id(id)?`

#### Strategy 2: Database Rebuild (Nuclear Option)
**Best for**: Widespread corruption, multiple affected schemas

```bash
# 1. Backup current database (if recovery needed)
cp ~/.cache/lithos/lithos.db ~/.cache/lithos/lithos.db.backup

# 2. Delete database
rm ~/.cache/lithos/lithos.db

# 3. Re-run application (will re-ingest all schemas)
lithos schema list
```

**Warning**: This rebuilds the entire database from source files. Only use when:
- Multiple schemas are corrupted
- Corruption affects core tables (metadata, property bank)
- Recovery from source files is acceptable

#### Strategy 3: Surgical Table Repair
**Best for**: Specific table corruption with known cause

```rust
// Example: Fix orphaned schema (missing metadata)
// This is a manual repair - use with caution

use redb::TableDefinition;

const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_metadata");

// If schema exists but metadata is missing, recreate metadata
let metadata = StoredMetadata::new(
    current_bank_version,
    Some(Timestamp::now()),
    None,
);

db.batch_write(|batch| {
    batch.put(SCHEMA_METADATA, id_key.as_str(), &metadata)?;
    Ok(())
})?;
```

**Warning**: Only use if you understand the data model. Incorrect repairs can cause further corruption.

### 3. Validation Errors

**Characteristics**:
- Domain rule violations
- Invalid property references
- Constraint violations

**Examples**:
- `SchemaCommandError::Validation` - Schema validation failed
- `SchemaError::InvalidPropertyReference` - Property not in PropertyBank
- `SchemaError::CircularInheritance` - Inheritance cycle detected

**Detection**:
```rust
match command.save_one(&schema) {
    Err(SchemaCommandError::Validation(msg)) => {
        // Validation error - fix schema definition
    }
    _ => {}
}
```

**Recovery**:
1. **Fix source file** - Validation errors indicate invalid schema definition
2. **Check property references** - Ensure all referenced properties exist in PropertyBank
3. **Verify inheritance** - Check for circular dependencies

**Example: Invalid Property Reference**
```toml
# Bad: References non-existent property
[properties.author]
type = "string"
ref = "nonexistent-prop"  # This will fail validation

# Good: Reference existing property or remove ref
[properties.author]
type = "string"
# No ref, or ref = "existing-prop"
```

### 4. Staleness Detection Failures

**Characteristics**:
- Cannot determine if schema needs update
- Missing timestamp metadata
- Version mismatches

**Examples**:
- `is_schema_stale()` returns error
- Missing created_at/modified_at timestamps

**Detection**:
```rust
match query.is_schema_stale(id, created, modified, bank_version) {
    Ok(true) => { /* Schema is stale, needs update */ }
    Ok(false) => { /* Schema is fresh */ }
    Err(e) => { /* Staleness check failed */ }
}
```

**Recovery**:
1. **Re-save schema** - Regenerates metadata with current timestamps
2. **Force update** - Delete and re-ingest to reset staleness state

**Prevention**:
- File system must support modification times
- Use `save_batch_with_metadata()` to include timestamps

## Observability

### Tracing

The system emits debug/warn traces for swallowed errors:

**File metadata failures** (`debug` level):
```
Failed to read file metadata, timestamps will be unavailable
Failed to read modified time from metadata
Modified time before UNIX_EPOCH
```

**Regex cache poison recovery** (`warn` level):
```
Regex cache poisoned (panic during validation), recovering
```

**Configure tracing**:
```toml
# .lithos/lithos.toml
[global]
log_level = "debug"  # To see file metadata traces
```

### Error Messages

All errors include context:
- **Corruption errors**: Include schema ID and affected table
- **Deserialization errors**: Include table name and key
- **Validation errors**: Include property name and rule violated

**Example error messages**:
```
schema 550e8400-e29b-41d4-a716-446655440000 exists but metadata is missing (database corruption detected)
```

```
property 'author' in schema 'article' not found in PropertyBank
```

## Best Practices

### Prevention

1. **Always use CQRS APIs** - Never modify database tables directly
2. **Use transactions** - Batch operations are atomic
3. **Test with mis-task runner** - Run `mise run test` before deployment
4. **Monitor logs** - Watch for `warn` level traces indicating issues

### Detection

1. **Enable debug logging** during development
2. **Test error paths** - Use corruption tests as examples
3. **Monitor metrics** - Track error rates, retry counts

### Recovery

1. **Start with least invasive** - Try transient retry → re-ingest → rebuild
2. **Backup before repairs** - Always backup database before manual fixes
3. **Verify after recovery** - Run `mise run verify` to ensure system health

## Testing Error Recovery

The test suite includes corruption detection tests:

```bash
# Run all corruption tests
cargo test --test schema_cqrs_critical -- query_detects

# Tests cover:
# - Corrupted schema data (ERROR-001)
# - Missing metadata (ERROR-002)
# - Corrupted property bank metadata (ERROR-003)
# - Corrupted name index (ERROR-004)
```

**Use these tests as recovery templates** - They demonstrate how to:
- Detect corruption
- Verify error messages
- Manually repair specific corruption types

## Related Documentation

- [ADR 005: Error Handling Framework](../adr/005-error-handling.md) - Overall error strategy
- [ADR 002: Port-Based CQRS](../adr/002-port-based-cqrs.md) - CQRS architecture
- [Schema CQRS Design](../design/009-schema-cqrs.md) - System design
- [Database Retry Implementation](../../lithos-core/src/db/retry.rs) - Retry logic

## Version History

| Version | Date       | Changes                              |
| ------- | ---------- | ------------------------------------ |
| 1.0     | 2026-03-02 | Initial version - Week 3 deliverable |
