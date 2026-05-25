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
The migration concludes. The architecture permanently enforces rigid path taxonomy (`Relative*Path` in config, `DirPath`/`FilePath` in I/O, `PathKey` at repository bounds). Transitional aliases (`NormalizedPath`) and the `RelativePath` type itself are deleted based on the audit.

**Key interfaces:**
- Architecture test suite (`lithos-core/tests/path_migration_architecture.rs`).
- `PathKey` and `fs::path` module exports.

**Acceptance criteria:**
- [ ] Architecture tests enforce: repository boundaries use `PathKey`, `Relative*Path` restricted to config, no conversion methods on `Relative*Path`.
- [ ] The `NormalizedPath` type alias is completely deleted.
- [ ] Final `RelativePath` removal criteria are validated against the usage audit, and the type is deleted if zero references remain.
- [ ] Migration cleanup notes document stable boundary rules and approved conversion seams.

**Out of scope:**
- Introducing new path semantics.

## TDD & Implementation Plan

### 1. Planning & Design
- Delete temporary migration structures and strictly enforce the final state.

**Behaviors to Test (Prioritized):**
1. The codebase compiles and tests pass with `NormalizedPath` completely absent.
2. Architecture tests enforce a zero-tolerance policy on `RelativePath` (pending audit completion).

### 2. Tracer Bullet: Alias Removal
**Behavior:** The codebase compiles and tests pass with `NormalizedPath` completely absent.
- **RED:** Delete `pub type NormalizedPath = PathKey;`. Codebase fails to compile.
- **GREEN:** Resolve any lingering references.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Final Architecture Gate
**Behavior:** Architecture tests enforce a zero-tolerance policy on `RelativePath`.
- **RED:** Write architecture test asserting `RelativePath` does not exist anywhere in `lithos-core` outside of explicitly allowed legacy tests.
- **GREEN:** Ensure the test suite passes, and remove any `#[expect(deprecated)]` tags.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added
