---
labels: [ready-for-agent]
---

## What to build

Update `BaseSchemaProcessor` to use `PropertyDiffer` and `PropertyMapBuilder` in place of `PropertyDeltaEngine::diff_schema` and the direct `RefExpander` calls. The orchestration layer should no longer contain any domain property construction logic — it calls the differ for raw change sets and hands those directly to the builder.

The external interface and behaviour of `BaseSchemaProcessor` must remain identical. All existing tests must pass without modification.

## Forced-ref orchestration

When the `PropertyBank` changes, certain `$ref` properties must be re-resolved even if the schema file itself hasn't changed. In the current code (see `analysis` stage in `base_processor.rs`), this is handled by passing `expandable_refs` (forced ref names) to `PropertyDeltaEngine::diff_schema`, which injects them into the raw upserts before expansion.

After refactoring, the orchestrator must:

1. Compute raw change set via `PropertyDiffer::diff()` → `(RawPropertyMap<RawProperty>, Vec<PropertyName>, RawPropertyHashIndex)`
2. Union forced-ref entries into the upsert map by pulling the original raw entries from `status.raw.properties()` for each forced name
3. Call `RawPropertyMap::<RawProperty>::split_entries()` to partition upserts into inline and ref sets
4. Call `PropertyMapBuilder::update()` with inline upserts and `PropertyMapBuilder::update_refs()` with ref upserts
5. Call `PropertyMapBuilder::remove()` for removals
6. Wrap the result into `PropertyDelta` for the `BaseSchemaResolution::Stale` variant:

```rust
let property_delta = PropertyDelta::new(updated_properties, removals);
```

This wrapping is necessary because `BaseSchemaResolution::Stale` carries a `PropertyDelta` (containing fully resolved `PropertyMap` upserts + removal names) for downstream consumers like `SchemaProcessor`. The builder operates on the raw/delta level, so the orchestrator constructs the `PropertyDelta` after building.

## Acceptance criteria

- [ ] `BaseSchemaProcessor` no longer imports `PropertyDeltaEngine` or `RefExpander`
- [ ] `BaseSchemaProcessor` uses `PropertyDiffer` to compute raw change sets
- [ ] `BaseSchemaProcessor` uses `PropertyMapBuilder` (`build`, `update`, `update_refs`, `remove`) to produce `PropertyMap` values
- [ ] Forced-update logic (properties that must be re-resolved when the `PropertyBank` changes) is handled by the orchestrator: union forced names' raw entries from `status.raw.properties()` into the differ's raw upserts, then partition inline vs ref via `split_entries()`, then call appropriate builder methods
- [ ] After building, the orchestrator wraps the result into `PropertyDelta::new(built_map, removals)` for the `BaseSchemaResolution::Stale` variant
- [ ] All existing `BaseSchemaProcessor` tests pass without modification
- [ ] `cargo test` passes with no regressions

## Blocked by

- `02-implement-property-differ.md`
- `03-implement-property-builders.md`
