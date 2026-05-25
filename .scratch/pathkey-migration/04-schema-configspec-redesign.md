---
title: "Issue 04: Redesign SchemaConfigSpec around config semantics"
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-25
date_completed: 2026-05-25
---

# Issue 04: Redesign SchemaConfigSpec around config semantics

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Redesign `SchemaConfigSpec` to store config semantics (`root`, `schema_directory`, `property_bank_file`) and derive operational paths/keys at boundary seams.

Decision update: even though `VaultRoot` must migrate from `PathBuf` to `DirPath`, `SchemaConfigSpec` should store `DirPath` directly (not `VaultRoot`) so `config::paths` does not need to import additional `config` components.

## Agent Brief

**Category:** enhancement
**Summary:** Redesign `SchemaConfigSpec` to store config declarations and derive operational paths/keys lazily without existence checks.

**Current behavior:**
`SchemaConfigSpec` stores `RelativePath`s but exposes mixed relative/absolute accessors with `unreachable!` branches. It forces target path existence checks at config construction time (causing panics via `expect()`) instead of at operational boundaries.

**Desired behavior:**
`SchemaConfigSpec` becomes execution-facing. It stores exact configuration semantics (`DirPath`, `RelativeDirPath`, `RelativeFilePath`). Operational execution paths (`DirPath`, `FilePath`) and persistence boundaries (`PathKey`) are derived lazily via fallible methods. The constructor never checks filesystem existence.

`VaultRoot` still migrates to a thin newtype over `DirPath`, but `SchemaConfigSpec` consumes the root as `DirPath` to keep the spec boundary minimal and avoid coupling `config::paths` to extra `config` modules.

**Key interfaces:**

1. **Struct Definition:**
```rust
pub struct SchemaConfigSpec {
    root: DirPath,
    schema_directory: RelativeDirPath,
    property_bank_file: RelativeFilePath,
}
```

2. **Derived Methods (Fallible):**
```rust
impl SchemaConfigSpec {
    // Uses root.append_dir(&self.schema_directory)
    pub fn schema_directory_path(&self) -> Result<DirPath, PathError> { /* ... */ }

    // Uses root.append_file(&self.property_bank_file)
    pub fn property_bank_file_path(&self) -> Result<FilePath, PathError> { /* ... */ }

    // Derives via schema_directory_path()?.as_key(root)
    pub fn schema_directory_key(&self) -> Result<PathKey, PathError> { /* ... */ }

    // Derives via property_bank_file_path()?.as_key(root)
    pub fn property_bank_key(&self) -> Result<PathKey, PathError> { /* ... */ }
}
```

3. **Construction:**
- `Config::to_schema_spec()` must be refactored to use fallible conversions (`Result<SchemaConfigSpec, ConfigError>`) instead of panic-based `expect()`.
- `Config::to_schema_spec()` should pass `metadata.root()` as `DirPath` into `SchemaConfigSpec` (extracting from `VaultRoot` at the config seam).

**Acceptance criteria:**
- [x] `SchemaConfigSpec` stores `DirPath`, `RelativeDirPath`, and `RelativeFilePath`.
- [x] `VaultRoot` is migrated from a raw `PathBuf` wrapper to a thin newtype over `DirPath`.
- [x] Derived methods materialize execution paths using the `DirPath::append_*` generic seam.
- [x] Derived methods expose repository boundary keys (`PathKey`).
- [x] Construction does not require the schema directory or property bank file to exist on disk.
- [x] `Config::to_schema_spec()` uses no `expect()` or panics for path assembly.
- [x] `SchemaConfigSpec` introduces no new dependency on `config::vault` types.

**Out of scope:**
- Schema repository hard cuts (done in Slice 05).

## GitNexus impact analysis (pre-implementation)

Run date: 2026-05-25.

### Symbols analyzed
- `Struct:lithos-core/src/config/vault.rs:VaultRoot`
- `Struct:lithos-core/src/config/paths.rs:SchemaConfigSpec`
- `Function:lithos-core/src/config/aggregate.rs:Config.to_schema_spec#0`
- `Function:lithos-core/src/schema/builder.rs:load_all`

### Findings
- `Config::to_schema_spec` is the critical seam for this issue. It currently uses panic-based conversion (`expect`) and feeds schema discovery.
- `SchemaBuilder::load_all` calls `self.config.to_schema_spec()` directly. Any signature/behavior change in `to_schema_spec` must be propagated to schema loading error flow.
- Direct graph impact for struct nodes is low/noisy, but function-level call paths confirm this is an execution-path-sensitive change.
- Existing direct callers detected for `to_schema_spec` include config tests and the schema ingestion path.

### Risk assessment
- **Overall:** MEDIUM (function-level blast radius is small, but path is operationally central to schema ingestion).
- **Primary risk:** changing panic behavior to `Result` changes error semantics and requires caller updates.
- **Secondary risk:** `VaultRoot(PathBuf) -> VaultRoot(DirPath)` can affect constructor and conversion surfaces.

### Affected call sites to update
- `lithos-core/src/config/aggregate.rs` (`to_schema_spec` definition and tests)
- `lithos-core/src/schema/builder.rs` (`load_all` call site and error propagation)

### Constraints to preserve
- Keep `SchemaConfigSpec` in `config::paths` storing `DirPath` directly.
- Do not introduce dependency from `config::paths` to `config::vault` types.
- Keep filesystem existence checks out of `SchemaConfigSpec` construction.

## TDD & Implementation Plan

## Implementation notes and current status (2026-05-25)

