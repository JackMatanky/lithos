---
feature: Schema Models (Aggregates + Value Objects)
status: Draft
author: Jack Matanky (drafted with GitHub Copilot)
ticket: TBD
date_created: 2026-02-03
date_updated: 2026-02-12
tags: [schema, domain-models, type-driven-design, rkyv, performance, incremental-resolution]
---

# Tech Spec: Schema Models (Aggregates + Value Objects)

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The schema bounded context is responsible for:

- representing schemas (names, inheritance relationships, property sets),
- representing reusable property definitions (property bank),
- validating metadata values at runtime,
- **incremental resolution and staleness detection** (minimize re-resolution on subsequent runs).

Current implementation lives under:

- `lithos-core/src/schema/aggregate.rs` (`Schema`, `SchemaName`, `PropertyBank`)
- `lithos-core/src/schema/property.rs` (`Property`, `PropertyName`)
- `lithos-core/src/schema/property_spec.rs` (`PropertySpecDef`, `PropertySpec`, type-driven invariant helpers)
- `lithos-core/src/schema/raw.rs` (`RawSchema`, `RawProperty*`)
- `lithos-core/src/schema/resolver.rs` (resolution logic)
- `lithos-core/src/schema/graph.rs` (inheritance graph)

The system aims to be lean and performant, with a strong zero-copy inclination.

**Resolution Strategy**: Schemas are resolved once at first run and stored with **resolution metadata** for staleness detection. On subsequent runs, only changed schemas (based on PropertyBank version, parent schema hashes, or definition file changes) are re-resolved. This mirrors the note indexing strategy: **minimize expensive computation, maximize read speed**.

**PropertyBank Role**: The PropertyBank acts as a **registry/lookup service**, NOT a traditional aggregate with dual ownership. It is **always loaded first** at program start before any schema resolution. After schemas are resolved, the system rarely needs to access the PropertyBank (schemas are self-contained snapshots).

**Storage Key Strategy**: Schemas use **UUID-first storage** (`SchemaId` as primary key) with **name→id index** for user lookups. This enables schema renames without data rewrites and maintains consistency with the note context identity model.

Key constraints from the overall architecture:

- redb is transaction-scoped and returns guard-based views of values.
- rkyv enables zero-copy access to archived data, but only after validation at trust boundaries.
- changing rkyv "format control" choices or archived model layout can be a breaking on-disk format change.

### 1.2 Goals & Non-Goals

**Goals**

- Make schema model invariants unforgeable via type-driven design.
- Enable **incremental resolution**: track resolution dependencies and minimize re-resolution work.
- Clarify and separate the three "shapes" of data:
  - **wire/input** (serde DTO-ish; may be invalid) — `Raw*` types,
  - **validated runtime/domain** (invariants enforced) — domain types,
  - **archived/persisted** (rkyv, read via redb guards).
- Keep runtime validation ergonomic while preserving performance:
  - avoid stringly-typed identifiers,
  - prefer `Box<str>` / borrowed views over `String` cloning.
- **UUID-first storage** with name indexing for stable identity.

**Non-Goals**

- Redesigning the storage layer (`db.rs`) in this spec (see 009-schema-cqrs.md).
- Introducing new dependencies.
- Implementing the refactor immediately; this document defines the design and migration plan.
- Supporting runtime schema refresh without restart (LSP concern for future).

### 1.3 Constraints (The Hard Limits)

