# Markdown Ingestion Pipeline Redesign

## Scope

This document covers the full review, critical analysis, and redesign plan for the
markdown ingestion pipeline rooted in `lithos-core/src/note/`. It supersedes the
earlier modular ingestion plans that proposed `ExtractionSink`, `EventSink`, and
`BlockFinalizer` trait hierarchies.

Files in scope for redesign:
- `src/note/parser.rs`
- `src/note/scanner.rs`
- `src/note/processor.rs`
- `src/note/raw/aggregate.rs` and all sibling raw types

---

## 1. Current Architecture Review

### What the code actually is

`MarkdownParser` is a unit struct with a single static `parse` method. All mutable
state lives in `ParserState`, a nested aggregate of five sub-structs passed by
`&mut` through every handler call:

```
ParserState<'source, 'spec>
├── BlockState   { stack: Vec<ActiveBlock>, depth: u32 }
├── ListState    { stack: Vec<RawListKind>, contexts: Vec<ListContext>,
│                  open_item_by_depth: Vec<SourceByteOffset> }
├── LinkState    { current: Option<LinkFrame>, ref_defs: RefDefs }
├── MetaState    { in_metadata: Option<...>, buffer: String }
├── OutputState  { headings, sections, links, tags, lists, list_items,
│                  inline_fields, block_refs, frontmatter }
├── pool: StringPool
└── task_spec: &'spec TaskConfigSpec
```

`MetadataHandler`, `StartTagHandler`, `EndTagHandler`, and `TextHandler` are unit
structs whose associated methods are effectively namespaced free functions. They
exist only because `MarkdownParser` is a unit struct and cannot hold state, so the
state had to be factored into `ParserState`.

`NoteScanner` and `scanner.rs` are well-designed and should not change structurally.
The single-pass, resumable, zero-copy cursor with `LineStartRule`/`BodyRule` internal
traits is the correct design for that problem.

`NoteProcessor` in `processor.rs` uses the typestate pattern correctly. The pipeline
stages (`Discovery → Comparison → Analysis → Construction → Completed`) are genuinely
sequential and mutually exclusive, which is the right condition for typestate.

### Diagnosed problems

| Problem | Location | Description |
|---|---|---|
| Parser-as-unit-struct | `parser.rs:36` | Forces all state into `ParserState`; handlers become namespaced functions |
| `ListState` consulted at finalization | `parser.rs:463–482` | `list_kind` and `parent_pos` are read from `ListState` at `TagEnd::Item`, not encoded at push time |
| Handler unit structs | `parser.rs:613–996` | `MetadataHandler`, `StartTagHandler`, etc. exist only because `MarkdownParser` cannot hold state |
| `ScanContext` copy struct | `parser.rs:1042–1046` | Three-field wrapper passing `scanner`/`source`/`task_spec` through layers that would have access via `self` |
| Dual-write in `finalize_list_item` | `parser.rs:535–537` | Tags and fields written to both `RawListItem` and global collections — invisible invariant |
| `scan_block_artifacts` on `MarkdownParser` | `parser.rs:246–283` | Free function on a unit struct with scanner/source/task_spec passed explicitly |
| First-line boundary logic | `parser.rs:430–461` | Belongs on `NoteScanner`, not on the parser |
| `#[expect(clippy::too_many_lines)]` | `parse`, `on_start`, `on_end` | Structural signal that scope has exceeded method boundaries |
| `NotePath` in `MarkdownParser::parse` | `parser.rs:54–58`, `raw/aggregate.rs` | Path is file-system identity metadata; the parser only needs the text. `NoteProcessor` already owns the path and passes it in solely to retrieve it from `raw.path` in `persist` — a round-trip through the parser that serves no purpose |
| `load_content` duplication | `processor.rs:391–424, 428–460` | Identical body on `Comparison<Missing>` and `Analysis<Suspect>` |

---

## 2. Rejected Approaches

### 2a. Original modular plan (ExtractionSink trait)

Proposed `Vec<Box<dyn ExtractionSink<'source>>>` with a single trait covering all
extraction. Rejected for four reasons:

1. **Text accumulation left unaddressed.** `scannable_ranges` are built during the
   event loop by tracking non-link text events. Extractors that finalize blocks
   (tags, fields, block refs, tasks) need this accumulated data. The plan had no
   mechanism for getting it to them without the orchestrator still accumulating it —
   making the "externalization" cosmetic.

