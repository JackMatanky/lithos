# Lithos Config Module: Critical Issues & Architecture Review

**Date:** 2026-03-04
**Scope:** `lithos-core/src/config/` entire context module
**Status:** Implementation in progress - Hybrid ingestion pattern finalized

---

## Implementation Status Tracker

### Phase 1: Add Metadata Storage (1 hour) - ⏳ NOT STARTED

- [ ] Create `config/adapter/stored.rs` with `ConfigMetadata` type
- [ ] Add `CONFIG_METADATA` table to `config/mod.rs`
- [ ] Add `GlobalVersion` type to `config/global.rs`
- [ ] Add `VaultVersion` type to `config/vault.rs`

### Phase 2: Global Config Path Resolution (30 min) - ⏳ NOT STARTED

- [ ] Create `config/adapter/ingestor.rs`
- [ ] Implement `resolve_global_config_path()` with priority order

### Phase 3: Ingestor with Metadata (1 hour) - ⏳ NOT STARTED

- [ ] Implement `load_vault_config()` with metadata extraction
- [ ] Implement `load_global_config()` with metadata extraction
- [ ] Implement `compute_metadata()` helper

### Phase 4: Query Port Extensions (30 min) - ⏳ NOT STARTED

- [ ] Add `is_global_stale()` to Query trait in `config/ports.rs`
- [ ] Add `is_vault_stale()` to Query trait
- [ ] Add `find_vault_id_by_path()` to Query trait
- [ ] Implement staleness methods in `config/adapter/query.rs`

### Phase 5: Command Updates (30 min) - ⏳ NOT STARTED

- [ ] Update `record_global()` to include metadata parameter
- [ ] Update `record_vault()` to include metadata parameter
- [ ] Implement batch writes for config + metadata

### Phase 6: Service Orchestration (1 hour) - ⏳ NOT STARTED

- [ ] Create `application/config.rs`
- [ ] Implement `ConfigService::load()` with hybrid staleness detection
- [ ] Implement `merge_configs()` helper
- [ ] Update `config/command.rs` to accept pre-built Config

### Phase 7: RawConfig Schema Alignment (30 min) - ⏳ NOT STARTED

- [ ] Add `vault_path`, `name`, `version` fields to `RawConfig`
- [ ] Flatten `RawLogging` with `#[serde(flatten)]`
- [ ] Update `Config::build()` to use new fields
- [ ] Add round-trip tests

### Phase 8: Multi-Format Support (1 hour) - ⏳ DEFERRED

- [ ] Enable figment `json` and `yaml` features in Cargo.toml
- [ ] Update ConfigIngestor to probe multiple formats
- [ ] Test JSON/YAML config files

### Additional Critical Fixes - ⏳ NOT STARTED

- [ ] Fix version overflow (Section 3.1)
- [ ] Add trusted_vaults to Config (Section 3.2)
- [ ] VaultRoot wraps AbsolutePath (Section 3.3)
- [ ] FrontmatterKey uses Box<str> (Section 5.2.1)
- [ ] ConfigUpdated.source uses Box<str> (Section 5.2.2)

**Total Estimated Effort:** ~6 hours (Phases 1-7)

---

## 1. Architecture & Layering Violations

### 1.1 ingest.rs Belongs in Adapter Layer, Not Domain

**Location:** `lithos-core/src/config/ingest.rs` (entire file)
**Severity:** CRITICAL
**Status:** ✅ RESOLVED IN PLAN - Will move to `config/adapter/ingestor.rs` (Phase 2-3)

**Problem:**
The `ingest.rs` module performs real filesystem I/O operations (`path.exists()`, file reading via Figment) and uses Figment (an external infrastructure library) while residing in the domain layer alongside pure domain types like `aggregate.rs`, `ports.rs`, and `query.rs`.

According to the project architecture (AGENTS.md):

- "Application services orchestrate pipelines: Services coordinate File → Raw → Domain → Database"
- "File ingestion MUST use `FileSource` trait"
- Domain modules should not perform I/O

**Current Broken Flow:**

```
Command::rebuild_merged()          # Domain layer
    ↓ [direct call - VIOLATION]
ingest::build_merged_raw()         # Does filesystem I/O
    ↓
Config::build()                    # Domain layer
```

