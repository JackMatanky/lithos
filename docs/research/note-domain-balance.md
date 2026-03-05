# Note Domain Balance: Ingest Artifact vs Persistent Aggregate

This document is a deep dive into the Note context architecture, the
SourceLocation dilemma, and the balance between DDD and a cache-driven
file-backed model. It is based on:

- The full `lithos-core/src/note/` module (aggregate, adapters, ports,
  structures, tasks, values, positions).
- Obsidian API types (Loc/Pos and metadata cache positioning).
- Basalt note_editor (source ranges and editor-centric positions).
- rust-analyzer architecture (input source data vs derived caches).
- Rust best practices for lean, performant, idiomatic data models.

Goal
Provide a critical, adversarial decision record and a concrete refactor plan
that preserves DDD benefits without letting a Note aggregate block the
capabilities Lithos needs (vault indexing, templates, LSP/IDE).

----------------------------------------------------------------
1. Evidence and constraints
----------------------------------------------------------------

1.1 Obsidian API (positioning model)
- Obsidian exposes positions via `MetadataCache` items, not a persisted
  domain entity. `CacheItem.position: Pos` where:
  - `Loc` = { line (0-based), col, offset (character count) }
  - `Pos` = { start: Loc, end: Loc }
- `frontmatterPosition` is a Pos as well.
- This implies positions are cached metadata, derived from source, and
  can be rebuilt on demand.

1.2 Basalt note_editor (editor model)
- Basalt keeps source ranges in AST nodes and updates them with edits.
- Positions are tied to the editing model and are not domain state.
- This matches a read-model or editor cache, not a persistent aggregate.

1.3 Lithos note module (current model)
- Note aggregate stores:
  - `links`, `tags`, `headings`, `lists`, `tasks`, `sections`.
  - Each sub-entity stores `SourceByteOffset` or ranges.
- There is a `SourceLineIndex` and `SourceLocation` but no stored source
  text or line index in the Note aggregate.
- Task projection exists (`StoredTask`) for task queries and indexes.
- Adapters parse markdown and emit domain entities directly.

1.4 rust-analyzer architecture (input vs derived)
- rust-analyzer treats source text as input state and semantic structures
  as derived state.
- It never does I/O inside the semantic model, and all derived data can be
  recomputed from input changes.
- The architecture separates input source from derived caches to enable
  incremental updates and correctness.

1.5 Rust best practices relevant here
- Avoid storing data twice unless performance or correctness demands it.
- Derived data belongs in caches or projections, not core domain models.
- Prefer explicit rebuildable caches to avoid stale state.
- Keep domain types lean and validation-focused.
- Use newtypes for offsets and IDs, but avoid bloating domain entities with
  editor-specific state unless it is a true invariant.

----------------------------------------------------------------
2. The core dilemma
----------------------------------------------------------------

We need positions (line/col/offset) for:
- LSP/IDE features (go-to, diagnostics, quick edits).
- Task queries and UI rendering with source references.
- Template queries that reference specific blocks.

But current Note aggregate only stores byte offsets. It cannot compute
`SourceLocation` without source text or a line index. If we add line/col
to domain entities, we pollute the domain with editor concerns and risk
stale data when the file changes outside the DB.

The question becomes: what is the role of the Note aggregate?

----------------------------------------------------------------
3. Adversarial decision analysis
----------------------------------------------------------------

3.1 What are the real invariants?
From your answers and the code:
- List semantics: checkbox vs task promotion.
- Frontmatter validation against config-defined fields.
- Future schema alignment between frontmatter and schema.

These are ingest-time invariants. They do not require a persistent
aggregate for long-lived identity or lifecycle management. They require a
validation pass during parsing and indexing.

3.2 Is Note a domain entity or a parsing artifact?
- If Note is the source of truth, then raw markdown is secondary. That
  contradicts the file-backed vault model.
- If files are the source of truth, then Note is an ingest artifact and
  projections are the durable read model.

