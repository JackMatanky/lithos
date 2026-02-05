---
feature: Schema Graph + Resolver (Inheritance + $ref)
status: Draft
author: Jack Matanky (drafted with GitHub Copilot)
ticket: TBD
date_created: 2026-02-03
tags: [schema, inheritance, resolver, graph, type-driven-design, performance]
---

# Tech Spec: Schema Graph + Resolver (Inheritance + $ref)

## 0. Definition of Done

- Inheritance resolution is specified as a deterministic, testable algorithm.
- `$ref` semantics are precisely defined, including what is adapter-parsed vs domain-resolved.
- Cycle detection and missing-parent behavior is specified and mapped to typed errors.
- Resolver design aligns with type-driven models (no stringly-typed map keys in core).
- The design accounts for performance (minimize cloning) and redb+rkyv zero-copy constraints.

## 1. Problem Space (The "Why")

### 1.1 Context & Background

Schemas can extend a parent schema and can:

- inherit parent properties,
- exclude inherited properties by name,
- define additional inline properties,
- reference reusable properties via `$ref` (property bank).

The bounded context currently implements:

- `Graph`: a topological ordering + cycle detection service.
- `Resolver`: merges parent properties, applies excludes, and resolves `$ref` through `PropertyBank`.

The main risks in this area are:

- correctness (cycle detection, missing parent behavior, excludes precedence),
- determinism (stable ordering for reproducible behavior),
- type-driven clarity (avoid string keys where validated types exist),
- performance (avoid cloning large property definitions unnecessarily).

### 1.2 Goals & Non-Goals

**Goals**

- Specify a deterministic, easy-to-reason-about inheritance + resolution pipeline.
- Clearly define `$ref` semantics, including boundary responsibilities:
  - adapters parse ref formats,
  - domain resolves typed references.
- Reduce stringly-typed logic:
  - key maps by `PropertyName` not `String`,
  - use `SchemaName` as the graph node id.
- Provide a migration path toward fewer clones (future: ids/references).

**Non-Goals**

- Implementing multi-parent inheritance or mixins.
- Adding filesystem I/O validation for file properties.

### 1.3 Constraints (The Hard Limits)

