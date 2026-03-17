# Note Parser Redesign Findings and Plan

## Purpose
This document captures the research findings and a complete redesign plan for
Lithos note parsing, based on pulldown-cmark developer documentation and the
Basalt project. It also specifies the new parsing and projection architecture
and outlines the implementation sequence.

## Scope
Focuses on the `lithos-core/src/note` parsing pipeline and storage projection
generation. This plan keeps the note context isolated and aligns with the
CQRS + projection architecture of Lithos.

## Research Findings

### pulldown-cmark Developer Guide Highlights
Sources: https://pulldown-cmark.github.io/pulldown-cmark/dev/index.html and
subpages (block parsing, inline processing, string handling, performance).

- pulldown-cmark uses a two-pass model:
  - Block structure parsing builds a block tree.
  - Inline processing happens on demand during event iteration.
- The parser exposes a pull-based event stream; the event stream is the
  canonical source. Consumers are expected to transform or extract from it.
- String handling uses `CowStr` for zero-copy or inlined strings.
- Performance relies on streaming and linear passes, not repeated scanning.

Implication for Lithos: our parser should consume the event stream once and
produce a single, minimal AST. All projections should be derived from that AST
to avoid repeated parsing and helper duplication.

### Basalt Markdown Parser Approach
Source: https://github.com/erikjuhani/basalt/blob/main/basalt-core/src/markdown.rs

Key characteristics:
- Single AST with `Node` and `MarkdownNode` variants.
- Text is centralized via `Text` and `TextNode`.
- Minimal node types and a simple parse loop.
- No persistence or indexing in the parser.

Implication for Lithos: match the simplicity by generating a single AST, then
derive all note-specific projections separately.

## CQRS Pipeline Lessons Applied
This section maps the pipeline stages you provided to Lithos, highlights
current violations, and defines how the redesign fixes them.

### Pipeline Mapping (Target State)
- Ingestion: fs reader discovers note, assigns note id, captures metadata
  (timestamps, size, hash), and stores raw text (or reference).
- Parsing: pulldown-cmark parses markdown into AST + frontmatter only.
- Validation/Normalization: normalizers validate domain constraints and
  canonicalize values (tags, task dates, link targets).
- Command creation: normalized facts (`Note`) become a command payload
  for persistence.
- Command handling: command adapter enforces invariants and persists write
  model (StoredNote) plus indexes.
- Write model update: StoredNote stored as authoritative snapshot.
- Event publication: Note events emitted as metadata and stored (optional).
- Read model projection: indexes and query tables derived from StoredNote.
- Query access: db_query reads from read-model tables only.

### Current Violations (Bloat Drivers)
- Parsing and projection logic live in `note/reader/*` and are used directly
  by storage adapters. This collapses parsing + validation + command creation.
- `db_command.rs` accepts ParsedNote (parser output) and performs projection
  responsibilities, mixing parsing and persistence concerns.
- Domain types (`task.rs`, `link.rs`, `frontmatter.rs`) embed parsing helpers
  that belong in projections.

### Redesign Fix
- Parser emits only AST + frontmatter + timestamps.
- Normalizers perform all validation/normalization, producing domain types.
- Command adapter consumes `Note` only.
- Query adapters remain projection-only readers.

## Design Principles

1. Single canonical parse output: `ParsedMarkdown` containing AST + source +
   frontmatter + timestamps (frontmatter parsed from metadata events, not AST).
2. Normalization is derived from AST; no parsing inside storage adapters.
3. Minimal AST; parity is achieved in projections, not in AST complexity.
4. Byte offsets are canonical; line/column computed on demand using `LineIndex`.

## Current Module Inventory (Audit)
This inventory captures what each file currently contains, what to keep, and
where it should live in the redesign. The goal is to remove overlap and keep
each component focused on one responsibility.

