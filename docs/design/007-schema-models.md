---
feature: Schema Models (Aggregates + Value Objects)
status: Draft
author: Jack Matanky (drafted with GitHub Copilot)
ticket: TBD
date_created: 2026-02-03
tags: [schema, domain-models, type-driven-design, rkyv, performance]
---

# Tech Spec: Schema Models (Aggregates + Value Objects)

## 0. Definition of Done

- The schema bounded context has a clearly documented model boundary:
  - persisted (serde-friendly) input types,
  - validated runtime/domain types,
  - archived/persistence representation (rkyv).
- All model invariants are enforced by construction (type-driven), not by convention.
- Model types are lean and idiomatic:
  - borrowed inputs where possible,
  - no hidden allocations in getters,
  - minimal cloning in hot paths.
- Zero-copy constraints are explicitly documented for redb + rkyv (validation, lifetime/guard scope).
- Any proposed breaking changes include an explicit migration strategy.

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The schema bounded context is responsible for:

- representing schemas (names, inheritance relationships, property sets),
- representing reusable property definitions (property bank),
- validating metadata values at runtime.

Current implementation lives under:

- `lithos-core/src/schema/aggregate.rs` (`Schema`, `SchemaName`, `PropertyBank`)
- `lithos-core/src/schema/property.rs` (`Property`, `PropertyName`)
- `lithos-core/src/schema/property_spec.rs` (`PropertySpecDef`, `PropertySpec`, type-driven invariant helpers)
- `lithos-core/src/schema/raw.rs` (`RawSchema`, `RawProperty*`)

The system aims to be lean and performant, with a strong zero-copy inclination.

Key constraints from the overall architecture:

- redb is transaction-scoped and returns guard-based views of values.
- rkyv enables zero-copy access to archived data, but only after validation at trust boundaries.
- changing rkyv “format control” choices or archived model layout can be a breaking on-disk format change.

### 1.2 Goals & Non-Goals

**Goals**

- Make schema model invariants unforgeable via type-driven design.
- Clarify and separate the three “shapes” of data:
  - **wire/input** (serde DTO-ish; may be invalid),
  - **validated runtime/domain** (invariants enforced),
  - **archived/persisted** (rkyv, read via redb guards).
- Keep runtime validation ergonomic while preserving performance:
  - avoid stringly-typed identifiers,
  - prefer `Box<str>` / borrowed views over `String` cloning.

**Non-Goals**

- Redesigning the storage layer (`db.rs`) in this spec.
- Introducing new dependencies.
- Implementing the refactor immediately; this document defines the design and migration plan.

### 1.3 Constraints (The Hard Limits)

