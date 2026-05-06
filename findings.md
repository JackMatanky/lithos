# Findings

## Repeated Type Discovery

### UUID-backed ID wrappers (strong duplication)

- `SchemaId` in `lithos-core/src/schema/identifier.rs:54`
- `PropertyId` in `lithos-core/src/schema/property.rs:552`
- `NoteId` in `lithos-core/src/note/aggregate.rs:54`
- `ListItemId` in `lithos-core/src/note/list.rs:58`
- `VaultId` in `lithos-core/src/config/vault.rs:41`

Shared shape across these five types:

- new UUID v7 constructor (`new`)
- `Default` delegates to `new`
- `Display` prints inner UUID
- private tuple-field wrapping `Uuid`

Small API differences today:

- `SchemaId`/`PropertyId` expose `from_uuid`, `as_uuid`, `into_uuid`
- `NoteId` exposes `parse`, `From<Uuid>`, `From<NoteId> for Uuid`
- `VaultId` exposes `uuid() -> Uuid` by value
- `ListItemId` has only `new` + `Default` + `Display`

Implication: There is enough commonality for a reusable support ID primitive while preserving per-context newtypes.

### Non-UUID ID wrapper (should stay specialized)

- `BlockRefId(Box<str>)` in `lithos-core/src/note/structure.rs:151`

This is a semantic ID (string-backed), not a UUID identity. Do not fold into UUID support type.

### Name wrappers with repeated Box<str> validation pattern (secondary candidate class)

- `PropertyName(Box<str>)` in `lithos-core/src/schema/property.rs:634`
- `VaultName(Box<str>)` in `lithos-core/src/config/vault.rs:692`
- `StatusName(Box<str>)` in `lithos-core/src/config/task.rs:568`
- `FieldName(Box<str>)` in `lithos-core/src/config/value.rs:590`
- `FileName(Box<str>)` in `lithos-core/src/fs/file.rs:45` (not validated the same way; utility semantics)

These share storage and conversion style but differ in validation constraints and domain meaning. Candidate for helper traits/macros, not a single shared concrete type.

## Candidate Design Direction

1. Add shared UUID primitive in support/root (e.g., `support::id::UuidV7Id` or `support::id::EntityId`).
2. Keep domain-specific wrappers (`SchemaId`, `NoteId`, etc.) as newtypes over shared primitive (preferred over type aliases).
3. Standardize trait surface across wrappers:
   - `new()`
   - `from_uuid(Uuid)`
   - `as_uuid(&self) -> &Uuid`
   - `into_uuid(self) -> Uuid`
   - `Display`, `Default`, `From<Uuid>`, `From<Self> for Uuid`
4. Optionally generate repetitive impls via a small internal macro (e.g., `uuid_id_type!(SchemaId)`) to cut boilerplate while preserving type safety.

## Constraints / Risks

- Prefer newtypes over aliases for context isolation and preventing accidental cross-context ID usage.
- Ensure `rkyv` derives remain available at wrapper level; do not hide derives solely in the shared primitive if it weakens archived type clarity.
- Maintain current UUID v7 behavior as persisted identity contract.

## DB / Persistence UUID Review

### What exists now

- DB layer has UUID-specialized APIs that avoid allocation by encoding UUIDs into stack buffers:
  - `Database::{get_by_uuid,get_owned_by_uuid}` in `lithos-core/src/db/reader.rs:75` and `lithos-core/src/db/reader.rs:102`
  - `Database::{put_by_uuid,delete_by_uuid}` in `lithos-core/src/db/writer.rs:50` and `lithos-core/src/db/writer.rs:99`
  - Same pattern repeated in `BatchReader`, `BatchWriter`, and `ReadWriteUnitOfWork` methods (`reader.rs` and `writer.rs`).
- Repository adapters already consume these APIs in many places:
  - `schema/storage.rs` calls `.get_owned_by_uuid(...)` / `.put_by_uuid(...)` with `id.into_uuid()`.
  - `note/storage.rs` calls `.get_by_uuid(...)` / `.get_owned_by_uuid(...)` with `Uuid::from(id)`.
  - `template/adapter/command.rs` calls `.put_by_uuid(...)` / `.delete_by_uuid(...)` with raw template UUIDs.

### Improvement opportunities tied to a primary UUID newtype

