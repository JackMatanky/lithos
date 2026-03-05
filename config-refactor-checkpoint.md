# Config Table Refactoring - Checkpoint

**Date:** 2026-03-04
**Status:** IN PROGRESS - Adapters updated, compilation errors need fixing
**Token Budget:** ~60k remaining

---

## Completed Work

### ✅ Phase 1-2: Domain Types and Table Definitions (COMMITTED)
- Added version fields to Global and Vault types
- Created new table definitions (GLOBAL_CONFIG, VAULT_CONFIG, CONFIG_VERSIONS)
- Deprecated old tables
- Commits: 2d2361e7, 603734c9

### ✅ Phase 3-4: Port Trait Updates (NOT YET COMMITTED - compilation errors)
- Updated Command port: removed `activate_version`, renamed `record_merged` → `record_config`
- Updated Query port: renamed `find_merged` → `find_config`
- Removed `ActivationTarget` enum
- Updated `CommandState::next_version` to scan for max version

### ✅ Phase 5-6: Adapter Implementation (NOT YET COMMITTED - compilation errors)
- CommandAdapter: all methods use new tables
- QueryAdapter: all methods use new tables
- Added `Config::version()` accessor
- Removed `merged_version_key` helper

---

## Remaining Compilation Errors (16 errors)

### Critical Fixes Needed:

1. **Add Ord trait to Version** (2 errors)
   ```rust
   // In aggregate.rs, Version needs:
   #[derive(Ord, PartialOrd)]
   pub struct Version(u64);
   ```

2. **Fix Config.version() return type** (1 error)
   ```rust
   // Currently returns &AppVersion, should return Version
   pub const fn version(&self) -> Version {
       *self.vault_metadata.version()  // Dereference
   }
   ```

3. **Add type annotations to scan_range calls** (4 errors)
   ```rust
   // Need to specify type parameter:
   .scan_range::<Global>(GLOBAL_CONFIG, "")?
   .scan_range::<Vault>(VAULT_CONFIG, &prefix)?
   ```

4. **Remove unused imports** (2 warnings)
   ```rust
   // Remove: GlobalVersion, VaultVersion from adapter/command.rs
   ```

5. **Update command.rs facade** (~10 errors)
   - Remove `ActivationTarget` import
   - Implement `record_config` instead of `record_merged`
   - Remove `activate_version` method
   - Update tests to use new constants

6. **Update query.rs facade** (~5 errors)
   - Implement `find_config` instead of `find_merged`
   - Update tests to use new constants

7. **Update adapter tests** (~4 errors)
   - Fix tests using old CONFIG constant
   - Update to use new table names

---

## Files Modified (Not Yet Committed)

```
lithos-core/src/config/
├── ports.rs              - Updated trait signatures
├── aggregate.rs          - Added version() accessor (needs Ord fix)
├── adapter/
│   ├── mod.rs            - Removed merged_version_key helper
│   ├── command.rs        - Uses new tables (needs cleanup)
│   └── query.rs          - Uses new tables (needs type annotations)
├── command.rs            - Needs update to match new ports
└── query.rs              - Needs update to match new ports
```

---

## Next Steps (Estimated: 2-3 hours)

### Immediate (to fix compilation):
1. Add `#[derive(Ord, PartialOrd)]` to Version
2. Fix Config.version() to dereference
3. Add type annotations to all scan_range calls
4. Remove unused imports

### After Compilation Fixes:
5. Update command.rs facade (record_config, remove activate_version)
6. Update query.rs facade (find_config)
7. Update all adapter tests
8. Update command.rs tests
9. Update query.rs tests
10. Update ConfigService (application layer)
11. Update integration tests
12. Remove deprecated table definitions

---

## Testing Status

- ✅ Global/Vault unit tests passing (25 tests)
- ❌ Config module tests: compilation blocked
- ❌ Integration tests: compilation blocked

---

## Decision Point

**Options:**
1. **Continue fixing compilation errors** - Complete the refactoring (2-3 hours estimated)
2. **Revert uncommitted changes** - Return to stable state, plan smaller increments
3. **Commit WIP state** - Document current progress, continue later

**Recommendation:** Option 1 - We're close to compilation success. The remaining fixes are mechanical:
- Add Ord derive (1 line)
- Fix version() return (1 line)
- Add type annotations (4 lines)
- Remove unused imports (2 lines)
- Update facades/tests (bulk find-replace for method names)

Once compilation succeeds, we can commit and tackle tests incrementally.
