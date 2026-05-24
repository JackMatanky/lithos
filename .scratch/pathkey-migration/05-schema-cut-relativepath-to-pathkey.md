---
title: "Issue 05: Schema context hard cut from RelativePath to PathKey"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 05: Schema context hard cut from RelativePath to PathKey

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Perform schema context hard cut so all repository/storage boundaries use `PathKey` instead of `RelativePath`.

## Agent Brief

**Category:** enhancement
**Summary:** Complete schema-context repository/storage migration from `RelativePath` to `PathKey`.

**Current behavior:**
Schema repository trait signatures, discovery pipelines (`DiscoveryEngine`), and config-to-schema handoffs (`Builder`) use `RelativePath`, requiring ad hoc `strip_prefix` chains.

**Desired behavior:**
All Schema-related repository traits and storage boundaries mandate `PathKey`. Upstream callers (`DiscoveryEngine`, `Builder`) construct `PathKey`s via `entry.path().as_key(root)` instead of manual prefix stripping.

**Key interfaces:**
- `schema::repository::ReadRepository` & `WriteRepository` (e.g., `find_raw_schema_views_by_paths(&[PathKey])`)
- `schema::storage` table definitions
- `DiscoveryEngine::separate_property_bank`
- `Builder::load_property_bank`

**Acceptance criteria:**
- [ ] Schema boundaries (`ReadRepository`, `WriteRepository`) accept `&PathKey` exclusively; no `RelativePath`.
- [ ] `DiscoveryEngine` and `Builder` use `as_key(root)` for repository lookups without `strip_prefix`.
- [ ] All schema integration tests pass, confirming accurate key round-tripping.
- [ ] Traceable to PRD User Stories: #1, #4, #5, #6, #10, #19, #20, #24, #25.

**Out of scope:**
- Vault context or note context repository signatures.
