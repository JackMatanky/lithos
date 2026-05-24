---
title: "Issue 06: Vault context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 06: Vault context hard cut from RelativePath to PathKey

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Migrate vault repository/storage boundaries and callers from `RelativePath` to `PathKey` to align with canonical key semantics.

## Agent Brief

**Category:** enhancement
**Summary:** Migrate vault context boundaries to `PathKey` and eliminate `RelativePath` key usage.

**Current behavior:**
Vault boundary APIs use `RelativePath` semantics for persistence-facing keys, decoupling them from the new `PathKey` canonical format.

**Desired behavior:**
Vault repository and storage interfaces are strictly updated to `PathKey`. Callers must derive these keys using root-scoped seams (`as_key(root)`) before crossing the vault repository boundary.

**Key interfaces:**
- Vault repository read/write traits (e.g., `find_vault_file_by_path`, batch methods).
- Vault persistence storage definitions (Redb table keys).
- Call sites orchestrating vault operations.

**Acceptance criteria:**
- [ ] Vault repository interfaces no longer reference `RelativePath`; they mandate `&PathKey` or `&[PathKey]`.
- [ ] Vault storage table definitions use `PathKey`.
- [ ] Caller derivation relies on formal `as_key(root)` boundaries.
- [ ] All vault integration and unit tests pass, verifying key lookups are unbroken.

**Out of scope:**
- Note and template repository migration.

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- Align vault context boundaries to `PathKey` without duplicating persistence logic.

**Behaviors to Test (Prioritized):**
1. Vault repository retrieves files exclusively by canonical keys.

### 2. Tracer Bullet: Vault Repository Takes PathKey
**Behavior:** Vault repository retrieves files exclusively by canonical keys.
- **RED:** Modify `find_vault_file_by_path` tests to construct and pass `&PathKey`.
- **GREEN:** Update vault repository traits and storage table definitions to enforce `PathKey`.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 3. Refactor
- [ ] Review borrowing and ensure table definitions do not gratuitously allocate Strings.
