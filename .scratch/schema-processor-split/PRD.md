---
title: PRD - Schema Processor Split Architecture
labels: needs-triage
status: draft
created: 2026-05-24
---

# PRD: Schema Processor Split Architecture

## Problem Statement

The current schema processing pipeline (`lithos-core/src/schema/schema_processor.rs`, ~3200 lines) conflates two distinct responsibilities: individual schema file processing (parsing, validation, property expansion) and cross-schema concerns (inheritance graph construction, property merging). This monolithic design prevents:

1. **Parallel processing** of independent schema files during the parse/validate/expand phase
2. **Incremental updates** with surgical property re-expansion when only property bank references change
3. **Clear separation** between file-local validation and cross-schema resolution
4. **Maintainability** due to rigid coupling between file processing and graph-based inheritance

The processor cannot be meaningfully refactored because its stages mix file-level and graph-level operations throughout, making it impossible to isolate parallelizable work from sequential dependency resolution.

## Solution

Introduce **BaseSchema** as an intermediate domain type between `RawSchema` (syntax-validated input) and `Schema` (fully resolved aggregate with inheritance applied). BaseSchema represents a **self-contained, file-local schema** with:

- Resolved `PropertyMap` (all `$ref` entries expanded to domain `Property` types)
- Validated schema name, extends reference, and excludes list
- No cross-schema dependencies resolved (inheritance not applied)

Split schema processing into **two independent processors**:

1. **BaseSchemaProcessor** (file-parallel): processes each schema file independently through typestate pipeline (mirroring `property_bank_processor` pattern), producing BaseSchema + deltas
2. **InheritanceProcessor** (graph-sequential): consumes BaseSchemas + deltas, builds inheritance graph, applies property merging, produces final `Schema` aggregates

This architecture enables:

- Parallel file processing (each schema processed independently until BaseSchema constructed)
- Incremental property re-expansion when only bank references change (surgical updates, not full rebuilds)
- Clean separation of concerns (file validation vs. inheritance resolution)
- Maintainable, testable processors with clear contracts

## User Stories

1. As a vault user, I want schema files to be processed in parallel when possible, so that vault loading is faster for large schema directories.

2. As a vault user, I want property bank changes to trigger minimal schema rebuilds, so that adding or modifying a property definition doesn't force full re-processing of all schemas.

3. As a developer, I want to understand schema file processing independently from inheritance logic, so that I can maintain and extend the system confidently.

4. As a developer, I want to test file-level schema validation without needing to set up complex inheritance graphs, so that I can write focused unit tests.

5. As a developer, I want to test inheritance resolution separately from file parsing, so that I can verify merge semantics in isolation.

6. As a developer, I want explicit delta types (PropertyDelta, ExcludesDelta, ExtendsDelta) passed between processing stages, so that I can reason about what changed and why.

7. As a vault user, I want schemas with only timestamp changes (no semantic changes) to be quickly refreshed without re-parsing, so that vault operations stay fast.

8. As a vault user, I want schemas with only content hash changes (but identical property hashes) to skip expensive rebuild steps, so that cosmetic file edits don't trigger unnecessary work.

9. As a developer, I want StaleReferences status to handle property bank changes independently from file staleness, so that I can apply targeted re-expansion for affected properties only.

10. As a developer, I want property IDs preserved by name during re-expansion, so that schema identity remains stable across property bank updates.

11. As a developer, I want BaseSchema persisted with per-ID storage, so that I can incrementally fetch and update individual schemas without loading all schemas.

12. As a developer, I want RawSchemaView to remain the source of content hashes and staleness checks, so that BaseSchema persistence stays focused on domain state.

13. As a developer, I want views/properties.rs removed, so that there's no overlap or confusion between BasePropertiesView and BaseSchema persistence.

14. As a developer, I want deterministic handoff between BaseSchemaProcessor and InheritanceProcessor, so that I can test and replay processing pipelines reliably.

15. As a developer, I want lifecycle events (Fresh, New, Stale, Deleted) emitted in deterministic SchemaId order, so that test assertions are stable and reproducible.

