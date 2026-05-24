---
title: PRD - BaseSchema Domain Type and File Processor (Phase 1)
labels: needs-triage
status: draft
created: 2026-05-24
context: .scratch/schema-processor-split/
---

# PRD: BaseSchema Domain Type and File Processor

> **Parent PRD**: `.scratch/schema-processor-split/PRD.md`
> This implements **Phase 1** of the schema processor split architecture.
> **Depends on**: Phase 0 (RawSchema.extends and SchemaVersion.extends refactored to Vec<SchemaName>)
> See parent PRD for full architectural rationale.

## Problem Statement

Schema processing currently lacks an intermediate domain state between `RawSchema` (syntax-validated input) and `Schema` (fully resolved aggregate with inheritance applied). The existing `schema_processor.rs` (~3200 lines) conflates file-local processing (parsing, validation, property expansion) with cross-schema concerns (inheritance graph construction, property merging).

This Phase 1 PRD focuses on introducing the missing intermediate layer: **BaseSchema** as a persisted domain type with its own file-local processor pipeline.

## Solution

Introduce **BaseSchema** as a persisted intermediate domain type representing a **self-contained, file-local schema**:

- Resolved `PropertyMap` (all `$ref` entries expanded to domain `Property` types)
- Validated schema name, extends reference (name-based), and excludes list
- No cross-schema dependencies resolved (inheritance not applied)

Implement **BaseSchemaProcessor** using the proven typestate pattern from `property_bank_processor`:

- File-local processing (no cross-schema existence checks)
- Incremental updates with targeted property re-expansion for StaleReferences
- Deterministic handoff contract (`BaseSchemaChange` lifecycle events)
- PropertyId stability preservation across updates

## User Stories (Phase 1 Focused)

1. As a developer, I want BaseSchema to store only schema-local semantic data (SchemaId, name, properties, extends, excludes), so that persistence does not duplicate raw view metadata concerns.

2. As a developer, I want BaseSchema processing to follow the PropertyBank typestate pattern, so that orchestration style is consistent across processors.

3. As a developer, I want explicit deltas for properties, excludes, and extends, so that downstream inheritance processing can decide incremental vs rebuild paths deterministically.

4. As a developer, I want extends deltas to remain name-based in BaseSchema processing, so that file-local processing does not perform cross-schema existence checks.

5. As a developer, I want stale property-bank reference detection based on name intersection from raw schema views, so that reference-driven updates are deterministic and cheap.

6. As a developer, I want stale reference effects represented through `PropertyDelta.upserts`, so that handoff payload stays minimal and semantic.

7. As a developer, I want targeted re-expansion for affected references, so that updates are incremental-first and avoid full rebuild by default.

8. As a developer, I want property IDs preserved by property name during targeted re-expansion, so that identity stability is maintained.

9. As a developer, I want full rebuild fallback only for parse/view corruption, incoherent delta results, or structural reference conflicts, so that rebuild escalation is predictable.

10. As a developer, I want normalized handoff outcomes (`Fresh`, `New`, `Stale`, `Deleted`), so that Phase 2 inheritance processor receives semantic lifecycle events only.

11. As a developer, I want `StaleTimestamps` and `StaleContent` to normalize to `Fresh` when semantic schema state is unchanged, so that metadata-only drift does not trigger downstream semantic churn.

12. As a developer, I want deletion lifecycle events emitted and BaseSchema persistence removed immediately, so that storage and downstream processors stay consistent.

13. As a developer, I want deterministic `SchemaId` ordering in handoff output, so that tests and replay runs are reproducible.

14. As a developer, I want unit/component tests colocated with processor code and integration tests in `lithos-core/tests`, so that test boundaries are explicit.

15. As a developer, I want to use `InMemoryRepository` by default, so that component tests stay fast and realistic.

## Implementation Decisions

### Core Types

**BaseSchema Domain Type**
- Location: `lithos-core/src/schema/base_schema.rs` (new file)
- Fields:
  - `id: SchemaId` (explicit identity, avoids key/payload ambiguity)
  - `name: SchemaName` (validated)
  - `properties: PropertyMap` (fully expanded, domain `Property` types)
  - `extends: Vec<SchemaName>` (name-based, not ID-based)
    - Uses `Vec` to align with final `Schema.parents: Vec<SchemaId>` capability
    - Raw parsing currently only supports `Option<SchemaName>`; BaseSchema paves the way for multiple inheritance
  - `excludes: Vec<PropertyName>`
