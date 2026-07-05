# Rkyv redb Codec Refactor Findings

## GitNexus Findings

- `ArchivedEntity` is defined in `crates/db/src/codec.rs`.
- GitNexus context found no direct incoming graph references to the trait.
- GitNexus impact reported `LOW`: 0 direct callers, 0 affected processes, 0 affected modules.
- Text search shows broad usage because blanket trait methods are called via method syntax (`to_bytes`, `from_bytes`, `with_archived`).

## Current `ArchivedEntity` Responsibilities

`ArchivedEntity` combines three separate concerns:

- Encode a value to rkyv bytes.
- Decode bytes to an owned value.
- Validate and borrow archived data for zero-copy reads.

It also directly couples domain types to DB storage behavior through a blanket trait.

## Existing Call-Site Patterns

Examples from storage adapters:

```rust
let bytes = schema.to_bytes()?;
table.insert(*schema.id(), bytes.as_slice())?;
```

```rust
let schema = Schema::from_bytes(guard.value())?;
```

```rust
String::with_archived(guard.value(), |archived| archived.as_str().to_owned())
```

Some indexer storage code bypasses `ArchivedEntity` and calls raw rkyv directly:

```rust
rkyv::to_bytes::<rkyv::rancor::Error>(record)
    .map_err(|e| DbError::Serialization(e.to_string()))?
```

## Existing DB Table Wrapper Pattern

`crates/db/src/table.rs` already uses wrapper types around redb definitions:

- `UuidTable<K, V>`
- `UuidMultimap<K, V>`
- `PathTable<V>`
- `PathUuidTable<V>`
- `UuidPathTable<K>`
- `Table<K, V>`
- `EventTable<V>`

These wrappers are const-constructible and expose `.definition()`.

This makes `RkyvTable<K, V>` and `RkyvMultimap<K, V>` a natural extension, not a new architectural pattern.

## `redb::TypeName`

Docs reviewed: <https://docs.rs/redb/latest/redb/struct.TypeName.html>

- `TypeName::new(name: &str) -> TypeName`.
- redb recommends prefixing names with the crate name to reduce collision risk.
- `std::any::type_name::<Self>()` is suitable for `RkyvValue<T>` and `RkyvKey<T>` because `Self` includes the wrapper and wrapped type.
- No custom `format!` is required for type names.

## `rust-docs-mcp` / redb Cache Finding

- `.rust-docs/cache/crates/redb/` was cleaned to contain only `4.1.0/`.
- The remaining MCP issue was not duplicate cache.
- `rust-docs-mcp` treats the normalized `redb` manifest as a workspace root because `[workspace].members` exists.
- Passing `members:["."]` still fails, likely due to a mixed-workspace-root handling limitation.

## Naming Decision

Use `RkyvBytes`, not `RkyvEntity`.

Reason:

- The stored value is typed bytes, not necessarily valid until checked.
- It works for IDs, strings, event payloads, view structs, and values.
- It avoids putting domain language into the DB crate.

## RkyvTable / RkyvMultimap Assessment

Adding `RkyvTable<K, V>` and `RkyvMultimap<K, V>` helps table definitions more than raw read/write call sites.

It avoids repeated definitions like:

```rust
Table<RkyvKey<SchemaId>, RkyvValue<Schema>>
```

in favor of:

```rust
RkyvTable<SchemaId, Schema>
```

This does not eliminate internal adapter types. `RkyvTable` still wraps a `TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>`.

The first implementation should be intentionally small: `new()` and `definition()` only.

## Complete Designed Components

The full design includes all of these components:

- `CodecError`: typed rkyv codec failures with `#[source] rkyv::rancor::Error`.
- `CodecErrorKind`: codec-local classification (`Encode`, `Access`, `Decode`).
- `DbError::Codec`: embeds `CodecError` in the DB error type.
- `DbErrorKind::Codec`: replaces generic serialization/deserialization classification.
- Deprecated `DbError::Serialization(String)` / `DbError::Deserialization(String)`: temporary migration only.
- `RkyvBytes<'a, T>`: typed borrowed-or-owned byte carrier using `Cow<'a, [u8]>`.
- `RkyvEncode`: local trait hiding rkyv high-serializer bounds.
- `RkyvDecode`: local trait hiding rkyv validation/deserialization bounds.
- `encode_rkyv`: encodes `T` into `RkyvBytes<'static, T>`.
- `decode_rkyv`: validates and decodes bytes into `T`.
- `with_archived_rkyv`: validates and borrows `T::Archived` for zero-copy reads.
- `RkyvValue<T>`: zero-sized redb value adapter.
- `RkyvKey<T>`: zero-sized redb key adapter.
- `RkyvTable<K, V>`: table-definition wrapper hiding `RkyvKey<K>` / `RkyvValue<V>`.
- `RkyvMultimap<K, V>`: multimap-definition wrapper hiding `RkyvKey<K>` / `RkyvKey<V>`.
- Deprecated `ArchivedEntity` shim: temporary compatibility layer over the new codec functions.

