---
labels: [ready-for-agent]
---

## What to build

Update `PropertyBankProcessor` to use `PropertyDiffer` and `PropertyMapBuilder` in place of `PropertyDeltaEngine::diff_property_bank` and direct `TryFrom` calls. Bank generation must follow the same standards as schema generation: differ for raw change sets, builder for domain construction.

The external interface and behaviour of `PropertyBankProcessor` must remain identical. All existing tests must pass without modification.

## Acceptance criteria

- [ ] `PropertyBankProcessor` no longer imports `PropertyDeltaEngine` or `RefExpander`
- [ ] `PropertyBankProcessor` uses `PropertyDiffer` to compute raw change sets
- [ ] `PropertyBankProcessor` uses `PropertyMapBuilder` (`build`, `update`, `remove`) to produce `PropertyMap` values
- [ ] All existing `PropertyBankProcessor` tests pass without modification
- [ ] `cargo test` passes with no regressions

## Blocked by

- `02-implement-property-differ.md`
- `03-implement-property-builders.md`
