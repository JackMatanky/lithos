---
feature: Note Frontmatter (FieldValue + Frontmatter Access)
status: Draft
author: Jack Matanky
ticket: TBD
date_created: 2026-02-02
tags: [note, frontmatter, schema, validation, ergonomics]
---

# Tech Spec: Note Frontmatter (FieldValue + Frontmatter Access)

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The note bounded context supports YAML frontmatter (Obsidian-compatible) as a dynamic metadata container.

This component should provide:

- `FieldValue`: a dynamically-typed value enum (mirrors `serde_json::Value` shape) with inspection helpers (`is_*`, `as_*`).
- `Frontmatter`: a `HashMap<String, FieldValue>` wrapper with convenience accessors (`get`, strict typed getters like `try_get`/`try_get_ref`, and configured-key helpers like `title`, `file_class`, `aliases`).
- `FromFieldValue` / `FromFieldValueRef`: local conversion traits used by strict accessors (kept local to avoid orphan rules).

Integration points:

- `Note` aggregate stores `Option<Frontmatter>`.
- Configuration defines which keys represent special fields (title, aliases, file_class, etc.) via `crate::config::types::Frontmatter` and aggregated config.
- A domain event `FrontmatterValidated` exists to represent app-layer schema validation having occurred.

The design must address a few tensions that show up in production usage:

- **Error transparency**: optional getters lose “why” (missing vs wrong type vs partially invalid array). The strict accessors must return `Result<_, FrontmatterError>`.
- **Strict vs lenient conversions**: strict extraction should fail loudly for schema-driven pipelines, but user-facing flows may still want lenient behavior (e.g., string arrays with mixed types).
- **Allocation signaling**: some accessors return owned values (`String`, `Vec<String>`) without making allocation explicit in naming.
- **Construction invariants**: constructor/validation responsibilities must be explicit so callers don’t assume invariants that aren’t enforced.

This spec proposes an idiomatic Rust alignment that preserves the ergonomic “dynamic value” model while making strictness and allocations explicit.

### 1.2 Goals & Non-Goals

**Goals**

- Preserve the dynamic frontmatter model while improving idiomatic Rust ergonomics:
  - Keep strict, typed access with structured errors.
  - Keep “borrowed view” getters allocation-free where possible.
- Make strict vs lenient behavior explicit:
  - Keep lenient helpers where they are intentionally user-friendly.
  - Add strict APIs that return typed errors (for schema-driven pipelines and debugging).
- Improve error transparency and debugging:
  - Provide an error type that distinguishes missing keys from type mismatches and malformed values.
- Align naming with project style rules:
  - If an accessor allocates/clones, its name should communicate that (or it should return a borrowed value).

**Non-Goals**

- Redesigning the schema system or replacing `FieldValue` with `serde_json::Value`.
- Introducing filesystem I/O (frontmatter remains in-memory, deterministic).
- Adding new external dependencies.
- Implementing a YAML parser in this module (parsing remains elsewhere).

### 1.3 Constraints (The Hard Limits)

- **rkyv compatibility**: `FieldValue` and `Frontmatter` must remain `rkyv::{Archive, Serialize, Deserialize}` friendly.
- **Backwards compatibility**: existing public APIs should remain available; new strict APIs should be additive initially.
- **Bounded context boundaries**: frontmatter errors must remain note-context errors (no schema error leakage).

Additional persisted-format constraint:

- **Persisted bytes contract**: changes to `FieldValue` / `Frontmatter` structure or `rkyv` format-control features can invalidate existing on-disk data; treat these as migration events.

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

Frontmatter supports two primary usage patterns:

1. **Unknown type (runtime inspection)**

```rust
use lithos_core::note::frontmatter::FieldValue;

let value = FieldValue::String("hello".to_owned());

if value.is_string() {
    assert_eq!(value.as_str(), Some("hello"));
}
```

2. **Known type (schema-driven extraction)**

Today (strict extraction with errors):

```rust
use lithos_core::note::frontmatter::{Frontmatter, FieldValue};
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("priority".to_owned(), FieldValue::Number(5.0));
let fm = Frontmatter::new(fields).unwrap();

let priority: Option<f64> = fm.try_get("priority").unwrap();
assert_eq!(priority, Some(5.0));
```

