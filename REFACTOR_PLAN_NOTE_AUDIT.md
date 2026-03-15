# Note Context Refactor Plan Audit (Start)

**Context:** `lithos-core/src/note/`
**Template:** `REFACTOR_PLAN_CHECKLIST_TEMPLATE.md`
**Primary Authority:** `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`

---

## 0) Inputs and Constraints

- [x] Read `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md` (primary authority).
- [x] Read `_bmad-output/project-context.md` and confirm latest rules.
- [x] Read ADR 002 (Repository) only for historical context.
- [x] Read `docs/refs/rust/naming-taxonomy.md` and confirm method naming rules.
- [x] Confirm context isolation: note must not import schema/template.
- [x] Confirm file-based source-of-truth requirement for this refactor.
- [x] Reviewed Obsidian API + Dataview + Tasks references for semantics (reference only, not replicas).
- [x] Reviewed pulldown-cmark research and Rust parser digests (Basalt + obsidian-parser).

### Section Map (How to Read This Audit)

- **1–3:** current state inventory and alignment gaps
- **4–6:** target module structure + pipeline design + Raw/Domain boundaries
- **7–8:** migration steps and verification plan
- **9:** output deliverables
- **10–11:** optional Markdown module split and extractor strategy (decision guidance)
- **12:** context-specific parsing lessons applied

---

## 1) Full File and Component Audit (Context Inventory)

### 1.1 File Inventory

Directory: `lithos-core/src/note/`

- `mod.rs` — module registry + table definitions; exports CQRS modules + domain types; deps: `redb`; ownership: public API + crate-private tables.
- `loader.rs` — orchestration; types: `Loader`, `LoadError`; deps: `thiserror`, `DbError`; ownership: public API.
- `db_command.rs` — write adapter + indexing; types: `CommandAdapter`, `IndexData`, `TaskIndexData`; deps: `redb`, `uuid`, `blake3`, `itoa`, `ryu`; ownership: public adapter.
- `db_query.rs` — read adapter + indexes; types: `QueryAdapter`; deps: `redb`, `uuid`, `itoa`; ownership: public adapter.
- `ports.rs` — CQRS traits; types: `Command`, `Query`; deps: none (uses domain types); ownership: public API.
- `reader/mod.rs` — ingestion parser; types: `NoteReader`, `ParsedNote`; deps: `pulldown_cmark`, `FsReader`; ownership: public adapter (helpers internal).
- `reader/frontmatter.rs` — frontmatter tag/link collection; deps: `Config`; ownership: internal.
- `reader/links.rs` — link builder wiring; deps: `pulldown_cmark::LinkType`; ownership: internal.
- `reader/lists.rs` — list item + task promotion; deps: `Config`, `TaskBuilder`; ownership: internal.
- `reader/sections.rs` — section tracking + block ref scan; deps: `pulldown_cmark`; ownership: internal.
- `reader/tags.rs` — tag collection helper; deps: none; ownership: internal.
- `reader/state.rs` — list item record helper; deps: `StatusSymbol`; ownership: internal.
- `stored.rs` — stored projections; types: `StoredNote`, `StoredTask`, `StoredListItem`; deps: `rkyv`, `SystemTime`; ownership: storage shape (public).
- `events.rs` — event log types; types: `NoteEvent*`; deps: `rkyv`, `uuid`; ownership: public.
- `error.rs` — domain + ingest + storage errors; types: `NoteError`, `NoteIngestError`, `TaskError`, `FrontmatterParseError`; deps: `thiserror`; ownership: public.
- `frontmatter.rs` — frontmatter parsing + accessors; types: `Frontmatter`, `FrontmatterFormat`; deps: `serde_yaml`, `toml`, `Config`; ownership: public (parsing helpers should move).
- `value.rs` — dynamic value + conversions; types: `FieldValue`, `FieldValueType`; deps: `serde_yaml`, `serde_json`, `toml`, `chrono`; ownership: public.
- `heading.rs` — heading domain + builder; types: `Heading`, `HeadingLevel`, `HeadingBuilder`; deps: none; ownership: public (builder internal).
- `link.rs` — link domain + parsing helpers; types: `Link`, `Target`, `Anchor`, `LinkBuilder`; deps: none; ownership: public (builders/internal helpers should move).
- `list.rs` — list domain + builders; types: `List`, `ListItem`, `ListItemBuilder`; deps: `StatusSymbol`; ownership: public (builders/internal helpers should move).
- `tag.rs` — tag domain + scanning; types: `Tag`, `scan_tags`; deps: none; ownership: public (scan_tags should move).
- `task.rs` — task domain + parsing; types: `Task`, `TaskBuilder`, `TaskMetadata`; deps: `Config`, `chrono`, `uuid`; ownership: public (parsing helpers should move).
- `structure.rs` — sections + block refs; types: `Section`, `BlockRef`; deps: none; ownership: public.
- `paths.rs` — path value objects; types: `NotePath`, `FolderPath`; deps: none; ownership: public.
- `position.rs` — source positions/ranges; types: `SourceByteOffset`, `SourceByteRange`, `SourceLocation`; deps: none; ownership: public.
- `identity.rs` — note identity; types: `NoteId`, `AliasName`, `FileClassName`; deps: `uuid`; ownership: public.

