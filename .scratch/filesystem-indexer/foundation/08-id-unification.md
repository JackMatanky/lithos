---
title: 08-context-id-unification-onto-fsrecordid
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-07-01
date_completed:
---

# Issue 08: Unify context `*Id` types onto `FsRecordId`

## Why this is a foundation issue

This carries out an **already-accepted architecture decision** — ADR
`docs/adr/indexer/0001-fileid-as-universal-identity.md` (accepted 2026-05-28):
*"remove context-specific identity types (`SchemaId`, `NoteId`, `TemplateId`) and
use [the universal file id] as the identity for all file-backed entities."* That
ADR was written against the older `FileId` / `FILE_ID_BY_PATH` names; this issue
implements the same decision against the names foundation issue 07 establishes:
**`FsRecordId`** and the single **`FS_ID_BY_PATH`** table keyed by
`(PathKey, FsRecordKind)`.

It is foundation-owned because it re-keys storage across `crates/schema/`,
`crates/note/`, and `crates/template/` (inheritance graph, aggregate primary
keys, path indexes) — cross-context storage surgery, not integration-wire work.
The integration PRD (`.scratch/filesystem-indexer/integration/PRD.md` §5.1) names
it as a **hard prerequisite** of its schema section and does not describe the
refactor inline.

## Relationship to ADR `indexer/0001`

ADR `indexer/0001` is the originating decision and remains the rationale of
record. This issue does **not** re-open or re-status it; it only reconciles the
terminology and sequences the work after issue 07:

| ADR `indexer/0001` term | This issue (post-07) term |
| --- | --- |
| `FileId` | `FsRecordId` |
| `FILE_ID_BY_PATH` | `FS_ID_BY_PATH` (keyed `(PathKey, FsRecordKind)`) |
| `SchemaId` / `NoteId` / `TemplateId` | removed; replaced by `FsRecordId` |
| `SCHEMA_ID_BY_PATH` / `NOTE_ID_BY_PATH` | removed; resolve via `FS_ID_BY_PATH` |

If any of ADR `indexer/0001`'s siblings (the archived
`docs/adr/indexer/` design cluster) conflict with what shipped, resolving that is
out of scope here — this issue implements only the identity-unification decision.

## What to build

### 1. Delete the context id newtypes

Remove `SchemaId` (`crates/schema/src/identifier.rs:52`), `NoteId`
(`crates/note/src/aggregate.rs`), and `TemplateId`
(`crates/template/src/aggregate.rs`). Replace every use site with `FsRecordId`
(the indexer's record identity, `crates/indexer/src/model.rs`). This is a large,
mechanical-but-wide change — `SchemaId` alone has 100+ references across
`aggregate.rs`, `index.rs`, `repository.rs`, `schema_processor.rs`,
`base_processor.rs`.

### 2. Re-key schema's inheritance graph and index onto `FsRecordId`

- `InheritanceGraph<()>` topo order and edges (`crates/schema/src/discovery.rs`,
  `crates/schema/src/inheritance.rs`) become `FsRecordId`-keyed
  (file-to-file relationships, per ADR `indexer/0001` line 15).
- The schema index (`crates/schema/src/index.rs`) name↔id / path↔id maps
  (`NameIdPairs`, `PathIdPairs`) key on `FsRecordId`.
- Schema/Note/Template aggregate primary keys become `FsRecordId`
  (`SCHEMAS`, note/template aggregate tables).

### 3. Route all `*_id_by_path` lookups through `FS_ID_BY_PATH`

Remove the per-context path indexes (`SCHEMA_ID_BY_PATH`, `NOTE_ID_BY_PATH`, the
template equivalent). Path resolution becomes the two-step ADR `indexer/0001`
pattern: query `FS_ID_BY_PATH` → get `FsRecordId` → query the context aggregate
table by `FsRecordId`. `find_schema_ids_by_paths`
(`crates/schema/src/repository.rs:240`, `discovery.rs:305`) and its note/template
analogues resolve through issue 07's `find_id_by_path` instead of a context table.

### 4. Identity is derived from the file, not minted

There is no independent `SchemaId::new()` (`crates/schema/src/aggregate.rs:62`,
`crates/schema/src/base_processor.rs:317`) or `NoteId::new()` / `TemplateId::new()`
minting a fresh id. A file-backed entity's identity **is** its file's
`FsRecordId`, which the indexer already assigned and carries on the
`FileIndexEvent`. The processor adopts that id. One file → one entity. This is the
property the integration PRD depends on: `FileDeleted { id: FsRecordId }` lets the
delete path (`repo.delete_base_schema(id)` + `repo.delete_schema(id)`,
`crates/schema/src/base_processor.rs:64-66`) run straight off the event with **no
`PathKey → id` lookup**.

> ponytail: this is a big diff but a mechanical one — one id type instead of
> three, one path table instead of four. The lazy correct move is to let the file
> own identity and delete the parallel bookkeeping, exactly as ADR
> `indexer/0001` already decided.

## Out of scope (explicitly deferred)

- The `IndexEvent` sink, consumers, services — integration PRD, not this issue.
- Wire-domain `*Entry`→`*Event` renames — integration PRD §1.
- Resolving any other archived `docs/adr/indexer/` cluster decisions.

## Acceptance criteria

- [ ] `SchemaId`, `NoteId`, `TemplateId` are removed; `FsRecordId` is the only
      id for file-backed entities across schema/note/template.
- [ ] Schema's `InheritanceGraph` and index are `FsRecordId`-keyed; existing
      inheritance-resolution tests pass with the re-key.
- [ ] Per-context `*_ID_BY_PATH` tables are gone; path resolution goes through
      `FS_ID_BY_PATH` / `find_id_by_path` (issue 07).
- [ ] No `*Id::new()` mint sites remain for file-backed entities; identity is
      adopted from the file's `FsRecordId`.
- [ ] A schema/note/template can be deleted given only its `FsRecordId` (no
      path→id repo read), unblocking the integration PRD's deletion model.
- [ ] All existing tests pass (`mise run test`); no clippy warnings
      (`mise run lint`); formatted (`mise run fmt`).

## Blocked by

- **Foundation issue 07** (`07-storage-consolidation.md`) — provides
  `FsRecordId`, `FsRecordKind`, the single `FS_ID_BY_PATH` table, and
  `find_id_by_path`. Issue 08 re-keys the context storage onto those.

## Notes for the integration PRD (context, not scope)

- Once merged, the integration PRD's §5.1 collapses to a one-line "depends on
  foundation issue 08" and the schema deletion model (§6/§7.1/§8) needs no
  path→id lookup.
- The originating rationale lives in ADR
  `docs/adr/indexer/0001-fileid-as-universal-identity.md`; cite it, don't restate
  it.
