# BaseSchemaProcessor StaleReferences Redesign

**Date:** 2026-06-05
**Branch target:** `feat/base-schema/05-stale-refs`
**Replaces:** current implementation of `StaleReferences` handling in `base_processor.rs`

---

## Problem Statement

The current implementation places stale bank-reference handling inside
`Construction/Fresh`, which is architecturally wrong: `Fresh` should mean the
cached schema can be returned unchanged. Bank-reference re-expansion is also
performed directly in the processor, duplicating responsibility that belongs
in `delta.rs`. The analysis path bypasses bank-reference checks for the
empty-delta and stale-timestamp branches. `CorruptNew` is an unnecessary
second construction path; corrupt views should route to the normal `New`
rebuild as `PropertyBankProcessor` does.

---

## Goals

1. Move bank-reference detection to the comparison/analysis routing layer,
   before any `Fresh` or `StaleTimestamps` resolution is emitted.
2. Centralise all property expansion in `delta.rs` via a single
   `diff_schema` extension.
3. Introduce a minimal `StaleReferences` status and typed pipeline path so
   the typestate encodes "file has been read, bank refs are known stale."
4. Remove `CorruptNew` and align corrupt-view handling with
   `PropertyBankProcessor`.
5. Preserve `PropertyId` for same-name upserts, consistent with
   `PropertyBankProcessor::update`.

---

## Architectural Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| A1 | `diff_schema` gains a `forced_refs: &[PropertyName]` parameter | Keeps all expansion logic in `delta.rs`; backward-compatible (empty slice = today's behaviour) |
| A2 | Bank-reference check happens in comparison stage, not construction | `Fresh` and `StaleTimestamps` are only emitted when the schema and its bank references are both unchanged |
| A3 | New `StaleReferences` status carries `content_string`, `content_hash`, `view`, `schema_id`, `ref_delta` | File is read before this status is constructed; `ref_delta` is a `Vec<PropertyName>` from `changed_bank_references()` |
| A4 | New `ParsedStaleReferences` status carries `raw`, `content_hash`, `view`, `schema_id`, `ref_delta` | Intermediate state after parsing on the stale-references path |
| A5 | `impl<Analysis, ParsedStaleReferences>` produces only `PropertyDelta` from `ref_delta`, empty `ExcludesDelta` and `ExtendsDelta`, transitions directly to `Changed` | Excludes/extends did not change; only bank-referenced property content changed |
| A6 | Remove `CorruptNew`; corrupt view routes to `AnalysisBranch::Corrupt(New)` as in `PropertyBankProcessor` | A single `New` construction path reduces surface area |
| A7 | `PropertyId` preserved for same-name upserts via `with_ids()` in `update()`, matching `PropertyBankProcessor` | Consistent identity semantics across both processors |
| A8 | Missing bank target on `StaleReferences` path returns `SchemaLoaderError::Resolution`, not a hidden `New` fallback | Makes the failure visible; the caller can decide how to recover |

---

## Changes by File

### `lithos-core/src/schema/delta.rs`

**Modify** `PropertyDeltaEngine::diff_schema`:

```rust
pub(crate) fn diff_schema(
    &self,
    expander: &RefExpander,
    forced_refs: &[PropertyName],
) -> Result<PropertyDelta, SchemaLoaderError>
```

Behaviour change: after `compute_change_set()` identifies raw-hash-changed
entries, append any entry whose name is in `forced_refs` and whose raw
entry is a `RawProperty::Ref`, if not already present in the upsert set.
Expansion proceeds identically for both normal and forced upserts.

All existing call sites pass `&[]` for `forced_refs`; no behavioural change
for those paths.

---

### `lithos-core/src/schema/base_processor.rs`

#### New statuses

```rust
/// Proven: file was read and bank references are stale; carries pre-computed
/// ref names that require forced re-expansion.
struct StaleReferences {
    content_string: String,
    content_hash:   Blake3Hash,
    view:           RawSchemaView,
    schema_id:      SchemaId,
    ref_delta:      Vec<PropertyName>,
}

/// Proven: stale-references content parsed into a raw schema.
struct ParsedStaleReferences {
    raw:          RawSchema,
    content_hash: Blake3Hash,
    view:         RawSchemaView,
    schema_id:    SchemaId,
    ref_delta:    Vec<PropertyName>,
}
```