3.3 Adversarial stance
- A persistent Note aggregate increases staleness risk, especially for
  positions and file edits outside the DB.
- Positions are derived metadata; they belong in a cache, not the domain.
- Keeping Note as a persistent aggregate forces the system to choose
  between stale positions or duplicated source storage.

Conclusion: In a file-backed vault, Note should not be a persistent
domain aggregate. It should be a validated ingest artifact, and the
read model should be projection-driven.

----------------------------------------------------------------
4. Recommended architecture
----------------------------------------------------------------

4.1 Core principle
- Source files are the ground truth.
- Parsing produces a validated ingest artifact (ParsedNote).
- Read models are stored projections (StoredTask, StoredNoteIndex).
- Projections are rebuildable and versioned.

4.2 DDD preserved where it matters
- Keep domain validation in task, tag, frontmatter, schema modules.
- These enforce real invariants at ingest.
- Avoid treating Note itself as an aggregate with lifecycle state.

4.3 Proposed data flow
1) Read markdown file.
2) Parse with NoteReader into ParsedNote.
3) Validate frontmatter and task metadata against config/schema.
4) Produce projections:
   - StoredTask (task list)
   - StoredNote (note metadata cache, positions)
5) Write projections to DB.

4.4 Position model
- Compute `Loc`/`Pos` during parse using:
  - SourceByteOffset
  - SourceLineIndex (built from source text)
- Store positions in StoredNote and StoredTask.
- Keep domain entities with byte offsets only.

----------------------------------------------------------------
5. Proposed module structure
----------------------------------------------------------------

5.1 note::domain (invariants only)
- frontmatter.rs
- tag.rs
- task.rs
- value.rs
- paths.rs
- position.rs
- link.rs
- list.rs
- structure.rs

5.2 note::adapter::reader (transient)
- ParsedNote resides in adapter::reader as the output of NoteReader.
- Introduce adapter::ingest.rs only if ingestion is split from parsing.
- adapter::extract_* remain parsing logic.

5.3 note::projection (read models)
- adapter::stored::StoredTask (already done)
- new StoredNote for note-level cache
- query adapters use projections, not Note aggregate

5.4 note::ports
- Command ports accept ParsedNote or raw parse results to write projections.
- Query ports return projections, not domain aggregates.

----------------------------------------------------------------
6. Per-file impact analysis (note/ module)
----------------------------------------------------------------

This section outlines the exact impact for every file in
`lithos-core/src/note/`.

aggregate.rs
- Rename Note -> ParsedNote (or keep Note but mark as ingest-only).
- Remove persistent storage expectations (no DB storage of Note).
- Keep it as in-memory validated structure for projections.

command.rs
- Update command API to accept ParsedNote or parse outcome rather than Note.
- Add projection write methods: write StoredTask, StoredNote.
- Remove Note storage calls from the command port implementation.

error.rs
- Split errors into:
  - Parse/validate errors (ingest)
  - Projection/storage errors (read models)
- Keep TaskError/TagError/FrontmatterError as validation errors.

events.rs
- If Note is ingest-only, domain events should be re-evaluated.
- Consider emitting events in application services rather than domain.

frontmatter.rs
- Remains domain validation and access, used in ingest.
- Add helpers for projection serialization if needed.

link.rs
- Remains domain value object; positions remain byte offsets.
- Position conversions happen in projection layer.

list.rs
- Remains domain value object; may gain parent linkage in ingest only.

mod.rs
- Update module docs to reflect ingest artifact + projections.
- Expose projection types in adapter or new projection module.
- Rename NOTE table definitions if Note storage is removed.

paths.rs
- No changes (still core domain value object).

ports.rs
- Command port becomes projection write boundary.
  - create/update/delete are likely replaced by ingest + project methods.
- Query port should return projections (StoredTask, StoredNote), not Note.
- Add a clear rebuild API for projections.

position.rs
- Remains domain primitives.
- SourceLineIndex used by projections, not by domain.

query.rs
- Query facade returns projections, not Note aggregates.
- Provide task and note cache query methods.