### Core Orchestration
- `note/mod.rs`
  - Current: module registry + table definitions + ParsedNote alias.
  - Keep: module registry, table definitions.
  - Change: replace `ParsedNote` alias with `ParsedMarkdown` (parser output).

- `note/loader.rs`
  - Current: orchestration for parse + persist.
  - Keep: high-level orchestration.
  - Change: swap `NoteReader` -> `note/parser::parse_note` and
    projections pipeline.

### Parser (current)
- `note/reader/mod.rs` + `note/reader/*`
  - Current: pulldown-cmark parsing + helper parsing state + projections.
  - Problem: mixes parsing, projection, and validation.
  - Plan: replace with `note/parser/` and delete reader module after migration.

#### Reader Submodules (current)
- `note/reader/frontmatter.rs`
  - Current: frontmatter tag/link extraction logic (recursive FieldValue walk).
  - Issues: duplicates link parsing logic; belongs in projections.
  - Plan: move to `note/normalize/frontmatter.rs`.

- `note/reader/links.rs`
  - Current: link builder selection by pulldown-cmark LinkType.
  - Issues: parsing decisions embedded in reader.
  - Plan: move into parser or link normalization, depending on AST approach.

- `note/reader/lists.rs`
  - Current: checkbox promotion + list-item indexing helpers.
  - Issues: parsing + projection mixed.
  - Plan: move into `note/normalize/tasks.rs` and `list_items.rs`.

- `note/reader/sections.rs`
  - Current: section boundaries, block ref scan, frontmatter parse.
  - Issues: block-ref scan does manual text walk outside parser.
  - Plan: move into normalizers (sections/block_refs) and parser frontmatter.

- `note/reader/state.rs`
  - Current: parser-only state (ListItemRecord, builders).
  - Plan: replace with `note/parser/ast` and projection state helpers.

- `note/reader/tags.rs`
  - Current: inline tag collection.
  - Plan: move to `note/normalize/tags.rs`.

### Domain Types (keep, remove parsing)
- `note/heading.rs`
  - Current: heading model + HeadingBuilder.
  - Keep: Heading model + validation.
  - Change: remove HeadingBuilder; use AST Text and projection mapping.

- `note/task.rs`
  - Current: Task model + attributes + metadata parsing + schedule parsing.
  - Keep: Task model + validation.
  - Change: move parsing + promotion logic to `note/normalize/tasks.rs`.
  - Public types (current):
    - `Task`, `TaskId`, `TaskAttributes`, `TaskAttributesBuilder`
    - `TaskSchedule`, `TaskText`, `TaskTags`, `TaskTimestamp`
    - `TaskDateKind`, `TaskPriority`, `TaskFieldKey`, `TaskMetadata`
    - `TaskMetadataFields`
  - Bloat risk: parsing, config-driven promotion, and value conversion mixed
    with pure domain model.

- `note/link.rs`
  - Current: Link model + parsing helpers + builders + frontmatter link parse.
  - Keep: Link model.
  - Change: move parsing helpers to `note/normalize/links.rs`.
  - Public types (current):
    - `Link`, `FrontmatterLink`, `Target`, `Anchor`, `Style`, `EmbedType`,
      `LinkAlias`
  - Bloat risk: parsing + builder logic embedded with domain type.

- `note/list.rs`
  - Current: List model + ListItem + ListItemEntry + ListItemBuilder.
  - Keep: List, ListItem, ListItemEntry.
  - Change: parsing builders move to parser/projections.
  - Public types (current):
    - `List`, `ListDepth`, `ListItems`, `ListItem`, `ListItemEntry`, `ListType`
  - Bloat risk: builder/state types in domain module.

- `note/tag.rs`
  - Current: Tag model + scan_tags.
  - Keep: Tag model + validation.
  - Change: move `scan_tags` into `note/normalize/tags.rs`.
  - Public types (current):
    - `Tag`
  - Bloat risk: scanning logic mixed with model.

