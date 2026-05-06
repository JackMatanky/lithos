# Config Table Design Fix

**Date:** 2026-03-04
**Status:** 🔴 CRITICAL - Design flaw identified
**Severity:** HIGH - Affects data model correctness

---

## Problem Statement

The current config table design has three fundamental flaws:

1. **Misleading terminology**: "MERGED" in table names implies the data is merged, but "merged" should only describe the *action* of merging. The final config is just "Config", not "MergedConfig".

2. **Unnecessary table split**: `MERGED_CONFIG_VERSIONS` stores versioned configs while `MERGED_CONFIG_ACTIVE` stores a pointer to the active version. This creates a denormalized "active pointer" that must be kept in sync.

3. **Missing version history**: Global and Vault configs are stored in a generic type-unsafe `CONFIG` table with only the latest version. Historical configs cannot be reconstructed.

---

## Current Broken Design

```rust
// Type-unsafe mixed table
pub(crate) const CONFIG: TableDefinition<&str, &[u8]> =
    TableDefinition::new("config");
    // Keys: "global" → Global, "{vault_id}" → Vault
    // Problem: Mixes types, can't version

// Versioned final configs
pub(crate) const MERGED_CONFIG_VERSIONS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("merged_config_versions");
    // Keys: "{vault_id}:{version}" → Config

// Denormalized active pointer
pub(crate) const MERGED_CONFIG_ACTIVE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("merged_config_active");
    // Keys: "{vault_id}" → Version
```

**Consequences:**
- ❌ Can't reconstruct Config from Global + Vault at historical versions
- ❌ Type-unsafe `CONFIG` table (runtime type checking required)
- ❌ Redundant active pointer must stay synchronized
- ❌ Confusing "merged" terminology in table names
- ❌ No version history for Global/Vault configs

---

## Proposed Correct Design

### Table Structure

```rust
// Versioned global config (singleton)
pub(crate) const GLOBAL_CONFIG: TableDefinition<&str, &[u8]> =
    TableDefinition::new("global_config");
    // Keys: "{version}" → Global
    // Example: "1" → Global { ... }

// Versioned vault-specific config
pub(crate) const VAULT_CONFIG: TableDefinition<&str, &[u8]> =
    TableDefinition::new("vault_config");
    // Keys: "{vault_id}:{version}" → Vault
    // Example: "abc123:1" → Vault { ... }

// Versioned final config (result of merging global + vault)
pub(crate) const CONFIG_VERSIONS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("config_versions");
    // Keys: "{vault_id}:{version}" → Config
    // Example: "abc123:1" → Config { ... }

// Metadata for staleness checks
pub(crate) const CONFIG_METADATA: TableDefinition<&str, &[u8]> =
    TableDefinition::new("config_metadata");
    // Keys: "global:{version}" → ConfigMetadata
    // Keys: "{vault_id}:{version}" → ConfigMetadata

// Vault path mappings (unchanged)
pub(crate) const VAULT_ID_BY_PATH: TableDefinition<&str, &[u8]> =
    TableDefinition::new("vault_id_by_path");
pub(crate) const VAULT_PATH_BY_ID: TableDefinition<&str, &[u8]> =
    TableDefinition::new("vault_path_by_id");
```

### Active Version Tracking

Instead of a separate `MERGED_CONFIG_ACTIVE` table, track the active version in two ways:

**Option 1: Scan for max version** (simple, no denormalization)
```rust
fn get_active_version(&self, vault_id: VaultId) -> Result<Option<Version>, DbError> {
    let prefix = format!("{vault_id}:");
    let versions: Vec<Version> = self.db
        .scan_range(CONFIG_VERSIONS, &prefix..)
        .filter_map(|(key, _)| {
            key.strip_prefix(&prefix)
                .and_then(|v| v.parse::<u64>().ok())
                .and_then(|v| Version::try_from(v).ok())
        })
        .collect();
    Ok(versions.into_iter().max())
}
```

**Option 2: Store in Config aggregate** (slightly more work, no scan)
```rust
pub struct Config {
    vault_metadata: Metadata,
    version: Version,  // ← Add this field
    is_active: bool,   // ← Add this field
    // ... rest of fields
}
```

**Recommendation:** Use Option 1 initially (simpler, fewer fields), switch to Option 2 if scans become a bottleneck.

---

## Benefits of New Design

