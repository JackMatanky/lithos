# Schema Refactor Plan: File-Centric Read Model Architecture

**Status**: Planning
**Started**: 2026-03-06
**Target Completion**: TBD

---

## Executive Summary

### Goals

1. **Eliminate false DDD abstractions**: Remove `Schema` aggregate (no domain behavior), use `StoredSchema` as read model
2. **File-centric source of truth**: Store raw file content with versioning (5 versions, zstd-compressed)
3. **Hash-based staleness**: Blake3 hashing for granular change detection (timestamp fast path, hash slow path)
4. **Event-driven coordination**: Fine-grained pipeline events for observability and reactive orchestration
5. **Incremental resolution**: PropertyBank changes trigger property-level re-resolution (not full schema)
6. **Type-driven validation**: Raw validation uses types + regex (syntax only), resolution validates semantics

### Non-Goals (Deferred to Phase 3+)

- Per-property hashing for ultra-granular diffing
- LSP integration (event handlers prepared, but not implemented)
- Network-based file sources (HTTP, S3)
- Schema migration tooling

---

## Current State Analysis

### What Works Well (Keep)

✅ **Port-based CQRS with GATs**: `QueryPort` trait with `with_metadata<F, R>()` for zero-copy reads
✅ **Zero-copy reads**: GAT methods enable 2-33x faster operations (no deserialization)
✅ **Batch operations**: `find_many_by_ids()`, `are_many_stale()` amortize transactions
✅ **Staleness detection**: Timestamp-based with cascade to descendants
✅ **Resolution pipeline**: `Dereferencer → Extender → Resolver` for inheritance
✅ **Two-shape serialization**: `RawSchema` (serde) → `StoredSchema` (rkyv)

### What Doesn't Work (Change)

❌ **Fake aggregate**: `Schema` pretends to have domain behavior (events, state transitions) but is just parsed data
❌ **Generic wrappers**: 1204 lines of boilerplate (`query.rs`, `command.rs`) doing only error conversion
❌ **Nested structure**: `adapter/` folder creates confusion (ports vs adapters vs wrappers)
❌ **No raw file cache**: Can't diff changes, can't rollback, can't support offline work
❌ **Coarse staleness**: Timestamp-only detection misses file identity changes (rename, touch)
❌ **Full re-resolution**: PropertyBank changes trigger full schema re-resolution (wasteful)
❌ **No events**: Pipeline is opaque (no observability, no LSP hooks)
❌ **Orchestration in wrong layer**: `application/schema.rs` should be `schema/loader.rs` (cohesion)

---

## Target Architecture

### Data Flow

```
┌──────────────────────────────────────────────────────────────┐
│ FILE SYSTEM (Source of Truth)                               │
│ property-bank.toml + schemas/*.toml                          │
└────────────────┬─────────────────────────────────────────────┘
                 │
                 ▼ parse (serde) + hash (blake3) + validate (syntax)
┌──────────────────────────────────────────────────────────────┐
│ RAW PROPERTY BANK STORAGE                                    │
│ Table: raw_property_bank_file (singleton)                    │
│ - versions: RingBuffer<RawFileVersion, 5>                    │
│ - current_version: u8                                        │
└────────────────┬─────────────────────────────────────────────┘
                 │
                 ▼ validate (semantics) + build PropertyBank
┌──────────────────────────────────────────────────────────────┐
│ PROPERTY BANK (Read Model)                                   │
│ Table: property_bank (singleton)                             │
│ - version: BankVersion                                       │
│ - properties: Vec<Property>                                  │
│ - source_file_hash: Blake3Hash                               │
└────────────────┬─────────────────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────────────────┐
│ RAW SCHEMA STORAGE                                           │
│ Table: raw_schema_files (key: file_path)                     │
│ - versions: RingBuffer<RawFileVersion, 5>                    │
│ - current_version: u8                                        │
└────────────────┬─────────────────────────────────────────────┘
                 │
                 ▼ validate + resolve (Dereferencer → Extender → Resolver)
┌──────────────────────────────────────────────────────────────┐
│ SCHEMA (Read Model)                                          │
│ Table: schema_by_id (key: SchemaId)                          │
│ - StoredSchema: { id, name, parent_id, properties }          │
│ - source_file_hash: Blake3Hash                               │
│                                                              │
│ Table: schema_metadata (key: SchemaId)                       │
│ - bank_version, created_at, modified_at, recorded_at         │
│                                                              │
│ Table: schema_children (multimap: parent_id → child_id)      │
│ Table: schema_parent (key: child_id → parent_id)             │
└──────────────────────────────────────────────────────────────┘
```

