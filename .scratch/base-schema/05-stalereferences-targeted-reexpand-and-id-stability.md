---
title: 05-stalereferences-targeted-reexpand-and-id-stability
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- base-schema
- ready-for-agent

## Parent

- `.scratch/base-schema/PRD.md`

## What to build

Add `StaleReferences` handling: when the property bank changes, schema properties
that reference changed bank properties get targeted re-expansion (not full rebuild).
`PropertyId` values are preserved by name. Structural reference conflicts escalate
to full rebuild fallback.

## Acceptance criteria

- [ ] `Fresh`-path schema with `bank_delta` present and matching bank references
      triggers targeted re-expansion of only the affected properties.
- [ ] `Fresh`-path schema with `bank_delta` present but no matching bank references
      returns `Fresh` (no-op, no file read).
- [ ] `Fresh`-path schema with `bank_delta == None` returns `Fresh` (no-op).
- [ ] `Analysis`-stage schema (stale content) also checks bank references against
      `bank_delta` and folds matched properties into `PropertyDelta.upserts`.
- [ ] Targeted re-expansion uses `PropertyMap::with_ids()` to preserve `PropertyId`
      by name for unaffected properties.
- [ ] Structural reference conflict (bank property missing after delta) escalates
      to full rebuild fallback.
- [ ] `property_delta.upserts` contains only the re-expanded properties; no full
      `PropertyMap` replacement.
- [ ] Tests cover: fresh+staleRefs, staleTimestamps+staleRefs,
      staleContent+staleRefs, property ID stability, no-bank-delta no-op,
      no-matching-refs no-op.

## Blocked by

- ~~`.scratch/base-schema/04-base-processor-stale-analysis-and-normalization.md`~~ ✅ Closed

## Agent Brief

**Category:** enhancement
**Summary:** Property bank changes trigger targeted re-expansion of only the schema properties that reference changed bank properties, preserving `PropertyId` stability. Never triggers full rebuild solely due to a bank delta.

**Current behavior:**
`BaseSchemaProcessor` handles three paths: missing → `New`, view+timestamps-match → `Fresh` (fetch cached), view+timestamps-mismatch → staleness pipeline (parse, diff, refresh/update). There is no response to property bank changes. A schema whose file is `Fresh` but whose property bank references point to now-changed bank properties remains stale silently — the next pipeline run returns `Fresh` with outdated expanded values.

The legacy `schema_processor.rs` handles this via `NodeStatus::StaleBankReferences`, which triggers a **full rebuild** of the entire schema. This is unnecessarily expensive: only the properties whose bank targets changed need re-expansion.

**Desired behavior:**

`PropertyBankProcessor::run()` already produces `PropertyBankResolution { bank, delta: Option<HashSet<PropertyName>> }`. The Builder passes `Option<&PropertyBankResolution>` into `BaseSchemaProcessor::run()`. The `StaleReferences` response is an **orthogonal, targeted intervention** that occurs in two places — it is **not** a new typestate pipeline branch.

**Path A — Fresh + StaleReferences (orthogonal check after timestamp match):**

```
Init/run(view=Some, source, bank_resolution=Some(res))
  → Present → check_timestamps
    → Match → Construction/Fresh → check bank refs
      → if res.delta() is Some and non-empty:
        → SchemaVersion::changed_bank_references(res.delta())
        → if non-empty:
          → parse file to RawSchema (forced read, but only for ref targets)
          → re-expand only the affected properties against res.bank()
          → merge with existing PropertyMap via with_ids() — preserves IDs for unaffected
          → persist updated BaseSchema + new SchemaVersion
          → return Stale { property_delta: upserts=[re-expanded], ...empties... }
        → if empty:
          → return Fresh (no-op, no file read)
      → if res.delta() is None:
        → return Fresh (no-op, no file read)
```

**Path B — Analysis + StaleReferences (additional delta during stale pipeline):**

```
  → Mismatch → Suspect → check_content → Mismatch → Parsed → Analysis
    → compute property/excludes/extends deltas (existing 04 logic)
    → if bank_delta is Some and non-empty:
      → SchemaVersion::changed_bank_references(bank_delta)
      → if non-empty: add re-expanded properties to PropertyDelta.upserts
    → continue existing Empty/Delta/Corrupt branch logic
```

