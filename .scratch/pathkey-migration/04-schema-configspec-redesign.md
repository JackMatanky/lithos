---
title: "Issue 04: Redesign SchemaConfigSpec around config semantics"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 04: Redesign SchemaConfigSpec around config semantics

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Redesign `SchemaConfigSpec` to store config semantics (`root`, `schema_directory`, `property_bank_file`) and derive operational paths/keys at boundary seams.

## Agent Brief

**Category:** enhancement
**Summary:** Redesign `SchemaConfigSpec` to store config declarations and derive operational paths/keys lazily without existence checks.

**Current behavior:**
`SchemaConfigSpec` stores `RelativePath`s but exposes mixed relative/absolute accessors with `unreachable!` branches. It forces target path existence checks at config construction time (causing panics via `expect()`) instead of at operational boundaries.

**Desired behavior:**
`SchemaConfigSpec` becomes execution-facing. It stores exact configuration semantics (`RelativeDirPath`, `RelativeFilePath`). Operational execution paths (`DirPath`, `FilePath`) and persistence boundaries (`PathKey`) are derived lazily via fallible methods. The constructor never checks filesystem existence.

**Key interfaces:**

1. **Struct Definition:**
```rust
pub struct SchemaConfigSpec {
    root: VaultRoot, // Thin wrapper over DirPath
    schema_directory: RelativeDirPath,
    property_bank_file: RelativeFilePath,
}
```

2. **Derived Methods (Fallible):**
```rust
impl SchemaConfigSpec {
    // Uses root.as_dir_path().append_dir(&self.schema_directory)
    pub fn schema_directory_path(&self) -> Result<DirPath, PathError> { /* ... */ }

    // Uses root.as_dir_path().append_file(&self.property_bank_file)
    pub fn property_bank_file_path(&self) -> Result<FilePath, PathError> { /* ... */ }

    // Derives via schema_directory_path()?.as_key(root)
    pub fn schema_directory_key(&self) -> Result<PathKey, PathError> { /* ... */ }

    // Derives via property_bank_file_path()?.as_key(root)
    pub fn property_bank_key(&self) -> Result<PathKey, PathError> { /* ... */ }
}
```

3. **Construction:**
- `Config::to_schema_spec()` must be refactored to use fallible conversions (`Result<SchemaConfigSpec, ConfigError>`) instead of panic-based `expect()`.

**Acceptance criteria:**
- [ ] `SchemaConfigSpec` stores `VaultRoot`, `RelativeDirPath`, and `RelativeFilePath`.
- [ ] `VaultRoot` is migrated from a raw `PathBuf` wrapper to a thin newtype over `DirPath`.
- [ ] Derived methods materialize execution paths using the `DirPath::append_*` generic seam.
- [ ] Derived methods expose repository boundary keys (`PathKey`).
- [ ] Construction does not require the schema directory or property bank file to exist on disk.
- [ ] `Config::to_schema_spec()` uses no `expect()` or panics for path assembly.

**Out of scope:**
- Schema repository hard cuts (done in Slice 05).