The `Command` CQRS handler (domain) directly calls infrastructure code, violating dependency direction.

**Target Architecture (Matches Schema Module):**

```
ConfigService::load(vault_root)     # Application layer
    ↓
ConfigIngestor::load_vault_config() # Adapter layer (I/O)
ConfigIngestor::load_global_config() # Adapter layer (I/O)
    ↓
Query::is_vault_stale()            # Domain layer (staleness check)
Query::is_global_stale()           # Domain layer (staleness check)
    ↓
Config::build(raw, ...)            # Domain layer (pure validation)
    ↓
Command::rebuild_merged(config)    # Domain layer (pure persistence)
```

**Implementation:** Phase 2-3, 6

---

### 1.2 Command Orchestrates When It Should Be Pure

**Location:** `lithos-core/src/config/command.rs:140`
**Severity:** CRITICAL
**Status:** ✅ RESOLVED IN PLAN - Will refactor in Phase 6

**Problem:**
`Command::rebuild_merged` does three things:

1. Calls `ingest::build_merged_raw(vault_root)` - I/O operation
2. Calls `Config::build(&raw_merged, ...)` - domain logic
3. Persists to database - infrastructure operation

This violates CQRS - commands should accept already-validated input.

**Target Signature:**

```rust
// Current (wrong):
pub fn rebuild_merged(&self, vault_id: VaultId, vault_root: &VaultRoot)
    -> Result<Version, DbError>

// Target (correct):
pub fn rebuild_merged(
    &self,
    vault_id: VaultId,
    vault_root: &VaultRoot,
    config: &Config  // Pre-built by service
) -> Result<Version, ConfigCommandError>
```

**Implementation:** Phase 6

---

## 2. Hybrid Config Loading Strategy

### 2.1 Hybrid Loading Implementation (Schema Module Pattern)

**Location:** `config/ingest.rs`, `config/command.rs`, `config/query.rs`
**Severity:** MAJOR
**Status:** ✅ DESIGN FINALIZED - Implementation Phases 1-7

**Design Decisions Finalized:**

1. **Version-based staleness tracking** (not content hash)
   - Matches schema module's `BankVersion` pattern
   - Three independent version sequences: `GlobalVersion`, `VaultVersion`, `Config::Version`
   - All start at 1, increment independently on change detection

2. **Metadata without version field**
   - `ConfigMetadata { created_at, modified_at, recorded_at }`
   - Versions tracked separately in Global/Vault domain types
   - Staleness compares file timestamps only

3. **File replacement handling**
   - `created_at` change indicates file replaced
   - Treated as regular version increment (no special handling)
   - Cannot differentiate new file vs replaced (acceptable limitation)

4. **Staleness scope**
   - Only check global and vault configs
   - Merged config derived from both (always rebuilt if either stale)
   - No cascading like schema module (configs are independent layers)

**Target Architecture Flow:**

```
┌─────────────────────────────────────────────────────────────────────┐
│              Hybrid Config Loading Flow (Final Design)              │
└─────────────────────────────────────────────────────────────────────┘

ConfigService::load(vault_root)
    │
    ├─► Step 1: Load file metadata via ConfigIngestor
    │   ├─ load_vault_config(vault_root) → (RawConfig, ConfigMetadata)
    │   └─ load_global_config() → Option<(RawConfig, ConfigMetadata)>
    │
    ├─► Step 2: Find or create vault ID
    │   └─ Query::find_vault_id_by_path(vault_root) → VaultId
    │
    ├─► Step 3: Check staleness (like schema module)
    │   ├─ Query::is_vault_stale(vault_id, current_metadata) → bool
    │   └─ Query::is_global_stale(current_metadata) → bool
    │
    ├─► Step 4: If both fresh → return cached
    │   └─ Query::get(vault_id) → Config
    │
    └─► Step 5: If any stale → rebuild from files
        ├─ Merge: Figment(defaults + global_raw + vault_raw)
        ├─ Config::build(merged_raw, vault_id, vault_root)
        └─ Command::rebuild_merged(vault_id, vault_root, config)
```

**Key Implementation Points:**

