---
title: "Issue 05: Schema context hard cut from RelativePath to PathKey"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
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

Canonical key derivation policy for all interfaces below:
- Use `SchemaConfigSpec` key-oriented methods where the path is
  config-derived and already represented canonically by config APIs.
- Use `FsPath`/entry conversion (`as_key(root)` path) where the path originates
  from discovered filesystem entries.
- Do not use ad hoc `strip_prefix + RelativePath::try_from` conversion chains
  at schema repository/storage boundaries.

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
- `DiscoveryEngine::separate_property_bank`: Replace manual `strip_prefix` + `RelativePath::try_from` with canonical key derivation for discovered entries (`file.path().as_key(spec.root())` path).
- `Builder::load_property_bank`: Replace manual `strip_prefix` chain with canonical key derivation using `SchemaConfigSpec` key methods when config-derived, and entry-based conversion when sourced from discovered filesystem entries.
- Update `query_cached_state` to accept `PathKey`.

**Acceptance criteria:**
- [ ] All schema repository boundary signatures use `PathKey` exclusively.
- [ ] Manual `strip_prefix + RelativePath::try_from` chains are removed from discovery and builder call sites.
- [ ] All schema integration tests pass, confirming accurate key round-tripping through Redb storage.

**Out of scope:**
- Vault context or note context repository signatures.

## TDD & Implementation Plan

This issue must be delivered as strict vertical-slice TDD (one RED->GREEN cycle
at a time). Do not batch all tests first.

### 1. Design decisions and test boundaries

**Canonical key policy:**
- Repository/storage boundaries in Schema context take `PathKey` only.
- Root-scoped key derivation should use the most direct domain interface:
  - Use `SchemaConfigSpec` key-oriented methods where available (for known
    config-derived paths like property bank).
  - Use `FsPath`/entry conversion (`as_key(root)` path) for discovered
    filesystem entries.
- No manual `strip_prefix + RelativePath::try_from` chains remain at schema
  boundaries.

**Allowed `PathKey` construction sources (hard rule):**
- `FsPath::as_key(root)`
- `FilePath::as_key(root)`
- `DirPath::as_key(root)`
- `SchemaConfigSpec::{property_bank_key, schema_directory_key}` for
  config-derived canonical keys

**Forbidden patterns (hard rule):**
- `PathKey::try_new(...)` when input originates from `RelativePath` or ad hoc
  boundary strings
- Any new helper that converts `RelativePath -> PathKey` through string
  extraction
- Any boundary-level `strip_prefix + RelativePath::try_from` chain

**Repository boundary rule:**
- Only `PathKey` crosses `ReadRepository` / `WriteRepository` path-key APIs.
- If internal domain models still contain `RelativePath` during migration,
  boundary conversion must originate from root-scoped filesystem/config objects,
  not from `RelativePath` string bridging.

**Units under test:**
- `ReadRepository` + `WriteRepository` path-keyed lookup/write APIs.
- Redb read/write adapters and table key serialization for schema path indexes.
- Discovery/Builder orchestration where filesystem/config paths are transformed
  into canonical keys.

**Test-suite standards to follow:**
- Follow `docs/engineering/testing/unit.md` for scope, determinism, and
  explicit assertions.
- Follow `docs/engineering/testing/unit-naming.md` naming formula:
  verb-first names (`returns_*`, `rejects_*`, `accepts_*`) and Structure A
  submodules when file complexity requires it.

### 2. Behavior inventory (priority order)

1. Repository read path lookups accept canonical `PathKey` and preserve lookup
   semantics.
2. Repository write path APIs accept canonical `PathKey` and round-trip through
   Redb without key drift.
3. Discovery converts scanned filesystem entries into canonical keys at the
   boundary before repository access.
4. Builder property-bank flow uses canonical key methods from
   `SchemaConfigSpec`/entry APIs and no ad hoc prefix stripping.
5. Batch lookups use borrowed slices (`&[PathKey]`) and avoid unnecessary
   cloning.

### 2.1 Migration scope guardrails

**Must migrate in this issue:**
- `lithos-core/src/schema/repository.rs` path-keyed trait signatures
- `lithos-core/src/schema/storage/{mod,read,write,testing}.rs`
  repository/storage key boundaries
- `lithos-core/src/schema/discovery.rs` boundary calls (`separate_property_bank`,
  `query_cached_state`)
- `lithos-core/src/schema/builder.rs` property-bank boundary key derivation

**May remain `RelativePath` for now (unless scope is explicitly expanded):**
- Internal schema domain payloads/views not used as repository boundary
  contracts
- Error payload fields not part of repository/storage boundary signatures

Goal: enforce a hard cut at schema repository/storage boundaries without
requiring an unrelated full-domain type migration in one issue.

### 3. RED->GREEN slices

#### Slice 0 (Policy gate): boundary contract and forbidden-pattern guard

**Behavior:** schema repository/storage boundaries reject `RelativePath` APIs and
avoid forbidden conversion patterns.