Proposed additions (strict extraction, with errors):

```rust
use lithos_core::note::frontmatter::{Frontmatter, FieldValue, FrontmatterError};
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("aliases".to_owned(), FieldValue::Array(vec![
    FieldValue::String("ok".to_owned()),
    FieldValue::Number(123.0),
]));
let fm = Frontmatter::new(fields).unwrap();

// Strict: fails instead of silently dropping non-strings.
let aliases = fm.try_get_string_vec_strict("aliases");
assert!(matches!(aliases, Err(FrontmatterError::ArrayElementTypeMismatch { .. })));

// Lenient: tolerant conversion (drops non-strings).
let aliases_lenient = fm.get("aliases").and_then(FieldValue::as_string_array_lossy);
assert_eq!(aliases_lenient, Some(vec!["ok".to_owned()]));
```

Configured keys remain supported:

```rust
use lithos_core::config::aggregate::Config;
use lithos_core::config::global::Global;
use lithos_core::config::vault::Vault;
use lithos_core::note::frontmatter::{Frontmatter, FieldValue};
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("title".to_owned(), FieldValue::String("My Note".to_owned()));
let fm = Frontmatter::new(fields).unwrap();

let global = Global::default();
let vault = Vault::default();
let config = Config::build(Some(&global), "/vault", vault).unwrap();

// Option A (existing): allocates
let title = fm.title(&config);
assert_eq!(title, "My Note");

// Option B: borrow (generic borrowed accessor)
let title_ref: Option<&str> = fm.try_get_ref(&config.frontmatter.title_key).unwrap();
assert_eq!(title_ref, Some("My Note"));
```

### 2.2 Mental Model

- `Frontmatter` is a **typed map** whose values are dynamic.
- `FieldValue` is the **runtime sum type**.
- There are two extraction modes:
  - **Lenient**: ergonomic, tolerant (Obsidian compatibility).
  - **Strict**: schema/debugging oriented; returns typed errors.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

Frontmatter is a note-context leaf component used by:

- note parsing / ingestion (creates `Frontmatter`)
- note aggregate (`Note` stores it)
- schema validation in the app layer (emits `FrontmatterValidated`)

This module remains sync, deterministic, and I/O-free.

### 3.2 Data Models

`FieldValue` is the persisted/frontmatter value model:

- `Array(Vec<FieldValue>)`
- `Boolean(bool)`
- `Date(i64)`
- `Number(f64)`
- `Object(HashMap<String, FieldValue>)`
- `String(String)`

Note: Any changes to `FieldValue` variant set are a persisted-format concern due to `serde` + `rkyv`.

Also note: recursion is persisted. The rkyv derives must continue to use the documented recursion strategy (`#[rkyv(omit_bounds)]` on recursive fields) unless/until a migration is performed.

Note: for ergonomics, APIs may expose `DateTime<Utc>` (e.g., `FieldValue::as_datetime()`), but the persisted representation remains an integer timestamp to keep the stored format simple and rkyv-friendly.

#### Date/time fidelity vs semantics

Storing `Date(i64)` is optimized for **semantic operations** (sorting, filtering, comparisons) and rkyv friendliness, but it is not a lossless representation of YAML frontmatter date/time.

- **Potential fidelity loss**:
  - timezone offset / original zone (`-0500` vs `Z`)
  - textual form (date-only `2026-02-02` vs datetime `2026-02-02T10:00:00Z`)
  - sub-second precision
- **Operational impact**:
  - Parsing adapters must normalize input into the chosen domain representation.
  - If we later need exact round-trip reproduction of frontmatter values (including original formatting), `Date(i64)` alone is insufficient.

Design stance (best practice):

- Default the domain model to **semantic** representation (`i64`) because it is cheap and predictable.
- If/when exact round-trip fidelity is required, add an explicit lossless representation (e.g., an additional variant like `DateRaw(Box<str>)` or `DateString(Box<str>)`) and treat it as a persisted-format migration.

#### Query semantics for `Date(i64)`

Storing a date/time as `i64` (Unix timestamp) generally makes querying _easier_ and more performant, because comparisons become numeric.

