---
title: "Issue 04: Redesign SchemaConfigSpec around config semantics"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
date_completed: null
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
- [ ] `SchemaConfigSpec` stores `DirPath`, `RelativeDirPath`, and `RelativeFilePath`.
- [ ] `VaultRoot` is migrated from a raw `PathBuf` wrapper to a thin newtype over `DirPath`.
- [ ] Derived methods materialize execution paths using the `DirPath::append_*` generic seam.
- [ ] Derived methods expose repository boundary keys (`PathKey`).
- [ ] Construction does not require the schema directory or property bank file to exist on disk.
- [ ] `Config::to_schema_spec()` uses no `expect()` or panics for path assembly.
- [ ] `SchemaConfigSpec` introduces no new dependency on `config::vault` types.

**Out of scope:**
- Schema repository hard cuts (done in Slice 05).

## TDD & Implementation Plan

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
- **RED:** Write `test_schema_config_spec_new` with valid syntax but non-existent files. Assert no panics.
- **GREEN:** Change `SchemaConfigSpec` fields. Remove any `expect()` or `fs::metadata` checks from `Config::to_schema_spec()`. Return `Result<SchemaConfigSpec, ConfigError>`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Deriving Execution Paths
**Behavior:** System derives operational file/dir paths using the append seam on demand.
- **RED:** Write `test_schema_directory_path` asserting correct join of root + relative config.
- **GREEN:** Implement `schema_directory_path` and `property_bank_file_path` using `append_dir` and `append_file`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 4. Incremental Loop: Deriving Persistence Keys
**Behavior:** System derives persistence boundary keys (`PathKey`) using root-scoping on demand.
- **RED:** Write `test_property_bank_key` asserting correct `PathKey` representation.
- **GREEN:** Implement `schema_directory_key` and `property_bank_key` utilizing `.as_key(root)`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 5. Refactor
- [ ] Ensure all `expect()` calls in `to_schema_spec()` are converted to `Result` handling with `?` (Rust Best Practice: Error Handling).
- [ ] Ensure `VaultRoot` is a thin newtype over `DirPath`.
- [ ] Ensure `SchemaConfigSpec` remains decoupled from `VaultRoot` by storing root as `DirPath`.
