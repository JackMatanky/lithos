# Schema Module Review: Findings & Implementation Plan

**Date**: 2026-02-17
**Scope**: `lithos-core/src/schema/` — complete critical review
**Status**: Pre-implementation planning document

---

## Executive Summary

The schema module has a solid architectural skeleton (CQRS ports, domain aggregates, rkyv storage, events), but contains a critical layer that is fundamentally broken relative to the actual vault file format: `raw.rs` cannot deserialize any real vault schema file. Additionally, several correctness bugs, incomplete workflows, and performance issues were found. This document catalogs all findings and provides a precise implementation plan.

---

## Part 1: Critical Findings (Blocking)

### F-01 — `RawSchema` Cannot Deserialize Real Vault Files

**Severity**: Critical — the entire ingestion pipeline produces no output
**Files**: `raw.rs`, `resolver.rs`

The actual vault schema format (illustrative; real `task.json` has 13 properties, `task_child.json` uses `extends`/`excludes`):
```json
{
  "name": "task_child",
  "properties": {
    "date": { "$ref": "property_bank#/date_iso_8601" },
    "status": { "$ref": "property_bank#/task_status" },
    "type": { "type": "string", "options": ["action_item", "habit"] }
  },
  "extends": "task",
  "excludes": ["date_start", "date_end"]
}
```

Current `RawSchema` expects:
- `id: Uuid` — **not present in vault files**
- `properties: Vec<RawProperty>` — **vault uses a JSON object (map), not an array**

The `properties` field is the most critical mismatch. The property name is the **map key** in the vault format, not a field inside the value. The current `Vec<RawProperty>` shape cannot even begin to deserialize a real schema file.

**Root cause**: The raw types were designed without reading the actual vault file format.

---

### F-02 — `RawPropertyInline` Has Stale Field Name (`array` → `multi`)

**Severity**: Critical
**File**: `raw.rs:115`

```rust
pub array: bool,   // WRONG — vault uses "multi"
```

The schema format redesign renamed `array` to `multi`. Any vault file using `multi: true` will silently deserialize to `false` (the default), producing incorrect multiplicity on every property.

---

### F-03 — `$ref` Format Not Handled by `PropertyRef::parse`

**Severity**: Critical
**File**: `property.rs:471`

The actual vault `$ref` format is `"property_bank#/<name>"` (e.g. `"property_bank#/date_iso_8601"`).

`PropertyRef::parse` currently handles:
- `"#/properties/<name>"` — never used in vault files
- `"$bank:<uuid>"` — never used in vault files
- Plain UUID strings
- Plain name strings

It does **not** handle `"property_bank#/<name>"`. Every `$ref` in the vault will fall through to plain-name parsing, which will strip nothing and attempt to look up `"property_bank#/date_iso_8601"` as a property name — which will always fail.

---

### F-04 — `RawPropertyBank` Does Not Exist

**Severity**: Critical — property bank cannot be loaded from vault files
**File**: `raw.rs`

The actual `property_bank.json` format:
```json
{
  "properties": {
    "task_status": {
      "type": "string",
      "options": { "1": "to_do", "2": "in_progress", "3": "done" }
    },
    "date_iso_8601": { "type": "date", "format": "%Y-%m-%d" },
    "pillar": { "multi": true, "type": "file", "directory": "(20_pillars)/" }
  }
}
```

There is no `RawPropertyBank` struct to deserialize this. The property bank can only be created programmatically. The entire vault ingestion pipeline for the property bank is missing.

---

### F-05 — `StringSpecDef` Uses Wrong Field Name (`enum_values` → `options`)

**Severity**: Critical
**File**: `raw.rs:298`

```rust
pub enum_values: Option<Vec<Box<str>>>,  // WRONG — vault uses "options"
```

Every vault schema using `options` (e.g. `"options": ["do", "due"]`) will deserialize to `None`, silently stripping all enum constraints.

Additionally, the field type `Vec<Box<str>>` only handles Mode 1 (plain array). The vault uses three options modes:
- **Mode 1**: `["a", "b", "c"]` — plain array
- **Mode 2**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed object
- **Mode 3**: `[{"value": "jan", "label": "January", "order": 1}]` — rich entries

Mode 2 appears in the actual `property_bank.json` (`task_status`). Without `RawOptions`, the bank cannot be loaded.

---

### F-06 — `RawPropertyRef` Cannot Carry Override Fields

**Severity**: Critical
**File**: `raw.rs:121-127`

```rust
pub struct RawPropertyRef {
    pub ref_path: Box<str>,  // only field
}
```

Per the meta-schema (`note-metadata.schema.json`), a `$ref` property can carry override fields alongside `$ref`: `required`, `multi`, `options`, `pattern`, `step`, `min`, `max`, `format`, `directory`, `file_class`. This allows schemas to use a bank property as a base and customize it. The current struct silently discards all override fields on deserialization.

---

### F-07 — `RawSchema` Contains `id: Uuid` That Is Not In Vault Files

**Severity**: Critical
**File**: `raw.rs:57-58`

Schema files have no `id` field. Every vault schema parsed by the current code will fail deserialization because serde cannot find the required `id` field. IDs must be assigned at ingestion time:
- **Existing schema** (name already in DB): reuse the stored `SchemaId`
- **New schema** (name not in DB): generate a new `SchemaId::new()`

---

### F-08 — `save_with_metadata` Is Not Atomic

**Severity**: High — data integrity bug
**File**: `adapter/command.rs:41-64`

`save_with_metadata` performs three separate DB writes (schema, metadata, name index) without a transaction. A crash between any two puts leaves the database in partial state. The `save_batch` method uses `batch_write` (atomic) and does the same job. `save_with_metadata` is a non-atomic duplicate.

---

### F-09 — `SchemaHash::compute` Uses `DefaultHasher` (Not Stable Across Rust Versions)

**Severity**: High — staleness detection silently breaks after Rust upgrades
**File**: `aggregate.rs:557-573`

`DefaultHasher`'s output is explicitly not guaranteed stable across Rust versions. `SchemaHash` is stored in `ResolutionMetadata` on disk and compared against freshly-computed hashes to detect parent schema changes. A Rust version upgrade can change `DefaultHasher`'s output, making every stored hash compare unequal to the new computation — forcing re-resolution of all schemas on every startup after an upgrade. `blake3` is already a workspace dependency in `lithos-core` and is stable, fast, and cryptographically sound.

---

### F-10 — `process_changed` Does Not Actually Skip Unchanged Schemas

**Severity**: High — incremental resolution is a no-op
**File**: `resolver.rs:64-78`

`process_changed` accepts `existing_metadata: &[ResolutionMetadata]` and populates `file_mtimes`, but never calls `is_stale()` on any schema. Every schema passed in is resolved unconditionally. The incremental skipping logic is completely absent. The method is a façade that provides no performance benefit over `process`.

---

## Part 2: Semantic Bugs

### F-11 — `Schema::new` Fires `SchemaCreated` on Every Resolution Pass

**Severity**: High
**File**: `aggregate.rs:119-123`

`Schema::new` is called by `resolve_single` every time a schema is resolved — including re-resolution of schemas that already exist in the DB. This means re-resolving an unchanged schema after a bank version bump fires a spurious `SchemaCreated` event. With an event-driven architecture, this creates false create notifications for existing schemas.

The fix requires two constructors:
- `Schema::new(id, name, properties)` → for genuinely new schemas → emits `SchemaCreated` + `SchemaResolved`
- `Schema::resolve(id, name, properties)` → for re-resolution of existing schemas → emits `SchemaResolved` only

---

### F-12 — `PropertyBank::register` Emits Event on Idempotent Re-Registration

**Severity**: Medium
**File**: `bank.rs:161-163`

When a property with an already-registered ID is passed to `register`, the code takes the `Entry::Occupied` branch and emits a `PropertyBankUpdated` event even though nothing changed. An idempotent no-op should produce no domain events.

---

### F-13 — `Schema::get` and `Schema::has` Use Linear Scan Despite Sorted Invariant

**Severity**: Medium — O(n) where O(log n) is trivial
**File**: `aggregate.rs:173, 180`

Properties in a resolved `Schema` are always sorted by name (the resolver and merge sort both guarantee this). Both `get` and `has` use `iter().find()` and `iter().any()` respectively — O(n) linear scans. They should use `binary_search_by(|p| p.name().as_str().cmp(name.as_str()))`.

---

### F-14 — UUID-to-String Heap Allocation on Every DB Read

**Severity**: Medium — per-read allocation in hot query paths
**File**: `adapter/query.rs:37, 45, 51, 84`

Every query call uses `id.into_uuid().to_string()` — a 36-byte heap allocation per call. AGENTS.md explicitly flags this as an anti-pattern. The fix: use `uuid::fmt::Hyphenated::encode_lower` with a stack-allocated `[u8; 36]` buffer, then pass a `&str` to the DB method.

---

### F-15 — Two Duplicate "Parent Not Found" Error Variants

**Severity**: Low — confusing API
**File**: `error.rs:69-73`

```rust
ParentSchemaNotFound(String),  // from earlier development
ParentNotFound(String),        // the one actually used
```

Both exist and both have the same semantic meaning. `ParentSchemaNotFound` is never constructed in the current codebase. Remove it.

---

### F-16 — `SchemaError::Storage(String)` and `SchemaError::Resolver(String)` Are Untyped Catch-Alls

**Severity**: Low
**File**: `error.rs:77, 163`

Both variants carry an untyped `String`, discarding error structure. `SchemaCommandError::Storage(DbError)` is correctly typed. `SchemaError::Storage` should either be removed (all storage errors go through `SchemaCommandError`) or converted to hold `DbError`. `SchemaError::Resolver` should be removed and its use-sites replaced with `ValidationFailed` or `ParentNotFound`.

---

### F-17 — No `SchemaUpdated`, `SchemaDeleted`, or `PropertyRegistered` Events

**Severity**: Medium — event-driven architecture cannot react to schema lifecycle
**File**: `events.rs`

Only `SchemaCreated` and `PropertyBankUpdated` are defined. Missing:
- `SchemaResolved` — emitted on every resolution (create or re-resolve)
- `SchemaDeleted` — when a schema is removed
- `PropertyRegistered` — when a property is added to the bank
- `PropertyBankLoaded` — when the full bank is loaded/reloaded

`PropertyBankUpdated` is too coarse for event-driven consumers that need to know which specific property changed.

---

### F-18 — `DateSpec` Does Not Validate the Format String at Construction

**Severity**: Medium
**File**: `property_spec.rs:221-230`

`DateSpec::try_new` only checks that `format` is non-empty. An invalid strftime string (e.g. `"%Q%W%X"`) passes construction and silently fails every date validation at runtime with a confusing error. The format should be probed at construction time by attempting to format a known date.

---

### F-19 — `save_with_metadata` and `save_batch` Redundancy in Port and Command

**Severity**: Medium — two code paths with divergent semantics
**File**: `ports.rs:26-48`, `command.rs:52-93`, `adapter/command.rs:41-117`

`save_with_metadata` is a single-schema version of `save_batch`. The port, `Command` struct, and `CommandAdapter` each implement both. `save_with_metadata` in the adapter is non-atomic (F-08); `save_batch` is atomic. Removing `save_with_metadata` from the port eliminates the redundancy and forces all writes through the atomic path.

---

### F-20 — `PropertyBank` Has No `update` or `remove` Method

**Severity**: Medium — bank is immutable once built; re-load requires rebuild
**File**: `bank.rs`

If a user edits `property_bank.json` and removes or renames a property, the current `PropertyBank` cannot be updated incrementally. The only option is to discard the existing bank and rebuild from scratch on each load. The append-only constraint is undocumented. At minimum, add a doc comment stating this is intentional; at most, add a `remove` method that maintains index consistency.

---

### F-21 — No Application Service / End-to-End Ingestion Pipeline