- **Zero-copy boundaries**: rkyv validation (bytecheck / `rkyv::access`) occurs at trust boundaries before archived data is used.
- **redb guard lifetimes**: do not return `AccessGuard` or any reference derived from it beyond the transaction scope; closure-based APIs are preferred.
- **Sync-first core**: schema model construction/validation remains synchronous.
- **Lean models**: avoid "stringly typed" keys and avoid hidden allocations.
- **API clarity**: prefer borrowed argument and accessor types (e.g. `&str` rather than `&String`) and keep allocation decisions explicit (see https://rust-analyzer.github.io/book/contributing/style.html).
- **Incremental resolution**: Resolution must be incremental to avoid re-processing unchanged schemas on every run.

### 1.4 Definition of Done

- The schema bounded context has a clearly documented model boundary:
  - persisted (serde-friendly) input types (`Raw*`),
  - validated runtime/domain types (newtypes with private fields),
  - archived/persistence representation (rkyv).
- All model invariants are enforced by construction (type-driven), not by convention.
- Model types are lean and idiomatic:
  - borrowed inputs where possible,
  - no hidden allocations in getters,
  - minimal cloning in hot paths.
- **Resolution metadata types defined** for staleness detection (parent hashes, PropertyBank version, timestamps).
- **Semantic enums** replace boolean blindness (`Cardinality`, `Multiplicity`).
- Zero-copy constraints are explicitly documented for redb + rkyv (validation, lifetime/guard scope).
- Any proposed breaking changes include an explicit migration strategy.

### 1.5 Minimizing "derive-everything" Blast Radius (rkyv)

Schemas and property banks are attractive to archive "as-is", but large rkyv derive surfaces create a maintenance hazard: small model refactors can silently become **persisted-format changes**.

Guidance:

- Prefer isolating rkyv derives onto **persistence DTOs** (storage-layer types) when it reduces coupling. Domain types stay ergonomic; persisted types stay stable.
- Keep archived compute closure-based and local to the storage/query tier; do not leak archived references outside transaction scope.
- Treat any change to archived layout, rkyv attributes, or format-control feature set as a migration decision.
- Introduce projections for hot queries rather than forcing the primary persisted schema blob to satisfy every read shape.

### 1.6 Raw → Domain boundary (designing to avoid `Stored*` models)

The primary lever for avoiding `StoredSchema`/`StoredPropertyBank` is _not_ introducing a storage DTO early; it is designing the **validated domain types** to be both:

- ergonomic for business logic, and
- reasonably rkyv-friendly for persistence and archived reads.

Recommended boundary discipline:

1. **Raw (wire/input)** types (`RawSchema`, `RawProperty*`) stay serde-friendly and error-reporting friendly.

   - Use `String`, `Option`, and "tolerant shapes" to capture good diagnostics.
   - Keep parsing/format concerns (like `$ref` syntax) in adapters or raw parsing helpers.

2. **Domain** types (`Schema`, `Property`, `PropertySpec`, `SchemaName`, `PropertyName`, ids/newtypes) enforce invariants and choose lean representations.

   - Prefer `Box<str>` identifiers and private fields.
   - Prefer semantic enums over boolean flags.
   - Avoid domain shapes that are known to create persistence friction (e.g., high-churn `HashMap<String, ...>` in the persisted aggregate) unless there is a demonstrated need.

3. **Persistence** stores domain types directly by default.

   - If the domain shape later proves inefficient (profiling/benchmarks), introduce `Stored*` types as an explicit optimization/migration decision.

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

A schema author defines schemas and properties using serde-friendly definitions (YAML/JSON/TOML). The system compiles those definitions into validated runtime types.

**Workflow**:

1. **First run**: Parse raw schema input into `RawSchema` / `RawProperty*` (wire types) → Compile/validate into runtime types (`Schema`, `Property`, `PropertySpec`) → Persist with resolution metadata.
2. **Subsequent runs**: Check staleness (PropertyBank version, parent hashes, file timestamps) → Re-resolve only changed schemas → Load unchanged schemas from DB.
3. **Runtime validation**: Use `Property::validate_value(&serde_json::Value)` to validate metadata values against resolved schemas.

**Key invariants**:

- PropertyBank is loaded **before** any schema resolution.
- Schemas are **fully resolved snapshots** (no runtime resolution needed).
- Staleness detection is **hash-based** (content-addressed, not timestamp-only).

### 2.2 Mental Model

- **Names and IDs are distinct types**.
  - A `SchemaName` is a validated identifier for "human named" schema references.
  - A `SchemaId` is the stable identity stored as a UUIDv7 (primary key).
  - Users look up schemas by name; system translates to ID internally.
- **Property definitions are reusable**.
  - A `PropertyBank` stores canonical property definitions (loaded first, acts as registry).
  - A `Schema` embeds fully resolved properties (self-contained for validation).
- **Resolution is incremental**.
  - First run: resolve all schemas.
  - Subsequent runs: re-resolve only when PropertyBank, parent schema, or definition file changes.
- **Persisted bytes are not trusted**.
  - redb provides bytes; rkyv validation produces an archived view; callers compute results in a closure.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

```mermaid
flowchart LR
  Raw[RawSchema / RawProperty (serde)] --> Compile[Compile + Validate]
  Compile --> Domain[Schema / Property / PropertySpec]
  Domain --> Metadata[Add Resolution Metadata]
  Metadata --> Persist[(redb + rkyv bytes)]
  Persist --> Staleness[Staleness Check on next run]
  Staleness --> |Changed| Compile
  Staleness --> |Unchanged| Read[zero-copy read via closure]
  Domain --> Validate[validate metadata values]
```

**Key flow**:

1. **Load PropertyBank** (first, always)
2. **Check staleness** for each schema (compare metadata)
3. **Re-resolve changed** schemas only
4. **Load unchanged** schemas from DB
5. **Persist** newly resolved schemas with updated metadata

### 3.2 Data Models

This section defines all types used in the schema context, organized by layer.

#### Wire/Input Layer (`Raw*` types)

Raw types are serde-friendly and may be invalid. They capture user input before validation.

- **`RawSchema`** (Raw/Input, serde)
  - Purpose: User-provided schema definition from YAML/JSON/TOML.
  - Rules: May have invalid names, missing fields, or malformed `$ref` syntax.
  - Notes: Compiled into `Schema` via resolver.

- **`RawPropertyInline`** / **`RawPropertyRef`** (Raw/Input, serde)
  - Purpose: Property definitions in raw schema (inline or `$ref`).
  - Rules: May reference non-existent properties or have invalid specs.
  - Notes: Compiled into `Property` via resolver.

**Rule**: Wire types may use `String` and "raw" options. Wire types must be converted into domain types as early as possible.

#### Validated Runtime/Domain Layer

Domain types enforce invariants by construction (private fields, validated constructors).

##### Core Aggregates

**`Schema`** (Domain)

- **Purpose**: Fully resolved schema used for metadata validation.
- **Key rules**:
  - Unique property names (within schema).
  - Deterministic property order (sorted by name).
  - Fully resolved (no `$ref` references remain).
- **Resolution metadata**: Tracks parent hash, PropertyBank version, and resolution timestamp for staleness detection.
- **Shape**:

```rust
pub struct Schema {
    id: SchemaId,               // UUID v7 (primary key)
    name: SchemaName,           // Private field (validated)
    properties: Vec<Property>,  // Fully resolved, self-contained

    // Resolution metadata (for staleness detection)
    resolved_at: Timestamp,           // When resolution occurred
    parent_hash: Option<SchemaHash>,  // Hash of parent schema (if inherited)
    bank_version: BankVersion,        // PropertyBank version at resolution time
}
```

**`PropertyBank`** (Domain, Registry)

- **Purpose**: Registry for reusable property definitions (NOT a traditional aggregate).
- **Key rules**:
  - Unique `PropertyId` and `PropertyName`.
  - Loaded **first**, before any schema resolution.
  - Versioned for staleness detection (`bank_version` increments on change).
- **Lookup strategy**: Dual indices (`HashMap<PropertyId, usize>` and `HashMap<PropertyName, usize>`).
- **Shape**:

```rust
pub struct PropertyBank {
    properties: Vec<Property>,
    by_id: HashMap<PropertyId, usize>,     // Typed keys (not String!)
    by_name: HashMap<PropertyName, usize>, // Typed keys (not String!)
    version: BankVersion,                  // Type-safe versioning (increments on change)
}
```

**Design note**: `PropertyBank::decode(&str)` for `$ref` parsing is adapter-level; domain-level lookup accepts `PropertyId` or `PropertyName` directly.

**`Property`** (Domain)

- **Purpose**: Validated property definition used for runtime metadata validation.
- **Key rules**:
  - Semantic enums for shape (no boolean blindness).
  - Private fields (id, name cannot be forged).
- **Shape**:

```rust
pub struct Property {
    id: PropertyId,              // Private field
    name: PropertyName,          // Private field (validated)
    cardinality: Cardinality,    // Optional | Required (NOT bool!)
    multiplicity: Multiplicity,  // Single | Many (NOT bool!)
    spec: PropertySpec,          // Validated
}

// Semantic enums (avoid boolean blindness)
pub enum Cardinality { Optional, Required }
pub enum Multiplicity { Single, Many }
```

**Rationale**: Two booleans (`required`, `array`) are error-prone (boolean blindness). Semantic enums improve readability and prevent invalid combinations.

##### Resolution Metadata Types

**`ResolutionMetadata`** (Domain)

- **Purpose**: Track resolution dependencies for staleness detection.
- **Key rules**:
  - Stored alongside each schema (separate table for fast staleness checks).
  - Used to determine if re-resolution is needed without loading full schema.
  - All fields use typed newtypes (no raw primitives).
- **Shape**:

```rust
pub struct ResolutionMetadata {
    schema_id: SchemaId,                    // Schema this metadata applies to
    resolved_at: Timestamp,                 // When resolution occurred
    parent_hash: Option<SchemaHash>,        // Hash of parent schema content (if inherited)
    bank_version: BankVersion,              // PropertyBank version at resolution time
    file_modified: Option<Timestamp>,       // File mtime (if sourced from file)
}
```

**Staleness triggers** (re-resolve if any is true):

1. `bank_version.is_older_than(current_bank_version)` - PropertyBank changed
2. `parent_hash != Some(SchemaHash::compute(parent))` - Parent schema content changed
3. `file_modified.map_or(false, |t| t < current_file_mtime)` - Definition file modified

**Type safety benefits**:

- Can't mix version with hash: `BankVersion` vs `SchemaHash` are distinct types.
- Can't mix timestamps with offsets: `Timestamp` prevents confusion.
- Clear intent: `bank_version.is_older_than()` vs `bank_version < current` is more readable.

##### Validated Identifiers (Newtypes)

All identifiers have **private fields** to enforce invariants by construction.

**`SchemaId`** (Domain, `Uuid`)

- **Purpose**: Stable schema identity (primary key) for UUID-first storage.
- **Backing**: `Uuid` (16 bytes, UUID v7)
- **Rules**: Always valid UUID v7.
- **Key traits**: `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`
- **Notes**: Copy-cheap; prevents mixing with PropertyId. Provide `new()` (generates UUID v7), `from_uuid(Uuid)`, `as_uuid(&self) -> &Uuid`.

**`PropertyId`** (Domain, `Uuid`)

- **Purpose**: Stable property identity for type-safe property references.
- **Backing**: `Uuid` (16 bytes, UUID v7)
- **Rules**: Always valid UUID v7.
- **Key traits**: `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`
- **Notes**: Copy-cheap; prevents accidental mixing with SchemaId. Provide `new()`, `from_uuid(Uuid)`, `as_uuid(&self)`.

**`SchemaName`** (Domain, `Box<str>`)

- **Purpose**: Validated schema name for user references (display, lookup).
- **Backing**: `Box<str>` (lean for immutable identifiers)
- **Rules**: Non-empty, <= 64 bytes, matches `patterns::ALPHANUMERIC_NAME` (lowercase alphanumeric + hyphens/underscores).
- **Key traits**: `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`, `Display`, `AsRef<str>`, `Borrow<str>`
- **Notes**: **Private field** (must use `try_from(&str)` constructor). Provide `as_str(&self) -> &str` accessor. **NO `Deref<Target=str>`** (use explicit `as_str()` per Rust API Guidelines).

**`PropertyName`** (Domain, `Box<str>`)

- **Purpose**: Validated property name for type-safe property references.
- **Backing**: `Box<str>` (lean for immutable identifiers)
- **Rules**: Non-empty, <= 64 bytes, matches `patterns::ALPHANUMERIC_NAME`.
- **Key traits**: `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`, `Display`, `AsRef<str>`, `Borrow<str>`
- **Notes**: **Private field** (must use `try_from(&str)` constructor). Provide `as_str(&self) -> &str`. Prevents mixing with SchemaName. **NO `Deref<Target=str>`**.

**`SchemaNameKey`** (Domain, `Box<str>`)

- **Purpose**: Normalized schema name for storage indexing (lowercase, canonical form).
- **Backing**: `Box<str>` (storage-layer key)
- **Rules**: Derived from `SchemaName` via `.to_lowercase()` normalization.
- **Key traits**: `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`
- **Notes**: Used only in storage layer (name→id index). Prevents mixing normalized keys with user-facing SchemaName. Provide `From<&SchemaName>`, `as_str(&self) -> &str`.

**`BankVersion`** (Domain, `u64`)

- **Purpose**: PropertyBank version for incremental resolution staleness detection.
- **Backing**: `u64` (monotonically increasing counter)
- **Rules**: Increments on any property add/update/remove in PropertyBank.
- **Key traits**: `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Debug`, `Display`
- **Notes**: Prevents mixing with arbitrary `u64` values (timestamps, hashes, counts). Provide `initial() -> Self` (returns 0), `increment(self) -> Self`, `as_u64(self) -> u64`, `is_older_than(self, other: Self) -> bool`. Type-safe staleness checks: `meta.bank_version.is_older_than(current_bank_version)`.

**`SchemaHash`** (Domain, `u64`)

- **Purpose**: Content hash for schema staleness detection (parent change tracking).
- **Backing**: `u64` (computed hash, e.g., `std::hash::Hash` or xxhash)
- **Rules**: Computed from schema content (name, properties); excludes metadata (timestamps, IDs).
- **Key traits**: `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Debug`
- **Notes**: Prevents mixing hashes with versions, timestamps, IDs. Provide `compute(schema: &Schema) -> Self`, `as_u64(self) -> u64`. Type-safe comparisons: `meta.parent_hash != SchemaHash::compute(parent)`. Future-proof: can switch hash algorithm without changing call sites.

**`Timestamp`** (Domain, `i64`)

- **Purpose**: Unix timestamp (seconds since epoch) for resolution tracking.
- **Backing**: `i64` (Unix timestamp)
- **Rules**: Unix timestamp (positive or negative for dates before 1970).
- **Key traits**: `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Debug`
- **Notes**: Prevents mixing timestamps with other `i64` values (offsets, counts, byte positions). Provide `now() -> Self` (via `chrono::Utc::now().timestamp()`), `from_secs(i64) -> Self`, `as_secs(self) -> i64`. Used for `resolved_at` in `ResolutionMetadata`. Could add methods like `.elapsed()`, `.is_before()` later.

**Type-driven design rules** (apply to all newtypes):

- All inner fields MUST be **private** so callers cannot forge invalid values.
- Provide explicit constructors (`new()`, `try_from()`, `from_*()`) and accessors (`as_*()`, `into_*()`).
- Implement relevant traits (`Display`, `AsRef<str>`, `Borrow<str>` for string types; `PartialOrd`, `Ord` for versioned/timestamped types).
- **NO `Deref`** on domain newtypes: Use explicit accessors (`as_str()`, `as_u64()`) to avoid surprising method resolution (Rust API Guidelines).

**Rationale**:

- Private fields enforce invariants by construction (Rust best practice).
- Newtypes prevent mixing conceptually distinct values (version vs hash, timestamp vs offset).
- Type-safe operations: `BankVersion::increment()` vs `version + 1` documents intent.
- `Box<str>` is lean for immutable identifiers (avoids excess capacity of `String`).
- Explicit accessors are clearer than implicit `Deref` coercion for domain newtypes.

#### Archived/Persistence Layer

- Archived types are derived from the domain types via rkyv derives.
- **Rule**: Do not expose archived references outside a closure that is scoped to the redb transaction/guard.
- **Rule**: Treat any change to archived layout as a migration event (documented + tested).

Storage is detailed in `009-schema-cqrs.md`.

### 3.3 Component & Interface Specifications

This section defines the contracts for key components.

#### Component: `SchemaName` (validated identifier)

- **Responsibility**: Represent a validated schema identifier that is safe for indexing/lookup and comparisons.
- **Invariants**:
  - Non-empty
  - Max length 64 bytes
  - Matches `patterns::ALPHANUMERIC_NAME` (lowercase alphanumeric + hyphens/underscores)
- **Public Interface** (target):
  - `SchemaName::try_from(&str) -> Result<SchemaName, SchemaError>`
  - `as_str(&self) -> &str`
  - Implements `Display`, `AsRef<str>`, `Borrow<str>`
- **Representation**:

```rust
pub struct SchemaName(Box<str>); // Private field!
```

**Type-driven design rule**: The inner field MUST be private so callers cannot forge invalid values.

**Rationale** (Rust API Guidelines + Lithos style rules):

- Private fields enforce invariants by construction.
- `Box<str>` is lean for immutable identifiers.
- Avoid `Deref<Target=str>` on domain newtypes. For domain identifiers, prefer explicit `as_str()`.
  - Follows Rust API Guidelines' `Deref` guidance: https://rust-lang.github.io/api-guidelines/checklist.html

#### Component: `PropertyName` (validated identifier)

Same intent and rules as `SchemaName`, with its own error variants.

#### Component: `SchemaId` and `PropertyId`

- **Responsibility**: Prevent accidental mixing of UUIDs across concepts (type safety).
- **Representation**:

```rust
pub struct SchemaId(Uuid);   // Private field
pub struct PropertyId(Uuid); // Private field
```

- **Ergonomics**:
  - `Copy` is fine (Uuid is Copy).
  - Provide `new()` (generates UUID v7), `from_uuid(Uuid)`, `as_uuid(&self) -> &Uuid`.

#### Component: `Property` (reusable definition)

- **Responsibility**: A validated definition used for runtime metadata validation.
- **State**:
  - `id: PropertyId`
  - `name: PropertyName`
  - `cardinality: Cardinality` (Optional | Required)
  - `multiplicity: Multiplicity` (Single | Many)
  - `spec: PropertySpec` (validated)
- **Public Interface**:
  - `new(id, name, cardinality, multiplicity, spec) -> Self`
  - `validate_value(&self, value: &serde_json::Value) -> Result<(), ValidationError>`
  - Getters: `id()`, `name()`, `cardinality()`, `multiplicity()`, `spec()`.

**Type-driven improvement (REQUIRED)**: Replace two booleans (`required`, `array`) with semantic enums:

```rust
pub enum Cardinality { Optional, Required }
pub enum Multiplicity { Single, Many }
```

This improves readability and reduces mistakes at call sites (Rust API Guidelines: avoid multiple boolean parameters).

#### Component: `PropertyBank` (registry)

- **Responsibility**: Registry for reusable property definitions and fast lookup. Acts as a **source of truth**, loaded first before any schema resolution.
- **Invariants**:
  - Unique `PropertyId`
  - Unique `PropertyName`
  - Version increments on any change (add/update/remove property)
- **Lookup strategy**:
  - Keep dual indices for O(1) lookups: `HashMap<PropertyId, usize>` and `HashMap<PropertyName, usize>`
  - Use **typed keys** (not `String`!)
- **Public Interface**:
  - `new(properties: Vec<Property>) -> Result<Self, SchemaError>`
  - `get(&self, id: PropertyId) -> Option<&Property>`
  - `get_by_name(&self, name: &PropertyName) -> Option<&Property>`
  - `version(&self) -> u64`
- **Lifecycle**:
  - Loaded **first** at program start.
  - After schema resolution, system rarely needs PropertyBank (schemas are self-contained).

**Design note**: PropertyBank is a **registry, NOT an aggregate with dual ownership**. It's a lookup service, not a persistent entity that schemas reference at runtime.

#### Component: `Schema` (aggregate)

- **Responsibility**: Represent a fully resolved schema used for validation. Self-contained (no runtime lookups needed).
- **Invariants**:
  - Unique property names (within the schema)
  - Deterministic property order (sorted by name for stable output)
  - Fully resolved (no `$ref` references remain)
- **Resolution metadata**: Tracks dependencies for staleness detection.
- **Public Interface**:
  - `new(id, name, properties, metadata) -> Self`
  - `id(&self) -> SchemaId`
  - `name(&self) -> &SchemaName`
  - `properties(&self) -> &[Property]`
  - `metadata(&self) -> &ResolutionMetadata`
  - `validate_metadata(&self, metadata: &HashMap<PropertyName, serde_json::Value>) -> Result<(), ValidationError>`

**Options for representing properties**:

1. **Embedded properties** (current direction)
   - `Schema { properties: Vec<Property> }`
   - Pros: Simple; schema is self-contained for validation; no runtime lookups.
   - Cons: Duplicates property defs across schemas; more bytes persisted.

2. **Referential properties** (future optimization)
   - `Schema { properties: Vec<PropertyId> }` with `PropertyBank` as dependency
   - Pros: Better dedup; smaller persisted schema.
   - Cons: Validation becomes a two-step lookup; need strong coherence rules; PropertyBank must be loaded at validation time.

**Decision**: Keep embedded properties for now (simplicity, self-contained validation). Design IDs/newtypes so option (2) remains viable in the future.

#### Component: `ResolutionMetadata` (staleness tracking)

- **Responsibility**: Track resolution dependencies for incremental re-resolution.
- **State**:
  - `schema_id: SchemaId`
  - `resolved_at: i64` (Unix timestamp)
  - `parent_hash: Option<u64>` (hash of parent schema content, if inherited)
  - `bank_version: u64` (PropertyBank version at resolution time)
  - `file_modified: Option<i64>` (file mtime, if sourced from file)
- **Staleness check algorithm**:

```rust
fn is_stale(
    &self,
    current_bank_version: BankVersion,
    current_parent_hash: Option<SchemaHash>,
    current_file_mtime: Option<Timestamp>,
) -> bool {
    // Re-resolve if:
    self.bank_version.is_older_than(current_bank_version)        // PropertyBank changed
    || self.parent_hash != current_parent_hash                   // Parent schema changed
    || self.file_modified.map_or(false, |stored_mtime| {
        current_file_mtime.map_or(false, |current| stored_mtime < current)
    })                                                           // Definition file changed
}
```

### 3.4 Integration & Data Flow

**Startup flow** (incremental resolution):

1. **Load PropertyBank** from DB (or parse from definitions if first run).
2. **Load schema metadata** for all schemas.
3. **Check staleness** for each schema:
   - If stale: parse `RawSchema`, resolve, persist with new metadata.
   - If fresh: load from DB (zero-copy).
4. **Schema resolution available** for metadata validation.

**Resolution flow** (detailed in `010-schema-graph-resolver.md`):

1. Parse `RawSchema` from YAML/JSON/TOML.
2. Build dependency graph (inheritance, `$ref` to PropertyBank).
3. Topologically sort schemas (resolve parents before children).
4. Resolve each schema:
   - Inherit parent properties.
   - Resolve `$ref` to PropertyBank.
   - Merge and validate property names (no duplicates).
   - Sort properties by name (deterministic order).
5. Compute resolution metadata (hash parent, record PropertyBank version).
6. Persist `Schema` + `ResolutionMetadata`.

**Storage layer** (detailed in `009-schema-cqrs.md`):

- Primary table: `schema_by_id: SchemaId → ArchivedSchema`
- Index table: `schema_id_by_name: SchemaNameKey → SchemaId`
- Metadata table: `schema_metadata: SchemaId → ResolutionMetadata`

### 3.5 Core Logic & Algorithms

#### Validation

- Validation belongs in **constructors** and in `PropertySpec` validators.
- Avoid "validate() that does nothing"; invariants should be either:
  - enforced by construction (private fields, validated constructors), or
  - explicitly tested and documented (e.g., property name uniqueness within schema).

#### Staleness Detection Algorithm

See `ResolutionMetadata::is_stale()` above. Key points:

- **Hash-based** (content-addressed), not just timestamp-based.
- **Three triggers**: PropertyBank version bump, parent schema content change, definition file mtime change.
- **Incremental**: Only re-resolve changed schemas, not entire graph.

#### Property Order Normalization

- Properties within a schema are **sorted by name** for deterministic output.
- Sorting happens during resolution (after merging inherited properties).
- Ensures stable serialization and comparisons.

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Private fields for validated identifiers

- **Choice**: `SchemaName` and `PropertyName` have private inner fields.
- **Why**: Prevents invalid construction; makes invariants enforceable; aligns with Rust best practices.
- **Alternative**: Keep tuple structs with `pub String` (rejected; breaks type-driven invariants; allows forging invalid values).

#### Decision: Prefer `as_str()` over `Deref<Target=str>` for identifiers

- **Choice**: `as_str()` explicit accessor.
- **Why**: Keeps APIs explicit and avoids surprising method resolution; aligns with "useful types" and ownership clarity.
- **Alternative**: `Deref`-based implicit conversion (rejected for domain newtypes per Rust API Guidelines).

#### Decision: Semantic enums for Property shape (Cardinality, Multiplicity)

- **Choice**: Replace `required: bool` and `array: bool` with `cardinality: Cardinality` and `multiplicity: Multiplicity`.
- **Why**: Avoids boolean blindness; improves readability; prevents invalid combinations.
- **Alternative**: Keep two booleans (rejected; error-prone at call sites).

#### Decision: Embedded properties in Schema (not referential)

- **Choice**: `Schema { properties: Vec<Property> }` (fully resolved, self-contained).
- **Why**: Simplifies validation (no runtime lookups); schemas are self-contained after resolution.
- **Alternative**: `Schema { properties: Vec<PropertyId> }` with PropertyBank as runtime dependency (rejected for now; adds complexity; requires PropertyBank at validation time).
- **Future**: Referential properties remain viable if profiling shows memory/storage pressure.

#### Decision: UUID-first storage with name index

- **Choice**: `SchemaId` (UUID v7) as primary key; `SchemaName` → `SchemaId` index for user lookups.
- **Why**: Enables schema renames without data rewrites; consistent with note context; stable identity.
- **Alternative**: Name-based primary key (rejected; renames require expensive rewrites; fragile for references).

#### Decision: Incremental resolution with staleness detection

- **Choice**: Resolve once, re-resolve only on change (PropertyBank version, parent hash, file mtime).
- **Why**: Minimizes expensive resolution work on subsequent runs (like note indexing); improves startup time.
- **Alternative**: Re-resolve all schemas on every run (rejected; slow for large schema sets; wasteful computation).

#### Decision: PropertyBank as registry (not aggregate)

- **Choice**: PropertyBank is a **registry/lookup service**, loaded first, rarely accessed after resolution.
- **Why**: Clarifies lifecycle (load once, use during resolution, schemas self-contained after); avoids dual ownership confusion.
- **Alternative**: PropertyBank as aggregate with runtime references from schemas (rejected; adds complexity; requires PropertyBank at validation time).

#### Decision: ResolutionMetadata stored separately

- **Choice**: Store `ResolutionMetadata` in separate table (not embedded in `Schema`).
- **Why**: Enables fast staleness checks without deserializing full schema; cleaner separation of concerns.
- **Alternative**: Embed in `Schema` (rejected; forces full schema deserialization for staleness checks).

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

- Model construction and validation is deterministic and should not emit logs by default.
- Higher layers (CQRS/adapters) should instrument operations:
  - Resolution time per schema (for performance tracking).
  - Staleness check results (how many schemas re-resolved vs loaded from DB).
  - PropertyBank version changes (triggers broad re-resolution).
- Avoid logging full metadata payloads (can be large).

### 5.2 Migration Strategy

Incremental, low-risk migration path:

1. **Phase 1: Type safety** (no storage changes)
   - Introduce newtypes (`SchemaId`, `PropertyId`) and private-field identifiers (`SchemaName`, `PropertyName`).
   - Provide fallible conversions from existing strings (`TryFrom<&str>`, `TryFrom<String>`).
   - Gradually replace `HashMap<String, …>` with typed keys.

2. **Phase 2: Semantic enums** (no storage changes)
   - Introduce `Cardinality` and `Multiplicity` enums.
   - Update `Property` construction and validation logic.
   - Update tests and usage sites.

3. **Phase 3: Resolution metadata** (storage migration)
   - Add `ResolutionMetadata` type and storage table.
   - Compute metadata during resolution.
   - Persist alongside schemas.
   - **Migration**: On first run with new code, recompute metadata for all existing schemas.

4. **Phase 4: Incremental resolution** (behavior change)
   - Implement staleness detection logic.
   - Update startup flow to check metadata and re-resolve only changed schemas.
   - Measure performance improvement (benchmark startup time with 100+ schemas).

5. **Phase 5: UUID-first storage** (storage migration)
   - Update CQRS ports and storage keys (see `009-schema-cqrs.md`).
   - Migrate existing name-keyed data to UUID-keyed with name index.
   - **Migration**: Generate UUIDs for existing schemas; build name→id index; verify data integrity.

### 5.3 Security & Privacy

- All persisted bytes are treated as untrusted input and must be validated (rkyv safe access) before use.
- `$ref` parsing belongs in adapters; domain logic should operate on typed identifiers (prevents injection attacks).
- Schema names are user-controlled; enforce length limits and character set restrictions (alphanumeric + hyphens/underscores).

## 6. Pre-Mortem (The "Inversion")

Assume it is 6 months from now and this system failed. Why?

- **Risk**: "Validated" name types can be forged, leading to inconsistent indexing and hard-to-debug behavior.
  - _Mitigation_: Private fields + constructors only; no public tuple struct fields; enforcement via code review.

- **Risk**: Archived references escape transaction scope, causing use-after-free / UB.
  - _Mitigation_: Closure-based query APIs; never return guards; lifetime bounds on port traits; static analysis via clippy.

- **Risk**: On-disk format breaks due to changing archived layouts.
  - _Mitigation_: Treat rkyv format control and archived model changes as migration events (documented + tested); versioned storage; ADR for any rkyv attribute changes.

- **Risk**: Staleness detection false negatives (missed changes, stale schemas served).
  - _Mitigation_: Hash parent content (not just timestamp); PropertyBank versioning; file mtime tracking; integration tests for staleness scenarios.

- **Risk**: Staleness detection false positives (unnecessary re-resolution, slow startup).
  - _Mitigation_: Precise hash computation (exclude metadata fields); stable property ordering; idempotent resolution (same input → same output).

- **Risk**: PropertyBank version changes trigger expensive full re-resolution of all schemas.
  - _Mitigation_: Track **which properties changed** in PropertyBank (future optimization); for now, accept full re-resolution as rare event (PropertyBank stable in production).

- **Risk**: Incremental resolution adds complexity without measurable benefit.
  - _Mitigation_: Benchmark before and after; document performance improvement; measure with realistic schema sets (100+ schemas); if no benefit, revert to simpler "always resolve" strategy.

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                   | Resolution                                                                                   |
| :--------- | :------------------------------------------------- | :------------------------------------------------------------------------------------------- |
| 2026-02-03 | Tuple struct identifiers are forgeable             | Require private fields; keep `try_from` and `as_str` accessors                               |
| 2026-02-03 | Booleans for property shape are easy to misuse     | Recommend semantic enums `Cardinality`/`Multiplicity`                                         |
| 2026-02-03 | Zero-copy lifetime risks not explicit in models    | Document closure-based access and guard scoping                                               |
| 2026-02-12 | Missing incremental resolution strategy            | Add `ResolutionMetadata` type; hash-based staleness detection; PropertyBank versioning        |
| 2026-02-12 | PropertyBank role unclear (aggregate vs registry)  | Clarify: PropertyBank is registry/lookup service, loaded first, schemas self-contained after  |
| 2026-02-12 | Storage key strategy not defined                   | UUID-first with name index; enables renames; consistent with note context                     |
| 2026-02-12 | Boolean blindness in Property (required, array)    | Replace with semantic enums: `Cardinality`, `Multiplicity`                                    |
| 2026-02-12 | Lack of staleness detection mechanism              | Three-trigger staleness: PropertyBank version, parent hash, file mtime                        |
| 2026-02-12 | Embedding vs referential properties unclear        | Embedded for now (simplicity); referential remains viable future optimization                 |
| 2026-02-12 | Raw primitives in metadata (u64, i64) cause confusion | Add newtypes: `BankVersion`, `SchemaHash`, `Timestamp` for type safety and clear intent    |

## 8. References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Analyzer Style Guide](https://rust-analyzer.github.io/book/contributing/style.html)
- [redb Documentation](https://docs.rs/redb/)
- [rkyv Documentation](https://docs.rs/rkyv/)
- `docs/design/009-schema-cqrs.md` (storage layer)
- `docs/design/010-schema-graph-resolver.md` (resolution logic)
- `docs/design/011-property-spec.md` (property validation)
