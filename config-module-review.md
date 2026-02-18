# Lithos Config Module: Critical Issues & Architecture Review

**Date:** 2026-02-17
**Scope:** `lithos-core/src/config/` entire context module
**Status:** Pre-refactor documentation

---

## 1. Architecture & Layering Violations

### 1.1 ingest.rs Belongs in Adapter Layer, Not Domain

**Location:** `lithos-core/src/config/ingest.rs` (entire file)
**Severity:** CRITICAL
**Status:** AGREED - Must move to adapter layer

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

The `Command` CQRS handler (domain) directly calls infrastructure code, violating dependency direction. This makes the domain layer untestable without a real filesystem and couples business logic to Figment.

**Why This Is Dangerous:**

1. **Testability:** Cannot unit test config building without hitting the filesystem
2. **Swappability:** Cannot swap Figment for a different config loader without touching domain code
3. **Architecture drift:** Sets precedent for other domain modules to do I/O
4. **Clear violation:** AGENTS.md explicitly states "CQRS ports MUST NOT have file I/O methods"

**Correct Architecture:**

```
CLI/Application Layer
    ↓
FileSource trait (abstract over filesystem)
    ↓
ingest::build_merged_raw()  [moved to adapter/ or application/]
    ↓
Command::rebuild_merged(RawConfig, ...)  [pure domain, accepts already-loaded raw]
    ↓
Database persistence
```

**Required Action:**
Move `ingest.rs` to `config/adapter/ingest.rs`. Modify `Command::rebuild_merged` to accept `RawConfig` as a parameter instead of a path. Create an application service that orchestrates: FileSource → ingest → Command.

---

### 1.2 Command Orchestrates When It Should Be Pure

**Location:** `lithos-core/src/config/command.rs:14, 140`
**Severity:** CRITICAL
**Related to:** 1.1 above

**Problem:**
`Command::rebuild_merged` does three things:

1. Calls `ingest::build_merged_raw(vault_root)` - I/O operation
2. Calls `Config::build(&raw_merged, ...)` - domain logic
3. Persists to database - infrastructure operation

This is application service orchestration, not a pure domain command. The CQRS Command pattern expects commands to accept already-validated input and perform state mutations, not to load their own input from files.

**Why This Matters:**
CQRS commands should be deterministic and side-effect-free except for the intended state change. Having commands do I/O:

- Makes them non-deterministic (file might change between calls)
- Prevents testing command logic without filesystem setup
- Violates single responsibility (orchestration vs business logic)

**Required Action:**
Refactor `rebuild_merged` signature from:

```rust
pub fn rebuild_merged(&self, vault_id: VaultId, vault_root: &VaultRoot) -> Result<Version, DbError>
```

To:

```rust
pub fn rebuild_merged(&self, vault_id: VaultId, vault_root: VaultRoot, raw: &RawConfig) -> Result<Version, DbError>
```

Move the `ingest::build_merged_raw(vault_root.as_path())` call to an application service that orchestrates the full flow.

---

## 2. Config Loading Strategy Gap

### 2.1 No Hybrid Config Loading Implementation

**Location:** `config/ingest.rs`, `config/command.rs`, `config/query.rs`
**Severity:** MAJOR
**Status:** DESIGN FINALIZED - Ready for implementation

**User Requirements Confirmed:**
- Goal: Sensible defaults so configs rarely change (users should almost never need to edit configs)
- Comparison: "Is this different from file?" is sufficient (content hash + file mtime)
- Version tracking: Uses existing `config::aggregate::Version` for config history
- Approach: Full hybrid loading (Option A) matching other parts of the system

**Problem:**
Currently, the system always performs a full Figment merge and rebuild from files. There's no optimization for the case where configs haven't changed.

**Target Architecture (Final Design):**

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Hybrid Config Loading Flow                        │
└─────────────────────────────────────────────────────────────────────┘

Step 1: Ingest files (via FileSource trait)
    ├─ Global config file → (RawGlobalConfig, content_hash, mtime)
    └─ Vault config file  → (RawVaultConfig, content_hash, mtime)

Step 2: Batch query DB (single read transaction)
    ├─ Query::get_global() → Option<(Global, GlobalMetadata)>
    └─ Query::get_vault()  → Option<(Vault, VaultMetadata)>

Step 3: Compare and decide
    ├─ Global file_hash == stored_hash? → Use cached Global
    │                                      (Domain → Raw conversion for merge)
    └─ Different? → Parse fresh, validate, store

Step 4: Hybrid merge when one cached, one fresh
    ├─ Convert cached Global → RawConfig (via From<&Global> for RawConfig)
    ├─ Load fresh Vault RawConfig from file
    └─ Figment merge: Serialized::defaults(cached_raw) + Serialized::defaults(fresh_vault)

Step 5: Build final Config
    └─ Config::build(&merged_raw, vault_id, vault_root)