2. **StringPool ownership unsolvable.** The pool cannot be held mutably by both the
   orchestrator and extractors simultaneously. The plan said "pass during
   initialization or via event context" but neither option worked — the former creates
   a cross-borrow lifetime over the full parse; the latter requires `&mut` inside a
   shared event context.

3. **Object safety and harvest.** `finalize(self)` consumes the extractor and is not
   object-safe. The plan noted this but left the `harvest` mechanism unspecified.

4. **TagFinalizer and FieldFinalizer as separate extractors double-scan.** The scanner
   produces tags, fields, and block refs in a single pass via `scan_ranges_raw`.
   Separate extractors for each artifact type would call it multiple times.

### 2b. Revised two-trait plan (EventSink + BlockFinalizer)

Improved by acknowledging the two-phase distinction. Rejected in final form because:

1. **`BlockFinalizer::on_block_end` signature missing `block_range`.** Needed for
   section recording and block-ref tail scan; not derivable from `&ActiveBlock` alone.

2. **List finalization cannot be purely a `BlockFinalizer`.** `context_list.item_positions.push(...)` writes to `ListState`; the finalizer receiving `&ActiveBlock` cannot do this.

3. **Paragraph finalization has an orchestrator-owned side effect.** Propagating
   paragraph text to a parent `ListItem` requires `stack.last_mut()`, which is the
   orchestrator's stack.

4. **TaskFinalizer labelled "stateful" incorrectly.** `task_marker: Option<bool>` lives
   on `ActiveBlockMeta`, not on the finalizer. The finalizer reads it from
   `&ActiveBlock`. It has no independent state.

5. **`RefDefs` delivery to `LinkExtractor` unaddressed.**

### 2c. Typestate for extraction phases

Proposed using generic phase parameters to unify `EventSink` and `BlockFinalizer`
into one trait. Rejected because typestate models sequential state transitions within
a single object, not routing of different events to different processors.

`LinkExtractor` is permanently an event sink. `ScanFinalizer` is permanently a block
finalizer. There is no state transition to encode. A unified generic or enum payload
would force every extractor to handle both phases, adding dispatch work and hiding
intent behind no-op match arms. The two-trait split was the correct formalization of
the actual routing structure — but the full trait abstraction layer was unnecessary
overhead for a closed, non-plugin system.

---

## 3. Final Design

### Core paradigm shift

`MarkdownParser` becomes a stateful struct — the parser IS the state machine. This
eliminates `ParserState` as a wrapper, collapses the handler unit structs into methods
on the parser, and makes `self` the natural way to access scanner, source, pool, and
task_spec everywhere they're needed.

### 3a. `MarkdownParser` struct

```rust
struct MarkdownParser<'s, 'cfg> {
    // Dependencies
    source:    &'s str,
    ref_defs:  RefDefs,
    pool:      StringPool,

    // PDA state
    stack:      Vec<Block<'s>>,           // open block frames
    depth:      u32,                      // container nesting depth
    link:       Option<LinkFrame<'s>>,    // open inline link
    list_kinds: Vec<RawListKind>,         // parallel to stack for list type at push time
    list_ctxs:  Vec<ListCtx>,            // item_positions accumulator per open list
    open_items: Vec<SourceByteOffset>,   // for parent_pos at push time

    // Owned components
    extractor: BlockExtractor<'s, 'cfg>, // scan-based finalization
    out:       RawNote<'s>,              // accumulates directly; no wrapper needed
}
```

Public API is unchanged from what `NoteProcessor` already calls:

```rust
impl MarkdownParser<'_, '_> {
    pub fn parse<'s, 'cfg>(
        source: &'s str,
        path: NotePath,
        task_spec: &'cfg TaskConfigSpec,
    ) -> Result<RawNote<'s>, NoteIngestError>;
}
```

Internally, `parse` constructs the engine and runs the event loop:

```rust
pub fn parse<'s, 'cfg>(
    source: &'s str,
    task_spec: &'cfg TaskConfigSpec,
) -> Result<RawNote<'s>, NoteIngestError> {
    let base = pulldown_cmark::Parser::new_ext(source, Self::options());
    let offset_iter = base.into_offset_iter();
    let ref_defs = RefDefs::from_definitions(offset_iter.reference_definitions());

    let mut parser = Self::new(source, task_spec, ref_defs);
    let normalized = offset_iter.map(|(ev, r)| (normalize_breaks(ev), r));

    for (event, range) in TextMergeWithOffset::new(normalized) {
        parser.step(event, range)?;
    }

    parser.out.sections.sort_by_key(|s| u32::from(s.range.start()));
    Ok(parser.out)
}
```

`normalize_breaks` is a named free function (not an inline closure):

```rust
fn normalize_breaks(event: Event<'_>) -> Event<'_> {
    match event {
        Event::SoftBreak => Event::Text(CowStr::Borrowed(" ")),
        Event::HardBreak => Event::Text(CowStr::Borrowed("\n")),
        other => other,
    }
}
```

### 3b. Block frames — embedded list context

`BlockKind::ListItem` carries all context computed at push time. This is the most
important structural change: it eliminates the `ListState` lookup at finalization.

```rust
enum BlockKind {
    Metadata(MetadataBlockKind),
    Heading(u8),
    Paragraph,
    ListItem {
        list_kind:  RawListKind,
        list_depth: RawListDepth,
        parent_pos: Option<SourceByteOffset>,
    },
    List,
    BlockQuote,
    CodeBlock,
}

struct Block<'s> {
    kind:         BlockKind,
    start:        SourceByteOffset,
    text:         String,             // pool-backed
    scannable:    Vec<Range<usize>>,  // non-link text ranges
    task_checked: Option<bool>,
    _marker:      PhantomData<&'s str>,
}
```

On `Start(Tag::Item)`, the orchestrator computes `list_kind`, `list_depth`, and
`parent_pos` from `list_kinds` and `open_items` before pushing the block. At
`End(TagEnd::Item)`, finalization only reads `block.kind` — no secondary state lookup.

`_marker` ties the `Block` to `'s` even though it holds no direct `&'s` reference,
because `scannable` ranges are indices into `source`. The phantom keeps the lifetime
visible at the type level and prevents incorrect moves.

### 3c. Event dispatch

```rust
fn step(&mut self, event: Event<'s>, range: Range<usize>) -> Result<(), NoteIngestError> {
    match event {
        Event::Start(tag)              => self.on_start(tag, range.start)?,
        Event::End(tag)                => self.on_end(tag, range.end)?,
        Event::Text(text)              => self.on_text(text, range),
        Event::Code(text)              => self.on_code(text),
        Event::TaskListMarker(checked) => self.on_task_marker(checked),
        _                              => {}
    }
    Ok(())
}
```

`on_text` is the single location for link-exclusion logic:

```rust
fn on_text(&mut self, text: CowStr<'s>, range: Range<usize>) {
    if let Some(block) = self.stack.last_mut() {
        block.text.push_str(text.as_ref());
        if self.link.is_none() {
            block.scannable.push(range); // excluded when inside a link span
        }
    }
    if let Some(link) = self.link.as_mut() {
        link.alias.push_str(text.as_ref());
    }
}
```

`on_code` contributes to `full_text` (needed for list item text computation) but never
to `scannable` (inline code is not scanned for tags or fields).

### 3d. End dispatch

