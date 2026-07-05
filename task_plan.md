# Rkyv redb Codec Refactor Plan

## Goal

Replace ad-hoc rkyv/redb persistence glue with a DB-owned codec and table-wrapper design that:

- Removes per-domain `redb::Value` / `redb::Key` implementations where possible.
- Phases out `ArchivedEntity` from `crates/db/src/codec.rs`.
- Preserves strict rkyv validation and alignment safety.
- Replaces generic `DbError::Serialization(String)` / `DbError::Deserialization(String)` usage with typed codec errors.
- Keeps repository adapters simple at call sites.

## Current State

- `ArchivedEntity` is a blanket trait in `crates/db/src/codec.rs`.
- It provides three capabilities: `to_bytes`, `from_bytes`, and `with_archived`.
- It returns `DbError` directly and erases rkyv source errors into strings.
- GitNexus impact for the `ArchivedEntity` trait reports LOW graph impact, but text usage is broad because blanket trait methods are called via method syntax.
- Existing DB table wrappers live in `crates/db/src/table.rs` and expose `.definition()` around redb `TableDefinition` / `MultimapTableDefinition`.

## Target Components

This section is the full designed component inventory. Do not treat any item as optional unless explicitly marked optional.

## Concrete API Sketch

This is the concrete design we agreed on earlier. Keep it in the plan because it is the easiest implementation reference.

### Public Types

```rust
pub struct RkyvValue<T>(PhantomData<T>);

pub struct RkyvKey<T>(PhantomData<T>);

pub struct RkyvBytes<'a, T> {
    bytes: Cow<'a, [u8]>,
    _ty: PhantomData<T>,
}
```

Raw usage without `RkyvTable` wrappers:

```rust
pub(crate) const NOTES: TableDefinition<RkyvKey<NoteId>, RkyvValue<Note>> =
    TableDefinition::new("notes");
```

Important correction for multimaps: redb multimap values must implement `Key`, so the generic rkyv multimap form should use `RkyvKey<V>`, not `RkyvValue<V>`.

```rust
pub(crate) const NOTE_TAGS: MultimapTableDefinition<
    RkyvKey<TagName>,
    RkyvKey<NoteId>,
> = MultimapTableDefinition::new("note_tags");
```

Preferred usage with table wrappers:

```rust
pub(crate) const NOTES: RkyvTable<NoteId, Note> =
    RkyvTable::new("notes");

pub(crate) const NOTE_TAGS: RkyvMultimap<TagName, NoteId> =
    RkyvMultimap::new("note_tags");
```

### `RkyvBytes`

`RkyvBytes` is the shared carrier for borrowed redb reads and owned insert values.

```rust
impl<'a, T> RkyvBytes<'a, T> {
    pub const fn borrowed(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            _ty: PhantomData,
        }
    }

    pub fn owned(bytes: Vec<u8>) -> RkyvBytes<'static, T> {
        RkyvBytes {
            bytes: Cow::Owned(bytes),
            _ty: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub fn decode(&self) -> Result<T, CodecError>
    where
        T: RkyvDecode,
    {
        decode_rkyv::<T>(self.as_bytes())
    }

    pub fn with_archived<R, F>(&self, f: F) -> Result<R, CodecError>
    where
        T: RkyvDecode,
        F: FnOnce(&T::Archived) -> R,
    {
        with_archived_rkyv::<T, R, F>(self.as_bytes(), f)
    }
}

impl<T> RkyvBytes<'static, T> {
    pub fn encode(value: &T) -> Result<Self, CodecError>
    where
        T: RkyvEncode,
    {
        encode_rkyv(value)
    }
}
```

Note: earlier sketches had `encode_rkyv(value).map(Self::owned)`. The cleaner final shape is for `encode_rkyv` to return `RkyvBytes<'static, T>` directly. If implementation pressure says returning `Vec<u8>` is simpler, keep `RkyvBytes::encode` as the stable public interface and hide the choice internally.

### Codec Bounds