```

**Key Design Decisions:**

1. **Store validated Domain types, not RawConfig**
   - Database always contains valid data (Global, Vault, Config domain types)
   - Metadata stored separately: `ConfigMetadata { content_hash, file_mtime, version }`
   - **Rationale:** Zero validation risk. RawConfig can be corrupted/malformed; domain types guarantee invariants.

2. **Convert Domain → Raw when merge needed**
   ```rust
   impl From<&Global> for RawConfig { ... }
   impl From<&Vault> for RawConfig { ... }
   ```
   - Conversion is infallible (valid domain data → valid raw data)
   - Context-appropriate: Global sets `vault_path: None`, Vault sets `trusted_vaults: None`

3. **Use Figment's Serialized provider for merge**
   ```rust
   let merged: RawConfig = Figment::new()
       .merge(Serialized::defaults(cached_global_raw))  // No string round-trip!
       .merge(Serialized::defaults(fresh_vault_raw))
       .extract()?;
   ```

**Why This Approach:**

| Approach | Safety | Performance | Complexity |
|----------|--------|-------------|------------|
| Store RawConfig in DB | ❌ Risky (no validation) | ✅ Fast | Low |
| **Domain → Raw conversion** | ✅ Safe (validated data) | ✅ Fast (no I/O) | Medium |
| Figment every time | ✅ Safe | ❌ Slow (always read files) | Low |

**Trade-offs Accepted:**
- ~200 lines of `From` implementations (acceptable maintenance burden)
- Store metadata alongside domain types (minimal storage overhead)

**Files to Modify:**

- `config/raw.rs` → Update RawConfig to match schema (add vault_path, name, version, flatten logging)
- `config/global.rs` → Add ConfigMetadata, implement From<&Global> for RawConfig
- `config/vault.rs` → Add ConfigMetadata, implement From<&Vault> for RawConfig
- `config/adapter/command.rs` → Update storage to include metadata
- `config/adapter/query.rs` → Add get_global(), get_vault() with metadata
- `config/application/` → Create new application service for hybrid loading

---

### 2.2 Global and Vault Configs Purpose Clarification

**Location:** `config/global.rs`, `config/vault.rs`, `config/aggregate.rs`
**Severity:** MAJOR → RESOLVED
**Related to:** 2.1

**Clarification:**
`Global` and `Vault` are NOT dead code - they serve a critical purpose in the hybrid loading strategy:

**Three Config Representations (Each Has a Role):**

1. **`Config`** - Fully merged, always-valid aggregate (runtime truth)
   - Used by: Application code, queries, business logic
   - Storage: Yes, as the final merged result

2. **`Global`** - Global-layer settings (intermediate representation)
   - Used by: Hybrid loading as cached base layer
   - Storage: Yes, with metadata for staleness detection
   - Conversion: `From<&Global> for RawConfig` for Figment merge

3. **`Vault`** - Vault-layer overrides (intermediate representation)
   - Used by: Hybrid loading as cached overrides
   - Storage: Yes, with metadata for staleness detection
   - Conversion: `From<&Vault> for RawConfig` for Figment merge

**Separation Rationale:**
As user noted: "their clarity comes from their full qualification, e.g. vault::Paths and global::Paths"

- **Type safety:** Can't accidentally use vault-only `cache` in global context
- **Clear intent:** Full qualification shows purpose (`vault::Paths` vs `global::Paths`)
- **Different validation:** Global and Vault have different field requirements

**Implementation Required:**
- ✅ Keep separation (do not unify structs)
- ✅ Add `From<&Global> for RawConfig` conversion
- ✅ Add `From<&Vault> for RawConfig` conversion
- ✅ Add metadata fields for staleness detection
- ✅ Document the three-representation architecture

---

### 2.3 RawConfig Schema Compatibility Gap

**Location:** `config/raw.rs`
**Severity:** CRITICAL
**Status:** MUST FIX BEFORE SCHEMA CAN BE USED

**Problem:**
The current `RawConfig` struct does NOT match the schema defined in `schema/config.schema.json`. This means config files written to the schema cannot be parsed by the current code.

**Schema Structure (Expected):**
```json
{
  "vault_path": "/path/to/vault",  // ← TOP LEVEL (required for vault)
  "name": "My Vault",               // ← TOP LEVEL (optional)
  "version": "0.1.0",               // ← TOP LEVEL (optional)
  "log_level": "info",              // ← TOP LEVEL (flattened)
  "paths": { ... },                 // ← NESTED
  "frontmatter": { ... },           // ← NESTED
  "task": { ... },                  // ← NESTED
  "trusted_vaults": { ... }         // ← NESTED (global-only)
}
```

**Current RawConfig (Wrong):**
```rust
pub struct RawConfig {
    // MISSING: vault_path, name, version
    pub logging: Option<RawLogging>,  // ← Nested [logging] section
    pub paths: RawPathsConfig,
    pub frontmatter: Option<RawFrontmatter>,
    pub task: Option<RawTaskConfig>,
    pub trusted_vaults: Option<RawTrustedVaults>,
}
```

**Required Changes to `raw.rs`:**

```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

**Impact of Missing Fields:**

| Field | Impact | Required For |
|-------|--------|--------------|
| `vault_path` | CRITICAL | Vault configs won't work without it |
| `name` | Medium | Nice-to-have display name |
| `version` | Low | Future migration tracking |
| `log_level` flattened | High | Schema shows top-level, code expects nested |

**Testing Required:**
- Round-trip tests: RawConfig → Domain → RawConfig produces equivalent result
- Schema validation: All examples in `schema/examples/` parse correctly
- Hybrid loading: Domain → Raw conversion works for merge scenarios

---

## 3. Data Integrity & Edge Cases

### 3.1 Version Overflow Silently Corrupts Data

**Location:** `config/adapter/command.rs:164-165`, `config/command.rs:342-343`
**Severity:** MAJOR
**Status:** AGREED - Should fix

**Problem:**

```rust
let next_version = v.next().unwrap_or_else(|_| Version::initial());
```

When `Version::next()` overflows (reaches `u64::MAX`), it falls back to `Version::initial()` which is `Version(1)`. But version `1` already exists from the first config! This would:

1. Create a new config with version 1 (duplicate key)
2. Overwrite or conflict with the original version 1 config
3. Make `activate_version(Exact(Version(1)))` ambiguous

**Why This Is Dangerous:**
Even though 2^64 rebuilds is practically impossible, the fallback behavior is dangerous because:

1. It silently corrupts data instead of failing
2. It violates the invariant that versions are monotonically increasing
3. It could mask other bugs that trigger overflow (e.g., a bug that increments version multiple times per rebuild)

**User Assessment:** "far off edge case, but we can fix the overflow"

**Required Action:**
Change from silent fallback to error propagation:

```rust
let next_version = v.next()
    .map_err(|_| DbError::Serialization("config version overflow - vault has exceeded maximum rebuilds".into()))?;
```

