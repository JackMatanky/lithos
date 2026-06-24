---
labels: [ready-for-agent]
---

## What to build

Implement `PropertyBuilder` and `PropertyMapBuilder` in `crates/schema/src/property/builder.rs`. These two types become the single source of truth for constructing and updating domain `Property` and `PropertyMap` values. `PropertyBuilder` absorbs `RefExpander`; `PropertyMapBuilder` replaces the `TryFrom<RawPropertyMap>` implementations.

**`PropertyBuilder`** handles the single-property lifecycle. It is a stateless free function (no struct needed). Interface:

> **Note**: Add a `MissingPropertyBank` variant to the existing `PropertyBuilderError` in `crates/schema/src/error.rs` (the enum already exists with variant `OverridePropertyRefSpecTypeMismatch`). Per hexagonal architecture principles, missing bank is a **construction error** (builder was misconfigured), not a property reference error. The variant is:
> ```rust
> #[error("property bank required to resolve $ref, but no bank was provided")]
> MissingPropertyBank,
> ```
>
> The `PropertyBuilder` variant on `SchemaBuilderError` and the `From<PropertyBuilderError>` impl for `SchemaError` already exist — no changes needed there.

```rust
pub fn build_inline(raw: RawPropertyInline) -> Result<Property, SchemaError>

pub fn build_ref(
    raw: RawPropertyRef,
    bank: &PropertyBank,
) -> Result<Property, SchemaError>
```

`build_inline` converts a raw inline property to a domain `Property` with a new `PropertyId`. `build_ref` resolves a `$ref` against the bank, preserves the bank entry's ID, applies overrides, and returns a validated `Property`. The bank is **required** (not optional) — callers must have a bank when calling `build_ref`.

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

The builder accepts `RawPropertyMap<T>` (the validated map wrapper) for all raw inputs, which matches the `PropertyDiffer` return type. The orchestrator converts differ output to builder input via `RawPropertyMap::from_map(...)` if needed — though when the differ returns `RawPropertyMap<T>` directly, no conversion is required.

> **Error variant**: Calling `build_ref` (via `update_refs`) without setting a bank via `with_bank` returns `SchemaError::PropertyBuilder(PropertyBuilderError::MissingPropertyBank)`.

## Acceptance criteria

- [ ] `crates/schema/src/property/builder.rs` exists and is part of the `property` module
- [ ] `crates/schema/src/error.rs` gains `PropertyBuilderError::MissingPropertyBank` variant on the existing enum
- [ ] `build_inline` converts `RawPropertyInline` → `Property` with a new ID
- [ ] `build_ref` resolves `RawPropertyRef` against `PropertyBank` → `Property` preserving the bank entry's ID and applying overrides
- [ ] `build_ref` returns `SchemaError::PropertyRef(NotFound)` when the bank target is absent
- [ ] `build_ref` returns `SchemaError::PropertyRef(TypeMismatch)` when override fields don't match the base property type
- [ ] `PropertyMapBuilder::build` constructs a fresh `PropertyMap` from a mixed `RawPropertyMap<RawProperty>` (inline + refs)
- [ ] `PropertyMapBuilder::update` applies raw upserts and preserves IDs for names present in `existing`
- [ ] `PropertyMapBuilder::update_refs` applies ref upserts and preserves IDs for names present in `existing`
- [ ] `PropertyMapBuilder::remove` drops named entries and returns the remaining map
- [ ] `PropertyMapBuilder::with_bank` is required before any method that resolves `$ref` entries; calling a ref-resolving method without a bank returns `SchemaError::PropertyBuilder(PropertyBuilderError::MissingPropertyBank)`
- [ ] Unit tests ported from `crates/schema/src/expander.rs` (ref resolution, type mismatches, missing bank targets, MissingPropertyBank error, optionality/multiplicity overrides)
- [ ] Unit tests ported from `crates/schema/src/property.rs` `TryFrom` tests (bank required override, empty options, duplicate options)
- [ ] `cargo test` passes with no regressions

## Blocked by

- `01-restructure-property-module.md`
