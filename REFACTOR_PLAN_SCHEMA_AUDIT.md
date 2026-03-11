# Schema Context Refactor Plan Audit (Start)

**Context:** `lithos-core/src/schema/`
**Template:** `REFACTOR_PLAN_CHECKLIST_TEMPLATE.md`
**Primary Authority:** `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`

---

## 0) Inputs and Constraints

- [x] Read `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md` (primary authority).
- [x] Read `_bmad-output/project-context.md` and confirm latest rules.
- [x] Read ADR 002 (Repository) only for historical context.
- [x] Read `docs/refs/rust/naming-taxonomy.md` and confirm method naming rules.
- [x] Confirm context isolation: no cross-imports between business contexts (not verified in code yet).
- [x] Confirm file-based source-of-truth requirement for this refactor.
- [x] Reviewed schema planning docs (ignore irrelevant architectural aspects):
  - `SCHEMA_REFACTOR_MIGRATION.md`
  - `SCHEMA_REFACTOR_PLAN.md`
  - `SCHEMA_REFACTOR_RESEARCH.md`

---

## 1) Full File and Component Audit (Context Inventory)

### 1.1 File Inventory

Directory: `lithos-core/src/schema/`

- `bank.rs` - PropertyBank aggregate, versioning, events, rkyv derives
- `db_command.rs` - Redb-backed Command adapter (CQRS write port)
- `db_query.rs` - Redb-backed Query adapter (CQRS read port)
- `error.rs` - SchemaError + SchemaCommandError/SchemaQueryError/SchemaIngestionError
- `events.rs` - Domain + pipeline events, event handlers, rkyv derives
- `expander.rs` - RefExpander stage (resolves property $ref)
- `extender.rs` - Extender stage (builds SchemaTree, topological order)
- `id.rs` - SchemaId, SchemaName value objects
- `ingestor.rs` - FsReader-based file ingestion and raw parsing
- `loader.rs` - Pipeline orchestrator (staleness, expansion, resolution, persistence)
- `mod.rs` - Context module wiring + db table definitions + migration compat
- `ports.rs` - CQRS ports: Command + Query traits
- `property.rs` - Property entity, PropertyId, Optionality, Multiplicity
- `raw.rs` - RawSchema + RawPropertyBank + Raw* specs, syntax validation
- `resolver.rs` - Resolver stage (merge properties, build StoredSchema)
- `storage.rs` - StoredSchema, StoredProperty, StoredMetadata, raw file storage

Submodule: `lithos-core/src/schema/property_spec/`

- `mod.rs` - PropertySpec enum + conversions from raw
- `bool.rs`, `date.rs`, `file.rs`, `number.rs`, `string.rs` - Spec variants + validation

### 1.2 Component Inventory (Initial)

- Domain types: `Property`, `PropertyBank`, `SchemaId`, `SchemaName`, `PropertyId`, `PropertyName`, `PropertySpec` and variants
- Raw types: `RawSchema`, `RawPropertyBank`, `RawProperty*`, `Raw*Spec` (in `raw.rs`)
- Stored/Projection types: `StoredSchema`, `StoredProperty`, `StoredMetadata`, raw file version types (in `storage.rs`)
- Ports/Adapters: `ports::Command`, `ports::Query`, `db_command::Command`, `db_query::Query`
- Loader/orchestration: `loader::Loader`
- Pipeline stages: `RefExpander`, `Extender`, `Resolver`
- Errors: `SchemaError`, `SchemaCommandError`, `SchemaQueryError`, `SchemaIngestionError`
- Events: domain + pipeline events (`events.rs`)

### 1.3 Cross-File Coupling Audit (Initial)

- CQRS split present: `ports.rs` + `db_query.rs` + `db_command.rs`
- Stored read-model types used throughout pipeline (`StoredSchema` in resolver/loader)
- Raw layer contains validation methods (`RawSchema::validate`, `RawPropertyBank::validate`)
- Events system spans domain + pipeline stages
- Context isolation: no `crate::note` or `crate::template` imports found; `crate::config` imports present in `ingestor.rs` (allowed)

