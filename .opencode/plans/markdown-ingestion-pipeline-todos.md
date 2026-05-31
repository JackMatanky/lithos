# Markdown Ingestion Pipeline — Implementation Todos

Reference plan: `.opencode/plans/markdown-ingestion-pipeline-redesign.md`

Tasks are ordered by dependency. Complete each phase before starting the next.
Within a phase, tasks marked *(independent)* can be done in any order.

---

## Phase 1 — Raw Type Changes (no breaking changes to public API yet)

These tasks add new items or make additive changes. The existing parser still
compiles throughout this phase.

### 1.1 Move `inline_field_from_token` *(independent)*

- In `raw/inline_field.rs`, add a `pub(crate)` free function `field_token_to_raw`:
  ```rust
  pub(crate) fn field_token_to_raw<'s>(
      token: RawInlineFieldToken<'s>,
      task_spec: &TaskConfigSpec,
  ) -> RawInlineField<'s>
  ```
- Copy the body verbatim from `MarkdownParser::inline_field_from_token` in
  `parser.rs` (lines ~285–317).
- Do not delete `inline_field_from_token` from `parser.rs` yet — that happens
  in Phase 3 when the old parser is replaced.

### 1.2 Add `ScannedRawArtifacts::fields_into_raw` *(independent)*

- In `scanner.rs`, add to `impl<'s> ScannedRawArtifacts<'s>`:
  ```rust
  pub(crate) fn fields_into_raw(
      self,
      task_spec: &TaskConfigSpec,
  ) -> Vec<RawInlineField<'s>>
  ```
- Body: iterate `self.inline_fields`, map each through `field_token_to_raw`
  (the function added in 1.1), collect.
- Add the required import for `field_token_to_raw`.

### 1.3 Add `NoteScanner::scan_task_marker_first_line` *(independent)*

- In `scanner.rs`, add to `impl NoteScanner`:
  ```rust
  pub(crate) fn scan_task_marker_first_line(
      source: &str,
      block_range: SourceByteRange,
  ) -> Result<Option<RawTaskStatusSymbol>, NoteError>
  ```
- Body: extract `start = block_range.start().as_usize()`, slice source from
  start, find first `'\n'` or end-of-slice (capped at 80 bytes), construct a
  `SourceByteRange` for that prefix, delegate to `Self::scan_task_marker`.
- This is the first-line-boundary logic currently in `finalize_list_item`
  (`parser.rs` lines ~430–461).

### 1.4 Add `RawNote::accept_list_item` *(independent)*

- In `raw/aggregate.rs`, add to `impl<'s> RawNote<'s>`:
  ```rust
  pub(crate) fn accept_list_item(&mut self, item: RawListItem<'s>)
  ```
- Body:
  ```rust
  self.tags.extend(item.tags.iter().cloned());
  self.inline_fields.extend(item.inline_fields.iter().cloned());
  self.list_items.push(item);
  ```
- Add a doc comment explaining the dual-write invariant: tags and inline fields
  appear in both the item and the global collections.

### 1.5 Remove `path` from `RawNote`

- In `raw/aggregate.rs`, delete `pub path: NotePath` from `RawNote<'source>`.
- Update `RawNote::new`: remove the `path: NotePath` parameter and the
  `path: path` field assignment.
- Update `RawNote::into_owned`: remove the `path: self.path` field.
- Fix the one call site in `parser.rs` (`Ok(RawNote::new(path, ...))`) to drop
  the `path` argument — this will cause a compile error that is resolved in
  Phase 3 when `parse()` is rewritten.
- Update `raw/mod.rs` if `NotePath` import is now unused there.

---

## Phase 2 — `BlockExtractor` (new file)

Create `src/note/extractor.rs`. Declare it in `src/note/mod.rs` as
`pub(crate) mod extractor;` (or `mod extractor;` if kept private to the crate).

### 2.1 Define the `BlockExtractor` struct

```rust
pub(crate) struct BlockExtractor<'s, 'cfg> {
    source:    &'s str,
    scanner:   NoteScanner,
    task_spec: &'cfg TaskConfigSpec,
}
```

Add a `pub(crate) fn new(...)` constructor.

### 2.2 Define `Block<'s>` and updated `BlockKind`

