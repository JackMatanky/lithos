---
name: separate-schema-diffing-from-resolution
status: accepted
date_proposed: 2026-06-11
date_decided: 2026-06-11
date_implemented:
stakeholders: [Engineering]
---

# ADR 023: Separate Schema Diffing from Resolution

## Context

During the refactoring of the `schema/` module (specifically around `BaseSchemaProcessor` and `PropertyBankProcessor`), we identified significant friction in how properties were being updated.

Currently, the `PropertyDeltaEngine` (which diffs raw property maps against a cached hash index) also handles the expansion of `$ref` pointers (via `RefExpander`). This forces the `PropertyDeltaEngine` to produce a `PropertyDelta` containing fully resolved, domain-validated `PropertyMap` values.

This design couples raw hash diffing with domain resolution. It also forces the `PropertyDeltaEngine` to require a `PropertyBank` reference. Meanwhile, `BaseSchemaProcessor` has to take the fully resolved delta and manually splice it into the existing `PropertyMap` to preserve identity (IDs). Furthermore, the conversion from raw properties to domain properties is scattered across various `TryFrom` trait implementations.

## Decision

We will strictly separate the concern of "diffing" from the concern of "resolution and building".

Specifically:
1. **Shrink `PropertyDeltaEngine` to a pure `PropertyDiffer`.** It will only compare `RawPropertyMap` against `RawPropertyHashIndex` and return raw differences. It will no longer use `RefExpander` to resolve `$ref` pointers.
2. **Introduce `PropertyMapBuilder`.** This will act as the single source of truth for constructing a `PropertyMap`. It will provide two distinct interfaces: one for building a map from scratch (`build()`), and one for applying a raw delta to an existing map while preserving IDs (`update()`).
3. **Introduce `PropertyBuilder` inside `PropertyMapBuilder`.** This builder will absorb `RefExpander` and handle the resolution of individual `RawPropertyInline` and `RawPropertyRef` values into domain `Property` values.
4. **Remove the `TryFrom<RawPropertyMap>` trait implementations.** All domain map construction must go through `PropertyMapBuilder`.

## Alternatives Considered

### Alternative 1: Add `with_bank()` to `PropertyDeltaEngine`
- **Description**: Provide the engine with a `PropertyBank` reference so it can continue to produce a fully resolved `PropertyDelta` using the new `PropertyBuilder`.
- **Pros**: Minimal disruption to `BaseSchemaProcessor::update()`, which already expects a resolved delta.
- **Cons**: We rejected this because it leaks domain resolution logic (how to turn a raw property into a domain property, how to handle the bank) into the diffing layer. It splits the responsibility for domain construction between `PropertyMapBuilder` (for new schemas) and `PropertyDeltaEngine` (for updates).

## Technical Validation

The application of the **deletion test** demonstrated that removing `RefExpander` and absorbing it into `PropertyBuilder` concentrates the complexity of validation and default fallbacks into a single place. Separating diffing from resolution follows the principle of high locality: the differ only knows about hashes, and the builder only knows about validation and IDs.

## Consequences

- **Positive**: High locality. All logic turning a raw shape into a domain `Property` is behind one seam (`PropertyMapBuilder`). Diffing is pure and easy to test.
- **Positive**: `BaseSchemaProcessor` becomes simpler, as it delegates the complex merge-and-preserve-ID logic to the builder.
- **Negative**: The orchestrator (`BaseSchemaProcessor`) must now juggle raw differences returned by the differ and feed them to the builder, rather than just receiving a ready-to-use domain delta.
- **Risks**: We must ensure that the raw delta retains enough semantic meaning (especially around forced updates when bank targets are missing) to accurately drive the builder's update logic.

## References
- `schema/CONTEXT.md`
- `docs/refs/rust/guides/hexagonal_architecture.md`