### Module Structure (Flat + Loader)

```
lithos-core/src/schema/
├── mod.rs                    # Public API
├── error.rs                  # All error types (LoadError, SchemaError, etc.)
├── events.rs                 # SchemaEvent, PropertyBankEvent
│
├── raw.rs                    # RawSchema, RawPropertyBank (serde, syntax validation)
├── stored.rs                 # StoredSchema, StoredMetadata (rkyv, read models)
├── bank.rs                   # PropertyBank (read model, not aggregate)
├── property.rs               # Property, PropertyName (domain value objects)
├── property_spec.rs          # PropertySpec variants (domain)
├── formats.rs                # StringFormat (domain)
│
├── ports.rs                  # QueryPort, CommandPort traits with GATs (imports stored.rs)
├── db_query.rs               # Redb QueryPort implementation
├── db_command.rs             # Redb CommandPort implementation
├── db_tables.rs              # Table definitions (const TABLE_NAME: TableDefinition)
│
├── dereferencer.rs           # Property dereferencing (validates refs exist in bank)
├── extender.rs               # Inheritance tree building (validates refs exist, no cycles)
├── resolver.rs               # Property merging (validates depth, conflicts)
│
├── ingestor.rs               # File scanning + parsing (no validation)
└── loader.rs                 # Pipeline orchestration (File → Raw → Resolved → DB)
```

**Dependency graph** (no cycles):
```
raw.rs, stored.rs, property.rs, bank.rs  ← no dependencies
    ↓
ports.rs  ← imports stored.rs for return types (has GAT methods)
    ↑ implemented by
db_query.rs, db_command.rs  ← imports db_tables.rs
    ↑ used by
loader.rs  ← orchestrates: ingestor + dereferencer + extender + resolver + ports
```

**Key changes from current**:
- ❌ Removed `query.rs`, `command.rs` (1204 lines of boilerplate)
- ❌ Removed `adapter/` folder (flatten: `adapter/query.rs` → `db_query.rs`)
- ❌ Removed `application/schema.rs` (moved to `schema/loader.rs`)
- ✅ Kept GAT methods in `ports.rs` (zero-copy performance)
- ✅ Error conversion via `From` trait (automatic)

---

## Phase Breakdown

### Phase 0: Planning & Design ✅ (Current)

- [x] Identify architectural issues (aggregate vs. read model)
- [x] Define target architecture (file-centric, event-driven)
- [x] Choose module structure (Option A: flat)
- [x] Document validation boundaries (raw vs. resolution)
- [x] Create refactor plan (this document)

**Deliverable**: `SCHEMA_REFACTOR_PLAN.md` approved

---

### Phase 1: Infrastructure (Raw File Storage + Blake3)

**Goal**: Add raw file caching without breaking existing code.

#### Tasks

- [ ] **1.1**: Add `blake3` dependency to `Cargo.toml`
- [ ] **1.2**: Create `Blake3Hash` newtype wrapper
  - `pub struct Blake3Hash([u8; 32])`
  - Implements `Archive`, `Serialize`, `Deserialize` (rkyv)
  - Implements `Display` (hex encoding), `FromStr` (hex decoding)
  - Method: `compute(bytes: &[u8]) -> Self`
