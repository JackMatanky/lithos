# Rkyv redb Codec Refactor Progress

## Session Log

### 2026-07-05

- Loaded planning-with-files workflow.
- Ran session catchup command; no unsynced context was reported.
- Confirmed no project-root `task_plan.md`, `findings.md`, or `progress.md` existed.
- Reviewed current `crates/db/src/codec.rs` content supplied by user.
- Used GitNexus exploring on `ArchivedEntity`.
- GitNexus impact for `ArchivedEntity` reported LOW, but text search found broad method-syntax usage.
- Reviewed `crates/db/src/table.rs`, `read.rs`, and `write.rs` to understand existing table wrapper seam.
- Created root planning files for this refactor.

## Files Created

- `task_plan.md`
- `findings.md`
- `progress.md`

## Current Decision

Use `RkyvBytes<'a, T>` as the typed byte carrier.

Add `RkyvTable<K, V>` and `RkyvMultimap<K, V>` as minimal table-definition wrappers to hide repeated `RkyvKey<T>` / `RkyvValue<T>` usage from table constants.

### 2026-07-05 Follow-up

- User noted planning files did not include all designed components.
- Expanded `task_plan.md` with the full component inventory and detailed APIs.
- Expanded `findings.md` with component relationships, `ArchivedEntity` replacement detail, alignment requirement, key ordering, multimap constraints, and export requirements.
- Confirmed `RkyvTable` / `RkyvMultimap` should remain minimal initially.

### 2026-07-05 Concrete API Correction

- User pointed out earlier concrete API sketches were still not fully represented.
- Added a dedicated `Concrete API Sketch` section to `task_plan.md` with public types, `RkyvBytes` methods, codec bounds, and `CodecError`/`CodecErrorKind` code.
- Added matching concrete API notes to `findings.md`.
- Captured correction that generic rkyv multimap values use `RkyvKey<V>`, not `RkyvValue<V>`, because redb multimap values must implement `Key`.

### 2026-07-05 Visibility Policy

- Added least-visibility policy to `task_plan.md`.
- Added visibility findings to `findings.md`.
- Decided standalone codec functions should start private, not public, because `RkyvBytes` methods are the intended public interface.
- Noted `RkyvEncode` / `RkyvDecode` may need to be public sealed traits if they appear in public method bounds.

## Next Step

Before implementation, choose the first low-risk table for a vertical migration and run GitNexus impact on the concrete symbols to be edited.
