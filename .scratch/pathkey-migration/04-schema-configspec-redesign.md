---
title: "Issue 04: Redesign SchemaConfigSpec around config semantics"
category: "enhancement"
label: "needs-triage"
status: "open"
date_created: "2026-05-25"
date_completed: null
---

# Issue 04: Redesign SchemaConfigSpec around config semantics

Labels: `needs-triage`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Redesign `SchemaConfigSpec` to store config semantics (`root`, `schema_directory`, `property_bank_file`) and derive operational paths/keys at boundary seams.

## Acceptance criteria

- [ ] `SchemaConfigSpec` stores `VaultRoot`, `RelativeDirPath`, and `RelativeFilePath`.
- [ ] Derived methods materialize execution paths through `DirPath::append_*` seam.
- [ ] Derived methods expose repository boundary keys (`PathKey`) as fallible results.
- [ ] Construction does not require schema directory/property-bank file to exist.
- [ ] `Config::to_schema_spec()` is fallible and uses no panic-based `expect()`.

## Blocked by

- `.scratch/pathkey-migration/02-relative-config-types.md`
- `.scratch/pathkey-migration/03-dirpath-append-seam.md`