- [ ] **1.3**: Create `RingBuffer<T, N>` generic type
  - Fixed-size array `[Option<T>; N]`
  - Methods: `push()`, `current()`, `iter()`, `len()`
  - Implements `Archive`, `Serialize`, `Deserialize`
- [ ] **1.4**: Add `zstd` compression wrapper for rkyv
  - `pub struct ZstdCompressed;` (rkyv `with` attribute)
  - Compresses on serialize, decompresses on deserialize
  - Level 3 (balance speed vs. size)
- [ ] **1.5**: Create `RawFileVersion` type
  - Fields: `content` (zstd-compressed), `content_hash`, `created_at`, `modified_at`, `recorded_at`
  - Method: `from_file_content(content: &str, metadata: FileMetadata) -> Self`
- [ ] **1.6**: Create `RawSchemaFile` type
  - Fields: `file_path`, `versions: RingBuffer<RawFileVersion, 5>`, `current_version`
  - Methods: `add_version(content, metadata)`, `current()`, `previous_versions()`
- [ ] **1.7**: Create `RawPropertyBankFile` type (similar structure)
- [ ] **1.8**: Add database tables
  - `RAW_SCHEMA_FILES: TableDefinition<&str, &[u8]>` (key: file_path)
  - `RAW_PROPERTY_BANK_FILE: TableDefinition<&str, &[u8]>` (key: singleton "property-bank")
- [ ] **1.9**: Update `Ingestor` to compute hashes during file scan
  - Add `FileMetadata { created_at, modified_at }` extraction
  - Add `Blake3Hash::compute()` calls
  - Return `(RawSchema, Blake3Hash, FileMetadata)` tuples
- [ ] **1.10**: Update `Command` adapter to save raw files
  - New method: `save_raw_schema_file(file: &RawSchemaFile)`
  - New method: `save_raw_property_bank_file(file: &RawPropertyBankFile)`
- [ ] **1.11**: Add unit tests
  - `RingBuffer::push()` evicts oldest when full
  - `Blake3Hash` matches reference implementation
  - Zstd compression roundtrip (compress → decompress = original)
  - `RawFileVersion::from_file_content()` produces correct hash

**Verification**:
- [ ] All tests pass (`mise run test`)
- [ ] Benchmarks show <10 µs overhead per file (hash + compress)
- [ ] Raw files are saved alongside resolved schemas (DB size increase <10 MB for 1000 schemas)

**Deliverable**: Raw file storage infrastructure ready (not yet used in staleness detection)

---

### Phase 2: Two-Tier Staleness Detection

**Goal**: Use timestamp fast path + hash slow path for accurate change detection.

#### Tasks

- [ ] **2.1**: Add `source_file_hash: Blake3Hash` to `StoredMetadata`
- [ ] **2.2**: Add `created_at: Option<SystemTime>` to `StoredMetadata` (if missing)
- [ ] **2.3**: Update `partition_by_staleness()` in `SchemaService`
  - Step 1: Timestamp comparison (fast path)
  - Step 2: Hash comparison (slow path, if timestamp differs)
  - Step 3: Touch-only update (if hash matches but timestamp differs)
- [ ] **2.4**: Implement `diff_raw_files()` helper
  - Compare `RawFileVersion.content_hash` between cached and current
  - Return `FileChange::Unchanged | Modified | Renamed`
- [ ] **2.5**: Update `check_property_bank_staleness()`
  - Compare `PropertyBank.source_file_hash` to current file hash
  - If changed, emit `PropertyBankEvent::Stale { changed_properties }`
- [ ] **2.6**: Implement `find_schemas_using_properties()`
  - Query all schemas, filter by property references to bank
  - Return `HashMap<SchemaId, Vec<PropertyName>>` (affected schemas)
