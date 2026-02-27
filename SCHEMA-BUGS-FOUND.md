# Schema Module - Actual Bugs & Issues Found

**Date**: 2026-02-27
**Method**: Adversarial deep code review
**Total Issues**: 29 (3 critical, 5 high, 6 medium, 7 low, 5 test gaps, 3 architectural notes)

---

## 🔴 CRITICAL BUGS (Fix Immediately)

### CRITICAL-001: Silent Integer Overflow in Rich Options Sorting
**Location**: `lithos-core/src/schema/raw.rs:709`
**Severity**: 🔴 CRITICAL - Silent data corruption

**Bug**:
```rust
let order = entry.order.unwrap_or_else(|| {
    u32::try_from(idx).unwrap_or(u32::MAX)  // SILENT FAILURE!
});
```

**Problem**: If option list index exceeds u32::MAX (>4 billion entries), multiple entries get assigned `u32::MAX`, breaking sort stability and causing non-deterministic ordering.

**Fix**:
```rust
let order = entry.order.unwrap_or_else(|| {
    u32::try_from(idx).expect("Option list has >4 billion entries - exceeds u32::MAX")
});
```

**Why It Matters**: Silent data corruption in option ordering. Extremely unlikely in practice (4 billion options), but violates fail-fast principles.

---

### CRITICAL-002: UUID String Allocation in Hot Path
**Location**: `lithos-core/src/schema/adapter/command.rs:151, 196`
**Severity**: 🔴 CRITICAL - Performance violation

**Bug**:
```rust
let id_key = schema.id().into_uuid().to_string();  // 36-byte allocation per save!
batch.put(SCHEMA_BY_ID, id_key.as_str(), &stored)?;
```

**Problem**: Allocates 36 bytes per schema save/delete operation. Violates documented "UUID to_string() in hot paths" anti-pattern.

**Fix**: Options:
1. Change Database API to accept `&Uuid` directly (best)
2. Use thread-local buffer for UUID formatting
3. Add adapter-level caching

**Why It Matters**: Per anti-patterns doc, this is a documented "must avoid" pattern. Batch operations will be slow.

---

### CRITICAL-003: FileSpec Validation Logic Bug
**Location**: `lithos-core/src/schema/property_spec.rs:547`
**Severity**: 🔴 CRITICAL - Business logic error

**Bug**:
```rust
if value_path == dir_path || !value_path.starts_with(dir_path) {
    return Err(SchemaError::InvalidDirectoryPath(...));
}
```

**Problem**: The `value_path == dir_path` check allows exact directory path, but constraint is "must be IN directory", not "must be AT directory level".

**Example**: `FileSpec { directory: "assets" }` would incorrectly accept `"assets"` as valid.

**Fix**:
```rust
// Reject if not in directory OR if exactly equals directory
if !value_path.starts_with(dir_path) || value_path == dir_path {
    return Err(SchemaError::InvalidDirectoryPath(format!(
        "File {} must be inside (not at) directory {}", value, dir.as_str()
    )));
}
```

**Why It Matters**: Business logic bug - violates the "file must be inside directory" constraint.

---

## 🟠 HIGH SEVERITY (Fix Soon)

### HIGH-001: Regex Recompilation on Every Validation
**Location**: `lithos-core/src/schema/property_spec.rs:985`
**Severity**: 🟠 HIGH - Major performance issue

**Bug**:
```rust
// In StringSpec::validate_pattern()
if let Some(pattern) = self.pattern.as_ref() {
    let re = regex::Regex::new(pattern).map_err(...))?;  // RECOMPILE EVERY TIME!
    if !re.is_match(value) { ... }
}
```

**Problem**: Custom user patterns compile regex on EVERY validation call instead of caching. O(n) compilations for n validations.

**Fix**: Cache compiled regex (but this conflicts with zero-copy rkyv requirements). Options:
1. Store `Arc<Regex>` in `StringSpec` (not rkyv-serializable)
2. Use LazyLock with pattern as key
3. Accept the cost (pattern validates at construction, so pattern is guaranteed valid)

**Why It Matters**: Major performance degradation for repeated validations with custom patterns.

**Note**: We fixed built-in formats (E-05) but missed custom user patterns!

---