These types will be used by both `extractor.rs` and `parser.rs`. Place them in
`parser.rs` (they are internal parser types); `extractor.rs` imports them.

**`BlockKind`** — update the existing enum:
- Keep: `Heading(u8)`, `Paragraph`, `List`, `BlockQuote`, `CodeBlock`
- Add `Metadata(pulldown_cmark::MetadataBlockKind)` to cover frontmatter
  (currently handled separately via `MetaState`)
- Replace the bare `ListItem` variant with:
  ```rust
  ListItem {
      list_kind:  RawListKind,
      list_depth: RawListDepth,
      parent_pos: Option<SourceByteOffset>,
  }
  ```

**`Block<'s>`** — replaces `ActiveBlock`. Fields:
```rust
struct Block<'s> {
    kind:         BlockKind,
    start:        SourceByteOffset,
    text:         String,              // pool-backed
    scannable:    Vec<Range<usize>>,   // non-link text ranges
    task_checked: Option<bool>,
    _marker:      PhantomData<&'s str>,
}
```

Add `fn range_to(&self, byte_end: usize) -> Result<SourceByteRange, NoteIngestError>`.

### 2.3 Add free functions to `parser.rs`

These are module-level free functions (not methods):

- **`normalize_breaks`** — extract the inline closure currently in the event
  iterator map:
  ```rust
  fn normalize_breaks(event: Event<'_>) -> Event<'_>
  ```

- **`block_ref_tail_range`** — move `MarkdownParser::block_ref_tail_range` to
  a free function. Keep it in `parser.rs` next to `BlockExtractor`'s caller.

- **`pop_block`** — a small helper used in `on_end`:
  ```rust
  fn pop_block(stack: &mut Vec<Block<'_>>) -> Result<Block<'_>, NoteIngestError>
  ```
  Returns an error if the stack is empty (mismatched tags).

- **`depth_to_raw`** — converts `u32` depth to `RawListDepth`:
  ```rust
  fn depth_to_raw(depth: u32) -> RawListDepth
  ```

- **`depth_raw_to_u32`** — inverse, for section push:
  ```rust
  fn depth_raw_to_u32(depth: RawListDepth) -> u32
  ```

- **`to_offset`** — promote the existing `let to_offset = |start| { ... }`
  closure to a free function so it can be called from both `parser.rs` and
  `extractor.rs`:
  ```rust
  fn to_offset(byte: usize) -> Result<SourceByteOffset, NoteIngestError>
  ```

### 2.4 Implement `BlockExtractor` private helpers

Implement in order (later methods depend on earlier ones):

**`scan_block`** (private):
- Calls `self.scanner.scan_ranges(self.source, &block.scannable, false)`.
- If `raw.block_refs` is empty and the last scannable range ends at
  `block_range.end()`, calls `block_ref_tail_range` and does a second
  `scan_ranges` on the tail, extending `raw.block_refs`.
- Returns `ScannedRawArtifacts<'s>`.

**`scan_task_marker_first_line`** (private wrapper):
- Delegates to `NoteScanner::scan_task_marker_first_line(self.source, block_range)`.
- Maps the `NoteError` to `NoteIngestError::Domain`.

**`compute_item_text`** (private):
- Takes `&Block<'s>` and `block_range: SourceByteRange`.
- Returns `Result<(String, SourceByteRange), NoteIngestError>`.
- Body: trim `block.text`, compute `text_range` from leading whitespace offset
  and `block.scannable.first()`. Matches the logic in the current
  `finalize_list_item` lines ~493–517.

**`extend_output`** (private):
- Signature: `fn extend_output(&self, scan: ScannedRawArtifacts<'s>, out: &mut RawNote<'s>)`
- Body:
  ```rust
  out.tags.extend(scan.tags);
  out.inline_fields.extend(
      scan.inline_fields.into_iter().map(|t| field_token_to_raw(t, self.task_spec))
  );
  out.block_refs.extend(scan.block_refs);
  ```

### 2.5 Implement `BlockExtractor` public methods