16. As a developer, I want Fresh schemas (unchanged) to skip expensive processing but still appear in the handoff, so that InheritanceProcessor has a complete view of all schemas.

17. As a developer, I want New schemas to be represented distinctly from Stale schemas in the handoff, so that create vs. update semantics are explicit.

18. As a developer, I want Deleted schemas to remove BaseSchema persistence immediately and emit lifecycle events, so that storage remains truthful and downstream processors see deletions deterministically.

19. As a developer, I want full rebuild fallback triggered only for parse/view corruption, incoherent deltas, or structural ref conflicts, so that incremental updates are the default path.

20. As a developer, I want the typestate pattern from property_bank_processor replicated exactly for BaseSchemaProcessor, so that I can leverage proven stage/status architecture.

21. As a developer, I want InMemoryRepository used for component tests by default, so that I can write fast, isolated tests without file I/O.

22. As a developer, I want custom test fakes only for unreachable/error-injection branches, so that test infrastructure stays minimal and focused.

23. As a developer, I want colocated unit/component tests with the processor code, so that tests live near the implementation they verify.

24. As a developer, I want integration tests in lithos-core/tests/, so that end-to-end behavior is verified with real adapters.

25. As a developer, I want fixtures specific to each unit suite, so that test data is scoped appropriately.

26. As a developer, I want integration test fixtures in common/ or inline, so that shared setup is reusable but not mandatory.

27. As a developer, I want this work delivered in three phases (BaseSchema + processor, InheritanceProcessor rewrite, behavioral migration), so that changes are reviewable and risk is isolated.

28. As a developer, I want Phase 1 to introduce BaseSchema and BaseSchemaProcessor without changing existing behavior, so that I can verify new infrastructure before migration.

29. As a developer, I want Phase 2 to rewrite InheritanceProcessor from scratch using BaseSchema + deltas, so that inheritance logic is clean and maintainable.

30. As a developer, I want Phase 3 to migrate Builder to use the new processors and remove old overlap, so that the system fully adopts the new architecture.

31. As a developer, I want extends_delta to be name-based (not ID-based) at BaseSchema stage, so that file-local processing doesn't require global schema existence checks.

32. As a developer, I want SchemaId included inside BaseSchema payload, so that KV store denormalization avoids key/payload ambiguity.

33. As a developer, I want all three deltas (property, excludes, extends) always carried with explicit Unchanged/Empty variants in StaleReady, so that downstream logic is deterministic without Option unpacking.

34. As a developer, I want StaleReferences to use RawSchemaView.current().changed_bank_references(delta_names) for intersection checks, so that reference staleness detection is consistent with existing flow.

35. As a developer, I want stale_reference_names included in PropertyDelta.upserts (not separate), so that the handoff contract stays minimal and semantic.

## Three-Phase Implementation Roadmap

### Phase 1: BaseSchema + BaseSchemaProcessor

**Implemented in**: `.scratch/base-schema/`

**Goals**: Introduce new intermediate domain type and file-local processor without changing existing behavior.

**Deliverables**:
- `BaseSchema` domain type with persisted state (id, name, properties, extends, excludes)
- `BaseSchemaProcessor` typestate pipeline mirroring PropertyBankProcessor pattern
- `ExtendsDelta` type for name-based inheritance change tracking
- `BaseSchemaChange` handoff envelope (Fresh/New/Stale/Deleted lifecycle events)
- Repository methods for BaseSchema persistence
- Unit, component, and integration tests
- StaleReferences handling for property bank changes

**Acceptance Criteria**:
- All tests pass
- BaseSchema can be persisted and retrieved
- BaseSchemaProcessor produces correct `BaseSchemaChange` handoff
- No changes to existing `Builder` or `schema_processor` flow (old code still runs)

**Estimated Effort**: ~1200-1500 lines new code + tests

### Phase 2: InheritanceProcessor Rewrite

**Goals**: Clean implementation of inheritance logic consuming BaseSchema + deltas.

**Deliverables**:
- `InheritanceProcessor` built around `BaseSchemaChange` stream
- Name → ID resolution for extends relationships
- Graph construction with cycle detection
- Incremental vs. full-merge decision logic based on deltas
- Property merging via `Merger::inherit_properties`
- Integration tests for inheritance scenarios