- **Staleness logic:** Compare stored vs current `modified_at` and `created_at`
- **Metadata storage:** Separate `CONFIG_METADATA` table (key: "global" or vault_id)
- **Service orchestrates:** All staleness logic in `ConfigService::load()`
- **Command is pure:** Accepts pre-built `Config`, no I/O

**Implementation:** Phases 1-6

---

### 2.2 Global Config Path Resolution

**Severity:** MAJOR
**Status:** ✅ DESIGN FINALIZED - Implementation Phase 2

**Global Config Paths (Priority Order):**

| Priority | Path                                 | Description              |
| -------- | ------------------------------------ | ------------------------ |
| 1        | `$LITHOS_GLOBAL_CONFIG`                | Environment override     |
| 2        | `$XDG_CONFIG_HOME/lithos/lithos.toml`  | XDG config home          |
| 3        | `$HOME/.config/lithos/lithos.toml`     | XDG default              |
| 4        | `$HOME/.lithos/lithos.toml`            | Legacy user home         |
| 5        | `/etc/lithos/lithos.toml`              | System-wide              |

**Key Point:** Global and vault configs are completely separate:
- **Vault config:** `$VAULT_ROOT/.lithos/lithos.toml`
- **Global config:** System/user paths (never inside vault)

**Implementation:** Phase 2 - `ConfigIngestor::resolve_global_config_path()`

---

### 2.3 RawConfig Schema Compatibility Gap

**Location:** `config/raw.rs`
**Severity:** CRITICAL
**Status:** ⏳ NOT STARTED - Implementation Phase 7

**Problem:**
Current `RawConfig` doesn't match `schema/config.schema.json`:

**Current (Wrong):**
```rust
pub struct RawConfig {
    pub logging: Option<RawLogging>,  // ← Nested [logging] section
    pub paths: RawPathsConfig,
    pub frontmatter: Option<RawFrontmatter>,
    pub task: Option<RawTaskConfig>,
    pub trusted_vaults: Option<RawTrustedVaults>,
}
```

**Target (Correct):**
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawConfig {
    // Top-level vault metadata (vault configs only)
    pub vault_path: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,

    // Flattened logging - exposes log_level at top level
    #[serde(flatten)]
    pub logging: Option<RawLogging>,

    // Nested configuration sections
    #[serde(default)]
    pub paths: RawPathsConfig,
    pub frontmatter: Option<RawFrontmatter>,
    pub task: Option<RawTaskConfig>,
    pub trusted_vaults: Option<RawTrustedVaults>,
}
```

**Impact:**
- Config files written to schema won't parse
- `log_level` should be top-level (flattened), not nested

**Implementation:** Phase 7

---

## 3. Data Integrity & Edge Cases

### 3.1 Version Overflow Silently Corrupts Data

**Location:** `config/adapter/command.rs:164-165`, `config/command.rs:342-343`
**Severity:** MAJOR
**Status:** ⏳ NOT STARTED

**Problem:**

```rust
let next_version = v.next().unwrap_or_else(|_| Version::initial());
```

When `Version::next()` overflows, it falls back to `Version(1)`, potentially overwriting existing data.

**Required Action:**

```rust
let next_version = v.next()
    .map_err(|_| DbError::Serialization(
        "config version overflow - vault has exceeded maximum rebuilds".into()
    ))?;