- Persistence: per-ID table via repository adapter
- Archived with `rkyv` for efficient serialization

**ExtendsDelta Type**
- Location: `lithos-core/src/schema/delta.rs` (extend existing module)
- Enum variants designed for multiple inheritance support:
  - `Unchanged`
  - `Added(SchemaName)`
  - `Removed(SchemaName)`
  - `Rewired { from: SchemaName, to: SchemaName }`
- Note: Can operate on individual parents within the `Vec<SchemaName>`; batch operations handled by applying multiple deltas
- Name-based (not ID-based) to keep BaseSchemaProcessor file-local

**BaseSchemaChange Handoff Envelope**
- Location: `lithos-core/src/schema/base_schema_processor.rs` (export from processor module)
- Enum variants:
  - `Fresh { schema_id: SchemaId }` (unchanged, Phase 2 can fetch if needed)
  - `New { schema_id: SchemaId, base_schema: BaseSchema }` (newly created)
  - `Stale { schema_id: SchemaId, base_schema: BaseSchema, property_delta: PropertyDelta, excludes_delta: ExcludesDelta, extends_delta: ExtendsDelta }` (semantic changes)
  - `Deleted { schema_id: SchemaId }` (removed from source)
- Emission order: deterministic `SchemaId` sort for test stability
- Normalization: `StaleTimestamps` + `StaleContent` => `Fresh` when semantic state unchanged

### BaseSchemaProcessor Architecture

**Typestate Pipeline** (mirrors `property_bank_processor` exactly)
- Stages: `Discovery`, `Comparison`, `Parsed`, `Analysis`, `Refresh`, `Construction`, `Completed`
- Statuses: `Unknown`, `Missing`, `Present`, `Suspect`, `Stale`, `ParsedStale`, `StaleTimestamps`, `StaleContent`, `StaleReferences`, `New`, `Changed`, `Fresh`, `FreshReady`, `NewReady`, `StaleReady`
- Location: `lithos-core/src/schema/base_schema_processor.rs` (new file, ~800-1000 lines)

**StaleReferences Handling**
- Trigger: `RawSchemaView.current().changed_bank_references(delta_names)` non-empty
- Orthogonal to file staleness (Fresh + StaleReferences is valid state)
- Incremental path:
  - Parse file
  - Targeted re-expand: only properties referencing changed bank entries
  - Preserve existing `PropertyId` by name via `with_ids(...)`
  - Avoid full rebuild unless fallback triggered

**Full Rebuild Fallback Policy**
- Triggered only when:
  1. Parse fails or view is corrupt
  2. Delta engine cannot produce coherent targeted update
  3. Structural conflict in refs (e.g., ref target missing after bank update)
- Otherwise always incremental

**Terminal Outputs**
- `FreshReady { base_schema }` → `.into_base_schema()`
- `NewReady { base_schema }` → `.into_base_schema()`
- `StaleReady { base_schema, property_delta, excludes_delta, extends_delta }` → `.into_base_schema_with_changes()`
- All deltas carried explicitly (no `Option` for sparse representation)

### Repository Adapter Changes

**New Methods**
- `save_base_schema(id: SchemaId, base: &BaseSchema) -> Result<()>`
- `get_base_schema(id: SchemaId) -> Result<Option<BaseSchema>>`
- `find_base_schemas_by_ids(ids: &[SchemaId]) -> Result<Vec<BaseSchema>>`
- `delete_base_schema(id: SchemaId) -> Result<()>`

**Table Schema**
- Table: `base_schema_by_id`
- Key: `SchemaId`
- Value: `BaseSchema` (archived with `rkyv`)

**Migration Path**
- Phase 1: add new methods, no existing table changes
- Phase 3: remove `views/properties.rs` and `BasePropertiesView` table after migration complete

### Handoff Contract for Phase 2

The `BaseSchemaChange` enum serves as the contract between BaseSchemaProcessor (Phase 1) and InheritanceProcessor (Phase 2):