- `note/structure.rs`
  - Current: Section + BlockRef types.
  - Keep: domain types.
  - Change: move section/block-ref extraction to normalizers.
  - Public types (current):
    - `SectionKind`, `Section`, `BlockRef`, `BlockRefId`

- `note/value.rs`
  - Current: FieldValue + parsing and conversion helpers.
  - Keep: FieldValue model.
  - Change: move YAML/TOML conversion helpers into `note/frontmatter` and
    `note/task` projections where needed, or split into `note/value/` submodule.
  - Public types (current):
    - `FieldValue`, `FieldValueType`, `FieldObjectFields`, `FieldArrayItems`
    - `FieldValueError`, `FieldValueParseError`, `FieldValueYamlError`,
      `FieldValueConversionError`
  - Bloat risk: parsing/conversion inside value model file.

- `note/frontmatter.rs`
  - Current: Frontmatter model + parse + sanitize.
  - Keep: Frontmatter model + parse.
  - Change: parse invoked from parser metadata events; frontmatter normalization
    in `note/normalize/frontmatter.rs`.
  - Public types (current):
    - `Frontmatter`, `FrontmatterFormat`, `AliasValues`, `FrontmatterBuilder`
  - Bloat risk: parsing + sanitize + accessors + builder in one file.

- `note/position.rs`, `note/paths.rs`, `note/identity.rs`
  - Current: shared primitives and validations.
  - Keep: unchanged.
  - Bloat risk: low; shared primitives are cohesive.

- `note/error.rs`, `note/events.rs`
  - Current: error types + ingestion events.
  - Keep: unchanged, but update error sources to new parser/projection pipeline.
  - Bloat risk: low; ensure errors reference parser/projection layers correctly.

### CQRS and Storage
- `note/ports.rs`
  - Current: CQRS ports, uses `ParsedNote`.
  - Change: use `ParsedMarkdown` or `Note` depending on pipeline.

- `note/db_command.rs`
  - Current: storage + indexing + parsing concerns (uses ParsedNote).
  - Change: consume `Note` only; no parsing here.
  - Bloat risk: large file combining indexing, persistence, and parsing.

- `note/db_query.rs`
  - Current: read model queries.
  - Keep: unchanged (no parsing).

- `note/stored.rs`
  - Current: stored projections for queries.
  - Keep: unchanged, but constructed from `Note`.
  - Bloat risk: lower; mostly data containers.

## Target Module Layout (Post-Refactor)

### note/parser/
- `ast.rs`: minimal AST + Text/TextNode.
- `parser.rs`: pulldown-cmark event ingestion; no domain parsing.
- `frontmatter.rs`: parse metadata block events into `Frontmatter`.
- `mod.rs`: `ParsedMarkdown` + `parse_note` entry point.

### note/normalize/
- `mod.rs`: `Note` and constructor `from_parsed`.
- `headings.rs`: Heading normalization.
- `tasks.rs`: Task normalization + task metadata parsing.
- `links.rs`: Link normalization.
- `tags.rs`: Tag normalization.
- `sections.rs`: Section normalization.
- `list_items.rs`: List item hierarchy normalization.
- `frontmatter.rs`: frontmatter-derived tags/links.
- `block_refs.rs`: Block reference normalization.

### note/projections/
- `mod.rs`: read-model projection builders.
- `notes.rs`: build note query views.
- `tasks.rs`: build task query views.
- `tags.rs`: build tag index views.
- `links.rs`: build link index views.

## Module Layout

### New Note Parser Module
Note-specific parser, scoped to `note/parser/`.

```
note/parser/
  mod.rs        // parse_note entry point, ParsedMarkdown
  ast.rs        // Node, MarkdownNode, Text, TextNode, Style
  parser.rs     // pulldown-cmark event ingestion -> AST
  frontmatter.rs// frontmatter extraction from MetadataBlock events
```