**Acceptance Criteria**:
- All tests pass
- InheritanceProcessor produces correct `Schema` aggregates
- Incremental updates preserve PropertyId stability
- Error handling matches existing schema_processor behavior
- Old schema_processor.rs still present (not removed yet)

**Estimated Effort**: ~800-1000 lines new code + tests

### Phase 3: Behavioral Migration + Cleanup

**Goals**: Wire new processors into Builder, remove old code.

**Deliverables**:
- Modified `Builder` flow using BaseSchemaProcessor → InheritanceProcessor pipeline
- Removed `schema_processor.rs` (~2900 lines)
- Removed `views/properties.rs` (~73 lines)
- Removed BasePropertiesView table
- Integration tests confirming identical behavior

**Acceptance Criteria**:
- All existing integration tests pass
- `Builder::load_all()` produces identical `Schema` output as before
- Performance comparable or better
- No dead code remaining

**Estimated Effort**: ~100-150 lines modifications, ~3300 lines deleted (~3200 from schema_processor.rs, ~73 from views/properties.rs, ~30 from storage)

## Key Architectural Decisions

### Processor Separation
- **BaseSchemaProcessor** = file-local, parallelizable, no cross-schema dependencies
- **InheritanceProcessor** = graph-based, sequential, requires all BaseSchemas

### Handoff Contract
- `BaseSchemaChange` enum with explicit lifecycle states (Fresh/New/Stale/Deleted)
- Deterministic emission order (sorted by SchemaId)
- Fresh schemas carry only ID (Phase 2 fetches from repository if needed)
- Stale schemas carry BaseSchema + all three deltas (property, excludes, extends)

### Incremental-First Strategy
- Targeted property re-expansion for StaleReferences (not full rebuild)
- PropertyId preservation by name during updates
- Full rebuild escalation only for: parse failures, incoherent deltas, structural ref conflicts

### TypeState Pattern
- Mirror PropertyBankProcessor exactly for consistency
- Stages: Discovery, Comparison, Parsed, Analysis, Refresh, Construction, Completed
- Additional status: StaleReferences (orthogonal to file staleness)

### Dependency Direction
- BaseSchemaProcessor → no dependencies on InheritanceProcessor
- InheritanceProcessor → depends on BaseSchemaChange contract
- Builder → orchestrates both processors

## Out of Scope

- Actual parallel execution infrastructure for BaseSchemaProcessor (Phase 1 establishes file-local independence; parallelism implementation deferred)
- Performance micro-optimizations beyond architectural improvements
- Fingerprint optimization for `Fresh` handoff (deferred to post-Phase 3 profiling)
- Property bank processor modifications
- Changes to RawSchemaView or schema parsing logic
- Raw `Deserialize` custom validation logic redesign

## Risk Mitigation

**Three-phase delivery reduces risk**:
- Phase 1: new infrastructure coexists with old, no behavior change, easy to verify correctness
- Phase 2: isolated rewrite of inheritance logic, old processor still present for comparison/fallback
- Phase 3: behavioral migration is minimal (swap processor calls in Builder), easy to revert if issues arise

**Explicit test-first development**:
- Each phase gate requires test coverage passing before proceeding
- Integration tests serve as regression detection between phases
- Contract tests ensure handoff stability across processor boundary

**Typestate enforcement**:
- Compile-time guarantees prevent invalid state transitions
- Proven pattern (property_bank_processor) reduces implementation risk
- Exhaustive branch coverage via `#[must_use]` and explicit enums

## Further Notes

- This PRD treats the inheritance processor replacement as a **rewrite from scratch**, not a refactor, due to the rigid coupling in the existing ~3200-line `schema_processor.rs`
- The architecture is optimized for maintainability, deterministic incremental behavior, and clear separation of concerns
- Phase 1 is intentionally non-breaking: it adds infrastructure without changing existing behavior
- The BaseSchemaProcessor pattern deliberately mirrors the proven PropertyBankProcessor typestate design for consistency
- After PRD approval, decompose into actionable issues per phase
- **Phase 1 implementation details are in `.scratch/base-schema/PRD.md`**