Key invariant: a bank delta **never** triggers full rebuild on its own. Full rebuild fallback is reserved for structural reference conflicts only.

**PropertyId stability:**
`PropertyMap::with_ids(existing)` already exists. The targeted re-expansion path applies it so that unaffected properties keep their existing `PropertyId`. Only the re-expanded properties receive newly generated `PropertyId` values.

**Key interfaces:**

- `BaseSchemaProcessor<Init, Unknown>::run()` — signature takes `bank_resolution: Option<&PropertyBankResolution>`. `bank` and `delta` extracted internally via `PropertyBankResolution::bank()` and `PropertyBankResolution::delta()`.

- `PropertyBankResolution::bank() -> &PropertyBank` and `PropertyBankResolution::delta() -> Option<&HashSet<PropertyName>>` — borrowed accessors added (commit `55e5a7be`).

- `SchemaVersion::changed_bank_references(&HashSet<PropertyName>) -> Vec<PropertyName>` — already exists in `snapshots.rs`.

- `BaseSchemaResolution` — `Stale` variant already carries `property_delta: PropertyDelta`. No changes needed.

- `PropertyMap::with_ids(&PropertyMap) -> PropertyMap` — already exists in `property.rs`.

**Out of scope:**
- `Builder::load_all` integration wiring (Phase 3)
- Legacy `schema_processor.rs` `NodeStatus::StaleBankReferences` removal (Phase 3)
- `BaseSchemaResolution::Deleted` variant (issue 06)
- Transitive schema-inheritance bank-reference propagation
- Property bank delta reconciliation with multiple schemas referencing the same bank target

**Acceptance criteria (Agent Brief):**
- [ ] `run(view=Some(ts_match), source, bank_resolution=Some(...))` where schema has referencing property → returns `Stale` with only that property in `property_delta.upserts`, unaffected properties keep IDs via `with_ids()`, file is read and parsed
- [ ] `run(view=Some(ts_match), ...)` where schema has NO referencing properties → returns `Fresh`, no file read
- [ ] `run(view=Some(ts_match), ..., bank_resolution=None)` → returns `Fresh`, no file read
- [ ] `run(view=Some(ts_match), ...)` where referencing property's bank target is missing from `PropertyBank` → full rebuild fallback (`New`), diagnostic emitted
- [ ] `run(view=Some(ts_mismatch), ..., bank_delta=Some([changed_bank_prop]))` where content differs → Analysis stage folds bank-reference re-expansions into existing `PropertyDelta.upserts`
- [ ] Re-expanded properties receive new `PropertyId` values; all other properties retain their prior IDs after `with_ids()`
- [ ] `changed_bank_references` returns empty set → no targeted re-expansion, no file read on Fresh path, no additional delta on Analysis path
- [ ] All existing `base_processor.rs` tests continue to pass unchanged
- [ ] `cargo clippy --all-targets -- -D warnings` — 0 warnings
- [ ] `cargo fmt --check` — clean

**Test matrix (unit tests in `#[cfg(test)] mod tests` in `base_processor.rs`):**

| # | Test name | Scenario |
|---|---|---|
| 1 | `stale_references::returns_stale_with_relevant_upserts_when_fresh_and_bank_changed` | Fresh timestamp + bank_delta with matched ref → targeted re-expand, Stale with property_delta |
| 2 | `stale_references::skips_file_read_when_no_referencing_properties_exist` | Fresh timestamp + bank_delta but schema has no refs → Fresh, no file read |
| 3 | `stale_references::skips_file_read_when_bank_delta_is_none` | Fresh timestamp + bank_delta=None → Fresh, no file read |
| 4 | `stale_references::skips_file_read_when_bank_delta_is_empty` | Fresh timestamp + bank_delta=Some(empty) → Fresh, no file read |
| 5 | `stale_references::preserves_unaffected_property_ids_via_with_ids` | Re-expanded props get new IDs, unaffected keep old IDs |
| 6 | `stale_references::escalates_to_full_rebuild_when_bank_target_missing_fresh_path` | Fresh path: referenced bank property missing → full rebuild fallback |
| 7 | `stale_references::analysis_path_folds_bank_delta_into_property_delta` | Timestamp mismatch + content delta + bank delta → Stale with combined deltas |
| 8 | `stale_references::analysis_path_ignores_bank_delta_when_no_refs_match` | Timestamp mismatch + content delta + bank delta that doesn't match → Stale with content-only deltas |
| 9 | `stale_references::analysis_path_escalates_to_full_rebuild_when_bank_target_missing` | **NEW (Defect 3 fix)** Analysis path: bank target missing → full rebuild fallback |

