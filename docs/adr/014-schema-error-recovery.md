---
name: schema-cqrs-error-recovery-strategies
status: accepted
stakeholders: [Architecture Team, Schema Domain Owner]
date_proposed: 2026-03-04
date_decided: 2026-03-04
date_implemented: 2026-03-04
---

# ADR 014: Schema CQRS Error Recovery Strategies

---

## Context

The schema CQRS system handles critical operations including:
- Property bank version management and retention
- Schema persistence with metadata tracking
- Inheritance tree resolution
- File-based ingestion workflows

Errors can occur at multiple levels:
1. **Database errors**: Corruption, I/O failures, deserialization errors
2. **Domain errors**: Invalid property references, circular inheritance
3. **Infrastructure errors**: File system failures during version cleanup
4. **Silent failures**: Missing parents during resolution, failed cleanup operations

Without proper error handling and recovery strategies, these failures can lead to:
- Data loss (orphaned schemas, missing metadata)
- Inconsistent state (partial writes, stale indices)
- Silent corruption (swallowed errors with no visibility)

---

## Decision

### 1. Error Classification

We classify errors into three categories:

#### **Unrecoverable Errors** (Fail Fast)
Return error to caller, abort operation:
- Database corruption (invalid rkyv data)
- Missing required metadata
- Circular inheritance
- Invariant violations

**Recovery**: User intervention required, operation must be retried.

#### **Recoverable Errors** (Warn & Continue)
Log warning, use safe default, continue:
- Missing parent during resolution → Use empty properties
- Failed version cleanup → Continue with save, warn about disk growth
- Optional timestamp unavailable → Use None, continue

**Recovery**: Automatic, degraded functionality acceptable.

#### **Expected Failures** (Silent)
Normal control flow, no logging needed:
- Schema not found by ID → Return None
- Property not in bank → Return None
- Empty database queries → Return empty vec

**Recovery**: None needed, expected behavior.

### 2. Error Logging Standards

#### **Warn-Level Logging**
For recoverable errors that indicate potential problems:

```rust
// Example: Version cleanup failure
.unwrap_or_else(|error| {
    tracing::warn!(
        version = %old_version.as_u64(),
        table = "BANK_PROPERTY_BY_ID",
        %error,
        "Failed to scan old property bank version for cleanup, \
         retention may not work correctly"
    );
    Vec::new()
})
```

#### **Error-Level Logging**
For unrecoverable errors before returning:

```rust
// Example: Corruption detection
Err(e) => {
    tracing::error!(
        schema_id = %id,
        %error,
        "Schema metadata corruption detected: entry exists but \
         metadata is missing"
    );
    Err(SchemaQueryError::Corruption(format!(...)))
}
```

### 3. Recovery Strategies by Operation

#### **Property Bank Save with Version Retention**

**Error**: Version cleanup scan fails
**Impact**: Old versions not deleted, disk growth continues
**Recovery**:
1. Log warning with version and table details
2. Continue with save operation (new version still written)
3. Manual cleanup may be needed later

**Prevention**: Integration tests validate retention works correctly

#### **Schema Resolution with Missing Parent**

**Error**: Parent not in resolved_cache or known_parents
**Impact**: Child schema gets empty parent properties
**Recovery**:
1. Log warning with schema_id and parent_id
2. Use empty properties for parent
3. Schema saves successfully but loses inherited properties

**Prevention**: Extender ensures all parents are available before resolution

#### **Database Corruption Detection**

**Error**: Invalid rkyv bytes, missing metadata
**Impact**: Cannot deserialize schema/property bank
**Recovery**:
1. Return corruption error immediately
2. Log error with entity ID and corruption type
3. User must restore from backup or delete corrupted entity

**Prevention**: Corruption detection tests validate error paths

#### **File-Based Ingestion Failures**

**Error**: File read failure, parse error, missing property bank
**Impact**: Schemas not loaded from files
**Recovery**:
1. Return error to caller (SchemaService)
2. Caller logs error and continues with other files
3. Failed schemas remain stale in database