**RED:**
- Add/adjust tests and compile-time API usage so boundary methods fail until
  signatures are `PathKey`-only.
- Add migration review checklist entry to fail review when forbidden patterns
  appear in touched schema migration files.

**GREEN:**
- Boundary path APIs in repository/storage compile and pass with `PathKey`
  inputs only.
- No forbidden conversion pattern remains in touched files.

#### Slice A (Tracer bullet): single-path read lookup via `PathKey`

**Behavior:** a raw schema view is retrievable by canonical key.

**RED:**
- Update an existing schema storage integration test to call
  `find_raw_schema_view_by_path` with `PathKey`.
- Keep assertion focused on behavior (view found/not found), not internals.

**GREEN:**
- Migrate trait signature and implementations (`ReadRepository` + storage test
  repository adapters) for this method to `&PathKey`.
- Keep changes minimal to pass only this test.

#### Slice B: batch read lookup via borrowed `&[PathKey]`

**Behavior:** multiple raw schema views are returned in input order when queried
by canonical keys.

**RED:**
- Add/adjust test for `find_raw_schema_views_by_paths` using `Vec<PathKey>` at
  call site and `&[PathKey]` at boundary.

**GREEN:**
- Migrate trait and implementations to `paths: &[PathKey]`.
- Update storage readers and mapping logic while preserving order semantics.

#### Slice C: schema-id lookup by canonical keys

**Behavior:** ID lookups by path (`single` and `batch`) work with `PathKey`.

**RED:**
- Update or add tests for `find_schema_id_by_path` and
  `find_schema_ids_by_paths` to use `PathKey` values.

**GREEN:**
- Migrate signatures and implementations for these APIs.
- Ensure error and `None` behavior remain unchanged.

#### Slice D: property-bank raw view lookup/write use `PathKey`

**Behavior:** raw property bank views persist and reload with canonical key.

**RED:**
- Add or extend integration test to save and fetch raw property bank view via
  `PathKey` (including reopen/survive-restart scenario if an existing test is
  already present).

**GREEN:**
- Migrate `get_raw_property_bank_view` and `save_raw_property_bank_view`
  signatures and implementations.
- Update Redb key encoding/decoding adapters as needed.

#### Slice E: discovery boundary emits canonical keys

**Behavior:** discovery separates property bank and schema entries while
producing canonical schema keys for downstream cache queries.

**RED:**
- Add/adjust a discovery-focused test that exercises the public loader flow and
  validates key-based lookup behavior indirectly (no private function coupling).

**GREEN:**
- Replace manual `strip_prefix + RelativePath::try_from` in discovery with
  canonical key derivation from file entries.
- Update `query_cached_state` inputs and repository calls to `PathKey`.
- Use config key methods (`SchemaConfigSpec::property_bank_key`) for
  config-derived property-bank key access.

#### Slice F: builder property-bank key derivation policy

**Behavior:** builder property-bank handling uses canonical config/entry key
derivation and preserves existing load/update behavior.

**RED:**
- Extend existing loader tests covering property bank ingest/persist/update.
- Assert behavior remains stable after key migration.

**GREEN:**
- Replace builder manual prefix-stripping conversion chain with direct canonical
  key derivation using `SchemaConfigSpec` key methods where available and
  entry-based conversion when the path originates from discovered filesystem
  entries.
- Do not introduce `RelativePath -> PathKey` string bridge helpers.

### 4. Refactor pass (after all slices are green)

- Remove stale `RelativePath` path-key boundary code, examples, and docs in
  schema repository/storage APIs.
- Normalize touched test names/modules to unit naming standard when a block is
  materially edited.
- Remove unnecessary clones and owned intermediates in hot lookup paths.

### 5. Verification gates

Run at minimum:
- `mise run test:unit`
- Targeted integration tests for schema storage/loader paths under change
- `mise run test`

Optional final quality gate before merge:
- `mise run quality`

### 6. Per-slice checklist (repeat for every RED->GREEN cycle)

- [ ] Test names follow verb-first naming and one behavior per test.
- [ ] Test exercises public behavior, not private implementation details.
- [ ] Minimal implementation change to satisfy current failing test only.
- [ ] No `unwrap()`/`panic!` introduced in production code.
- [ ] Borrowing preferred (`&PathKey`, `&[PathKey]`) over cloning.

### 7. Migration-specific definition of done

- [ ] No path-based schema repository boundary signature accepts
      `RelativePath`.
- [ ] No boundary-level `strip_prefix + RelativePath::try_from` chain remains
      in touched schema migration files.
- [ ] No `PathKey::try_new(...)` call in touched migration files uses inputs
      sourced from `RelativePath`/ad hoc boundary strings.
- [ ] Discovery and builder boundary flows use only allowed canonical key
      derivation sources.
- [ ] `mise run test:unit`, targeted schema integration tests, and
      `mise run test` pass.