**Cross-case integration tests (in `lithos-core/tests/base_processor.rs`):**
- Cold start + property bank change → targeted re-expansion on next run
- Multiple schemas referencing same bank target → each independently expanded

**Note:** Cross-case integration tests are **not yet implemented**. These remain as open acceptance criteria.

## Implementation Log

### Current Status: In Progress — 3 commits on branch, all tests passing

All implementation work is committed on `feat/base-schema/05-stale-refs` in
`.worktrees/feat-base-schema-05-stale-refs`.

**Test suite state:** 1564 unit + 36 integration + 1 e2e = all passing, 0 failures.
Clippy clean (`cargo clippy --all-targets -- -D warnings`). Fmt clean.

### Commits

| Hash | Message |
|---|---|
| `55e5a7be` | feat(schema): add SchemaName::TryFrom<BaseName> |
| `be869054` | feat(schema): add base_processor_v2 — typestate rewrite from bank processor template |
| `51ee4641` | refactor(schema): replace base_processor with v2 typestate rewrite |

### Architectural Decisions Made

The implementation was restructured from a partial bolt-on approach to a full rewrite
using `property_bank_processor.rs` as the structural template. Key decisions:

| # | Decision | Rationale |
|---|---|---|
| D1 | `SchemaName::TryFrom<BaseName>` added to `identifier.rs` | Eliminates `schema_name_from_path` static helper; type-driven conversion |
| D2 | `bank_snapshot` removed from `Changed`; `update()` takes `bank: &PropertyBank` | No clone into state; consistent with "Shape A" threading |
| D3 | Missing path reads and parses the file | Mirrors bank processor exactly; new schemas may contain `$ref` entries |
| D4 | `sync_metadata` writes only the view | Correct: no BaseSchema changes on metadata-only updates |
| D5 | `SchemaId` resolved once in `run_present`, carried in all downstream states | Eliminates 3 redundant `find_schema_id_by_path` calls |
| D6 | `fetch()` uses carried `schema_id` from `Fresh` struct | No re-lookup needed |
| D7 | `SchemaName` derived via `TryFrom<BaseName>` at parse time | No static helpers; derivation is local to where file is read |
| D8 | `with_bank_delta_upserts` returns `Result<Self, BaseSchemaResolution>` | Enables conflict escalation without changing `Changed`'s type |
| D9 | `CorruptNew` status used for structural conflict escalation | Avoids re-attempting $ref expansion against the missing bank |

### State Carrying Plan (as implemented)

```
Present { view }
  → run_present: find_schema_id_by_path called once
  → check_timestamps(source, schema_id) →
  → Fresh { view, schema_id }
  → Suspect { view, content, schema_id }
  → Stale { content, content_hash, view, schema_id }
  → ParsedStale { raw, content_hash, view, schema_id }
  → Changed { raw, view, schema_id, content_hash, property_delta, excludes_delta, extends_delta }
  → StaleContent { view, content_hash, schema_id }
  → StaleTimestamps { view, schema_id }
```

### New Status Types

| Status | Stage | Purpose |
|---|---|---|
| `CorruptNew` | Construction | Full rebuild after view corruption or structural bank conflict; uses inline-only properties (no $ref expansion) |
| `ParsedMissing` | Parsed | Intermediate state after parsing on the missing path (transitions to `New`) |

### StaleReferences method placement

