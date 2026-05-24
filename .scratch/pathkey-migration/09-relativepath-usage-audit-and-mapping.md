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
- A markdown inventory file: `.scratch/pathkey-migration/relativepath-usage-inventory.md`
- Target replacement types: `PathKey`, `DirPath`, `FilePath`, `RelativeDirPath`, `RelativeFilePath`.

**Acceptance criteria:**
- [ ] An inventory artifact is committed listing all remaining `RelativePath` usage.
- [ ] Each entry designates the target type with a brief rationale based on semantic boundaries.
- [ ] Any ambiguous/unresolvable usages are explicitly flagged.
- [ ] Follow-up migration tasks are linked from inventory sections.

**Out of scope:**
- Executing the actual code replacements defined in the inventory.

## TDD & Implementation Plan

### 1. Planning & Design
- This is a documentation/audit slice, verifying completeness of the migration strategy.

### 2. Tracer Bullet: Audit Validation
**Behavior:** System identifies all remaining `RelativePath` references.
- **RED:** Write a CI script/test that searches the codebase for `RelativePath` and fails if unmapped usages exist.
- **GREEN:** Generate `.scratch/pathkey-migration/relativepath-usage-inventory.md` mapping every finding to its target type.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added