**Contract guarantees:**
- Deterministic emission order (sorted by `SchemaId`)
- Complete coverage (every schema in source produces exactly one event)
- Semantic normalization (metadata-only changes emit `Fresh`, not `Stale`)
- Explicit deltas in `Stale` variant (all three deltas always present, using Unchanged/Empty variants)

**Phase 2 expectations:**
- `Fresh` schemas: fetch from repository if needed for inheritance graph
- `New` schemas: construct initial inheritance projection
- `Stale` schemas: apply incremental or full merge based on deltas
- `Deleted` schemas: remove from inheritance graph and persisted aggregates

## Testing Strategy

### What Makes a Good Test

Tests should verify **behavior through public interfaces**, not implementation details. A good test:

- Uses only public APIs (processor entry points, typestate transitions, output contracts)
- Survives internal refactors (renaming private functions should not break tests)
- Describes what the system does, not how it does it
- Reads like a specification (e.g., "Fresh schema with StaleReferences triggers targeted re-expand")

Avoid:
- Testing private methods directly
- Mocking internal collaborators (use real types or integration-level doubles)
- Asserting on internal state (test observable outputs only)
- Brittle assertions on structure (test semantic correctness, not field order)

### Unit/Component Tests (Colocated)

**Location**: `#[cfg(test)] mod tests` inside `base_schema_processor.rs`

**Scope**: typestate transitions, delta classification, staleness detection

**Doubles**: `InMemoryRepository` (default), custom fakes only for error injection

**Coverage**:
- Each stage → status branch (e.g., `Discovery → Missing`, `Discovery → Present`)
- `StaleReferences` cross-cases (`Fresh + StaleReferences`, `StaleTimestamps + StaleReferences`, `Stale + StaleReferences`)
- Incremental re-expand with `PropertyId` preservation
- Full rebuild fallback triggers (corruption, incoherent delta, ref conflict)
- Delta normalization (`StaleTimestamps`/`StaleContent` → `Fresh`)
- Lifecycle events (`Fresh`, `New`, `Stale`, `Deleted`)

**Estimated**: ~15 typestate transition tests + ~5 cross-case tests

### Integration Tests

**Location**: `lithos-core/tests/base_schema_processor.rs`

**Scope**: end-to-end with real `FsReader`, repository adapter, fixture files

**Prior art**: `lithos-core/tests/property_bank_processor.rs` (replicate pattern)

**Fixtures**: in-file or `lithos-core/tests/common/`

**Coverage**:
- Cold start (all missing) → construct all new
- Incremental run (some fresh, some stale, some new, some deleted)
- Property bank change triggering `StaleReferences` across multiple schemas
- Handoff batch emission (order, completeness)

**Estimated**: ~3 full-pipeline tests

### Contract Tests (Phase 1 → Phase 2 Boundary)

**Location**: `lithos-core/tests/base_schema_handoff_contract.rs`

**Scope**: verify `BaseSchemaChange` envelope determinism

**Coverage**:
- `SchemaId` ordering
- Delta completeness (all three deltas present when expected)
- Lifecycle event accuracy (Fresh/New/Stale/Deleted match actual state)

**Estimated**: ~5 contract verification tests

## File Changes

### New Files

1. **`lithos-core/src/schema/base_schema.rs`** (~150 lines)
   - `BaseSchema` struct + accessors
   - Persistence trait implementations (`Archive`, `Serialize`, `Deserialize`)

2. **`lithos-core/src/schema/base_schema_processor.rs`** (~800-1000 lines)
   - Typestate stages + statuses (mirror `property_bank_processor`)
   - Branch enums (`ComparisonBranch`, `TimestampBranch`, `ContentBranch`, `AnalysisBranch`)
   - `BaseSchemaChange` handoff envelope
   - Terminal `.into_base_schema()` / `.into_base_schema_with_changes()` APIs

3. **`lithos-core/tests/base_schema_processor.rs`** (~200-300 lines)
   - Integration tests mirroring `property_bank_processor` flow
   - Fixture setup in common/

### Modified Files

1. **`lithos-core/src/schema/delta.rs`** (~40 lines added)
   - Add `ExtendsDelta` enum (ExcludesDelta already exists in this module)

2. **`lithos-core/src/schema/mod.rs`** (~10 lines)
   - Export `base_schema`, `base_schema_processor`

