# Findings & Decisions

## Requirements
From user request:
- Update `SchemaConfigSpec` in `lithos-core/src/config/paths.rs`
- Change from `RelativePath` to `Path` or `PathBuf`
- Join schema directory to VaultRoot from vault metadata
- Join property bank file on top of that
- Result: `SchemaConfigSpec` passed to builder.rs and discovery.rs doesn't need separate vault root

## Research Findings

### Current Architecture
- `SchemaConfigSpec` is a minimal, filesystem-focused view for discovery engine
- Currently stores two `RelativePath` fields: `directory` and `property_bank`
- `property_bank` is the FULL joined path (e.g., "schemas/property_bank.json"), not just filename
- Created by `Config::to_schema_spec()` in aggregate.rs:226
- Consumed by `DiscoveryEngine::run()` in discovery.rs:197 and `Builder::load_all()` in builder.rs:61

### Current Flow
1. `Config::to_schema_spec()` calls `self.paths.property_bank_path()` which joins schemas dir + filename
2. Converts PathBuf to RelativePath
3. Creates SchemaConfigSpec with two RelativePath fields
4. DiscoveryEngine::run() receives SchemaConfigSpec + vault_root separately
5. Discovery internally uses vault_root.as_path() to make paths absolute

### VaultRoot Structure
- Defined in config/vault.rs:88
- Wraps PathBuf: `pub struct VaultRoot(#[rkyv(with = AsString)] PathBuf)`
- Has `.as_path()` method returning `&Path`
- Available in Config aggregate as `self.vault.root()`

### Impact Analysis Results
- GitNexus impact on SchemaConfigSpec: **LOW RISK**
- 0 direct dependents (impactedCount: 0)
- 0 processes affected
- 0 modules affected
- Symbol has 2 properties: `directory` and `property_bank`
- No incoming references (no callers/importers detected by gitnexus)

### Key Consumers
1. **aggregate.rs:226** - `Config::to_schema_spec()` - CONSTRUCTOR
2. **discovery.rs:197** - `DiscoveryEngine::run()` - CONSUMER
3. **builder.rs:61** - `Builder::load_all()` - CONSUMER

### DirPath and FilePath Types Discovery
- `DirPath` and `FilePath` exist in `lithos-core/src/fs/path.rs`
- Both wrap `PathBuf` internally: `pub struct FilePath(#[rkyv(with = AsString)] PathBuf)`
- Both have `.new(PathBuf)` that validates filesystem (checks `.is_file()` / `.is_dir()`)
- **CRITICAL**: Both have `impl From<PathBuf>` that bypasses filesystem validation (lines 473, 653)
- Both have `.as_path()` accessor method
- Both can be absolute or relative paths
- FilePath::new() checks path is not empty and path.is_file()
- DirPath::new() checks path is not empty and path.is_dir()
- Using `From<PathBuf>` is safe for absolute paths constructed via join operations

### Impact Analysis on DiscoveryEngine::run
- **Risk Level**: LOW
- **Direct callers (d=1)**: 2 test functions + 1 production caller
  1. `accepts_read_repository_only` (test in discovery.rs)
  2. `run_skips_schema_batch_lookups_when_no_schema_files_exist` (test in discovery.rs)
  3. `Builder::load_all` (production in builder.rs:54)
- **Affected processes**: 3 builder processes
  - `Builder_load_all_orchestrates_discovery → Directory`
  - `Builder_load_all_orchestrates_discovery → New`
  - `Builder_load_all_orchestrates_discovery → Property_bank`
- **Affected modules**: Schema module only
- **Impact**: Changing signature will require updating 3 call sites (1 prod + 2 tests)

### Impact Analysis on Builder::load_all
- **Risk Level**: LOW
- **Direct callers (d=1)**: 1 test function
  1. `builder_load_all_orchestrates_discovery` (test in builder.rs:350)
- **Affected processes**: 11 related test processes
- **Impact**: Minimal - only test code affected

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Use `DirPath` for schema directory | Type-safe, matches domain (it's a directory), has From<PathBuf> |
| Use `FilePath` for property bank | Type-safe, matches domain (it's a file), has From<PathBuf> |
| Use `From<PathBuf>` not `.new()` | Bypass filesystem validation for constructed paths |
| Resolve paths in `to_schema_spec()` | Single responsibility: Config owns vault root, joins there |
| Keep accessor methods `directory()` and `property_bank()` | Maintains encapsulation pattern used throughout crate |
| Don't serialize SchemaConfigSpec | Already doesn't derive Archive/Serialize - runtime-only type |
| Remove `vault_root` param from `DiscoveryEngine::run()` | Impact: 3 call sites (1 prod + 2 tests), LOW risk |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| GitNexus ambiguity on SchemaConfigSpec | Used target_uid to specify Struct variant |

## Resources
Files to modify:
- `lithos-core/src/config/paths.rs` - SchemaConfigSpec struct definition
- `lithos-core/src/config/aggregate.rs` - Config::to_schema_spec() implementation
- `lithos-core/src/schema/discovery.rs` - DiscoveryEngine::run() signature
- `lithos-core/src/schema/builder.rs` - Builder::load_all() usage

Test files to update:
- `lithos-core/src/config/paths.rs` (mod tests::schema_config_spec)
- `lithos-core/src/config/aggregate.rs` (mod tests - to_schema_spec tests)
- `lithos-core/src/schema/discovery.rs` (mod tests)
- `lithos-core/src/schema/builder.rs` (mod tests)

## Visual/Browser Findings
N/A - code analysis only