### 1.2 Component Inventory (Initial)

- Domain types: `Heading`, `Task`, `Link`, `List`, `Tag`, `Section`, `BlockRef`, `NoteId`
- Parsing/Projection: `reader/*` (mixed responsibilities)
- Raw types: **none yet** (to be introduced in `raw/`)
- Storage types: `StoredNote`, `StoredTask`, `StoredListItem` (to be removed in favor of domain storage shape)
- Ports/Adapters: CQRS `ports.rs`, `db_command.rs`, `db_query.rs`
- Orchestration: `loader.rs`
- Errors: `error.rs` (needs audit of variants + construction patterns)
- Ownership boundary: public API is `NoteReader`, `ParsedNote`, domain types, `db_*` adapters; internal helpers are `reader/*` submodules and builders (`HeadingBuilder`, `LinkBuilder`, `ListItemBuilder`, `TaskBuilder`).

### 1.3 Cross-File Coupling Audit (Initial)

- Reader is used by storage adapter, mixing parsing and persistence.
- Domain files contain parsing helpers (task/link/tag/value/frontmatter).
- CQRS split used across note ports + db_command/db_query.
- Cyclic dependencies: none obvious in module graph, but reader ↔ domain coupling is tight (reader uses domain builders and helpers).
- God modules: `reader/mod.rs` (single-pass parse + extraction), `db_command.rs` (indexing + projection + event logging), `frontmatter.rs` (parse + access + conversions).

---

## 2) Workflow and Pipeline Audit (Behavioral Inventory)

### 2.1 Pipeline Map (Current)

1) Reader parses markdown and derives projections.
2) `db_command` consumes ParsedNote (parser output) and persists stored projections.
3) `db_query` reads stored projections.

**Entry points:** `loader::load_content`, `NoteReader::parse`, `NoteReader::parse_content`, `CommandAdapter::upsert_parsed_note`.
**Error boundaries:** `NoteReader` → `NoteIngestError`; `db_command`/`db_query` → `DbError`; `loader` → `LoadError` (wraps ingest + storage).

### 2.2 Bloat and Inefficiency Checks (Initial)

- Parsing + projection mixed in reader (duplicated logic across helpers).
- Domain types include parsing helpers (violates parse/validate boundary).
- Storage adapter performs projection logic (layering violation).

### 2.3 Modularity and Isolation Checks (Initial)

- pulldown-cmark types leak outside parser via reader helpers.
- No Raw* layer for notes yet (missing file → raw → domain boundary).
- Loader is not the only orchestrator yet (reader + db_command coordinate pipeline).

---

## 3) Architecture Alignment Audit (Initial)

- [ ] Raw types serde-only? **No** (no RawNote type; reader parses directly).
- [ ] Raw parsing helpers to avoid persisting invalid raw views? **No** (no Raw layer).
- [ ] Domain types validated and used as storage shape? **Partial** (Stored* types dominate).
- [ ] Unified Repository trait? **No** (CQRS ports present).
- [x] File ingestion uses FsReader? (`NoteReader::parse` uses `FsReader`).
- [x] Context isolation? (note imports config/db/fs only; no schema/template imports observed).
- [ ] Naming taxonomy? **Needs audit (are_many/get_ etc.).**
- [ ] Zero-copy access points (`with_archived`) identified? **Needs audit.**

### 3.2 Zero-Copy Access Points (Current)

- `db_query::QueryAdapter::with_archived_by_id` provides closure-based archived access for `StoredNote`.

### 3.1 Raw vs Domain Boundary Violations (Current)