**`finalize_heading`**:
- Signature: `pub fn finalize_heading(&self, block: Block<'s>, block_range: SourceByteRange, depth: u32, out: &mut RawNote<'s>, pool: &mut StringPool) -> Result<(), NoteIngestError>`
- Destructure `BlockKind::Heading(level)`.
- Call `self.scan_block(&block, block_range)?`.
- Push to `out.headings` and `out.sections`.
- Call `self.extend_output(scan, out)`.
- Return text to pool: `pool.put(block.text)`.

**`finalize_paragraph`**:
- Same signature shape (no level, takes `depth: u32`).
- Call `self.scan_block`, push section, call `self.extend_output`, return text
  to pool. No heading push.

**`finalize_list_item`**:
- Destructure `BlockKind::ListItem { list_kind, list_depth, parent_pos }`.
- Call `self.scan_block`.
- Push list section.
- Call `self.scan_task_marker_first_line` if `block.task_checked.is_some()`.
- Call `self.compute_item_text`.
- Extend `out.block_refs` directly (block refs are not dual-written).
- Build `RawListItem` with `scan.tags` and `scan.fields_into_raw(self.task_spec)`.
- Call `out.accept_list_item(item)` — this applies the dual-write.
- Return text to pool.

---

## Phase 3 — `MarkdownParser` Refactor (the main change)

This phase replaces the unit-struct parser with a stateful struct. The old
`ParserState` and all handler unit structs are deleted.

### 3.1 Define the `MarkdownParser` struct

Replace the existing `pub struct MarkdownParser;` with:

```rust
pub(crate) struct MarkdownParser<'s, 'cfg> {
    // Dependencies
    source:    &'s str,
    ref_defs:  RefDefs,
    pool:      StringPool,

    // PDA state
    stack:      Vec<Block<'s>>,
    depth:      u32,
    link:       Option<LinkFrame<'s>>,
    list_kinds: Vec<RawListKind>,
    list_ctxs:  Vec<ListCtx>,
    open_items: Vec<SourceByteOffset>,

    // Components
    extractor: BlockExtractor<'s, 'cfg>,
    out:       RawNote<'s>,
}
```

Add a private `fn new(source: &'s str, task_spec: &'cfg TaskConfigSpec, ref_defs: RefDefs) -> Self` constructor that initialises all fields with appropriate capacities (mirror the current `Vec::with_capacity` hints in `ParserState`).

`RawNote` initialisation in `new()`: all `Vec` fields empty, `frontmatter: None`.

### 3.2 Rewrite `MarkdownParser::parse`

Keep the function public with the updated signature:
```rust
pub fn parse<'s>(
    source: &'s str,
    task_spec: &TaskConfigSpec,
) -> Result<RawNote<'s>, NoteIngestError>
```

Body:
1. Construct `pulldown_cmark::Parser`, get `offset_iter`.
2. Collect `ref_defs` from `offset_iter.reference_definitions()`.
3. Call `Self::new(source, task_spec, ref_defs)` to get `parser`.
4. Map events through `normalize_breaks`, wrap in `TextMergeWithOffset`.
5. `for (event, range) in merged { parser.step(event, range)?; }`
6. `parser.out.sections.sort_by_key(|s| u32::from(s.range.start()));`
7. `Ok(parser.out)`

Remove the `path: NotePath` parameter entirely.

### 3.3 Implement `step`

```rust
fn step(&mut self, event: Event<'s>, range: Range<usize>) -> Result<(), NoteIngestError>
```

Clean match:
```rust
match event {
    Event::Start(tag)              => self.on_start(tag, range.start)?,
    Event::End(tag)                => self.on_end(tag, range.end)?,
    Event::Text(text)              => self.on_text(text, range),
    Event::Code(text)              => self.on_code(text),
    Event::TaskListMarker(checked) => self.on_task_marker(checked),
    _                              => {}
}
Ok(())
```

### 3.4 Implement `on_start`

Replaces `StartTagHandler::on_start`. Signature:
```rust
fn on_start(&mut self, tag: Tag<'s>, byte_start: usize) -> Result<(), NoteIngestError>
```

Match on `tag`. Key cases:

- `Tag::MetadataBlock(kind)` → push `Block` with `BlockKind::Metadata(kind)`.
- `Tag::Heading { level, .. }` → push `Block` with `BlockKind::Heading(level_to_u8(level))`.
- `Tag::Paragraph` → push `Block` with `BlockKind::Paragraph`.
- `Tag::List(n)` → compute `kind`, push to `self.list_kinds`, push a new
  `ListCtx` to `self.list_ctxs`, increment `self.depth`, push `Block` with
  `BlockKind::List`.
- `Tag::Item` → compute `list_kind` from `self.list_kinds.last()`, compute
  `list_depth` from `self.depth`, compute `parent_pos` from
  `self.open_items.get(depth_index - 1)`, call `self.record_open_item(start, depth)`,
  push `Block` with `BlockKind::ListItem { list_kind, list_depth, parent_pos }`.
- `Tag::BlockQuote(_)` → increment `self.depth`, push `Block` with
  `BlockKind::BlockQuote`.
- `Tag::CodeBlock(_)` → push `Block` with `BlockKind::CodeBlock`.
- `Tag::Link { link_type, dest_url, .. }` → call `self.open_link(...)`.
- `Tag::Image { link_type, dest_url, .. }` → call `self.open_link(...)` with
  `is_embed = matches!(link_type, WikiLink { .. })`.
- All other tags → `{}`.

Add private helper `fn record_open_item(&mut self, start: SourceByteOffset, depth: u32)`:
- Resize `self.open_items` to `depth_index + 1` if needed.
- Set `self.open_items[depth_index] = start`.
- Truncate `self.open_items` to `depth_index + 1`.
(This is the `open_item_by_depth` logic from `StartTagHandler::on_start` lines ~773–788.)

Add private helper `fn open_link(&mut self, link_type, dest_url, is_embed, start)`:
- Resolve reference target via `self.ref_defs`.
- Set `self.link = Some(LinkFrame { ... alias: self.pool.take() ... })`.

Add private helper `fn push_block(&mut self, kind: BlockKind, start: SourceByteOffset)`:
- Constructs `Block { kind, start, text: self.pool.take(), scannable: Vec::with_capacity(4), task_checked: None, _marker: PhantomData }`.
- Pushes to `self.stack`.

### 3.5 Implement `on_end`

Replaces `EndTagHandler::on_end`. Signature:
```rust
fn on_end(&mut self, tag: TagEnd, byte_end: usize) -> Result<(), NoteIngestError>
```

Match per the plan's `on_end` listing in §3d. Key cases:

- `TagEnd::Link | TagEnd::Image` → `self.finalize_link()`.
- `TagEnd::MetadataBlock(_)` → `self.finalize_metadata(byte_end)?`.
- `TagEnd::Heading(_)` → `pop_block`, compute range, call
  `self.extractor.finalize_heading(...)`.
- `TagEnd::Paragraph` → `pop_block`, propagate text to parent if parent is
  `ListItem` with empty text, compute range, call
  `self.extractor.finalize_paragraph(...)`.
- `TagEnd::Item` → `pop_block`, capture `item_start = block.start`, compute
  range, call `self.extractor.finalize_list_item(...)`, then push `item_start`
  into `self.list_ctxs.last_mut()`.
- `TagEnd::List(_)` → `pop_block`, compute range, decrement `self.depth`,
  pop `self.list_ctxs` and `self.list_kinds`, push to `self.out.lists`,
  return text to pool.
- `TagEnd::BlockQuote(_)` → `pop_block`, compute range, decrement
  `self.depth`, push section to `self.out.sections`, return text to pool.
- `TagEnd::CodeBlock` → `pop_block`, compute range, push section, return
  text to pool.
- All others → `{}`.

### 3.6 Implement `on_text`, `on_code`, `on_task_marker`

**`on_text`** (replaces `TextHandler::on_scannable_text`):
```rust
fn on_text(&mut self, text: CowStr<'s>, range: Range<usize>)
```
- If `self.stack.last_mut()` exists: `push_str` to `block.text`; if
  `self.link.is_none()`, push `range` to `block.scannable`.
- If `self.link.as_mut()` exists: `push_str` to `link.alias`.

**`on_code`** (replaces `TextHandler::on_unscannable_text`):
```rust
fn on_code(&mut self, text: CowStr<'s>)
```
- If `self.stack.last_mut()` exists: `push_str` to `block.text` only (no
  scannable range — inline code is not scanned).
