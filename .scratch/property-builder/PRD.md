---
labels: [ready-for-agent]
---

# PRD: Schema Diffing and Resolution Refactor

## Problem Statement

Currently, in the schema module, property construction, resolution, and updating logic is scattered across multiple locations, including `TryFrom` trait implementations, `RefExpander`, and `PropertyDeltaEngine`. This scatter creates friction when building schemas, updates, and handling dependencies like the `PropertyBank`. Specifically, the `PropertyDeltaEngine` combines pure raw-property hash diffing with the complex domain logic of resolving property references and expanding defaults. This prevents the domain construction logic from being isolated and deep, making it hard to test, reason about, and orchestrate within the `BaseSchemaProcessor` and `PropertyBankProcessor`.

## Solution

Isolate the schema difference computation from the schema resolution computation. We will create dedicated, deep modules for building properties (`PropertyBuilder` and `PropertyMapBuilder`) and a pure computation module for difference computation (`PropertyDiffer`). The differ will only concern itself with finding the differences between raw maps via hash indexes. The builders will take exclusive ownership of generating and updating domain-valid `PropertyMap` values, correctly handling `PropertyBank` references, inline validations, and identity preservation.

## User Stories

1. As a developer, I want all domain property instantiation logic isolated in a `PropertyBuilder` so that I can validate and verify property construction (both inline and references) in a single place.
2. As a developer, I want a `PropertyMapBuilder` to orchestrate property maps from raw inputs so that I no longer rely on shallow, implicit `TryFrom` traits.
3. As a developer, I want the `PropertyMapBuilder` to require a `PropertyBank` via a `with_bank` method, so that dependency requirements are explicit when resolving `$ref` elements.
4. As a developer, I want the `PropertyMapBuilder` to handle updating an existing map with raw deltas and preserving property IDs, so that the orchestration layer does not manually handle ID injection.
5. As a developer, I want `PropertyDeltaEngine` simplified to a `PropertyDiffer` that only computes raw hash-based differences, so that diffing is a pure function isolated from domain resolution.
6. As a developer, I want `BaseSchemaProcessor` to use the `PropertyDiffer` and `PropertyMapBuilder` correctly, so that the orchestration layer logic is heavily simplified and free of domain-model instantiation specifics.
7. As a developer, I want `PropertyBankProcessor` to use the `PropertyDiffer` and `PropertyMapBuilder` correctly, so that bank generation matches the schema generation standards.
8. As a developer, I want all existing tests for `BaseSchemaProcessor` and `PropertyBankProcessor` to continue passing without modifications to the external interface, guaranteeing backwards compatibility.
9. As a developer, I want exhaustive tests for the `PropertyBuilder`, `PropertyMapBuilder`, and `PropertyDiffer`, so that I have strong confidence in the correctness of these new deep modules.

## Implementation Decisions

### Restructuring and Module Placement
- Rename the `schema/property_spec/` directory (if it exists as a directory) or conceptually group specs into `schema/property/`.
- Move `PropertyDiffer` to a new file: `schema/property/diff.rs`.
- Move `PropertyBuilder` and `PropertyMapBuilder` to a new file: `schema/property/builder.rs`.

### Modified and New Interfaces

**`PropertyDiffer` (replaces `PropertyDeltaEngine` in `diff.rs`)**
- Responsibility: Pure hash-based comparison. Takes a `RawPropertyMap` and a `RawPropertyHashIndex`.
- Returns: Raw differences (`RawPropertyMap` or raw upserts/removals) instead of a fully resolved domain `PropertyDelta`.

**`PropertyBuilder` (absorbs `expander.rs`, lives in `builder.rs`)**
- Responsibility: Single property lifecycle.
- Interface: Takes a `RawPropertyInline` or `RawPropertyRef` (plus an optional `PropertyBank`) and returns a fully validated domain `Property` with a new ID.

**`PropertyMapBuilder` (replaces `TryFrom` in `property.rs`, lives in `builder.rs`)**
- Responsibility: Map lifecycle and ID preservation.
- Interface:
  - `pub fn new() -> Self`
  - `pub fn with_bank(bank: &'bank PropertyBank) -> Self`
  - `pub fn build(self, raw: RawPropertyMap) -> Result<PropertyMap, SchemaError>`
  - `pub fn update(self, existing: PropertyMap, raw_upserts: HashMap<PropertyName, RawProperty>, removals: &[PropertyName]) -> Result<PropertyMap, SchemaError>` (or a raw delta wrapper).

### Code Deletions
- Delete the `TryFrom<RawPropertyMap>` trait implementations in `property.rs`.
- Delete `expander.rs` entirely (absorbed into `PropertyBuilder`).

## Testing Decisions

### Test Strategy
A good test validates external behavior and output determinism without coupling to the implementation steps. Tests should cover edge cases (e.g., missing `$ref` targets, invalid formats) to ensure the seams correctly enforce domain rules.

### Modules to Test
- **`schema/property/diff.rs`**: Unit test `PropertyDiffer` to verify it correctly identifies added, removed, and updated fields via hash comparisons without any side effects or resolution logic.
- **`schema/property/builder.rs`**: Unit test `PropertyBuilder` for inline validation, `$ref` expansion, missing bank targets, type mismatches. Unit test `PropertyMapBuilder` for collection creation, successful ID preservation on updates, and delta applications.
- **Integration/Regression**: Verify `BaseSchemaProcessor` and `PropertyBankProcessor` continue to pass all existing tests, proving the seam replacement is behavior-preserving.

### Prior Art
- Check existing tests in `expander.rs` and `delta.rs` for property resolution scenarios. They will be ported/adapted to test the new builder modules.
- Check existing `TryFrom` property map tests to adapt them into builder `build()` tests.

## Out of Scope

- Changing how `RawSchema` is parsed from the filesystem.
- Changing the schema aggregation phase (cross-schema inheritance) outside of the base processing phase.
- Changing the JSON representation or serialization formats of the properties.

## Further Notes

- ADR 023 has been accepted and outlines the architectural justification for these changes. Ensure the implementation strictly aligns with the "no domain resolution during diffing" constraint.