- [ ] **2.7**: Add `StalenessReason::BankPropertyChanged` variant
- [ ] **2.8**: Add integration test
  - Scenario: Touch file (timestamp changes, hash stays same) → no re-resolution
  - Scenario: Modify file (timestamp + hash change) → re-resolution
  - Scenario: Rename file (same hash, different path) → detect via `created_at`

**Verification**:
- [ ] Benchmarks show timestamp check is <1 µs (fast path)
- [ ] Hash check is <10 µs (slow path)
- [ ] Touch-only files don't trigger re-resolution
- [ ] File renames are detected correctly

**Deliverable**: Accurate staleness detection with minimal overhead

---

### Phase 3: Event System (Pre-LSP)

**Goal**: Add fine-grained events for observability and reactive coordination.

#### Tasks

- [ ] **3.1**: Define `SchemaEvent` enum
  - Scan events: `ScanStarted`, `FileDiscovered`, `ScanCompleted`
  - Staleness events: `SchemaFresh`, `SchemaStale`
  - Resolution events: `SchemaResolutionStarted`, `SchemaResolved`, `SchemaResolutionCompleted`
  - Persistence events: `RawFileCached`, `SchemaPersisted`
  - Error events: `ParseError`, `ValidationError`, `ResolutionError`
- [ ] **3.2**: Define `PropertyBankEvent` enum (separate from `SchemaEvent`)
  - `Fresh`, `Stale`, `ResolutionStarted`, `Resolved`, `Persisted`, `TriggeredCascade`
- [ ] **3.3**: Create `SchemaEventHandler` trait
  - Method: `handle(&self, event: &SchemaEvent)`
- [ ] **3.4**: Implement `LoggingHandler` (tracing integration)
- [ ] **3.5**: Implement `MetricsHandler` (prometheus/statsd integration)
- [ ] **3.6**: Implement `ReactiveHandler` (prefetch fresh schemas, cascade staleness)
- [ ] **3.7**: Update `SchemaService::load()` to emit events
  - Add `event_handlers: Vec<Box<dyn SchemaEventHandler>>` field
  - Add `emit(event: SchemaEvent)` helper
  - Emit events at each pipeline stage
- [ ] **3.8**: Add event testing utilities
  - `EventCollector` handler that records all events
  - Test assertions: `assert_event_sequence(expected, actual)`
- [ ] **3.9**: Add integration test
  - Scenario: Full pipeline emits correct event sequence
  - Scenario: Error during parsing emits `ParseError` event

**Verification**:
- [ ] All events are emitted in correct order
- [ ] Event handlers don't slow down pipeline (overhead <5%)
- [ ] Error events include full context for debugging

**Deliverable**: Observable, event-driven pipeline

---

### Phase 4: Incremental Property Resolution

**Goal**: PropertyBank changes trigger property-level re-resolution (not full schema).

#### Tasks

- [ ] **4.1**: Implement `diff_property_bank()`
  - Compare `PropertyBank.properties` by name + spec
  - Return `Vec<PropertyName>` (changed properties)
- [ ] **4.2**: Update `find_schemas_using_properties()`
  - Given changed property names, find schemas with `RawProperty::Ref(name)`
  - Return `HashMap<SchemaId, Vec<PropertyName>>` (schema → affected properties)
- [ ] **4.3**: Implement `Resolver::resolve_affected_properties()`
  - Take existing `StoredSchema` + changed properties
  - Re-dereference only changed properties (not full schema)
  - Return updated `StoredSchema`
- [ ] **4.4**: Update `SchemaService::load()` to use incremental resolution
  - If `PropertyBankEvent::Stale`, call `resolve_affected_properties()`
  - Skip full resolution for schemas with only bank-property changes
- [ ] **4.5**: Add benchmark
  - Measure: Full resolution vs. incremental resolution (1 property changed)
  - Target: Incremental is 10x faster

**Verification**:
- [ ] PropertyBank change affects only referencing schemas
- [ ] Only changed properties are re-dereferenced
- [ ] Benchmark shows 10x speedup for small changes