**Severity**: Critical — no path from vault files to domain state
**File**: `application/` (empty)

The archived `SchemaIngestionService` was a stub with TODO comments and no actual resolution. There is currently no code path that:
1. Reads `property_bank.json` from the vault
2. Scans the schemas directory for `.json` files
3. Resolves schemas through `SchemaResolver`
4. Persists the results via `Command`

All the domain components exist but are not connected.

---

### F-22 — `RawPropertyBankEntry` Missing from `raw.rs` (name from map key, not field)

**Severity**: Critical
**File**: `raw.rs`

The property bank format uses the property name as the JSON object **key**, not a field inside the value. A `RawPropertyBankEntry` struct must not include a `name` field — the name is provided by the map iterator when deserializing `HashMap<Box<str>, RawPropertyBankEntry>`. The `required` field is also absent from bank entries (it's a schema-level concern per the meta-schema). Both of these were missed in the initial design.

---

### F-23 — `$ref` Override Validation Is Missing

**Severity**: Medium
**File**: `resolver.rs`

When `$ref` override fields are added (see Plan A4), the resolver must validate that override fields are compatible with the base property's type. Applying a `format` override to a `string` property, or an `options` override to a `number` property, should produce a validation error, not silently succeed. This validation is currently absent because override fields don't exist yet.

---

### F-24 — Ingestion Adapter (`Ingestor`) Should Not Hold a `Query` Port

**Severity**: Architecture — separation of concerns
**File**: (planned `adapter/ingestion.rs`)

ID assignment (looking up existing schema IDs by name) requires reading from the DB. If the `Ingestor` adapter holds a `Query` port, it becomes an orchestrator, not a focused file-translation adapter. The correct split:

- **`Ingestor`**: `FileSource` + `Paths` → `(RawPropertyBank, Vec<RawSchema>)`. Pure file-to-raw translation. No DB access.
- **`Query<Q>`**: already exposes `find_property_bank`, `list_metadata`, and the new `list_name_id_pairs` method for all read-side needs. No separate catalog adapter needed.
- **`SchemaService.load`**: calls `Ingestor` → `Query<Q>` (via `batch_read`) → generates new IDs → `SchemaResolver` → `Command`. Thin orchestration only.

---

## Part 3: Implementation Plan

### Group A — Raw Layer Redesign (Blocking All Other Work)

#### A1. Redesign `RawSchema`

Remove `id: Uuid`. Change `properties` to `HashMap<Box<str>, RawPropertyEntry>` (name is map key). Change `excludes` from `BTreeSet<Box<str>>` to `Vec<Box<str>>` (minor style improvement — serde correctly deserializes JSON arrays into either type, but `Vec` is more idiomatic for a small list where insertion order is not guaranteed to matter). Result:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    pub name: Box<str>,
    pub extends: Option<Box<str>>,
    #[serde(default)]
    pub excludes: Vec<Box<str>>,
    pub properties: HashMap<Box<str>, RawPropertyEntry>,
}
```

#### A2. Add `RawPropertyEntry` Enum

Replaces `RawProperty`. Discriminated by `$ref` vs `type` presence via `#[serde(untagged)]` (Ref is tried first because it has a required `$ref` field that Inline never has):

```rust
#[serde(untagged)]
pub enum RawPropertyEntry {
    Ref(RawPropertyRef),
    Inline(RawPropertyInline),
}
```

#### A3. Redesign `RawPropertyInline`

Remove `id` and `name` (both come from the map key / are assigned at ingestion). Rename `array` → `multi`. Keep `required` (valid in inline schema definitions per the meta-schema `CommonFields`). Flatten the spec so `type` and type-specific fields are at the same JSON level as `required`/`multi`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multi: bool,
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}
```

`RawPropertySpec` retains its `#[serde(tag = "type")]` internal tag. The `flatten` + internal tag combination is supported by serde.

#### A4. Redesign `RawPropertyRef`

Add all possible override fields. **No `type` override** — per meta-schema, `$ref` properties never change the base type. Override fields are `Option` because they are all optional:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    #[serde(rename = "$ref")]
    pub ref_path: Box<str>,
    // Cardinality/multiplicity overrides
    pub required: Option<bool>,
    pub multi: Option<bool>,
    // String-type overrides
    pub options: Option<RawOptions>,
    pub pattern: Option<Box<str>>,
    // Number-type overrides
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    // Date-type overrides
    pub format: Option<Box<str>>,
    // File-type overrides
    pub directory: Option<Box<str>>,
    pub file_class: Option<Box<str>>,
}
```

#### A5. Add `RawOptions` and `RawOptionEntry`

The three options modes from the meta-schema, discriminated via `#[serde(untagged)]`:

```rust
#[serde(untagged)]
pub enum RawOptions {
    List(Vec<Box<str>>),                       // Mode 1: ["a", "b"]
    Map(BTreeMap<Box<str>, Box<str>>),          // Mode 2: {"1": "val", "2": "val2"}
    Rich(Vec<RawOptionEntry>),                  // Mode 3: [{value, label?, order?}]
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RawOptionEntry {
    pub value: Box<str>,
    pub label: Option<Box<str>>,
    pub order: Option<u32>,
}
```

Serde `untagged` disambiguation order: `Rich` before `List` because `Vec<RawOptionEntry>` (objects) is distinct from `Vec<Box<str>>` (strings). `Map` is a JSON object, distinct from both arrays. Serde tries variants in declaration order; `List` and `Rich` are both arrays but items differ in type (string vs object), so serde can distinguish them.

#### A6. Add `RawPropertyBank` and `RawPropertyBankEntry`

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBank {
    pub properties: HashMap<Box<str>, RawPropertyBankEntry>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBankEntry {
    // name = map key; required NOT present (bank is schema-agnostic)
    #[serde(default)]
    pub multi: bool,
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}
```

#### A7. Update `StringSpecDef`: rename `enum_values` → `options`, use `RawOptions`

```rust
pub struct StringSpecDef {
    pub options: Option<RawOptions>,   // was: enum_values: Option<Vec<Box<str>>>
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<Box<str>>,
}
```

Update `RawPropertySpec::try_into_validated` to normalize `RawOptions` → `Vec<OptionEntry>` (sorted by display order, preserving labels).

#### A8. Update `StringSpec` to store `OptionEntry` values

```rust
pub struct OptionEntry {
    pub value: Box<str>,
    pub label: Option<Box<str>>,   // preserved for UI consumers
}

pub struct StringSpec {
    options: Option<Vec<OptionEntry>>,  // sorted by display order
    length: Bounds<usize>,
    pattern: Option<Box<str>>,
}
```

`RawOptions` → `Vec<OptionEntry>` normalization:
- **Mode 1** (`List`): entries have `value = item`, `label = None`, array order preserved
- **Mode 2** (`Map`): parse integer keys, sort by key value, entries have `value = map_value`, `label = None`
- **Mode 3** (`Rich`): sort by `order` field if present (then array position), entries have `value` and `label`

Validation only checks `value` membership (`options.iter().any(|e| e.value.as_ref() == value)`).

#### A9. Fix `PropertyRef::parse` for vault `$ref` format

Add `"property_bank#/<name>"` handling. Remove the stale `"#/properties/<name>"` prefix (no vault files use it). Updated parsing order:

```rust
// 1. Actual vault format
if let Some(name) = reference.strip_prefix("property_bank#/") {
    return Ok(Self::ByName(PropertyName::try_from(name)?));
}
// 2. Programmatic UUID format
if let Some(id_str) = reference.strip_prefix("$bank:") { ... }
// 3. Plain UUID
if let Ok(id) = Uuid::parse_str(reference) { ... }
// 4. Plain name (fallback)
Ok(Self::ByName(PropertyName::try_from(reference)?))
```

---

### Group B — ID Management

#### B1. Remove `id` from `RawSchema`

Covered in A1. No `Uuid` field in `RawSchema`.

#### B2. Update `SchemaResolver` to accept pre-assigned IDs

The public API changes to take `(SchemaId, RawSchema)` pairs, since the resolver no longer generates IDs:

```rust
pub fn process(
    &self,
    raw_schemas: Vec<(SchemaId, RawSchema)>,
) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>

pub fn process_changed<F>(
    &self,
    raw_schemas: Vec<(SchemaId, RawSchema)>,
    existing_metadata: &[ResolutionMetadata],
    parent_loader: F,
) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
where
    F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>
```

Internal `build_forest` and `resolve_one` are updated accordingly.

#### B3. Update `resolve_property` to handle `RawPropertyEntry`

`RawPropertyEntry::Inline(name, inline)`:
- Name comes from the map key (passed in alongside the entry)
- `PropertyId` is generated at ingestion time by the `Ingestor` (passed in)
- Convert `inline.required` → `Cardinality`, `inline.multi` → `Multiplicity`
- Call `inline.spec.try_into_validated()`

`RawPropertyEntry::Ref(name, ref_entry)`:
- Parse `ref_entry.ref_path` with updated `PropertyRef::parse`
- Look up base `Property` in bank
- Apply override fields on top:
  - `required` → override `Cardinality`
  - `multi` → override `Multiplicity`
  - Type-specific overrides: match on `base.spec()`, merge overrides into a new spec
  - Validate that override fields are compatible with base type (e.g. `format` only valid on `Date`, `options`/`pattern` only on `String`)
- Return new `Property` with overridden fields (preserving the bank property's `PropertyId`)

---

### Group C — Events Redesign

#### C1. New events in `events.rs`

```rust
pub enum Events {
    // Schema lifecycle
    SchemaCreated(SchemaCreated),         // first resolution (new ID)
    SchemaResolved(SchemaResolved),       // every resolution pass
    SchemaDeleted(SchemaDeleted),         // schema removed from vault
    // Property bank lifecycle
    PropertyRegistered(PropertyRegistered), // single new property added
    PropertyBankLoaded(PropertyBankLoaded), // full bank loaded/reloaded
}

pub struct SchemaResolved { pub id: SchemaId, pub name: SchemaName, pub timestamp: Timestamp }
pub struct SchemaDeleted  { pub id: SchemaId, pub name: SchemaName, pub timestamp: Timestamp }
pub struct PropertyRegistered { pub id: PropertyId, pub name: PropertyName, pub timestamp: Timestamp }
pub struct PropertyBankLoaded { pub property_count: usize, pub bank_version: BankVersion, pub timestamp: Timestamp }
```

Remove `PropertyBankUpdated` (too coarse).

#### C2. Fix `Schema` constructors

```rust
impl Schema {
    /// For genuinely new schemas: emits SchemaCreated + SchemaResolved.
    pub fn new(id: SchemaId, name: SchemaName, properties: Vec<Property>)
        -> Result<Self, SchemaError>

    /// For re-resolution of existing schemas: emits SchemaResolved only.
    pub fn resolve(id: SchemaId, name: SchemaName, properties: Vec<Property>)
        -> Result<Self, SchemaError>
}
```

The `SchemaService` determines which to call based on the `list_name_id_pairs` lookup: new `SchemaId` (name not found in DB) → `Schema::new`; reused `SchemaId` (name found in DB) → `Schema::resolve`.

#### C3. Fix `PropertyBank::register` event emission

In the `Entry::Occupied` branch: return `Ok(())` with no event. In `Entry::Vacant`: emit `PropertyRegistered { id, name, timestamp }` instead of `PropertyBankUpdated`.

---

### Group D — Correctness Fixes

#### D1. Replace `SchemaHash::DefaultHasher` with blake3

`blake3` is already a workspace dependency in `lithos-core`. Use it for a stable, cross-version hash:

```rust
pub fn compute(schema: &Schema) -> Self {
    let mut hasher = blake3::Hasher::new();
    hasher.update(schema.name().as_bytes());
    for prop in schema.properties() {
        hasher.update(prop.name().as_bytes());
        // Canonical 1-byte encoding for cardinality and multiplicity
        hasher.update(&[prop.cardinality() as u8, prop.multiplicity() as u8]);
        // Stable spec bytes: type discriminant + sorted field values
        prop.spec().hash_into_blake3(&mut hasher);
    }
    let hash_bytes = hasher.finalize();
    Self::from_u64(u64::from_le_bytes(
        hash_bytes.as_bytes()[..8].try_into().expect("blake3 output >= 8 bytes")
    ))
}
```

`PropertySpec::hash_into_blake3` feeds the spec type discriminant and all constraint values (as little-endian bytes) into the hasher in a stable, canonical order.

An ADR must be written documenting this choice (blake3 over rkyv-bytes, reason: rkyv format is a persisted-format contract; changing endianness/alignment features would invalidate all stored hashes).

#### D2. Remove `save_with_metadata` from port; consolidate to `save_batch`

- Remove `fn save_with_metadata` from `ports::Command` trait
- Remove `save_with_metadata` implementation from `CommandAdapter`
- In `command::Command<C>` struct: add `save_one` convenience method that delegates to `save_batch(&[(schema, metadata)])`
- All test code updated to use `save_one` or `save_batch` directly

#### D3. Implement actual incremental skipping in `process_changed`

In `build_forest`, when `existing_metadata` is provided:
1. For each raw schema, check if existing metadata exists for its `SchemaId`
2. Retrieve current `file_mtime` from the raw schema's source (passed in alongside the raw)
3. If `!metadata.is_stale(current_bank_version, current_parent_hash, current_file_mtime)`: skip resolution, load from DB via `parent_loader`, insert into `resolved_cache`
4. Only stale schemas enter the resolution pipeline

This requires the resolver to receive file modification times alongside the raw schemas: `Vec<(SchemaId, RawSchema, Option<Timestamp>)>`.

---

### Group E — Performance

#### E1. Binary search in `Schema::get` and `Schema::has`

```rust
pub fn get(&self, name: &PropertyName) -> Option<&Property> {
    self.properties
        .binary_search_by(|p| p.name().as_str().cmp(name.as_str()))
        .ok()
        .map(|i| &self.properties[i])
}

pub fn has(&self, name: &PropertyName) -> bool {
    self.properties
        .binary_search_by(|p| p.name().as_str().cmp(name.as_str()))
        .is_ok()
}
```

#### E2. Use existing `get_owned_by_uuid` in query adapter

`Database::get_owned_by_uuid()` already exists in `db/reader.rs` and handles UUID-to-key conversion with a stack buffer internally. The fix is simply to use it:

```rust
// In QueryAdapter::find_by_id:
fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
    self.db.get_owned_by_uuid(SCHEMA_BY_ID, id.into_uuid())
}

