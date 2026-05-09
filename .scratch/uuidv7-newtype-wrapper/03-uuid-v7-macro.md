---
parent: UUIDv7 Hardening PRD
labels: needs-triage
status: pending
---

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