This will never trigger in practice (would require rebuilding config 18 quintillion times), but if it ever does, we'll know there's a bug rather than silently corrupting data.

---

### 3.2 Trusted Vaults Configured but Never Used

**Location:** `config/raw.rs:36`, `config/aggregate.rs:100-141`
**Severity:** MAJOR

**Problem:**
`RawConfig.trusted_vaults` is deserialized from config files but completely ignored in `Config::build()`. Users can configure:

```toml
trusted_vaults = ["/vaults/work", "/vaults/personal"]
```

And it will parse successfully but have **zero effect**.

**Why This Is Unacceptable:**

1. **Silent failure:** User thinks they've configured cross-vault trust, but it's ignored
2. **Security implications:** If cross-vault operations are implemented later, old configs with `trusted_vaults` might unexpectedly become active
3. **Broken promise:** The schema includes the field, creating expectation it works

**User Assessment:** "trusted_vaults should be added, but it is a key that is really only needed when working with lithos outside vault"

**Required Action:**

1. Add `trusted_vaults: Option<TrustedVaults>` field to `Config` struct
2. Populate it from `raw.trusted_vaults` in `Config::build()`
3. Add validation that paths are absolute
4. Document that this is primarily for global config (cross-vault operations)

---

### 3.3 VaultRoot Lacks Critical Validation

**Location:** `config/vault.rs:360-368`
**Severity:** MAJOR

**Problem:**
`VaultRoot::try_new()` only checks that path is non-empty:

```rust
if path.as_os_str().is_empty() {
    return Err(ConfigError::ValidationFailed { ... });
}
```

It does NOT validate:

- Path is absolute (doc comment says "Should ideally be an absolute path")
- Path is a directory (not a file)
- Path exists on filesystem

**User Assessment:** "VaultRoot is supposed to embed AbsolutePath from paths.rs"

**Current Implementation:**
Actually, looking at the code, `VaultRoot` wraps `PathBuf` directly:

```rust
pub struct VaultRoot(PathBuf);
```

But `AbsolutePath` exists in `paths.rs` as a validated wrapper. The intent (per user) is that `VaultRoot` should use `AbsolutePath`, not raw `PathBuf`.

**Why This Matters:**

1. **Relative paths accepted:** `VaultRoot::try_new("./vault")` succeeds, but will break when CWD changes
2. **Type system gap:** We have `AbsolutePath` type but don't use it for vault roots
3. **Silent failures:** Path validation deferred to runtime, causing mysterious errors later

**Required Action:**

1. Change `VaultRoot` to wrap `AbsolutePath` instead of `PathBuf`:

```rust
pub struct VaultRoot(AbsolutePath);
```

2. Update `try_new` to validate absoluteness via `AbsolutePath::try_new`
3. Consider adding directory existence check (may be I/O, belongs in adapter)

---

### 3.4 Only TOML Supported Despite Documentation Claiming JSON/YAML

**Location:** `config/ingest.rs:61,67`, `fs/parsers.rs`
**Severity:** MAJOR

**Problem:**
`ingest.rs` hardcodes `Toml::file(path)`:

```rust
figment = figment.merge(Toml::file(path));
```

Meanwhile, `fs/parsers.rs` has a sophisticated `Dispatcher` that supports TOML, JSON, and YAML with content-based auto-detection. But it's never used for config files.

The doc comment in `raw.rs` says: "deserialization from TOML/YAML/JSON files" - this is misleading.

**Why This Is Inconsistent:**

1. **Infrastructure exists but is unused:** Wasted code in `parsers.rs`
2. **User confusion:** Docs say multiple formats supported, but only TOML works
3. **Missed opportunity:** Users who prefer JSON/YAML for configs are forced to use TOML

**Required Action:**
Option A: Extend `build_merged_raw_impl` to probe for `.toml`, `.json`, `.yaml` variants using `Dispatcher`
Option B: Remove JSON/YAML support from `parsers.rs` and update documentation to clarify TOML-only

Recommendation: Option A (implement support), since infrastructure exists.

---

### 3.5 global_config_path_from_env Is a Misleading Stub

**Location:** `config/ingest.rs:74-81`
**Severity:** MINOR
**Status:** ACCEPTABLE per user instruction

**Problem:**

```rust
fn global_config_path_from_env() -> Option<PathBuf> {
    // Placeholder for environment variable support.
    // TODO: Implement via `figment::providers::Env` with LITHOS_ prefix.
    // Reserved for future use.
    // Example implementation:
    // std::env::var_os("LITHOS_GLOBAL_CONFIG").map(PathBuf::from)
    None
}
```

Function name implies it reads from environment. It always returns `None`.

**User Assessment:** "It should only be a stub for now because we are not using environment variables yet"

**Why This Is Currently Acceptable:**

- Not blocking any current functionality
- Clear TODO comment explaining future intent
- Simple to implement when needed

**Why It Could Be Better:**
The function name over-promises. Consider renaming to `unimplemented_global_config_path()` or adding `#[deprecated(note = "stub implementation")]` to signal it's not ready.

**Required Action:** NONE - acceptable as stub per user. Consider adding doc comment clarifying it's intentionally non-functional.

---

## 4. Code Quality Issues

### 4.1 String Allocation Anti-Patterns (AGENTS.md Violations)

**Locations:** Multiple files
**Severity:** MINOR
**Status:** AGREED - Must fix

**Problem:**
Multiple instances of patterns explicitly banned in AGENTS.md:

**Pattern 1: `.to_owned().into()` on string literals**

```rust
// WRONG - found in paths.rs:641, task.rs:378,385,494,507, value.rs:432
message: "file name must not contain path separators"
    .to_owned()
    .into(),

// CORRECT
message: "file name must not contain path separators".into(),
```

**Pattern 2: `.to_owned().into_boxed_str()` on &str**