- **Instant/range queries** (recommended): convert the query inputs into a UTC timestamp range and filter numerically.
  - Example semantics: “created within [start, end)” becomes `start_ts <= created_ts && created_ts < end_ts`.
- **Local calendar date queries**: if users specify a local date (e.g., “2026-02-02” in a specific timezone), convert that to a UTC range first (start-of-day to next-start-of-day in that timezone), then apply the numeric range query.

Limitations to be aware of:

- If the original YAML value included a timezone offset or used a date-only textual form, that _format_ is not queryable once normalized to an `i64` unless we store additional metadata.
- If we later decide users must be able to query “date-only values” distinctly from “datetime values”, we will need a richer representation (e.g., separate variants or a tagged wrapper).

### 3.3 Component & Interface Specifications

#### Component: `FieldValue`

- **Responsibility**: represent dynamic frontmatter values.
- **Public Interface**:
  - `is_*()` / `as_*()` typed inspection and extraction.
- **State/Invariants**:
  - Enum is `#[non_exhaustive]` to allow future extension.
  - Recursion is supported (`Array(Vec<FieldValue>)`, `Object(HashMap<String, FieldValue>)`).

##### Serialization (Phase 7) — Target Strategy

We want `FieldValue` and `Frontmatter` to remain persistable via **rkyv** (with **bytecheck** validation) _without_ redesigning away recursion.

The target strategy is the documented rkyv approach for recursive types:

- Derive `rkyv::{Archive, Serialize, Deserialize}` (+ `CheckBytes` / bytecheck).
- Add `#[rkyv(omit_bounds)]` on the recursive fields so rkyv’s default “perfect derive” bounds do not cause rustc trait-solver overflow.

If this strategy ever becomes insufficient (e.g., due to format evolution needs), fall back to a separate-frontmatter storage format as an explicit migration (see Alternatives).

Conversion design note:

This module uses _local_ traits (`FromFieldValue` / `FromFieldValueRef`) instead of `TryFrom<&FieldValue>` to avoid Rust's orphan rules.

#### Component: `Frontmatter`

- **Responsibility**: provide safe/ergonomic access to frontmatter fields.
- **Public Interface (existing)**:
  - `get(&self, key: &str) -> Option<&FieldValue>`
  - Strict accessors: `try_get`, `try_get_required`, `try_get_ref`, `try_get_required_ref`
  - Config-driven helpers: `title`, `file_class`, `aliases`
- **State/Invariants**:
  - Map keys are case-sensitive.
  - Accessors must be deterministic and I/O-free.

Planned improvements (strict APIs + borrowing):

- Add strict accessors that preserve error context:
  - `try_get<T>(&self, key: &str) -> Result<Option<T>, FrontmatterError>` where `T: TryFrom<&FieldValue, Error = FrontmatterError>` (or a dedicated conversion error type)
  - `try_get_required<T>(&self, key: &str) -> Result<T, FrontmatterError>`

Borrowing stance:

Prefer _generic_ borrowed extraction via `try_get_ref` + `FromFieldValueRef` over adding many specialized `*_str` helpers. This keeps the API surface smaller and makes “borrow vs allocate” an explicit call-site choice.

#### Component: `FrontmatterError`

- **Responsibility**: communicate extraction and validation failures precisely.
- **Public Interface**:
  - `enum FrontmatterError { Missing { key }, TypeMismatch { key, expected, actual }, ArrayElementTypeMismatch { key, index, expected, actual }, ... }`
- **Integration**:
  - Either embed in `NoteError` as a structured variant, or convert into existing `NoteError::Frontmatter(String)` at note-context boundaries.

### 3.4 Integration & Data Flow

- **Sequence Diagram**:

```mermaid
sequenceDiagram
  participant Parser as Frontmatter Parser (adapter/app)
  participant FM as Frontmatter
  participant Note as Note Aggregate
  participant Schema as App-layer Schema Validator

  Parser->>FM: Frontmatter::new(fields)
  FM-->>Parser: Ok(frontmatter)
  Parser->>Note: note.set_frontmatter(Some(frontmatter))
  Schema->>Note: read note.frontmatter()
  Schema->>Schema: validate against schema
  Schema-->>Schema: emit FrontmatterValidated
```

- **Events/Messages**:
  - `FrontmatterValidated { note_id, field_count, timestamp }` (already defined)

