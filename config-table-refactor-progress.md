# Config Table Refactoring - Implementation Progress

**Started:** 2026-03-04
**Status:** IN PROGRESS (Phase 2 of 9 complete)
**Branch:** main (commit 2d2361e7)

---

## Completed Phases

### ✅ Phase 1: Add version fields to Global and Vault types (Commit: 2d2361e7)

**Changes:**
- Added `version: GlobalVersion` field to `Global` struct
- Added `version: VaultVersion` field to `Vault` struct
- Added `version()` getter methods
- Updated constructors to accept version parameter
- Updated `TryFrom<&RawConfig>` implementations
- Fixed test `vault_new_constructs_with_given_values`
- Added clippy exception for `Global::new` (domain constructor pattern)

**Tests:** ✅ All global/vault tests passing (25 tests)

### ✅ Phase 2: Create new table definitions (Commit: pending)

**Changes:**
- Added `GLOBAL_CONFIG` table: `"{version}"` → `Global`
- Added `VAULT_CONFIG` table: `"{vault_id}:{version}"` → `Vault`
- Added `CONFIG_VERSIONS` table: `"{vault_id}:{version}"` → `Config`
- Updated `CONFIG_METADATA` docs: keys now include version
- Deprecated old tables: `CONFIG`, `MERGED_CONFIG_VERSIONS`, `MERGED_CONFIG_ACTIVE`
- Fixed documentation formatting (backticks, punctuation)

**Impact:** 57 deprecation warnings (expected - code still using old tables)

---

## Remaining Phases

### Phase 3: Update Command port methods

**Current signatures (OLD):**
```rust
fn record_global(&self, config: &Global, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<(), Self::Error>;
fn record_vault(&self, vault_id: VaultId, config: &Vault, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<(), Self::Error>;
fn record_merged(&self, vault_id: VaultId, version: Version, config: &Config) -> Result<(), Self::Error>;
fn activate_version(&self, vault_id: VaultId, target: ActivationTarget) -> Result<Version, Self::Error>;
```

**New signatures (PROPOSED):**
```rust
// Version comes from config.version() now
fn record_global(&self, config: &Global, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<(), Self::Error>;
fn record_vault(&self, vault_id: VaultId, config: &Vault, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<(), Self::Error>;

// Renamed: merged → config
fn record_config(&self, vault_id: VaultId, config: &Config) -> Result<(), Self::Error>;

// REMOVED: activate_version (active = max version from scan)
```

**Files to update:**
- `lithos-core/src/config/ports.rs` - trait definitions
- `lithos-core/src/config/command.rs` - facade implementation

### Phase 4: Update Query port methods

**Current signatures (OLD):**
```rust
fn find_merged(&self, vault_id: VaultId, version: Version) -> Result<Option<Config>, Self::Error>;
fn find_vault_id_by_path(&self, vault_root: &VaultRoot) -> Result<Option<VaultId>, Self::Error>;
fn get_active_version(&self, vault_id: VaultId) -> Result<Option<Version>, Self::Error>;
fn get_global(&self) -> Result<Option<Global>, Self::Error>;
fn get_vault(&self, vault_id: VaultId) -> Result<Option<Vault>, Self::Error>;
fn with_archived<R, F>(&self, vault_id: VaultId, version: Version, f: F) -> Result<Option<R>, Self::Error>;
```

**New signatures (PROPOSED):**
```rust
// Add version parameter (defaults to active if None)
fn get_global(&self, version: Option<GlobalVersion>) -> Result<Option<Global>, Self::Error>;
fn get_vault(&self, vault_id: VaultId, version: Option<VaultVersion>) -> Result<Option<Vault>, Self::Error>;

// Renamed: find_merged → find_config
fn find_config(&self, vault_id: VaultId, version: Option<Version>) -> Result<Option<Config>, Self::Error>;

// NEW: Scan CONFIG_VERSIONS for max version
fn get_active_version(&self, vault_id: VaultId) -> Result<Option<Version>, Self::Error>;

// Renamed: find_merged → find_config
fn with_archived<R, F>(&self, vault_id: VaultId, version: Version, f: F) -> Result<Option<R>, Self::Error>;

// Unchanged
fn find_vault_id_by_path(&self, vault_root: &VaultRoot) -> Result<Option<VaultId>, Self::Error>;

// Keep staleness methods (unchanged)
fn is_global_stale(&self, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<bool, Self::Error>;
fn is_vault_stale(&self, vault_id: VaultId, created_at: Option<Timestamp>, modified_at: Timestamp) -> Result<bool, Self::Error>;
```

**Files to update:**
- `lithos-core/src/config/ports.rs` - trait definitions
- `lithos-core/src/config/query.rs` - facade implementation