---

## 2) Workflow and Pipeline Audit (Behavioral Inventory)

### 2.1 Pipeline Map (Current)

From `loader.rs`:

1) Load existing DB state:
   - `Query::list_name_id_pairs()` -> name->id map
   - `Query::get_property_bank()` -> cached PropertyBank

2) PropertyBank staleness check:
   - `Query::is_bank_stale()` (timestamp check)
   - Reload via `Ingestor::load_raw_property_bank()` if stale

3) Scan schema files from FS:
   - `Ingestor::scan_raw_schemas()` -> Vec<RawSchema + file times + content hash>

4) Schema staleness partitioning:
   - `Query::are_many_stale()` -> O(1) stale check by timestamps
   - Cascade staleness via inheritance

5) Process only stale schemas:
   - `RefExpander::expand_all()`
   - `Extender::build()` -> SchemaTree
   - `Resolver::resolve()` -> Vec<StoredSchema>

6) Persist changes:
   - `Command::save_many()` + `save_inheritance_many()`
   - `Command::save_property_bank()` if stale

**Missing (required by updated architecture):**
- Raw file version cache (`RawSchemaFile` → `RawSchemaView`) and content hash comparison for incremental updates
- Hash-based diff to detect which properties changed (avoid full re-resolution)

### 2.2 Bloat and Inefficiency Checks (Initial)

- CQRS split likely excessive for file-based pipeline (Query/Command ports).
- Domain vs Stored duplication (StoredSchema is primary type; Domain schema type appears removed).
- Raw layer performs validation; may violate "parse, don't validate" if validation returns unit.
- Multi-stage pipeline is heavy; needs review if all stages required for schema.
- Events system may be unnecessary if not consumed (check usage later).
- Staleness detection is timestamp-first only; missing raw-file hash diff and property-level incremental update.

### 2.3 Modularity and Isolation Checks (Initial)

- Parsing dependency isolation: schema parsing in `ingestor` + `raw` (ok).
- Property spec submodule exists (`property_spec/`) but may need raw/spec separation.
- Domain logic uses rkyv + storage types directly (StoredSchema). Evaluate if Domain type should replace Stored.
- Loader orchestrates FS + DB + pipeline (as intended).

---

## 3) Architecture Alignment Audit (Initial)

- [ ] Raw types serde-only? **No** (Raw types include validation methods returning Result<()>).
- [ ] Raw types have parsing helpers to prevent persisting malformed file views? **Partial** (parse is in ingestor; add Raw parsing methods to enforce basic standards before persistence).
- [ ] Domain types validated and used as storage shape? **No** (StoredSchema is primary type; Schema aggregate removed).
- [ ] No Stored/View types unless needed? **No** (StoredSchema + StoredProperty + StoredMetadata are core types).
- [ ] StoredMetadata is ingestion-only metadata? **Yes, but should be removed in favor of raw views.**
- [ ] Unified Repository trait? **No** (CQRS Query/Command ports present).
- [ ] Zero-copy access via `with_archived`? **Present in db layer, verify usage later.**
- [x] Context isolation? **Checked: no note/template imports; config usage allowed.**
- [ ] Naming taxonomy? **Contains `are_many_stale` in ports/query.**

---

## 4) Refactor Targets and Removal Candidates (Initial)

- [ ] CQRS ports (`ports.rs`, `db_query.rs`, `db_command.rs`) -> unify into Repository.
- [ ] StoredSchema -> refactor into `Schema` aggregate in `aggregate.rs`.
- [ ] Move `SchemaId` and `SchemaName` into `aggregate.rs` (remove `id.rs`).
- [ ] StoredProperty -> likely becomes `Property` (unless view-specific fields required).
- [ ] StoredMetadata -> remove (superseded by raw views).
- [ ] StoredMetadata -> remove (superseded by raw views).
- [ ] Stored* types -> move into `views/` and rename to `*View`.
- [ ] Raw* file cache types -> rename `*File` → `*View` and place in `views/`.
- [ ] Raw validation methods -> convert to TryFrom boundary with parsed types.
- [ ] Keep Raw parsing helpers (syntax + basic standards) to avoid persisting invalid raw views.
- [ ] Events system -> verify usage; remove if unused or move to observability only.
- [ ] Migration compat re-export in `mod.rs` -> plan removal.