**Deliverable**: Efficient incremental updates

---

### Phase 5: Raw Validation (Type-Driven)

**Goal**: Raw validation enforces syntax + basic correctness only.

#### Tasks

- [ ] **5.1**: Define `RawValidationError` enum
  - `EmptyName`, `InvalidNameSyntax`, `InvalidPropertyNameSyntax`, `SecurityViolation`, `DuplicatePropertyName`
- [ ] **5.2**: Implement `RawSchema::validate()`
  - Check: File name syntax (alphanumeric + dash/underscore, lowercase)
  - Check: Unique property names
  - Check: `extends` name syntax (if present)
  - Check: `excludes` property name syntax
  - Does NOT check: Property ref existence, depth, circular inheritance
- [ ] **5.3**: Implement `RawPropertyBank::validate()`
  - Check: Property names are unique
  - Check: Property specs are valid (already enforced by serde types)
- [ ] **5.4**: Move semantic validation to resolution layer
  - Dereferencer: Check property refs exist in bank
  - Extender: Check schema refs exist, detect circular inheritance
  - Resolver: Check depth limits, property conflicts
- [ ] **5.5**: Update `Ingestor` to call `raw.validate()` after parsing
- [ ] **5.6**: Add unit tests
  - Valid schema passes validation
  - Invalid name (uppercase, special chars) fails
  - Duplicate property names fail
  - Path traversal attempts fail (`../../etc/passwd`)

**Verification**:
- [ ] Raw validation catches syntax errors
- [ ] Semantic errors caught during resolution (not parsing)
- [ ] Security violations are rejected

**Deliverable**: Type-driven validation with clear boundaries

---

### Phase 6: Flatten Module Structure + Remove Wrappers

**Goal**: Flatten structure, remove generic wrappers, move orchestration to loader.

#### Tasks

**Part A: Flatten adapter/ folder**
- [ ] **6.1**: Move `schema/adapter/stored.rs` → `schema/stored.rs`
- [ ] **6.2**: Rename `schema/adapter/query.rs` → `schema/db_query.rs`
- [ ] **6.3**: Rename `schema/adapter/command.rs` → `schema/db_command.rs`
- [ ] **6.4**: Rename `schema/adapter/ingestor.rs` → `schema/ingestor.rs`
- [ ] **6.5**: Delete `schema/adapter/mod.rs` (no longer needed)
- [ ] **6.6**: Create `schema/db_tables.rs` (extract table definitions from `db_query.rs`)

**Part B: Remove generic wrappers (saves 1204 lines)**
- [ ] **6.7**: Delete `schema/query.rs` (810 lines of error conversion boilerplate)
- [ ] **6.8**: Delete `schema/command.rs` (394 lines of error conversion boilerplate)
- [ ] **6.9**: Update `schema/error.rs` - ensure `From<DbError>` impls exist for all error types
- [ ] **6.10**: Update imports across codebase:
  - Replace `schema::Query<adapter::Query>` → `schema::db_query::Query`
  - Replace `schema::Command<adapter::Command>` → `schema::db_command::Command`
  - Replace `schema::adapter::*` → `schema::*`

**Part C: Move orchestration to loader**
- [ ] **6.11**: Move `application/schema.rs` → `schema/loader.rs`
- [ ] **6.12**: Rename `SchemaService` → `Loader`
- [ ] **6.13**: Update loader to use concrete port types:
  - `query: db_query::Query<'db>` (not generic)
  - `command: db_command::Command<'db>` (not generic)
- [ ] **6.14**: Delete `application/schema.rs`
- [ ] **6.15**: Update `schema/mod.rs` public API:
  ```rust
  pub mod loader;        // ← Entry point for loading
  pub mod db_query;      // ← Concrete query implementation
  pub mod db_command;    // ← Concrete command implementation
  pub mod ports;         // ← Port traits (with GATs)
  // ... other modules
  ```