```rust
fn on_end(&mut self, tag: TagEnd, byte_end: usize) -> Result<(), NoteIngestError> {
    match tag {
        TagEnd::Link | TagEnd::Image  => self.finalize_link(),
        TagEnd::MetadataBlock(_)      => self.finalize_metadata(byte_end)?,
        TagEnd::Heading(_) => {
            let block = pop_block(&mut self.stack)?;
            let range = block.range_to(byte_end)?;
            self.extractor.finalize_heading(block, range, self.depth,
                                            &mut self.out, &mut self.pool)?;
        }
        TagEnd::Paragraph => {
            let block = pop_block(&mut self.stack)?;
            // Orchestrator: propagate text to parent list item if it has none yet.
            // Must happen before block is moved to extractor.
            if let Some(parent) = self.stack.last_mut() {
                if matches!(parent.kind, BlockKind::ListItem { .. })
                    && parent.text.is_empty()
                {
                    parent.text.push_str(block.text.trim());
                }
            }
            let range = block.range_to(byte_end)?;
            self.extractor.finalize_paragraph(block, range, self.depth,
                                              &mut self.out, &mut self.pool)?;
        }
        TagEnd::Item => {
            let block = pop_block(&mut self.stack)?;
            let item_start = block.start; // capture before block is moved
            let range = block.range_to(byte_end)?;
            self.extractor.finalize_list_item(block, range,
                                              &mut self.out, &mut self.pool)?;
            // Orchestrator: record item position in the parent list context.
            if let Some(ctx) = self.list_ctxs.last_mut() {
                ctx.item_positions.push(item_start);
            }
        }
        TagEnd::List(_) => {
            let block = pop_block(&mut self.stack)?;
            let range = block.range_to(byte_end)?;
            self.depth -= 1;
            if let (Some(ctx), Some(kind)) =
                (self.list_ctxs.pop(), self.list_kinds.pop())
            {
                self.out.lists.push(RawList::new(
                    kind, depth_to_raw(self.depth), range, ctx.item_positions,
                ));
            }
            self.pool.put(block.text);
        }
        TagEnd::BlockQuote(_) => {
            let block = pop_block(&mut self.stack)?;
            let range = block.range_to(byte_end)?;
            self.depth -= 1;
            self.out.sections.push(RawSection::new(RawSectionKind::BlockQuote, range, block.meta.depth));
            self.pool.put(block.text);
        }
        TagEnd::CodeBlock => {
            let block = pop_block(&mut self.stack)?;
            let range = block.range_to(byte_end)?;
            self.out.sections.push(RawSection::new(RawSectionKind::CodeBlock, range, self.depth));
            self.pool.put(block.text);
        }
        _ => {}
    }
    Ok(())
}
```

`on_end` is now a match where every arm is either a 3-line section push or a
`self.extractor.finalize_*` call. It will not grow significantly as new block types
are added.

### 3e. `BlockExtractor` — scan-based finalization component

`BlockExtractor` is a concrete struct (not a trait) that owns the three shared
dependencies required by all scan-based block finalizers. Its private methods are the
shared helpers those finalizers depend on.

**Purpose:** Extracts raw artifacts from completed parser blocks. Receives a
`Block<'s>` with accumulated text and scannable ranges, runs `NoteScanner` on the
appropriate ranges, and writes the resulting raw artifacts into `NoteOutput`.

**Boundary:** Orchestrator-level concerns (stack mutations, list context updates,
text propagation to parent blocks) are the caller's responsibility. `BlockExtractor`
only handles artifact extraction.

**Scope rule:** If finalizing a block type requires calling the scanner, the method
belongs in `BlockExtractor`. If not, it stays as a few lines in `on_end`. This rule
is unambiguous and gives new contributors a clear placement decision.