### Normalization and Projection Modules
Domain types remain in `note/`. Normalization lives in `note/normalize/` and
read-model projections live in `note/projections/`.

```
note/normalize/
  mod.rs
  headings.rs
  tasks.rs
  links.rs
  tags.rs
  sections.rs
  list_items.rs
  frontmatter.rs
  block_refs.rs

note/projections/
  mod.rs
  notes.rs
  tasks.rs
  tags.rs
  links.rs
```

### Storage Consumption
`db_command` and `stored` use normalized facts only; projections build read
models after persistence.

## AST Shape (Minimal, Note-Specific)

### Required nodes
- Heading
- Paragraph
- ListItem (ordered/unordered + task status)
- CodeBlock (to avoid parsing inside)
- BlockQuote (optional but useful for sections/callouts)
- No MetadataBlock node (frontmatter is parsed from events and stored in
  ParsedMarkdown, but does not become an AST node)

### Text
- `Text(Vec<TextNode>)`
- `TextNode { content, style }`
- `Style` variants: Code, Emphasis, Strong, Strikethrough

### Byte Ranges
Each node stores `Range<usize>` from pulldown-cmark offsets.

## Frontmatter Strategy

Frontmatter is parsed from pulldown-cmark metadata events, not stored in the
AST. The parser captures `Tag::MetadataBlock` event payloads and parses them
using `Frontmatter::parse`. A pre-scan fallback can be kept only if required
for compatibility with disabled metadata blocks.

## ParsedMarkdown and NoteProjections

### ParsedMarkdown (parser output)
```
ParsedMarkdown {
  source: Box<str>,
  nodes: Vec<Node>,
  frontmatter: Option<Frontmatter>,
  created_at: Option<SystemTime>,
  modified_at: Option<SystemTime>,
}
```

### Note (normalization helper)
Purpose: compute validated domain facts once and pass them through command
handling. This is not a read projection.

```
Note {
  headings: Vec<Heading>,
  sections: Vec<Section>,
  links: Vec<Link>,
  frontmatter_links: Vec<FrontmatterLink>,
  tags: Vec<Tag>,
  list_items: Vec<ListItemEntry>,
  tasks: Vec<Task>,
  block_refs: Vec<BlockRef>,
}
```

Note is not StoredNote; it is the in-memory validated domain facts used to
construct StoredNote. Read projections are built from StoredNote or events.

## Detailed Implementation Plan

### Phase 1: AST + Parser
1. Create `note/parser/ast.rs` with Node, MarkdownNode, Text, TextNode, Style.
2. Create `note/parser/parser.rs` that consumes pulldown-cmark events:
   - Use `Parser::new_ext(...).into_offset_iter()` and `TextMergeWithOffset`.
   - Build nodes based on `Event::Start/End` and `TagEnd` mapping.
   - Fill `Text` by collecting `Event::Text` and `Event::Code`.
3. Create `note/parser/frontmatter.rs` to capture metadata blocks.
4. Create `note/parser/mod.rs` with `ParsedMarkdown` and `parse_note()`.

### Phase 2: First Projection (Headings)
1. Add `note/projections/headings.rs` to map Heading nodes to `Heading` domain type.
2. Validate parity with existing heading tests.

### Phase 3: List/Task Projection
1. `note/projections/list_items.rs` extracts list item positions, depth, and parents.
2. `note/projections/tasks.rs` handles checkbox promotion and task metadata parsing.

### Phase 4: Links, Tags, Sections, Block Refs
1. `note/projections/links.rs` extracts link targets from nodes and inline styles.
2. `note/projections/tags.rs` parses tag tokens from Text, respecting code spans.
3. `note/projections/sections.rs` uses Heading + BlockQuote + CodeBlock boundaries.
4. `note/projections/block_refs.rs` extracts `^block-id` from Text.

### Phase 5: Storage Integration
1. Replace `NoteReader` usage with `parse_note` -> `NoteProjections` pipeline.
2. Build `StoredNote` from projections only.
3. Update `db_command` indexing to consume `NoteProjections`.

