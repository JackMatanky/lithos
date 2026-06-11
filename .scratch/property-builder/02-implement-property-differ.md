---
labels: [ready-for-agent]
---

## What to build

Implement `PropertyDiffer` in `schema/property/diff.rs` as a pure hash-based differ. It replaces the `compute_change_set` logic currently buried inside `PropertyDeltaEngine` and removes all domain resolution from the diffing layer.

`PropertyDiffer` accepts a `RawPropertyMap<T>` and a `RawPropertyHashIndex`, computes per-entry hashes, and returns the raw change set: a map of upserted entries, a sorted vec of removed names, and the updated hash index. It must have no knowledge of `PropertyBank`, `RefExpander`, or any domain construction logic.

## Acceptance criteria

- [ ] `schema/property/diff.rs` exists and is part of the `schema::property` module
- [ ] `PropertyDiffer` accepts `RawPropertyMap<T>` + `RawPropertyHashIndex` and returns `(HashMap<PropertyName, T>, Vec<PropertyName>, RawPropertyHashIndex)`
- [ ] Returned removals are sorted deterministically
- [ ] `PropertyDiffer` has zero imports of `PropertyBank`, `RefExpander`, or any domain builder
- [ ] Unit tests cover: changed entry detected via hash mismatch, removed entry detected when key disappears, unchanged entry ignored when hash matches, empty map against non-empty index produces only removals
- [ ] All tests ported from the `engine` test module in `delta.rs` that relate to `compute_change_set`
- [ ] `cargo test` passes with no regressions

## Blocked by

- `01-restructure-property-module.md`
