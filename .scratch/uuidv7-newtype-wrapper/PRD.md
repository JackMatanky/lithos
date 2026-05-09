# UUIDv7 Hardening - Shared Support Type Implementation

## Problem Statement

The lithos codebase contains multiple UUID-backed ID wrapper types (`SchemaId`, `PropertyId`, `NoteId`, `ListItemId`, `VaultId`, and later `TemplateId`) that share nearly identical implementation patterns:

**Duplicated types identified:**
- SchemaId, PropertyId exposed: `from_uuid`, `as_uuid`, `into_uuid`
- NoteId exposed: `parse`, `From<Uuid>`, `From<NoteId> for Uuid`
- VaultId exposed: `uuid() -> Uuid` by value
- ListItemId had only: `new` + `Default` + `Display`

**Shared pattern:**
- `new()` constructor generating UUID v7
- `Default` delegates to `new`
- `Display` prints the inner UUID
- Private tuple-field wrapping `uuid::Uuid`

This duplication creates maintenance burden and prevents enforcing UUID v7 invariants at system boundaries (e.g., in database APIs).

**Pre-existing infrastructure:** The DB layer already had UUID-specialized APIs (`get_by_uuid`, `put_by_uuid`, etc.) that encoded UUIDs into stack buffers to avoid allocation. However, these accepted any `uuid::Uuid` (any version), not just v7.

## Solution

Introduce a shared `UuidV7` primitive type that:
- Enforces UUID v7 invariant at construction/parsing time
- Provides zero-copy access to inner `uuid::Uuid`
- Integrates with rkyv for archive/serialize/deserialize
- Becomes the canonical type for database UUID-keyed APIs

Domain ID wrappers wrap `UuidV7` as their inner storage while maintaining their distinct types for context isolation.

## User Stories

1. As a developer, I want a single source of truth for UUID v7 generation so that I don't duplicate generation logic across contexts
2. As a developer, I want database APIs to accept only valid UUID v7 values so that I cannot accidentally insert non-v7 UUIDs
3. As a developer, I want consistent conversion methods across all ID wrappers so that adapter code is predictable
4. As a developer, I want ID wrappers to remain distinct types so that I cannot accidentally use a `SchemaId` where a `NoteId` is expected

## Implementation Decisions

### Completed Steps

| Step | Description | Status |
|------|-------------|--------|
| 1 | Add `UuidV7` support type with unit tests | Complete |
| 2 | Add internal macro for context ID wrappers | **Pending** |
| 3 | Pilot migration: `SchemaId` and `PropertyId` wrap `UuidV7` | Complete |
| 4 | Migrate DB UUID API | Complete |
| 5 | Migrate remaining ID wrappers | Complete |
| 6 | Verify and benchmark | Complete |
| 7 | API hardening | Complete |
| 8 | DB UUID helper dedupe | Complete |

### Design Decisions

- **Newtypes over type aliases** for context isolation
- **rkyv derives at wrapper level** to preserve archived type clarity
- **Validated construction only** (`try_from_uuid`, not unchecked)
- **DB signature migration** from `uuid::Uuid` to `UuidV7`
- **Standardized trait surface** across all wrappers: `new()`, `as_uuid()`, `into_uuid()`, `Display`, `Default`, `From<Uuid>`

### Open Decisions (Resolved)

- Only validated construction, no fast-path `from_uuid`
- Full DB migration, no parallel raw-UUID APIs
- Serde derives added only where context wrappers need them

## Testing Decisions

- Unit tests for `UuidV7` construction, parsing, and roundtrip
- Integration tests for DB UUID-keyed operations
- Targeted tests for each migrated ID wrapper
- Full verification via `mise run verify`

## Out of Scope

- Binary UUID keys for database tables (separate ADR)
- `BlockRefId` (string-backed semantic ID)
- Name wrappers (`*Name` types with `Box<str>` validation)

## Further Notes

Implementation completed in a single day (2026-05-06). The codebase is in a working state with all tests passing.