```rust
// WRONG - found in task.rs:390, 511, value.rs:437
Ok(Self(text.to_owned().into_boxed_str()))

// CORRECT
Ok(Self(Box::from(text)))
// or
Ok(Self(text.into()))
```

**Why This Matters:**

1. **Performance:** Unnecessary allocations (String heap overhead + copy)
2. **Code clarity:** Extra noise for no benefit
3. **Consistency:** AGENTS.md explicitly calls these out as "Must Avoid"
4. **Precedent:** Fixing these establishes pattern for rest of codebase

**Required Action:**
Systematic fix across:

- `config/paths.rs`: Error messages in validation
- `config/task.rs`: `StatusName::try_new`, `TaskTag::try_new`
- `config/value.rs`: `FieldName::try_new`
- `config/frontmatter.rs`: Consider `Box<str>` for `FrontmatterKey`
- `config/events.rs`: `ConfigUpdated.source` should be `Box<str>`

---

### 4.2 Dispatcher Silently Swallows Parse Errors

**Location:** `fs/parsers.rs:123-155`
**Severity:** MINOR
**Status:** AGREED - Should fix

**Problem:**

```rust
fn parse(&self, content: impl AsRef<[u8]>) -> Result<Value, ParseError> {
    if let Some(ext) = self.extension {
        match ext {
            Some(Format::Toml) => Toml.parse(&content),
            // ...
        }
        // If parse fails, falls through to content detection!
    }
    // Content detection tries other formats...
}
```

When a `.toml` file contains invalid TOML:

1. `Toml.parse()` returns `Err(ParseError::Toml { ... })`
2. Dispatcher ignores this specific error
3. Falls through to content detection
4. Content detection fails → returns `ParseError::UnsupportedFormat`

**Why This Is Bad:**

- User with malformed `lithos.toml` gets "unsupported format" error
- Actual error (syntax error at line 5) is lost
- Makes debugging extremely difficult

**Example:**

```toml
# lithos.toml with syntax error
log_level = debug   # missing quotes around 'debug'
```

User sees: `"Unsupported file format"`
Should see: `"TOML parse error: invalid string at line 1, column 13"`

**Required Action:**
When extension-based parser fails, return the format-specific error rather than trying fallback. Only fall back to content detection when no extension matches.

---

### 4.3 glob::Pattern Recompiled Per File (Performance)

**Location:** `fs/source.rs:163-196`
**Severity:** MINOR
**Status:** Pushed off per user instruction, but needs explanation

**Problem:**

```rust
fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
    // ...
    let entries: Vec<_> = walker
        .filter_map(|entry| {
            let pattern = glob::Pattern::new(pattern_str).ok()?;  // ← INSIDE LOOP!
            // ...
        })
        .collect();
}
```

`glob::Pattern::new(pattern_str)` parses the glob string every iteration. For a vault with 10,000 files, the pattern is compiled 10,000 times.

**Why This Is Inefficient:**

- `glob::Pattern::new()` does string parsing and regex-like compilation
- Result is identical for every file
- Should be done once outside the loop

**The Fix:**

```rust
fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
    let pattern = glob::Pattern::new(pattern)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    let entries: Vec<_> = walker
        .filter_map(|entry| {
            // Use pre-compiled `pattern` here
            let path = entry.ok()?.path();
            if pattern.matches(path.to_str()?) {
                Some(path)
            } else {
                None
            }
        })
        .collect();
}
```

**User Assessment:** "you need to explain this issue... For the sake of moving forward with development, I think we can push off this issue"

**Explanation Provided Above** - This is a classic "algorithmic inefficiency" where O(N) work is done N times, making it O(N²). For now, acceptable to defer since typical vaults have <1000 files.

---

### 4.6 Additional Edge Cases (Not Previously Covered)

#### 4.6.1 Vault Root Directory Not Validated

**Location:** `config/ingest.rs:64-68`, `config/command.rs:134-158`
**Severity:** MINOR

**Problem:**
If `vault_root` doesn't exist as a directory, `vault_config_path.exists()` returns `false` and defaults are used silently. No warning that the vault root itself doesn't exist. Same for permission-denied `.lithos/lithos.toml` - Figment silently ignores it.

**Why This Is Problematic:**

- User thinks config is loaded but actually using defaults
- Permission errors masked as "file not found"
- Silent failures make debugging difficult

**Required Action:**
After `vault_config_path.exists()` check, distinguish "file not found" from "permission denied" by attempting a metadata read. Log appropriate warnings.

---

#### 4.6.2 Config Migration Strategy Missing

**Location:** `config/vault.rs` (entire `AppVersion` type), `config/aggregate.rs`
**Severity:** MAJOR

**Problem:**
`AppVersion` tracks Lithos version that created config, but:

1. No code reads stored `AppVersion` and applies migrations
2. On upgrade, old `rkyv` bytes with different field layout may fail validation or produce corrupt data
3. `rkyv(bytecheck(bounds()))` provides some safety, but schema changes (adding required fields) silently use zeroed memory

**Why This Matters:**

- Breaking changes to config format will corrupt user data
- No upgrade path defined
- Version field exists but serves no purpose

**Required Action:**
At minimum, document migration strategy. Consider versioned wrapper: `StoredConfig { schema_version: u32, data: Config }` or use JSON for migration flexibility, reserving `rkyv` for hot-path reads only.

---

#### 4.6.3 Root Path "/" Produces "unnamed" Vault

**Location:** `config/vault.rs:524-529`
**Severity:** MINOR

**Problem:**
`VaultName::from_root("/")` calls `file_name()` which returns `None` for root paths. Fallback is `"unnamed".to_owned()`, meaning vault at root `/` is named "unnamed". Could cause collisions if multiple vaults root at `/`.

**Required Action:**
Document this edge case or add validation that vault root must have a filename component (i.e., cannot be filesystem root).

---

#### 4.6.4 Circular Trusted Vault References Not Detected

**Location:** `config/global.rs`, `config/raw.rs:161-170`
**Severity:** MINOR