---

## 5) Proposed Module Structure (Target State) (Draft)

**Note:** Do not collapse modules; retain context-specific files to avoid bloat. Keep `bank.rs`.

```
schema/
├── mod.rs              # Public API + re-exports
├── aggregate.rs        # Schema aggregate + SchemaId/SchemaName
├── raw.rs              # Raw schema input + parsing helpers
├── bank.rs             # PropertyBank (core schema registry)
├── property.rs         # Property, PropertyName, PropertyId
├── property_spec/      # Property spec language (bool/date/file/number/string)
├── views/              # *View structs (Stored* -> *View, Raw*File -> *View)
│   ├── mod.rs
│   └── ...
├── storage.rs          # Repository trait + concrete repository struct
├── ingestor.rs         # FsReader ingestion + raw parsing
├── loader.rs           # Orchestration + staleness logic
├── expander.rs         # Ref expansion stage
├── extender.rs         # Inheritance tree stage
├── resolver.rs         # Merge/resolve stage
└── error.rs            # Error types
```

---

## 6) Target Pipeline Design (Target State) (Draft)

- Canonical pipeline: File → Raw → Domain → Storage
- Multi-phase (schema):
  1) Parse raw (serde)
  2) Raw parsing helpers validate syntax + basic invariants (not semantics)
  3) Resolve property bank references
  4) Build inheritance graph
  5) Resolve properties (merge + excludes)
  6) Populate `Schema.children` during tree build (Extender output)
  7) Persist domain type
- Staleness pipeline (required):
  1) Timestamp fast path
  2) If modified: compare raw file hash with last `RawSchemaView` version
  3) If hash changed: compute property-level diff to avoid full re-resolution
  4) Resolve only impacted properties/schemas
- Validation boundary: TryFrom<Raw*> to Domain
- FsReader use: only in Ingestor/Loader
- No validation inside domain methods
- Zero-copy access: `with_archived` for hot queries
- Views/projections: only if profiling shows need

### 6.1 Domain Shape (Target)

- `Schema` aggregate includes:
  - `recorded_at` (persisted, **private**, no public accessor)
  - `children: Vec<SchemaId>` (populated by Extender)
  - core fields: `id`, `name`, `parent_id`, `properties`

- `PropertyBank` includes:
  - `recorded_at` (persisted, private)
  - `version`, `properties`

### 6.2 Views (Target)

- `RawSchemaView` and `RawPropertyBankView` contain `RawFileVersion` history
- `MetadataView` removed; staleness reads from raw views
- Mapping flow: file path -> SchemaName (TryFrom) -> SchemaId
- Per-property hashes stored in `RawFileVersion` to support incremental rebuilds
- `RawSchemaView` should also persist `extends` and `excludes` for incremental diffs

### 6.3 Target Struct Shapes (Draft)

```rust
// schema/aggregate.rs
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Schema {
    id: SchemaId,
    name: SchemaName,
    parent_id: Option<SchemaId>,
    children: Vec<SchemaId>,
    properties: Vec<Property>,
    recorded_at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaName(Box<str>);

// schema/bank.rs
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyBank {
    properties: BTreeMap<PropertyName, Property>,
    version: BankVersion,
    recorded_at: SystemTime,
}

// schema/views/mod.rs
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    versions: RingBuffer<RawFileVersion, 5>,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBankView {
    versions: RingBuffer<RawFileVersion, 5>,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawFileVersion {
    compressed_content: Vec<u8>,
    content_hash: [u8; 32],
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    recorded_at: SystemTime,
    property_hashes: std::collections::BTreeMap<PropertyName, [u8; 32]>,
}
```