- `relevant_bank_refs(&self, bank_resolution) -> Vec<PropertyName>` — on `impl BaseSchemaProcessor<Construction, Fresh>`; returns sorted vec for determinism
- `re_expand_bank_references(self, source, repository, bank, relevant_refs: &[PropertyName]) -> Result<BaseSchemaResolution, SchemaLoaderError>` — on `impl BaseSchemaProcessor<Construction, Fresh>`; annotated `#[expect(clippy::too_many_lines)]`
- `with_bank_delta_upserts(self, bank_resolution, repository) -> Result<Self, BaseSchemaResolution>` — on `impl BaseSchemaProcessor<Construction, Changed>`
- `escalate_bank_conflict_to_new(self, prop_name, repository) -> Result<Self, BaseSchemaResolution>` — on `impl BaseSchemaProcessor<Construction, Changed>`; extracted helper to reduce nesting depth

### Known Open Items / Defects in Current Implementation

The next session must review these (no changes without owner approval):

**O1 — `update()` does not take `bank: &PropertyBank` as expected by Decision D2**

The `update()` method signature in the final implementation is:
```rust
fn update<R: Repository>(self, repository: &R) -> Result<...>
```
It does NOT take `bank: &PropertyBank`. Instead it fetches the existing `BaseSchema` from
the repository and applies the `property_delta` upserts/removals to it. This works because
`property_delta` was already populated with expanded properties (by `analyze()` and
`with_bank_delta_upserts()`). However, this means `update()` re-fetches the base schema
from the repository, whereas `create()` takes `bank` explicitly. The inconsistency should
be reviewed.

**O2 — `CorruptNew` discards the `content_hash`**

`CorruptNew` was originally designed to carry `{ raw, content_hash }` but the `content_hash`
field was removed (clippy: dead_code) because `create_from_raw()` does not write a view at all
— it only saves the `BaseSchema`. This means a schema rebuilt from a corrupt view path does NOT
get a `RawSchemaView` written to the repository. On the next pipeline run, the view will be
missing (`view = None`), which correctly routes to the missing path and rebuilds. Whether this
is correct behaviour or a bug needs review.

**O3 — Missing cross-case integration tests**

The Agent Brief specifies two cross-case integration tests in `lithos-core/tests/base_processor.rs`:
- Cold start + property bank change → targeted re-expansion on next run
- Multiple schemas referencing same bank target → each independently expanded

These have not been written. They require a file-backed repository rather than the in-memory test
repository used in unit tests.

**O4 — `re_expand_bank_references` annotated `#[expect(clippy::too_many_lines)]`**

The function is ~130 lines (limit 100). It is a single sequential pipeline:
fetch → read → parse → re-expand → persist. The annotation suppresses the lint.
Review whether a helper extraction would improve clarity without harming readability.

**O5 — `with_bank_delta_upserts` and `escalate_bank_conflict_to_new` annotated `#[expect(clippy::result_large_err)]`**

`BaseSchemaResolution::Stale` is the large variant (carries `BaseSchema` + 3 deltas).
The `Err` variant of `Result<Self, BaseSchemaResolution>` is used as an early-exit
sentinel (not a true error). This is unconventional. Review whether a dedicated enum
(e.g., `BankAugmentOutcome`) would be cleaner.

**O6 — Agent Brief key interface mismatch**

The Agent Brief (pre-rewrite) described the `run()` signature as:
```
run(view, source, bank, bank_delta)
```
The implementation uses:
```
run(view, source, repository, bank_resolution: Option<&PropertyBankResolution>)
```
The `bank` and `bank_delta` are bundled in `PropertyBankResolution`. The brief was
updated during the session. Verify the acceptance criteria wording still maps correctly
to the new signature.

### Preconditions (from issue 04 completion)

- `BaseSchemaProcessor<Init, Unknown>::run(view, source, repository, bank_resolution)` exists.
- `BaseSchemaResolution::Stale { schema_id, base_schema, property_delta, excludes_delta, extends_delta }` exists.
- `SchemaVersion::changed_bank_references(&HashSet<PropertyName>) -> Vec<PropertyName>` exists.
- `PropertyMap::with_ids(&PropertyMap) -> PropertyMap` exists.
- `RefExpander::new(&PropertyBank)` exists.
- `PropertyDeltaEngine::diff_schema(expander)` works with `RefExpander`.
- `RawSchemaView::current() -> Option<&SchemaVersion>` exists.
- All typestate statuses and stages exist on `BaseSchemaProcessor`.
- `PropertyBankResolution::bank() -> &PropertyBank` and `::delta() -> Option<&HashSet<PropertyName>>` added.
