---
title: 07a-note-context-storage-migration
category: enhancement
label: ready-for-agent
status: in-progress
date_created: 2026-05-25
---

## Type

Implementation

## Labels

- ready-for-agent
- part-of-07

## What to build

Implement the new segregated storage module for the Note context following ADR 016 (Segregated Unified Repository Traits). This creates the foundation for migrating note persistence from `&'db Database` to `Arc<Store>` with proper trait segregation.

## Scope

This issue covers the Note context storage layer ONLY:
- Repository trait definitions (`note/repository.rs`)
- Storage implementation (`note/storage/`)
- In-memory test double (`note/storage/testing.rs`)
- Error handling updates

Out of scope:
- Vault context migration (see 07b)
- Call site migration (`note/processor.rs`, vault processor, tests) (see 07c)

## Current Status (2026-05-25)

✅ **Complete** — Phases 0-7 done:
- Phase 0: Design decisions approved
- Phase 1: `note/repository.rs` with `ReadRepository`, `WriteRepository`, `Repository` traits
- Phase 2: `note/storage/tables.rs` with typed table wrappers
- Phase 3: `note/storage/mod.rs` with `RedbRepository` struct using `Arc<Store>`
- Phase 4: `note/storage/read.rs` implementing `ReadRepository` (7 tests passing)
- Phase 5: `note/storage/write.rs` implementing `WriteRepository` (7 tests passing)
- Phase 6: `note/storage/testing.rs` with `InMemoryRepository` (22 tests passing)
- Phase 7: Error reduction to 4 tuple variants + naming alignment

## Key Decisions Made

1. **Naming**: `cache_list_view` → `save_list_view`, `invalidate_list_view` → `delete_list_view` per naming taxonomy
2. **Error design**: 4 tuple variants: `Storage(DbError)`, `NotFoundById(NoteId)`, `NotFoundByPath(NotePath)`, `DuplicatePath(NotePath)`
3. **No batch adapters**: High-level batch methods (`save_many`, `find_many_by_id`, `delete_many`) manage transactions internally
4. **Table types**: `UuidTable<NoteId, &[u8]>` for `NOTES`/`LIST_VIEWS`; `PathTable<&[u8]>` for `NOTE_ID_BY_PATH`
5. **Transaction scope**: Per-method — each repository method opens/closes its own transaction
6. **InMemoryRepository pattern**: Follows Schema pattern with `InMemoryHarness`, operation counters, failure injection

## Deliverables

- [x] `note/repository.rs` — segregated traits
- [x] `note/storage/mod.rs` — `RedbRepository` struct
- [x] `note/storage/tables.rs` — typed table definitions
- [x] `note/storage/read.rs` — `ReadRepository` impl (7 integration tests)
- [x] `note/storage/write.rs` — `WriteRepository` impl (7 integration tests)
- [x] `note/storage/testing.rs` — `InMemoryRepository` (22 unit tests)
- [x] `note/error.rs` — reduced to 4 variants + `From<InMemoryDbError>` impl
- [x] All tests pass strict clippy + fmt

## Blocks

- Issue 07b: Vault context storage migration
- Issue 07c: Processor and test migration (depends on 07a + 07b)

## Files Modified

```
lithos-core/src/note/
├── repository.rs          (new)
├── storage/
│   ├── mod.rs             (new)
│   ├── tables.rs          (new)
│   ├── read.rs            (new)
│   ├── write.rs           (new)
│   └── testing.rs         (new)
├── error.rs               (updated)
└── mod.rs                 (exports updated)
```

## Next Steps

This issue is complete. See:
- **07b** for vault context migration (vault module needs full db-refactor treatment)
- **07c** for processor/test migration (blocked until 07b provides `Database` → `Store` bridge)

---

**Status**: ✅ Complete (2026-05-25)