**Problem:**
If vault A lists vault B as trusted, and vault B lists vault A as trusted, no cycle detection exists. Current codebase doesn't traverse `trusted_vaults` transitively, but future features (e.g., "load schemas from trusted vaults") would loop infinitely.

**Required Action:**
Document that `trusted_vaults` is flat/non-transitive and cycles are not traversed. Add cycle detection if transitive traversal is implemented.

---

#### 4.6.5 Chrono Format Validation Is Broken

**Location:** `config/value.rs:596-615`
**Severity:** MINOR

**Problem:**
`validate_chrono_format` formats current UTC time and checks if result is non-empty. But `chrono::format()` never returns empty for valid format strings - it renders unrecognized specifiers literally. Example: `format = "%Q"` (invalid) produces `"Q"` (non-empty), passing validation.

**Why This Is Wrong:**
Invalid format strings pass validation, causing runtime errors when actually parsing dates.

**Required Action:**
Use `chrono::format::StrftimeItems` to parse format string and validate each specifier, or attempt round-trip parse after formatting.

---

#### 4.6.6 ActivationTarget::Previous Has No Upper Bound Validation

**Location:** `config/ports.rs:19-24`, `config/adapter/command.rs:112-144`
**Severity:** MINOR

**Problem:**
`steps: u32` can be up to `4,294,967,295`. While `saturating_sub` prevents underflow, a `steps` value larger than current version saturates to `0`, triggering "activation underflow" error. No validation that `steps > 0` (activating with `steps = 0` re-activates current version - is this intentional?).

**Required Action:**
Add `if steps == 0 { return Err(...) }` validation or document zero-step behavior as no-op re-activation.

---

## 5. Code Quality Issues (Rust Best Practices)

### 5.1 String Allocation Anti-Patterns (AGENTS.md Violations)

See Section 4.5 for main coverage. Additional instances:

#### 5.1.1 Additional String Anti-Pattern Locations

**Locations:** `config/paths.rs:641-643`, `config/task.rs:378,385,494,507`, `config/value.rs:432`
**Severity:** MINOR

**Pattern:** `"literal".to_owned().into()` instead of `"literal".into()`

**Full List of Violations:**

- `paths.rs:641`: "file name must not contain path separators"
- `task.rs:378`: "status name must be 1-32 characters"
- `task.rs:385`: "status name must be ASCII alphanumeric or '\_'"
- `task.rs:494`: "task tag must start with '#' and be non-empty"
- `task.rs:507`: "task tag must be ASCII alphanumeric, '\_' or '-'"
- `value.rs:432`: "field name must be ASCII alphanumeric, '\_' or '-'"

**Required Action:**
Replace all `.to_owned().into()` with just `.into()`.

---

### 5.2 Type Safety and Encapsulation

#### 5.2.1 FrontmatterKey Uses String Instead of Box<str>

**Location:** `config/frontmatter.rs:152`
**Severity:** MINOR

**Problem:**
`FrontmatterKey(String)` stores validated, immutable string as `String` (24 bytes header on 64-bit). Should be `Box<str>` (16 bytes) per AGENTS.md.

**Required Action:**

```rust
pub struct FrontmatterKey(Box<str>);
```

---

#### 5.2.2 ConfigUpdated.source Uses String Instead of Box<str>

**Location:** `config/events.rs:60`
**Severity:** NITPICK

**Problem:**
`source: String` set at construction via `source: source.into()` (where `source: &str`) and never mutated. Should be `Box<str>`.

---

#### 5.2.3 Path Conversions Are Lossy for Non-UTF-8 Paths

**Location:** `config/paths.rs:741-746, 767-770`
**Severity:** MINOR

**Problem:**
`From<RelativePath> for String` and `From<AbsolutePath> for String` use `to_string_lossy().into_owned()`. On non-UTF-8 paths (Windows paths with invalid Unicode, or Linux paths with arbitrary bytes), this silently replaces non-Unicode bytes with `U+FFFD`.

**Why This Matters:**

- Serde round-tripping corrupts non-UTF-8 paths
- Linux file paths can contain arbitrary bytes
- Silent data loss

**Required Action:**
Either: (a) add UTF-8 validation in `try_new` and document UTF-8 requirement; or (b) don't use `String` as serde representation - use bytes or `OsString`.

---

#### 5.2.4 vault_id.to_string() Allocates on Every DB Operation

**Location:** `config/adapter/command.rs:75,90,108,116,138`, `config/adapter/query.rs:46,65`
**Severity:** MINOR

**Problem:**
AGENTS.md explicitly calls out `UUID to_string() in hot paths` as anti-pattern. `VaultId` is `uuid::Uuid` wrapper. Every DB operation calls `vault_id.to_string()` allocating 36 bytes. In `activate_version(Previous { steps })`, called twice in same transaction (`get_owned` + `put`).

**Required Action:**
Compute key string once at method start and reuse:

```rust
let vault_key = vault_id.to_string();
self.db.put(CONFIG, &vault_key, config)
```

---

### 5.3 API Design Issues

#### 5.3.1 TryFrom and from_raw Are Redundant

**Location:** `config/task.rs:533-540`
**Severity:** NITPICK

**Problem:**

```rust
impl TryFrom<RawTaskConfig> for Task {
    type Error = ConfigError;
    fn try_from(raw: RawTaskConfig) -> Result<Self, Self::Error> {
        Self::from_raw(raw)  // Just delegates!
    }
}
```

`TryFrom` impl just calls `Task::from_raw(raw)`. Both are public, creating confusion about which is canonical. `#[serde(try_from = "RawTaskConfig")]` uses `TryFrom`, so `from_raw` is redundant.

**Required Action:**
Make `Task::from_raw` non-public (`pub(crate)`) or remove and derive `TryFrom` automatically.

---

#### 5.3.2 #[non_exhaustive] on Version Newtype Is Redundant

**Location:** `config/aggregate.rs:189`
**Severity:** MINOR