```rust
pub(crate) struct BlockExtractor<'s, 'cfg> {
    source:    &'s str,
    scanner:   NoteScanner,
    task_spec: &'cfg TaskConfigSpec,
}

impl<'s, 'cfg> BlockExtractor<'s, 'cfg> {
    // --- Public: one method per scan-based block type ---

    pub fn finalize_heading(
        &self,
        block: Block<'s>,
        block_range: SourceByteRange,
        depth: u32,
        out: &mut RawNote<'s>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::Heading(level) = block.kind else { return Ok(()) };
        let scan = self.scan_block(&block, block_range)?;
        out.headings.push(RawHeading::new(
            level,
            Cow::Owned(block.text.trim().to_owned()),
            block_range,
            block.start,
        ));
        out.sections.push(RawSection::new(RawSectionKind::Heading, block_range, depth));
        self.extend_output(scan, out);
        pool.put(block.text);
        Ok(())
    }

    pub fn finalize_paragraph(
        &self,
        block: Block<'s>,
        block_range: SourceByteRange,
        depth: u32,
        out: &mut RawNote<'s>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let scan = self.scan_block(&block, block_range)?;
        out.sections.push(RawSection::new(RawSectionKind::Paragraph, block_range, depth));
        self.extend_output(scan, out);
        pool.put(block.text);
        Ok(())
    }

    pub fn finalize_list_item(
        &self,
        block: Block<'s>,
        block_range: SourceByteRange,
        out: &mut RawNote<'s>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::ListItem { list_kind, list_depth, parent_pos } = block.kind
        else { return Ok(()) };

        let scan = self.scan_block(&block, block_range)?;
        out.sections.push(RawSection::new(
            RawSectionKind::List, block_range, depth_raw_to_u32(list_depth),
        ));

        let task_marker = if block.task_checked.is_some() {
            self.scan_task_marker_first_line(block_range)?
        } else {
            None
        };

        let (raw_text, text_range) = self.compute_item_text(&block, block_range)?;

        // block_refs route directly; tags/fields go via accept_list_item (dual-write)
        out.block_refs.extend(scan.block_refs);
        let item = RawListItem::new(
            list_kind, list_depth,
            Cow::Owned(raw_text),
            block.task_checked, task_marker,
            block_range, text_range, parent_pos,
            scan.tags,
            scan.fields_into_raw(self.task_spec),
        );
        out.accept_list_item(item);
        pool.put(block.text);
        Ok(())
    }

    // --- Private: shared by all public methods ---

    /// Routes scan artifacts into the output. Tags and inline fields are
    /// written directly; block refs too. The task-spec conversion for inline
    /// fields happens here where task_spec is available.
    ///
    /// Not used for list items — those go through `RawNote::accept_list_item`
    /// to enforce the dual-write invariant.
    fn extend_output(&self, scan: ScannedRawArtifacts<'s>, out: &mut RawNote<'s>) {
        out.tags.extend(scan.tags);
        out.inline_fields.extend(
            scan.inline_fields.into_iter().map(|t| field_token_to_raw(t, self.task_spec))
        );
        out.block_refs.extend(scan.block_refs);
    }


    /// Scans a block's scannable ranges for metadata artifacts.
    /// Includes the block-ref tail fallback when no refs are found in
    /// the primary scan.
    fn scan_block(
        &self,
        block: &Block<'s>,
        block_range: SourceByteRange,
    ) -> Result<ScannedRawArtifacts<'s>, NoteIngestError> {
        let mut raw = self.scanner
            .scan_ranges_raw(self.source, &block.scannable, false)
            .map_err(NoteIngestError::Domain)?;

        if raw.block_refs.is_empty() {
            let last_end = block.scannable.last().map(|r| r.end);
            if last_end == Some(block_range.end().as_usize()) {
                if let Some(tail) = block_ref_tail_range(self.source, block_range) {
                    let tail_raw = self.scanner
                        .scan_ranges_raw(self.source, &[tail], false)
                        .map_err(NoteIngestError::Domain)?;
                    raw.block_refs.extend(tail_raw.block_refs);
                }
            }
        }
        Ok(raw)
    }

    /// Scans the first line of a list item block for a task status symbol.
    /// Caps at 80 bytes since checkboxes always appear near the start.
    fn scan_task_marker_first_line(
        &self,
        block_range: SourceByteRange,
    ) -> Result<Option<RawTaskStatusSymbol>, NoteIngestError> {
        let start = block_range.start().as_usize();
        let slice = self.source.get(start..).unwrap_or("");
        let first_line_len = slice.find('\n').unwrap_or(slice.len()).min(80);
        let end = to_offset(start.saturating_add(first_line_len))?;
        let prefix = SourceByteRange::new(block_range.start(), end)
            .map_err(NoteIngestError::Domain)?;
        NoteScanner::scan_task_marker(self.source, prefix)
            .map_err(NoteIngestError::Domain)
    }

    /// Computes the trimmed text content and its source-mapped byte range
    /// for a list item. Used to populate `RawListItem::text` and
    /// `RawListItem::text_range`.
    fn compute_item_text(
        &self,
        block: &Block<'s>,
        block_range: SourceByteRange,
    ) -> Result<(String, SourceByteRange), NoteIngestError> {
        let raw_text = block.text.trim().to_owned();
        let text_range = if raw_text.is_empty() {
            SourceByteRange::new(block_range.start(), block_range.start())
                .map_err(NoteIngestError::Domain)?
        } else {
            let leading_trim = block.text.len()
                .saturating_sub(block.text.trim_start().len());
            let base_start = block.scannable.first()
                .and_then(|r| SourceByteOffset::try_from(r.start).ok())
                .unwrap_or(block_range.start());
            let text_start = base_start
                .add_offset(leading_trim)
                .map_err(NoteIngestError::Domain)?;
            let text_end = text_start
                .add_offset(raw_text.len())
                .map_err(NoteIngestError::Domain)?;
            SourceByteRange::new(text_start, text_end)
                .map_err(NoteIngestError::Domain)?
        };
        Ok((raw_text, text_range))
    }
}
```