structure.rs
- Remains domain values; positional info stays byte offsets.

tag.rs
- Remains domain value object.

task.rs
- Remains domain entity for validation, but not persisted as Note.
- May move TaskId generation to ingest rules if needed.

value.rs
- Remains shared domain value type for metadata.

adapter/reader.rs
- Builds ParsedNote and projections with SourceLineIndex.
- Should compute `Pos` for each extracted entity if StoredNote is introduced.

adapter/extract_*.rs
- Continue to produce domain value objects (Task, Tag, Heading).
- Should surface offsets/ranges sufficient for projection positions.

adapter/stored.rs
- StoredTask already exists. Keep it in adapter layer.
- Introduce StoredNote (note metadata cache) here.

adapter/command.rs
- Becomes projection writer (StoredTask, StoredNote).
- Stop storing ParsedNote/Note in DB.

adapter/query.rs
- Query DB against projections; do not load Note aggregates.

----------------------------------------------------------------
7. Ports and adapters architecture impact
----------------------------------------------------------------

7.1 Command ports
- Replace create/update/delete on Note with:
  - ingest_and_index (input: ParsedNote or raw text)
  - rebuild_note_indexes (rebuild projection from file set)
  - rebuild_task_indexes (already exists)

7.2 Query ports
- Return stored projections and cache items:
  - StoredTask for task queries
  - StoredNote for note metadata queries
- Optional: provide zero-copy access with `with_*` pattern for large caches.

7.3 Adapter responsibilities
- Reader parses raw markdown and collects offsets.
- Command adapter writes projections.
- Query adapter reads projections.

----------------------------------------------------------------
8. StoredNote proposal (minimal)
----------------------------------------------------------------

Fields (example, no schema commitment yet):
- note_id, path, title
- frontmatter map (optional)
- tags
- headings with Pos
- sections with Pos
- list items and tasks with Pos
- block IDs and anchors

Purpose
- Match Obsidian MetadataCache semantics.
- Enable fast query + LSP without domain storage.

----------------------------------------------------------------
9. Naming: Stored* vs *View
----------------------------------------------------------------

Recommendation: keep `StoredTask` and `StoredNote` as storage DTOs.

Why not `TaskView`/`NoteView`
- "View" implies a UI or presentation layer. These structs are storage
  projections optimized for indexes and queries.
- The codebase already uses `Stored*` in adapter storage types.
- Keeping `Stored*` makes it obvious the data is rebuildable and not
  canonical.

If a UI-facing type is needed later, add a separate `TaskView` in the UI
layer or a `note::view` module that maps from `Stored*`.

RawNote vs ParsedNote
- `RawNote` should refer to raw source content + path (unparsed input).
- `ParsedNote` should be the validated ingest artifact (lists, tasks,
  frontmatter, headings) produced from `RawNote`.
- They are not the same. `RawNote` is input; `ParsedNote` is derived.

----------------------------------------------------------------
10. Pros and cons of the shift
----------------------------------------------------------------

Pros
- Eliminates stale position data in domain.
- Aligns with Obsidian and Basalt models.
- Encourages clear rebuildable caches.
- Improves LSP/IDE readiness.
- Keeps DDD for validation, not for storage.

Cons
- Requires refactor of ports and adapters.
- Loses the ability to treat Note as a persisted entity.
- Requires more explicit rebuild and invalidation strategies.

----------------------------------------------------------------
11. Lean, performant, idiomatic Rust alignment
----------------------------------------------------------------

- Keep domain types small and validation-focused.
- Keep projections in adapter layer (rkyv optimized).
- Avoid storing source text in DB unless profiling demands it.
- Use SourceLineIndex at parse time to build Loc/Pos.
- Keep query interfaces focused on read models.
- Use newtypes and avoid allocations for hot paths (Box<str> vs String).

----------------------------------------------------------------
12. Migration steps (high level)
----------------------------------------------------------------

