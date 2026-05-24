---
title: "Issue 02: Add passive RelativeDirPath and RelativeFilePath config types"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 02: Add passive RelativeDirPath and RelativeFilePath config types

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Introduce `RelativeDirPath` and `RelativeFilePath` as declarative config value wrappers (string-based), with validation but no conversion/materialization behavior.

## Agent Brief

**Category:** enhancement
**Summary:** Add passive relative config path wrappers (`RelativeDirPath`, `RelativeFilePath`) constrained to declaration-only semantics.

**Current behavior:**
Relative paths in configuration share types with operational I/O or persistence keys, leading to boundary leakage and ad hoc resolution logic scattered across the codebase.

**Desired behavior:**
Configuration declarations use `RelativeDirPath` and `RelativeFilePath`. These are strict string wrappers that validate relative, multi-segment formats (canonical separators, no traversal) but have absolutely no active conversion or materialization methods.

**Key interfaces:**
- `RelativeDirPath` and `RelativeFilePath` structs.
- Construction validation rejecting absolute, empty, or traversal components.

**Acceptance criteria:**
- [ ] Both types are string wrappers, explicitly avoiding `PathBuf`.
- [ ] Validation correctly rejects absolute paths, empty paths, and `.`/`..` components.
- [ ] Types expose basic accessors (e.g., `as_str()`) but NO conversion APIs (`to_path`, `resolve`, `as_key`).
- [ ] Traceable to PRD/Post-PRD User Stories: #17, #23.

**Out of scope:**
- Key or filesystem materialization logic (handled by `DirPath` seams).