1. **Type safety**: Each table stores exactly one type
2. **Version history**: Can reconstruct any Config from Global + Vault at any version
3. **Clear terminology**: No "merged" in table names (it's just "Config")
4. **Single source of truth**: Version is part of the key, no pointer synchronization
5. **Auditability**: Full history of all config changes preserved
6. **Consistency**: Matches how schema module handles versioning

---

## Migration Plan

### Phase 1: Add New Tables (Non-Breaking)
- Add `GLOBAL_CONFIG` and `VAULT_CONFIG` tables
- Implement parallel writes to old and new tables
- Add version to Global and Vault types if not present

### Phase 2: Migrate Queries (Gradual)
- Update Query port to read from new tables
- Keep Command writing to both old and new tables
- Verify correctness in tests

### Phase 3: Remove Old Tables (Breaking)
- Remove old `CONFIG` (mixed types), `MERGED_CONFIG_VERSIONS`, `MERGED_CONFIG_ACTIVE`
- Update Command port to only write to new tables
- Update all references

### Phase 4: Cleanup (Polish)
- Remove migration compatibility code
- Update documentation
- Add ADR documenting the change

---

## Implementation Checklist

### New Types
- [ ] Add `version` field to `Global` type
- [ ] Add `version` field to `Vault` type
- [ ] Update serialization derives

### New Tables
- [ ] Add `GLOBAL_CONFIG` table definition
- [ ] Add `VAULT_CONFIG` table definition
- [ ] Rename `MERGED_CONFIG_VERSIONS` → `CONFIG_VERSIONS`
- [ ] Remove old `CONFIG` table (mixed types)
- [ ] Remove `MERGED_CONFIG_ACTIVE` table
- [ ] Update `CONFIG_METADATA` keys to include version

### Command Port
- [ ] Update `record_global(global, version, created, modified)`
- [ ] Update `record_vault(vault_id, vault, version, created, modified)`
- [ ] Update `record_merged` → `record_config(vault_id, config)`
- [ ] Remove `activate_version` (version is in data itself)

### Query Port
- [ ] Update `get_global(version)` → returns Global at specific version
- [ ] Update `get_vault(vault_id, version)` → returns Vault at specific version
- [ ] Rename `find_merged` → `find_config(vault_id, version)` (uses CONFIG_VERSIONS)
- [ ] Add `get_active_version(vault_id)` → scans CONFIG_VERSIONS for max version
- [ ] Update `with_archived` to use new key format

### Service Layer
- [ ] Update `ConfigService::load()` to use new tables
- [ ] Update version management logic
- [ ] Handle version incrementing for Global/Vault independently

### Tests
- [ ] Update all tests to use new table structure
- [ ] Add tests for version history reconstruction
- [ ] Add tests for active version detection
- [ ] Verify staleness detection still works

---

## Example Usage After Fix

```rust
// Record a new global config (version auto-increments)
let global = Global::default();
let version = GlobalVersion::initial(); // or compute next
command.record_global(&global, version, created_at, modified_at)?;
// Writes to: GLOBAL_CONFIG["{version}"]

// Record a new vault config (version auto-increments per vault)
let vault = Vault::default();
let version = VaultVersion::initial(); // or compute next
command.record_vault(vault_id, &vault, version, created_at, modified_at)?;
// Writes to: VAULT_CONFIG["{vault_id}:{version}"]

// Merge and record final config
let config = Config::build(&global, &vault, vault_id, vault_root)?;
let config_version = Version::initial(); // or compute next
command.record_config(vault_id, &config, config_version)?;
// Writes to: CONFIG_VERSIONS["{vault_id}:{version}"]

// Query active config (scans CONFIG_VERSIONS for max version)
let active_version = query.get_active_version(vault_id)?;
let config = query.find_config(vault_id, active_version)?;

// Reconstruct historical config
let global_v1 = query.get_global(GlobalVersion::from(1))?;
let vault_v2 = query.get_vault(vault_id, VaultVersion::from(2))?;
let reconstructed = Config::build(&global_v1, &vault_v2, vault_id, vault_root)?;
```

---

## References

- **Schema Module**: Uses single `PROPERTY_BANK` table with `{vault_id}:{version}` keys
- **AGENTS.md**: "Store domain types directly; keep conversions mechanical"
- **ADR 003**: Three-shape serialization model (Raw → Domain → Stored)
- **ADR 006**: Persistence & Cache Infrastructure (redb + rkyv)

---

## Risk Assessment

**Risk Level:** MEDIUM
- Requires database schema change (table rename/restructure)
- Affects all config read/write operations
- No data loss (can migrate existing data)
- Can be done gradually with parallel writes

**Mitigation:**
- Phase migration with parallel writes
- Comprehensive test coverage before switching
- Keep old tables until migration verified
- Document rollback procedure