1) Introduce StoredNote in adapter layer.
2) Update NoteReader to compute positions and build StoredNote.
3) Update CommandAdapter to write StoredNote instead of Note.
4) Update QueryAdapter to read StoredNote.
5) Deprecate Note persistence and rename Note -> ParsedNote.
6) Add rebuild endpoints for projections.

----------------------------------------------------------------
13. Event-driven design: full option space
----------------------------------------------------------------

This section enumerates all viable event-driven patterns for a file-backed
vault. Each option includes pros/cons and suitability for Lithos.

Option A: Pure event-driven indexing (notifications only)
- Emit events at ingest boundaries:
  - NoteParsed
  - FrontmatterValidated
  - TasksIndexed
  - NoteIndexUpdated
- Events are used to trigger projections and downstream pipelines.

Pros
- Simple and lean; minimal storage overhead.
- Does not conflict with file-as-truth.
- Easy to implement incrementally.

Cons
- Events are not replayable for rebuilding state.
- Limited audit/debug value (events are ephemeral unless persisted).

Option B: Persisted event log (audit stream)
- Persist the same events from Option A into an append-only log.
- Do not replay for reconstruction; use as audit + debugging trail.

Pros
- Preserves auditability without fighting file truth.
- Enables debugging and regression analysis.

Cons
- Extra storage and migration complexity.
- Still requires file parsing for state reconstruction.

Implementation note: NOTE_EVENTS table
- Add a `NOTE_EVENTS` table to persist ingest events without storing raw
  file content.
- Suggested fields: event_id, note_id, path, event_type, timestamp,
  source_hash, task_count, tag_count, error_code.
- Keep the payload small and stable; use versioned event types.

Option C: Change-capture events (diff-based)
- Emit events with file hashes, size, timestamp, and optional diff summary.
- Projections rebuild from file content, but downstream systems can act on
  deltas rather than reprocessing the whole file.

Pros
- Efficient for large vaults; supports incremental work.
- Good for caches and watchers.

Cons
- Diff computation adds cost.
- Still not replayable to rebuild without files.

Incremental updates with Option C
1) Store file fingerprints (hash, size, mtime) in StoredNote.
2) On scan, compare current file hash to stored hash.
3) If unchanged, skip parse/index and emit no event.
4) If changed, emit `NoteChanged` (old_hash, new_hash, size, mtime).
5) Reparse only that file and update its projections + indexes.
6) Mark updated `last_indexed_at` and `source_hash` in StoredNote.

This yields staleness tracking without full vault reindex.

Option D: Cache invalidation events
- Emit `InvalidateNote(path)` events on file change.
- Consumers recompute only affected projections.

Pros
- Simple and performant for large vaults.
- Aligns with Obsidian/MetadataCache semantics.

Cons
- Requires robust dependency tracking to avoid missed invalidations.

Option E: Projection event sourcing (derived-state log)
- Store events representing projection updates (e.g., TaskAdded,
  TaskRemoved, MetadataChanged) instead of raw file changes.
- Projections can be rebuilt by replaying these events, but only if they
  were generated from file ingestion.

Pros
- Replayable for projections without storing full source.
- Useful for debugging cache anomalies.

Cons
- Complex to keep consistent with file truth.
- Requires strict guarantees that ingestion is the only write path.

Option F: Full event sourcing of file state (not recommended now)
- Store raw file changes as events and replay to reconstruct files.
- Treat event log as canonical state; file system becomes a view.

Pros
- Complete audit history and deterministic rebuilds.

Cons
- Conflicts with file-backed vaults unless Lithos is the only editor.
- Very high complexity and performance cost.

Recommendation (optimal for Lithos)
- Combine Option A + Option D as the baseline:
  - Emit ingest events for observability and downstream pipelines.
  - Use invalidation events for incremental cache rebuilds.
- Add Option C selectively when diff-based deltas materially reduce work
  (large vaults, expensive projections, or heavy downstream consumers).
- Option B is a safe add-on if auditability becomes important.
- Avoid Options E/F until Lithos controls all writes.

----------------------------------------------------------------
14. Open questions
----------------------------------------------------------------

