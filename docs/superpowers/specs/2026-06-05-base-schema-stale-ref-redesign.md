# Design: BaseSchemaProcessor Stale Reference Redesign

**Date:** 2026-06-05
**Status:** Draft
**Context:** `feat/base-schema/05-stale-refs`

## Goal
Correct the architectural placement of stale bank-reference handling in `BaseSchemaProcessor`. Move detection and targeted re-expansion from the `Construction` stage into the `Comparison` and `Analysis` stages, aligning with the `PropertyBankProcessor` design and ensuring `delta.rs` owns property hydration.

## Architecture

### 1. Delta Engine Extension (`delta.rs`)
The `PropertyDeltaEngine::diff_schema` method will be updated to accept an optional set of property names that MUST be included in the delta regardless of whether their raw content hashes have changed.

```rust
impl<'data> PropertyDeltaEngine<'data, RawProperty> {
    pub(crate) fn diff_schema(
        &self,
        expander: &RefExpander,
        forced_refs: &[PropertyName], // NEW
    ) -> Result<PropertyDelta, SchemaLoaderError>
}
```

**Logic:**
1. Compute the raw change set via hashes (standard behavior).
2. For each name in `forced_refs`, if it is not already in the raw upsert map, fetch its raw entry from the schema and add it to the upsert map.
3. Perform hydration (ref expansion and inline conversion) as today.
4. Conflict detection: If a `forced_ref` target is missing from the bank, `expander.expand_property` returns `Err(SchemaLoaderError::Resolution)`. This is a hard error, not a fallback signal.

### 2. Typestate Redesign (`base_processor.rs`)

#### New Statuses
- `StaleReferences { content_string, content_hash, view, schema_id, ref_delta: Vec<PropertyName> }`
- `ParsedStaleReferences { raw, content_hash, view, schema_id, ref_delta: Vec<PropertyName> }`

#### Flow Logic
Detection moves early into the pipeline:

**Path A: Timestamp Match**
1. `check_timestamps` matches.
2. If `bank_resolution` has a non-empty delta:
   - Call `view.current().changed_bank_references(delta)`.
   - If non-empty (`ref_delta`):
     - Read file content.
     - Transition to `StaleReferences`.
     - Route to `ParsedStaleReferences` -> `Analysis` -> `Delta`.
   - Else: Transition to `Fresh`.
3. Else: Transition to `Fresh`.

**Path B: Content Hash Match**
1. `check_content` matches.
2. If `bank_resolution` has a non-empty delta:
   - Call `view.current().changed_bank_references(delta)`.
   - If non-empty (`ref_delta`):
     - Transition to `StaleReferences` (content already read).
     - Route to `ParsedStaleReferences` -> `Analysis` -> `Delta`.
   - Else: `sync_metadata` -> `Fresh`.
3. Else: `sync_metadata` -> `Fresh`.

**Path C: Content Hash Mismatch**
1. Normal stale path logic runs.
2. `ParsedStale::analyze(bank_resolution)`:
   - Extracts `ref_delta` from `bank_resolution`.
   - Calls `diff_schema(expander, &ref_delta)`.
   - If raw delta AND `ref_delta` are empty -> `Empty`.
   - Else -> `Delta`.

### 3. ID Stability
Align with `PropertyBankProcessor`: all upserts (including re-expanded refs) preserve their existing `PropertyId` if a property with the same name exists in the repository.

```rust
// base_processor.rs: update()
let upserts = status.property_delta.upserts().clone().with_ids(&existing_properties);
```

### 4. Removal of CorruptNew
The `CorruptNew` status and `escalate_bank_conflict_to_new` helper will be removed.
- If a view is corrupt (missing current version), route to `AnalysisBranch::Corrupt(BaseSchemaProcessor<Construction, New>)`.
- `New` path uses the standard `create` logic (ref expansion against current bank).
- If `create` fails due to a missing bank target, it returns a `SchemaLoaderError`.

## Data Flow

```text
Present
  ├─ Timestamps match
  │    ├─ Bank delta exists AND references match → [StaleReferences]
  │    └─ No bank changes or no matches → [Fresh]
  └─ Timestamps mismatch
       ├─ Content hash matches
       │    ├─ Bank delta exists AND references match → [StaleReferences]
       │    └─ No bank changes or no matches → [Refresh] → [Fresh]
       └─ Content hash mismatch → [ParsedStale] → [Analysis]
```

## Testing Strategy
- **Unit**: Mock `RefExpander` results and verify `PropertyDelta` contents in `delta.rs`.
- **Typestate**: verify `run()` results for:
  - Fresh timestamp + bank change -> `Stale` resolution.
  - Stale timestamp + content match + bank change -> `Stale` resolution.
  - Stale timestamp + content mismatch + bank change -> `Stale` resolution with combined deltas.
- **Integration**:
  - `lithos-core/tests/base_processor_integration.rs` (new file).
  - Test 1: Cold start -> bank change -> targeted update.
  - Test 2: Two schemas -> same bank target change -> both update.

## Owner Approval Required
- Redesign of `diff_schema` signature.
- Removal of `CorruptNew` in favor of standard `New` resolution.
- Preservation of IDs for re-expanded properties.
