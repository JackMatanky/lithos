# Note Parser + Scanner Refactor Plan (Comprehensive)

**Date:** 2026-04-09
**Scope:** `lithos-core/src/note/parser.rs`, `lithos-core/src/note/scanner.rs`, `lithos-core/src/note/raw/*`
**Constraints (User):** no new files yet; keep semantic parsing in `RawFrontmatter` and `RawFieldValue`; keep manual scanning of task markers.

**Goals**
- Make the parsing/ingestion pipeline cleaner, more modular, and easier to evolve without regressions.
- Preserve existing behavior and public APIs while reducing complexity and long parameter lists.
- Avoid god-objects by splitting state into focused sub-structures and rule handlers.

**Non-Goals**
- No new modules/files in this phase.
- No behavior change unless explicitly called out.
- No removal of `RawFrontmatter::parse_fields` or `RawFieldValue::from_str_with_spec`.

**Primary Inputs / Outputs**
- Input: markdown source `&str`, `NotePath`, `TaskConfigSpec`.
- Output: `RawNote` with `frontmatter`, `headings`, `sections`, `links`, `tags`, `lists`, `list_items`, `inline_fields`, `block_refs`.

**How to Use This Plan**
- This document is self-contained. Follow the ordered steps in "Implementation Sequence".
- The "Behavior Checklist" is the acceptance criteria. Do not change behavior unless explicitly called out.
- The "Mapping Table" is the authoritative guide to preserve behavior while moving code.
- All changes stay inside existing files only.

## Findings (Current Code Review)

**High-impact complexity drivers**
- `MarkdownParser::parse` orchestrates everything and feeds long-parameter helper methods.
- `handle_end_tag` and `finalize_list_item` depend on 10+ parameters each and blend multiple responsibilities.
- `scanner.rs` is a single, implicit rule set; ordering is hard-coded and scattered.

**Coupling hotspots**
- Parser and scanner overlap in task marker logic: pulldown-cmark `Event::TaskListMarker` marks state, then scanner rescans for marker char.
- Block ref extraction uses both scannable ranges and a tail fallback scan; behavior is distributed.
- Raw DTOs are mostly clean, but constructors are parameter-heavy (RawNote, RawListItem, RawLink).

**Behavior-sensitive areas**
- Task marker scanning must remain manual to preserve non-standard markers (`[!]`, `[-]`, etc.).
- Inline field typing must stay in Raw layer (per constraint).
- Tag and block ref boundaries are strict (word boundary and line tail).

## First-Principles Capability List (Must/Should/Optional)

**Must**
- Structural extraction with byte ranges: headings, paragraphs, lists, list items, block quotes, code blocks, frontmatter sections.
- Link extraction: wiki + markdown links with alias/anchor, embed flag, and position.
- Inline metadata: tags, inline fields, emoji fields, block refs, task marker symbols.
- Preserve zero-copy behavior and precise `SourceByteRange/Offset`.

**Should**
- Keep consistent link alias handling for `[[target|alias]]` and anchors `[[target#heading|alias]]`.
- Preserve non-standard task marker symbols and positions.

**Optional**
- Callouts, tables, footnotes, and HTML blocks as structured raw artifacts.

## pulldown-cmark Capability Notes (No new files; reference only)

- Metadata blocks: `Options::ENABLE_YAML_STYLE_METADATA_BLOCKS`, `Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS`, `Tag::MetadataBlock`.
  - https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html
  - https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.Tag.html
- Task lists: `Options::ENABLE_TASKLISTS`, `Event::TaskListMarker(bool)`.
  - https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.Event.html
- Wikilinks: `Options::ENABLE_WIKILINKS`, `Tag::Link` with `LinkType::WikiLink`.
  - https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.LinkType.html
- Offsets: `Parser::into_offset_iter()` yields `(Event, Range<usize>)`.
  - https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Parser.html
- Text merging: `TextMergeWithOffset` already in use and should remain.

## Refactor Plan (No New Files Yet)

### Phase A: Parser Internal Restructure (parser.rs)

1) **Introduce internal state structs** (all private, same file):
   - `ParserState` with composed sub-states to avoid a god-object.
   - `BlockState`, `ListState`, `LinkState`, `MetaState`, `OutputState`.

2) **Add handler types** (private, same file):
   - `MetadataHandler`, `StartTagHandler`, `EndTagHandler`, `TextHandler`.
   - Each handler takes only the sub-state it needs.

3) **Reduce signature size**
   - Use small param structs (private) such as `ScanContext`, `BlockFinalizeContext`, `ListFinalizeContext`.
   - Replace long parameter lists with those context structs.