**Extensibility rule:** When a new block type requires scanning (e.g., definition list
items), add a `finalize_<kind>` method to `BlockExtractor`. Private helpers are
already available to it. The orchestrator's `on_end` gains one new arm calling the
new method. No other file changes.

### 3f. `RawNote` as the direct accumulator

`NoteOutput` is not introduced as a separate type. `MarkdownParser` holds `out: RawNote<'s>`
and accumulates into it directly. The parser's `parse()` sorts sections in place
before returning `out`:

```rust
parser.out.sections.sort_by_key(|s| u32::from(s.range.start()));
Ok(parser.out)
```

Without `path`, `NoteOutput` and `RawNote` would have been structurally identical —
introducing a builder type whose `build()` does nothing except move and sort one `Vec`
adds indirection without value. A builder is warranted when the intermediate state
is meaningfully different from the result, or when construction requires invariants
the result type should not expose. Neither applies here.

`RawNote` gains one `pub(crate)` method — the only place with a real invariant to
enforce:

```rust
impl<'s> RawNote<'s> {
    /// Pushes a list item and performs the required dual-write.
    ///
    /// List item tags and inline fields appear in both the item itself
    /// (for item-level queries) and the global collections (for note-level
    /// queries). This is the single location that invariant is applied.
    pub(crate) fn accept_list_item(&mut self, item: RawListItem<'s>) {
        self.tags.extend(item.tags.iter().cloned());
        self.inline_fields.extend(item.inline_fields.iter().cloned());
        self.list_items.push(item);
    }
}
```

All other accumulation in `BlockExtractor` is direct field access
(`out.tags.extend(...)`, `out.sections.push(...)`) or the private `extend_output`
helper (§3e) which handles the task-spec conversion for inline fields. The section
sort at parse end is one inline call, not a method.

This keeps `RawNote` a simple data type with one named invariant, and avoids
adding a `pub(crate)` mutation surface beyond what is strictly necessary.

### 3g. `scanner.rs` changes

`NoteScanner` is largely unchanged. One method moves from `parser.rs` to the scanner
where it belongs:

```rust
impl NoteScanner {
    /// Scans a block range for a task status symbol, capped to the first line.
    ///
    /// Moved from `MarkdownParser`: the first-line boundary heuristic is a
    /// scanning concern, not a parsing concern.
    pub(crate) fn scan_task_marker_first_line(
        source: &str,
        block_range: SourceByteRange,
    ) -> Result<Option<RawTaskStatusSymbol>, NoteError> { ... }
}
```

Note: `BlockExtractor::scan_task_marker_first_line` is a thin wrapper calling this
after computing the range. The core logic lives on `NoteScanner`.

### 3h. `RawNote` — removing `path`

`path: NotePath` is removed from `RawNote`. The name stays. `MarkdownParser::parse`
returns `RawNote<'source>` as before, just without file identity in the struct.

The `New` and `Changed` processor status structs pick up a `path: NotePath` field to
carry the path through to `persist`, since `raw` no longer holds it:

```rust
pub struct New {
    raw:  RawNote<'static>,
    path: NotePath,
}

pub struct Changed {
    raw:  RawNote<'static>,
    path: NotePath,
}
```

`NoteProcessor::parse` takes the path from its own state when transitioning:

```rust
fn parse(
    self,
    task_spec: &TaskConfigSpec,
) -> Result<AnalysisBranch, NoteProcessError> {
    let raw  = MarkdownParser::parse(&self.status.content, task_spec)
        .map(RawNote::into_owned)?;
    let path = self.status.info.path.clone(); // already owned here

    if self.status.is_new {
        Ok(AnalysisBranch::New(Self::transition(Construction, New { raw, path })))
    } else {
        Ok(AnalysisBranch::Changed(Self::transition(Construction, Changed { raw, path })))
    }
}
```

`persist` uses `self.status.path` directly:

```rust
fn persist(...) -> Result<NoteProcessReport, NoteProcessError> {
    let path    = self.status.path.clone();
    let note_id = repository.find_by_path(&path)?
        .map_or_else(NoteId::new, |n| n.id());
    let facts   = Note::try_from((
        self.status.raw, &path, note_id, frontmatter_spec, task_spec,
    ))?;
    ...
}
```

