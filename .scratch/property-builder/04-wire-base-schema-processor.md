---
labels: [ready-for-agent]
---

## What to build

Update `BaseSchemaProcessor` to use `PropertyDiffer` and `PropertyMapBuilder` in place of `PropertyDeltaEngine::diff_schema` and the direct `RefExpander` calls. The orchestration layer should no longer contain any domain property construction logic — it calls the differ for raw change sets and hands those directly to the builder.

The external interface and behaviour of `BaseSchemaProcessor` must remain identical. All existing tests must pass without modification.

## Acceptance criteria

- [ ] `BaseSchemaProcessor` no longer imports `PropertyDeltaEngine` or `RefExpander`
- [ ] `BaseSchemaProcessor` uses `PropertyDiffer` to compute raw change sets
- [ ] `BaseSchemaProcessor` uses `PropertyMapBuilder` (`build`, `update`, `update_refs`, `remove`) to produce `PropertyMap` values
- [ ] Forced-update logic (properties that must be re-resolved when the `PropertyBank` changes) is handled by the orchestrator unioning forced names into the raw upserts before calling `PropertyMapBuilder::update_refs`
- [ ] All existing `BaseSchemaProcessor` tests pass without modification
- [ ] `cargo test` passes with no regressions

## Blocked by

- `02-implement-property-differ.md`
- `03-implement-property-builders.md`