// In QueryAdapter::find_metadata_by_id:
fn find_metadata_by_id(&self, id: SchemaId) -> Result<Option<ResolutionMetadata>, Self::Error> {
    self.db.get_owned_by_uuid(SCHEMA_METADATA, id.into_uuid())
}

// In QueryAdapter::find_property_bank:
fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
    self.db.get_owned_by_uuid(PROPERTY_BANK, PropertyBankId::singleton().into_uuid())
}

// In QueryAdapter::with_archived_by_id:
fn with_archived_by_id<F, R>(&self, id: SchemaId, f: F) -> Result<Option<R>, Self::Error>
where F: for<'a> FnOnce(Self::Archived<'a>) -> R,
{
    self.db.get_by_uuid::<Schema, _, R>(SCHEMA_BY_ID, id.into_uuid(), f)
}
```

---

### Group F — Validation Gaps

#### F1. `DateSpec` format validation at construction

Probe the format string at `try_new` time:

```rust
pub fn try_new(format: &str) -> Result<Self, SchemaError> {
    if format.is_empty() {
        return Err(SchemaError::InvalidDateFormat("Format cannot be empty".into()));
    }
    // Probe: attempt to format a known date with this format string
    let probe = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .expect("static date");
    let result = probe.format(format).to_string();
    // If format is valid, result is non-empty; if invalid, chrono panics
    // Use a safe wrapper:
    if chrono::NaiveDate::parse_from_str(&result, format).is_err()
        && chrono::NaiveDateTime::parse_from_str(&result, format).is_err()
    {
        return Err(SchemaError::InvalidDateFormat(
            format!("Format string '{format}' is not a valid strftime pattern")
        ));
    }
    Ok(Self { format: format.into() })
}
```

---

### Group G — Error Cleanup

#### G1. Remove `ParentSchemaNotFound` from `SchemaError`

Keep `ParentNotFound`. Remove `ParentSchemaNotFound`. Update all match arms.

#### G2. Remove or type `SchemaError::Storage` and `SchemaError::Resolver`

- `SchemaError::Storage(String)` → remove; storage errors propagate through `SchemaCommandError::Storage(DbError)`
- `SchemaError::Resolver(String)` → remove; replace call sites with `SchemaError::ValidationFailed` or `SchemaError::ParentNotFound`

---

### Group H — New Adapters

#### H1. `schema::adapter::Ingestor<S: FileSource>`

**Pure file-to-raw translation. No DB access. No ID assignment.**

```rust
pub struct Ingestor<S: FileSource> {
    source: S,
    paths: Paths,   // from Config — provides schemas_dir and property_bank filename
}

impl<S: FileSource> Ingestor<S> {
    /// Load and deserialize property_bank.json → RawPropertyBank.
    ///
    /// Uses paths.property_bank_path() to locate the file.
    pub fn load_raw_property_bank(&self)
        -> Result<RawPropertyBank, IngestionError>;

    /// Scan schemas dir for all .json files (excluding property bank),
    /// deserialize each → RawSchema. Returns (RawSchema, file_mtime).
    ///
    /// Uses paths.schema.schemas_dir() for the directory.
    /// Excludes the property bank filename.
    pub fn scan_raw_schemas(&self)
        -> Result<Vec<(RawSchema, Option<Timestamp>)>, IngestionError>;
}
```

The `Ingestor` does NOT assign IDs. It does NOT query the DB. It returns raw deserialized data and file modification times (for staleness tracking).

#### H2. No standalone catalog adapter — extend `Query<Q>` instead

`Query::list()` already returns `Vec<Schema>` from which `(name, id)` pairs can be extracted. However, deserialising full `Schema` objects (including all properties) just to obtain names and IDs is wasteful for large vaults. The lean solution is a new focused method on the existing `Query<Q>` struct that reads only the lightweight `SCHEMA_ID_BY_NAME` index table:

```rust
impl<Q: schema_ports::Query> Query<Q> {
    /// Returns (SchemaName, SchemaId) pairs for all known schemas.
    /// Reads only the name-index table — does not deserialise full Schema objects.
    pub fn list_name_id_pairs(
        &self,
    ) -> Result<Vec<(SchemaName, SchemaId)>, SchemaQueryError>
}
```

Add the corresponding port method:

```rust
// ports::Query
fn list_name_id_pairs(&self) -> Result<Vec<(SchemaName, SchemaId)>, Self::Error>;
```

Implemented in `QueryAdapter` by iterating the `SCHEMA_ID_BY_NAME` table (key = name `&str`, value = `SchemaId`). The `SchemaService` builds its `HashMap<SchemaName, SchemaId>` from this result:

```rust
let existing: HashMap<SchemaName, SchemaId> = query
    .list_name_id_pairs()?
    .into_iter()
    .collect();
```

`find_property_bank` and `list_metadata` are already present on `Query<Q>` and need no changes. No separate adapter type is required.

**Note**: `ports::Query` already has `lookup_id_by_name(&self, name: &SchemaName)` for single-name lookups (one DB read per call). `list_name_id_pairs` is a bulk operation that scans the entire `SCHEMA_ID_BY_NAME` index in one pass — necessary for the `SchemaService` to preload all existing IDs without N separate round-trips.

#### H3. `PropertyBank::from_raw`

```rust
impl PropertyBank {
    /// Build a PropertyBank from raw vault data.
    /// Preserves existing property IDs by name when `existing` is provided.
    /// New properties (not found in existing by name) get generated IDs.
    pub fn from_raw(
        raw: RawPropertyBank,
        existing: Option<&PropertyBank>,
    ) -> Result<Self, SchemaError>
}
```

For each entry in `raw.properties`:
1. Look up existing `PropertyId` by name in `existing.get_by_name(name)` if `existing` is `Some`
2. If found: reuse the `PropertyId`
3. If not found: `PropertyId::new()`
4. Build `Property` from `RawPropertyBankEntry` + resolved ID + name
5. Register into new `PropertyBank`

---

### Group I — Port Updates

#### I1. Remove `save_with_metadata` from `ports::Command` (see D2)

#### I2. Add `list_name_id_pairs` to `ports::Query`

```rust
fn list_name_id_pairs(&self) -> Result<Vec<(SchemaName, SchemaId)>, Self::Error>;
```

Implemented in `QueryAdapter` by scanning `SCHEMA_ID_BY_NAME` — no full `Schema` deserialisation. Used by `SchemaService` to build its existing-ID map without loading all properties.

**Note**: `ports::Query` already has `lookup_id_by_name` for single lookups; `list_name_id_pairs` is the bulk variant for preloading all name→ID mappings in one pass.

#### I3. Add `batch_read` to `ports::Query` via a new `BatchReader` abstraction

Add a `BatchReader` struct to `db/reader.rs` (parallel to `BatchWriter` in `writer.rs`) that wraps `redb::ReadTransaction` privately and exposes typed read methods (`get_owned`, `get`, `list_owned`). Update `Database::batch_read` to pass `&BatchReader` to the closure rather than `&redb::ReadTransaction`:

```rust
// db/reader.rs
pub struct BatchReader {
    tx: redb::ReadTransaction,  // private — never exposed
}

impl BatchReader {
    pub(super) fn new(tx: redb::ReadTransaction) -> Self { Self { tx } }

    pub fn get_owned<V>(&self, table: TableDefinition<&str, &[u8]>, key: &str)
        -> Result<Option<V>, DbError> { ... }

    pub fn get<V, F, R>(&self, table: ..., key: &str, f: F)
        -> Result<Option<R>, DbError> { ... }

    pub fn list_owned<V>(&self, table: ...) -> Result<Vec<V>, DbError> { ... }
}
```

Port method (no redb type exposed):

```rust
// ports::Query
fn batch_read<R, F>(&self, f: F) -> Result<R, Self::Error>
where
    F: FnOnce(&crate::db::BatchReader) -> Result<R, Self::Error>;