### HIGH-002: PropertyBank Non-Deterministic Iteration
**Location**: `lithos-core/src/schema/bank.rs:167`, `extender.rs:256`
**Severity**: 🟠 HIGH - Non-determinism

**Bug**: Multiple `#[expect(clippy::iter_over_hash_type)]` suppressions without justification.

**Problem**: PropertyBank property order is non-deterministic due to HashMap iteration, affecting:
- Hash stability
- Debugging (property order changes between runs)
- Test stability

**Fix**: Use `BTreeMap` for `properties` HashMap, or sort before iteration where order matters.

**Why It Matters**: Makes debugging harder, hashes unstable.

---

### HIGH-003: Depth Calculation Bug in Resolver
**Location**: `lithos-core/src/schema/resolver.rs:152`
**Severity**: 🟠 HIGH - Correctness bug

**Bug**:
```rust
fn resolve_parent_depth(...) -> Result<usize, SchemaError> {
    if let Some(_parent) = known_parents.get(&pid) {
        return Ok(1);  // WRONG! Loses parent's actual depth
    }
    // ... DB lookup also returns 1
}
```

**Problem**: DB-fresh parents always get depth=1, ignoring their actual inheritance depth.

**Example**:
- Schema A (depth 1, no parent)
- Schema B extends A (depth 2, stored in DB)
- Schema C extends B (resolving now)
- C should have depth 3, but gets depth 2 because B returns 1

**Fix**: Store depth with schemas, or recursively compute depth from DB parents.

**Why It Matters**: Depth validation is incorrect for multi-level inheritance from DB.

---

### HIGH-004: Missing Override Validation Error Handling
**Location**: `lithos-core/src/schema/property_spec.rs:854`
**Severity**: 🟠 HIGH - Silent failures

**Bug**: `NumberSpec::apply_overrides` can fail (min > max after override), but dereferencer may not handle errors properly.

**Problem**: Override application can silently create invalid specs.

**Fix**: Audit all call sites of `apply_overrides` methods to ensure errors are propagated.

**Why It Matters**: Data integrity - invalid specs could be created.

---

### HIGH-005: PropertyBank Idempotency Doesn't Verify Content
**Location**: `lithos-core/src/schema/bank.rs:261`
**Severity**: 🟠 HIGH - Data integrity

**Bug**:
```rust
Entry::Occupied(_) => {
    // Idempotent success: no event, no version increment
    Ok(())  // DANGEROUS - doesn't verify content matches!
}
```

**Problem**: Registering same ID with different content succeeds silently.

**Example**:
1. Register Property(id=123, name="status", type=string)
2. Register Property(id=123, name="priority", type=number)
3. Second call succeeds, but bank now has stale definition

**Fix**: Add content hash check or version field to Property, error on mismatch.

**Why It Matters**: Data corruption - bank could have wrong property definitions.

---

## 🟡 MEDIUM SEVERITY

### MEDIUM-001: Floating Point Epsilon Hardcoded
**Location**: `lithos-core/src/schema/property_spec.rs:773`
**Issue**: Step validation uses hardcoded `1e-10f64` epsilon, may fail for extreme values.

### MEDIUM-002: Schema vs Property Name Case Inconsistency
**Location**: `aggregate.rs:623` vs `property.rs:647`
**Issue**: SchemaName is lowercase-only `^[a-z0-9_-]+$`, PropertyName allows mixed case `^[A-Za-z_][A-Za-z0-9_-]*$`. No documentation explaining why.

### MEDIUM-003: Missing Inheritance Depth Limit Test
**Location**: `resolver.rs:34`
**Issue**: `INHERITANCE_MAX_DEPTH = 10` defined but no test validates depth=11 fails.

### MEDIUM-004: PropertyRef Parse Error Message Confusing
**Location**: `property.rs:749-752`
**Issue**: Error mentions "format" instead of "name validation" when PropertyName validation fails.

### MEDIUM-005: Duplicate Name Check Race Condition
**Location**: `extender.rs:268-273`
**Issue**: Checks for duplicate names AFTER inserting into name_to_id map.

### MEDIUM-006: Unnecessary to_lowercase() Watch
**Location**: N/A (monitoring item)
**Issue**: No violations found, but watch for future additions of `.to_lowercase()` on pre-validated lowercase-only fields.