## Concrete API Preserved From Design Discussion

The implementation plan should preserve these concrete names and shapes:

```rust
pub struct RkyvValue<T>(PhantomData<T>);
pub struct RkyvKey<T>(PhantomData<T>);

pub struct RkyvBytes<'a, T> {
    bytes: Cow<'a, [u8]>,
    _ty: PhantomData<T>,
}
```

`RkyvBytes` methods:

- `borrowed(bytes: &'a [u8]) -> Self`
- `owned(bytes: Vec<u8>) -> RkyvBytes<'static, T>`
- `as_bytes(&self) -> &[u8]`
- `decode(&self) -> Result<T, CodecError>`
- `with_archived(&self, f) -> Result<R, CodecError>`
- `encode(value: &T) -> Result<RkyvBytes<'static, T>, CodecError>`

Bounds traits:

- `RkyvEncode`
- `RkyvDecode`

Error types:

- `CodecError`
- `CodecErrorKind`

Important correction: earlier notes showed `MultimapTableDefinition<RkyvKey<TagName>, RkyvValue<NoteId>>`. redb multimap values must be `Key`, so the correct generic rkyv multimap value adapter is `RkyvKey<NoteId>`.

## Component Relationships

`RkyvTable<K, V>` expands internally to:

```rust
TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>
```

`RkyvMultimap<K, V>` expands internally to:

```rust
MultimapTableDefinition<'static, RkyvKey<K>, RkyvKey<V>>
```

`RkyvKey<T>` and `RkyvValue<T>` both use:

```rust
type SelfType<'a> = RkyvBytes<'a, T>;
```

So when tables are fully migrated, redb read guards naturally expose typed `RkyvBytes<'_, T>` values.

## ArchivedEntity Replacement Detail

`ArchivedEntity` should not be replaced as a standalone broad refactor. It should become a compatibility shim first.

Old:

```rust
let bytes = value.to_bytes()?;
let value = T::from_bytes(bytes)?;
let field = T::with_archived(bytes, |archived| archived.field)?;
```

New transitional:

```rust
let bytes = RkyvBytes::<T>::encode(&value)?;
let value = RkyvBytes::<T>::borrowed(bytes.as_bytes()).decode()?;
let field = RkyvBytes::<T>::borrowed(bytes.as_bytes())
    .with_archived(|archived| archived.field)?;
```

New after table migration:

```rust
let value = guard.value().decode()?;
```

## Alignment Requirement

`with_archived_rkyv` and `decode_rkyv` must preserve existing alignment safety:

- Check pointer alignment before direct `rkyv::access`.
- Copy to `AlignedVec<16>` when unaligned.
- Never use `rkyv::access_unchecked`.

This behavior is currently documented in `crates/db/src/codec.rs` and must remain true after the refactor.

## Key Ordering Finding

Generic `RkyvKey<T>` key ordering should initially decode both sides and compare `T: Ord`.

This is correct but may be slower for hot B-tree paths. The planned comment should explicitly mark the ceiling:

```rust
// ponytail: decode-on-compare; specialize hot keys only after profiling.
```

If hot keys need faster ordering later, add domain-specific ordered byte encodings rather than complicating the generic adapter now.

## Multimap Constraint

redb multimap values must implement `Key`, so `RkyvMultimap<K, V>` must use `RkyvKey<V>`, not `RkyvValue<V>`.

This means multimap values must have meaningful total ordering (`V: Ord`) if using the generic rkyv multimap wrapper.

If a multimap value has no semantic ordering, do not force it into `RkyvMultimap`; keep a raw/specialized definition.

## Export Finding

`crates/db/src/lib.rs` currently re-exports only:

```rust
pub use codec::ArchivedEntity;
```

The refactor needs exports for codec errors, typed bytes, bounds traits, codec functions, and table wrappers. Keep `ArchivedEntity` during migration but deprecate it.