Hide rkyv's long bounds behind local traits.

```rust
pub trait RkyvEncode:
    Archive
    + for<'a> Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'a>,
            rkyv::rancor::Error,
        >,
    >
{
}

impl<T> RkyvEncode for T where
    T: Archive
        + for<'a> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >
{
}

pub trait RkyvDecode: Archive {}

impl<T> RkyvDecode for T
where
    T: Archive,
    T::Archived: rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>,
        > + Deserialize<
            T,
            rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
        >,
{
}
```

### `CodecError`

Codec failures get their own error type.

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("failed to serialize {type_name} with rkyv")]
    RkyvSerialize {
        type_name: &'static str,
        #[source]
        source: rkyv::rancor::Error,
    },

    #[error("failed to validate archived {type_name}")]
    RkyvAccess {
        type_name: &'static str,
        #[source]
        source: rkyv::rancor::Error,
    },

    #[error("failed to deserialize archived {type_name}")]
    RkyvDeserialize {
        type_name: &'static str,
        #[source]
        source: rkyv::rancor::Error,
    },
}
```

Optional helper:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecErrorKind {
    Encode,
    Access,
    Decode,
}

impl CodecError {
    pub const fn kind(&self) -> CodecErrorKind {
        match self {
            Self::RkyvSerialize { .. } => CodecErrorKind::Encode,
            Self::RkyvAccess { .. } => CodecErrorKind::Access,
            Self::RkyvDeserialize { .. } => CodecErrorKind::Decode,
        }
    }
}
```

### `DbError` and `DbErrorKind`