- `note/reader/*` builds **domain types directly** (`Task`, `Link`, `Heading`, `Tag`, `Section`) instead of Raw*.
- `note/task.rs` contains **parsing logic** (inline fields, emojis, task promotion) that belongs in Raw extraction.
- `note/link.rs` contains **frontmatter parsing helpers** and target splitting that should be Raw parsing helpers.
- `note/tag.rs` includes **scan_tags** (parsing) which should be Raw helper, not Domain.
- `note/frontmatter.rs` performs YAML/TOML parsing inside Domain; this should be Raw parsing stage.
- `ParsedNote` mixes raw source + timestamps + domain instances; should become RawNote + Raw* components.

---

## 4) Refactor Targets and Removal Candidates (Initial)

- [ ] Introduce RawNote layer (file → Raw → Domain) with parsing helpers.
- [ ] Replace `note/reader/` with `note/parser/` (ingestion) and `note/raw/` (AST output).
- [ ] Add `aggregate.rs` and move identity into it.
- [ ] Move parsing helpers out of domain types into `parser/` + `raw/`.
- [ ] Replace CQRS ports with unified `Repository` in `note/storage.rs`.
- [ ] Remove `stored.rs`; persist domain types directly (per Raw → Domain → Archived rules).
- [ ] Only add `views/` if archived domain shape is inefficient for queries.
- [ ] Ensure storage consumes domain facts only (no parsing inside db_command).

---

## 5) Proposed Module Structure (Target State) (Draft)

```
note/
├── mod.rs
├── aggregate.rs         # Note aggregate + NoteId
├── paths.rs
├── position.rs
├── parser/              # Ingestion + pulldown-cmark event stream
│   ├── mod.rs
│   ├── parser.rs
│   ├── ast.rs
│   └── frontmatter.rs    # metadata block detection + raw capture only
├── raw/                 # AST output + raw extraction (ex-normalize)
│   ├── mod.rs
│   ├── note.rs          # RawNote (content + timestamps + hash)
│   ├── headings.rs
│   ├── tasks.rs
│   ├── links.rs
│   ├── tags.rs
│   ├── sections.rs
│   ├── list_items.rs
│   ├── block_refs.rs
│   └── task_tokens.rs   # Task metadata tokens (emoji + inline)
├── views/               # Lithos projections (not API/plugin replicas)
│   ├── mod.rs
│   ├── note.rs
│   ├── tasks.rs
│   └── links.rs
├── storage.rs           # Repository trait + concrete repository
├── loader.rs            # Orchestration
├── error.rs
├── events.rs
├── heading.rs
├── task.rs
├── link.rs
├── list.rs
├── tag.rs
├── structure.rs
├── value.rs
└── frontmatter.rs
```

**Note:** Domain types remain top-level for now; a future `note/domain/` split is optional and should not block the Raw/Domain boundary work.
**Note:** `views/` is optional and only created if archived domain shape proves inefficient for queries.

### 5.1 Target Module Tree Status (Gaps)

- **To add:** `aggregate.rs`, `parser/ast.rs`, `parser/parser.rs`, `raw/*`, `views/*`, `storage.rs`
- **To move:** `identity.rs` → `aggregate.rs`; parsing helpers → `parser/` + `raw/`
- **To remove:** `ports.rs`, `db_command.rs`, `db_query.rs`, `reader/`, `stored.rs`
- **To verify:** `loader.rs` ownership of orchestration only (no parsing or projection)
- **To confirm:** `frontmatter.rs` stays domain-only; parsing helpers move to `raw/`
- **To confirm:** `value.rs` stays domain-only; parsing helpers move to `raw/`

---

## 6) Target Pipeline Design (Target State) (Draft)

- Canonical pipeline: File → Raw → Domain → Storage
- Parsing/validation boundary:
  1) `FsReader` reads note file + timestamps
  2) `parser/` ingests file and emits AST (`parser/ast.rs`) + raw frontmatter block
  3) `raw/` extracts raw facts from AST (headings, tasks, links, tags, sections, list items, block refs)
  4) `TryFrom<Raw*>` parses semantics and produces domain (`Note`, `Task`, `Link`, `Tag`, `Frontmatter`, etc.)
  5) Repository persists domain + Lithos views

- Staleness pipeline:
  1) Timestamp fast path
  2) If modified: compare raw note hash vs last RawNoteView
  3) Rebuild only if hash changed

**MVP constraint:** Inline field parsing outside tasks is **TBD**; do not assume full Dataview parity.

### 6.1 Pipeline Inputs/Outputs (Concrete)

- **Input**: file bytes + file stats (ctime/mtime/size)
- **Parser output**: AST + raw frontmatter block (metadata block) + byte offsets
- **Raw extraction output**: Raw headings/links/tags/sections/list items/block refs (tasks derived from list items)
- **Domain output**: validated domain types (`Note`, `Task`, `Link`, `Tag`, `Section`, `Frontmatter`)
- **View output**: Lithos query views (note/task/link indexes)