Refined visibility decision: do not export codec free functions unless needed. Prefer exporting `RkyvBytes` methods and keeping `encode_rkyv`, `decode_rkyv`, and `with_archived_rkyv` private implementation helpers.

## Visibility Policy Finding

Use least visibility:

- Public: types and methods repository adapters must name from other crates.
- Public sealed traits: only if needed in public method bounds (`RkyvEncode`, `RkyvDecode`).
- Private: free functions, alignment helpers, type-name helpers, and codec internals.

`RkyvEncode` and `RkyvDecode` likely need to be public because public `RkyvBytes` methods use them in bounds. Make them sealed to avoid inviting manual impls.

Standalone codec functions are not part of the preferred public interface. They exist to avoid duplicating logic between `RkyvBytes` methods, redb adapters, and the temporary `ArchivedEntity` shim. They should start private.

## Complexity Assessment

The full payoff appears only when a table migrates to `RkyvTable` / `RkyvMultimap`.

Before table migration, `RkyvBytes` is mostly a clearer, typed, better-error replacement for `ArchivedEntity`.

After table migration, redb `AccessGuard::value()` returns `RkyvBytes<'_, T>`, so call sites naturally become:

```rust
let value = guard.value().decode()?;
```

## Design Guardrails

- Do not big-bang replace `ArchivedEntity` across the repo.
- Migrate one table vertically first.
- Keep `RkyvTable`/`RkyvMultimap` minimal until usage proves helper methods are worth adding.
- Keep ordered-byte key specialization out until profiling proves decode-and-compare is too slow.

## GitNexus and Rust-Skills Review Gaps

### Duplicate Planning Sections

`task_plan.md` had duplicate `CodecError` / `CodecErrorKind` sections after iterative updates. This was cleaned up. Keep only one source of truth per component.

### Graph Impact Blind Spot

GitNexus impact reports LOW for `ArchivedEntity` and `DbError`, but text usage is broad. This is likely because blanket trait methods and enum variants are not fully represented as direct graph edges.

Planning should continue to use both GitNexus and text search for this refactor.

### Existing Optimized Keys Must Stay

Rust/API review identified a key design risk: generic `RkyvTable<K, V>` would replace optimized key encodings if used everywhere.

Existing key integrations should generally remain:

- `impl_redb_uuid!` / `UuidV7DbType`
- `DbPathKey`
- `EventId`

The first migration should likely use `RkyvValue<V>` behind an existing optimized key wrapper, not `RkyvKey<K>` for everything.

### Value-Only Wrapper Gap

The plan now needs to consider value-only wrappers such as:

```rust
TableDefinition<'static, K, RkyvValue<V>>
```

This may be exposed as existing wrappers with `RkyvValue<V>` or a small dedicated wrapper only if repetition appears.

### Debug Bound Requirement

`redb::Value` requires both the redb value type and `SelfType<'a>` to implement `Debug`.

Therefore these must implement `Debug`:

- `RkyvValue<T>`
- `RkyvKey<T>`
- `RkyvBytes<'a, T>`

Avoid adding `T: Debug` if possible; manual `Debug` impls can avoid leaking unnecessary bounds.

### Public Export Correction

The public exports section must not export private codec free functions by default. Public API should prefer `RkyvBytes` methods; free functions stay private unless a real caller requires them.

## Missing Implementation Inventory

Checked against `task_plan.md` after the first codec/table-wrapper slices.

Implemented:

- `CodecError` and `CodecErrorKind`.
- `DbError::Codec` and `DbErrorKind::Codec`.
- `RkyvBytes<'a, T>` with borrowed/owned/as_bytes/encode/decode/with_archived.
- Private `encode_rkyv`, `decode_rkyv`, and `with_archived_rkyv`.
- `RkyvValue<T>` redb `Value` adapter.
- `RkyvKey<T>` redb `Value + Key` adapter.
- Minimal `RkyvTable<K,V>` and `RkyvMultimap<K,V>` wrappers.
- `ArchivedEntity` compatibility shim routes through typed codec errors for reads and archived access.

Not implemented or incomplete:

- `RkyvDecode` trait is not implemented/exported. Current code repeats long decode bounds directly on public methods and helpers.
- `RkyvKey<T>::compare` does bytewise comparison, not plan-required decode-and-compare with `T: Ord` and invalid-byte panic.
- `RkyvKey<T>::compare` has no `ponytail:` comment documenting the performance ceiling and upgrade path.
- `RkyvTable<K,V>` currently wraps `TableDefinition<'static, K, RkyvValue<V>>`, not planned `TableDefinition<'static, RkyvKey<K>, RkyvValue<V>>`.
- `RkyvMultimap<K,V>` currently wraps `MultimapTableDefinition<'static, K, RkyvKey<V>>`, not planned `MultimapTableDefinition<'static, RkyvKey<K>, RkyvKey<V>>`.
- `ArchivedEntity` is not marked deprecated.
- `ArchivedEntity::to_bytes` still calls `rkyv::to_bytes` directly instead of delegating to `RkyvBytes::encode`/`encode_rkyv`; this preserves the `AlignedVec` return but duplicates encode mapping.
- `ArchivedEntity` doc comments still mention `DbError::Serialization` / `DbError::Deserialization` instead of `DbError::Codec`.
- `DbError::Serialization(String)` and `DbError::Deserialization(String)` are not deprecated.
- No first vertical migration has been done. Existing repository adapters still mostly use `ArchivedEntity::{to_bytes, from_bytes, with_archived}` or direct `rkyv::to_bytes`.
- No per-domain redb trait impls have been removed.
- No broad cleanup has removed `ArchivedEntity` or legacy string serialization/deserialization variants.
- No value-only migrated table using an optimized key (`UuidTable<K, RkyvValue<V>>`, `PathTable<RkyvValue<V>>`, or equivalent) has been added.

Test gaps from the plan:

- No direct tests for `CodecError::kind()` returning all of `Encode`, `Access`, and `Decode`.
- No direct `DbError::Codec` kind test, only legacy variants and codec-returning invalid-byte paths.
- No direct `RkyvBytes::with_archived()` test through the new `RkyvBytes` API; current zero-copy tests still use `ArchivedEntity::with_archived`.
- Invalid/truncated bytes are tested through `ArchivedEntity::from_bytes`, not directly through `RkyvBytes::borrowed(...).decode()` and `CodecError::RkyvAccess`/`RkyvDeserialize`.
- No `RkyvKey<T>::compare` test against semantic `T::cmp`.
- Debug no-`T: Debug` requirement is implemented manually but not covered by a compile test using a non-`Debug` `T`.

## Full Implementation Follow-up

Completed after the gap review:

- Added public sealed `RkyvDecode` with codec methods so public `RkyvBytes` APIs no longer expose long rkyv bounds.
- Added direct tests for `CodecError::kind`, `DbError::Codec`, `RkyvBytes::with_archived`, direct invalid-byte decode, no-`T: Debug` debug impls, semantic `RkyvKey::compare`, and fully generic `RkyvTable`/`RkyvMultimap` definitions.
- Changed `RkyvKey<T>::compare` to decode-and-compare with `T: Ord`, with the planned ponytail performance-ceiling comment.
- Changed `RkyvTable<K,V>` and `RkyvMultimap<K,V>` to use `RkyvKey<K>` for their redb key side.
- Migrated `EventTable<V>` to the value-only optimized-key shape: `TableDefinition<'static, EventId, RkyvValue<V>>`.
- Changed `EventStore<E>` bounds from `ArchivedEntity` to `RkyvEncode + RkyvDecode`.
- Deprecated `ArchivedEntity` and legacy string codec error variants, with local compatibility allows where old behavior is intentionally tested.
- Updated `ArchivedEntity` docs to report `DbError::Codec` and routed `to_bytes` through the new codec helper.

Still intentionally not done:

- Broad migration of all workspace repository adapters off `ArchivedEntity`.
- Removal of `ArchivedEntity`, `DbError::Serialization`, and `DbError::Deserialization`; they remain migration shims.
- Removal of per-domain redb trait impls outside the first DB-owned `EventTable` migration.

## Adapter Collapse Update

`RkyvValue<T>` and `RkyvKey<T>` were collapsed into `DbRkyvType<T>` after review. The split exposed two names for the same redb byte adapter; the only extra behavior was `redb::Key`.

Current shape:

```rust
pub struct DbRkyvType<T>(PhantomData<T>);

impl<T: 'static> redb::Value for DbRkyvType<T> { /* typed RkyvBytes */ }

impl<T> redb::Key for DbRkyvType<T>
where
    T: RkyvDecode + Ord + 'static,
{
    /* decode-and-compare */
}
```

`RkyvBytes<'a, T>` remains separate because it is the byte carrier and validation/decode interface, not the redb adapter type.

Updated table shapes:

```rust
TableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>>
MultimapTableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>>
TableDefinition<'static, EventId, DbRkyvType<V>>
```
