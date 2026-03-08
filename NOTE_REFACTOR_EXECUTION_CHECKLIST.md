# Note Refactor Execution Checklist (File Source of Truth + Flat Structure)

Status: Draft for execution
Date: 2026-03-06

This checklist adapts the schema refactor lessons to the note module while
respecting note-specific decisions (persistent NOTE_EVENTS, ParsedNote +
Stored* remain core artifacts).

----------------------------------------------------------------
0) Pre-flight
----------------------------------------------------------------
- [ ] Working tree clean (git status)
- [ ] Current tests pass (mise run test)
- [ ] Read docs/research/note-domain-balance.md
- [ ] Confirm event log remains persistent (NOTE_EVENTS is retained)

----------------------------------------------------------------
1) Establish flat module structure
----------------------------------------------------------------
- [ ] Move adapter files to note root (rename in-place):
  - [ ] note/adapter/command.rs -> note/db_command.rs
  - [ ] note/adapter/query.rs -> note/db_query.rs
  - [ ] note/adapter/reader.rs -> note/reader.rs
  - [ ] note/adapter/stored.rs -> note/stored.rs
  - [ ] note/adapter/extract_*.rs -> note/extract_*.rs
- [ ] Remove note/adapter/mod.rs and adapter/ directory
- [ ] Add note/db_tables.rs and move table constants from note/mod.rs
- [ ] Update all imports across the workspace to new module paths
- [ ] Update note/mod.rs exports to flat modules only
- [ ] Move vault-wide discovery to application/vault.rs (VaultService::load)

----------------------------------------------------------------
2) Clarify ingest and read model types
----------------------------------------------------------------
- [ ] Keep ParsedNote as the ingest artifact, but relocate it to avoid
      adapter-coupling in ports:
  - [ ] Option A: move ParsedNote to note/parsed.rs
  - [ ] Option B: keep in note/reader.rs but re-export as note::ParsedNote
      and ensure ports refer to note::ParsedNote (not adapter path)
- [ ] Keep StoredNote and StoredTask as read models (projections)
  - [ ] Ensure they remain storage-optimized and rebuildable
  - [ ] Avoid calling them "DTO" in code/docs

----------------------------------------------------------------
3) Ports and facades update (file-centric, flat)
----------------------------------------------------------------
- [ ] Update note/ports.rs to depend on note::ParsedNote and note::stored::*
      (not adapter module paths)
- [ ] Ensure Command port remains projection-writer:
  - [ ] record_parsed_note(path, parsed)
  - [ ] record_deleted_note(note_id)
  - [ ] rebuild_note_indexes()
  - [ ] rebuild_task_indexes()
- [ ] Ensure Query port returns StoredNote/StoredTask and offers zero-copy
      accessors where needed (with_* pattern)
- [ ] Keep thin command/query facades only if they add value (error conversion)

----------------------------------------------------------------
4) NOTE_EVENTS persistence (note-specific)
----------------------------------------------------------------
- [ ] Retain NOTE_EVENTS table and event DTOs in note/events.rs
- [ ] Ensure only one event system remains (remove legacy NoteEvents)
- [ ] Emit persistent NOTE_EVENTS in db_command on ingest/update/delete
- [ ] Add event versioning stability notes in rustdoc

----------------------------------------------------------------
5) Loader (or service) consolidation
----------------------------------------------------------------
- [ ] Move note orchestration out of application/note.rs into note/loader.rs
- [ ] Loader owns: scan -> parse -> validate -> project -> index -> emit events
- [ ] Ensure incremental staleness logic stays in loader (hash/mtime/size)
- [ ] Update application module to use note::loader

----------------------------------------------------------------
6) Remove legacy aggregate
----------------------------------------------------------------
- [ ] Remove note/aggregate.rs from public API
- [ ] Delete legacy aggregate events from note/events.rs
- [ ] Rewrite tests to use ParsedNote + StoredNote/StoredTask only

----------------------------------------------------------------
7) Reader and extractor performance fixes
----------------------------------------------------------------
- [ ] Avoid CowStr/Range clones per extractor in reader loop
- [ ] Remove duplicate string buffers in extract_list
- [ ] Avoid HashSet<Box<str>> duplication in extract_tag
- [ ] Make SourceLineIndex optional/lazy when positions not needed

----------------------------------------------------------------
8) Stored projection minimization
----------------------------------------------------------------
- [ ] Audit StoredNote fields; keep only query/index needs
- [ ] Remove redundant/unused fields (e.g., legacy status_type)
- [ ] Ensure StoredTask and StoredNote are rebuildable from source

----------------------------------------------------------------
9) Value model boundary cleanup
----------------------------------------------------------------
- [ ] Move JSON/YAML/TOML conversions out of note/value.rs
- [ ] Keep FieldValue as pure domain type (no format IO)

----------------------------------------------------------------
10) Tests and verification
----------------------------------------------------------------
- [ ] Update unit tests for new module paths
- [ ] Add/adjust tests for NOTE_EVENTS persistence
- [ ] Run full verification (mise run verify)

----------------------------------------------------------------
Notes on ports (answering design questions)
----------------------------------------------------------------
- StoredNote/StoredTask are read models (projections). They remain necessary.
- ParsedNote is the ingest artifact and should remain.
- Ports should therefore accept ParsedNote (ingest input) and return Stored*
  (read model output). The key change is removing adapter-path dependencies by
  relocating or re-exporting ParsedNote in note root.
