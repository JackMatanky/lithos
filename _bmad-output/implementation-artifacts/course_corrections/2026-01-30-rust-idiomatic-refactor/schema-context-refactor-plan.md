# Schema Context Refactor Plan

Date: 2026-01-30
Owner: Jack
Scope: lithos-core/src/schema/

Goal
- Align schema context with updated design docs and idiomatic Rust patterns.
- Introduce typed IDs/newtypes, replace boolean flags with semantic enums, and
  normalize events/ports/query/command interfaces.
- Preserve raw input DTOs as primitives (Uuid + bool flags) and map at the
  resolver boundary.

Alignment Targets
- docs/design/008-schema-models.md (domain models, newtypes, metadata)
- docs/design/009-schema-cqrs.md (CQRS ports, storage/index strategy, metadata)
- docs/design/010-schema-graph-resolver.md (resolver + graph unification, staleness)
- docs/design/011-property-spec.md (raw spec shape, validation behavior, modularization)

Alignment Checklist (must be true at completion)
- Domain models match 008: newtypes, private fields, semantic enums, metadata types.
- CQRS surface matches 009: UUID-first storage, name index, metadata tier, errors split.
- Resolver matches 010: unified SchemaResolver, deterministic ordering, typed refs.
- PropertySpec matches 011: raw spec type naming, modularization, validation rules.

Constraints
- Context isolation: schema must not import note/template domains.
- Port-based CQRS with split storage ports.
- Type-driven design: private fields, validated constructors, newtypes.
- Zero-copy patterns with rkyv where applicable.
- Test-first (red-green-refactor).
- ADR required if any architectural decision is introduced.
- Run pre-commit hooks per phase until all hooks pass.

Phases

Phase 1: Baseline test coverage (StringSpec UTF-8 bytes)
- Add tests to lock in UTF-8 byte-length semantics for StringSpec.
- Target file: lithos-core/src/schema/property_spec.rs
- Exit criteria: tests cover multibyte example (e.g., "caf\u{00e9}").

Phase 2: Core newtypes + semantic enums + schema module updates
- Add newtypes: SchemaId, PropertyId, BankVersion, SchemaHash, Timestamp.
- Add enums: Cardinality, Multiplicity.
- Update Property, PropertyBank, Schema to use newtypes/enums.
- Update resolver/graph/command/query/ports/events for typed IDs/names.
- Update docs/tests/fixtures to use newtypes and as_str().
- Exit criteria: no raw Uuid/required/array usage in domain types; raw layer
  stays primitive; schema modules compile; hooks pass.

Phase 3: RawPropertySpec rename/move
- Rename RawPropertySpec to PropertySpecDef (or equivalent per design spec).
- Move raw spec definitions to schema/property_spec module layout.
- Update raw schema DTOs to reference the renamed type.
- Update resolver adapters and tests accordingly.
- Exit criteria: compilation success; raw DTOs unchanged (primitives retained).

Phase 4: PropertySpec modularization
- Split PropertySpec definitions into focused submodules.
- Ensure public API is stable and re-exported from property_spec mod.
- Move tests alongside submodules or keep consolidated if preferred by style.
- Exit criteria: no regressions; consistent module organization.

Phase 5: Storage boundary alignment
- Ensure ports traits remain CQRS-split with typed IDs/names.
- Align schema storage keys and query patterns for newtypes.
- Update any serialization helpers for newtypes (if required).
- Exit criteria: storage adapters compile and pass tests.

Phase 6: Error and validation refinements
- Replace generic errors with structured variants where needed.
- Ensure validation errors include context for newtypes.
- Document panics/errors in public APIs.
- Exit criteria: clippy missing-docs clean; error types are explicit.

Phase 7: Event model stabilization
- Confirm event payloads reflect newtypes and timestamps.
- Add/adjust event tests or docs.
- Ensure pending event emission remains consistent.
- Exit criteria: event consumers compile; doc examples pass.

Phase 8: CLI / integration alignment (if applicable)
- Update CLI commands and parsing to use SchemaName/PropertyName as needed.
- Update any integration tests or fixtures referencing schema IDs/names.
- Exit criteria: CLI build/tests green.

Phase 9: Test cleanup and coverage checks
- Remove outdated fixtures or duplicated helpers.
- Add missing tests for newtype conversion behaviors.
- Ensure required doc tests run when doc examples changed.
- Exit criteria: coverage for critical paths is maintained or improved.

Phase 10: Documentation and ADRs
- Update schema docs and references to newtypes/enums.
- Add ADR if a nontrivial architectural choice was made.
- Exit criteria: adr:validate green; docs consistent.

Phase 11: Final verification
- Run mise run verify.
- Ensure no TODOs/debug logs remain.
- Exit criteria: verify green, code clean.

Current Status
- Phase 1: complete.
- Phase 2: complete; newtypes/enums/ports/events updated, tests/docs aligned.
- Next: Phase 3 (RawPropertySpec rename/move).

Notes
- Raw DTOs stay primitive (Uuid + bool required/array). Resolver maps them to
  Cardinality/Multiplicity.
- Use #[expect] only when necessary and with descriptive reasons.