Completed with follow-up dependency on Issue `config-spec-errors/01` (`.scratch/config-spec-errors/01-projection-error-boundary.md`).

### Landed changes

- `SchemaConfigSpec` now stores declarative config semantics with `DirPath` root and relative declarations in `lithos-core/src/config/paths.rs`.
- Field names were finalized as `directory` and `property_bank` (instead of `schema_directory` and `property_bank_file`) to align with existing config language while preserving the same type semantics.
- Fallible derived methods were added:
  - `schema_directory_path() -> Result<DirPath, PathError>`
  - `property_bank_file_path() -> Result<FilePath, PathError>`
  - `schema_directory_key() -> Result<PathKey, PathError>`
  - `property_bank_key() -> Result<PathKey, PathError>`
- `VaultRoot` migrated to a thin newtype over `DirPath` in `lithos-core/src/config/vault.rs`.
- `Config::to_schema_spec()` now returns `Result<SchemaConfigSpec, ConfigError>` and no longer uses panic-based assembly.
- `SchemaBuilder::load_all` was updated to handle fallible `to_schema_spec()`.

### Tests added/updated

- `schema_config_spec_constructor_accepts_declarative_nonexistent_targets`
- `schema_directory_path_returns_dirpath_when_root_and_relative_dir_are_valid`
- `property_bank_file_path_returns_filepath_when_root_and_relative_file_are_valid`
- `schema_directory_key_returns_pathkey_when_root_scoped_dir_is_valid`
- `property_bank_key_returns_pathkey_when_root_scoped_file_is_valid`
- `to_schema_spec_returns_result_without_panicking`

### Verification notes

- Hooks passed on commit for formatting, clippy, and tests.
- Additional doctest fixes were applied for `config::vault::Metadata` examples to avoid filesystem-dependent execution failures.

### Remaining follow-up (moved to dedicated stream)

- Cross-context projection errors are still adapted with string-based mapping at the schema builder seam.
- Follow-up issue created: `.scratch/config-spec-errors/01-projection-error-boundary.md`.
- That follow-up introduces a shared projection error enum in `lithos-core/src/config/error.rs` so downstream contexts can import a narrow error contract instead of full `ConfigError`.

### 1. Planning & Design
**Deep Modules / Testability:**
- `SchemaConfigSpec` strictly stores declarative semantics. Execution paths and persistence boundaries are derived lazily.
- Removes panics and FS checks from config loading.
- Maintains clean context boundaries: `SchemaConfigSpec` stays in `config::paths` and stores `DirPath` directly.

**Behaviors to Test (Prioritized):**
1. System successfully loads configuration semantics without interacting with the filesystem.
2. System derives operational file/dir paths using the append seam on demand.
3. System derives persistence boundary keys (`PathKey`) using root-scoping on demand.
4. System preserves module boundary hygiene by avoiding `VaultRoot` in `SchemaConfigSpec`.

### 2. Tracer Bullet: Pure Declarative Construction
**Behavior:** System successfully loads configuration semantics without interacting with the filesystem.
- **RED:** Write `to_schema_spec_returns_error_instead_of_panicking_when_projection_fails` and `schema_config_spec_constructor_accepts_declarative_nonexistent_targets`.
- **GREEN:** Change `SchemaConfigSpec` fields. Remove any `expect()` or `fs::metadata` checks from `Config::to_schema_spec()`. Return `Result<SchemaConfigSpec, ConfigError>` and map path errors explicitly.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 3. Incremental Loop: Deriving Execution Paths
**Behavior:** System derives operational file/dir paths using the append seam on demand.
- **RED:** Write `schema_directory_path_returns_dirpath_when_root_and_relative_dir_are_valid` and `property_bank_file_path_returns_filepath_when_root_and_relative_file_are_valid`.
- **GREEN:** Implement `schema_directory_path` and `property_bank_file_path` using `append_dir` and `append_file`.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 4. Incremental Loop: Deriving Persistence Keys
**Behavior:** System derives persistence boundary keys (`PathKey`) using root-scoping on demand.
- **RED:** Write `property_bank_key_returns_pathkey_when_root_scoped_file_is_valid` and `schema_directory_key_returns_pathkey_when_root_scoped_dir_is_valid`.
- **GREEN:** Implement `schema_directory_key` and `property_bank_key` utilizing `.as_key(root)`.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 5. Refactor
- [x] Ensure all `expect()` calls in `to_schema_spec()` are converted to `Result` handling with `?` (Rust Best Practice: Error Handling).
- [x] Ensure `VaultRoot` is a thin newtype over `DirPath`.
- [x] Ensure `SchemaConfigSpec` remains decoupled from `VaultRoot` by storing root as `DirPath`.

### 6. Caller adaptation and regression checks
- [x] Update `SchemaBuilder::load_all` to handle `Config::to_schema_spec() -> Result<_, _>` without introducing panic paths.
- [ ] Add/adjust tests to verify schema loading surfaces config-spec construction failures as typed errors. (Moved to `.scratch/config-spec-errors/01-projection-error-boundary.md`)
- [x] Re-run existing schema loader integration tests that exercise property bank and discovery flows.

## Test naming and quality gates (project standards)

- Use canonical test modules (`constructor`, `validation`, `conversions`, `lookup`) as applicable.
- Use verb-first, single-behavior names (for example `returns_error_when_*`, `returns_pathkey_when_*`).
- Keep tests deterministic and avoid hidden assertions.
- Prefer public-interface assertions over implementation details.
- Run:
  - `mise run test:unit`
  - `mise run test`
  - `mise run lint`
  - `mise run fmt`
