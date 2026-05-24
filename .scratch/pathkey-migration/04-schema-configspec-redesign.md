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
`SchemaConfigSpec` stores `RelativePath`s but exposes mixed relative/absolute accessors. It enforces filesystem existence too early (causing panics via `expect()`) and pushes component joining to consumers.

**Desired behavior:**
The spec stores exact configuration intent (`RelativeDirPath`, `RelativeFilePath`). Operational execution paths (`DirPath`, `FilePath`) and persistence boundaries (`PathKey`) are derived lazily via fallible methods. The constructor never checks filesystem existence.

**Key interfaces:**
- `SchemaConfigSpec` (fields: `root: VaultRoot`, `schema_directory: RelativeDirPath`, `property_bank_file: RelativeFilePath`)
- `SchemaConfigSpec::directory_path() -> Result<DirPath, PathError>`
- `SchemaConfigSpec::directory_key() -> Result<PathKey, PathError>`
- `Config::to_schema_spec() -> Result<SchemaConfigSpec, ConfigError>`

**Acceptance criteria:**
- [ ] `SchemaConfigSpec` initialization is decoupled from target path existence on disk.
- [ ] `directory_key()` and `property_bank_key()` reliably yield `PathKey`s via boundary conversion rules.
- [ ] `Config::to_schema_spec()` uses `TryFrom`/fallible logic—no `expect()` panics.
- [ ] Traceable to PRD User Stories: #2, #3, #9, #17, #18.

**Out of scope:**
- Modifying vault or note repository behavior.
- Full discovery engine refactoring (keys derived here, but usage is next slice).