---

## 7) Migration Plan (Ordered Steps) (Pending)

### 7.1 Sequence (Draft)

1) **Define target types and locations**
   - `Schema` aggregate in `aggregate.rs` (move from `StoredSchema`)
   - Move `SchemaId`, `SchemaName` into `aggregate.rs`
   - Remove `StoredMetadata` (superseded by raw views)
   - Convert `Stored*` -> `*View` under `views/`
   - Convert `Raw*File` -> `Raw*View` under `views/`
   - Add `extends`/`excludes` to `RawSchemaView`
   - Add `children` to `Schema` (IDs only)
   - Keep inheritance views with full relationship metadata

2) **Replace CQRS ports with unified Repository**
   - Remove `ports.rs`, `db_query.rs`, `db_command.rs`
   - Create `schema::Repository` trait in `storage.rs`
   - Implement concrete `RedbRepository` in `storage.rs` (or submodule)

3) **Align raw parsing boundary**
   - Add Raw parsing helpers (`RawSchema::parse_*`, `RawPropertyBank::parse_*`)
   - Ensure ingestion uses Raw parsing helpers before persisting raw views

4) **Rewire loader orchestration**
   - Update `Loader` to depend on `Repository` (no CQRS split)
   - Update staleness pipeline to use raw views for diffing
   - Add property-level diffing to avoid full re-resolution

5) **Update persistence and metadata**
   - Rename and relocate view types
   - Update table keys/types as needed for renamed views
   - Ensure metadata view remains ingestion-only

6) **Refactor call sites and tests**
   - Update all imports to `aggregate`, `views`, `storage::Repository`
   - Update tests to new type names and locations

### 7.4 Risks (Draft)

- **Breaking API surface**: Moving `SchemaId/SchemaName` into `aggregate.rs` will break imports.
- **Behavioral regressions**: Incorrect incremental diffing can miss needed re-resolution.
- **Persistence mismatch**: Renaming view types requires table migration or compatibility layer.
- **Performance risk**: Property hashing may increase ingestion cost; benchmark required.
- **Extender complexity**: Populating `children` may introduce cycles or stale child lists if not updated consistently.

### 7.2 Rename/Move Operations (Draft)

- `id.rs` -> remove; move `SchemaId`, `SchemaName` into `aggregate.rs`
- `storage.rs` -> restrict to `Repository` trait + `RedbRepository`
- `StoredSchema` -> `Schema` (aggregate)
- `StoredProperty` -> `Property` (if storage parity)
- `StoredMetadata` -> remove (superseded by raw views)
- `RawSchemaFile` -> `RawSchemaView` (views/)
- `RawPropertyBankFile` -> `RawPropertyBankView` (views/)
- `db_query.rs`, `db_command.rs`, `ports.rs` -> remove
- `views/` new folder with `mod.rs`

### 7.3 Compatibility Shims (Draft)

- Temporary re-exports in `mod.rs` for `SchemaId`, `SchemaName` paths
- Type aliasing for `StoredSchema` -> `Schema` (temporary) if needed for staged migration

---

## 8) Test and Verification Plan (Pending)

### 8.1 Parsing Boundary Tests

- [ ] `RawSchema` parsing helpers reject invalid syntax before persistence
- [ ] `RawPropertyBank` parsing helpers enforce version checks and basic limits

### 8.2 Resolution Correctness Tests

- [ ] RefExpander resolves `$ref` correctly and errors on missing bank entry
- [ ] Extender builds correct inheritance tree (topological order)
- [ ] Resolver merges properties and enforces depth limits

### 8.3 Loader Pipeline Tests

- [ ] Timestamp-only touch does not trigger re-resolution
- [ ] Hash change triggers property-level diff and incremental resolution
- [ ] Raw views persisted on file change (RawSchemaView/RawPropertyBankView)