#### New `impl` blocks

```rust
impl BaseSchemaProcessor<Parsed, StaleReferences> {
    fn parse(self) -> Result<
        BaseSchemaProcessor<Analysis, ParsedStaleReferences>,
        SchemaLoaderError,
    >
}

impl BaseSchemaProcessor<Analysis, ParsedStaleReferences> {
    fn analyze(
        self,
        bank: &PropertyBank,
    ) -> Result<BaseSchemaProcessor<Construction, Changed>, SchemaLoaderError>
    // calls diff_schema(expander, &self.status.ref_delta)
    // sets empty ExcludesDelta and ExtendsDelta
    // transitions to Construction/Changed
}
```

#### New internal helper: `check_bank_references`

Private function that receives a `RawSchemaView` reference, a
`PropertyBankResolution` reference, and returns `Vec<PropertyName>`
(possibly empty). Calls `view.current().changed_bank_references(delta)`.
Used in both timestamp-match and content-match paths.

#### Modified routing — timestamp match path

```text
Present -> check_timestamps -> Match(Fresh { view, schema_id })
  -> check_bank_references(view, bank_resolution)
  -> if non-empty:
       read content
       construct StaleReferences { content_string, content_hash, view, schema_id, ref_delta }
       -> parse -> ParsedStaleReferences
       -> analyze(bank) -> Changed
       -> update -> Completed
  -> if empty:
       Fresh -> fetch -> Completed
```

#### Modified routing — content match path (stale timestamps)

```text
Suspect -> check_content -> Match(StaleTimestamps { view, schema_id })
  -> check_bank_references(view, bank_resolution)
  -> if non-empty:
       content_hash = Blake3Hash::compute(suspect.content.as_bytes())
       // content was already read in check_timestamps and is in Suspect; no second read
       construct StaleReferences {
           content_string: suspect.content,
           content_hash,
           view,
           schema_id,
           ref_delta,
       }
       -> parse -> ParsedStaleReferences
       -> analyze(bank) -> Changed
       -> update -> Completed
  -> if empty:
       sync_metadata -> Fresh -> fetch -> Completed
```

Note: `Suspect` already carries the content string read during
`check_timestamps`. `check_content` computes the hash; that hash must be
forwarded into `StaleReferences` so the new `SchemaVersion` recorded by
`update()` reflects the actual file content hash.

#### Modified routing — content mismatch path

```text
Suspect -> check_content -> Mismatch(Stale)
  -> parse -> ParsedStale
  -> analyze(bank, bank_resolution)
       -> get ref_delta from view.current().changed_bank_references(bank_delta)
       -> call diff_schema(expander, &ref_delta)   // forced_refs may be empty
       -> compute ExcludesDelta, ExtendsDelta as today
       -> AnalysisBranch::Empty | Delta | Corrupt(New)
```

`analyze()` on `ParsedStale` gains `bank_resolution: Option<&PropertyBankResolution>`
to extract `ref_delta`. If `bank_resolution` is `None` or `delta()` is `None`
or empty, `forced_refs` is `&[]` and behaviour is unchanged.

#### `CorruptNew` removal

Remove `struct CorruptNew` and `impl BaseSchemaProcessor<Construction, CorruptNew>`.

`AnalysisBranch::Corrupt` carries `BaseSchemaProcessor<Construction, New>`,
as in `PropertyBankProcessor`. Construction via the corrupt path calls the
existing `create(repository, bank)` method, which expands refs against the
current bank. A missing bank target on this path returns
`SchemaLoaderError::Resolution`.

#### `update()` — ID preservation

`update()` already calls `with_ids()` on the upsert map. No change needed
here; A7 is already correct in the existing `update()` implementation.

#### Removal of stale helpers

Remove:
- `relevant_bank_refs` from `impl<Construction, Fresh>`
- `re_expand_bank_references` from `impl<Construction, Fresh>`
- `with_bank_delta_upserts` from `impl<Construction, Changed>`
- `escalate_bank_conflict_to_new` from `impl<Construction, Changed>`