### Phase 6: Remove Old Parsing Helpers
1. Delete parsing logic in `note/reader/` when fully migrated.
2. Remove legacy `ParsedNote` if no longer referenced.

## Risks and Mitigations

- Risk: loss of Obsidian parity from minimal AST.
  Mitigation: enforce parity at projection layer with targeted tests.

- Risk: projection performance due to multiple passes.
  Mitigation: create `NoteProjections::from(ParsedMarkdown)` to compute once.

- Risk: frontmatter parsing divergence.
  Mitigation: use pulldown-cmark metadata events when enabled, and keep
  a pre-scan fallback for strict compatibility.

## Component Responsibilities (Required)

### note/parser/
- `mod.rs`: public `parse_note` API, `ParsedMarkdown` definition, option wiring.
- `ast.rs`: AST types only (`Node`, `MarkdownNode`, `Text`, `TextNode`, `Style`).
- `parser.rs`: pulldown-cmark event ingestion; no domain parsing.
- `frontmatter.rs`: parse YAML/TOML from MetadataBlock events.

### note/projections/
- `mod.rs`: `NoteProjections` struct and `from_parsed` constructor.
- `headings.rs`: project `Heading` + heading locations.
- `tasks.rs`: project `Task` + metadata; handles checkbox promotion.
- `links.rs`: project `Link` and `FrontmatterLink` from inline events/text.
- `tags.rs`: project `Tag` from text while respecting code spans.
- `sections.rs`: project `Section` based on block boundaries.
- `list_items.rs`: project `ListItemEntry` and list hierarchy.
- `frontmatter.rs`: project frontmatter-derived tags/links if needed.
- `block_refs.rs`: project `BlockRef` from caret syntax in text.

### note/ domain types
- `heading.rs`, `task.rs`, `link.rs`, `list.rs`, `structure.rs`, `tag.rs` remain
  domain models with validation and accessors. Parsing moves out into projections.

### note/storage
- `stored.rs`: write-model snapshots built from `Note`.
- `db_command.rs`: persistence + index maintenance using Note only.
- `db_query.rs`: read/query path; consumes read projections.

## Open Decisions

1. Whether to capture block quotes as separate nodes or only for section
   boundaries.
2. Whether to include tables in AST (not needed unless we plan to extract
   table content).
3. Whether to store `Text` as `CowStr` in the AST or convert to owned String
   at AST creation.
## Detailed File Inventory (Public Types)
This section enumerates public types in the largest files to reduce blind
spots during refactor.

### task.rs
- `Task`, `TaskId`, `TaskAttributes`, `TaskAttributesBuilder`
- `TaskSchedule`, `TaskText`, `TaskTags`, `TaskTimestamp`
- `TaskDateKind`, `TaskPriority`, `TaskFieldKey`, `TaskMetadata`
- `TaskMetadataFields`

#### Critical Assessment (task.rs)
Keep (core domain types):
- `Task`, `TaskId`, `TaskText`, `TaskSchedule`, `TaskTimestamp`, `TaskDateKind`
- `TaskMetadata` (as a container type)

Candidates to move out or remove:
- `TaskAttributesBuilder`: move to projections (parser-only state).
- `TaskTags`: replace with standard iterator over `Task::tags()`.
- `TaskFieldKey`: move into projections or `task/metadata` submodule; may be
  unnecessary if metadata keys are strings.
- `TaskPriority`: keep only if used by query/index or config; otherwise merge
  into `TaskMetadata` as a plain number.

Parsing/validation to relocate:
- Inline metadata parsing (currently in task.rs) should move to
  `note/projections/tasks.rs`.

### value.rs
- `FieldValue`, `FieldValueType`
- `FieldObjectFields`, `FieldArrayItems`
- `FieldValueError`, `FieldValueParseError`, `FieldValueYamlError`,
  `FieldValueConversionError`

