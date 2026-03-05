# Config Module Adversarial Review

## Critical Issues Found

### 🚨 CRITICAL: Race Condition in Version Allocation

**Location**: `adapter/command.rs::next_version()`

**Issue**: `next_version()` performs a non-atomic read-compute-write:
1. Scan for max version
2. Compute next = max + 1
3. Write new config

**Race Scenario**:
```
Thread A: scan → max=5
Thread B: scan → max=5
Thread A: compute next=6, write version 6
Thread B: compute next=6, write version 6 (COLLISION!)
```

**Impact**: **CRITICAL DATA LOSS**
- Two concurrent rebuilds get same version number
- Second write silently overwrites first
- Lost configuration snapshot
- No error returned to caller

**Evidence**:
```rust
// adapter/command.rs:129-159
fn next_version(&self, vault_id: VaultId) -> Result<Version, Self::Error> {
    let max_version = self.db.scan_range::<Config>(CONFIG_VERSIONS, &prefix)?
        .into_iter()
        .filter_map(|(key, _)| { ... })
        .max();

    match max_version {
        Some(v) => v.next().map_err(...),  // ← Non-atomic!
        None => Ok(Version::initial()),
    }
}
```

**Proof of Vulnerability**:
- No transaction wrapping `next_version()` + `record_config()`
- `rebuild_merged()` calls them separately:
  ```rust
  let version = self.next_version(vault_id)?;  // ← Read
  // ... (other work happens here)
  self.command_port.record_config(vault_id, &merged)?;  // ← Write
  ```

**Fix Required**: Atomic read-modify-write transaction
```rust
fn next_version_and_record(
    &self,
    vault_id: VaultId,
    config: &Config,
) -> Result<Version, Self::Error> {
    self.db.read_write_unit_of_work(|tx| {
        let max = scan_max_version(tx, vault_id)?;
        let next = max.map_or(Version::initial(), |v| v.next()?);
        tx.put(CONFIG_VERSIONS, &format!("{}:{}", vault_id, next.value()), config)?;
        Ok(next)
    })
}
```

---

### 🚨 CRITICAL: Version Overflow Panics on Increment

**Location**: `aggregate.rs::Version::next()`

**Issue**: `next()` uses checked arithmetic but returns Result. If max version (u64::MAX) is reached, `next()` returns Err, but callers may not handle this properly.

**Evidence**:
```rust
// adapter/command.rs:151-157
Some(v) => v.next().map_err(|_err| {
    DbError::Serialization(
        "config version overflow - vault has exceeded maximum rebuilds".into(),
    )
}),
```

**Impact**: MEDIUM
- Vault becomes permanently unusable after u64::MAX rebuilds
- Error message is misleading (not a serialization error)
- No recovery path

**Likelihood**: LOW (u64::MAX = 18 quintillion rebuilds)

**Fix**: Better error type, document limitation in user-facing docs

---

### ⚠️ HIGH: Silent Data Loss on Concurrent Global/Vault Updates

**Location**: `adapter/command.rs::record_global()`, `record_vault()`

**Issue**: Recording Global or Vault configs can overwrite each other if versions collide.

**Race Scenario**:
```
Global v1 exists in DB
Thread A: Updates global.toml → creates Global v2
Thread B: Updates global.toml → creates Global v2 (same version!)
Thread A: Writes GLOBAL_CONFIG["2"] = Global{log_level: Debug}
Thread B: Writes GLOBAL_CONFIG["2"] = Global{log_level: Info}  // ← Overwrites!
```

**Root Cause**: Global/Vault versioning is managed by the domain aggregate, not by atomic DB operations.

**Evidence**:
```rust
// global.rs/vault.rs: version is part of the domain object
pub struct Global {
    version: GlobalVersion,  // ← Set at construction time
    // ...
}
```

**Impact**: HIGH
- Silent data loss (no error returned)
- Last writer wins
- No version conflict detection

**Fix Required**: Either:
1. Use optimistic locking (check version before write)
2. Use atomic sequence generators (DB-level)
3. Document that concurrent global/vault updates are unsupported

---

### ⚠️ HIGH: Metadata Desync After Failed Writes

**Location**: `adapter/command.rs::record_global()`, `record_vault()`

