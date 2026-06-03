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

`PropertyBankProcessor::run()` already produces `PropertyBankResolution { bank, delta: Option<HashSet<PropertyName>> }`. The Builder passes `(&bank, delta)` into `BaseSchemaProcessor::run()`. The `StaleReferences` response is an **orthogonal, targeted intervention** that occurs in two places:

**Path A — Fresh + StaleReferences (orthogonal check after timestamp match):**

```
Init/run(view=Some, source, bank, bank_delta)
  → Present → check_timestamps
    → Match → Construction/Fresh → fetch from repo
      → if bank_delta is Some and non-empty:
        → SchemaVersion::changed_bank_references(bank_delta)
        → if non-empty:
          → parse file to RawSchema (forced read, but only for ref targets)
          → re-expand only the affected properties against current PropertyBank
          → merge with existing PropertyMap via with_ids() — preserves IDs for unaffected
          → persist updated BaseSchema + new SchemaVersion
          → return Stale { property_delta: upserts=[re-expanded], ...empties... }
        → if empty:
          → return Fresh (no-op, no file read)
      → if bank_delta is None:
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

Key invariant: a bank delta **never** triggers full rebuild on its own. It only adds targeted properties to the re-expansion set. Full rebuild fallback is reserved for:
1. Parse/view corruption
2. Incoherent delta (not expected before Phase 3)
3. **Structural reference conflict** — a referenced bank property name does not exist in the current `PropertyBank`. This indicates the bank delta removed a target that a schema property needs. Escalate to full rebuild and emit a diagnostic.

**PropertyId stability:**
`PropertyMap::with_ids(existing)` already exists and is used by the property bank processor. The targeted re-expansion path must apply it so that unaffected properties keep their existing `PropertyId`. Only the re-expanded properties receive newly generated `PropertyId` values.

**Key interfaces:**

- `BaseSchemaProcessor<Init, Unknown>::run()` — signature already accepts `bank: &PropertyBank`. Add `bank_delta: Option<&HashSet<PropertyName>>` parameter. The caller (`Builder::load_all`) first runs `PropertyBankProcessor`, receives `PropertyBankResolution`, and decomposes it into `(bank, delta)`.

- `SchemaVersion::changed_bank_references(&HashSet<PropertyName>) -> Vec<PropertyName>` — already exists in `snapshots.rs`. Returns schema property names whose bank targets are in the changed set. Unguarded call (returns empty vec for schemas with no refs).

- `BaseSchemaResolution` — the `Stale` variant already carries `property_delta: PropertyDelta`. No changes needed. The targeted re-expansion path produces `Stale` with `property_delta` containing only upserts for the re-expanded properties and empty `excludes_delta`/`extends_delta`.

- `PropertyMap::with_ids(&PropertyMap) -> PropertyMap` — already exists in `property.rs`. Used to copy `PropertyId` from the existing (pre-bank-change) map for properties not in the re-expansion set.

- `RefExpander` — already exists; constructed from the current `PropertyBank`. Used to resolve `$ref` entries during re-expansion.

- `RawSchemaView::current() -> Option<&SchemaVersion>` — already exists. Used to extract the previous `bank_references` map and unchanged property IDs.

**Out of scope:**
- `Builder::load_all` integration wiring (Phase 3 — the `bank_delta` plumbing will be done then; this issue adds the processor capability only)
- Legacy `schema_processor.rs` `NodeStatus::StaleBankReferences` removal (Phase 3)
- `BaseSchemaResolution::Deleted` variant (issue 06)
- Transitive schema-inheritance bank-reference propagation (a parent schema's refs changing does not trigger child re-expansion in Phase 1)
- Property bank delta reconciliation with multiple schemas referencing the same bank target (each schema handles its own refs independently; dedup is an optimization for later)

**Acceptance criteria:**
- [ ] `run(view=Some(ts_match), source, bank, bank_delta=Some([changed_bank_prop]))` where schema has referencing property → returns `Stale` with only that property in `property_delta.upserts`, unaffected properties keep IDs via `with_ids()`, file is read and parsed
- [ ] `run(view=Some(ts_match), source, bank, bank_delta=Some([changed_bank_prop]))` where schema has NO referencing properties → returns `Fresh`, no file read
- [ ] `run(view=Some(ts_match), source, bank, bank_delta=None)` → returns `Fresh`, no file read
- [ ] `run(view=Some(ts_match), source, bank, bank_delta=Some([changed_bank_prop]))` where referencing property's bank target is missing from `PropertyBank` → full rebuild fallback (`New`), diagnostic emitted
- [ ] `run(view=Some(ts_mismatch), source, bank, bank_delta=Some([changed_bank_prop]))` where content differs → Analysis stage folds bank-reference re-expansions into existing `PropertyDelta.upserts`
- [ ] Re-expanded properties receive new `PropertyId` values; all other properties retain their prior IDs after `with_ids()`
- [ ] `changed_bank_references` returns empty set → no targeted re-expansion, no file read on Fresh path, no additional delta on Analysis path
- [ ] All existing `base_processor.rs` tests continue to pass unchanged
- [ ] `cargo clippy --all-targets -- -D warnings` — 0 warnings
- [ ] `cargo fmt --check` — clean

**GitNexus impact note — CRITICAL risk:**
`SchemaVersion::changed_bank_references` has 8 upstream dependents, 5 affected processes (`compare`, `analyze_properties`, `load_all`), and 3 affected modules (Schema, Views, Tests). The d=1 caller is `bank_changed()` in `schema_processor.rs`. The BaseSchema path introduces **behavior parity risk**: the legacy path triggers full rebuild on stale bank refs; the new path does targeted re-expansion. Both must produce semantically equivalent `PropertyMap` output for the same input delta. A regression test that runs both processors over the same delta is recommended.

**Rust implementation constraints:**
- `PropertyId` preservation via `PropertyMap::with_ids()` — reuse the existing method, do not invent a new mechanism
- `bank_delta` is `Option<&HashSet<PropertyName>>` — when `None` or empty, the stale-references check is a no-op (zero overhead per schema)
- `changed_bank_references` returns `Vec<PropertyName>` — iterate, expand each affected property against current `PropertyBank` via `RefExpander`, collect into the new `PropertyMap`
- Parse the file only when the Fresh path detects matching bank references; otherwise skip reading entirely
- Deterministic set operations: sort the affected property names before iteration for reproducible output order
- `StaleReferences` is NOT a pipeline status — it is runtime data carried through existing transitions; do not add a `StaleReferences` variant to the typestate status enum

**Test matrix (unit tests in `#[cfg(test)] mod tests` in `base_processor.rs`):**

