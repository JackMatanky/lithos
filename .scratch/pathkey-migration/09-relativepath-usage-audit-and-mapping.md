---
title: "Issue 09: Audit all RelativePath usage and map replacement target types"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 09: Audit all RelativePath usage and map replacement target types

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Create a full inventory of remaining `RelativePath` usage and map each call site to a target type (`PathKey`, `DirPath`, `FilePath`, `RelativeDirPath`, `RelativeFilePath`) before final removal.

## Agent Brief

**Category:** enhancement
**Summary:** Produce authoritative `RelativePath` usage inventory and replacement mapping before final removal.

**Current behavior:**
`RelativePath` references remain scattered in parsing, legacy domain logic, or test utilities. It is unclear exactly which variant (`PathKey`, `Relative*Path`, `DirPath`/`FilePath`) each site should adopt.

**Desired behavior:**
A comprehensive audit maps every single remaining `RelativePath` reference to its strict successor type. The output is a committed markdown document tracking completion state.

**Key interfaces:**
- A markdown inventory (e.g., `.scratch/pathkey-migration/relativepath-usage-inventory.md`)
- Source code references containing `RelativePath`

**Acceptance criteria:**
- [ ] An inventory artifact is committed listing all remaining `RelativePath` usage.
- [ ] Each entry designates the target type (`PathKey`, `DirPath`, `FilePath`, `RelativeDirPath`, or `RelativeFilePath`) with a brief rationale.
- [ ] Any ambiguous/unresolvable usages are explicitly flagged.
- [ ] Traceable to PRD User Stories: #14, #23.

**Out of scope:**
- Executing the actual code replacements defined in the inventory.