### 6.2 Frontmatter Handling

- Use metadata block events when available.
- Fallback pre-scan only if metadata blocks are absent.
- Raw frontmatter stores raw block text + format.
- Parse to `FieldValue` in `TryFrom<RawFrontmatter>` (no parsing helpers in domain files).
- `parser/frontmatter.rs` only detects/captures the raw block; parsing happens in `raw/` and `TryFrom`.

### 6.3 Task Parsing Semantics

- Raw task token parsing uses config (`use_emoji`, status types, date fields).
- Status symbol → `StatusType` mapping occurs inside Raw extraction (pre-TryFrom).
- Inline task fields (Dataview format) parsed only for tasks in MVP.

### 6.4 Raw Timestamps

- `Raw*` types include `created_at` / `modified_at` from ingestion.
- Domain conversion does **not** carry these fields unless explicitly needed.
- Persistence uses raw views for staleness; domain remains timestamp-free.

### 6.5 Raw / Domain / View Boundaries (Clarified)

- **Raw**: derived from AST and frontmatter; contains offsets, tokens, and timestamps for staleness.
- **Domain**: validated facts only (e.g., `Task`, `Tag`, `Link`); no parser/AST types, no timestamps by default.
- **Views**: Lithos projections for query needs only; avoid Dataview/Tasks/Obsidian shapes.

### 6.6 Raw vs Domain Boundary Matrix (Explicit)

**Raw layer (from parser + extraction):**
- `RawNote`: file path, content hash, byte length, created_at/modified_at, raw frontmatter block (format + text), and arrays of raw components.
- `RawFrontmatter`: raw block text + format; parse errors retained for diagnostics.
- `RawHeading`: level (u8), raw text (boxed), byte range, section id reference (if needed).
- `RawSection`: kind, byte range, optional heading reference id, nesting depth.
- `RawList`/`RawListItem`: list type, depth, raw text, checkbox marker (bool), status symbol (char), byte offset.
- `RawTask`: derived from `RawListItem` with checkbox marker; status symbol (char), raw text, raw tags, raw inline fields (key/value strings), raw emoji dates, byte offset.
- `RawLink`: style, is_embed, target raw string, alias raw string, anchor raw string (if any), byte offset.
- `RawTag`: raw token string, byte offset.
- `RawBlockRef`: raw id string, byte offset.

**Domain layer (TryFrom<Raw*>):**
- `Note`: validated aggregate composed of validated domain types; no raw strings, no offsets except where the domain explicitly stores positions (e.g., `SourceByteOffset`).
- `Heading`: validated level + normalized heading text; no parser state.
- `Section`: validated kind + byte range + optional heading reference (domain fact; not a view).
- `List`/`ListItem`: validated depth/type, trimmed text; status symbol normalized via config.
- `Task`: validated status name, cleaned text, validated metadata fields, schedule timestamps; config decides promotion.
- `Link`: validated target (external/unresolved/resolved), anchor parsed to `Anchor`, alias optional.
- `Tag`: validated segments only; raw scanning lives in Raw.
- `Frontmatter`: parsed into `FieldValue` map (no config interpretation at Raw level).

**Boundary rules (must hold):**
- No `pulldown_cmark::*` types outside `note/parser/*`.
- No `Config` dependency inside domain types; only in Raw parsing or TryFrom stage.
- Raw types may include parsing helpers, but must not enforce domain invariants.
- Domain types must not parse raw strings; they only validate and store.
- AST types live in `note/parser/ast.rs` and are consumed by `note/raw/` only.
- Raw types have no behavior beyond parsing helpers (per Raw → Domain rules).

---

## 6.7 Gap Analysis Summary (Concise)

- **Boundary violations:** parsing logic lives in domain types (`task`, `link`, `tag`, `frontmatter`, `list`); reader builds domain directly.
- **Missing layers:** no Raw* types; no parser/AST boundary module; no aggregate.rs.
- **Storage shape:** Stored* projections are primary storage; conflicts with Raw → Domain → Archived rule.
- **CQRS split:** ports + db_* reinforce command/query separation against unified Repository rule.
- **Orchestration:** loader is not the sole pipeline orchestrator; reader + db_command coordinate logic.
- **View strategy:** Stored* types act as views without explicit view policy or profiling trigger.

---

## 7) Migration Plan (Ordered Steps) (Draft)