---

## 🟢 LOW SEVERITY

### LOW-001: Arc<Property> Cloning in Resolver
**Location**: `resolver.rs:109, 197, 202, 213, 231`
**Issue**: Resolver clones Property from Arc multiple times during merge, could reuse Arc.

### LOW-002: String Instead of Box<str> in Errors
**Location**: `error.rs:34, 55`
**Issue**: Error variants use `String` instead of `Box<str>` for immutable messages.

### LOW-003: Missing #[inline] on Trivial Getters
**Location**: Various (e.g. `bank.rs:329`)
**Issue**: Some simple getters missing `#[inline]` attribute.

### LOW-004: Timestamp Conversion Silently Ignores Overflow
**Location**: `adapter/ingestor.rs:171, 184`
**Issue**: `i64::try_from(duration.as_secs()).ok()` silently ignores overflow beyond 2038.

### LOW-005: SchemaTree.roots API Ergonomics
**Location**: `extender.rs:175`
**Issue**: Returns `&[SchemaId]` but could return `impl Iterator` for better ergonomics.

### LOW-006: Missing Port Error Type Documentation
**Location**: `ports.rs:51, 181`
**Issue**: Associated `Error` types don't specify which errors are transient vs permanent.

### LOW-007: Serde Skip on pending_events
**Location**: `bank.rs:96`
**Issue**: `#[serde(skip)]` means events lost on serialize/deserialize cycle (likely intentional).

---

## 🧪 MISSING TEST COVERAGE

### GAP-001: No Circular Inheritance Test
**Location**: `extender.rs:320`
**Issue**: Cycle detection exists but no test exercises A→B→C→A cycle.

### GAP-002: No Depth Limit Exceeded Test
**Location**: `resolver.rs:92`
**Issue**: No test validates depth > 10 fails with InheritanceDepthExceeded.

### GAP-003: No PropertyBank Idempotency Edge Case Test
**Location**: `bank.rs:265`
**Issue**: Registering same ID with different content not tested.

### GAP-004: No FileSpec Directory Equality Test
**Location**: `property_spec.rs:547`
**Issue**: No test validates that directory path itself fails validation.

### GAP-005: Missing Multi-Value Property Test
**Location**: `property.rs:328`
**Issue**: `validate_value` array handling has limited test coverage.

---

## 🏗️ ARCHITECTURAL NOTES (Not Bugs)

### ARCH-001: Port Abstraction Slightly Leaky
**Location**: `adapter/command.rs:108`
**Note**: `save_batch_with_metadata` is adapter-specific, not in trait.

### ARCH-002: Events Not Persisted
**Location**: `aggregate.rs:79`
**Note**: `pending_events` not serialized - event sourcing incomplete.

### ARCH-003: Zero-Copy Opportunity Missed
**Location**: Query port
**Note**: `find_by_id() -> Option<Schema>` instead of closure-based zero-copy pattern.

---

## 📊 Priority Summary

**Immediate Action Required** (3 critical bugs):
1. CRITICAL-003: FileSpec validation logic bug
2. CRITICAL-002: UUID allocation hot path
3. CRITICAL-001: Integer overflow (unlikely but dangerous)

**Fix Soon** (5 high severity):
1. HIGH-001: Regex recompilation performance
2. HIGH-003: Depth calculation bug
3. HIGH-005: PropertyBank idempotency
4. HIGH-004: Override validation
5. HIGH-002: Non-deterministic iteration

**Backlog** (13 medium/low + 5 test gaps)

**Total Work Estimate**: ~3-5 days for critical+high, ~2 days for medium/low/tests

---

## 🎯 Next Steps

1. **Fix CRITICAL-003** (FileSpec bug) - 30 minutes
2. **Fix CRITICAL-002** (UUID allocation) - 1-2 hours (requires DB API changes)
3. **Fix HIGH-001** (regex caching) - 1-2 hours (architecture decision needed)
4. **Add missing tests** (GAP-002, GAP-004) - 1 hour
5. **Fix HIGH-003** (depth calculation) - 2-3 hours (requires design decision)

This is the ACTUAL work needed, based on code inspection, not review document checkmarks.