| # | Test name | Scenario |
|---|---|---|
| 1 | `run::stale_references::returns_stale_with_relevant_upserts_when_fresh_and_bank_changed` | Fresh timestamp + bank_delta with matched ref → targeted re-expand, Stale with property_delta |
| 2 | `run::stale_references::skips_file_read_when_no_referencing_properties_exist` | Fresh timestamp + bank_delta but schema has no refs → Fresh, no file read |
| 3 | `run::stale_references::skips_file_read_when_bank_delta_is_none` | Fresh timestamp + bank_delta=None → Fresh, no file read |
| 4 | `run::stale_references::skips_file_read_when_bank_delta_is_empty` | Fresh timestamp + bank_delta=Some(empty) → Fresh, no file read |
| 5 | `run::stale_references::preserves_unaffected_property_ids_via_with_ids` | Re-expanded props get new IDs, unaffected keep old IDs |
| 6 | `run::stale_references::escalates_to_full_rebuild_when_bank_target_missing` | Referenced bank property missing from current bank → full rebuild fallback |
| 7 | `run::stale_references::analysis_path_folds_bank_delta_into_property_delta` | Timestamp mismatch + content delta + bank delta → Stale with combined deltas |
| 8 | `run::stale_references::analysis_path_ignores_bank_delta_when_no_refs_match` | Timestamp mismatch + content delta + bank delta that doesn't match → Stale with content-only deltas |

**Cross-case integration tests (in `lithos-core/tests/base_processor.rs`):**
- Cold start + property bank change → targeted re-expansion on next run
- Multiple schemas referencing same bank target → each independently expanded

## Implementation Log

### Commits

| Hash | Message |
|---|---|

### Preconditions (from issue 04 completion)

- `BaseSchemaProcessor<Init, Unknown>::run(view, source, bank, repo)` exists and accepts `&PropertyBank`.
- `BaseSchemaResolution::Stale { schema_id, base_schema, property_delta, excludes_delta, extends_delta }` exists.
- `SchemaVersion::changed_bank_references(&HashSet<PropertyName>) -> Vec<PropertyName>` exists.
- `PropertyMap::with_ids(&PropertyMap) -> PropertyMap` exists.
- `RefExpander::new(&PropertyBank)` exists.
- `PropertyDeltaEngine::diff_schema(expander)` works with `RefExpander`.
- `RawSchemaView::current() -> Option<&SchemaVersion>` exists.
- All typestate statuses: `Present`, `Suspect`, `Stale`, `ParsedStale`, `StaleTimestamps`, `StaleContent`, `New`, `Changed`, `Fresh` exist on `BaseSchemaProcessor`.
- `Comparison`, `Parsed`, `Analysis`, `Refresh`, `Construction`, `Completed` stages exist.
- `IntoBank`, `IntoBankWithChanges` / equivalent terminal methods exist.