3. **`lithos-core/src/schema/repository.rs`** (~40 lines)
   - Add `save_base_schema`, `get_base_schema`, `find_base_schemas_by_ids`, `delete_base_schema` to trait

4. **`lithos-core/src/schema/storage/write.rs`** (~40 lines added)
   - Implement BaseSchema write repository methods (`save_base_schema`, `delete_base_schema`)

5. **`lithos-core/src/schema/storage/read.rs`** (~40 lines added)
   - Implement BaseSchema read repository methods (`get_base_schema`, `find_base_schemas_by_ids`)

6. **`lithos-core/src/schema/storage/tables.rs`** (~10 lines added)
   - Add `BASE_SCHEMA_BY_ID` table definition

**Total new code**: ~1250-1550 lines (including tests)

**Note**: Storage implementation split across `schema/storage/` module (not a monolithic `redb_adapter.rs`)

## Test Matrix

### Typestate Transition Tests (~15 tests)

- **Discovery branches**:
  - Missing view → parsed new path
  - Present view → comparison path

- **Comparison branches**:
  - Timestamp match → fresh
  - Timestamp mismatch + content match → stale timestamps refresh → fresh
  - Timestamp mismatch + content mismatch → parsed stale → analysis

- **Analysis branches**:
  - No semantic deltas → stale content refresh → fresh
  - Semantic delta → stale
  - Corrupt/missing current version → new/full rebuild fallback

### StaleReferences Cross-Cases (~5 tests)

- Fresh + stale refs → targeted re-expand incremental stale
- Stale timestamps + stale refs → targeted re-expand + metadata sync behavior
- Stale content + stale refs → combined semantic stale behavior
- Stale with fallback trigger → full rebuild (not incremental)

### Fallback Policy Tests (~3 tests)

- Parse failure → full rebuild
- Incoherent delta → full rebuild
- Structural reference conflict → full rebuild

### Identity Stability Tests (~2 tests)

- Property ID preserved by name on targeted re-expansion
- Property ID preserved across multiple incremental updates

### Lifecycle Output Tests (~5 tests)

- Metadata-only outcomes normalized to `Fresh`
- Semantic changes emit `Stale` with all deltas
- New files emit `New`
- Deletions remove persistence and emit `Deleted`
- Output ordering is deterministic `SchemaId` sort

### Integration Tests (~3 tests)

- Cold start: all missing → all new
- Incremental: mixed fresh/stale/new/deleted
- Property bank change → StaleReferences → targeted updates

### Contract Tests (~5 tests)

- SchemaId ordering verification
- Delta completeness verification
- Lifecycle event accuracy
- Fresh normalization correctness
- Deleted persistence cleanup

**Total: ~38 tests covering all critical paths**

## Acceptance Criteria

- [ ] All tests pass (unit, component, integration, contract)
- [ ] BaseSchema can be persisted and retrieved
- [ ] BaseSchemaProcessor produces correct `BaseSchemaChange` handoff
- [ ] PropertyId stability preserved across incremental updates
- [ ] StaleReferences triggers targeted re-expansion (not full rebuild)
- [ ] Metadata-only changes normalize to `Fresh`
- [ ] Deleted schemas remove persistence and emit lifecycle event
- [ ] Handoff output is deterministic (SchemaId-ordered)
- [ ] No changes to existing `Builder` or `schema_processor` flow (old code still runs)

## Out of Scope (Phase 1)

- InheritanceProcessor implementation (Phase 2)
- Builder integration changes (Phase 3)
- Removal of old schema_processor.rs (Phase 3)
- Removal of views/properties.rs (Phase 3)
- Actual parallel execution infrastructure (deferred post-Phase 3)
- Fingerprint optimization for Fresh (TODO added, defer to post-Phase 3)

## Further Notes

- This phase is intentionally **non-breaking**: it adds new infrastructure without changing existing vault loading behavior
- The BaseSchemaProcessor pattern deliberately mirrors PropertyBankProcessor for consistency and proven reliability
- The handoff contract (`BaseSchemaChange`) is carefully designed for Phase 2 consumption with minimal coupling
- PropertyId stability is critical for downstream inheritance processor correctness
- StaleReferences handling is the most complex branch - requires careful testing of incremental re-expansion logic
- After implementation, create Phase 2 PRD for InheritanceProcessor rewrite