`Note::try_from` receives path as an explicit argument. The domain object is
constructed from parse artifacts plus file identity, with those two concerns
visibly separate at the call site.

### 3i. `raw/` changes

All existing raw types remain unchanged except two targeted changes:

**`raw/aggregate.rs`** — `path: NotePath` field removed from `RawNote`.
`accept_list_item` added as a `pub(crate)` method (see §3f).

**`ScannedRawArtifacts` gains a convenience method** for converting inline field
tokens to `RawInlineField` with task spec application:

```rust
impl<'s> ScannedRawArtifacts<'s> {
    pub(crate) fn fields_into_raw(
        self,
        task_spec: &TaskConfigSpec,
    ) -> Vec<RawInlineField<'s>> {
        self.inline_fields.into_iter()
            .map(|t| field_token_to_raw(t, task_spec))
            .collect()
    }
}
```

`field_token_to_raw` (currently `MarkdownParser::inline_field_from_token`) moves to
`raw/inline_field.rs` as a free function, adjacent to the types it operates on.

### 3i. `processor.rs` changes

Minimal. The typestate pipeline is correct. One cleanup: `load_content` is implemented
identically on both `NoteProcessor<Comparison, Missing>` and
`NoteProcessor<Analysis, Suspect>`. Factor the shared body into a private free
function that both delegate to:

```rust
fn read_and_process<R: Repository<Error = NoteRepositoryError>>(
    suspect: NoteProcessor<Analysis, Suspect>,
    repository: &R,
    source: &FsReader,
    task_spec: &TaskConfigSpec,
    frontmatter_spec: &FrontmatterConfigSpec,
) -> Result<NoteProcessReport, NoteProcessError> { ... }
```

Both `impl` blocks become one-line delegations. The public typestate API is unchanged.

---

## 4. Module Structure

```
src/note/
├── mod.rs
├── parser.rs          // MarkdownParser stateful struct
│                      // on_start, on_end, on_text, on_code, on_task_marker
│                      // finalize_link, finalize_metadata
│                      // normalize_breaks, block_ref_tail_range (free functions)
│                      // Block, BlockKind, LinkFrame, ListCtx, RefDefs, StringPool
├── extractor.rs       // BlockExtractor struct + finalize_heading/paragraph/list_item
│                      // Private: scan_block, scan_task_marker_first_line,
│                      //          compute_item_text
├── scanner.rs         // NoteScanner (unchanged except scan_task_marker_first_line)
├── processor.rs       // NoteProcessor typestate (load_content deduped only)
├── error.rs           // unchanged
└── raw/
    ├── mod.rs         // unchanged
    ├── aggregate.rs   // RawNote: path field removed; accept_list_item added pub(crate)
    ├── block_ref.rs   // unchanged
    ├── frontmatter.rs // unchanged
    ├── heading.rs     // unchanged
    ├── inline_field.rs// + field_token_to_raw free function (moved from parser.rs)
    ├── link.rs        // unchanged
    ├── list.rs        // unchanged
    ├── section.rs     // unchanged
    ├── tag.rs         // unchanged
    └── value.rs       // unchanged
```

No `extractors/` directory. No trait files. No `context.rs`. No `raw/output.rs`.
One new file (`extractor.rs`) with one clear purpose derivable from its name.

---

## 5. What Changes vs What Stays

### Changes

