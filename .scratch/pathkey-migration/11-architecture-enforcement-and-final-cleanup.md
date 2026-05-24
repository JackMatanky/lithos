---
title: "Issue 11: Architecture enforcement and final cleanup for path migration"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 11: Architecture enforcement and final cleanup for path migration

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Finalize migration governance with architecture tests enforcing boundary rules, remove transitional aliases when criteria are met, and publish cleanup documentation.

## Agent Brief

**Category:** enhancement
**Summary:** Finalize boundary enforcement and retire transitional path-migration artifacts.

**Current behavior:**
Migration scaffolding (like the `NormalizedPath` alias and isolated `RelativePath` references) persists, waiting for final validation before deletion.

**Desired behavior:**
The migration concludes. The architecture enforces rigid path taxonomy (`Relative*Path` in config, `DirPath`/`FilePath` in I/O, `PathKey` at repository bounds). Transitional aliases (`NormalizedPath`) and the `RelativePath` type itself are deleted.

**Key interfaces:**
- Architecture test suite
- `PathKey` and `fs::path` module exports

**Acceptance criteria:**
- [ ] The `NormalizedPath` type alias is completely deleted.
- [ ] The `RelativePath` type is completely deleted, governed by the previously compiled usage inventory.
- [ ] Architecture tests are hardened to enforce the strict 3-tier boundary rules permanently.
- [ ] Traceable to PRD User Stories: #13, #14, #21, #22.

**Out of scope:**
- Introducing new path semantics.
- Re-opening earlier repository migration contexts.
