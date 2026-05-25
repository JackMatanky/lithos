---
title: "Issue 10: Start RelativePath deprecation and prevent reintroduction"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
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

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- Implement compile-time static analysis (architecture tests + `#[deprecated]`) to enforce the migration boundaries automatically.

**Behaviors to Test (Prioritized):**
1. The compiler formally warns on new `RelativePath` usage.
2. Architecture tests explicitly fail if `RelativePath` exists within schema, vault, or note boundaries.

### 2. Tracer Bullet: Deprecation Attribute
**Behavior:** The compiler formally warns on new `RelativePath` usage.
- **RED:** Add `#[deprecated]` to `RelativePath`. Build the project and verify warnings appear.
- **GREEN:** Apply `#[expect(deprecated)]` explicitly to the legacy modules whitelisted in the audit (Rust Best Practice: Linting).
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Architecture Boundary Tests
**Behavior:** Architecture tests explicitly fail if `RelativePath` exists within schema, vault, or note boundaries.
- **RED:** Write a test in `path_migration_architecture.rs` scanning the AST of `src/schema/repository.rs` for `RelativePath`.
- **GREEN:** Run tests. They should pass if the previous slices were completed cleanly, locking the boundary.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added