**Problem:**
`Version(u64)` has `#[non_exhaustive]` but field `0: u64` is already private. External code cannot construct `Version(42)` regardless. `#[non_exhaustive]` adds nothing but confusion.

**Required Action:**
Remove `#[non_exhaustive]` from `Version`. Keep for enums with variant-addition semantics, not newtypes.

---

#### 5.3.3 LogLevel::as_str Takes &self But Could Take self (Copy Type)

**Location:** `config/logging.rs:118-127`
**Severity:** NITPICK

**Problem:**

```rust
pub fn as_str(&self) -> &'static str {
    match *self { ... }  // Immediate deref
}
```

For `Copy` types, taking by value is clearer:

```rust
pub const fn as_str(self) -> &'static str { ... }
```

---

#### 5.3.4 Template::templates_dir and Cache::cache_dir Are pub

**Location:** `config/paths.rs:257, 317`
**Severity:** MINOR

**Problem:**
`Template` has `pub templates_dir: RelativePath` while `Schema` uses private field with accessor. Breaks encapsulation. Both types have accessors (`templates_dir()`, `cache_dir()`), making `pub` fields redundant.

---

#### 5.3.5 Paths Struct Has All pub Fields

**Location:** `config/paths.rs:55-64`
**Severity:** MINOR

**Problem:**
`config::paths::Paths` has all fields `pub` (`cache`, `template`, `schema`, `property_bank`), while individual domain types use private fields with accessors. Callers can freely mutate `paths.cache = Cache::default()` after construction, bypassing invariant enforcement.

**Required Action:**
Make `Paths` fields private and expose accessors, or document as pure data holder with no invariants.

---

#### 5.3.6 ConfigError::InvalidEnumValue.allowed Uses Vec<String>

**Location:** `config/error.rs:44`
**Severity:** NITPICK

**Problem:**
`allowed: Vec<String>` when strings are built from static array:

```rust
["error", "warn", "info", "debug", "trace"]
    .iter()
    .map(ToString::to_string)
    .collect()
```

Should be `Vec<Box<str>>` or `&'static [&'static str]`.

---

#### 5.3.7 FieldSpec::PartialEq Manual Impl Needs Documentation

**Location:** `config/value.rs:585-590`
**Severity:** NITPICK

**Problem:**
Manual `impl Eq for FieldSpec` excludes `Arc<Regex>` field. Comment says "Default implementation is fine" but this is incorrect - `Arc<Regex>` doesn't implement `Eq`. Intentional exclusion needs explicit `// SAFETY` or `// Note` comment.

---

## 6. Test Coverage Gaps

### 6.1 Permission-Denied Config File

**Location:** `config/ingest.rs` (tests)
**Severity:** MINOR

**Gap:**
No test for when `.lithos/lithos.toml` exists but is unreadable (permissions error). Figment silently skips it because `path.exists()` returns `true` but read fails. Currently produces incorrect default config silently.

**Required Action:**
Add test with read-protected config file, verify appropriate error is returned (not silent fallback).

---

### 6.2 Config::build with Invalid RawLogging Values

**Location:** `config/aggregate.rs` (tests)
**Severity:** MINOR

**Gap:**
Test `merged_config_with_sample_overrides` uses valid `log_level = Some("debug")`. No test for `RawLogging { log_level: Some("invalid_level") }` to verify error propagation.

**Required Action:**
Add test with invalid log level, verify `ConfigError::InvalidEnumValue` is returned.

---

### 6.3 VaultRoot with Relative Paths

**Location:** `config/vault.rs` (tests)
**Severity:** MINOR

**Gap:**
`vault_root_rejects_empty` tests empty case. No test for relative paths like `"./vault"` or `"vault"` which currently pass validation despite doc comment saying they "should ideally be absolute".

**Required Action:**
Add test documenting current behavior (relative paths accepted) or intended behavior (relative paths rejected).

---

### 6.4 with_archived Test Doesn't Read Archived Data

**Location:** `config/query.rs:413-442`
**Severity:** MINOR

**Gap:**
`with_archived_returns_data_via_closure` test uses closure returning hardcoded `true` without accessing archived config. Comment says "Config has private fields, so we test the pattern works by checking the closure is called". But `ArchivedConfig::paths()` is public via `ArchivedPaths` - test should access at least one field.

**Required Action:**
Update test to access archived field (e.g., `archived.paths().cache().to_str()`).

---

### 6.5 Dispatcher Error Swallowing

**Location:** `config/fs/parsers.rs` (tests)
**Severity:** MINOR

**Gap:**
`should_provide_toml_error_context` calls `Toml::parse` directly, not `Dispatcher::parse`. No test verifying that invalid TOML returns `ParseError::Toml` (not `ParseError::UnsupportedFormat`).

**Required Action:**
Add test calling `Dispatcher::parse` with invalid TOML content, verify specific TOML error is returned.

---

### 6.6 CheckboxStatus::from_raw Duplicate Name Check Is Unreachable

**Location:** `config/task.rs:279-283`
**Severity:** MINOR

**Gap:**
`by_name.contains_key(&status_name)` check can never trigger because `HashMap` keys are unique - duplicate names deduplicated before reaching `from_raw`. Check is dead code, no test demonstrates it triggers.

**Required Action:**
Either remove dead check, or change raw input from `HashMap<String, char>` to `Vec<(String, char)>` to preserve duplicates.

---

### 6.7 Full CQRS Round-Trip Integration Test

**Location:** `config/command.rs`, `config/query.rs` (tests)
**Severity:** MINOR

**Gap:**
`rebuild_merged_reads_vault_config_file` verifies config file is read, but reads directly from DB via `db.get_owned`. No test uses both `Command::rebuild_merged` AND `Query::get` together to verify full CQRS cycle.

**Required Action:**
Add integration test: `rebuild_merged` → `Query::get` → verify retrieved config matches original.

---