1) **Create `aggregate.rs`**
   - Move `NoteId` from `identity.rs` into `aggregate.rs`
   - Define `Note` aggregate or `NoteFacts` domain boundary

2) **Introduce parser + raw extraction**
   - Add `parser/ast.rs` + `parser/parser.rs` (pulldown-cmark event boundary)
   - Add `raw/` modules for extraction (headings/tasks/links/tags/etc.)

3) **Replace reader module**
   - Remove `note/reader/` after Raw pipeline is complete
   - Move parsing helpers out of domain files into `parser/` + `raw/`

4) **Replace CQRS with Repository**
   - Remove `ports.rs`, `db_command.rs`, `db_query.rs` wrappers
   - Create `storage.rs` with unified `Repository` and Redb implementation

5) **Wire loader to new pipeline**
   - `loader.rs` orchestrates file → Raw → Domain → Storage
   - Staleness checks use RawNoteView

6) **Update views**
    - Add Lithos projections only if archived domain shape is inefficient for queries
    - Ensure projections are built from domain facts (not parser)

**Temporary shims/adapters:** Not defined yet (needed if old CQRS APIs must coexist during refactor).

---

## 7.1 File-by-File Move Plan (Component-Level)

**Legend:** keep = stays in place; move = relocate; split = extract pieces; delete = remove.

- `mod.rs` — keep tables temporarily; update exports to new `storage`, `raw`, `parser`; remove CQRS references.
- `loader.rs` — keep file; move orchestration logic to File → Raw → Domain → Storage; remove direct `NoteReader` + `CommandAdapter` dependencies.
- `ports.rs` — delete; replaced by unified `storage::Repository`.
- `db_command.rs` — split:
  - move indexing + persistence into `storage.rs` (Repository impl).
  - move staleness/event logic into `storage.rs` or `loader.rs` (or new `storage/events.rs` if needed).
  - delete CQRS adapter wrapper.
- `db_query.rs` — split:
  - move read methods + `with_archived` into `storage.rs` (Repository impl).
  - remove Stored* projections usage (domain storage shape).
- `stored.rs` — delete; domain types become storage shape; reintroduce `views/` only if profiling proves necessary.
- `reader/mod.rs` — split:
  - move pulldown-cmark boundary to `parser/parser.rs` (event handling).
  - move AST node types to `parser/ast.rs` (minimal structure + byte ranges).
  - move ParsedNote → `raw/note.rs` (RawNote) with timestamps + hash.
  - remove domain construction from reader.
- `reader/sections.rs` — split:
  - move metadata block capture to `parser/frontmatter.rs`.
  - move block ref scan + section extraction to `raw/sections.rs` and `raw/block_refs.rs`.
- `reader/lists.rs` — split:
  - move list item capture to `raw/list_items.rs`.
  - move task promotion + inline parsing to `raw/tasks.rs` + `raw/task_tokens.rs`.
- `reader/links.rs` — split:
  - move link tokenization to `raw/links.rs` (raw target/alias/anchor).
  - keep pulldown-cmark LinkType mapping in `parser/`.
- `reader/tags.rs` — split:
  - move tag scanning into `raw/tags.rs` (raw tag tokens + offsets).
- `reader/frontmatter.rs` — split:
  - move frontmatter tag/link extraction into `raw/frontmatter.rs` (RawFrontmatter parsing helpers).
- `reader/state.rs` — delete; replace with raw list item records.
- `frontmatter.rs` — split:
  - keep `Frontmatter` domain type + accessors.
  - move `Frontmatter::parse` and YAML/TOML sanitization into `raw/frontmatter.rs`.
- `value.rs` — keep domain value type; move any parsing helpers used only for ingestion into `raw/` (if needed).
- `task.rs` — split:
  - keep domain types `Task`, `TaskMetadata`, `TaskSchedule`, `TaskFieldKey`.
  - move `TaskBuilder`, inline field parsing, emoji parsing to `raw/tasks.rs` + `raw/task_tokens.rs`.
- `tag.rs` — split:
  - keep `Tag` domain type.
  - move `scan_tags` into `raw/tags.rs`.
- `link.rs` — split:
  - keep domain `Link`, `Target`, `Anchor`, `EmbedType`.
  - move `LinkBuilder`, `parse_frontmatter_link`, and target splitting into `raw/links.rs`.
- `list.rs` — split:
  - keep domain `List`, `ListItem`, `ListDepth`.
  - move `ListItemBuilder`, `InlineText`, `ListItemEntry` into `raw/list_items.rs`.
