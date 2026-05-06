# Schema Context Refactor Plan

Date: 2026-01-30
Owner: Jack
Scope: lithos-core/src/schema/

Goal
- Align schema context with design docs and idiomatic Rust patterns.
- Enforce type-driven domain invariants while keeping raw inputs tolerant.
- Fully implement incremental resolution and CQRS metadata support.

Alignment Targets
- docs/design/008-schema-models.md (domain models, newtypes, metadata)
- docs/design/009-schema-cqrs.md (CQRS ports, storage/index strategy, metadata)
- docs/design/010-schema-graph-resolver.md (resolver + graph unification, staleness)
- docs/design/011-property-spec.md (raw spec shape, validation behavior, modularization)

Alignment Checklist (must be true at completion)
- Raw inputs are tolerant: RawSchema and RawProperty use String-based fields; validation happens in resolver.
- Domain models match 008: private-field newtypes, semantic enums, SchemaHash, BankVersion, Timestamp, ResolutionMetadata.
- Resolver matches 010: unified SchemaResolver, deterministic ordering, typed PropertyRef, resolve_all + resolve_changed.
- CQRS matches 009: UUID-first storage, name index, metadata table, save_with_metadata + list_metadata + archive access.
- PropertySpec matches 011: RawPropertySpec in schema::raw, modularized property_spec, validation rules enforced.

Constraints
- Context isolation: schema must not import note/template domains.
- Port-based CQRS with split storage ports.
- Type-driven design: private fields, validated constructors, newtypes.
- Zero-copy patterns with rkyv where applicable.
- Test-first (red-green-refactor).
- ADR required if any architectural decision is introduced.
- Run pre-commit hooks per phase until all hooks pass.

Phases

Phase 1: Baseline PropertySpec coverage (StringSpec UTF-8 bytes)
- Add tests to lock in UTF-8 byte-length semantics for StringSpec.
- Target file: lithos-core/src/schema/property_spec.rs
- Exit criteria: tests cover multibyte example (e.g., "cafe\u{00e9}").

Phase 2: Domain model alignment and invariants
- Ensure SchemaName/PropertyName enforce lowercase-only pattern per design (update patterns and tests).
- Keep private-field newtypes and semantic enums (Cardinality, Multiplicity).
- Add SchemaHash compute helper and ResolutionMetadata type (if missing).
- Exit criteria: domain types match 008 exactly; tests updated for lowercase rules.

Phase 3: Raw input layer alignment
- Make RawSchema fields tolerant: name, extends, excludes use String-based types.
- Keep RawPropertyInline fields primitive (String name, bool flags, RawPropertySpec).
- Retain RawPropertyRef as string at adapter boundary only; parse into typed PropertyRef before domain resolver.
- Exit criteria: raw types can be deserialized even when invalid; resolver owns validation.

Phase 4: Resolver alignment (SchemaResolver)
- Rename Resolver to SchemaResolver and keep graph internal.
- Use HashMap<PropertyName, Property> working set (no String keys).
- Add resolve_changed and return ResolutionMetadata with parent hash + bank version.
- Remove $ref string parsing from resolver; accept PropertyRef from adapters.
- Exit criteria: deterministic order, typed refs, incremental API available.

Phase 5: CQRS metadata + UUID-first storage alignment
- Add schema_metadata table and ResolutionMetadata persistence.
- Implement save_with_metadata and save_batch in Command and ports.
- Implement list_metadata/find_metadata_by_id/lookup_id_by_name in Query and ports.
- Ensure SchemaId is the canonical storage key with SchemaNameKey index.
- Exit criteria: CQRS API matches 009 and metadata round-trips.

Phase 6: PropertySpec modularization
- Split property_spec into submodules (validated, invariants, path, regex_cache).
- Keep RawPropertySpec in schema::raw; remove any PropertySpecDef remnants.
- Ensure validate methods use borrowed &str and path component semantics.
- Exit criteria: module layout matches 011; no behavior regressions.

Phase 7: Error model alignment
- Add structured errors per design: ParentNotFound, PropertyRefNotFound, DuplicateProperty.
- Align SchemaCommandError/SchemaQueryError variants with 009 (NotFound/Corruption/Conflict).
- Exit criteria: error variants are structured and used consistently.

Phase 8: Event model + tests
- Ensure events reflect newtypes and timestamps only (no raw primitives).
- Update tests/docs to follow new constraints (lowercase names, typed refs).
- Exit criteria: event tests green and docs updated.

Phase 9: CLI and integration alignment
- Update CLI parsing to construct SchemaName/PropertyName from raw strings.
- Update fixtures to reflect lowercase-only names and new CQRS APIs.
- Update callers to use resolve_changed with parent loader (incremental resolution).
- Exit criteria: CLI build/tests green.

Phase 10: Documentation and ADRs
- Update schema docs to match final APIs and behaviors.
- Add ADRs for storage migrations or format changes (if any).
- Exit criteria: adr:validate green; docs consistent.

Phase 11: Final verification
- Run mise run verify.
- Ensure no TODOs/debug logs remain.
- Exit criteria: verify green, code clean.

Current Status
- Phase 1: complete.
- Phase 2: partially complete (newtypes/enums done; lowercase-only validation pending).
- Next: Phase 3 (raw input layer alignment).

Notes
- Raw DTOs should be tolerant and string-based; validation belongs in resolver.
- $ref parsing is adapter responsibility; domain resolves typed PropertyRef only.
- Use #[expect] only when necessary and with descriptive reasons.