1. **Introduce shared `UuidV7` (or `Uuid7`) support type and use it in DB UUID APIs**
   - Today DB methods accept `uuid::Uuid` (any version).
   - Switching signatures to `UuidV7` enforces v7 invariant at DB boundary.
   - This prevents accidental insertion of non-v7 UUIDs from future call sites.

2. **Unify conversion surface to reduce adapter churn**
   - Current call sites bounce between `id.into_uuid()`, `Uuid::from(id)`, and raw `Uuid` fields.
   - With a common core ID type (`UuidV7`) and consistent conversions, adapters can pass IDs without ad-hoc conversion styles.

3. **Deduplicate UUID-to-key encoding boilerplate in DB internals**
   - Same 3-line buffer/encode snippet appears repeatedly across read/write types.
   - Add one internal helper (e.g., `uuid_key(uuid_v7) -> impl AsRef<str>`) to reduce repetition and drift risk.

4. **Template context remains a notable inconsistency**
   - `Template` currently stores `pub id: Uuid` in `lithos-core/src/template/aggregate.rs:268`.
   - Converting this to `TemplateId` over `UuidV7` would align with note/schema/config patterns and simplify DB APIs.

### Optional deeper optimization (separate decision)

- Current DB key type is `&str` for ID tables; UUIDs are stored as hyphenated text keys.
- Possible follow-up: use binary UUID keys (`[u8; 16]` or `&[u8]`) for ID-indexed tables to reduce key size and encode/decode overhead.
- This is a storage-schema decision and should be treated as a separate ADR-level change from introducing `UuidV7`.

## Draft: `UuidV7` Support Type

### Proposed location

- `lithos-core/src/support/uuid.rs` (re-exported via `support/mod.rs`)

### Proposed type surface

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct UuidV7(uuid::Uuid);

impl UuidV7 {
    pub fn new() -> Self;                       // generates now_v7
    pub const fn from_uuid(uuid: uuid::Uuid) -> Self;
    pub fn try_from_uuid(uuid: uuid::Uuid) -> Result<Self, UuidVersionError>;
    pub fn parse(s: &str) -> Result<Self, uuid::Error>; // parse + version check
    pub const fn as_uuid(&self) -> &uuid::Uuid;
    pub const fn into_uuid(self) -> uuid::Uuid;
}

