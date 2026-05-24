---
title: "Issue 10: Start RelativePath deprecation and prevent reintroduction"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 10: Start RelativePath deprecation and prevent reintroduction

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Begin staged `RelativePath` retirement by deprecating remaining approved uses and adding enforcement to prevent new references.

## Agent Brief

**Category:** enhancement
**Summary:** Start enforced `RelativePath` deprecation with CI/architecture gates against reintroduction.

**Current behavior:**
`RelativePath` is formally deprecated in intention, but no automated guardrails prevent developers from adding new uses during ongoing migration efforts.

**Desired behavior:**
`RelativePath` receives a formal `#[deprecated]` attribute detailing migration strategy. Architecture tests/lints strictly block the introduction of new `RelativePath` usages, confining existing ones to a legacy whitelist.

**Key interfaces:**
- `RelativePath` struct definition (`#[deprecated(note="...")]`).
- Architecture test module (`lithos-core/tests/path_migration_architecture.rs`).

**Acceptance criteria:**
- [ ] `RelativePath` struct holds a `#[deprecated(note = "...")]` attribute outlining the 3-tier taxonomy.
- [ ] Architecture tests explicitly fail if `RelativePath` is used in schema, vault, or note repository boundaries.
- [ ] Allowed legacy uses are contained and verified via code checks.
- [ ] CI fails on unauthorized new `RelativePath` references.

**Out of scope:**
- Complete purging of all `RelativePath` references from the codebase.