### Phase 5: Implement CommandAdapter with new tables

**Changes needed:**
- `record_global()`: Write to `GLOBAL_CONFIG["{version}"]` instead of `CONFIG["global"]`
- `record_vault()`: Write to `VAULT_CONFIG["{vault_id}:{version}"]` instead of `CONFIG["{vault_id}"]`
- `record_config()`: Write to `CONFIG_VERSIONS["{vault_id}:{version}"]` instead of `MERGED_CONFIG_VERSIONS`
- Remove `activate_version()` method
- Update `next_version()` to scan `CONFIG_VERSIONS` for max version
- Update metadata keys to include version: `"global:{version}"`, `"{vault_id}:{version}"`

**Files to update:**
- `lithos-core/src/config/adapter/command.rs`

### Phase 6: Implement QueryAdapter with new tables

**Changes needed:**
- `get_global()`: Read from `GLOBAL_CONFIG["{version}"]` or scan for latest
- `get_vault()`: Read from `VAULT_CONFIG["{vault_id}:{version}"]` or scan for latest
- `find_config()`: Read from `CONFIG_VERSIONS["{vault_id}:{version}"]` or scan for latest
- `get_active_version()`: Scan `CONFIG_VERSIONS` for max version with prefix `"{vault_id}:"`
- `with_archived()`: Use `CONFIG_VERSIONS` instead of `MERGED_CONFIG_VERSIONS`
- Update staleness metadata keys

**Files to update:**
- `lithos-core/src/config/adapter/query.rs`

### Phase 7: Update ConfigService

**Changes needed:**
- Use `config.version()` when recording configs
- Handle version incrementing for Global/Vault independently
- Use `get_active_version()` to find latest config
- Update to use `record_config()` instead of `record_merged()`

**Files to update:**
- `lithos-core/src/application/config.rs`
- `lithos-core/src/config/ingest.rs` (may need updates)

### Phase 8: Update all tests

**Affected test files:**
- `lithos-core/src/config/adapter/command.rs` - 4 tests
- `lithos-core/src/config/adapter/query.rs` - 10 tests
- `lithos-core/src/config/command.rs` - 11 tests
- `lithos-core/src/config/query.rs` - 3 tests
- Integration tests in `tests/config_flow.rs`

**Changes needed:**
- Update to use new table constants
- Update to use `record_config()` instead of `record_merged()`
- Update to use `find_config()` instead of `find_merged()`
- Remove tests for `activate_version()`
- Add tests for `get_active_version()` scanning logic

### Phase 9: Remove old tables and cleanup

**Final cleanup:**
- Remove `#[allow(deprecated)]` from old code
- Remove `CONFIG`, `MERGED_CONFIG_VERSIONS`, `MERGED_CONFIG_ACTIVE` table definitions
- Remove old `rebuild_merged()` method (if not needed)
- Update documentation
- Create ADR documenting the refactoring

---

## Deprecation Warnings (57 total)

Current usage of deprecated constants:

- `CONFIG`: 18 usages
  - `adapter/command.rs`: 6
  - `adapter/query.rs`: 2
  - `command.rs`: 6
  - `query.rs`: 4

- `MERGED_CONFIG_VERSIONS`: 9 usages
  - `adapter/command.rs`: 1
  - `adapter/query.rs`: 2
  - `command.rs`: 4
  - `query.rs`: 2

- `MERGED_CONFIG_ACTIVE`: 12 usages
  - `adapter/command.rs`: 4
  - `adapter/query.rs`: 2
  - `command.rs`: 6

- Dead code warnings: 3 (new tables not yet used)
  - `GLOBAL_CONFIG`, `VAULT_CONFIG`, `CONFIG_VERSIONS`

---

## Testing Strategy

1. **Phase-by-phase testing**: Run tests after each phase completion
2. **Parallel writes during migration**: Write to both old and new tables temporarily
3. **Integration tests first**: Ensure end-to-end flow works before removing old tables
4. **Rollback plan**: Keep old tables until all tests pass

---

## Risks & Mitigation

**Risk 1: Breaking existing code**
- Mitigation: Deprecation warnings, phased migration, parallel writes

**Risk 2: Version scanning performance**
- Mitigation: Measure scan performance, add `is_active` field if needed

**Risk 3: Test failures**
- Mitigation: Fix tests incrementally, keep old tables until verified

---

## Next Actions

1. Add `#[allow(deprecated)]` to all files using old constants
2. Implement Phase 3: Update Command port methods
3. Implement Phase 4: Update Query port methods
4. Continue with Phases 5-9

---

## References

- Design document: `config-table-design-fix.md`
- Original issue: User identified 3 critical design flaws
- Commits: 2d2361e7 (Phase 1), pending (Phase 2)