- If `self.link.as_mut()` exists: `push_str` to `link.alias`.

**`on_task_marker`**:
```rust
fn on_task_marker(&mut self, checked: bool)
```
- `if let Some(block) = self.stack.last_mut() { block.task_checked = Some(checked); }`

### 3.7 Implement `finalize_link` and `finalize_metadata`

**`finalize_link`**:
- If `self.link.take()` is `Some(link)`: split target into path + anchor via
  `LinkTarget::new(...).split()`, trim alias, push `RawLink` to
  `self.out.links`, return `link.alias` to pool.
- This is the `TagEnd::Link | TagEnd::Image` arm from `EndTagHandler::on_end`.

**`finalize_metadata`**:
```rust
fn finalize_metadata(&mut self, byte_end: usize) -> Result<(), NoteIngestError>
```
- `pop_block` from stack.
- Construct `block_range` from `block.start` and `to_offset(byte_end)`.
- Destructure `BlockKind::Metadata(kind)` from `block.kind`.
- Push `RawSection` with `RawSectionKind::Frontmatter` to `self.out.sections`.
- Set `self.out.frontmatter = Some(RawFrontmatter::new(kind.into(), block.text.into(), block_range))`.
- Note: `block.text` is moved into `RawFrontmatter`, so it does not return to
  the pool. This mirrors the current `MetadataHandler` behaviour.

### 3.8 Remove deleted code

Once all new methods are implemented and the file compiles:

- Delete `MetadataHandler` struct and `impl`.
- Delete `StartTagHandler` struct and `impl`.
- Delete `EndTagHandler` struct and `impl`.
- Delete `TextHandler` struct and `impl`.
- Delete `ParserState<'source, 'spec>` struct.
- Delete `BlockState` struct.
- Delete `ListState` struct.
- Delete `LinkState` struct.
- Delete `MetaState` struct.
- Delete `OutputState` struct.
- Delete `ActiveBlock`, `ActiveBlockMeta`, `ActiveBlockText` structs.
- Delete `ScanContext` struct.
- Delete `ListContext` struct (replaced by `ListCtx`).
- Delete `MarkdownParser::scan_block_artifacts` static method.
- Delete `MarkdownParser::inline_field_from_token` static method (moved to
  `raw/inline_field.rs` in Phase 1).
- Delete `MarkdownParser::block_ref_tail_range` static method (now a free function).
- Delete `MarkdownParser::finalize_paragraph` static method.
- Delete `MarkdownParser::finalize_list` static method.
- Delete `MarkdownParser::finalize_list_item` static method.
- Delete `MarkdownParser::heading_level_value` static method (replace with
  `level_to_u8` free function used inline).
- Delete `#[expect(clippy::too_many_lines)]` from `parse`, `on_start`, `on_end`
  once the new implementations no longer need them.
- Delete the `ScannedArtifacts<'source>` type alias (no longer used by parser).

### 3.9 Update `mod.rs`

- Add `pub(crate) mod extractor;` (or `mod extractor;`) to `src/note/mod.rs`.
- Remove any imports/re-exports that no longer exist.

---

## Phase 4 — `NoteProcessor` Updates

### 4.1 Add `path` to `New` and `Changed` status structs

In `processor.rs`:
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

### 4.2 Update `NoteProcessor<Analysis, Suspect>::parse`

- Remove `path` argument from `MarkdownParser::parse` call.
- Capture `let path = self.status.info.path.clone()` after parsing.
- Pass `path` into both `New { raw, path }` and `Changed { raw, path }`.

### 4.3 Update `persist` on `New` and `Changed`

In `NoteProcessor<Construction, New>::persist` and
`NoteProcessor<Construction, Changed>::persist`:

- Replace `let path = self.status.raw.path.clone()` with
  `let path = self.status.path.clone()`.
- Update `Note::try_from(...)` call to pass `&path` as a separate argument
  (see 4.4).

### 4.4 Update `Note::try_from`

In `aggregate.rs` (the `Note` domain type, not `RawNote`):

