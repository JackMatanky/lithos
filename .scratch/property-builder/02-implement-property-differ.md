---
title: 02-implement-property-differ
category: enhancement
label: ready-for-agent
status: pending
branch: issue-02-implement-property-differ
merge_commit:
date_created: 2026-06-20
date_completed:
---

## What to build

Implement `PropertyDiffer` in `crates/schema/src/property/diff.rs` as a pure hash-based differ. It replaces the `compute_change_set` logic currently buried inside `PropertyDeltaEngine` and removes all domain resolution from the diffing layer.

`PropertyDiffer` accepts a `RawPropertyMap<T>` and a `RawPropertyHashIndex`, computes per-entry hashes, and returns the raw change set: a map of upserted entries, a sorted vec of removed names, and the updated hash index. It must have no knowledge of `PropertyBank`, `RefExpander`, or any domain construction logic.

## Imports and types

```rust
use serde::Serialize;
use std::fmt::Debug;
use crate::{
    property::PropertyName,
    raw::property::RawPropertyMap,
    views::RawPropertyHashIndex,
};
```

## Interface

```rust
pub fn diff<T>(
    properties: &RawPropertyMap<T>,
    previous_hashes: &RawPropertyHashIndex,
) -> (RawPropertyMap<T>, Vec<PropertyName>, RawPropertyHashIndex)
where
    T: Clone + Serialize + Debug,
```

The upsert map is returned as `RawPropertyMap<T>` (not a bare `HashMap`) so consumers receive validated `PropertyName` keys directly compatible with `PropertyMapBuilder` methods. The orchestrator can convert via `.into_map()` or use `RawPropertyMap::<RawProperty>::split_entries()`. When `T = RawProperty`, callers can partition inline vs ref entries using the existing `split_entries()` method.

## Acceptance criteria

- [ ] `crates/schema/src/property/diff.rs` exists and is part of the `property` module
- [ ] `PropertyDiffer` is a free function or stateless struct with a `diff<T>()` method accepting `RawPropertyMap<T>` + `RawPropertyHashIndex`
- [ ] Returns `(RawPropertyMap<T>, Vec<PropertyName>, RawPropertyHashIndex)` where upserts use the validated map wrapper
- [ ] Returned removals are sorted deterministically
- [ ] `PropertyDiffer` has zero imports of `PropertyBank`, `RefExpander`, or any domain builder
- [ ] Unit tests cover: changed entry detected via hash mismatch, removed entry detected when key disappears, unchanged entry ignored when hash matches, empty map against non-empty index produces only removals
- [ ] All tests ported from the `engine` test module in `crates/schema/src/delta.rs` that relate to `compute_change_set`
- [ ] `cargo test` passes with no regressions

## Blocked by

- `01-restructure-property-module.md`