### 6.8 Domain → Raw → Domain Round-Trip Tests

**Location:** `config/global.rs`, `config/vault.rs` (tests)
**Severity:** MAJOR
**Related to:** 2.1, 2.3

**Gap:**
No tests verify that `From<&Global> for RawConfig` and `From<&Vault> for RawConfig` produce valid RawConfig that can be parsed back into equivalent domain types.

**Why This Is Critical:**
The hybrid loading strategy depends on this round-trip working correctly:
1. Global stored in DB (domain type)
2. Convert to RawConfig for Figment merge
3. Merge with fresh Vault RawConfig
4. Build new Config

If the conversion loses data or produces invalid RawConfig, the merge will fail or produce wrong results.

**Test Scenarios:**

```rust
#[test]
fn global_round_trip_preserves_data() {
    let original = Global::new(...);  // Create with all fields populated
    let raw = RawConfig::from(&original);
    let reconstructed = Global::try_from(raw).unwrap();

    assert_eq!(original.logging(), reconstructed.logging());
    assert_eq!(original.paths().templates_dir(), reconstructed.paths().templates_dir());
    // ... all fields
}

#[test]
fn vault_round_trip_preserves_data() {
    let original = Vault::new(...);
    let raw = RawConfig::from(&original);
    let reconstructed = Vault::try_from(raw).unwrap();

    assert_eq!(original.name(), reconstructed.name());
    assert_eq!(original.root(), reconstructed.root());
    // ... all fields
}
```

**Note:** While these are similar to e2e tests, they're actually **unit tests for the conversion logic** that the hybrid loading depends on. They should be fast, deterministic, and not require filesystem or database access.

**Required Action:**
Add comprehensive round-trip tests for both Global and Vault conversions.

---

## 7. Performance Issues

### 7.1 glob::Pattern Recompiled Per File

**Covered in Section 4.5** - User deferred this issue.

---

### 7.2 merged_version_key Allocates on Hot Path

**Location:** `config/adapter/mod.rs:15-20`
**Severity:** MINOR

**Problem:**
TODO comment already acknowledges: "TODO: Optimize with stack buffer to avoid format! allocation (57 bytes max)". `format!("{}:{}", vault_id, version.value())` allocates on every `get_merged_owned` and `with_archived` call.

**Required Action:**
Use `arrayvec::ArrayString<57>` or stack-allocated buffer:

```rust
let mut buf = [0u8; 57];
let key = format_to_buf(&mut buf, vault_id, version); // returns &str
```

---

### 7.3 validate_chrono_format Calls Utc::now() at Validation Time

**Location:** `config/value.rs:607`
**Severity:** NITPICK

**Problem:**
`validate_chrono_format` calls `chrono::Utc::now().naive_utc()` - a system clock call on every config validation. Unnecessary overhead.

**Better Approach:**
Use fixed date (e.g., epoch zero) instead of `now()`:

```rust
let _ = NaiveDateTime::from_timestamp(0, 0).format(format_str);
```

This avoids syscall overhead and makes validation deterministic.

---

### 7.4 FieldSpec::String Stores Pattern String AND Compiled Regex

**Location:** `config/value.rs:74`
**Severity:** NITPICK

**Problem:**

```rust
String {
    pattern: Option<String>,
    compiled: Option<Arc<Regex>>,
}
```

Stores both pattern string and compiled regex, doubling memory per field. For read-mostly configs this is acceptable but wasteful.

**Note:**
Low priority since configs built once and cached.

---

## 8. Code Redundancy Issues

### 8.1 RawLogging Is a Thin Wrapper Adding No Value

**Location:** `config/logging.rs:133-139`
**Severity:** MINOR

**Problem:**
`RawLogging { log_level: Option<String> }` has exactly one field. `TryFrom<RawLogging> for Logging` is 5 lines of trivial mapping. Could be `RawConfig.logging: Option<String>` directly.

**Counter-Argument:**
Forward compatibility for adding more logging fields later.

---

### 8.2 db_table Constants Couple Tests to Storage Schema

**Location:** `config/mod.rs:125-138`
**Severity:** MINOR

**Problem:**
`db_table` module is `pub(crate)` and used directly in test code. Tests bypass adapter layer, interacting directly with database table schema. If table layout changes, every test file importing `db_table` must change.

**Required Action:**
Consider if tests should use adapter methods instead of direct table access, or document this as acceptable test coupling.

---

### 8.3 VaultRoot::as_key() Duplicates From<VaultRoot> for String Logic

**Location:** `config/vault.rs:378-382, 396`
**Severity:** NITPICK

**Problem:**
`as_key()` returns `self.as_path().to_string_lossy().into_owned()`. `From<VaultRoot> for String` does `root.0.to_string_lossy().into_owned()` - identical operation.

**Required Action:**
Have `as_key()` call `String::from(self.clone())` using trait impl, or remove `as_key()` in favor of `Into<String>`.

---

### 8.4 ArchivedConfig Exposes Too Few Accessors

**Location:** `config/aggregate.rs:164-171`, `config/paths.rs:93-100`
**Severity:** MINOR

**Problem:**
`ArchivedConfig` has only one accessor: `paths()`. `ArchivedPaths` has only one accessor: `cache()`. Zero-copy API advertised as hot-path optimization, but callers needing `logging`, `frontmatter`, `task`, `schema`, `template` must fall back to `get_merged_owned` (full deserialization).

**Required Action:**
Expose archived accessors for all likely hot-path fields, or document which fields are available zero-copy vs requiring deserialization.

---

## 9. Design Decisions Acknowledged

**User Instruction:** "No, do not unify the Paths structs and their clarity comes from their full qualification, e.g. vault::Paths and global::Paths"

**Status:** RESPECTED - Do not unify

The current design with `vault::Paths` and `global::Paths` is intentional. Benefits:

1. **Type safety:** Can't accidentally use vault-only `cache` in global context
2. **Clarity:** Full qualification shows intent (`vault::Paths` vs `global::Paths`)
3. **Documentation:** Different defaults/behaviors can be documented per-type

No changes required here.

---

## 10. Summary of Required Actions

### Immediate (This Week)

| Priority | Issue | Action | Files |
|----------|-------|--------|-------|
| P0 | RawConfig schema compatibility | Add vault_path, name, version, flatten logging | `config/raw.rs` |
| P0 | ingest.rs location | Move to `adapter/` layer, update imports | `config/ingest.rs` → `adapter/` |
| P0 | Command coupling | Refactor `rebuild_merged` to accept `RawConfig` | `config/command.rs` |
| P0 | Version overflow | Replace `unwrap_or_else` with error propagation | `adapter/command.rs`, `command.rs` |
| P1 | Trusted vaults | Add to `Config` and populate from `RawConfig` | `config/aggregate.rs`, `raw.rs` |
| P1 | VaultRoot validation | Use `AbsolutePath` wrapper | `config/vault.rs` |
| P1 | String anti-patterns | Systematic fix of `.to_owned().into()` | `paths.rs`, `task.rs`, `value.rs` |
| P1 | Dispatcher errors | Return format-specific errors | `fs/parsers.rs` |
| P1 | Domain→Raw conversions | Implement From<&Global> and From<&Vault> | `config/global.rs`, `config/vault.rs` |
| P2 | FrontmatterKey Box<str> | Change from String to Box<str> | `config/frontmatter.rs` |
| P2 | Vault ID string alloc | Cache vault_id.to_string() in methods | `adapter/command.rs`, `adapter/query.rs` |

### Short Term (Next 2 Weeks)

| Priority | Issue                     | Action                                           | Files                             |
| -------- | ------------------------- | ------------------------------------------------ | --------------------------------- |
| P2       | Hybrid config loading     | Design comparison strategy, implement flow       | `adapter/ingest.rs`, `command.rs` |
| P2       | Global/Vault dead code    | Either implement hybrid loading or remove        | `global.rs`, `vault.rs`           |
| P2       | JSON/YAML support         | Use Dispatcher for config files                  | `ingest.rs`                       |
| P2       | Config migration          | Document strategy or implement versioned wrapper | `aggregate.rs`, `vault.rs`        |
| P2       | Paths pub fields          | Make private with accessors                      | `config/paths.rs`                 |
| P3       | merged_version_key        | Use stack buffer instead of format!              | `adapter/mod.rs`                  |
| P3       | #[non_exhaustive] removal | Remove from Version newtype                      | `aggregate.rs`                    |
| P3       | TryFrom/from_raw dedup    | Make from_raw non-public                         | `task.rs`                         |

### Deferred (Future)

| Priority | Issue                       | Rationale                                           |
| -------- | --------------------------- | --------------------------------------------------- |
| P3       | glob::Pattern optimization  | User pushed off; low impact for typical vault sizes |
| P3       | global_config_path_from_env | Acceptable stub per user                            |
| P3       | Config migration full impl  | Version field exists but no current need            |
| P3       | ArchivedConfig accessors    | Add accessors for hot-path fields                   |
| P3       | Non-UTF-8 path handling     | Edge case; UTF-8 is reasonable requirement          |
| P3       | Test coverage gaps          | Important but not blocking                          |
| P4       | Code style nits             | LogLevel::as_str, FieldSpec Eq docs, etc            |

---

## 11. Architecture Target State

```
lithos-core/src/config/
├── mod.rs                    # Public exports, db_table constants
├── aggregate.rs              # Config, Version, Metadata (domain)
├── ports.rs                  # CommandPort, QueryPort traits (domain)
├── events.rs                 # ConfigUpdated (domain)
├── error.rs                  # ConfigError, ConfigIngestError (domain)
│
├── raw.rs                    # RawConfig DTOs (domain boundary)
├── bounds.rs                 # Generic bounds validation (domain)
│
├── vault.rs                  # Vault, VaultId, VaultRoot, VaultName (domain)
├── global.rs                 # Global, TrustedVaults (domain)
│
├── paths.rs                  # Paths, Cache, Template, Schema, etc (domain)
├── frontmatter.rs            # Frontmatter, FrontmatterKey (domain)
├── logging.rs                # Logging, LogLevel (domain)
├── task.rs                   # Task, TaskTag, CheckboxStatus (domain)
├── value.rs                  # FieldSpec, DateSpec (domain)
│
├── adapter/
│   ├── mod.rs                # Adapter re-exports
│   ├── ingest.rs             # ← MOVED HERE (was config/ingest.rs)
│   ├── command.rs            # Command adapter for redb
│   └── query.rs              # Query adapter for redb
│
└── application/              # ← NEW: Application services
    └── mod.rs                # Orchestration logic
    └── config_service.rs     # Hybrid loading, file watching, etc

lithos-core/src/fs/
├── source.rs                 # FileSource trait
├── parsers.rs                # Dispatcher, format parsers
└── error.rs                  # FsError, ParseError
```

**Key Principles:**

1. Domain layer (`config/`) is pure - no I/O, no external deps
2. Adapter layer (`config/adapter/`) handles infrastructure concerns
3. Application layer (`config/application/` or `application/`) orchestrates
4. Clear dependency direction: fs → config::adapter → config → application → CLI

---

**Document Version:** 1.2
**Last Updated:** 2026-02-17
**Next Review:** After immediate actions completed
**Total Issues Documented:** 60 (8 Critical/Major Architecture, 13 Edge Cases, 18 Code Quality, 8 Test Gaps, 4 Performance, 9 Redundancy)

**Key Updates in v1.2:**
- Added RawConfig schema compatibility analysis (Section 2.3)
- Clarified Global/Vault purpose in hybrid loading (Section 2.2)
- Added Domain→Raw round-trip test requirement (Section 6.8)
- Finalized hybrid loading design with Figment Serialized provider approach
