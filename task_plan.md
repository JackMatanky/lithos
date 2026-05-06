# Task Plan

## Goal
Identify basic domain types duplicated across contexts (starting with UUID-backed `Id` types) and propose a shared support type strategy with concrete candidates and migration notes.

## Proposed Primary Type Naming

- Preferred: `UuidV7`
- Alternate: `Uuid7`
- Decision rationale: `UuidV7` is explicit, aligns with versioned naming style in Rust ecosystems, and is easy to grep.

## Phases
| Phase | Status | Notes |
|---|---|---|
| 1. Inventory repeated basic types | complete | Found 5 UUID-backed ID wrappers plus one string-backed ID |
| 2. Compare semantics and constraints | complete | Compared API differences (`parse`, `from_uuid`, `uuid`, conversions) |
| 3. Propose shared support design | complete | Proposed shared primitive + per-context newtype wrappers |
| 4. Summarize recommendations | complete | Included DB-specific UUID review and naming update (`UuidV7`/`Uuid7`) |

## Draft Execution Plan (UuidV7)

| Step | Status | Deliverable |
|---|---|---|
| 1. Add support primitive | complete | Added `support::uuid::UuidV7` + `UuidV7Error`, module export, and unit tests |
| 2. Add convenience macro | pending | Internal `uuid_v7_id_type!` macro for context ID wrappers |
| 3. Pilot one context | complete | Migrated `SchemaId` and `PropertyId` internals to wrap `UuidV7` and added pilot tests |
| 4. Migrate DB UUID API | complete | Migrated DB `*_by_uuid` signatures to `UuidV7` and adapted schema/note/template call sites |
| 5. Migrate remaining IDs | complete | Migrated `NoteId`, `ListItemId`, `VaultId`, and introduced `TemplateId` across template ports/adapters |
| 6. Verify and benchmark | complete | Fixed benchmark `Uuid`->`UuidV7` call sites, then ran `mise run lint` and `mise run verify` successfully |

## Errors Encountered
| Error | Attempt | Resolution |
|---|---|---|