#### Critical Assessment (value.rs)
Keep:
- `FieldValue`, `FieldValueType` (domain representation)
- Iterators (`FieldObjectFields`, `FieldArrayItems`) if used externally

Move or delete:
- Parsing/conversion errors (`FieldValueParseError`, `FieldValueYamlError`,
  `FieldValueConversionError`) should move to `note/error.rs` or be scoped to
  `note/frontmatter` and `note/projections/tasks` if they are internal.
- `FieldValueError` should be folded into `NoteError::Metadata` or
  `FrontmatterParseError` to reduce error surface.

Parsing helpers to relocate:
- YAML/TOML conversion should live in `note/frontmatter` or in a new
  `note/value/convert.rs` if reused.

### frontmatter.rs
- `Frontmatter`, `FrontmatterFormat`
- `AliasValues`, `FrontmatterBuilder`

#### Critical Assessment (frontmatter.rs)
Keep:
- `Frontmatter`, `FrontmatterFormat` and minimal accessors.

Remove or move:
- `FrontmatterBuilder` should be moved to projections or removed; keep builder
  only if production code constructs frontmatter outside parsing.
- `AliasValues` should be removed if it is only a convenience iterator; keep
  only if used by query surfaces.

Parsing location:
- `Frontmatter::parse` should only be called from parser metadata events.

### link.rs
- `Link`, `FrontmatterLink`, `Target`, `Anchor`, `Style`, `EmbedType`
- `LinkAlias`

#### Critical Assessment (link.rs)
Keep:
- `Link`, `FrontmatterLink`, `Target`, `Anchor`, `Style`, `EmbedType`.

Remove or move:
- Parsing helpers and builder types should move to `note/projections/links.rs`.
- `LinkAlias` is only a thin wrapper; if not used outside link parsing, fold
  into `Link` as a `Box<str>` with validation in projection layer.

### list.rs
- `List`, `ListDepth`, `ListItems`, `ListItem`, `ListItemEntry`, `ListType`

#### Critical Assessment (list.rs)
Keep:
- `List`, `ListItem`, `ListItemEntry`, `ListType`, `ListDepth`.

Remove or move:
- Parser-only builders (currently `ListItemBuilder`) should live in parser or
  projection state, not in the domain module.

### stored.rs
- `StoredNote`, `StoredTask`, `StoredListItem`

#### Critical Assessment (stored.rs)
Keep:
- Stored projections and accessors.

Consider reducing:
- Fields not used by queries or indexes should be removed to reduce storage
  footprint (audit required: heading_locations, section_locations, list_items).

## Remove/Add Checklist (Explicit)

### Remove or Move (Bloat Reduction)
- Remove `note/reader/` after parser/projections migration.
- Move parsing helpers out of domain types:
  - `HeadingBuilder` (heading.rs)
  - `TaskAttributesBuilder`, task metadata parsing (task.rs)
  - Link parsing helpers (link.rs)
  - Tag scanning logic (tag.rs)
  - Frontmatter link/tag extraction (reader frontmatter helpers)
- Move parsing errors out of `value.rs` into `note/error.rs` or projection
  modules.

### Add (Minimal, Required)
- `note/parser/` module (AST + parser)
- `note/projections/` module (all projections)
- `NoteProjections` helper to compute projections once

## Notes on Bloat Control
- If a type is only used during parsing, it must live in parser/projections.
- If a type is only used for storage, it must live in stored/db modules.
- If a type is used only as a validation wrapper, it must stay in domain and
  avoid parsing logic.

## Domain Type Pruning (Required Rethink)
The current domain types mix validation and normalization responsibilities.
This redesign treats domain types as *pure validation + accessors* only. Any
type that is only needed during parsing/normalization must be removed or moved.

