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
Vault repository and storage interfaces are strictly updated to `PathKey`. Callers must derive these keys using root-scoped seams before crossing the vault repository boundary.

**Key interfaces:**
- Vault repository read/write traits
- Vault persistence storage definitions
- Vault-level `as_key(root)` derivations at call sites

**Acceptance criteria:**
- [ ] Vault repository interfaces no longer reference `RelativePath`; they mandate `PathKey`.
- [ ] All vault integration and unit tests pass, verifying key lookups are unbroken.
- [ ] Caller derivation relies on formal `as_key(root)` boundaries.
- [ ] Traceable to PRD User Stories: #5, #11, #23.

**Out of scope:**
- Note and template repository migration.