- `heading.rs` — split:
  - keep domain `Heading`, `HeadingLevel`.
  - move `HeadingBuilder` into `raw/headings.rs`.
- `structure.rs` — keep domain `Section`, `BlockRef` (domain facts); move any parsing helpers into `raw/sections.rs` and `raw/block_refs.rs`.
- `paths.rs`, `position.rs`, `identity.rs`, `events.rs`, `error.rs` — keep as domain/infrastructure types; update errors to align with new Raw/Domain boundary (ingest errors move to loader).

---

## 7.2 Stepwise Checklist (Per-File Sequencing)

**Phase 1: Parser boundary (safe extraction)**
- `reader/mod.rs` → extract AST + parser loop to `parser/parser.rs` (keep old reader wired).
- `reader/links.rs` → move LinkType mapping to `parser/` (keep LinkBuilder calls intact).
- `reader/sections.rs` → move metadata block capture to `parser/frontmatter.rs`.

**Phase 2: Raw extraction (parallel to old reader)**
- `heading.rs` → move `HeadingBuilder` into `raw/headings.rs`; keep domain `Heading`.
- `list.rs` → move `ListItemBuilder`, `InlineText`, `ListItemEntry` into `raw/list_items.rs`; keep domain list types.
- `task.rs` → move `TaskBuilder` + inline parsing into `raw/tasks.rs` and `raw/task_tokens.rs`; keep domain Task types.
- `tag.rs` → move `scan_tags` into `raw/tags.rs`; keep domain Tag.
- `link.rs` → move `LinkBuilder`, `parse_frontmatter_link`, `split_target_and_anchor` into `raw/links.rs`; keep domain Link/Target/Anchor.
- `reader/tags.rs`, `reader/lists.rs`, `reader/frontmatter.rs`, `reader/links.rs` → rewire to use `raw/*` helpers (no domain creation).

**Phase 3: Raw → Domain boundary**
- `frontmatter.rs` → move `Frontmatter::parse` + YAML/TOML sanitize into `raw/frontmatter.rs`; keep domain accessors.
- `value.rs` → keep domain value type; any ingestion-only parsing helpers move into `raw/`.
- `reader/mod.rs` → replace `ParsedNote` with `RawNote` in `raw/note.rs`.

**Phase 4: Storage shape + orchestration**
- `ports.rs` → delete; add unified `storage::Repository`.
- `db_command.rs` + `db_query.rs` → fold into `storage.rs` (domain storage shape, with_archived).
- `stored.rs` → delete; replace Stored* usage with domain types and optional views.
- `loader.rs` → rewire to File → Raw → Domain → Storage pipeline.

**Phase 5: Cleanup and enforcement**
- `mod.rs` → update exports and remove CQRS references.
- `reader/*` → delete after `parser/` + `raw/` are fully wired.
- `events.rs` → keep; ensure event payloads use Raw timestamps and domain facts.

---

## 7.3 Split Risks + Mitigations

- `reader/mod.rs` split — risk: behavior drift in event handling; mitigation: snapshot tests on event→AST and Raw extraction outputs.
- `reader/sections.rs` split — risk: frontmatter or section ranges change; mitigation: tests for metadata block capture + section range boundaries.
- `reader/lists.rs` split — risk: list depth/parenting regressions; mitigation: nesting tests + list item parent assertions.
- `reader/links.rs` split — risk: wikilink alias parsing breaks; mitigation: alias/anchor tests for wiki + markdown links.
- `reader/tags.rs` split — risk: tag collection in code/link blocks; mitigation: tests for tag suppression inside code/links.
- `reader/frontmatter.rs` split — risk: frontmatter tag/link extraction changes; mitigation: frontmatter link/tag tests with nested objects.
- `frontmatter.rs` split — risk: YAML/TOML parsing changes; mitigation: frontmatter parsing tests with obsidian link sanitization.
- `task.rs` split — risk: inline field parsing changes; mitigation: task metadata + emoji date tests.
- `link.rs` split — risk: anchor/target parsing behavior changes; mitigation: anchor/target unit tests + external URL handling.
- `list.rs` split — risk: list item text normalization; mitigation: list item text tests + link text inclusion.
- `heading.rs` split — risk: heading text extraction changes; mitigation: heading text/level tests + link text inclusion.
- `stored.rs` removal — risk: query regressions; mitigation: repository query tests + with_archived coverage.

---

## 7.4 Stop/Go Criteria (Per Phase)