4) **Keep behavior and ordering**
   - `MetadataHandler` runs before any tag/text handling.
   - `Event::TaskListMarker` continues to set `block.task_marker`.
   - `finalize_list_item` still calls `scan_task_marker` (manual scan retained).

### Phase B: Scanner Rule Registry (scanner.rs)

1) **Introduce rule traits (private, same file)**
   - `LineStartRule`, `BodyRule` with `try_scan`.

2) **Create rule implementations**
   - Line start: `TaskRule`, `BareFieldRule`.
   - Body: `TagRule`, `DelimitedFieldRule`, `BlockRefRule`, `EmojiFieldRule`.

3) **Replace hard-coded branching**
   - `handle_line_start` and `handle_body` become small loops over rule arrays.
   - Rule ordering is explicit and matches current semantics.

### Phase C: DTO Ergonomics (raw/*)

- Keep semantic parsing in `RawFrontmatter` and `RawFieldValue`.
- Reduce parameter burden where safe:
  - Prefer struct literals internally where it does not alter public API.
  - Keep current `new()` constructors to avoid API churn.

## Internal Type/Layout Sketches

### parser.rs (planned private types)

```rust
struct ParserState<'source> {
    block: BlockState<'source>,
    list: ListState,
    link: LinkState<'source>,
    meta: MetaState,
    out: OutputState<'source>,
    pool: StringPool,
    task_spec: &'source TaskConfigSpec,
}

struct BlockState<'source> {
    stack: Vec<ActiveBlock>,
    depth: u32,
}

struct ListState {
    stack: Vec<RawListKind>,
    contexts: Vec<ListContext>,
    open_item_by_depth: Vec<SourceByteOffset>,
}

struct LinkState<'source> {
    current: Option<LinkFrame<'source>>,
    ref_defs: ReferenceMap,
}

struct MetaState {
    in_metadata: Option<(pulldown_cmark::MetadataBlockKind, SourceByteOffset)>,
    buffer: String,
}

struct OutputState<'source> {
    headings: Vec<RawHeading<'source>>,
    sections: Vec<RawSection>,
    links: Vec<RawLink<'source>>,
    tags: Vec<RawTag<'source>>,
    lists: Vec<RawList>,
    list_items: Vec<RawListItem<'source>>,
    inline_fields: Vec<RawInlineField<'source>>,
    block_refs: Vec<RawBlockRef<'source>>,
    frontmatter: Option<RawFrontmatter<'source>>,
}
```

### parser.rs (handler layout)

```rust
struct MetadataHandler;
impl MetadataHandler {
    fn on_event<'s>(event: &Event<'s>, range: &Range<usize>, state: &mut ParserState<'s>)
        -> Result<bool, NoteIngestError>;
}

struct StartTagHandler;
impl StartTagHandler {
    fn on_start<'s>(tag: Tag<'s>, start: SourceByteOffset, state: &mut ParserState<'s>);
}

struct EndTagHandler;
impl EndTagHandler {
    fn on_end<'s>(tag: TagEnd, range: Range<usize>, markdown: &'s str, scanner: &NoteScanner,
        state: &mut ParserState<'s>) -> Result<(), NoteIngestError>;
}

struct TextHandler;
impl TextHandler {
    fn on_scannable_text<'s>(text: &CowStr<'s>, range: &Range<usize>, state: &mut ParserState<'s>);
    fn on_unscannable_text<'s>(text: &CowStr<'s>, range: &Range<usize>, state: &mut ParserState<'s>);
}
```

### scanner.rs (rule registry layout)

```rust
trait LineStartRule {
    fn try_scan<'s>(&self, cursor: &mut Cursor<'s>, out: &mut Vec<ScannedArtifact<'s>>)
        -> Result<bool, NoteError>;
}

trait BodyRule {
    fn try_scan<'s>(&self, scanner: &NoteScanner, cursor: &mut Cursor<'s>, out: &mut Vec<ScannedArtifact<'s>>)
        -> Result<bool, NoteError>;
}

struct TaskRule;
struct BareFieldRule;
struct TagRule;
struct DelimitedFieldRule;
struct BlockRefRule;
struct EmojiFieldRule;
```

## Explicit Behavior Mapping Table