- Should ParsedNote be exposed publicly or stay internal to adapter?
- What minimal metadata does StoredNote need for templates and LSP?
- How to version projections for future migrations?
- What is the required level of consistency for frontmatter-schema
  validation (strict vs lenient)?

----------------------------------------------------------------
15. Implementation plan (TDD + checklists)
----------------------------------------------------------------

This plan is intentionally exhaustive. Each phase is structured as:
Red (tests) -> Green (implementation) -> Refactor (cleanup).

Phase 0: Baseline test scaffolding
- Tests
  - Adapter parse fixtures: empty note, frontmatter-only, tasks-only,
    mixed features, unicode, CRLF.
  - Position invariants: offsets and ranges for headings, tasks, links.
  - Task inline metadata: brackets and parens.
  - Emoji dates: config + default emoji (📅 ⏳ 🛫 ➕ ✅ ❌).
- Checklist
  - [ ] Deterministic RawNote fixtures (path + content)
  - [ ] Snapshot helpers for ParsedNote + StoredNote
  - [ ] Unicode boundary tests for SourceByteOffset

Phase 1: ParsedNote in adapter::reader
- Tests
  - ParsedNote is produced by NoteReader.
  - ParsedNote contains all domain entities with byte offsets only.
- Checklist
  - [ ] ParsedNote type defined in `adapter/reader.rs`
  - [ ] No persistence of ParsedNote
  - [ ] Update adapter docs/examples

Phase 2: StoredNote projection
- Tests
  - StoredNote includes headings/sections/tasks with Pos.
  - StoredNote stores source_hash/mtime/size/last_indexed_at.
- Checklist
  - [ ] Add StoredNote to `adapter/stored.rs`
  - [ ] Add table definition in `note::db_table`
  - [ ] Add serialization for Pos/Loc

Phase 3: SourceLocation computation pipeline
- Tests
  - Loc/Pos computed correctly for headings/tasks/links.
  - CRLF and unicode line/col correctness.
- Checklist
  - [ ] Build SourceLineIndex during parse
  - [ ] Convert SourceByteOffset -> Loc/Pos at projection time
  - [ ] Ensure line/col semantics documented (0 vs 1 based)

Phase 4: Command adapter writes projections only
- Tests
  - Insert/Update writes StoredNote + StoredTask.
  - Update removes old indexes for the note.
- Checklist
  - [ ] Remove NOTES persistence from command adapter
  - [ ] Write StoredNote table
  - [ ] Keep task indexes in sync

Phase 5: Query adapter reads projections
- Tests
  - list/find by path/tag/frontmatter returns StoredNote.
  - task queries return StoredTask unchanged.
- Checklist
  - [ ] Update Query port return types
  - [ ] Update QueryAdapter to use StoredNote
  - [ ] Update Query facade methods

Phase 6: Ingest validation rules
- Tests
  - Invalid frontmatter fields rejected per config.
  - Task metadata type validation errors.
- Checklist
  - [ ] Validation step before projection write
  - [ ] Consistent error reporting

Phase 7: NOTE_EVENTS table (Option B)
- Tests
  - Ingest emits NoteParsed + NoteIndexUpdated.
  - Failures emit NoteParseFailed with error code.
- Checklist
  - [ ] Add NOTE_EVENTS table
  - [ ] Add NoteEvent DTO
  - [ ] Emit events from command adapter

Phase 8: Incremental updates (Option C + D)
- Tests
  - Unchanged hash skips parse and indexing.
  - Changed hash reindexes only that note.
  - Deleted file removes projections.
- Checklist
  - [ ] Store hash/mtime/size in StoredNote
  - [ ] Emit NoteChanged / InvalidateNote events
  - [ ] Reindex single note path

Phase 9: Cleanup and migration
- Tests
  - Rebuild commands match full ingest output.
- Checklist
  - [ ] Remove aggregate storage paths
  - [ ] Update docs/examples
  - [ ] Add migration notes and versioning