**Prevention**: Integration tests cover malformed files, missing dependencies

### 4. Observability

#### **Structured Logging**
All error logs include relevant context:
- Entity IDs (schema_id, property_id, parent_id)
- Operation context (table name, version number)
- Error details (error message, error type)
- User guidance (what to check, how to recover)

#### **Error Aggregation**
Consider adding metrics for:
- Version cleanup failure count
- Missing parent warnings
- Corruption detection count
- Staleness check failures

### 5. Testing Strategy

#### **Integration Tests** (`tests/schema_cqrs.rs`)
- ✅ Corruption detection (4 tests)
- ✅ Missing entity handling (3 tests)
- ✅ Edge case validation (5 tests)

#### **Property Bank Retention Test**
Validates cleanup works correctly, file size bounded.

#### **Parent Resolution Test**
Future: Add test for missing parent warning (simulate Extender bug).

---

## Consequences

### Positive

✅ **Clear error boundaries**: Unrecoverable vs recoverable vs expected
✅ **Improved observability**: Structured logging with context
✅ **Graceful degradation**: System continues with warnings where safe
✅ **Testable error paths**: Comprehensive corruption detection tests
✅ **User guidance**: Error messages explain impact and recovery steps

### Negative

⚠️ **Partial failures possible**: Cleanup can fail silently with warning
⚠️ **Log volume**: Warnings may increase in degraded environments
⚠️ **Manual recovery needed**: Some errors require user intervention

### Mitigation

- **Monitoring**: Track warning frequency to detect systemic issues
- **Documentation**: Clear recovery procedures in error messages
- **Testing**: Comprehensive integration tests for all error paths
- **Metrics**: Future work to aggregate error counts for alerting

---

## Implementation Status

### Completed

✅ **Tracing added for version cleanup failures** (command.rs:300-325)
✅ **Warning added for missing parent resolution** (resolver.rs:106-122)
✅ **Corruption detection tests** (4 tests in schema_cqrs.rs)
✅ **Error recovery documentation** (this ADR)

### Future Work

- [ ] Add metrics for error aggregation
- [ ] Document manual recovery procedures for each error type
- [ ] Add missing parent resolution integration test
- [ ] Consider event persistence for audit trail (see ADR-015 proposal)

---

## Alternatives Considered

### Alternative 1: Fail Fast on All Errors
**Approach**: Return error for any failure, including version cleanup
**Rejected**: Too brittle - version cleanup failure shouldn't block schema saves

### Alternative 2: Silent Failures (Status Quo)
**Approach**: Use `unwrap_or_default()` without logging
**Rejected**: Hides bugs and makes debugging impossible

### Alternative 3: Event-Driven Error Recovery
**Approach**: Emit error events for async recovery processes
**Deferred**: Over-engineered for current needs, consider for P2/P3

---

## Technical Validation

### Error Detection Tests
- ✅ **Corruption tests** (4 tests): Validate error detection works
- ✅ **Missing entity tests** (3 tests): Verify None returns
- ✅ **Edge case tests** (5 tests): Cover boundary conditions

### Tracing Validation
- ✅ **Manual testing**: Trigger cleanup failures, verify logs appear
- ✅ **Code review**: All `.unwrap_or*` patterns reviewed for logging
- ⚠️ **Future**: Add integration test that validates warning messages

### Performance Impact
- ✅ **No runtime overhead** when errors don't occur (zero-cost abstractions)
- ✅ **Minimal overhead** when errors occur (single log line)
- ✅ **No allocations** in happy path

---

## References

- **Implementation**: `lithos-core/src/schema/`
  - `adapter/command.rs` - Version cleanup error handling
  - `resolver.rs` - Missing parent warning
  - `adapter/query.rs` - Corruption detection

- **Tests**: `lithos-core/tests/schema_cqrs.rs`
  - `corruption` module - Corruption detection tests
  - `critical` module - Edge case tests
  - `staleness` module - Missing entity tests

- **Related Plans**: `schema-cqrs-improvements-plan.md` - Week 3 P1 tasks