- **Zero-copy boundaries**: rkyv validation (bytecheck / `rkyv::access`) occurs at trust boundaries before archived data is used.
- **redb guard lifetimes**: do not return `AccessGuard` or any reference derived from it beyond the transaction scope; closure-based APIs are preferred.
- **Sync-first core**: schema model construction/validation remains synchronous.
- **Lean models**: avoid “stringly typed” keys and avoid hidden allocations.
- **API clarity**: prefer borrowed argument and accessor types (e.g. `&str` rather than `&String`) and keep allocation decisions explicit (see https://rust-analyzer.github.io/book/contributing/style.html).

### 1.4 Minimizing “derive-everything” Blast Radius (rkyv)

Schemas and property banks are attractive to archive “as-is”, but large rkyv derive surfaces create a maintenance hazard: small model refactors can silently become **persisted-format changes**.

Guidance:

- Prefer isolating rkyv derives onto **persistence DTOs** (storage-layer types) when it reduces coupling. Domain types stay ergonomic; persisted types stay stable.
- Keep archived compute closure-based and local to the storage/query tier; do not leak archived references outside transaction scope.
- Treat any change to archived layout, rkyv attributes, or format-control feature set as a migration decision.
- Introduce projections for hot queries rather than forcing the primary persisted schema blob to satisfy every read shape.

### 1.5 Raw → Domain boundary (designing to avoid `Stored*` models)

The primary lever for avoiding `StoredSchema`/`StoredPropertyBank` is _not_ introducing a storage DTO early; it is designing the **validated domain types** to be both:

- ergonomic for business logic, and
- reasonably rkyv-friendly for persistence and archived reads.

Recommended boundary discipline:

1. **Raw (wire/input)** types (`RawSchema`, `RawProperty*`) stay serde-friendly and error-reporting friendly.

- Use `String`, `Option`, and “tolerant shapes” to capture good diagnostics.
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

Conceptually:

1. Parse raw schema input into `RawSchema` / `RawProperty*` (wire types).
2. Compile/validate into runtime types (`Schema`, `Property`, `PropertySpec`).
3. Persist runtime types via rkyv (or persist defs and compile on load; decision in CQRS spec).
4. Validate metadata values using `Property::validate_value(&serde_json::Value)`.

### 2.2 Mental Model

- **Names and IDs are distinct types**.
  - A `SchemaName` is a validated identifier for “human named” schema references.
  - A `SchemaId` is the stable identity stored as a UUIDv7.
- **Property definitions are reusable**.
  - A `PropertyBank` stores canonical property definitions.
  - A `Schema` references properties (either by embedding definitions or by id; see Alternatives).
- **Persisted bytes are not trusted**.
  - redb provides bytes; rkyv validation produces an archived view; callers compute results in a closure.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

```mermaid
flowchart LR
  Raw[RawSchema / RawProperty (serde)] --> Compile[Compile + Validate]
  Compile --> Domain[Schema / Property / PropertySpec]
  Domain --> Persist[(redb + rkyv bytes)]
  Persist --> Read[zero-copy read via closure]
  Domain --> Validate[validate metadata values]
```

### 3.2 Component & Interface Specifications

#### Component: `SchemaName` (validated identifier)

- **Responsibility**: represent a validated schema identifier that is safe for indexing/lookup and comparisons.
- **Invariants**:
  - non-empty
  - max length (64)
  - matches `patterns::ALPHANUMERIC_NAME`
- **Public Interface** (target):
  - `SchemaName::try_from(&str) -> Result<SchemaName, SchemaError>`
  - `as_str(&self) -> &str`
  - implements `Display`, `AsRef<str>`, and (optionally) `Borrow<str>`

**Type-driven design rule**: the inner field MUST be private so callers cannot forge invalid values.

Recommended representation:

```rust
pub struct SchemaName(Box<str>);
```

Rationale (Rust API Guidelines + Lithos style rules):

- private fields enforce invariants by construction;
- `Box<str>` is lean for immutable identifiers;
- avoid `Deref<Target=str>` on domain newtypes unless you are explicitly implementing the “owned type derefs to borrowed view” pattern (e.g., `String -> str`). For domain identifiers, prefer explicit `as_str()`.
  - This follows the Rust API Guidelines’ `Deref` guidance for pointer-like types: https://rust-lang.github.io/api-guidelines/checklist.html

#### Component: `PropertyName` (validated identifier)

Same intent and rules as `SchemaName`, with its own error variants.

#### Component: `SchemaId` and `PropertyId`

- **Responsibility**: prevent accidental mixing of UUIDs across concepts.
- **Representation**:

```rust
pub struct SchemaId(Uuid);
pub struct PropertyId(Uuid);
```

- **Ergonomics**:
  - `Copy` is fine (Uuid is Copy).
  - Provide `into_uuid()` or `as_uuid()` (borrowed) only when needed.

#### Component: `Property` (reusable definition)

- **Responsibility**: a validated definition used for runtime metadata validation.
- **State**:
  - `id: PropertyId`
  - `name: PropertyName`
  - “shape” of values: scalar vs array, required vs optional
  - `spec: PropertySpec` (validated)

**Type-driven improvement (recommended)**: replace the two booleans (`required`, `array`) with semantic enums so invalid combinations and “boolean blindness” are avoided:

```rust
pub enum Cardinality { Optional, Required }
pub enum Multiplicity { Single, Many }
```

This improves readability and reduces mistakes at call sites (Rust API Guidelines: avoid multiple boolean parameters).

#### Component: `PropertyBank` (aggregate)

- **Responsibility**: registry for reusable property definitions and fast lookup.
- **Invariants**:
  - unique `PropertyId`
  - unique `PropertyName`
- **Lookup strategy**:
  - keep dual indices for O(1) lookups
  - prefer typed keys: `HashMap<PropertyId, usize>` and `HashMap<PropertyName, usize>`

Design note:

- `PropertyBank::decode(&str)` should be an adapter-level concern when `$ref` format parsing is required.
- domain-level lookup should accept `PropertyId` or `PropertyName` directly.

#### Component: `Schema` (aggregate)

- **Responsibility**: represent a resolved schema used for validation.
- **Invariants**:
  - unique property names (within the schema)
  - deterministic property order for stable output (either by sorting or by using ordered storage)

**Options for representing properties**:

1. **Embedded properties** (current direction)
   - `Schema { properties: Vec<Property> }`
   - Pros: simple; schema is self-contained for validation.
   - Cons: duplicates property defs across schemas; more bytes persisted.

2. **Referential properties** (future)
   - `Schema { properties: Vec<PropertyId> }` with `PropertyBank` as dependency
   - Pros: better dedup; smaller persisted schema.
   - Cons: validation becomes a two-step lookup; need strong coherence rules.

This spec recommends keeping embedded properties for now (simplicity), while designing IDs/newtypes so option (2) remains viable.

### 3.3 Integration & Data Flow

- Raw types are produced by adapters (YAML/TOML/JSON loaders).
- Domain compilation happens in the schema context.
- Persistence is via `Database` (redb+rkyv). This spec defines model representations that are compatible with either:
  - storing validated runtime types, or
  - storing raw defs and compiling on load.

### 3.4 Data Models

#### Wire/input layer

- `RawSchema` / `RawPropertyInline` / `RawPropertyRef` are serde-friendly and may be invalid.
- Rule: wire types may use `String` and “raw” options.
- Rule: wire types must be converted into domain types as early as possible.

#### Validated runtime/domain layer

Target “core” types:

```rust
pub struct Schema {
  id: SchemaId,
  name: SchemaName,
  properties: Vec<Property>,
}

pub struct Property {
  id: PropertyId,
  name: PropertyName,
  cardinality: Cardinality,
  multiplicity: Multiplicity,
  spec: PropertySpec,
}
```

#### Archived/persistence layer

- Archived types are derived from the domain types via rkyv derives.
- Rule: do not expose archived references outside a closure that is scoped to the redb transaction/guard.

### 3.5 Core Logic & Algorithms

- Validation belongs in constructors and in `PropertySpec` validators.
- Avoid “validate() that does nothing”; invariants should be either:
  - enforced by construction, or
  - explicitly tested and documented.

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Private fields for validated identifiers

- **Choice**: `SchemaName` and `PropertyName` have private inner fields.
- **Why**: prevents invalid construction; makes invariants enforceable.
- **Alternative**: keep tuple structs `pub String` (rejected; breaks type-driven invariants).

#### Decision: Prefer `as_str()` over `Deref<Target=str>` for identifiers

- **Choice**: `as_str()` explicit accessor.
- **Why**: keeps APIs explicit and avoids surprising method resolution; aligns with “useful types” and ownership clarity.
- **Alternative**: `Deref`-based implicit conversion (rejected for domain newtypes).

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

- Model construction and validation is deterministic and should not emit logs by default.
- Higher layers (CQRS/adapters) should instrument operations, but avoid logging full metadata payloads.

### 5.2 Migration Strategy

Incremental, low-risk migration path:

1. Introduce newtypes (`SchemaId`, `PropertyId`) and private-field identifiers (`SchemaName`, `PropertyName`).
2. Provide fallible conversions from existing strings (`TryFrom<&str>`, `TryFrom<String>`).
3. Gradually replace `HashMap<String, …>` with typed keys.
4. Update CQRS ports and storage keys (see CQRS spec).

### 5.3 Security & Privacy

- All persisted bytes are treated as untrusted input and must be validated (rkyv safe access) before use.
- `$ref` parsing belongs in adapters; domain logic should operate on typed identifiers.

## 6. Pre-Mortem (The "Inversion")

- **Risk**: “Validated” name types can be forged, leading to inconsistent indexing and hard-to-debug behavior.
  - _Mitigation_: private fields + constructors only.

- **Risk**: Archived references escape transaction scope, causing use-after-free / UB.
  - _Mitigation_: closure-based query APIs; never return guards.

- **Risk**: On-disk format breaks due to changing archived layouts.
  - _Mitigation_: treat rkyv format control and archived model changes as migration events (documented + tested).

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                | Resolution                                                     |
| :--------- | :---------------------------------------------- | :------------------------------------------------------------- |
| 2026-02-03 | Tuple struct identifiers are forgeable          | Require private fields; keep `try_from` and `as_str` accessors |
| 2026-02-03 | Booleans for property shape are easy to misuse  | Recommend semantic enums `Cardinality`/`Multiplicity`          |
| 2026-02-03 | Zero-copy lifetime risks not explicit in models | Document closure-based access and guard scoping                |
