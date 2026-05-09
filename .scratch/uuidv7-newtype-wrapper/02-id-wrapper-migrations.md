---
parent: UUIDv7 Hardening PRD
labels: needs-triage
status: completed
date_created: 2026-05-06
date_completed: 2026-05-06
---

## Parent

UUIDv7 Hardening PRD

## What to build

Migrate existing UUID-backed ID wrapper types and database APIs to use `UuidV7`, including pilot migration, DB signature changes, remaining wrappers, benchmark fixes, API hardening, and helper deduplication.

### Step 3: Pilot Migration (SchemaId, PropertyId)

Migrate first two ID wrappers to wrap `UuidV7` internally:

1. **SchemaId** (`lithos-core/src/schema/identifier.rs:55`)
   - Now wraps `UuidV7` internally
   - Exposes `as_uuid_v7(&self) -> &UuidV7`
   - Added test: `schema_id_exposes_uuid_v7_view`

2. **PropertyId** (`lithos-core/src/schema/property.rs:553`)
   - Now wraps `UuidV7` internally
   - Exposes `as_uuid_v7(&self) -> &UuidV7`
   - Added test: `property_id_exposes_uuid_v7_view`

### Step 4: DB UUID API Migration

Update `*_by_uuid` methods in:

- `lithos-core/src/db/reader.rs`
- `lithos-core/src/db/writer.rs`

Parameters changed from `id: uuid::Uuid` to `id: UuidV7`.

Updated call sites:
- `lithos-core/src/schema/storage.rs`: passes `*id.as_uuid_v7()`
- `lithos-core/src/note/storage.rs`: adapts with `UuidV7::from_uuid_unchecked(...)` (temporary bridge)
- `lithos-core/src/template/adapter/command.rs`: adapts with `UuidV7::from_uuid_unchecked(...)` (temporary bridge)
- DB internal tests in `reader.rs` and `writer.rs` updated to pass `UuidV7`

### Step 5: Remaining ID Wrapper Migrations

1. **NoteId** (`lithos-core/src/note/aggregate.rs:57`)
   - Now wraps `UuidV7` internally
   - Exposes `as_uuid_v7(&self) -> &UuidV7`

2. **ListItemId** (`lithos-core/src/note/list.rs:57`)
   - Now wraps `UuidV7` internally
   - Exposes `as_uuid_v7(&self) -> &UuidV7`

3. **VaultId** (`lithos-core/src/config/vault.rs:42`)
   - Now wraps `UuidV7` internally
   - Exposes `as_uuid_v7(&self) -> &UuidV7`

4. **TemplateId** (`lithos-core/src/template/aggregate.rs:51`)
   - New type introduced, wraps `UuidV7`
   - Migrated template interfaces:
     - `template/ports.rs`
     - `template/command.rs`
     - `template/query.rs`
     - `template/adapter/command.rs`
     - `template/adapter/query.rs`
     - `template/raw.rs`

### Step 6: Benchmark Fixes

Update benchmark call sites that passed raw `uuid::Uuid` to DB methods now typed as `UuidV7`:

- `lithos-core/benches/string_construction.rs`
- `lithos-core/benches/db_storage.rs`
- `lithos-core/benches/db_key_handling.rs`

Resolution pattern:
- Use `UuidV7::try_from_uuid(Uuid::now_v7()).expect(...)` for benchmark-generated IDs
- Use typed ID accessors (`*note.id().as_uuid_v7()`, `*id.as_uuid_v7()`) at note bench call sites

### Step 7: API Hardening

Remove `from_uuid_unchecked` from public API surface:
- Removed `UuidV7::from_uuid_unchecked` from `lithos-core/src/support/uuid.rs`
- Removed `TemplateId::from_uuid_unchecked` from `lithos-core/src/template/aggregate.rs`
- Migrated benchmark constructors to validated conversion with explicit invariant checks

Net effect: API surface now prefers validated construction (`try_from_uuid`, `TryFrom<Uuid>`)

### Step 8: DB Helper Deduplication

Add internal helper to reduce repeated UUID key encoding boilerplate:

- Added `with_uuid_v7_key` helper to `lithos-core/src/db/reader.rs`
- Added `with_uuid_v7_key` helper to `lithos-core/src/db/writer.rs`
- Replaced repeated inline buffer/encode blocks in Database, BatchReader, BatchWriter, and ReadWriteUnitOfWork UUID paths
- Preserved zero-allocation behavior and call-site signatures

## Acceptance criteria

- [x] SchemaId and PropertyId wrap UuidV7 (pilot tests pass)
- [x] DB UUID-keyed APIs accept UuidV7 parameters
- [x] NoteId, ListItemId, VaultId wrap UuidV7
- [x] TemplateId introduced and template interfaces migrated
- [x] Benchmark call sites updated to pass UuidV7
- [x] from_uuid_unchecked removed from public API
- [x] with_uuid_v7_key helper added to DB reader/writer
- [x] mise run lint passes
- [x] mise run verify passes (985 unit tests, 36 integration tests, doctests)

## Blocked by

- 001-uuidv7-support-type (must complete first - need `UuidV7` type to exist)

## Validation evidence

- `cargo check -p lithos-core` passes
- `cargo test -p lithos-core --test schema_storage` passes (10 tests)
- `cargo test -p lithos-core note::storage` passes (2 tests)
- `cargo test -p lithos-core template::adapter::command` passes
- `cargo test -p lithos-core note::storage` passes
- `cargo test -p lithos-core template::` passes
- `cargo test -p lithos-core config::vault` passes
- `cargo test -p lithos-core db::` passes
- `mise run lint` passes
- `mise run verify` passes (985/985 unit tests, 36/36 integration tests)

## Notes

- Covers Steps 3, 4, 5, 6, 7, and 8 of the implementation plan
- Maintains context isolation (distinct types prevent cross-context ID usage)
- GitNexus impact check was unavailable (`Not connected`) - proceeded with compile/test verification
- Error encountered: initial `mise run verify` failed at lint due to benchmark call sites - resolved by updating benchmarks