```

**Implementation:** Additional critical fixes (all version types)

---

### 3.2 Trusted Vaults Configured but Never Used

**Location:** `config/raw.rs:36`, `config/aggregate.rs:100-141`
**Severity:** MAJOR
**Status:** ⏳ NOT STARTED

**Problem:**
`RawConfig.trusted_vaults` is parsed but ignored in `Config::build()`.

**Required Action:**

1. Add `trusted_vaults: Option<TrustedVaults>` field to `Config` struct
2. Populate it from `raw.trusted_vaults` in `Config::build()`
3. Add validation that paths are absolute
4. Document that this is primarily for global config

**Implementation:** Additional critical fixes

---

### 3.3 VaultRoot Lacks Critical Validation

**Location:** `config/vault.rs:360-368`
**Severity:** MAJOR
**Status:** ⏳ NOT STARTED

**Problem:**
`VaultRoot` wraps `PathBuf` directly, but should use `AbsolutePath` for validation.

**Current:**
```rust
pub struct VaultRoot(PathBuf);
```

**Target:**
```rust
pub struct VaultRoot(AbsolutePath);
```

**Implementation:** Additional critical fixes

---

### 3.4 Multi-Format Support

**Location:** `config/ingest.rs:61,67`
**Severity:** MAJOR
**Status:** ⏳ DEFERRED TO PHASE 8

**Problem:**
Only TOML supported despite docs claiming JSON/YAML.

**Required Action:**
Enable figment `json` and `yaml` features, update ConfigIngestor to probe multiple formats.

**Implementation:** Phase 8 (after core refactoring complete)

---

## 4. Code Quality Issues

### 4.1 String Allocation Anti-Patterns (AGENTS.md Violations)

**Locations:** Multiple files
**Severity:** MINOR
**Status:** ⏳ NOT STARTED

**Pattern 1: `.to_owned().into()` on string literals**

Locations:
- `paths.rs:641`: "file name must not contain path separators"
- `task.rs:378,385,494,507`: Various validation messages
- `value.rs:432`: "field name must be ASCII alphanumeric"

**Correct Pattern:**
```rust
// WRONG
message: "text".to_owned().into(),

// CORRECT
message: "text".into(),
```

**Pattern 2: `.to_owned().into_boxed_str()` on &str**

```rust
// WRONG
Ok(Self(text.to_owned().into_boxed_str()))

// CORRECT
Ok(Self(Box::from(text)))
// or
Ok(Self(text.into()))
```

---

## 5. Type Safety Issues

### 5.1 FrontmatterKey Uses String Instead of Box<str>

**Location:** `config/frontmatter.rs:152`
**Severity:** MINOR
**Status:** ⏳ NOT STARTED

**Current:**
```rust
pub struct FrontmatterKey(String);
```

**Target:**
```rust
pub struct FrontmatterKey(Box<str>);
```

**Implementation:** Additional critical fixes

---

### 5.2 ConfigUpdated.source Uses String Instead of Box<str>

**Location:** `config/events.rs:60`
**Severity:** MINOR
**Status:** ⏳ NOT STARTED

**Current:**
```rust
pub struct ConfigUpdated {
    pub source: String,
    pub timestamp: i64,
}
```

**Target:**
```rust
pub struct ConfigUpdated {
    pub source: Box<str>,
    pub timestamp: i64,
}
```

**Implementation:** Additional critical fixes

---

## 6. Architecture Comparison: Config vs Schema

| Aspect                | Schema Module                              | Config Module (Target)                       |
| --------------------- | ------------------------------------------ | -------------------------------------------- |
| **Metadata storage**      | `SCHEMA_METADATA` table                      | `CONFIG_METADATA` table                        |
| **Metadata fields**       | `bank_version`, `created_at`, `modified_at`      | `created_at`, `modified_at`, `recorded_at`         |
| **Version types**         | `BankVersion` (PropertyBank)                 | `GlobalVersion`, `VaultVersion`, `Config::Version` |
| **Staleness check**       | `is_schema_stale()`, `is_bank_stale()`         | `is_global_stale()`, `is_vault_stale()`          |
| **Batch optimization**    | `are_many_stale()` (100+ schemas)            | Not needed (only 2 configs)                  |
| **Cascade staleness**     | Parent→children via inheritance            | Not needed (independent layers)              |
| **Service orchestration** | Partition stale/fresh → process stale only | Check both → use cache or rebuild            |
| **Ingestor location**     | `schema/adapter/ingestor.rs`                 | `config/adapter/ingestor.rs`                   |
| **Application service**   | `application/schema.rs`                      | `application/config.rs`                        |

---

## 7. Target Directory Structure

```
lithos-core/src/config/
├── mod.rs                    # Public exports, db_table constants (+ CONFIG_METADATA)
├── aggregate.rs              # Config, Version (merged config)
├── ports.rs                  # CommandPort, QueryPort traits (+ staleness methods)
├── events.rs                 # ConfigUpdated
├── error.rs                  # ConfigError, ConfigIngestError
│
├── raw.rs                    # RawConfig DTOs (+ vault_path, name, version)
│
├── vault.rs                  # Vault, VaultId, VaultRoot, VaultName, VaultVersion
├── global.rs                 # Global, TrustedVaults, GlobalVersion
│
├── paths.rs                  # Paths, Cache, Template, Schema, etc
├── frontmatter.rs            # Frontmatter, FrontmatterKey
├── logging.rs                # Logging, LogLevel
├── task.rs                   # Task, TaskTag, CheckboxStatus
├── value.rs                  # FieldSpec, DateSpec
│
├── adapter/
│   ├── mod.rs                # Adapter re-exports
│   ├── ingestor.rs           # ← NEW: File → Raw with metadata (was config/ingest.rs)
│   ├── stored.rs             # ← NEW: ConfigMetadata type
│   ├── command.rs            # Command adapter for redb (+ metadata writes)
│   └── query.rs              # Query adapter for redb (+ staleness checks)
│
└── command.rs                # Domain command (refactored to accept Config)
└── query.rs                  # Domain query

