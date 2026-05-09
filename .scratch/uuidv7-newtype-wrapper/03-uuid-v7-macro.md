---
title: 03-uuid-v7-macro
category: enhancement
labels: needs-triage
status: pending
date_created: 2026-05-06
date_completed:
---

# UUIDv7 Macro

## Parent

UUIDv7 Hardening PRD

## What to build

Implement an internal `uuid_v7_id_type!` macro in `lithos-core/src/support/` that generates boilerplate for UUID-backed ID wrapper types.

### Macro Design

The macro should generate for each wrapper type (e.g., `SchemaId`, `NoteId`, etc.):

- Tuple wrapper over `UuidV7`
- `new()` constructor (generates now_v7)
- UUID v7 methods: `from_uuid_v7`, `as_uuid_v7`, `into_uuid_v7`
- Convenience passthroughs: `as_uuid()` (returns `&Uuid`), `into_uuid()` (returns `Uuid`)
- Trait impls: `Display`, `Default`, `From<UuidV7>`, `From<Self> for Uuid`
- rkyv derives at wrapper level (not delegated to inner `UuidV7` alone)

### Context

This is the only pending step. All other steps (1, 3-8) are complete:
- Step 1: UuidV7 support type implemented and tested
- Step 3-8: All ID wrappers migrated, DB APIs updated, API hardened, benchmarks fixed

### Usage Example

```rust
uuid_v7_id_type!(SchemaId);
uuid_v7_id_type!(PropertyId);
uuid_v7_id_type!(NoteId);
uuid_v7_id_type!(ListItemId);
uuid_v7_id_type!(VaultId);
```

## Acceptance criteria

- [ ] Macro defined in `lithos-core/src/support/` module (internal, not publicly exported)
- [ ] Macro generates correct impl for at least one existing ID wrapper (`SchemaId` or `NoteId`)
- [ ] Generated code compiles and passes existing tests
- [ ] No runtime behavior change to existing ID wrappers

## Blocked by

- 001-uuidv7-support-type (must complete first)
- 002-id-wrapper-migrations (must complete first - provides the ID wrappers to macro-ize)

## Notes

- Step 2 of the implementation plan (pending)
- This is a convenience refactor; ID wrappers currently work without the macro
- Keep newtype wrappers (not type aliases) for context isolation
- Macro should be internal to avoid encouraging direct use outside the intended pattern