impl Default for UuidV7;
impl core::fmt::Display for UuidV7;
impl From<UuidV7> for uuid::Uuid;
impl core::convert::TryFrom<uuid::Uuid> for UuidV7;
```

Notes:

- Keep `from_uuid` only if we explicitly want a trusted fast-path.
- If strict invariants are mandatory everywhere, omit `from_uuid` and require `try_from_uuid`.
- Prefer explicit error type for version mismatch (e.g., `UuidVersionError::ExpectedV7`).

### Suggested wrapper macro (internal)

```rust
uuid_v7_id_type!(SchemaId);
uuid_v7_id_type!(PropertyId);
uuid_v7_id_type!(NoteId);
uuid_v7_id_type!(ListItemId);
uuid_v7_id_type!(VaultId);
```

Macro would generate:

- tuple wrapper over `UuidV7`
- `new`, `from_uuid_v7`, `as_uuid_v7`, `into_uuid_v7`
- pass-through `as_uuid`, `into_uuid` convenience methods
- `Display`, `Default`, `From<UuidV7>`, and optionally `From<Wrapper> for uuid::Uuid`

## Draft: DB API migration

### Before

- `put_by_uuid(..., id: uuid::Uuid, ...)`
- `delete_by_uuid(..., id: uuid::Uuid)`
- `get_by_uuid(..., id: uuid::Uuid, ...)`
- `get_owned_by_uuid(..., id: uuid::Uuid)`

### After

- `put_by_uuid(..., id: UuidV7, ...)`
- `delete_by_uuid(..., id: UuidV7)`
- `get_by_uuid(..., id: UuidV7, ...)`
- `get_owned_by_uuid(..., id: UuidV7)`

### Internal helper to remove duplication

```rust
#[inline]
fn uuid_v7_key(id: UuidV7) -> [u8; 36] {
    let mut buf = [0u8; 36];
    let _ = id.as_uuid().hyphenated().encode_lower(&mut buf);
    buf
}
```

Call sites then borrow key as `&str` once, centrally.

## Draft: Migration sequence

1. Add `UuidV7` support type + tests.
2. Add/adjust DB methods to accept `UuidV7` (keep temporary overloads if needed).
3. Migrate `SchemaId` and `PropertyId` to wrap `UuidV7`.
4. Migrate `NoteId`, `ListItemId`, `VaultId`.
5. Introduce `TemplateId` (replace raw `Uuid` in template aggregate/ports).
6. Remove temporary overloads and raw-UUID fallback APIs.
7. Run `mise run verify` and measure any perf regressions on DB hot paths.

## Open decisions to resolve during implementation

- Should `UuidV7::from_uuid` exist at all, or only `TryFrom<Uuid>`?
- Should DB keep parallel raw-UUID APIs during transition?
- Should we rename methods from `*_by_uuid` to `*_by_id` once `UuidV7` is first-class?
- Do we add serde derives to `UuidV7` now, or only where needed by context wrappers?

## Verification-stage findings (Step 6)

- Full verification initially failed at lint stage due to bench call sites still passing `uuid::Uuid` into DB methods now typed as `UuidV7`.
- Failing files and call sites were all benchmark-only:
  - `lithos-core/benches/string_construction.rs`
  - `lithos-core/benches/db_storage.rs`
  - `lithos-core/benches/db_key_handling.rs`
- Resolution pattern:
  - Use `UuidV7::from_uuid_unchecked(Uuid::now_v7())` for benchmark-generated IDs.
  - Use typed ID accessors (`*note.id().as_uuid_v7()`, `*id.as_uuid_v7()`) at note bench call sites.
- Post-fix validation:
  - `mise run lint` passed.
  - `mise run verify` passed (unit, integration, and doc tests green; only non-fatal mise warnings about expected output artifact paths).

## Step 1 Spec (Implementation-Ready): `support/uuid.rs`

### Scope

Implement only the shared primitive type and module wiring:

- add `lithos-core/src/support/uuid.rs`
- export from `lithos-core/src/support/mod.rs`
- no context ID migrations yet
- no DB signature changes yet

### Final API contract (Step 1)

```rust
pub struct UuidV7(uuid::Uuid);

impl UuidV7 {
    pub fn new() -> Self;
    pub fn parse(input: &str) -> Result<Self, UuidV7Error>;
    pub fn try_from_uuid(uuid: uuid::Uuid) -> Result<Self, UuidV7Error>;
    pub const fn from_uuid_unchecked(uuid: uuid::Uuid) -> Self;
    pub const fn as_uuid(&self) -> &uuid::Uuid;
    pub const fn into_uuid(self) -> uuid::Uuid;
}
```

Trait impls in Step 1:

- `Default` (delegates to `new`)
- `Display`
- `TryFrom<uuid::Uuid> for UuidV7`
- `From<UuidV7> for uuid::Uuid`
- `FromStr for UuidV7`

Derives in Step 1:

- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`
- `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`

### Error type contract (Step 1)

```rust
pub enum UuidV7Error {
    Parse(uuid::Error),
    WrongVersion { got: Option<uuid::Version> },
}
```

Behavior rules:

- `parse` parses then enforces `Version::SortRand` (v7).
- `try_from_uuid` validates version.
- `from_uuid_unchecked` is for trusted internal paths only.
- error messages should include expected v7 vs actual version.

### Naming + conventions

- Type name: `UuidV7`.
- File name: `uuid.rs`.
- Keep methods aligned with existing ID wrappers (`as_uuid`, `into_uuid`).

### Module wiring

- In `lithos-core/src/support/mod.rs`:
  - `pub mod uuid;`
  - `pub use uuid::{UuidV7, UuidV7Error};`

If `support/mod.rs` does not exist yet, create it and wire from crate root consistently with existing module layout.

### Test checklist (Step 1)

Unit tests for `lithos-core/src/support/uuid.rs`:

1. `new_creates_v7_uuid`
   - assert `as_uuid().get_version() == Some(Version::SortRand)`.

2. `parse_accepts_valid_v7`
   - parse a known/generated v7 string and assert success.

3. `parse_rejects_non_v7`
   - parse v4 string, assert `WrongVersion`.

4. `parse_rejects_invalid_string`
   - assert `Parse` variant.