- Pure logic: Graph/Resolver remain deterministic and I/O-free.
- Lean + performant: avoid unnecessary allocations; pre-allocate when possible.
- `$ref` must not become a "mini parser" inside the domain; format-specific parsing belongs in adapters.
- Type-driven preconditions: represent invariants and required normalization in types (e.g., typed `PropertyRef`), rather than relying on string conventions (see https://rust-analyzer.github.io/book/contributing/style.html).

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

Schema resolution happens in two steps:

1. Build/validate the inheritance order.
2. Resolve each schema using its parent’s resolved form.

At a high level:

```text
Raw schemas (name, extends, excludes, properties)
  -> Graph.resolve_order() gives [SchemaName] in parent-first order
  -> for each schema in order:
       Resolver.resolve(raw, parent_resolved, property_bank)
         - start with parent properties
         - remove excludes
         - resolve own properties (inline + refs)
         - ensure deterministic property order
         -> Schema
```

### 2.2 Mental Model

- Graph answers: “In what order can I resolve schemas safely?”
- Resolver answers: “Given one raw schema and an optional resolved parent, what is the final resolved schema?”

Resolver is intentionally local: it does not fetch parents from storage; that orchestration belongs in CQRS / application code.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

```mermaid
flowchart TD
  A[RawSchema set] --> B[Graph: resolve order]
  B --> C[for each schema in order]
  C --> D[Resolver.resolve(raw, parent, bank)]
  D --> E[Resolved Schema]
```

### 3.2 Component & Interface Specifications

#### Component: `Graph`

- **Responsibility**:
  - store a directed inheritance relation (child → parent),
  - detect cycles,
  - compute a deterministic order (parents before children).

- **Public Interface**:
  - `add_node(name: SchemaName, extends: Option<SchemaName>)`
  - `resolve_order() -> Result<Vec<SchemaName>, SchemaError>`

- **Errors**:
  - `CircularInheritance(schema_name)` if a cycle is detected.
  - `ParentSchemaNotFound(parent_name)` if a referenced parent is missing.

Determinism:

- `resolve_order` sorts initial keys before traversal.
- Implementation must be stable across runs.

Type-driven improvement:

- `Graph` should keep its internal map private.
- Expose iterators/accessors if needed.

#### Component: `Resolver`

- **Responsibility**:
  - merge parent properties,
  - apply excludes,
  - resolve inline properties,
  - resolve references via `PropertyBank`.

- **Public Interface**:
  - `resolve(raw: RawSchema, parent: Option<&Schema>, bank: &PropertyBank) -> Result<Schema, SchemaError>`

- **Precedence rules**:
  - Start with all parent properties.
  - Remove excluded property names.
  - Insert/override own properties.

Override behavior decision:

- If the child defines a property with the same name as the parent, the child’s definition wins.
- This should be explicit and tested (it is the most intuitive rule for schema inheritance).

Type-driven improvement:

- Internal resolved map should be keyed by `PropertyName`, not `String`.

```rust
HashMap<PropertyName, Property>
```

This avoids repeated `to_string()` allocations and ensures key validity.

#### `$ref` semantics

The domain resolver should operate on typed references.

Adapter responsibility:

- Parse `$ref` formats (e.g., `#/properties/name`) into a typed reference.
- Normalize and validate allowed ref prefixes.

Domain responsibility:

- Resolve a `PropertyRef` (typed) against the `PropertyBank`.

Recommended typed reference:

```rust
pub enum PropertyRef {
  ById(PropertyId),
  ByName(PropertyName),
}
```

This cleanly supports future expansions without leaking string parsing into the domain.

### 3.3 Integration & Data Flow

Graph + Resolver integration points:

- CQRS layer loads raw schemas and property bank.
- CQRS builds a graph from `RawSchema { name, extends }`.
- CQRS resolves schemas in the computed order.
- CQRS persists resolved schemas.

If a parent schema is missing:

- Graph or the orchestration layer returns `ParentSchemaNotFound`.
- Resolution stops (fail-fast), because partial resolution tends to hide errors.

### 3.4 Data Models

Minimal data for graph resolution:

- Node key: `SchemaName`
- Parent edge: `Option<SchemaName>`

Resolver inputs:

- `RawSchema` (wire)
- `Option<&Schema>` (resolved parent)
- `&PropertyBank`

Resolver output:

- `Schema` (resolved)

### 3.5 Core Logic & Algorithms

#### Graph algorithm

- Standard DFS topological sort with temporary marks for cycle detection.
- Deterministic iteration order by sorting node keys (or using `BTreeMap`).

Design note:

- Avoid relying on `HashMap` iteration order; if using `HashMap`, explicitly sort collected keys before traversal to keep output stable (see https://rust-lang.github.io/api-guidelines/checklist.html).

Trade-off:

- `HashMap + sort(keys)` is generally faster than `BTreeMap` for large maps.
- Keep the sorting local to `resolve_order`.

#### Resolver algorithm

- Use a map keyed by `PropertyName` as a working set.
- Insert parent properties first (skipping excludes).
- Insert child properties next.
- Output as `Vec<Property>` sorted by `PropertyName` for determinism.

Performance note:

- Current approach clones parent properties.
- This is acceptable initially (schemas are small), but the design keeps the door open to a future representation where schemas store `PropertyId` references.

Avoid the “clone to satisfy the borrow checker” anti-pattern; if cloning becomes hot-path expensive, prefer restructuring the working set or switching to id-based resolution rather than adding incidental clones (see https://rust-unofficial.github.io/patterns/).

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: `$ref` parsing stays out of domain

- **Choice**: adapters normalize and parse `$ref` syntax; domain resolves typed refs.
- **Why**: keeps domain small, deterministic, and avoids format lock-in.
- **Alternative**: parse strings in `Resolver` (rejected; becomes a hidden parser and mixes concerns).

#### Decision: child overrides parent on duplicate name

- **Choice**: child definition wins.
- **Why**: most common inheritance expectation; simplifies mental model.
- **Alternative**: treat as error (possible future toggle, but not default).

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

- Graph/Resolver should not log by default.
- CQRS/app layer can log:
  - number of schemas resolved,
  - resolution time,
  - cycle detection failures,
  - missing parent failures.

### 5.2 Migration Strategy

- Step 1: introduce typed `PropertyRef` and move `$ref` parsing into adapters.
- Step 2: change resolver working map to `HashMap<PropertyName, Property>`.
- Step 3 (optional future): store schemas as `Vec<PropertyId>` and resolve through `PropertyBank` at validation time.

### 5.3 Security & Privacy

- `$ref` parsing must reject traversal or unsupported prefixes (adapter-level).
- Domain must never interpret filesystem paths for schema inheritance.

## 6. Pre-Mortem (The "Inversion")

- **Risk**: subtle nondeterminism causes flaky tests and confusing diffs.
  - _Mitigation_: deterministic key ordering in Graph; sorted output properties.

- **Risk**: `$ref` parsing logic proliferates and becomes inconsistent.
  - _Mitigation_: define a single adapter-level parser producing `PropertyRef`.

- **Risk**: schema resolution clones too much, becoming hot-path expensive.
  - _Mitigation_: treat cloning as acceptable for now; keep a clear path to id-based schema storage.

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                            | Resolution                                                    |
| :--------- | :---------------------------------------------------------- | :------------------------------------------------------------ |
| 2026-02-03 | Domain resolver keyed by `String` loses type safety         | Specify `HashMap<PropertyName, Property>` working set         |
| 2026-02-03 | `$ref` parsing in domain mixes concerns and adds complexity | Move parsing to adapters; domain resolves typed `PropertyRef` |
| 2026-02-03 | Inheritance behavior on override not explicit               | Specify "child overrides parent" precedence rule              |