**Issue**: `batch_write()` is atomic within the transaction, but if the transaction succeeds partially (config written, metadata fails), we get inconsistent state.

**Evidence**:
```rust
// adapter/command.rs:55-58
self.db.batch_write(|tx| {
    tx.put(GLOBAL_CONFIG, &version_key, config)?;
    tx.put(CONFIG_METADATA, &metadata_key, &metadata)  // ← If this fails?
})
```

**Mitigation**: redb transactions are atomic, so this is safe. But we should verify redb guarantees.

**Action**: Add integration test to verify atomicity

---

### ⚠️ MEDIUM: Active Version Computation is Expensive

**Location**: `adapter/query.rs::get_active_version()`

**Issue**: Every `find()` call performs a full table scan to compute max version.

**Performance**:
```rust
// adapter/query.rs:78-101
fn get_active_version(&self, vault_id: VaultId) -> Result<Option<Version>, Self::Error> {
    let max_version = self.db
        .scan_range::<Config>(CONFIG_VERSIONS, &prefix)?  // ← Full scan!
        .into_iter()
        .filter_map(|(key, _)| { ... })
        .max();
    // ...
}
```

**Impact**:
- O(n) where n = number of versions per vault
- Happens on every config read
- If vault has 1000 versions → 1000 key scans per `find()`

**Workaround**: Users won't have thousands of versions in practice (rebuilds are rare)

**Future Optimization**: Cache active version in memory or denormalize

---

### ⚠️ MEDIUM: No Version GC Strategy

**Issue**: Versions accumulate forever. No mechanism to prune old versions.

**Impact**:
- Database grows unbounded
- Scan performance degrades over time
- No cleanup mechanism

**Recommendation**: Add version garbage collection:
- Keep last N versions
- Keep versions newer than T days
- Manual prune command

---

### ⚠️ MEDIUM: Missing Index on CONFIG_METADATA

**Issue**: `is_global_stale()` and `is_vault_stale()` perform two operations:
1. Scan for max version (to get latest)
2. Lookup metadata by version

**Problem**: If we already scanned and found max version, why scan again?

**Evidence**:
```rust
// adapter/query.rs:175-208
fn is_global_stale(...) -> Result<bool, Self::Error> {
    let max_version = self.db.scan_range::<Global>(GLOBAL_CONFIG, "")?  // ← Scan 1
        .into_iter()
        .filter_map(|(key, _)| { ... })
        .max();

    let metadata_key = format!("global:{}", latest_version.value());
    let stored = self.db.get_owned::<ConfigMetadata>(CONFIG_METADATA, &metadata_key)?;  // ← Scan 2
}
```

**Optimization**: Return version AND config from first scan, avoiding second lookup.

---

### ⚠️ LOW: Timestamp Precision Loss

**Location**: `aggregate.rs::Timestamp`

**Issue**: Timestamps are compared using `as_secs()`, losing sub-second precision.

**Evidence**:
```rust
// adapter/query.rs:214-216
if file_created.as_secs() != stored_created.as_secs() {
    return Ok(true); // Stale: created_at mismatch
}
```

**Impact**: LOW
- False negatives: File modified within same second not detected
- False positives: None

**Likelihood**: LOW (filesystem mtimes typically have second precision)

**Fix**: Use nanosecond precision if available

---

### ⚠️ LOW: String Allocation in Hot Paths

**Location**: All key formatting operations

**Issue**: Every DB operation allocates strings for keys.

**Evidence**:
```rust
// adapter/command.rs:51
let version_key = config.version().value().to_string();
let metadata_key = format!("global:{}", config.version().value());

// adapter/command.rs:75
let key = format!("{}:{}", vault_id, config.version().value());
```

**Impact**: LOW (micro-optimization)
- Allocation overhead on every operation
- GC pressure

**Future Optimization**: Use `itoa` or stack-allocated buffers

---

### ⚠️ LOW: Parse Errors Silently Ignored

**Location**: `adapter/query.rs` version scanning

**Issue**: Malformed keys are silently dropped via `filter_map()`.

**Evidence**:
```rust
// adapter/query.rs:93-98
.filter_map(|(key, _)| {
    key.strip_prefix(&prefix)
        .and_then(|v| v.parse::<u64>().ok())  // ← .ok() swallows error
        .and_then(|v| Version::try_from(v).ok())  // ← .ok() swallows error
})
```