lithos-core/src/application/
└── config.rs                 # ← NEW: ConfigService orchestration

FILES TO DELETE:
└── config/ingest.rs          # Logic moved to adapter/ingestor.rs
```

---

## 8. Files to Create/Modify

### New Files
1. ✅ `config/adapter/stored.rs` - ConfigMetadata type (Phase 1)
2. ✅ `config/adapter/ingestor.rs` - ConfigIngestor with path resolution (Phase 2-3)
3. ✅ `application/config.rs` - ConfigService orchestration (Phase 6)

### Modified Files
1. ✅ `config/mod.rs` - Add CONFIG_METADATA table (Phase 1)
2. ✅ `config/global.rs` - Add GlobalVersion type (Phase 1)
3. ✅ `config/vault.rs` - Add VaultVersion type (Phase 1)
4. ✅ `config/raw.rs` - Add vault_path, name, version; flatten logging (Phase 7)
5. ✅ `config/ports.rs` - Add staleness methods to Query trait (Phase 4)
6. ✅ `config/adapter/query.rs` - Implement staleness methods (Phase 4)
7. ✅ `config/adapter/command.rs` - Update to record metadata (Phase 5)
8. ✅ `config/command.rs` - Refactor rebuild_merged to accept Config (Phase 6)

### Deleted Files
1. ✅ `config/ingest.rs` - Logic moved to `adapter/ingestor.rs`

---

## 9. Test Coverage Requirements

### Phase 7 Tests (RawConfig Schema)
- [ ] Round-trip: RawConfig → Domain → RawConfig preserves data
- [ ] Schema validation: All examples in `schema/examples/` parse correctly
- [ ] Flattened logging: Both `log_level = "x"` and `[logging]` work

### Hybrid Loading Tests (Phase 6)
- [ ] Fresh configs: Service returns cached Config
- [ ] Stale vault: Service rebuilds when vault file changes
- [ ] Stale global: Service rebuilds when global file changes
- [ ] Missing configs: Service handles missing files gracefully

### Staleness Tests (Phase 4)
- [ ] `is_vault_stale()`: Detects modified_at changes
- [ ] `is_vault_stale()`: Detects created_at changes (file replacement)
- [ ] `is_global_stale()`: Same checks for global config

---

## 10. Key Design Principles

1. **Version-based staleness** - Matches schema module, not content hash
2. **Three independent versions** - GlobalVersion, VaultVersion, Config::Version
3. **Version increments on change only** - Not on every save
4. **All versions start at 1** - Increment independently
5. **File replacement = version increment** - No special handling
6. **Staleness via Query** - Service orchestrates, Query checks
7. **Command is pure** - Accepts pre-built Config, no I/O
8. **Multi-format deferred** - Phase 8 after core refactoring

---

**Document Version:** 2.0
**Last Updated:** 2026-03-04
**Status:** Implementation plan finalized, ready to execute
**Total Effort:** ~6 hours (Phases 1-7)

**Key Updates in v2.0:**

- Finalized hybrid ingestion pattern based on schema module
- Added implementation status tracker with checkboxes
- Clarified version-based staleness (not content hash)
- Defined global config path resolution priority
- Added architecture comparison table (Config vs Schema)
- Updated target directory structure
- Added test coverage requirements
- Removed outdated/resolved sections
- Consolidated all relevant information for implementation