Update `DbErrorKind` so codec failures are classified as codec failures, not generic serialization/deserialization:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbErrorKind {
    Database,
    Storage,
    Transaction,
    Table,
    Commit,
    Codec,
}
```

Embed codec errors in `DbError`:

```rust
#[error(transparent)]
Codec(#[from] CodecError),
```

Keep existing `Serialization(String)` and `Deserialization(String)` only as deprecated migration variants until old call sites are gone. During migration, both should classify as `DbErrorKind::Codec`.

`DbError::is_transient()` should treat all codec errors as permanent/non-transient.

### `RkyvBytes<'a, T>`

Use this name, not `RkyvEntity`.

Reason: the value is typed bytes, not necessarily a validated entity until `decode()` or `with_archived()` succeeds. It also works for IDs, strings, view structs, events, and projection values.

```rust
pub struct RkyvBytes<'a, T> {
    bytes: Cow<'a, [u8]>,
    _ty: PhantomData<T>,
}
```

Responsibilities:

- Borrow redb bytes on reads.
- Own encoded bytes on writes.
- Provide `as_bytes()` for redb insertion.
- Provide `decode()` and `with_archived()` for typed access.

Required API:

```rust
impl<'a, T> RkyvBytes<'a, T> {
    pub const fn borrowed(bytes: &'a [u8]) -> Self;
    pub fn owned(bytes: Vec<u8>) -> RkyvBytes<'static, T>;
    pub fn as_bytes(&self) -> &[u8];
    pub fn decode(&self) -> Result<T, CodecError>
    where
        T: RkyvDecode;
    pub fn with_archived<R, F>(&self, f: F) -> Result<R, CodecError>
    where
        T: RkyvDecode,
        F: FnOnce(&T::Archived) -> R;
}

impl<T> RkyvBytes<'static, T> {
    pub fn encode(value: &T) -> Result<Self, CodecError>
    where
        T: RkyvEncode;
}
```

Use `Cow<'a, [u8]>` so the same type can represent borrowed redb read bytes and owned encoded insert bytes. This is intentional; do not split into `RkyvRef`/`RkyvOwned` unless `Cow` proves confusing in implementation.

### Codec Bounds

Hide rkyv's long bounds behind local traits:

- `RkyvEncode`
- `RkyvDecode`

These replace the blanket `ArchivedEntity` trait as the bound surface.

Required behavior:

- `RkyvEncode` covers `Archive + Serialize<HighSerializer<AlignedVec, ArenaHandle<'_>, rancor::Error>>`.
- `RkyvDecode` covers `Archive` plus `Archived: Portable + CheckBytes<HighValidator<'_, rancor::Error>> + Deserialize<T, HighDeserializer<rancor::Error>>`.
- Bounds live inside `codec.rs` so repository adapters do not repeat long rkyv signatures.

### Codec Functions

Free functions back the `RkyvBytes` methods and provide migration-call-site escape hatches:

```rust
pub fn encode_rkyv<T>(value: &T) -> Result<RkyvBytes<'static, T>, CodecError>
where
    T: RkyvEncode;

pub fn decode_rkyv<T>(bytes: &[u8]) -> Result<T, CodecError>
where
    T: RkyvDecode;

pub fn with_archived_rkyv<T, R, F>(bytes: &[u8], f: F) -> Result<R, CodecError>
where
    T: RkyvDecode,
    F: FnOnce(&T::Archived) -> R;
```

Alignment behavior must preserve the current `ArchivedEntity` implementation:

- If input bytes are already 16-byte aligned, call `rkyv::access` directly.
- Otherwise copy into `AlignedVec<16>` before `rkyv::access`.
- Never use `access_unchecked`.

### `RkyvValue<T>` and `RkyvKey<T>`

Zero-sized redb adapter types:

```rust
pub struct RkyvValue<T>(PhantomData<T>);
pub struct RkyvKey<T>(PhantomData<T>);
```

Responsibilities:

- Implement `redb::Value` generically for rkyv-backed values.
- Implement `redb::Key` generically for rkyv-backed keys.
- Use `redb::TypeName::new(std::any::type_name::<Self>())` for distinct per-wrapper/per-type names.
- Implement `Debug` for `RkyvValue<T>`, `RkyvKey<T>`, and `RkyvBytes<'a, T>` because `redb::Value` requires the value type and `SelfType<'a>` to implement `Debug`.
- Avoid adding unnecessary `T: Debug` bounds. If `derive(Debug)` adds unwanted bounds, write manual `Debug` impls that print the wrapper type and byte length only.

`RkyvValue<T>: redb::Value`:

- `SelfType<'a> = RkyvBytes<'a, T>`.
- `AsBytes<'a> = &'a [u8]`.
- `fixed_width() = None`.
- `from_bytes(data) = RkyvBytes::borrowed(data)`.
- `as_bytes(value) = value.as_bytes()`.

`RkyvKey<T>: redb::Value` uses the same value behavior.

`RkyvKey<T>: redb::Key`:

- Initial implementation uses decode-and-compare.
- Bound should require `T: RkyvDecode + Ord + 'static`.
- Invalid key bytes should `expect("invalid rkyv key bytes")`; redb compare cannot return `Result`.
- Include a `ponytail:` comment naming the performance ceiling and upgrade path.

Do not add ordered-byte key specialization in the first pass. Add only if profiling shows decode-and-compare is hot.

### Existing Optimized Key Types

Do not assume `RkyvKey<T>` should replace every existing redb key implementation.

Existing optimized/stable key integrations include:

- `impl_redb_uuid!` / `UuidV7DbType` for UUID-backed IDs.
- `DbPathKey` for validated path keys.
- `EventId` for monotonic event-log keys.

These keys have direct sortable byte encodings and should generally stay as keys. The generic `RkyvKey<T>` is for key types that do not already have a better ordered encoding.

This means a migration may use `RkyvValue<V>` without using `RkyvKey<K>`.

### Optional `RkyvTable` and `RkyvMultimap`

Consider adding table-definition wrappers that hide `RkyvKey<T>` / `RkyvValue<T>` in table declarations:

```rust
pub struct RkyvTable<K, V> {
    definition: TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>,
}

pub struct RkyvMultimap<K, V> {
    definition: MultimapTableDefinition<'static, RkyvKey<K>, RkyvKey<V>>,
}
```

This follows the existing `Table`, `UuidTable`, and `UuidMultimap` wrapper pattern in `crates/db/src/table.rs`.

## RkyvTable / RkyvMultimap Decision

### Option A: Use only `RkyvKey<T>` / `RkyvValue<T>` in table constants

Example:

```rust
const SCHEMAS: Table<RkyvKey<SchemaId>, RkyvValue<Schema>> =
    Table::new("schemas");
```

Pros:

- Fewer wrapper types.
- Explicit redb adapter roles in type signatures.
- Minimal new abstraction.

Cons:

- Every table constant repeats `RkyvKey<...>` and `RkyvValue<...>`.
- Multimap values must decide between `RkyvKey<V>` and `RkyvValue<V>` because redb multimap values require `Key`.

### Option B: Add `RkyvTable<K, V>` and `RkyvMultimap<K, V>` definition wrappers

Example:

```rust
const SCHEMAS: RkyvTable<SchemaId, Schema> = RkyvTable::new("schemas");
const TAGS: RkyvMultimap<TagName, NoteId> = RkyvMultimap::new("tags");
```

Pros:

- Hides redb adapter noise from table definitions.
- Makes rkyv-backed tables obvious at the table-wrapper seam.
- Matches existing DB context language: "Table Wrapper".
- Avoids writing `RkyvKey<T>` / `RkyvValue<T>` on every table/multimap constant.

Cons:

- Adds two wrapper types.
- Does not by itself remove the need for `RkyvKey<T>` / `RkyvValue<T>` internally; it only hides them behind `.definition()`.
- Call sites still need `RkyvBytes` until typed open/read/write helpers exist.

### Recommendation

Add `RkyvTable<K, V>` and `RkyvMultimap<K, V>`.

This is not speculative abstraction because the codebase already uses table-definition wrappers as the seam. The new wrappers reduce repeated adapter types in table constants and create a clear migration target.

Keep them minimal at first: only `new()` and `definition()`. Do not add read/write helper methods until one migrated table proves the call-site pattern.

`RkyvTable<K, V>` should wrap:

```rust
TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>
```

`RkyvMultimap<K, V>` should wrap:

```rust
MultimapTableDefinition<'static, RkyvKey<K>, RkyvKey<V>>
```

Reason: redb multimap values must implement `Key`, not just `Value`.

Question to verify during implementation: whether every multimap value type has meaningful `Ord`. If not, some multimaps should stay on explicit table definitions or use domain-specific key encodings.

### Specialized Rkyv Value Table Wrappers

Because many existing tables use optimized UUID/path/event keys, consider value-only rkyv wrappers before forcing `RkyvKey<K>` everywhere.

Possible wrappers:

```rust
pub struct RkyvValueTable<K, V> {
    definition: TableDefinition<'static, K, RkyvValue<V>>,
}
```

Specialized aliases/wrappers may be clearer if usage is common:

```rust
pub struct UuidRkyvTable<K, V> {
    definition: TableDefinition<'static, K, RkyvValue<V>>,
}

pub struct PathRkyvTable<V> {
    definition: TableDefinition<'static, DbPathKey, RkyvValue<V>>,
}
```

Do not add all of these upfront. Pick the first vertical migration and add only the wrapper that removes real repetition.

Decision rule:

- Use existing optimized key wrappers when key encoding already exists (`UuidTable<K, RkyvValue<V>>`, `PathTable<RkyvValue<V>>`, etc.).
- Use `RkyvTable<K, V>` only when both key and value should be rkyv-backed generically.
- Use `RkyvMultimap<K, V>` only when both key and multimap value can tolerate decode-and-compare ordering.

### `ArchivedEntity` Compatibility Shim

Do not delete `ArchivedEntity` first.

Short-term plan:

- Mark `ArchivedEntity` deprecated.
- Re-implement its methods through `RkyvBytes` / codec functions.
- Preserve return type `DbError` by using `?` through `DbError::Codec`.
- Keep `to_bytes()` returning `AlignedVec` only if old call sites require it; otherwise use the closest compatible representation and migrate call sites quickly.

Purpose: avoid a big-bang rewrite across vault/schema/indexer/bench code.

### Public Exports

Update `crates/db/src/lib.rs` exports.

Replace or augment:

```rust
pub use codec::ArchivedEntity;
```

with:

```rust
pub use codec::{
    CodecError, CodecErrorKind, RkyvBytes, RkyvDecode, RkyvEncode,
};
pub use table::{RkyvMultimap, RkyvTable};
```

Keep `ArchivedEntity` export during compatibility migration, but deprecate it.

### Visibility Policy

Use the smallest visibility that still supports real callers.

Visibility ladder:

1. Private: default for helpers, alignment internals, type-name helpers, and free functions.
2. `pub(crate)`: shared only inside `traces_db`.
3. `pub`: only for types and methods external workspace crates must name.

Planned public API:

```rust
pub struct RkyvBytes<'a, T> { ... }
pub struct RkyvValue<T>(...);
pub struct RkyvKey<T>(...);
pub struct RkyvTable<K, V> { ... }
pub struct RkyvMultimap<K, V> { ... }

pub enum CodecError { ... }
pub enum CodecErrorKind { ... }
```

Public methods:

- `RkyvBytes::borrowed`
- `RkyvBytes::owned`
- `RkyvBytes::as_bytes`
- `RkyvBytes::encode`
- `RkyvBytes::decode`
- `RkyvBytes::with_archived`
- `RkyvTable::new`
- `RkyvTable::definition`
- `RkyvMultimap::new`
- `RkyvMultimap::definition`

Bounds traits decision:

- If public `RkyvBytes` methods need `RkyvEncode` / `RkyvDecode` in their bounds, those traits must be public or Rust will expose private bounds through public methods.
- Make `RkyvEncode` and `RkyvDecode` public but sealed if external crates need to call `RkyvBytes::encode`, `decode`, or `with_archived`.
- The blanket impls should still be controlled by `traces_db`; callers derive rkyv traits, they do not manually implement the codec traits.

Free function visibility:

- Keep `encode_rkyv`, `decode_rkyv`, and `with_archived_rkyv` private unless a real call site needs function-pointer style or method syntax is insufficient.
- Prefer the `RkyvBytes` methods as the public interface.

Reason: free functions duplicate the method interface and increase public surface. They are useful internally to keep `RkyvBytes` methods tiny and to support the deprecated `ArchivedEntity` shim, but they do not need to be public by default.

### Repository Adapter Call-Site Patterns

Before table migration:

```rust
let bytes = RkyvBytes::<Schema>::encode(schema)?;
table.insert(*schema.id(), bytes.as_bytes())?;

let schema = RkyvBytes::<Schema>::borrowed(guard.value()).decode()?;
```

After table migration to `RkyvTable`:

```rust
let bytes = RkyvBytes::<Schema>::encode(schema)?;
table.insert(RkyvBytes::<SchemaId>::encode(schema.id())?, bytes)?;

let schema = guard.value().decode()?;
```

Exact insert ergonomics must be verified against redb's `insert` bounds for `V::SelfType<'_>` and key borrowing behavior. Keep first vertical migration small to discover this safely.

If keeping an optimized key wrapper, the post-migration form is less disruptive:

```rust
let bytes = RkyvBytes::<Schema>::encode(schema)?;
table.insert(*schema.id(), bytes)?;

let schema = guard.value().decode()?;
```

This is likely the first migration shape for UUID-keyed tables.

### Event Store Bounds

Replace:

```rust
E: ArchivedEntity
```

with:

```rust
E: RkyvEncode + RkyvDecode
```

or method-local bounds if not every event-store method needs both encode and decode.

### Tests Required

Minimum tests for the first implementation slice:

- `CodecError::kind()` returns `Encode`, `Access`, and `Decode` correctly.
- `DbError::kind()` returns `DbErrorKind::Codec` for `DbError::Codec` and deprecated serialization/deserialization variants during migration.
- `RkyvBytes::encode(...).decode()` round-trips a test type.
- `RkyvBytes::with_archived()` reads a field without materializing the full type.
- Invalid bytes produce `CodecError::RkyvAccess`.
- Truncated bytes produce `CodecError::RkyvAccess` or `RkyvDeserialize`, whichever rkyv returns; test should match the actual boundary.
- `RkyvValue<T>` implements `redb::Value` and round-trips through a redb table.
- `RkyvKey<T>::compare` matches `T::cmp` for a test key.
- `RkyvTable<K, V>::definition()` has the expected `TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>` type.
- `RkyvMultimap<K, V>::definition()` has the expected `MultimapTableDefinition<'static, RkyvKey<K>, RkyvKey<V>>` type.
- `RkyvValue<T>`, `RkyvKey<T>`, and `RkyvBytes<'_, T>` satisfy redb's `Debug` requirements without requiring `T: Debug` unless unavoidable.
- A value-only migrated table using an existing optimized key (`UuidTable<K, RkyvValue<V>>` or equivalent) round-trips.

## Migration Phases

### Phase 1: Error Seam

Status: pending

- Add `CodecError`.
- Add `DbError::Codec`.
- Update `DbErrorKind` with `Codec`.
- Keep deprecated generic string variants until all call sites migrate.

### Phase 2: Codec Bytes

Status: pending

- Add `RkyvBytes<'a, T>`.
- Add `RkyvEncode` / `RkyvDecode`.
- Add `encode_rkyv`, `decode_rkyv`, and `with_archived_rkyv`.
- Preserve current alignment behavior from `ArchivedEntity::with_archived`.

### Phase 3: Compatibility Shim

Status: pending

- Keep `ArchivedEntity` temporarily.
- Re-implement it in terms of new codec functions.
- Mark deprecated.
- Avoid big-bang call-site churn.

### Phase 4: redb Adapters

Status: pending

- Add `RkyvValue<T>`.
- Add `RkyvKey<T>`.
- Implement `redb::Value` and `redb::Key`.
- Use decode-and-compare for keys initially, with a ponytail comment naming the performance ceiling.

### Phase 5: Table Wrappers

Status: pending

- Add `RkyvTable<K, V>`.
- Add `RkyvMultimap<K, V>`.
- Re-export them from `crates/db/src/lib.rs`.

### Phase 6: First Vertical Migration

Status: pending

- Pick one low-risk table.
- Migrate its table definition to `RkyvTable` or `RkyvMultimap`.
- Replace local `ArchivedEntity` usage with `RkyvBytes` only where needed.
- Add focused tests for insert/read/decode and key ordering.

### Phase 7: Broader Cleanup

Status: pending

- Migrate remaining storage adapters gradually.
- Remove per-domain redb trait impls replaced by generic adapters.
- Remove `ArchivedEntity` when unused.
- Remove generic string serialization/deserialization variants when unused.

## Errors Encountered

| Error | Attempt | Resolution |
|---|---|---|
| `rust-docs-mcp` could not analyze `redb 4.1.0` | Cached docs via crates.io/GitHub | Found MCP workspace-root limitation; used docs.rs for `TypeName` and cleaned duplicate cache |
| GitNexus showed no incoming refs for `ArchivedEntity` | `context`/`impact` on trait | Used text search because blanket trait methods are called via method syntax |

## Open Questions

- Which table should be the first vertical migration?
- Should `RkyvMultimap<K, V>` require `V: Ord` because redb multimap values are keys?
- Should `RkyvKey<T>::compare` panic on invalid bytes, or should hot key types use ordered byte encodings instead?
- Do we need `RkyvTable<K, V>` immediately, or should the first slice only add `RkyvValue<V>` behind existing optimized key wrappers?
- Should any public wrapper names be value-only (`RkyvValueTable`, `UuidRkyvTable`) rather than fully generic (`RkyvTable`) for the first migration?