5. `try_from_uuid_rejects_non_v7`
   - pass v4 UUID, assert `WrongVersion`.

6. `roundtrip_into_from_uuid`
   - `UuidV7 -> Uuid -> TryFrom<Uuid> -> UuidV7` roundtrip succeeds.

7. `display_matches_inner_uuid`
   - `format!("{}", id)` equals inner UUID display.

8. `default_is_v7`
   - `UuidV7::default()` produces v7.

### Non-goals (explicit)

- Do not migrate `SchemaId`, `NoteId`, etc. yet.
- Do not modify `db/reader.rs` or `db/writer.rs` signatures yet.
- Do not introduce macro generation yet.

### Exit criteria for Step 1

- `UuidV7` and `UuidV7Error` compile and are exported.
- all Step-1 unit tests pass.
- no behavioral changes outside `support` module.

## Step 4 Live Notes (DB API Migration)

- Active scope: `lithos-core/src/db/reader.rs` and `lithos-core/src/db/writer.rs` UUID-keyed API signatures.
- Target change: all `id: uuid::Uuid` parameters on `*_by_uuid` methods move to `id: UuidV7`.
- Call-site strategy: migrate existing `SchemaId`/`PropertyId` usage first, then note/template/config usages that still pass raw UUID.
- Tooling caveat: GitNexus impact checks are currently unavailable (`Not connected`), so change safety is being validated with compile/test loops plus targeted integration tests.

### Step 4 implementation outcome

- Completed signature migration in:
  - `lithos-core/src/db/reader.rs`
  - `lithos-core/src/db/writer.rs`
- All `id: uuid::Uuid` parameters for DB `*_by_uuid` methods now use `id: UuidV7`.
- Updated immediate call sites:
  - `lithos-core/src/schema/storage.rs` now passes `*id.as_uuid_v7()`.
  - `lithos-core/src/note/storage.rs` adapts legacy `NoteId` UUIDs with `UuidV7::from_uuid_unchecked(...)` (temporary bridge until `NoteId` wraps `UuidV7`).
  - `lithos-core/src/template/adapter/command.rs` adapts raw template UUIDs with `UuidV7::from_uuid_unchecked(...)` (temporary bridge until `TemplateId` migration).
- Updated db-internal tests in `reader.rs` and `writer.rs` to pass `UuidV7`.

### Validation evidence

- `cargo check -p lithos-core` passes.
- `cargo test -p lithos-core --test schema_storage` passes (10 tests).
- `cargo test -p lithos-core note::storage` passes (2 tests).
- `cargo test -p lithos-core template::adapter::command` passes (targeted filter, no failures).

## Step 5 Live Notes (Remaining ID wrappers)

- Target wrappers: `NoteId`, `ListItemId`, `VaultId`, and template aggregate identity (`TemplateId`).
- Impact tooling status: GitNexus impact API currently unavailable (`Not connected`), so Step 5 proceeds with compile/test verification loops.

### Step 5 implementation outcome

- `NoteId` now wraps `UuidV7` in `lithos-core/src/note/aggregate.rs` and exposes `as_uuid_v7()`.
- `ListItemId` now wraps `UuidV7` in `lithos-core/src/note/list.rs` and exposes `as_uuid_v7()`.
- `VaultId` now wraps `UuidV7` in `lithos-core/src/config/vault.rs` and exposes `as_uuid_v7()`.
- Added `TemplateId(UuidV7)` in `lithos-core/src/template/aggregate.rs` and migrated template aggregate ID field/methods to use `TemplateId`.
- Migrated template interfaces to `TemplateId`:
  - `template/ports.rs`
  - `template/command.rs`
  - `template/query.rs`
  - `template/adapter/command.rs`
  - `template/adapter/query.rs`
  - `template/raw.rs`

### Validation evidence

- `cargo check -p lithos-core` passes.
- `cargo test -p lithos-core note::storage` passes.
- `cargo test -p lithos-core template::` passes.
- `cargo test -p lithos-core config::vault` passes.
- `cargo test -p lithos-core db::` passes.

## Step 6 Live Notes (Cleanup + Convergence)

- Goal: remove temporary `from_uuid_unchecked` bridges where practical and confirm invariant boundaries.
- Boundary policy: raw input structs may still carry `uuid::Uuid` (e.g., template raw input), but domain IDs should use `UuidV7` wrappers.