| Current Behavior / Location | New Structure | Preservation Notes |
| --- | --- | --- |
| `MarkdownParser::parse` main event loop | same loop with `ParserState` + handlers | Event ordering stays identical; metadata handling remains first. |
| `handle_metadata` | `MetadataHandler::on_event` | Keeps same capture of metadata blocks and ranges. |
| `handle_start_tag` | `StartTagHandler::on_start` | Same block/list/link creation logic, just localized. |
| `handle_end_tag` | `EndTagHandler::on_end` | Same block finalize logic; delegates to `finalize_*` helpers with context structs. |
| `finalize_paragraph` | `ParagraphFinalizer::finalize` (private fn) | Same scanning and parent list item merge behavior. |
| `finalize_list_item` | `ListItemFinalizer::finalize` (private fn) | Keeps manual task marker scan and list metadata. |
| `scan_block_artifacts` | `ScanHandler::scan_block_artifacts` | Same scanner calls + block-ref tail scan logic. |
| `handle_scannable_text` | `TextHandler::on_scannable_text` | Same full text capture and scannable range tracking. |
| `handle_unscannable_text` | `TextHandler::on_unscannable_text` | Same behavior for code/inline code. |
| `NoteScanner::scan_cursor` | same public API, rule loop | Same state machine, now explicit rules. |
| `handle_line_start` | `LineStartRule` loop | Same order: task list, then bare field. |
| `handle_body` | `BodyRule` loop | Same order: tag, delimited field, block ref, emoji field. |
| `scan_task_marker` | retained | Manual scanning preserved. |

## Behavior Preservation Plan

**No API changes**
- `MarkdownParser::parse`, `NoteScanner::scan_block`, `scan_ranges`, `scan_task_marker` signatures remain.

**Test matrix (existing modules only)**
- Scanner tests cover tag boundary, emoji field spacing, bare fields only at line start, block refs at line tail, task marker extraction.
- Parser tests cover frontmatter capture, list nesting, link extraction, task marker + checkbox state, reference definitions, block refs in paragraph tail.

**Equivalence checks**
- Preserve sorting of sections by range start.
- Preserve task marker manual scan only when checkbox present.
- Preserve skip of inline scanning inside links and code.

## Risks and Mitigations

**Risk:** Sub-state split accidentally drops a behavior (e.g., block depth updates).
- Mitigation: explicit behavior mapping table + targeted tests.

**Risk:** Rule ordering changes behavior.
- Mitigation: maintain explicit order; add a rule order test.

**Risk:** Manual task marker scan shifts range semantics.
- Mitigation: keep identical range logic and cap behavior (first line only).

## Done Criteria

- All existing tests pass without edits.
- New targeted tests added in existing modules.
- No public API changes.
- No new files created.
- Manual task marker scanning preserved.

---

## Comprehensive Behavior Checklist (Acceptance Criteria)

**Scanner behavior**
- Tag detection: `#tag` requires non-alnum boundary before `#`; supports ASCII alnum + `_` `-` `/` and Unicode alnum in tags.
- Block ref detection: `^id` only when `^` is not preceded by alnum and only if the rest of the line contains whitespace only after the id.
- Inline fields:
  - Bracketed: `[key:: value]` and `(key:: value)` when both key and value are non-empty after trim.
  - Bare: `key:: value` only at line start (after whitespace).
  - Emoji: configured emoji key, then optional whitespace, then non-empty value until whitespace.
- Task marker detection: list prefix followed by `[x]` pattern; capture exact marker char and byte position.
- Cursor semantics: `prev_alnum` and `ScanMode` are preserved across continuous scanning within `scan_cursor` and `scan_ranges` behavior remains as-is.

**Parser behavior**
- Metadata blocks: start/end via `Tag::MetadataBlock`, range captured as `RawSectionKind::Frontmatter`, content preserved in `RawFrontmatter`.
- Headings: `RawHeading` with correct level and range; heading text trimmed as current.
- Sections: `RawSection` created for heading, paragraph, block quote, list, code block with correct depth.
- Links: `RawLink` contains style, embed, target, alias, anchor, position. Reference definitions resolved before event iteration.
- Lists:
  - List kinds (ordered/unordered) preserved.
  - Depth rules for `RawListDepth` preserved.
  - `RawListItem` text, range, text_range, parent, tags, inline fields preserved.
- Task markers: `Event::TaskListMarker` sets `is_checked`, manual scan extracts `RawTaskStatusSymbol` from first line range.
- Block refs:
  - Extracted from scanned ranges.
  - Tail fallback logic for block refs preserved.
- Inline field typing via `RawFieldValue::from_str_with_spec` remains in `parser.rs`.

**Output behavior**
- `RawNote::new` called with all vectors; `sections` sorted by range start.
- No changes to public signatures or return types.

---

## Implementation Sequence (Step-by-Step)

**Step 0: Pre-checks**
- Confirm no new files will be added.
- Ensure existing tests compile locally (do not run yet unless asked).

**Step 1: Add internal state structs to parser.rs**
- Add `ParserState`, `BlockState`, `ListState`, `LinkState`, `MetaState`, `OutputState` near existing helper structs.
- Move existing local variables into `OutputState` and `MetaState` fields in `parse`.