Phase 10: Performance and correctness gates
- Checklist
  - [ ] Benchmark parse/index on 1k/10k notes
  - [ ] Memory profile of StoredNote indexes
  - [ ] Verify no cross-context imports
  - [ ] `mise run verify` green

Pulldown-cmark optimization checklist
- [ ] Reuse a single Parser instance per note (no nested parser builds).
- [ ] Avoid allocating strings per Event; prefer CowStr borrowed path.
- [ ] Use event range offsets directly; avoid substring scans.
- [ ] Minimize intermediate Vec allocations by reserving capacity.
- [ ] Keep extractor state machines single-pass; no backtracking.
- [ ] Avoid expensive regex; rely on pulldown-cmark events.
- [ ] For inline field parsing, scan only task text (not full file).
- [ ] Ensure task metadata parsing does not re-scan section text.

----------------------------------------------------------------
16. File-by-file migration checklist (all note/ files)
----------------------------------------------------------------

The checklist below enumerates every file in `lithos-core/src/note/` and
the concrete migration steps required for the new architecture.

aggregate.rs
- [ ] Replace Note aggregate with ParsedNote alias or remove and redirect
      to adapter::reader::ParsedNote.
- [ ] Remove persistence assumptions and DB helpers.
- [ ] Update module docs to "ingest artifact" terminology.
- Key functions:
  - Note::try_new / Note::new (if still present)
  - Note::add_* mutators (should become ingest-only builders)
- Tests:
  - ParsedNote creation without persistence paths

adapter/reader.rs
- [ ] Define ParsedNote (if not already) as NoteReader output.
- [ ] Build SourceLineIndex from source text.
- [ ] Collect byte offsets + ranges for all entities.
- [ ] Expose ParsedNote + SourceLineIndex for projection building.
- Key functions:
  - NoteReader::read / parse entrypoint
  - ExtractionContext::new / source handling
- Tests:
  - ParsedNote includes tags/headings/tasks/links
  - SourceLineIndex line/col with unicode and CRLF

adapter/extract_heading.rs
- [ ] Ensure heading positions use SourceByteOffset.
- [ ] Provide heading range (start/end) if StoredNote requires Pos.
- [ ] Add tests for Unicode and CRLF line offsets.
- Key functions:
  - HeadingExtractor::process
  - HeadingBuilder::build
- Tests:
  - heading Pos from event ranges

adapter/extract_section.rs
- [ ] Ensure section ranges are valid boundaries.
- [ ] Confirm section heading assignment remains stable.
- [ ] Provide section Pos derived from ranges.
- Key functions:
  - SectionExtractor::start_block / end_block
  - SectionExtractor::close_current
- Tests:
  - section range maps to heading

adapter/extract_list.rs
- [ ] Ensure tasks store byte offsets.
- [ ] Parse inline fields (brackets + parens) and emoji dates.
- [ ] Ensure list nesting depth is preserved for parent linkage.
- [ ] Add edge cases for nested lists and checkbox vs task.
- Key functions:
  - ListExtractor::parse_* methods
  - InlineFieldState::parse_inline_fields
  - InlineFieldState::fill_default_emoji_dates
- Tests:
  - nested lists parent_id
  - task vs checkbox promotion

adapter/extract_tag.rs
- [ ] Ensure tags are extracted once (no duplication).
- [ ] Preserve offsets if required by StoredNote.
- Key functions:
  - TagExtractor::process
- Tests:
  - duplicate tags collapse

adapter/extract_link.rs
- [ ] Ensure link positions and anchors are captured.
- [ ] Provide block/heading anchors for StoredNote.
- Key functions:
  - LinkExtractor::process
- Tests:
  - block ref anchor Pos

adapter/stored.rs
- [ ] Add StoredNote DTO (note metadata cache with Pos/Loc).
- [ ] Ensure StoredTask remains adapter-only storage DTO.
- [ ] Add helpers for Pos/Loc serialization.
- Key functions:
  - StoredNote::new/accessors
  - Pos/Loc newtypes + rkyv derives