**Verification**:
- [ ] All imports resolve correctly
- [ ] No circular dependencies
- [ ] Error conversion works via `From` trait
- [ ] GAT methods still available in `ports::QueryPort`
- [ ] All tests pass

**Deliverable**: Flat, cohesive module structure with 1204 fewer lines

---

### Phase 7: Remove Aggregate Layer

**Goal**: Delete `Schema` aggregate, use `StoredSchema` as read model.

⚠️ **BREAKING CHANGE** - coordinate with CLI team

#### Tasks

- [ ] **7.1**: Remove `schema/aggregate.rs` entirely
- [ ] **7.2**: Remove `Schema::try_new()`, `Schema::resolve_existing()`, `Schema::reconstruct()`
- [ ] **7.3**: Remove event management code
  - Delete `pending_events` field
  - Delete `add_event()`, `take_events()`, `pending_events()` methods
- [ ] **7.4**: Update `Resolver` to return `Vec<StoredSchema>` (not `Vec<Schema>`)
- [ ] **7.5**: Update `Query` port to return `StoredSchema` (not `Schema`)
- [ ] **7.6**: Update `Command` port to accept `&StoredSchema` (not `&Schema`)
- [ ] **7.7**: Update `SchemaService` to work with `StoredSchema`
- [ ] **7.8**: Update application layer (`application/schema.rs`)
- [ ] **7.9**: Update CLI (coordinate with CLI team)
  - Replace `Schema` usage with `StoredSchema`
  - Update command handlers
- [ ] **7.10**: Update all tests
  - Replace `Schema::try_new()` with `StoredSchema::new()`
  - Remove event assertions

**Verification**:
- [ ] All tests pass
- [ ] CLI compiles and runs
- [ ] No references to `Schema` aggregate remain

**Deliverable**: Honest read model architecture

---

### Phase 8: Documentation & Cleanup

**Goal**: Document new architecture, remove dead code.

#### Tasks

- [ ] **8.1**: Update `AGENTS.md` with new architecture rules
  - Remove: "Schema is a DDD aggregate"
  - Add: "StoredSchema is a read model (no behavior)"
  - Add: "Raw validation is syntax-only (type-driven)"
  - Add: "Resolution validates semantics (Dereferencer, Extender, Resolver)"
- [ ] **8.2**: Update `_bmad-output/project-context.md`
- [ ] **8.3**: Create ADR: "Schema as Read Model"
  - Document: Why aggregates were removed
  - Document: File-centric source of truth
  - Document: Event-driven pipeline
- [ ] **8.4**: Update rustdoc comments
  - `stored.rs`: Document read model pattern
  - `raw.rs`: Document validation boundaries
  - `events.rs`: Document event-driven coordination
- [ ] **8.5**: Remove dead code
  - Search for unused types, methods
  - Remove commented-out code
  - Clean up imports
- [ ] **8.6**: Add architecture diagram to `docs/`
  - File flow diagram (file → raw → resolved → DB)
  - Module dependency graph
- [ ] **8.7**: Update `README.md` with new architecture summary

**Verification**:
- [ ] Documentation is accurate and complete
- [ ] No dead code remains
- [ ] Clippy passes with no warnings

**Deliverable**: Fully documented new architecture

---

## Validation Boundaries (Critical Reference)

| Layer | Validates | Does NOT Validate | Tool |
|-------|-----------|------------------|------|
| **Raw** (syntax) | File name format, property name syntax, unique names, security (path traversal) | Property ref existence, schema ref existence, depth, cycles | Regex, type system (serde) |
| **Dereferencer** (property refs) | Property refs exist in PropertyBank | Schema refs, depth, cycles | HashMap lookup |
| **Extender** (schema refs) | Schema refs exist, no circular inheritance | Depth, property conflicts | Graph traversal (BFS) |
| **Resolver** (semantics) | Inheritance depth, property conflicts | - | Tree traversal |