**Impact**: LOW
- Corrupted keys are ignored (defensive)
- No observability into corruption

**Fix**: Log warnings for unparseable keys

---

## Edge Cases to Test

### Concurrency

1. **Two threads rebuild same vault simultaneously**
   - Expected: Both succeed with different versions
   - Actual: Both get same version, second overwrites first ❌

2. **Rebuild during active version scan**
   - Expected: Read committed or read uncommitted behavior
   - Actual: Depends on redb isolation level (verify!)

3. **Multiple global config updates**
   - Expected: Sequential versioning
   - Actual: Version collisions possible ❌

### Boundary Conditions

4. **Version overflow (u64::MAX)**
   - Expected: Return error
   - Actual: Returns error, but caller must handle ✓

5. **Empty database (no versions)**
   - Expected: `get_active_version()` returns None
   - Actual: Returns None ✓

6. **Vault with single version**
   - Expected: Version 1 is active
   - Actual: Works correctly ✓

7. **Concurrent staleness checks**
   - Expected: Eventually consistent
   - Actual: Safe (read-only) ✓

### Data Integrity

8. **Corrupted version string in key**
   - Expected: Skip or error
   - Actual: Silently skipped ⚠️

9. **Metadata missing for existing version**
   - Expected: Treat as stale
   - Actual: Returns true (stale) ✓

10. **Version key exists but deserialization fails**
    - Expected: Return error
    - Actual: Likely panics or returns error (verify!)

### Performance

11. **Vault with 10,000 versions**
    - Expected: Slow but functional
    - Actual: O(n) scan on every read ⚠️

12. **Concurrent reads during scan**
    - Expected: All succeed
    - Actual: Likely safe (verify redb MVCC)

### Recovery

13. **Partial write (transaction rollback)**
    - Expected: Atomic (all or nothing)
    - Actual: redb guarantees atomicity ✓

14. **Database corruption recovery**
    - Expected: redb handles this
    - Actual: Outside our scope ✓

## Recommendations

### Immediate (P0) - Must Fix Before Production

1. **Fix version allocation race condition**
   - Implement atomic read-modify-write in `next_version()`
   - Add concurrency test

2. **Add version conflict detection for Global/Vault**
   - Optimistic locking or sequence generator
   - Return error on collision

### High Priority (P1) - Fix Soon

3. **Add integration test for concurrent rebuilds**
   - Verify race conditions are handled
   - Verify transaction atomicity

4. **Add version GC mechanism**
   - Prune old versions
   - Configurable retention policy

5. **Document concurrency limitations**
   - Warn users about concurrent updates
   - Provide usage guidelines

### Medium Priority (P2) - Nice to Have

6. **Optimize active version lookup**
   - Cache in memory
   - Or add denormalized active pointer

7. **Add observability for parse errors**
   - Log warnings for corrupted keys
   - Metrics for scan performance

8. **Use nanosecond timestamp precision**
   - Reduce false negatives

### Low Priority (P3) - Future Work

9. **Optimize string allocations**
   - Use `itoa` or stack buffers
   - Profile to verify impact

## Test Coverage Gaps

### Missing Unit Tests

- [ ] Concurrent `next_version()` calls (race condition)
- [ ] Version overflow handling
- [ ] Corrupted key formats in tables
- [ ] Metadata desync scenarios
- [ ] Large version counts (performance)

### Missing Integration Tests

- [ ] Concurrent rebuilds on same vault
- [ ] Rebuild during active read
- [ ] Transaction atomicity verification
- [ ] Recovery from partial writes

### Missing Property Tests

- [ ] Version monotonicity under concurrency
- [ ] Scan returns max version (fuzz keys)
- [ ] Metadata consistency invariants

## Conclusion

**Overall Assessment**: The refactoring is architecturally sound but has **critical race conditions** that must be fixed before production use.

**Risk Level**: 🔴 HIGH
- Version allocation race condition is a **data loss bug**
- No concurrency testing coverage
- Performance concerns at scale

**Recommendation**:
1. Fix P0 issues immediately
2. Add concurrency tests
3. Document limitations
4. Plan for P1 fixes in next iteration