- Tests:
  - StoredNote rkyv round-trip

adapter/command.rs
- [ ] Stop writing NOTES table.
- [ ] Write StoredNote + StoredTask.
- [ ] Update index insertion/removal to use StoredNote.
- [ ] Emit NOTE_EVENTS when enabled.
- Key functions:
  - insert_indexes / remove_indexes
  - collect_index_data -> StoredNote
- Tests:
  - update removes old indexes

adapter/query.rs
- [ ] Read StoredNote for note-level queries.
- [ ] Keep StoredTask queries intact.
- [ ] Add note query methods for headings/tags/links if needed.
- Key functions:
  - list_by_* methods returning StoredNote
- Tests:
  - queries return StoredNote Pos

adapter/mod.rs
- [ ] Re-export ParsedNote from reader.
- [ ] Re-export StoredNote/StoredTask as adapter DTOs.
- Key functions:
  - pub use declarations

command.rs
- [ ] Replace Note-based commands with ParsedNote ingestion methods.
- [ ] Add rebuild_note_indexes entrypoint.
- Key functions:
  - Command::ingest_and_index
  - Command::rebuild_note_indexes

query.rs
- [ ] Update query facade to return StoredNote for note-level queries.
- [ ] Add helper methods for Pos-based outputs.
- Key functions:
  - Query::find_by_path
  - Query::list_by_* returning StoredNote

ports.rs
- [ ] Command port: replace create/update/delete with ingest/index methods.
- [ ] Query port: return StoredNote for note queries.
- [ ] Add projection rebuild APIs (note + task).
- Key functions:
  - Command::ingest_and_index
  - Command::rebuild_note_indexes
  - Query::find_by_path -> StoredNote

mod.rs
- [ ] Update module docs to "ingest artifact + projections" model.
- [ ] Update type aliases for new ports/adapters.
- [ ] Update db_table definitions: remove NOTES, add STORED_NOTES.
- Key functions:
  - db_table definitions
  - type aliases

error.rs
- [ ] Split errors into ingest vs projection categories.
- [ ] Ensure NoteError doesn’t imply persistence semantics.
- Key functions:
  - NoteError variants and docs

events.rs
- [ ] Define NoteEvent DTOs (Parsed, Indexed, Changed, Failed).
- [ ] Make events independent of Note aggregate state.
- Key functions:
  - NoteEvent enum/structs

frontmatter.rs
- [ ] Ensure validation is performed during ingest.
- [ ] Add helpers for projection serialization if needed.
- Key functions:
  - Frontmatter::try_get* accessors

link.rs
- [ ] Keep as domain value object.
- [ ] Ensure anchor/block IDs are exposed for projections.
- Key functions:
  - Link::anchor / Anchor::text

list.rs
- [ ] Ensure list items expose depth for parent linkage.
- [ ] Keep task linkage logic in ingest, not domain.
- Key functions:
  - ListItem::depth / ListItem::task

paths.rs
- [ ] No changes expected; keep validation rules.
- Key functions:
  - NotePath::try_new / FolderPath::try_new

position.rs
- [ ] Clarify line/col semantics (0 vs 1 based) for Pos.
- [ ] Provide conversion utilities for StoredNote Pos.
- Key functions:
  - SourceLineIndex::line_column
  - SourceLocation::try_from_byte_offset

structure.rs
- [ ] Ensure headings/sections expose offsets for Pos.
- [ ] Add block-id anchors if required by StoredNote.
- Key functions:
  - Heading::position
  - Section::range

tag.rs
- [ ] No changes expected; ensure tag extraction matches StoredNote needs.
- Key functions:
  - Tag::try_new / Tag::try_from_token

task.rs
- [ ] Keep validation logic for task fields.
- [ ] Ensure task metadata is stored for projection indexing.
- Key functions:
  - Task::metadata / TaskMetadata::fields

value.rs
- [ ] No changes expected; used for frontmatter/task metadata.
- Key functions:
  - FieldValue accessors