- **Dependencies**:
  - `chrono` for `DateTime<Utc>`
  - `serde` + `rkyv` for serialization

### 3.5 Core Logic & Algorithms

Key policy decisions to make explicit:

1. **Lenient vs strict array extraction**

- Lenient (`get_string_array`): keep Obsidian compatibility by supporting both a single string and an array; when an array includes mixed types, ignore non-string elements.
- Strict (`try_get_string_vec_strict`): if the value is an array, require all elements to be strings; otherwise return an error.

2. **Typed conversion strategy**

- Keep today’s `FromFieldValue` as a convenience trait for ergonomic `Option`-based extraction.
- Keep strict conversions on the local traits (`FromFieldValue` / `FromFieldValueRef`).

Rationale:

- Implementing `TryFrom<&FieldValue>` for common Rust types (`bool`, `f64`, `String`, `Vec<String>`, etc.) is blocked by Rust’s orphan rules.
- Even if it were possible, value-level conversions still need `Frontmatter` to attach **key context** (`key`, array index) for actionable errors.

3. **Construction validation**

- Either:
  - make `Frontmatter::new` actually validate invariants (e.g., forbid empty keys, optionally enforce key normalization), or
  - change `Frontmatter::new` to be infallible (`fn new(...) -> Self`) and move validation to a separate `validate()` method.

This spec recommends the second option for clarity:

- `Frontmatter::new(fields) -> Self` (infallible constructor)
- `Frontmatter::validate() -> Result<(), FrontmatterError>` (explicit validation step)

…but this is an API change and should be staged (see Migration Strategy).

### 3.6 Leanness, Performance, and Allocation Transparency

This module can sit on a hot path (read per-note, potentially across large vaults). The design should default to **borrowed, allocation-free reads**, and only allocate when the caller explicitly requests owned data.

#### Allocation rules

- Prefer accessors that return borrowed views (e.g., `Option<&str>`) when the underlying data is already owned inside `Frontmatter`.
- If a method allocates, it should either:
  - have an allocating name (`*_owned`, `to_*`, `into_*`), or
  - have a parallel borrowed variant (`*_str`, `*_ref`, `*_view`).

Concrete application to this module:

- Keep `FieldValue::as_str(&self) -> Option<&str>` as the primary “cheap path”.
- Prefer `Frontmatter::try_get_ref::<&str>(key)` for allocation-free access (including configured keys).
- Prefer exposing “structural borrows” so callers can iterate without allocations:
  - `Frontmatter::get_array(&self, key: &str) -> Option<&[FieldValue]>`
  - `Frontmatter::get_object(&self, key: &str) -> Option<&std::collections::HashMap<String, FieldValue>>`

Note: with `HashMap<String, _>`, lookups by `&str` are allocation-free because `String: Borrow<str>`.

#### Strictness and perf are compatible

Strict extraction should still avoid cloning:

- Strict getters validate types and return structured errors, but should not require allocation.
- For arrays, strict mode can validate every element and then return a borrowed view; callers who need owned `Vec<String>` can explicitly `collect()`.

### 3.7 Rust-Idiomatic Conversion Strategy (Research Hardened)

This module cannot directly use `TryFrom<&FieldValue>` for `bool`, `f64`, `String`, etc. due to Rust’s orphan rules.

Preferred strategy:

- Keep the local traits (`FromFieldValue` / `FromFieldValueRef`) for strict typed extraction.
- Keep `Option`-returning helpers for deliberately lenient UX.

Key nuance: `TryFrom<&FieldValue>` conversions are value-level and cannot naturally include the map key. Therefore:

- Conversions should report value mismatch (`expected` vs `actual`) and any local detail (like array index).
- `Frontmatter` strict accessors should attach key context (wrapping the value-level error with `{ key, .. }`).

### 3.8 Modularization Plan (Code Organization)

Frontmatter is already growing into a large module (recursive rkyv derives, conversions, accessors, and tests). Plan for modularization early.

Design stance (Rust-idiomatic + note-context-aware): avoid “nesting for its own sake”.

Preferred (near-term):