- **Phase 1 (Parser boundary):** Stop if AST output diverges from current reader tests; Go when parser tests + existing reader tests pass unchanged.
- **Phase 2 (Raw extraction):** Stop if Raw outputs differ for list depth, tags, links, or headings; Go when raw extraction tests match baseline fixtures.
- **Phase 3 (Raw → Domain):** Stop if TryFrom introduces new parse failures for valid inputs; Go when domain validation tests + frontmatter parsing tests pass.
- **Phase 4 (Storage + orchestration):** Stop if query results differ from stored projections; Go when repository query tests + indexing tests are green.
- **Phase 5 (Cleanup):** Stop if any adapter still depends on reader/CQRS; Go when builds pass with old modules removed.

---

## 7.5 Risk + Mitigation Summary (Per Phase)

| Phase | Primary Risk | Mitigation | Evidence to Proceed |
| --- | --- | --- | --- |
| 1 | Parser boundary drift changes event interpretation | Snapshot tests for event → AST; keep reader tests unchanged | Parser tests + existing reader tests green |
| 2 | Raw extraction diverges from current projections | Raw extraction fixtures for headings/links/tags/lists/tasks | Raw fixtures match baseline |
| 3 | TryFrom rejects valid inputs | Domain validation tests + frontmatter parsing tests | No new parse failures on valid fixtures |
| 4 | Storage/query regressions | Repository tests + indexing tests + with_archived coverage | Query results match previous behavior |
| 5 | Legacy modules still referenced | Build + lint with old modules removed | No references to reader/CQRS modules |

---

## 7.6 Phase-by-Phase PR Checklist

**PR 1: Parser boundary**
- [ ] Add `note/parser/ast.rs` + `note/parser/parser.rs` (no domain types)
- [ ] Keep existing reader wired; add parser snapshot tests
- [ ] No behavior changes in current reader tests

**PR 2: Raw extraction scaffolding**
- [ ] Add `note/raw/*` modules with Raw types only
- [ ] Add Raw extraction tests for headings/links/tags/lists
- [ ] Reader still passes all tests

**PR 3: Raw helpers + builder moves**
- [ ] Move `HeadingBuilder`, `ListItemBuilder`, `LinkBuilder`, `TaskBuilder` into `raw/`
- [ ] Update reader helpers to use raw helpers (no domain creation)
- [ ] Add/adjust tests for task promotion, tag scanning, link aliases

**PR 4: Frontmatter boundary**
- [ ] Move frontmatter parsing to `raw/frontmatter.rs`
- [ ] Keep domain `Frontmatter` accessors only
- [ ] Add frontmatter parsing + link extraction tests

**PR 5: Raw → Domain conversion**
- [ ] Introduce `RawNote` and `TryFrom<Raw*>` for domain types
- [ ] Replace `ParsedNote` usage with Raw → Domain in pipeline
- [ ] No new parse failures on valid fixtures

**PR 6: Repository unification**
- [ ] Create `note/storage.rs` with unified Repository trait
- [ ] Fold `db_command.rs` + `db_query.rs` into storage
- [ ] Add repository tests + with_archived coverage

**PR 7: Storage shape cleanup**
- [ ] Remove `stored.rs` and replace Stored* usage
- [ ] Add views only if profiling proves necessary
- [ ] Query behavior matches previous output

**PR 8: Loader rewiring + cleanup**
- [ ] Loader orchestrates File → Raw → Domain → Storage
- [ ] Remove `reader/` and `ports.rs` modules
- [ ] All tests + verify tasks green

---

## 8) Test and Verification Plan (Draft)

### Parsing Boundary Tests
- [ ] pulldown-cmark event stream → AST (list nesting, tasks, headings)
- [ ] frontmatter metadata block parsing with fallback

### Raw Extraction Tests
- [ ] headings/sections/links/tags extraction from AST
- [ ] task metadata token parsing (emoji + inline fields in tasks)
- [ ] raw offsets and ranges preserved for downstream reference

### Domain Validation Tests
- [ ] Task status/priority/recurrence invariants (as configured)
- [ ] Tag normalization rules
- [ ] Link targets and block refs parsed to validated types

### Pipeline Tests
- [ ] File change triggers re-parse; touch-only does not
- [ ] Views built from domain facts (no parser leakage)
- [ ] Raw timestamp staleness fast-path honors mtime and content hash

### Repository Tests
- [ ] Unified Repository trait behavior with fake/in-memory backends

### Verification Tasks
- [ ] `mise run fmt`
- [ ] `mise run lint`
- [ ] `mise run test`

---

## 9) Output Deliverables (Pending)