| Before | After | Reason |
|---|---|---|
| `MarkdownParser` unit struct + `ParserState` | `MarkdownParser` IS the state | Eliminates the wrapper and enables methods with `&mut self` |
| Handler unit structs (`MetadataHandler`, etc.) | Methods on `MarkdownParser` | They were namespaced functions; `self` now provides context |
| `BlockKind::ListItem` (no data) + `ListState` at finalization | `BlockKind::ListItem { list_kind, list_depth, parent_pos }` computed at push | Eliminates secondary stack lookup at finalization |
| `finalize_*` methods on `MarkdownParser` | `BlockExtractor::finalize_*` for scan-based types; 3-liners stay on parser | Gives the extractor a clear home with shared private helpers |
| `OutputState` in `ParserState` | `RawNote<'s>` field on `MarkdownParser`, accumulated directly | No wrapper needed when accumulator and result are structurally identical; one `pub(crate)` method on `RawNote` enforces the only real invariant (dual-write for list items) |
| `ScanContext` copy struct | Eliminated | `self.source`, `self.scanner`, `self.task_spec` are on the struct |
| `scan_block_artifacts` free function on `MarkdownParser` | `BlockExtractor::scan_block` private method | Natural home with other scan helpers |
| First-line boundary logic in `finalize_list_item` | `NoteScanner::scan_task_marker_first_line` | Scanning concern on the scanner |
| `inline_field_from_token` on `MarkdownParser` | `field_token_to_raw` free function in `raw/inline_field.rs` | Adjacent to the types it operates on |
| `NotePath` in `MarkdownParser::parse` and `RawNote` | Removed. `MarkdownParser::parse` returns `ParsedNote` (no path). `NoteProcessor` supplies path from its own state to `Note::try_from` directly | File identity is not a parsing concern |
| `load_content` duplicated on two `impl` blocks | Delegated to one shared free function | Eliminates identical bodies |

### Unchanged

| Component | Reason |
|---|---|
| `NoteScanner`, `Cursor`, rule structs | Already optimal: single-pass, zero-copy, resumable |
| All `raw/` types | Well-designed; `RawNote`, `RawListItem`, etc. require no structural changes |
| `NoteProcessor` typestate pipeline | Correct pattern for the problem: sequential exclusive phases |
| `TextMergeWithOffset` usage | Correct pulldown-cmark usage pattern |
| `RefDefs` normalization logic | Correct RFC 5322-style case-folding with first-wins semantics |
| `StringPool` with thread-local metrics | Pool pattern is correct; metrics API should remain stable |
| `block_ref_tail_range` heuristic | Heuristic is correct; becomes a free function in `parser.rs` |

---

## 6. Extensibility Guidance

### Adding a new block type that requires scanning

1. Add a variant to `BlockKind` carrying whatever context should be computed at push time.
2. Handle `Start(Tag::NewKind)` in `MarkdownParser::on_start`: push the block with embedded context.
3. Add `BlockExtractor::finalize_new_kind(...)`. The private helpers `scan_block`, `scan_task_marker_first_line`, and `compute_item_text` are available.
4. Handle `End(TagEnd::NewKind)` in `MarkdownParser::on_end`: pop the block, apply any orchestrator side effects, call `self.extractor.finalize_new_kind(...)`.
5. If the new type produces artifacts that need to reach `RawNote`, add a field to `RawNote` and a `pub(crate)` method if it has an invariant (like the dual-write for list items). If not, extend the field directly from `BlockExtractor::extend_output`.

### Adding a new block type that does not require scanning

1–2 as above.
3. Handle `End(TagEnd::NewKind)` in `on_end` directly: pop block, push a `RawSection`, return text to pool. No changes to `BlockExtractor` needed.

### Adding a new scanner artifact type

1. Add a variant to `ScannedArtifact` and `ScannedRawArtifacts` in `scanner.rs`.
2. Implement the corresponding scan rule as a `LineStartRule` or `BodyRule` in `scanner.rs`.
3. Add the artifact type to `raw/`.
4. Add a field to `RawNote` and a `pub(crate)` method if there is an invariant to enforce; otherwise extend directly in `BlockExtractor::extend_output`.
5. `BlockExtractor::extend_output` routes the new artifact type from `ScannedRawArtifacts` to `RawNote`.

---

## 7. Compatibility Notes

- `MarkdownParser::parse` loses its `path: NotePath` parameter. It still returns `RawNote<'source>`. The `NoteProcessor` call site stops passing path and instead carries it in the `New`/`Changed` status structs.
- `MarkdownParser::extension_options` (public, used for compatibility checks) remains as a `const fn`.
- `RawNote` keeps its name; only `path: NotePath` is removed. `Note::try_from` receives path as a separate argument. Any code that reads `raw.path` must source the path from its own context (the processor already has it in `self.status.info.path`).
- `StringPool`, `get_string_pool_metrics`, and `reset_string_pool_metrics` remain public and unchanged.
- All existing tests in `parser.rs` should pass without modification; they test `MarkdownParser::parse` via `parse_raw` which is unchanged at the public API level.