- Keep a single public module file: `lithos-core/src/note/frontmatter.rs`.
- Keep `FrontmatterError` owned by the note context in `lithos-core/src/note/error.rs`.
- Keep configured-key helpers (`title`, `file_class`, `aliases`) colocated with `Frontmatter` (they are note concerns and depend only on cross-cutting config).

If/when the file becomes too large, split **flat** before splitting **deep**:

- Option A (flat files, no directory nesting):
  - `lithos-core/src/note/frontmatter.rs` (public API + `Frontmatter`)
  - `lithos-core/src/note/frontmatter_value.rs` (`FieldValue` + inspection)
  - `lithos-core/src/note/frontmatter_convert.rs` (local conversion traits + impls)

- Option B (one-level directory) only if Option A becomes awkward:
  - `lithos-core/src/note/frontmatter/mod.rs` re-exporting a small set of items
  - `value.rs`, `convert.rs` as internal modules

Both options keep module depth shallow and respect the existing note context layout (aggregate/error/events/task/etc.).

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Keep `FieldValue` modeled after `serde_json::Value`

- **Context**: dynamic frontmatter is a core requirement and mirrors common Rust ecosystem patterns.
- **Choice**: keep the enum + `is_*/as_*` shape.
- **Alternatives Considered**:
  - _Use `serde_yaml::Value` everywhere_: couples note domain to a parsing format; rejected.
  - _Use `serde_json::Value`_: similar shape but introduces cross-format impedance; rejected.

#### Decision: Add strict extraction APIs using `TryFrom`

- **Context**: schema-driven flows need actionable errors.
- **Choice**: keep strict helpers returning `Result` with a typed error, implemented via local conversion traits.
- **Alternatives Considered**:
  - _Only `Option` everywhere_: too little debugging signal.
  - _Panics on mismatch_: prohibited by no-panic policy.

Implementation note:

- Use `FromFieldValue` / `FromFieldValueRef` (local traits) rather than `TryFrom` to avoid orphan-rule dead-ends.

#### Decision: Preserve lenient helpers for Obsidian compatibility

- **Context**: Obsidian uses both scalar and array representations for certain fields.
- **Choice**: keep lenient APIs (e.g., string array fallback to single string).
- **Alternatives Considered**:
  - _Strict-only_: would create poor UX for existing vaults.

#### Decision: Serialize recursive `Frontmatter` via rkyv (Phase 7)

- **Context**: naive rkyv derives for recursive enums can trigger trait-solver overflow due to recursive where-clause expansion.
- **Choice**: keep frontmatter in the `Note` archive and use rkyv’s recommended recursion escape hatch:
  - Derive rkyv + bytecheck for `FieldValue` and `Frontmatter`.
  - Apply `#[rkyv(omit_bounds)]` on recursive fields to avoid recursive where-clause expansion.
- **Why this is preferable**:
  - One write/read per note (no separate "frontmatter" table).
  - Preserves frontmatter in the same persisted entity (simpler query path).
  - Keeps the option to expose zero-copy archived views for frontmatter later.
- **Alternatives considered**:
  - _Separate frontmatter storage (serde JSON/TOML)_: simple and flexible, but adds extra DB operations and loses zero-copy for frontmatter.
  - _Manual rkyv impls_: maintenance-heavy and error-prone.
  - _Flatten values (no recursion)_: breaks structured frontmatter.

#### Decision: Do not box `Object` by default

- **Question**: why not represent objects as `Object(Box<HashMap<String, FieldValue>>)`?
- **Finding**: in this shape, boxing `HashMap` typically adds allocation/indirection without a corresponding size reduction.
  - `String`, `Vec<T>`, and `HashMap<K,V>` are already pointer-heavy (multi-word) types.
  - The enum size is driven by the largest payload variant; with `String`/`Vec` already present, boxing only `Object` rarely reduces `FieldValue`’s maximum payload size.
- **Outcome**: prefer `Object(HashMap<...>)` unless a broader representation change demonstrates a real memory/layout win.

This is a performance-first choice: avoid extra allocations and pointer-chasing unless they buy something concrete.

#### Consideration: extract nested container values into dedicated structs

An alternative organization is to move container payloads out of `FieldValue` into dedicated types:

- Example: `Object(ObjectValue)` where `struct ObjectValue(HashMap<String, FieldValue>);`
- Example: `Array(ArrayValue)` where `struct ArrayValue(Vec<FieldValue>);`