- Refined note refactor audit reflecting raw/parser split and aggregate placement
- Explicit module move/add/remove list aligned with target module tree
- Test plan covering parser boundary, raw extraction, domain validation, and staleness
- Target module tree diagram (from Section 5)
- Gap analysis summary (current vs target)
- Ordered refactor steps with risks/mitigations

---

## 10) Markdown Module Split (Decision Pending)

**Decision:** Defer. The note module should keep a strict `parser/` boundary now, with an explicit path to extract a `markdown/` module later if needed.

### Pros
- isolates pulldown‑cmark churn and options in one place
- enforces “no parser types leak into domain” across contexts
- enables sharing AST + parser if template or other contexts need Markdown
- simplifies test harnesses for parsing and offsets

### Cons
- premature abstraction if only note uses Markdown
- risk of generic AST that fits no context well or grows too broad
- encourages shared extractors that re‑couple contexts
- added module boundary slows iteration while architecture is still settling

### Guardrails if we split
- `markdown/` contains only parser + minimal AST + offsets; **no** context extraction logic
- context modules own Raw extraction and Domain conversion
- AST is intentionally small and structural (headings, paragraphs, lists, tasks, links, block quotes, code blocks)

### Decision Trigger
- adopt a `markdown/` module once at least one other context (template or future) needs Markdown parsing **and** the note parser boundary is stable

---

## 11) Custom Extractors (Plan)

**Principle:** Keep extractors small, composable, and context‑owned. The markdown parser produces a minimal AST; extractors derive Raw* data per context.

### Baseline Extractors (Note)
- **Frontmatter extractor:** captures raw metadata block text + format (YAML/TOML)
- **Task extractor:** captures checkbox marker, raw text, inline metadata, emoji dates
- **Link extractor:** captures wiki links, embeds, block refs, anchors with byte ranges
- **Heading/Section extractor:** captures outline and section ranges
- **Tag extractor:** captures raw tag tokens with byte ranges
- **Inline field extractor (non-task):** deferred (MVP constraint)

### Rules
- Extractors operate on AST, not pulldown-cmark events, to keep parser churn isolated
- Extractors must not depend on Domain types; produce Raw* only
- Use `TextMergeWithOffset` to keep byte offsets aligned with source
- Avoid monolithic “mega extractor”; each extractor focuses on one concern
- Config usage allowed only in extraction/Raw parsing, not in Domain structs

## 12) Context-Specific Notes (Initial)

- Notes are read-heavy and require zero-copy reads for indexes.
- pulldown-cmark event stream should be consumed once (AST as canonical parse output).
- Dataview/Tasks used as reference only; do not mirror their view shapes directly.
- Performance hot paths: task + link queries (confirm via profiling before adding views)

### pulldown-cmark Lessons (Basalt + Obsidian-Export digests)

- **Do not assume single-event wikilinks:** Obsidian-style links can be split across multiple events; use a state machine when extracting `[[...]]` or `![[...]]` (obsidian-export).
- **Event stream is the source of truth:** consume once, build a minimal AST with byte offsets; avoid re-parsing strings later (Basalt + current reader).
- **Offsets are byte-based:** use `TextMergeWithOffset` and preserve byte offsets; avoid char-index math (Basalt cursor/offset lessons).
- **Frontmatter parsing needs sanitization:** unquoted `[[links]]` in YAML can break parsing; sanitize or parse as raw text first (obsidian-export + current frontmatter sanitizer).
- **Parser options can change events across versions:** treat pulldown-cmark upgrades as breaking for event patterns; isolate in `parser/` to contain churn (obsidian-export changelog).
- **Unicode normalization:** Obsidian-style link resolution benefits from NFC normalization on titles/targets (obsidian-export).
- **Avoid “smart punctuation” transformations:** they alter byte lengths and break offsets; if needed, treat as render-only (Basalt).
- **Explicit nesting tracking:** list/task nesting should be tracked with a depth stack; avoid implicit nesting (Basalt list fixes).

### Applied Constraints (Non-CQRS)

- Treat event stream as canonical; build minimal AST once.
- Preserve list nesting and task checkbox symbols at raw extraction stage.
- Keep offsets/byte ranges as canonical; compute line/column lazily.
- Raw extraction is the semantic parsing boundary (no parsing in storage).

---

## Boundary Checklist (Final Pass)

- Parser is the only place `pulldown_cmark::*` appears
- AST is minimal and structural; no domain facts embedded
- Raw extraction is the only place parsing helpers live
- Domain types validate only; they never parse raw strings
- Config usage restricted to Raw parsing / TryFrom; domain is config-free
- Storage consumes domain + views only; no parsing inside storage