### 8.4 Repository Tests

- [ ] Unified Repository saves and reads Schema/Raw*View
- [ ] `with_archived` access works for hot paths

### 8.5 Naming Taxonomy Audit

- [ ] Rename `are_many_stale` -> `all_*` or `any_*` per taxonomy
- [ ] Scan for `get_*` getters on simple fields (replace with field name)
- [ ] Scan for `validate_*` returning `Result<()>` in domain layer

---

## 9) Output Deliverables (Pending)

- [x] Context audit report (this document)
- [x] Target module tree diagram
- [x] Gap analysis
- [x] Ordered refactor steps + risks
- [x] Test plan

---

## 11) Gap Analysis (Initial)

### Architecture Conflicts

- CQRS ports still present; must move to unified Repository.
- `StoredSchema` is primary type; must refactor to `Schema` aggregate in `aggregate.rs`.
- `StoredMetadata` is core type; must remove and use raw views instead.
- `Raw*File` types should be renamed to `*View` and relocated to `views/`.
- `storage.rs` currently holds data models; must become Repository trait + concrete repository.

### Pipeline Gaps

- Missing raw-file diffing by comparing against last stored raw view.
- Missing property-level diffing to avoid full re-resolution.
- Staleness pipeline currently uses hash map with current hashes only; needs raw-view comparison and per-property diff.
- Need to add property-level hashing in `RawFileVersion` (shared by schema + bank).

### Module Structure Gaps

- `id.rs` should be removed in favor of `aggregate.rs`.
- `views/` module does not exist yet.
- `db_query.rs`/`db_command.rs` to be removed or folded into repository.

### View Shape Gaps

- `RawSchemaView` should persist `extends`/`excludes` for incremental resolution.
- Keep both: `Schema.children` (IDs only) and inheritance views (full metadata).

---

## 10) Context-Specific Notes (Initial)

- Schema has heavy multi-phase pipeline (expander/extender/resolver).
- Property specs are a distinct language; submodule already exists.
- Staleness detection uses timestamps + content hash.
- CQRS ports still in use; naming `are_many_stale` conflicts with taxonomy.

---

## Target Module Tree Diagram

```
schema/
├── mod.rs
├── aggregate.rs
├── raw.rs
├── bank.rs
├── property.rs
├── property_spec/
│   ├── mod.rs
│   ├── bool.rs
│   ├── date.rs
│   ├── file.rs
│   ├── number.rs
│   └── string.rs
├── views/
│   ├── mod.rs
│   ├── raw.rs
│   └── inheritance.rs
├── storage.rs
├── ingestor.rs
├── loader.rs
├── expander.rs
├── extender.rs
├── resolver.rs
└── error.rs
```

---

## 12) View Simplification Opportunities (New)

### 12.1 Replace StoredMetadata with Raw Views

- Proposal: Remove `StoredMetadata`; rely on `RawSchemaView` and `RawPropertyBankView` for staleness checks.
- Rationale: `RawFileVersion` already contains hash + timestamps + recorded_at; metadata duplication is avoidable.

### 12.2 Add recorded_at to Domain Types

- Proposal: Persist `recorded_at` on `Schema` and `PropertyBank` (possibly `Property`) to reduce separate view structs.
- Result: Stored* structs can be minimized or eliminated if domain types carry required ingestion timing metadata.

### 12.3 SchemaTree Simplification via Views

- Option A: Use `ChildSchemaView`/`ParentSchemaView`-style views to reconstruct inheritance without a separate tree structure.
- Option B: Add `children` attribute to `Schema` aggregate and persist it (stores child IDs only).
- Decision: Keep both. `Schema.children` stores child IDs for fast read paths; inheritance views store full relationship metadata.

### 12.4 Raw Schema Views with Extends/Excludes

- Proposal: Persist `RawSchemaView` with explicit `extends` + `excludes` to aid incremental updates and reduce recomputation.
