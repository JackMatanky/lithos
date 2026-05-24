---
title: "Issue 07: Note/template context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 07: Note/template context hard cut from RelativePath to PathKey

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Migrate note/template repository and storage boundaries from `RelativePath` to `PathKey`, completing canonical key usage across contexts.

## Agent Brief

**Category:** enhancement
**Summary:** Complete note/template context migration to `PathKey` repository/storage boundaries.

**Current behavior:**
Note and template persistence boundaries rely on `RelativePath`, leaving them out of sync with schema and vault context path type standards.

**Desired behavior:**
Note and template repositories process `PathKey` exclusively. This brings the entire system's repository boundaries under the unified `PathKey` paradigm.

**Key interfaces:**
- Note repository read/write trait signatures.
- Template repository read/write trait signatures.
- Note/template storage table key types in Redb schema.
- Boundary conversion call sites orchestrating note/template saves/lookups.

**Acceptance criteria:**
- [ ] Zero instances of `RelativePath` in note and template repository traits.
- [ ] Zero instances of `RelativePath` in note and template storage table definitions.
- [ ] Boundary conversions reliably map target files to `PathKey`s via root scope `as_key(root)`.
- [ ] End-to-end functionality for notes and templates is verified by existing test suites.

**Out of scope:**
- Broad removal of `RelativePath` outside of repository signatures.

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- Align note/template repositories to `PathKey`.

**Behaviors to Test (Prioritized):**
1. Note and template repositories process persistence requests exclusively via canonical keys.

### 2. Tracer Bullet: Note/Template Takes PathKey
**Behavior:** Note and template repositories process persistence requests exclusively via canonical keys.
- **RED:** Modify Note and Template integration tests to construct and pass `PathKey`.
- **GREEN:** Implement `&PathKey` in note/template domain services, applying `.as_key(root)?` at boundary orchestrations.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 3. Refactor
- [ ] Verify `RelativePath` is completely removed from note/template domain service arguments.
