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
- Note repository/storage trait signatures
- Template repository/storage trait signatures

**Acceptance criteria:**
- [ ] Zero instances of `RelativePath` in note and template repository traits.
- [ ] Boundary conversions reliably map target files to `PathKey`s via root scope.
- [ ] End-to-end functionality for notes and templates is verified by existing test suites.
- [ ] Traceable to PRD User Stories: #5, #12, #23.

**Out of scope:**
- Broad removal of `RelativePath` outside of repository signatures.
