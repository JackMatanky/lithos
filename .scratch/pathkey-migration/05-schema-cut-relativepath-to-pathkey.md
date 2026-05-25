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
Repository trait signatures accept `RelativePath` for database keys. `DiscoveryEngine` and `Builder` contain ad hoc `strip_prefix` + `RelativePath::try_from` conversion chains at every boundary.

**Desired behavior:**
All Schema-related repository traits and storage boundaries mandate `PathKey`. Upstream callers (`DiscoveryEngine`, `Builder`) construct `PathKey`s via `entry.path().as_key(root)` instead of manual prefix stripping.

**Key interfaces:**

1. **Repository Traits (`lithos-core/src/schema/repository.rs`):**
Replace all `&RelativePath` parameters with `&PathKey` in:
- `find_raw_schema_view_by_path`
- `find_raw_schema_views_by_paths`
- `get_raw_property_bank_view`
- `find_schema_id_by_path`
- `find_schema_ids_by_paths`

2. **Storage (`lithos-core/src/schema/storage/*`):**
- Update schema storage table key types from `RelativePath` to `PathKey`.

3. **Call Sites:**
- `DiscoveryEngine::separate_property_bank`: Replace manual `strip_prefix` + `RelativePath::try_from` with `file.path().as_key(spec.root())`.
- `Builder::load_property_bank`: Replace `strip_prefix` logic with `entry.path().as_key(root)`.
- Update `query_cached_state` to accept `PathKey`.

**Acceptance criteria:**
- [ ] All schema repository boundary signatures use `PathKey` exclusively.
- [ ] Manual `strip_prefix + RelativePath::try_from` chains are removed from discovery and builder call sites.
- [ ] All schema integration tests pass, confirming accurate key round-tripping through Redb storage.

**Out of scope:**
- Vault context or note context repository signatures.

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- Repositories exclusively take canonical `&PathKey` references.
- Integration tests must verify end-to-end data retrieval using `PathKey` via the Redb mocks.

**Behaviors to Test (Prioritized):**
1. Schema repositories retrieve data using explicit canonical keys (`PathKey`).
2. Discovery engine converts scanned filesystem paths into canonical keys before querying the repository.

### 2. Tracer Bullet: Repository Takes PathKey
**Behavior:** Schema repositories retrieve data using explicit canonical keys (`PathKey`).
- **RED:** Modify `find_raw_schema_view_by_path` test to pass a `PathKey` instead of `RelativePath`.
- **GREEN:** Update repository traits and storage/Redb implementations to accept `&PathKey`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Discovery Boundary Conversion
**Behavior:** Discovery engine converts scanned filesystem paths into canonical keys before querying the repository.
- **RED:** Write a test verifying `DiscoveryEngine::separate_property_bank` correctly resolves the key.
- **GREEN:** Replace manual `strip_prefix` chains in `DiscoveryEngine` and `Builder` with `.as_key(root)?`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 4. Refactor
- [ ] Batch read operations (`find_raw_schema_views_by_paths`) must take `&[PathKey]` instead of `Vec`s (Rust Best Practice: Borrowing).