Potential benefits:

- Cleaner API surface: object-specific helpers live on `ObjectValue` (typed lookups, key iteration policies).
- Easier to attach invariants/validation at container boundaries.
- A single place to hang rkyv/serde attributes and bounds.

Limitations:

- This does **not** remove recursion; it just moves it, so the rkyv recursion strategy (`#[rkyv(omit_bounds)]` on recursive fields) is still required.

Expected outcome:

- Mostly improved organization and maintainability; performance is typically neutral unless we also change the underlying container types or allocation strategy.

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

Frontmatter is pure; observability is primarily via surfaced errors.

- Strict APIs should return typed errors that include key name and expected/actual types.
- When mapping into `NoteError`, keep error messages stable and actionable.

### 5.2 Migration Strategy

- **Phase 1 (additive)**:
  - Keep strict APIs and `FrontmatterError`.
  - Keep allocating helpers (`title`, `file_class`, `aliases`) and use `try_get_ref` for allocation-free access where needed.
- **Phase 2 (optional cleanup)**:
  - Consider changing `Frontmatter::new` to be infallible and introducing explicit `validate()`.
  - Consider renaming allocating helpers (or adding borrowed variants) to match allocation transparency guidelines.

### 5.3 Security & Privacy

- Avoid path and I/O concerns: frontmatter is metadata-only.
- Ensure errors do not leak sensitive values in logs by default.

## 6. Pre-Mortem (The "Inversion")

- **Risk**: Strict extraction breaks existing vaults that relied on lenient coercion.
  - _Mitigation_: keep strict APIs additive; keep lenient ones for user-facing paths.

- **Risk**: Allocation-heavy APIs cause performance regressions when called per-note at scale.
  - _Mitigation_: prefer `try_get_ref` (borrowed extraction) and keep allocating methods for convenience with clear intent.

- **Risk**: Introducing a new error type proliferates conversions and boilerplate.
  - _Mitigation_: keep the error small and focused; provide helper constructors and `Display` messages.

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                    | Resolution                                                             |
| :--------- | :-------------------------------------------------- | :--------------------------------------------------------------------- |
| 2026-02-02 | "Option-based getters lose why extraction failed."  | Add strict `Result`-based APIs with `FrontmatterError`.                |
| 2026-02-02 | "Lenient coercion can hide data-quality issues."    | Make strict vs lenient behavior explicit; keep both with clear naming. |
| 2026-02-02 | "Frontmatter::new returns Result but never errors." | Specify staged plan: infallible constructor + explicit `validate()`.   |

## 8. References

- Rust API Guidelines (general idiomatic API + ownership guidance): https://rust-lang.github.io/api-guidelines/
- `TryFrom` / `TryInto` (standard conversion traits and usage guidance): https://doc.rust-lang.org/std/convert/trait.TryFrom.html
- `Cow` (clone-on-write and allocation behavior patterns): https://doc.rust-lang.org/std/borrow/enum.Cow.html
- `chrono` (DateTime types, parsing/formatting behavior): https://docs.rs/chrono/latest/chrono/
- `rkyv` (zero-copy + validation, persisted-format constraints): https://docs.rs/rkyv/latest/rkyv/
- Serde enum representations (background on tagged vs untagged tradeoffs): https://serde.rs/enum-representations.html

## Appendix A: Current Implementation & Change List (Temporary)

This appendix is intentionally implementation-specific. Remove it once the code matches the design.

Current implementation snapshot:

- Implementation is in `lithos-core/src/note/frontmatter.rs` (single file).
- `Frontmatter::new` returns `Result<Self, NoteError>` but is currently infallible.
- Strict accessors already exist: `try_get`, `try_get_required`, `try_get_ref`, `try_get_required_ref`.
- Conversions are implemented via local traits `FromFieldValue` / `FromFieldValueRef`.
- Config-driven helpers exist and allocate (`title`, `file_class`, `aliases`).

Change list (if/when needed):

- If we decide the constructor should be infallible, change `Frontmatter::new` to return `Self` and add an explicit `validate()` step.
- If the module grows too large, apply the “flat split before deep split” modularization plan in section 3.8.
