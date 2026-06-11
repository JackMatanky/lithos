---
labels: [ready-for-agent]
---

## What to build

Implement `PropertyBuilder` and `PropertyMapBuilder` in `schema/property/builder.rs`. These two types become the single source of truth for constructing and updating domain `Property` and `PropertyMap` values. `PropertyBuilder` absorbs `RefExpander`; `PropertyMapBuilder` replaces the `TryFrom<RawPropertyMap>` implementations.

**`PropertyBuilder`** handles the single-property lifecycle. It accepts a `RawPropertyInline` or `RawPropertyRef` (with an optional `&PropertyBank`) and returns a validated domain `Property` with a new ID. The `$ref` expansion logic currently in `RefExpander` moves here entirely.

**`PropertyMapBuilder`** handles the map lifecycle and ID preservation. Interface:

```rust
pub fn new() -> Self
pub fn with_bank(bank: &'bank PropertyBank) -> Self
pub fn build(self, raw: RawPropertyMap<RawProperty>) -> Result<PropertyMap, SchemaError>
pub fn update(self, existing: PropertyMap, raw_upserts: RawPropertyMap<RawProperty>) -> Result<PropertyMap, SchemaError>
pub fn update_refs(self, existing: PropertyMap, raw_refs: RawPropertyMap<RawPropertyRef>) -> Result<PropertyMap, SchemaError>
pub fn remove(self, existing: PropertyMap, removals: &[PropertyName]) -> PropertyMap
```

`update` and `update_refs` preserve IDs from `existing` for any property name already present. `remove` drops named entries and returns the remaining map.

## Acceptance criteria

- [ ] `schema/property/builder.rs` exists and is part of the `schema::property` module
- [ ] `PropertyBuilder::build_inline` converts `RawPropertyInline` → `Property` with a new ID
- [ ] `PropertyBuilder::build_ref` resolves `RawPropertyRef` against `PropertyBank` → `Property` preserving the bank entry's ID and applying overrides
- [ ] `PropertyBuilder::build_ref` returns `SchemaError::PropertyRef(NotFound)` when the bank target is absent
- [ ] `PropertyBuilder::build_ref` returns `SchemaError::PropertyRef(TypeMismatch)` when override fields don't match the base property type
- [ ] `PropertyMapBuilder::build` constructs a fresh `PropertyMap` from a mixed `RawPropertyMap<RawProperty>` (inline + refs)
- [ ] `PropertyMapBuilder::update` applies raw upserts and preserves IDs for names present in `existing`
- [ ] `PropertyMapBuilder::update_refs` applies ref upserts and preserves IDs for names present in `existing`
- [ ] `PropertyMapBuilder::remove` drops named entries and returns the remaining map
- [ ] `PropertyMapBuilder::with_bank` is required before any method that resolves `$ref` entries; calling a ref-resolving method without a bank returns an appropriate error
- [ ] Unit tests ported from `expander.rs` (ref resolution, type mismatches, missing bank targets, optionality/multiplicity overrides)
- [ ] Unit tests ported from `property.rs` `TryFrom` tests (bank required override, empty options, duplicate options)
- [ ] `cargo test` passes with no regressions

## Blocked by

- `01-restructure-property-module.md`