- The `TryFrom` impl currently takes `(RawNote, NoteId, &FrontmatterConfigSpec, &TaskConfigSpec)`.
- Update to take `(RawNote, &NotePath, NoteId, &FrontmatterConfigSpec, &TaskConfigSpec)`.
- Update the body to read path from the new argument instead of `raw.path`.
- Update all call sites (only in `processor.rs`).

### 4.5 Deduplicate `load_content`

The body of `NoteProcessor<Comparison, Missing>::load_content` and
`NoteProcessor<Analysis, Suspect>::load_content` are identical. Extract to a
private free function:

```rust
fn read_parse_and_persist<R: Repository<Error = NoteRepositoryError>>(
    path: NotePath,
    content: String,
    is_new: bool,
    repository: &R,
    source: &FileReader,
    task_spec: &TaskConfigSpec,
    frontmatter_spec: &FrontmatterConfigSpec,
) -> Result<NoteProcessReport, NoteProcessError>
```

Both `impl` blocks call this function. The public typestate API is unchanged.

---

## Phase 5 — Cleanup and Verification

### 5.1 Remove `parser.rs.bak`

The file `src/note/parser.rs.bak` exists in the directory listing. Delete it.

### 5.2 Audit imports

- In `parser.rs`: remove all imports for deleted types (`ScanContext`,
  `ActiveBlock`, handler unit structs, etc.). Add imports for `extractor.rs`
  types if needed.
- In `raw/aggregate.rs`: remove `use crate::note::paths::NotePath` if now
  unused.
- In `processor.rs`: ensure `NotePath` is imported where needed for the new
  `New`/`Changed` fields.

### 5.3 Fix `parse_raw` test helper

The test helper in `parser.rs` currently calls:
```rust
MarkdownParser::parse(markdown, path, &task_spec)
```
Update to:
```rust
MarkdownParser::parse(markdown, &task_spec)
```
The path is no longer passed in. If any test asserts on `raw.path`, remove
those assertions (path is no longer in `RawNote`).

### 5.4 Run the full test suite

```
cargo test -p lithos-core
```

Fix any remaining compilation errors. Common sources:
- Call sites of `RawNote::new` with the old `path` argument.
- `Note::try_from` tuple arity.
- Imports for removed types.
- `block.meta.depth` references (field is now `block.kind` for `ListItem` or
  just `self.depth` on the parser).

### 5.5 Remove suppression attributes that are no longer needed

After the refactor, `parse()`, `on_start()`, and `on_end()` should each be
short enough that `#[expect(clippy::too_many_lines)]` is no longer warranted.
Remove any that are now unnecessary. If clippy still fires on `on_start` or
`on_end`, investigate whether a helper method would improve clarity rather than
suppressing.

### 5.6 Run `cargo clippy` and address new lints

The refactor changes a lot of code paths. Run:
```
cargo clippy -p lithos-core -- -D warnings
```
Address any new lints. Do not use `#[expect]` as a first resort.

---

## Quick Reference — What Lives Where After the Refactor

| Item | Location |
|---|---|
| `MarkdownParser` struct + `parse()` | `parser.rs` |
| `step`, `on_start`, `on_end`, `on_text`, `on_code`, `on_task_marker` | `parser.rs` (methods) |
| `finalize_link`, `finalize_metadata` | `parser.rs` (methods) |
| `Block`, `BlockKind`, `LinkFrame`, `ListCtx`, `RefDefs` | `parser.rs` (private types) |
| `StringPool` + metrics | `parser.rs` (unchanged) |
| `normalize_breaks`, `block_ref_tail_range`, `pop_block`, `to_offset`, `depth_to_raw` | `parser.rs` (free functions) |
| `BlockExtractor` struct + `finalize_*` methods | `extractor.rs` |
| `scan_block`, `extend_output`, `compute_item_text`, `scan_task_marker_first_line` | `extractor.rs` (private) |
| `NoteScanner`, `Cursor`, scan rules | `scanner.rs` (unchanged except `scan_task_marker_first_line` added) |
| `RawNote` + `accept_list_item` | `raw/aggregate.rs` |
| `field_token_to_raw` | `raw/inline_field.rs` |
| `ScannedRawArtifacts::fields_into_raw` | `scanner.rs` |
| `NoteProcessor` typestate pipeline | `processor.rs` |