```

`QueryAdapter` implements this directly by delegating to `self.db.batch_read(f)`. The `SchemaService` (Group J) uses this to execute `list_name_id_pairs`, `find_property_bank`, and `list_metadata` in a single read transaction without any redb types appearing at the context or service layer.

---

### Group J — Schema Service (Planned, Not Implemented Here)

The `SchemaService` lives in `application/` and will have a `load` method that sequences:

1. `Ingestor::load_raw_property_bank()` → `RawPropertyBank`
2. `Ingestor::scan_raw_schemas()` → `Vec<(RawSchema, Option<Timestamp>)>`
3. `query.batch_read(|tx| { ... })` — single read transaction for all pre-resolution DB reads:
   - `list_name_id_pairs()` → `Vec<(SchemaName, SchemaId)>` — build existing-ID map
   - `find_property_bank()` → `Option<PropertyBank>` — existing bank for ID preservation
   - `list_metadata()` → `Vec<ResolutionMetadata>` — for staleness checking
4. `PropertyBank::from_raw(raw_bank, existing_bank)` → `PropertyBank`
5. For each raw schema: look up name in existing-ID map; reuse `SchemaId` or generate new; determine `Schema::new` vs `Schema::resolve`
6. `SchemaResolver::process_changed(id_raw_mtime_triples, existing_metadata, parent_loader)` → `Vec<(Schema, ResolutionMetadata)>`
7. `Command::save_batch(results)` + `Command::save_property_bank(bank)`

Steps 1–2 (file I/O) and step 3 (DB reads) are independent and could be parallelised. Steps 4–7 are sequential. The service has no business logic — it only sequences adapters and domain services. It is thin by design.

---

## Part 4: Execution Order

Dependencies flow from raw types (A) through ID assignment (B) to domain fixes (C, D) then new adapters (H):

```
A5 (RawOptions, RawOptionEntry)
  → A7 (StringSpecDef: enum_values→options, use RawOptions)
  → A8 (StringSpec: OptionEntry, preserve labels)
  → A3 (RawPropertyInline: remove id/name, multi rename, flatten spec)
  → A4 (RawPropertyRef: add override fields)
  → A6 (RawPropertyBank + RawPropertyBankEntry)
  → A1 (RawSchema: no id, properties as map)
  → A2 (RawPropertyEntry enum)
  → A9 (PropertyRef::parse: add property_bank# prefix)

B1 (id removed from RawSchema, covered by A1)
  → B2 (SchemaResolver accepts (SchemaId, RawSchema, Option<Timestamp>) tuples)
  → B3 (resolve_property handles RawPropertyEntry with override logic)

C1 (new events: SchemaResolved, SchemaDeleted, PropertyRegistered, PropertyBankLoaded)
  → C2 (Schema::resolve constructor, Schema::new emits both events)
  → C3 (PropertyBank::register: no event on idempotent; emit PropertyRegistered)

D1 (SchemaHash: blake3)     [independent]
D2 (remove save_with_metadata from port/adapter)    [independent]
D3 (process_changed: actual staleness check)     [depends on B2]

E1 (Schema::get/has: binary search)    [independent]
E2 (UUID stack buffer in query adapter)    [independent]

F1 (DateSpec format validation at try_new)    [independent]

G1 (remove ParentSchemaNotFound)    [independent]
G2 (remove SchemaError::Storage/Resolver)    [independent]

I1 (remove save_with_metadata from ports::Command)    [depends on D2]
I2 (add list_name_id_pairs to ports::Query)    [independent]
I3 (add batch_read to ports::Query and Query<Q>)    [independent]

H3 (PropertyBank::from_raw)    [depends on A6, C3]
H1 (Ingestor adapter)    [depends on A1-A9]

ADR for blake3 SchemaHash    [after D1]
```

---

## Part 5: What Is NOT Changing

- **`SchemaResolver` stays in the domain layer** — it is a pure in-memory computation over domain types with no I/O. It is correctly placed.
- **`PropertyBank` index structure** (Vec + dual HashMap) stays as-is with append-only constraint documented. `remove`/`update` are a future concern.
- **`RawPropertySpec` enum** stays as-is (only `StringSpecDef` is modified inside it).
- **The CQRS port split** (`Command` / `Query`) stays — it is correctly designed.
- **Context isolation** — the schema context does not import note or template; this boundary is respected.
- **`Ingestor` does not take a `Query` port** — the `SchemaService` reads all required DB state via `Query<Q>::batch_read` before calling the resolver; no separate catalog adapter is needed.

---

## Part 6: Resolver Refactor — Intermediate Design (Superseded by Part 7)

**Date**: 2026-02-18
**Status**: SUPERSEDED — Part 7 replaces the component design described here.

The intermediate design replaced `ResolutionContext` with free functions and `InheritanceForest`. This was an improvement but still conflated property-bank dereferencing, inheritance extension, and final assembly inside a single `SchemaResolver`. Part 7 documents the final decomposed design.

The following individual decisions from this part remain valid and are carried forward:

- **R-03 through R-09** (`Raw*Spec` rename, flattened overrides, `apply_overrides` methods, `From<bool>` impls) — implemented and committed, still correct.
- **R-10** (type-change override rejection via `is_type_mismatch`) — carried into `Dereferencer`.
- **R-11** (error variants `InvalidPropertyRef`, `PropertyTypeMismatch`) — implemented and committed, still correct.
- **R-01, R-02, R-12** (`ResolutionContext` elimination, `InheritanceForest`, staleness ordering) — superseded by the `Dereferencer`/`Extender`/`Resolver` split in Part 7.

---

## Part 7: Pipeline Decomposition — Full Redesign

**Date**: 2026-02-19
**Scope**: `resolver.rs`, `dereferencer.rs` (new), `extender.rs` (new), `adapter/stored.rs` (new),
`aggregate.rs`, `ports.rs`, `adapter/query.rs`, `adapter/command.rs`, `raw.rs`, `application/schema.rs` (new)

### Core Insight

The `SchemaResolver` conflated three distinct concerns. Each maps to a separate, independently-testable component with a clean I/O contract. Staleness checking moves entirely out of the resolver pipeline and into the `Query` port.

### Execution Pipeline

```
Ingestor
  → Vec<(RawSchema, Option<Timestamp>)>

Query port (is_schema_stale, is_bank_stale)
  → partition: stale Vec<(SchemaId, RawSchema, Option<Timestamp>)>
             + fresh Vec<SchemaId>  (load from DB as known_parents)

Dereferencer  (holds &PropertyBank)
  → Vec<(SchemaId, DereferencedSchema)>
     PropertyBank no longer needed after this step.

Extender
  → SchemaTree  (nodes in topological order, cycles rejected)

Resolver
  → Vec<Schema>

SchemaService  (application/schema.rs)
  → saves Vec<Schema> via Command port using StoredSchema adapter type
```

---

### Component 1: `StoredSchema` — `schema/adapter/stored.rs`

`StoredSchema` is the rkyv-serialized storage representation of a schema. It lives in the adapter layer (`schema/adapter/stored.rs`), not in the domain. The domain `Schema` struct stays minimal: `id`, `name`, `properties`, `pending_events` — no storage metadata embedded.

`StoredSchema` carries all fields needed for staleness checking and tree reconstruction, eliminating the need for a separate `SCHEMA_METADATA` / `SCHEMA_RESOLUTION_RECORD` table entirely.

```rust
// schema/adapter/stored.rs
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub(crate) struct StoredSchema {
    pub id: SchemaId,
    pub name: Box<str>,               // flattened from SchemaName newtype
    pub parent_id: Option<SchemaId>,  // for SchemaTree reconstruction
    pub properties: Vec<StoredProperty>,
    pub bank_version: BankVersion,    // staleness: did the bank change?
    pub created_at: Timestamp,        // from file metadata at first ingestion
    pub modified_at: Timestamp,       // from file metadata, updated on re-ingestion
}

pub(crate) struct StoredProperty {
    pub id: Uuid,          // flattened from PropertyId newtype
    pub name: Box<str>,    // flattened from PropertyName newtype
    pub required: bool,    // flattened from Cardinality
    pub multi: bool,       // flattened from Multiplicity
    pub spec: PropertySpec, // already has rkyv derives; kept as-is
}
```

Conversions:
- `StoredSchema` → `Schema`: `TryFrom<StoredSchema> for Schema` — infallible in practice (all fields already validated at write time), returns `SchemaError` on name/property validation failure.
- `Schema` + metadata → `StoredSchema`: a `fn to_stored(schema: &Schema, parent_id, bank_version, created_at, modified_at) -> StoredSchema` free function in `stored.rs`, called by `CommandAdapter::save_batch`.

**Eliminates**: `ResolutionMetadata` struct, `SCHEMA_METADATA` table constant, `find_metadata_by_id` and `list_metadata` port methods.

---

### Component 2: Staleness — `ports::Query` additions

Two new methods on the `Query` trait. Both are implemented by `QueryAdapter` by reading the `StoredSchema` from `SCHEMA_BY_ID` and comparing fields directly — a single table read, no second table.

```rust
// ports::Query
fn is_schema_stale(
    &self,
    id: SchemaId,
    file_mtime: Option<Timestamp>,
    current_bank_version: BankVersion,
) -> Result<bool, Self::Error>;

fn is_bank_stale(
    &self,
    current_version: BankVersion,
) -> Result<bool, Self::Error>;
```

`is_schema_stale`: loads `StoredSchema` for `id`; returns `true` if no stored record exists, or if `stored.bank_version != current_bank_version`, or if `stored.modified_at < file_mtime` (file has changed since last ingestion).

`is_bank_stale`: loads the stored `PropertyBank`; returns `true` if no stored bank exists or its version differs from `current_version`.

---

### Component 3: `Dereferencer` — `schema/dereferencer.rs`

**Input**: `Vec<(SchemaId, RawSchema)>` (stale schemas only) + `&PropertyBank`
**Output**: `Vec<(SchemaId, DereferencedSchema)>`
**Visibility**: `pub(crate)`

```rust
pub(crate) struct DereferencedSchema {
    pub name: Box<str>,
    pub extends: Option<Box<str>>,
    pub excludes: Vec<Box<str>>,
    pub properties: Vec<Property>,  // fully initialized, sorted by name
}
```

For each `RawProperty` in the schema:
- `RawProperty::Inline(inline)` → `RawPropertySpec::try_into_validated()` → `Property::new(...)`
- `RawProperty::Ref(ref_entry)` → bank lookup by `PropertyRef` → apply overrides → `Property::new(...)`

Properties are sorted by name before storage so `Extender` and `Resolver` can use two-pointer merges.

**Private methods**:
- `deref_entry(&str, RawProperty) -> Result<Property, SchemaError>`
- `apply_ref_overrides(&Property, &RawPropertyRef) -> Result<Property, SchemaError>` — applies `required`/`multi` overrides and spec-level overrides; calls `is_type_mismatch`
- `is_type_mismatch(PropertySpecType, &RawPropertyRef) -> bool` — returns `true` if any override field belongs to a type incompatible with the base spec type (carries forward R-10)

After `Dereferencer`, the `PropertyBank` is no longer referenced anywhere in the pipeline.

---

### Component 4: `Extender` — `schema/extender.rs`

**Input**: `Vec<(SchemaId, DereferencedSchema)>` + `&HashMap<SchemaId, Schema>` (known-fresh parents pre-loaded from DB by `SchemaService`)
**Output**: `SchemaTree`
**Visibility**: `pub(crate)`

```rust
pub(crate) struct SchemaTree {
    roots: Vec<SchemaId>,                    // schemas with parent_id = None
    nodes: HashMap<SchemaId, SchemaNode>,    // O(1) lookup by ID
    order: Vec<SchemaId>,                    // topological order (roots first)
}

pub(crate) struct SchemaNode {
    pub id: SchemaId,
    pub name: Box<str>,
    pub own_properties: Vec<Property>,  // from DereferencedSchema, sorted
    pub excludes: Vec<Box<str>>,
    pub parent_id: Option<SchemaId>,
    pub children: Vec<SchemaId>,
}
```

`Extender::build` is the only public method. It:
1. Resolves `extends` name strings to `SchemaId`s (batch first, then `known_parents`)
2. Calls private `is_treelike()` — DFS cycle detection → `Err(SchemaError::CircularInheritance)` on cycle
3. Validates no missing parents → `Err(SchemaError::ParentNotFound)`
4. Populates `children` on each node
5. Emits `SchemaTree` with `roots` and `order` computed via Kahn's algorithm

`SchemaTree` exposes `fn nodes(&self) -> &[SchemaId]` returning `order` for iteration in dependency order, and `fn get(&self, id: SchemaId) -> Option<&SchemaNode>` for O(1) node lookup.

---

### Component 5: `Resolver` — `schema/resolver.rs` (rewritten)

**Input**: `&SchemaTree` + `&HashMap<SchemaId, Schema>` (known-fresh parents)
**Output**: `Vec<Schema>`
**Visibility**: `pub`

```rust
pub struct Resolver;

impl Resolver {
    pub fn resolve(
        tree: &SchemaTree,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<Vec<Schema>, SchemaError>
}
```

Walks `tree.nodes()` in topological order. For each `SchemaNode`:
1. Get parent's resolved `Vec<Property>` — from `resolved_cache` (in-batch parent resolved earlier) or `known_parents` (fresh DB parent) or `[]` (root)
2. Call private `merge_properties(parent_props, &node.own_properties, &node.excludes)` → sorted `Vec<Property>`
3. Construct `Schema::new(node.id, name, merged_props)` or `Schema::resolve(...)` depending on whether the ID existed in DB (determined by presence in `known_parents`)
4. Insert into `resolved_cache` for use by children

**`merge_properties`** is a private method on `Resolver`. It performs a two-pointer sorted merge: parent properties + node's own properties, child overrides same-named parent properties, excluded names filtered from parent. It is not exposed as `pub(crate)` to prevent bypassing the correct pipeline.

---

### Component 6: `SchemaService` — `application/schema.rs`

Thin orchestration only. Sequences adapters and domain components. Lives at `lithos-core/src/application/schema.rs`.

Full pipeline:
1. `Ingestor::load_raw_property_bank()` → `RawPropertyBank`
2. `Ingestor::scan_raw_schemas()` → `Vec<(RawSchema, Option<Timestamp>)>`
3. `query.batch_read(|tx| { list_name_id_pairs(), find_property_bank() })` — single read transaction
4. `PropertyBank::from_raw(raw_bank, existing_bank)` → `PropertyBank`
5. `query.is_bank_stale(bank.version())` — if stale, all schemas are stale regardless of mtime
6. For each raw schema: look up `SchemaId` by name (reuse or generate); call `query.is_schema_stale(id, mtime, bank_version)` to partition stale vs fresh
7. Load fresh schemas from DB into `known_parents: HashMap<SchemaId, Schema>`
8. `Dereferencer::deref(stale_schemas, &bank)` → `Vec<(SchemaId, DereferencedSchema)>`
9. `Extender::build(deref_schemas, &known_parents)` → `SchemaTree`
10. `Resolver::resolve(&tree, &known_parents)` → `Vec<Schema>`
11. `Command::save_batch(&schemas, metadata...)` + `Command::save_property_bank(&bank)`

---

### `raw.rs` rename — `RawPropertyEntry` → `RawProperty`

`RawPropertyEntry` is renamed to `RawProperty` everywhere. The now-redundant old `RawProperty` enum (structurally identical) is removed. `RawSchema.properties` type changes from `HashMap<Box<str>, RawPropertyEntry>` to `HashMap<Box<str>, RawProperty>`.

---

### Files Changed by Part 7

| File                        | Action      | What changes                                                                              |
| --------------------------- | ----------- | ----------------------------------------------------------------------------------------- |
| `schema/adapter/stored.rs`    | **Create**  | `StoredSchema`, `StoredProperty`, `to_stored(...)`, `TryFrom<StoredSchema> for Schema`       |
| `schema/adapter/command.rs`   | Modify      | Write `StoredSchema` to `SCHEMA_BY_ID`; remove all `SCHEMA_METADATA` writes                   |
| `schema/adapter/query.rs`     | Modify      | Read `StoredSchema`, convert to `Schema`; implement `is_schema_stale`, `is_bank_stale`        |
| `schema/adapter/mod.rs`       | Modify      | Declare `stored` submodule                                                                  |
| `schema/dereferencer.rs`      | **Create**  | `Dereferencer`, `pub(crate) DereferencedSchema`                                               |
| `schema/extender.rs`          | **Create**  | `Extender`, `pub(crate) SchemaTree`, `pub(crate) SchemaNode`                                  |
| `schema/resolver.rs`          | **Rewrite** | `Resolver` struct, `resolve()`, private `merge_properties()`; no bank, no staleness          |
| `schema/ports.rs`             | Modify      | Add `is_schema_stale`, `is_bank_stale`; remove `find_metadata_by_id`, `list_metadata`        |
| `schema/aggregate.rs`         | Modify      | Remove `ResolutionMetadata`, `SchemaHash`; keep `Schema` minimal                             |
| `schema/raw.rs`               | Modify      | Rename `RawPropertyEntry` → `RawProperty`; remove duplicate `RawProperty` enum               |
| `schema/mod.rs`               | Modify      | Remove `SCHEMA_METADATA` constant; add `dereferencer`, `extender` module declarations        |
| `schema/command.rs`           | Modify      | Update `save_batch` — metadata now embedded in `StoredSchema`, not a separate parameter       |
| `schema/query.rs`             | Modify      | Remove `list_metadata`, `find_metadata_by_id`; add `is_schema_stale`, `is_bank_stale`        |
| `application/schema.rs`       | **Create**  | `SchemaService` stub with `load` signature                                                    |
| `application/mod.rs`          | Modify      | Declare `schema` submodule                                                                    |

---

### Implementation Order for Part 7

```
(a) raw.rs: rename RawPropertyEntry → RawProperty, remove duplicate
      [purely mechanical; run verify]

(b) schema/adapter/stored.rs: define StoredSchema + StoredProperty + conversions
      [no other files touched; run verify]

(c) adapter/command.rs + adapter/query.rs: write/read StoredSchema;
    remove SCHEMA_METADATA references; implement is_schema_stale, is_bank_stale
      [run verify]

(d) ports.rs + query.rs + command.rs: add new trait methods; remove
    find_metadata_by_id, list_metadata from trait and Query<Q> impl
      [run verify]

(e) aggregate.rs + mod.rs: remove ResolutionMetadata, SchemaHash;
    remove SCHEMA_METADATA constant
      [run verify]

(f) schema/dereferencer.rs: implement Dereferencer and DereferencedSchema
      [run verify]

(g) schema/extender.rs: implement Extender, SchemaTree, SchemaNode
      [run verify]

(h) schema/resolver.rs: rewrite as pure Resolver; merge_properties private
      [run verify]

(i) application/schema.rs + application/mod.rs: SchemaService stub
      [run verify]
```

Each step ends with `mise run verify` before proceeding to the next.

---

## Part 8: Post-Part-7 Correctness Fixes & Domain Migration

**Date**: 2026-02-23
**Scope**: Timestamp bugs, `parent_id` domain migration, adapter correctness fixes
**Status**: IMPLEMENTED (commit `9e1d5307`)

### Background

Part 7 successfully implemented the `Dereferencer → Extender → Resolver` pipeline and `SchemaService` orchestration. Post-implementation review identified five correctness issues and one architectural improvement:

1. **Timestamp bug**: `created_at`/`modified_at` used wall-clock (`Timestamp::now()`) instead of filesystem times
2. **`parent_id` architectural issue**: Stored only in adapter layer, causing awkward tree lookup in `SchemaService`
3. **Delete atomicity**: `CommandAdapter::delete` used 3 separate transactions (non-atomic)
4. **Silent corruption**: `list_name_id_pairs` silently dropped invalid names via `filter_map(...ok())`
5. **Wrong error variants**: `find_by_id`/`list` mapped errors to `DbError::Transaction` instead of `Deserialization`
6. **Dead binding**: `if let Some(_existing)` never used the bound value

### Key Architectural Decision: `parent_id` in Domain

**Rationale**: `parent_id` is a domain fact about schema structure (inheritance relationship), not infrastructure metadata. It belongs in the `Schema` aggregate alongside `id`, `name`, and `properties`.

**Benefits**:
- Eliminates awkward `tree.get(schema.id()).and_then(|node| node.parent_id)` lookup
- Makes `Schema` self-describing (inheritance hierarchy is queryable)
- Enables future use cases (LSP queries, graph visualization, cycle detection at load)
- Clearer separation: domain owns structure, adapter owns operational metadata

**Storage remains monolithic**: `StoredSchema` keeps all fields (domain + metadata) in one table for single-query staleness checks. Normalization deferred pending real usage patterns.

---

### Changes Implemented

#### 8.1 — Domain: Add `parent_id` to `Schema`

**File**: `lithos-core/src/schema/aggregate.rs`

```rust
pub struct Schema {
    id: SchemaId,
    name: SchemaName,
    parent_id: Option<SchemaId>,  // ← NEW
    properties: Vec<Property>,
    pending_events: Vec<Events>,
}
```

Updated constructors:
- `Schema::new(id, name, parent_id, properties)` — emits `SchemaCreated`
- `Schema::reconstruct(id, name, parent_id, properties)` — for DB loads, no events

Added accessor: `pub const fn parent_id(&self) -> Option<SchemaId>`

#### 8.2 — Resolver: Pass `parent_id` to `Schema` Constructors

**File**: `lithos-core/src/schema/resolver.rs`

Threaded `node.parent_id` from `SchemaNode` (already available) to both `Schema::new()` and `Schema::resolve()` constructors.

#### 8.3 — Ingestor: Capture Filesystem Times

**File**: `lithos-core/src/schema/adapter/ingestor.rs`

**Before**: Returned `(RawSchema, Option<Timestamp>)` — only `modified` time

**After**: Returns `(RawSchema, Option<Timestamp>, Option<Timestamp>)` — `(raw, modified, created)`

Type alias renamed: `RawSchemaWithMtime` → `RawSchemaWithFileTimes`

Extracts both `std::fs::Metadata::modified()` and `Metadata::created()` (birthtime). `created()` can fail on unsupported platforms/filesystems; `.ok()` fallback handles gracefully.

#### 8.4 — StoredSchema: Filesystem + DB Timestamps

**File**: `lithos-core/src/schema/adapter/stored.rs`

**Before** (BUGGY):
```rust
pub created_at: Timestamp,   // wall-clock (wrong)
pub modified_at: Timestamp,  // wall-clock (wrong)
```

**After**:
```rust
pub created_at: Option<Timestamp>,   // fs birthtime (Metadata::created)
pub modified_at: Option<Timestamp>,  // fs mtime (Metadata::modified)
pub recorded_at: Timestamp,          // wall-clock at DB write
```

**Breaking change**: rkyv on-disk format changed. Pre-existing DB data becomes unreadable. Acceptable for pre-release project.

Updated `to_stored(...)` signature to accept new timestamp fields. `parent_id` now comes from `schema.parent_id()` (domain) instead of being passed separately.

#### 8.5 — SchemaRecord: Match StoredSchema Fields

**File**: `lithos-core/src/schema/ports.rs`

**Before**:
```rust
pub parent_id: Option<SchemaId>,  // ← removed (now in Schema)
pub created_at: Timestamp,
pub modified_at: Timestamp,
```

**After**:
```rust
pub created_at: Option<Timestamp>,
pub modified_at: Option<Timestamp>,
pub recorded_at: Timestamp,
```

#### 8.6 — SchemaService: Wire Correct Timestamps

**File**: `lithos-core/src/application/schema.rs`

**Changes**:
1. Thread `(modified, created)` from ingestor through staleness loop via `StaleSchema` helper struct
2. Remove tree lookup: `parent_id` now comes from `schema.parent_id()` directly
3. Capture `recorded_at = Timestamp::now()` once before `save_batch`
4. Look up file times for each resolved schema from `stale` vec and pass to `SchemaRecord::new`

#### 8.7 — QueryAdapter: Fix Staleness + Error Variants

**File**: `lithos-core/src/schema/adapter/query.rs`

**4 fixes**:

1. **`is_schema_stale`**: Handle `Option<Timestamp>` for `stored.modified_at`:
   ```rust
   if let (Some(stored_mtime), Some(file_mtime)) = (stored.modified_at, file_mtime)
       && stored_mtime.as_secs() < file_mtime.as_secs()
   ```

2. **`list_name_id_pairs`**: Return `DbError::Deserialization` on corrupt names instead of silently dropping

3. **`find_by_id`**: Use `DbError::Deserialization` (not `Transaction`) for conversion errors

4. **`list`**: Use `DbError::Deserialization` (not `Transaction`) for conversion errors

#### 8.8 — CommandAdapter: Atomicity + Dead Binding

**File**: `lithos-core/src/schema/adapter/command.rs`

**2 fixes**:

1. **Dead binding**: `if let Some(_existing)` → `.is_some()`

2. **Delete atomicity**: Wrap read + 2 deletes in `read_write_unit_of_work` for single atomic transaction:
   ```rust
   self.db.read_write_unit_of_work(|tx| {
       if let Some(stored) = tx.get_owned::<StoredSchema>(...)? {
           tx.delete(SCHEMA_ID_BY_NAME, stored.name.as_ref())?;
       }
       tx.delete(SCHEMA_BY_ID, id_key.as_str())?;
       Ok(())
   })
   ```

---

### Verification

All changes passed `mise run verify` at commit `9e1d5307`:
- ✅ All unit tests passing
- ✅ All doc tests passing
- ✅ No clippy warnings
- ✅ Code formatted correctly
- ✅ No `#[allow]` directives added
- ✅ Minimal `#[expect]` usage with detailed reasons

---

### Summary of Bugs Fixed

| Bug | Severity | Fix |
|---|---|---|
| Timestamps from wall-clock (not filesystem) | High | Use `Metadata::created()` / `modified()` |
| `parent_id` not in domain (awkward lookup) | Medium | Add to `Schema` aggregate |
| `delete` non-atomic (3 separate txs) | High | Use `read_write_unit_of_work` |
| `list_name_id_pairs` silent corruption | Medium | Return `Deserialization` error |
| Wrong error variants (2 places) | Low | Use `Deserialization` not `Transaction` |
| Dead `_existing` binding | Low | Use `.is_some()` |

---

## Part 9: ADRs Required

| Decision | File |
|---|---|
| Use blake3 for `SchemaHash` instead of `DefaultHasher` | `docs/adr/XXXX-schema-hash-blake3.md` |
| `Ingestor` is pure file-to-raw; `Query<Q>` handles all read-side DB needs | `docs/adr/XXXX-schema-ingestion-adapter-split.md` |
| `save_batch` is the sole write path; `save_with_metadata` removed | `docs/adr/XXXX-schema-command-save-batch-only.md` |
| `StoredSchema` in adapter layer replaces `ResolutionMetadata` and second table | `docs/adr/XXXX-stored-schema-adapter-layer.md` |
| Pipeline decomposed into `Dereferencer`, `Extender`, `Resolver` | `docs/adr/XXXX-pipeline-decomposition.md` |
| `parent_id` belongs in domain `Schema`, not adapter-only | `docs/adr/XXXX-parent-id-domain-migration.md` |
| Filesystem times (`created`/`modified`) + DB time (`recorded_at`) for timestamps | `docs/adr/XXXX-schema-timestamps-filesystem-vs-db.md` |

---

## Part 10: Comprehensive Adversarial Code Review & Meta-Schema Analysis

**Date**: 2026-02-26
**Reviewer**: BMad Master Agent (Claude)
**Scope**: Full schema module audit (~6,500 LOC) + JSON Schema meta-schemas
**Status**: Review complete, awaiting approval for implementation

---

### Executive Summary of Review

Conducted thorough adversarial review of the schema module focusing on:
1. **Unnecessary allocations** and performance bottlenecks
2. **Superfluous code** that adds cognitive load without value
3. **Edge cases** not covered by current tests
4. **Best practices** for metadata schema systems (industry research)
5. **Idiomatic Rust** adherence per project standards

**Overall Assessment**: The schema module has excellent architecture (port-based CQRS, domain isolation, zero-copy storage via rkyv) but contains **21 actionable issues** ranging from critical performance problems to missing best practices.

---

### Section 1: Critical Performance Issues

#### P-01 — Property Cloning in Inheritance Merge (CRITICAL)

**Severity**: 🔴 HIGH — O(N×M) allocations during schema resolution
**Location**: `resolver.rs:138-145`, property merge loop

**Problem**: Every schema resolution clones ALL parent and child properties during inheritance:

```rust
// resolver.rs line 138-145 (excerpt)
match (p_iter.peek(), c_iter.peek()) {
    (Some(&p), Some(&c)) => {
        // ...
        Self::push_unless_excluded(&mut result, p, excludes);  // p.clone()
        result.push(c.clone());  // c.clone()
    }
}
```

**Impact Calculation**:
- Example vault has 41 schemas, many with inheritance chains 3-4 deep
- Average schema has ~8 properties
- Property clone cost: 200-500 bytes each (name, spec, etc.)
- Total wasted allocation: ~64KB per vault load

**Evidence from code**:
- `Property` contains `Box<str>` names, `PropertySpec` with nested options/patterns
- `PropertySpec::String` can have `Vec<OptionEntry>` with labels (unbounded size)
- Every re-resolution (bank version bump) re-clones everything

**Recommendation**: Use `Rc<Property>` or `Arc<Property>` for shared ownership:

```rust
pub struct Schema {
    properties: Vec<Rc<Property>>,  // Zero-copy sharing
}

// In merge:
result.push(Rc::clone(&c));  // Just increments refcount
```

**Alternative**: Arena allocation for entire resolution pass, then copy-out final schemas.

---

#### P-02 — HashMap Allocations for Name Indexes (HIGH)

**Severity**: 🟠 MEDIUM-HIGH — N+M heap allocations per resolution
**Location**: `extender.rs:169-172`, `dereferencer.rs:111-113`

**Problem**: Resolution pipeline creates temporary `HashMap<Box<str>, SchemaId>` indexes:

```rust
// extender.rs:169-172
let mut name_to_id: HashMap<Box<str>, SchemaId> =
    HashMap::with_capacity(cap.saturating_add(known_parents.len()));
let mut id_to_name: HashMap<SchemaId, Box<str>> =
    HashMap::with_capacity(cap);
```

**Why this allocates unnecessarily**:
- `Box<str>` keys are cloned from `RawSchema.name` (already `Box<str>`)
- The comment "_`Box<str>: Borrow<str>` so `.get(&str)` works_" shows this is for API convenience
- Name validation happens in `SchemaName::new` earlier, so we're paying for safety we have

**Measurement**:
- 41 schemas × 2 HashMaps × ~50 bytes/entry = ~4KB
- Plus hash table overhead (load factor, buckets)

**Recommendation**: Use `&str` keys with explicit lifetimes OR `Arc<str>` for zero-copy sharing.

---

#### P-03 — PropertyBank Triple-Allocation Design (MEDIUM)

**Severity**: 🟡 MEDIUM — 3N data structures for N properties
**Location**: `bank.rs:81-88`

**Design**:
```rust
pub struct PropertyBank {
    id_index: HashMap<PropertyId, usize>,    // Index 1
    name_index: HashMap<PropertyName, usize>, // Index 2
    properties: Vec<Property>,                // Storage
}
```

**Evidence of overhead**:
```rust
// bank.rs:261-264
let idx = self.properties.len();
id_entry.insert(idx);                      // HashMap allocation
self.name_index.insert(name.clone(), idx); // HashMap + PropertyName clone!
self.properties.push(property);            // Vec push
```

**Impact**:
- PropertyName is cloned on EVERY registration (line 246, 263)
- For 1000 properties: ~24KB index overhead + clone costs

**Recommendation**:
- **Option A**: Single `HashMap<PropertyName, Property>` + `HashMap<PropertyId, PropertyName>` (Copy-able IDs)
- **Option B**: Accept the overhead but document it (already optimal for dual-index lookup)

**Note**: Current design gives O(1) lookup by both ID and name. If that's required, document the tradeoff.

---

#### P-04 — UUID-to-String Allocation in Query Paths (MEDIUM)

**Severity**: 🟡 MEDIUM — per-query heap allocation
**Location**: Multiple query methods (already partially fixed in codebase)

**Status**: ✅ **ALREADY FIXED** — Database has `get_owned_by_uuid` with stack buffer

**Finding**: Review shows this was flagged in F-14 of original review and fixed in Group E. Confirmed fixed in current code.

---

### Section 2: Superfluous / Unused Code

#### S-01 — PropertyRef Wrapper Adds Zero Value (MEDIUM)

**Severity**: 🟡 MEDIUM — Cognitive load without runtime benefit
**Location**: `property.rs:715-758`

**Problem**: `PropertyRef` is a newtype over `PropertyName`:

```rust
#[repr(transparent)]
pub struct PropertyRef(PropertyName);
```

**Why it exists**:
- Parse `property_bank#/<name>` format
- Type safety for references vs inline names

**Why it's superfluous**:
- Parsing happens ONCE during ingestion
- After `Dereferencer` completes, `PropertyRef` is NEVER used (comment in `dereferencer.rs:14`)
- Zero runtime cost but adds cognitive load

**Recommendation**: Remove `PropertyRef`. Use `PropertyName::try_from(stripped_ref_path)` directly in `Dereferencer`.

---

#### S-02 — SchemaError Has 18 Variants, Many Unused (LOW)

**Severity**: 🟢 LOW — Binary bloat
**Location**: `error.rs:22-163`

**Analysis**: Many error variants are **never constructed** in production code:
- `NotFound` — only in tests
- `AlreadyExists` — only in extender (rare duplicate name scenario)
- `InvalidFileClass` — only in FileSpec validation

**Why this matters**:
- Error enum bloat increases binary size
- Match exhaustiveness gets complex
- Hard to know which errors are "real" vs defensive

**Recommendation**:
1. Audit which errors actually occur in practice
2. Split into focused types: `ValidationError`, `StorageError`, `ResolutionError`
3. Use `#[non_exhaustive]` on error types expected to grow

---

#### S-03 — PropertyBank.all() Only Used in Tests (LOW)

**Severity**: 🟢 LOW — API bloat
**Location**: `bank.rs:421-424`

```rust
pub fn all(&self) -> impl Iterator<Item = &Property> {
    self.properties.iter()
}
```

**Evidence**: Grep shows only test/doc usage, never in production paths.

**Recommendation**: Move to `#[cfg(test)]` OR keep if intentional for future LSP queries.

---

### Section 3: Edge Cases & Test Coverage Gaps

#### E-01 — No Concurrency Tests for PropertyBank (HIGH)

**Severity**: 🔴 HIGH — Potential race conditions
**Location**: `bank.rs` (no concurrency tests)

**Critical scenario NOT tested**:
```rust
// Thread 1: Registering property
bank.register(prop1)?;

// Thread 2: Reading property (DURING registration)
bank.get_by_name(&name);  // What if HashMap resizes mid-read?
```

**Why this matters**:
- `PropertyBank` is `Send + Sync` (required by port traits)
- No documented thread-safety guarantees
- Concurrent schema resolution could race

**Recommendation**:
1. Add `RwLock` around internal state
2. **OR** document PropertyBank is single-threaded during construction
3. Add concurrency stress tests with `std::thread::spawn`

---

#### E-02 — NumberSpec Epsilon Uses Magic Constant (MEDIUM)

**Severity**: 🟡 MEDIUM — Fragile floating-point comparison
**Location**: `property_spec.rs:763`

```rust
const EPSILON: f64 = 1e-10;
// ...
if remainder > EPSILON && (step.get() - remainder) > EPSILON {
```

**Edge cases NOT tested**:
- `step = 1e-11` (smaller than epsilon) — fails incorrectly
- Subnormal numbers near `f64::MIN_POSITIVE`
- Floating-point accumulation over large ranges

**Why this is fragile**:
- IEEE 754 has well-known precision issues
- Epsilon should be **relative** to magnitude, not global constant

**Recommendation**: Use relative epsilon:
```rust
let epsilon = step.get() * 1e-10;
```

**OR** use `approx` crate for proper float comparisons.

---

#### E-03 — No Inheritance Depth Limit (MEDIUM)

**Severity**: 🟡 MEDIUM — Stack overflow risk
**Location**: `extender.rs`, `resolver.rs`

**Attack scenario**:
```yaml
# schema_1.yaml
name: "s1"
extends: "s2"

# ... (repeat for 10,000 levels)
```

**What happens**:
- `CycleChecker` prevents **cycles** via visited set ✅
- BUT: No depth limit means deep inheritance → stack overflow ❌
- `Resolver::merge_properties` walks parent chains recursively

**Recommendation**: Add max depth limit (e.g., 100 levels) with explicit error:

```rust
const MAX_INHERITANCE_DEPTH: usize = 100;

if depth > MAX_INHERITANCE_DEPTH {
    return Err(SchemaError::InheritanceDepthExceeded(depth));
}
```

---

#### E-04 — RawOptions Deserialization Is Order-Dependent (HIGH)

**Severity**: 🔴 HIGH — Silent mismatches
**Location**: `raw.rs:448-458`

```rust
#[serde(untagged)]
pub enum RawOptions {
    List(Vec<Box<str>>),     // Tried first
    Map(BTreeMap<...>),      // Tried second
    Rich(Vec<RawOptionEntry>), // Tried third
}
```

**Problem**: Serde's `untagged` matching is fragile:

```json
// Intended as Rich, but might deserialize as List
[{"value": "a"}]  // Missing "label" — could match List if serde fails gracefully
```

**Best practice**: Use **tagged unions** for unambiguous deserialization:

```json
{
  "type": "list",
  "values": ["a", "b"]
}
```

**Alternative**: Keep untagged but add validation tests for all 3 modes.

---

#### E-05 — Regex Cache Has Unbounded Growth (MEDIUM)

**Severity**: 🟡 MEDIUM — Memory leak in long-running process
**Location**: `property_spec.rs:1189-1224`

```rust
static REGEX_CACHE: OnceLock<RegexCacheLock> = OnceLock::new();
type RegexCache = HashMap<Box<str>, Arc<regex::Regex>>;
```

**Problem**:
- If users define 10,000 unique regex patterns, all stay in memory FOREVER
- No LRU eviction
- No size limit

**Best practice**: Use `moka` (already in dependencies) with max capacity:

```rust
use moka::sync::Cache;

static REGEX_CACHE: LazyLock<Cache<Box<str>, Arc<Regex>>> =
    LazyLock::new(|| Cache::builder().max_capacity(1000).build());
```

---

### Section 4: Property Naming Constraint Mismatch

#### PN-01 — JSON Schema vs Rust Pattern Divergence (CRITICAL)

**Severity**: 🔴 CRITICAL — User confusion
**User correction**: Property names should support uppercase, lowercase, digits, underscores AND hyphens, can start with letter OR underscore

**Current state**:

**JSON Schema** (`property-bank.schema.json:18`):
```json
"pattern": "^[a-z][a-z0-9_]*$"
```

**Rust code** (`property.rs:634`, uses `patterns::ALPHANUMERIC_NAME_LOWER`):
```rust
r"^[a-z0-9_-]+$"
```

**Issues identified**:
1. JSON Schema is **more restrictive** than Rust (no hyphens, no leading underscore/digit)
2. JSON Schema forces lowercase only
3. User says uppercase should be allowed
4. User says leading underscore should be allowed

**Correct pattern** (based on user requirements):
```regex
^[A-Za-z_][A-Za-z0-9_-]*$
```

**Breakdown**:
- Start: `[A-Za-z_]` — letter (upper/lower) or underscore
- Continue: `[A-Za-z0-9_-]*` — alphanumeric, underscore, OR hyphen

**Recommendation**: Update BOTH schemas and Rust code:

```json
{
  "pattern": "^[A-Za-z_][A-Za-z0-9_-]*$",
  "description": "Property names: start with letter or underscore, continue with alphanumeric, underscores, or hyphens"
}
```

```rust
// patterns.rs
pub const PROPERTY_NAME_PATTERN: &str = r"^[A-Za-z_][A-Za-z0-9_-]*$";
```

**Schema name pattern** should remain lowercase-only (for consistency):
```regex
^[a-z][a-z0-9_]*$
```

---

### Section 5: Meta-Schema Improvements (Best Practices)

Based on research into JSON Schema, GraphQL, Apache Avro, and Protobuf:

#### MS-01 — Missing Schema Versioning (CRITICAL)

**Severity**: 🔴 CRITICAL — No evolution path
**Location**: Both schemas missing `$version` field

**Problem**: No way to evolve format without breaking old files.

**Industry standard**:
```json
{
  "properties": {
    "$version": {
      "type": "string",
      "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$",
      "default": "1.0.0"
    }
  }
}
```

**Migration handler**:
```rust
match raw_schema.version.as_ref() {
    "1.0.0" => parse_v1(raw_schema),
    "2.0.0" => parse_v2(raw_schema),
    unknown => Err(UnsupportedVersion(unknown)),
}
```

**Recommendation**: Add to BOTH schemas before v1.0 release.

---

#### MS-02 — No min/max Constraint Validation (HIGH)

**Severity**: 🟠 MEDIUM-HIGH — Runtime errors instead of schema errors
**Location**: `NumberProperty` in both schemas

**Problem**: JSON Schema can't express "min < max". Invalid schemas pass validation but fail at runtime.

**Current**:
```json
{
  "min": { "type": "number" },
  "max": { "type": "number" }
  // ❌ Nothing prevents min=100, max=10
}
```

**Best practice** (JSON Schema draft-07):
```json
{
  "allOf": [
    {
      "if": { "required": ["min", "max"] },
      "then": {
        "properties": {
          "max": {
            "exclusiveMinimum": { "$data": "1/min" }
          }
        }
      }
    }
  ]
}
```

**Alternative**: Custom validation in CI pipeline.

---

#### MS-03 — Missing Examples in Schema (MEDIUM)

**Severity**: 🟡 MEDIUM — Poor developer experience
**Location**: Both schemas

**Industry standard**:
```json
{
  "examples": [
    {
      "name": "project_note",
      "properties": {
        "status": {
          "$ref": "property_bank#/status",
          "required": true
        }
      }
    }
  ]
}
```

**Benefits**:
- Powers IDE autocomplete
- Generated docs show real usage
- Newcomers copy-paste working examples

---

#### MS-04 — No Deprecation Support (MEDIUM)

**Severity**: 🟡 MEDIUM — No migration path
**Location**: Both schemas

**Industry standard** (OpenAPI, GraphQL):
```json
{
  "properties": {
    "deprecated": { "type": "boolean", "default": false },
    "deprecation_reason": { "type": "string" }
  }
}
```

**Use case**:
```json
{
  "tags": {
    "$ref": "property_bank#/tags",
    "deprecated": true,
    "deprecation_reason": "Use 'categories' instead"
  }
}
```

---

#### MS-05 — Missing UI Metadata Fields (LOW)

**Severity**: 🟢 LOW — Nice-to-have
**Recommendation**: Add optional UI hint fields:

```json
{
  "ui_hint": {
    "type": "string",
    "enum": ["text", "textarea", "select", "date-picker", "file-picker"]
  },
  "placeholder": { "type": "string" },
  "help_text": { "type": "string" }
}
```

**Rationale**: Obsidian, Logseq support UI hints. Future-proofs for editor integrations.

---

#### MS-06 — No Property Default Values (LOW)

**Severity**: 🟢 LOW — Convenience feature

```json
{
  "properties": {
    "default": {
      "oneOf": [
        { "type": "string" },
        { "type": "number" },
        { "type": "boolean" }
      ]
    }
  }
}
```

**Use case**:
```json
{
  "status": {
    "type": "string",
    "options": ["todo", "done"],
    "default": "todo"
  }
}
```

---

### Section 6: Idiomatic Rust Violations

#### IR-01 — Box<str> for Static Error Messages (LOW)

**Severity**: 🟢 LOW — Unnecessary allocations
**Location**: `error.rs` — multiple variants

**Example**:
```rust
ValidationFailed(String),  // Often constructed with static strings
```

**Problem**:
```rust
SchemaError::ValidationFailed("min cannot be greater than max".into())
// ^ Allocates heap memory for compile-time constant
```

**Recommendation**: Use `Cow<'static, str>`:

```rust
ValidationFailed(Cow<'static, str>),

// Usage:
SchemaError::ValidationFailed("static message".into())  // No alloc
SchemaError::ValidationFailed(format!("dynamic {}", x).into())  // Allocates
```

---

#### IR-02 — Missing #[must_use] on Builder Methods (LOW)

**Severity**: 🟢 LOW — API misuse prevention
**Location**: `property.rs:781-863` (PropertyBuilder)

**Problem**:
```rust
builder.required(true);  // Forgot to chain - value lost!
builder.build()
```

**Recommendation**:
```rust
#[must_use = "builder methods have no effect unless chained"]
pub fn array(mut self, array: bool) -> Self { ... }
```

---

### Section 7: Comparison to Industry Standards

| Feature        | Lithos | JSON Schema | GraphQL | Avro | Protobuf |
|----------------|--------|-------------|---------|------|----------|
| Versioning     | ❌     | ✅          | ✅      | ✅   | ✅       |
| Inheritance    | ✅     | ⚠️ (allOf)  | ✅      | ❌   | ❌       |
| Deprecation    | ❌     | ✅          | ✅      | ✅   | ✅       |
| Defaults       | ❌     | ✅          | ✅      | ✅   | ✅       |
| Examples       | ❌     | ✅          | ✅      | ❌   | ❌       |
| UI Hints       | ❌     | ⚠️ (ext)    | ✅      | ❌   | ❌       |

**Best practices you're missing**:
1. Schema versioning (critical for evolution)
2. Deprecation workflow
3. Default values
4. Inline examples

---

### Section 8: Prioritized Action Plan

#### Phase 1: CRITICAL (Must-Fix Before v1.0)

**Property Naming**:
1. ✅ **PN-01**: Fix property name pattern mismatch
   - Update JSON schemas to `^[A-Za-z_][A-Za-z0-9_-]*$`
   - Update Rust code to match
   - Add test cases for uppercase, leading underscore, hyphens

**Meta-Schema**:
2. ✅ **MS-01**: Add `$version` field to both schemas
3. ✅ **MS-03**: Add examples to schemas
4. ✅ **MS-02**: Add min/max validation (or document limitation)

**Performance**:
5. ✅ **P-01**: Implement `Rc<Property>` sharing in inheritance

**Edge Cases**:
6. ✅ **E-04**: Fix RawOptions deserialization fragility

---

#### Phase 2: HIGH PRIORITY (Performance & Robustness)

**Performance**:
7. ✅ **P-02**: Optimize name index allocations
8. ✅ **P-03**: Document PropertyBank allocation tradeoff

**Edge Cases**:
9. ✅ **E-01**: Add PropertyBank concurrency tests
10. ✅ **E-03**: Add inheritance depth limit
11. ✅ **E-05**: Replace regex cache with Moka LRU

**Meta-Schema**:
12. ✅ **MS-04**: Add deprecation support

---

#### Phase 3: MEDIUM (Code Quality)

**Superfluous Code**:
13. ✅ **S-01**: Remove PropertyRef wrapper
14. ✅ **S-02**: Split SchemaError into focused types
15. ✅ **S-03**: Move PropertyBank.all() to #[cfg(test)]

**Edge Cases**:
16. ✅ **E-02**: Fix NumberSpec epsilon comparison

**Meta-Schema**:
17. ✅ **MS-05**: Add UI hint fields
18. ✅ **MS-06**: Add default value support

---

#### Phase 4: LOW (Polish)

**Idiomatic Rust**:
19. ✅ **IR-01**: Use Cow<'static, str> for errors
20. ✅ **IR-02**: Add #[must_use] to builder methods

---

### Section 9: Implementation Checklist

Each item requires:
- [ ] Code changes
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] `mise run verify` passes
- [ ] ADR created (if architectural)

**Total issues identified**: 21
**Critical**: 5
**High**: 6
**Medium**: 7
**Low**: 3

---

### Section 10: What You're Doing Right ✅

1. ✅ **Port-based CQRS** — Clean separation of Command/Query
2. ✅ **Type-driven design** — Validated newtypes (SchemaName, PropertyName)
3. ✅ **Pipeline architecture** — Clear Ingestor → Dereferencer → Extender → Resolver
4. ✅ **Domain isolation** — Context boundaries respected
5. ✅ **Zero-copy storage** — rkyv for efficient serialization
6. ✅ **Test coverage** — Comprehensive unit + property-based tests
7. ✅ **Documentation** — Excellent rustdoc with examples
8. ✅ **$ref mechanism** — Industry-standard JSON Pointer syntax
9. ✅ **Inheritance model** — extends + excludes for composition

---

### Section 11: Research References

**JSON Schema Best Practices**:
- https://json-schema.org/understanding-json-schema/
- https://www.schemastore.org/

**Schema Evolution**:
- [Schema Evolution in Avro, Protocol Buffers and Thrift](https://martin.kleppmann.com/2012/12/05/schema-evolution-in-avro-protocol-buffers-thrift.html)

**Rust Performance**:
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

**Zero-Copy Patterns**:
- rkyv documentation on format stability
- redb best practices (already followed)

---

**END OF PART 10**

---

## Part 11: Pattern Architecture & User-Facing Format System

**Date**: 2026-02-26
**Reviewer**: BMad Master Agent (Claude)
**Scope**: Pattern localization + user-facing format abstraction
**Status**: Critical design issues identified

---

### Executive Summary

Two fundamental architectural issues discovered with the pattern system:

1. **Pattern localization is broken**: Patterns in `patterns.rs` are NOT shared across contexts. Each pattern is used by exactly one module, violating encapsulation and creating false "shared infrastructure" that's actually domain-specific.

2. **No user-facing format system**: Users must write raw regex patterns instead of using named formats like "email" or "url". This is a **major UX problem** that makes schemas harder to write and maintain.

---

### Issue 1: Pattern Localization (CRITICAL)

**Severity**: 🔴 CRITICAL — Broken encapsulation
**Location**: `lithos-core/src/patterns.rs`

#### Current State (WRONG)

```rust
// patterns.rs — Falsely labeled as "shared"
pub const ALPHANUMERIC_NAME: &str = "^[a-zA-Z0-9_-]+$";        // UNUSED
pub const ALPHANUMERIC_NAME_LOWER: &str = "^[a-z0-9_-]+$";    // schema ONLY
pub const PROPERTY_NAME: &str = "^[A-Za-z_][A-Za-z0-9_-]*$";  // schema ONLY
pub const IDENTIFIER_NAME: &str = "^[a-zA-Z_][a-zA-Z0-9_]*$"; // template ONLY

// User-facing patterns (EMAIL, URL, PHONE_US, etc.) — NEVER USED!
pub const EMAIL: &str = r"^[^@]+@[^@]+\.[^@]+$";
pub const URL: &str = r"^https?://[^\s/$.?#].[^\s]*$";
pub const PHONE_US: &str = r"^\+?1?[-.\s]?\(?...";
pub const SLUG: &str = "^[a-z0-9]+(-[a-z0-9]+)*$";
pub const UUID_V4: &str = "^[0-9a-f]{8}-...";
pub const WIKILINK: &str = r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$";
pub const ZIP_CODE: &str = r"^\d{5}(-\d{4})?$";
```

#### Evidence of Non-Sharing

- `ALPHANUMERIC_NAME_LOWER` used ONLY in `schema/aggregate.rs` (SchemaName validation)
- `PROPERTY_NAME` used ONLY in `schema/property.rs` (PropertyName validation)
- `IDENTIFIER_NAME` used ONLY in `template/` module (variable names)
- `ALPHANUMERIC_NAME` is **completely unused** (dead code)

#### The Real Problem

**User-facing patterns** (EMAIL, URL, PHONE_US, SLUG, UUID_V4, WIKILINK, ZIP_CODE) exist in `patterns.rs` but are **NEVER REFERENCED** anywhere in the codebase. They were intended for users to reference by name, but there's **no mechanism to use them**.

---

### Issue 2: No User-Facing Format System (CRITICAL)

**Severity**: 🔴 CRITICAL — Poor UX
**Location**: `schema/property_spec.rs` — StringSpec has no format field

#### Current User Experience (BAD)

```json
{
  "email": {
    "type": "string",
    "pattern": "^[^@]+@[^@]+\\.[^@]+$"  // 🔴 User must write regex manually
  },
  "website": {
    "type": "string",
    "pattern": "^https?://[^\\s/$.?#].[^\\s]*$"  // 🔴 Copy-paste, error-prone
  }
}
```

#### Industry Standard (JSON Schema)

```json
{
  "email": {
    "type": "string",
    "format": "email"  // ✅ Built-in named format
  },
  "website": {
    "type": "string",
    "format": "uri"    // ✅ Named format
  }
}
```

#### Why This Matters

1. **Usability**: Writing regex is error-prone (escaping, anchors, etc.)
2. **Consistency**: Every user implements "email" differently
3. **Maintenance**: Changing regex requires vault-wide search-replace
4. **Industry standard**: JSON Schema, OpenAPI, GraphQL all support named formats

---

### Proposed Solution

#### Part A: Localize Domain-Specific Patterns

**Move patterns into their modules**:

```rust
// schema/aggregate.rs
const SCHEMA_NAME_PATTERN: &str = "^[a-z0-9_-]+$";

// schema/property.rs
const PROPERTY_NAME_PATTERN: &str = "^[A-Za-z_][A-Za-z0-9_-]*$";

// template/mod.rs
const IDENTIFIER_PATTERN: &str = "^[a-zA-Z_][a-zA-Z0-9_]*$";
```

**Remove** `patterns.rs` entirely (or make it module-private if template uses it internally).

---

#### Part B: Create User-Facing Format System

**1. Define StringFormat enum**:

```rust
// schema/formats.rs (NEW FILE)

/// Built-in string validation formats that users can reference by name.
///
/// These provide common validation patterns without requiring users to write regex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum StringFormat {
    /// Email address validation (RFC 5322 simplified)
    Email,
    /// HTTP/HTTPS URL validation
    Url,
    /// US phone number (various formats)
    PhoneUs,
    /// Kebab-case slug (lowercase, hyphens)
    Slug,
    /// UUID version 4
    UuidV4,
    /// Obsidian-style wikilink [[page|alias]]
    WikiLink,
    /// US ZIP code (5 or 9 digits)
    ZipCode,
}

impl StringFormat {
    /// Get the regex pattern for this format.
    pub const fn pattern(self) -> &'static str {
        match self {
            Self::Email => r"^[^@]+@[^@]+\.[^@]+$",
            Self::Url => r"^https?://[^\s/$.?#].[^\s]*$",
            Self::PhoneUs => r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$",
            Self::Slug => "^[a-z0-9]+(-[a-z0-9]+)*$",
            Self::UuidV4 => "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
            Self::WikiLink => r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$",
            Self::ZipCode => r"^\d{5}(-\d{4})?$",
        }
    }
}
```

**2. Update StringSpec**:

```rust
// property_spec.rs

pub struct StringSpec {
    format: Option<StringFormat>,      // NEW: Named format
    pattern: Option<Box<str>>,         // Custom regex (mutually exclusive with format)
    options: Option<Vec<OptionEntry>>,
}

impl StringSpec {
    pub fn try_new(
        format: Option<StringFormat>,
        pattern: Option<Box<str>>,
        options: Option<Vec<OptionEntry>>,
    ) -> Result<Self, SchemaError> {
        // Validate: format and pattern are mutually exclusive
        if format.is_some() && pattern.is_some() {
            return Err(SchemaError::ValidationFailed(
                "Cannot specify both 'format' and 'pattern'".into()
            ));
        }

        // Validate custom pattern if provided
        if let Some(ref p) = pattern {
            get_cached_regex(p)?;
        }

        Ok(Self { format, pattern, options })
    }

    fn validate_pattern(&self, value: &str) -> Result<(), SchemaError> {
        // Use format's pattern if specified
        let pattern_str = if let Some(fmt) = self.format {
            fmt.pattern()
        } else if let Some(ref p) = self.pattern {
            p.as_ref()
        } else {
            return Ok(()); // No pattern validation
        };

        let re = get_cached_regex(pattern_str)?;
        if !re.is_match(value) {
            return Err(SchemaError::ValidationFailed(
                format!("Value does not match expected format")
            ));
        }
        Ok(())
    }
}
```

**3. Update RawStringSpec**:

```rust
// raw.rs

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RawStringSpec {
    pub format: Option<StringFormat>,  // NEW
    pub pattern: Option<Box<str>>,
    pub options: Option<RawOptions>,
}
```

**4. Update JSON Schemas**:

```json
{
  "StringProperty": {
    "properties": {
      "type": { "const": "string" },
      "format": {
        "type": "string",
        "enum": ["email", "url", "phone_us", "slug", "uuid_v4", "wikilink", "zip_code"],
        "description": "Built-in validation format. Mutually exclusive with 'pattern'."
      },
      "pattern": {
        "type": "string",
        "format": "regex",
        "description": "Custom regex pattern. Mutually exclusive with 'format'."
      },
      "options": { ... }
    },
    "oneOf": [
      { "required": ["type", "format"] },
      { "required": ["type", "pattern"] },
      { "required": ["type", "options"] },
      { "required": ["type"] }
    ]
  }
}
```

---

### Benefits of This Design

#### Pattern Localization
✅ Each module owns its validation rules
✅ No false "shared" infrastructure
✅ Clear encapsulation boundaries
✅ Easier to refactor individual contexts

#### User-Facing Formats
✅ **Ease of use**: `"format": "email"` vs hand-writing regex
✅ **Consistency**: Everyone uses the same email validation
✅ **Maintainability**: Fix bugs in one place
✅ **Discoverability**: IDE autocomplete shows available formats
✅ **Industry alignment**: Matches JSON Schema, OpenAPI standards
✅ **Extensibility**: Add new formats without breaking existing schemas

---

### Implementation Plan

#### Step 1: Pattern Localization (BREAKING CHANGE)

1. Move `ALPHANUMERIC_NAME_LOWER` → `schema/aggregate.rs` as `SCHEMA_NAME_PATTERN`
2. Rename `PROPERTY_NAME` → `PROPERTY_NAME_PATTERN` and keep in `schema/property.rs`
3. Move `IDENTIFIER_NAME` → `template/mod.rs` as `IDENTIFIER_PATTERN`
4. Remove `ALPHANUMERIC_NAME` (dead code)
5. Delete `patterns.rs` (or keep as `pub(crate)` if template needs internal sharing)

#### Step 2: User-Facing Format System (FEATURE)

1. Create `schema/formats.rs` with `StringFormat` enum
2. Update `StringSpec` to add `format: Option<StringFormat>` field
3. Update `RawStringSpec` to deserialize format field
4. Add validation: format XOR pattern (mutually exclusive)
5. Update `validate_pattern` to check format or pattern
6. Update both JSON schemas to include format field with enum values
7. Add tests for all 7 built-in formats
8. Update documentation with format examples

#### Step 3: Migration Guide

**For users upgrading**:
```json
// OLD (still works)
{
  "email": { "type": "string", "pattern": "^[^@]+@..." }
}

// NEW (preferred)
{
  "email": { "type": "string", "format": "email" }
}
```

**Breaking change**: Rust code using `patterns::PROPERTY_NAME` must update imports.

---

### Acceptance Criteria

- [ ] All patterns localized to their modules
- [ ] `patterns.rs` removed (or made module-private)
- [ ] `StringFormat` enum with 7 formats implemented
- [ ] `format` field added to StringSpec + RawStringSpec
- [ ] JSON schemas updated with format enum
- [ ] Tests for all 7 formats (valid + invalid cases)
- [ ] Documentation updated with format examples
- [ ] Migration guide written
- [ ] `mise run verify` passes

---

**END OF PART 11**