### Pruning Criteria
- Remove types that are only used for parsing or builder-style accumulation.
- Collapse thin wrapper types that do not enforce a distinct invariant.
- Move parsing helpers into `note/normalize/` or `note/parser/`.
- Keep only types that are referenced by storage, queries, or external API.

### Candidate Removals / Collapses
These candidates must be reviewed and removed or collapsed unless they enforce
unique, externally visible invariants:

- `TaskAttributesBuilder` → move to normalize (parser-only).
- `TaskTags` → replace with iterator returned by `Task::tags()`.
- `TaskFieldKey` → replace with `Box<str>` if no invariant enforced.
- `TaskPriority` → collapse into `TaskMetadata` if only a numeric wrapper.
- `HeadingBuilder` → remove; derive from AST text in normalize.
- `LinkAlias` → collapse into `Box<str>` if no invariant enforced.
- `FrontmatterBuilder` → remove unless used outside parsing.
- `AliasValues` → remove if only a convenience iterator.
- `FieldValue*Error` types → move to `note/error.rs` or normalize modules.

### Required Domain Type Boundaries
- Domain types must be construction/validation only (no parsing text).
- No domain type may depend on pulldown-cmark or parsing utilities.
- Domain types must not walk Markdown text or AST; that belongs in normalize.

## Adversarial Review (Weak Points + Mitigations)
This section challenges the redesign and lists failure modes that must be
closed before implementation. The refactor must not proceed until these are
resolved or explicitly accepted.

### Weak Point 1: Minimal AST losing semantic data
- Risk: omitting node types (tables, footnotes, HTML) could break parity if
  downstream features or future requests need them.
- Mitigation: define a strict parity matrix (features we support now) and
  enforce it with tests; expand AST only when a new projection needs it.

### Weak Point 2: Frontmatter via MetadataBlock events only
- Risk: metadata events can be disabled or missing in certain config paths.
- Mitigation: keep a documented fallback pre-scan that is exercised by tests
  and used only when metadata events are not produced.

### Weak Point 3: Task list ambiguity
- Risk: task semantics are derived from list items and text; parsing errors or
  style mismatches could silently lose tasks.
- Mitigation: define deterministic rules for promotion, ensure all unchecked
  task cases are covered by tests, and log structured errors on invalid
  symbols/metadata.

### Weak Point 4: Normalization vs projections confusion
- Risk: “normalize” becomes a dumping ground and duplicates read projection
  work.
- Mitigation: enforce rule that normalization emits *domain facts only*, never
  query-optimized shapes. Read projections must only consume StoredNote/events.

### Weak Point 5: StoredNote as write model vs projection
- Risk: StoredNote could become a dumping ground for every projection result.
- Mitigation: audit StoredNote fields and remove any not used by queries or
  invariants. Add a field-usage matrix before migration.

### Weak Point 6: Line/column storage
- Risk: storing line/column in StoredNote bloats storage and can go stale if
  byte offsets are canonical.
- Mitigation: store byte ranges only; compute line/column lazily via LineIndex.

### Weak Point 7: Event stream and ingestion keys
- Risk: pipeline stages are described but not implemented; ingestion tracking
  could be skipped and idempotency lost.
- Mitigation: explicitly decide whether ingestion keys/event keys exist. If not
  implemented now, remove from scope and document the omission.

### Weak Point 8: Backwards compatibility
- Risk: changing ParsedNote/ports/reader could break downstream crates.
- Mitigation: introduce transitional adapters or feature flags; update ports to
  accept ParsedMarkdown or Note with explicit migration plan.

### Weak Point 9: Performance regressions
- Risk: multiple projections could re-walk AST and re-scan text.
- Mitigation: ensure Note is built once; projections must use Note or
  StoredNote, not re-parse AST repeatedly.

### Weak Point 10: Test surface gaps
- Risk: refactor removes implicit coverage provided by reader tests.
- Mitigation: move tests into parser + normalizer + projections explicitly;
  enforce parity tests for Obsidian behaviors.