---

## Full Typestate Map (after redesign)

```
Init/Unknown
  -> run(view=None)
       Parsed/Missing -> parse -> Construction/New -> create -> Completed/NewReady

  -> run(view=Some)
       Comparison/Present
         -> check_timestamps
              Match -> check_bank_references
                         refs non-empty:
                           read → Parsed/StaleReferences
                             -> parse → Analysis/ParsedStaleReferences
                             -> analyze → Construction/Changed
                             -> update → Completed/StaleReady
                         refs empty:
                           Construction/Fresh -> fetch → Completed/FreshReady

              Mismatch → Comparison/Suspect
                -> check_content
                     Match → check_bank_references
                               refs non-empty:
                                 Parsed/StaleReferences (content from Suspect)
                                   -> parse → Analysis/ParsedStaleReferences
                                   -> analyze → Construction/Changed
                                   -> update → Completed/StaleReady
                               refs empty:
                                 Refresh/StaleTimestamps
                                   -> sync_metadata
                                   -> Construction/Fresh -> fetch → Completed/FreshReady

                     Mismatch → Parsed/Stale
                       -> parse → Analysis/ParsedStale
                       -> analyze(bank, bank_resolution)
                            Empty  → Refresh/StaleContent
                                     -> sync_metadata
                                     -> Construction/Fresh -> fetch → Completed/FreshReady
                            Delta  → Construction/Changed -> update → Completed/StaleReady
                            Corrupt → Construction/New -> create → Completed/NewReady
```

---

## Test Coverage Required

### Unit tests (in `base_processor.rs`)

| # | Test name | What it proves |
|---|-----------|----------------|
| 1 | `stale_references::returns_stale_with_upserts_on_timestamp_match_with_bank_delta` | Fresh timestamp + bank delta with matched ref → `Stale` with `property_delta` containing re-expanded prop |
| 2 | `stale_references::returns_fresh_on_timestamp_match_with_no_matching_refs` | Fresh timestamp + bank delta but schema has no refs → `Fresh`, no file read |
| 3 | `stale_references::returns_fresh_on_timestamp_match_with_none_delta` | `bank_resolution=None` → `Fresh`, no file read |
| 4 | `stale_references::returns_fresh_on_timestamp_match_with_empty_delta` | `bank_delta=Some([])` → `Fresh`, no file read |
| 5 | `stale_references::returns_stale_on_content_match_with_bank_delta` | Stale timestamps, matching content + bank delta → `Stale`, no second file read |
| 6 | `stale_references::returns_fresh_on_content_match_with_no_matching_refs` | Stale timestamps, matching content, no matching refs → `Fresh` |
| 7 | `stale_references::preserves_unaffected_property_ids` | `inline_prop` ID unchanged; `ref_prop` ID preserved via `with_ids()` |
| 8 | `stale_references::re_expanded_prop_id_equals_existing_id` | Same-name re-expanded property keeps its prior `PropertyId` |
| 9 | `stale_references::analysis_path_includes_forced_refs_in_property_delta` | Content mismatch + bank delta → `Stale` with both content and bank upserts |
| 10 | `stale_references::analysis_path_ignores_bank_delta_when_no_refs_match` | Content mismatch + bank delta that doesn't match schema refs → delta contains only content upserts |
| 11 | `stale_references::missing_bank_target_returns_error` | Bank target absent after delta → `SchemaLoaderError::Resolution` |

Tests 2, 3, 4 must prove no file read by using a write-protected or deleted
source file after the view is constructed, so a file read would fail.

### Integration tests (`lithos-core/tests/base_processor.rs`)

| # | Scenario |
|---|----------|
| I1 | Cold start then bank change: first run returns `New`; second run with bank delta returns `Stale` with re-expanded ref |
| I2 | Two schemas reference same bank target: both independently return `Stale` with the re-expanded property |

---

## Out of Scope

- `Builder::load_all` wiring (Phase 3)
- Legacy `schema_processor.rs` `NodeStatus::StaleBankReferences` removal (Phase 3)
- `BaseSchemaResolution::Deleted` variant (issue 06)
- Transitive inheritance propagation of bank-reference changes