**Step 2: Introduce handler structs in parser.rs**
- Implement `MetadataHandler`, `StartTagHandler`, `EndTagHandler`, `TextHandler` with existing logic moved verbatim.
- `EndTagHandler` uses the same logic to finalize blocks; only signature changes to accept `ParserState`.

**Step 3: Replace long parameter lists with context structs**
- Create `ScanContext`, `BlockFinalizeContext`, `ListFinalizeContext`.
- Update `finalize_paragraph`, `finalize_list_item`, and `scan_block_artifacts` to accept these structs.

**Step 4: Wire the event loop**
- Replace direct calls to `handle_*` with handler invocations.
- Ensure `MetadataHandler` executes before other handlers.
- Preserve `Event::TaskListMarker` behavior.

**Step 5: Scanner rule registry in scanner.rs**
- Define `LineStartRule` and `BodyRule` traits.
- Add rule structs for task, bare fields, tags, delimited fields, block refs, emoji fields.
- Replace `handle_line_start` / `handle_body` bodies with ordered rule loops; logic stays the same.

**Step 6: DTO ergonomics (optional)**
- Only if needed, replace local `RawLink::new`/`RawListItem::new` calls with local builder helpers to reduce clippy noise.
- Keep public constructors intact.

**Step 7: Tests (same files only)**
- Add targeted unit tests inside existing `#[cfg(test)]` modules for each rule/handler.
- Verify no behavior change in existing tests.

---

## Detailed Behavior Mapping (Expanded)

| Area | Current Function/Block | New Location | Notes |
| --- | --- | --- | --- |
| Metadata capture | `handle_metadata` | `MetadataHandler::on_event` | Same buffer, same range mapping, same `RawFrontmatter` creation. |
| Block start | `handle_start_tag` | `StartTagHandler::on_start` | Same list/heading/link logic. |
| Block end | `handle_end_tag` | `EndTagHandler::on_end` | Same finalize logic; uses contexts. |
| Paragraph finalize | `finalize_paragraph` | `ParagraphFinalizer::finalize` (private fn) | Same scan + parent merge. |
| List finalize | `finalize_list` | `ListFinalizer::finalize` (private fn) | Same list depth logic. |
| List item finalize | `finalize_list_item` | `ListItemFinalizer::finalize` (private fn) | Same task scan and text range logic. |
| Text capture | `handle_scannable_text` | `TextHandler::on_scannable_text` | Same scannable range tracking. |
| Unscannable text | `handle_unscannable_text` | `TextHandler::on_unscannable_text` | Same behavior. |
| Scan artifacts | `scan_block_artifacts` | `ScanHandler::scan_block_artifacts` | Same scanner call + tail fallback. |
| Tag scan | `scan_tag` | `TagRule::try_scan` | Same boundary rules. |
| Delimited field scan | `scan_delimited_field` | `DelimitedFieldRule::try_scan` | Same parsing and range creation. |
| Emoji field scan | `scan_emoji_field` | `EmojiFieldRule::try_scan` | Same whitespace + value capture. |
| Bare field scan | `scan_bare_field` | `BareFieldRule::try_scan` | Same line-start gating. |
| Block ref scan | `scan_block_ref` | `BlockRefRule::try_scan` | Same tail whitespace rules. |
| Task marker scan | `handle_line_start` + `scan_task_marker` | `TaskRule::try_scan` + existing `scan_task_marker` | Same pattern; manual scan retained. |

---

## Validation Plan (No External Context Required)

**Minimum verification**
- Ensure `RawNote` output fields are populated in the same order as before.
- Ensure `sections` are sorted by `range.start()`.
- Ensure `task_marker` and `is_checked` logic unchanged.

**Test additions (existing files only)**
- Add tests for rule ordering in scanner.
- Add tests that verify list item text range is preserved.
- Add tests that verify block ref tail scan behavior.

**Manual review checklist**
- No public signature changes.
- No new files.
- No change to `RawFrontmatter::parse_fields` or `RawFieldValue::from_str_with_spec`.
- Manual task marker scan still happens in list item finalize logic.

---

## Decision Log

- Keep semantic parsing in Raw layer (user constraint).
- Retain manual task marker scan (user constraint; preserves non-standard markers).
- No new files in this phase.

---

## Glossary (Local to This Plan)

- **Scannable ranges**: Text ranges captured from pulldown-cmark events that are safe for inline scanning (exclude links and code).
- **Tail scan**: Fallback scan of last N bytes of a block for block refs.
- **Manual task marker scan**: `NoteScanner::scan_task_marker` called for list item first-line prefix.
- **Rule registry**: Explicit ordered list of scanner rules (line-start and body rules).