**Key principle**: **Validate as late as possible** (only when you have the data needed to validate).

---

## Rollback Plan

If issues arise during implementation:

### Phase 1-3 Rollback
- Keep raw file storage (no harm)
- Revert staleness detection changes
- Remove event handlers (optional feature)

### Phase 4-5 Rollback
- Revert to full re-resolution (no incremental)
- Keep validation layer (no harm)

### Phase 6-7 Rollback (Breaking)
- Restore `schema/aggregate.rs` from git
- Restore `schema/adapter/` structure
- Revert all imports

**Git strategy**: Each phase is a separate branch, merged only after verification passes.

---

## Success Metrics

### Performance
- [ ] Staleness detection: <1 µs per schema (timestamp fast path)
- [ ] Hash computation: <10 µs per schema (slow path)
- [ ] Incremental resolution: 10x faster than full resolution (1 property changed)
- [ ] Raw file storage: <10 MB for 1000 schemas (zstd compression)

### Correctness
- [ ] All tests pass (unit + integration)
- [ ] No regressions in existing functionality
- [ ] Security validation prevents path traversal

### Maintainability
- [ ] No circular dependencies
- [ ] Clear validation boundaries
- [ ] Event-driven pipeline is observable

---

## Decisions Made ✅

All architectural decisions have been finalized (see `SCHEMA_REFACTOR_DECISIONS.md` for details):

1. ✅ **Module structure**: Flat (no `adapter/` folder)
2. ✅ **Generic wrappers**: Remove (use concrete types + `From` trait for error conversion)
3. ✅ **Loader location**: `schema/loader.rs` (not `application/schema.rs`)
4. ✅ **Per-property hashing**: Defer to Phase 3+
5. ✅ **Event handler registration**: Dynamic (`add_handler()` method)
6. ✅ **PropertyBank cascade**: Individual `SchemaEvent::SchemaStale` events
7. ✅ **Event storage**: Transient (return from methods, no DB table)
8. ✅ **Malicious content**: Multi-layer validation (size + depth + regex limits)
9. ✅ **GATs**: Keep and expand (critical for zero-copy performance)

---

## Timeline Estimate

| Phase | Tasks | Estimated Time | Dependencies |
|-------|-------|---------------|--------------|
| Phase 0 | Planning | 2 hours | - |
| Phase 1 | Infrastructure | 8 hours | Phase 0 |
| Phase 2 | Staleness | 6 hours | Phase 1 |
| Phase 3 | Events | 6 hours | Phase 2 |
| Phase 4 | Incremental | 4 hours | Phase 3 |
| Phase 5 | Validation | 4 hours | Phase 1 |
| Phase 6 | Flatten + remove wrappers | 6 hours | Phase 5 |
| Phase 7 | Remove aggregate | 6 hours | Phase 6 |
| Phase 8 | Documentation | 4 hours | Phase 7 |
| **Total** | | **46 hours** (~6 days) | |

**Note**: Timeline assumes focused work with no interruptions. Add buffer for testing, code review, and unexpected issues.

---

## Next Steps

1. **Review this plan** - confirm approach is correct
2. **Answer open questions** - make architectural decisions
3. **Create feature branch** - `git checkout -b refactor/schema-read-model`
4. **Start Phase 1** - infrastructure (raw file storage)
5. **Track progress** - check off tasks as completed

---

## Tracking Progress

**Last Updated**: 2026-03-06
**Current Phase**: Phase 0 (Planning)
**Completed Phases**: None
**Blocked Tasks**: None

### Phase Checklist

- [x] Phase 0: Planning
- [ ] Phase 1: Infrastructure
- [ ] Phase 2: Staleness
- [ ] Phase 3: Events
- [ ] Phase 4: Incremental
- [ ] Phase 5: Validation
- [ ] Phase 6: Module structure
- [ ] Phase 7: Remove aggregate
- [ ] Phase 8: Documentation
