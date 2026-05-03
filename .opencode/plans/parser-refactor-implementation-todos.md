# Parser Refactor Implementation Todos

## Purpose

Execution checklist for the policy-driven parser refactor.

- Source plan: `.opencode/plans/parser-refactor-policy-and-traversal-plan.md`
- Scope: `lithos-core/src/note/parser/*` (legacy `mod.rs` is compatibility-only; refactor work should target `structure.rs`/`types.rs` unless safety fixes are required)
- Constraint: preserve math events and remove silent drops.

## Execution Rules

- Complete stages strictly in order (`0 -> 9`).
- At each stage boundary:
  - run the listed validation commands,
  - fix failures before moving to the next stage,
  - commit only the stage scope.
- If a stage requires temporary compatibility shims, add explicit follow-up todo in Stage 9 cleanup.

## Pre-Flight Gaps Confirmed From Re-Review

- `config.rs` now has extension + retention modeling, but critical integration gaps remain:
  - retention policy semantics are not yet wired into `stream.rs` unknown-event paths
  - metadata policy round-trip is currently lossy for pluses-only metadata options
  - default profile is defined in multiple places and needs single-source deduplication
- parser config review (subagent) identified additional cleanup/test gaps:
  - broad module-level dead-code suppression in `config.rs` hides useful unused-path signals
  - unknown-event policy diagnostics are too generic for triage/debugging
  - missing integration tests proving unknown-event enforcement in stream mapping
  - missing regression test for metadata round-trip fidelity
  - missing constructor/default coherence tests
- `stream.rs` still silently drops `SoftBreak`, `HardBreak`, `InlineMath`, `DisplayMath`, footnotes, and extension tags in mapper branches.
- `stream.rs` still exposes reverse conversion helpers (`as_start_tag`, `as_end_tag`) via block IR types.
- `block.rs` still imports pulldown types and stream internals.
- `structure.rs` still imports pulldown types and depends on `stream::BlockType`; stack invariants are split and one invalid attach path is silent.
- `visitor.rs` is trait-heavy and pulldown-leaking; `DocTree::walk` is coupled to it.

---

## Stage 0 - Baseline and Safety Rails

- [x] Capture baseline parser behavior with focused tests for current losses:
  - [x] math event currently dropped
  - [x] break behavior under each `BreakPolicy`
  - [x] optional extension tags dropped
- [x] Add/expand parser error variants needed by new policy/stack contract.
- [x] Add a short design note in docs/artifacts describing new parser contract boundaries.

Acceptance criteria:

- Baseline tests exist and explicitly document current behavior vs target behavior.
- Error variants compile and are imported where needed.

Validation:

- `mise run test:unit:note`

---

## Stage 1 - Introduce Policy Layer in `config.rs`

Files:

- `lithos-core/src/note/parser/config.rs`

Tasks:

- [x] Add `CmarkExtensionsPolicy` with explicit fields:
  - [x] `task_lists`
  - [x] `wikilinks`
  - [x] `math`
  - [x] `metadata_blocks`
  - [x] `strikethrough`
  - [x] `tables`
  - [x] `definition_lists`
  - [x] `footnotes`
- [x] Add `EventRetentionPolicy`:
  - [x] `unknown_block`
  - [x] `unknown_inline`
  - [x] `breaks`
- [x] Add typed policy enums (`Enabled/Disabled`, `Preserve/Reject/Degrade` variants as specified).
- [x] Refactor `EventStreamConfig` to carry policy fields.
- [x] Add option synthesis method from policy to pulldown `Options`.
- [x] Define explicit default profile with `math: PreserveAsMathEvents`.
- [x] Keep compatibility constructors if needed for phased migration.

Acceptance criteria:

- Policy model is complete and documented.
- Defaults are deterministic and tested.

Validation:

- `mise run lint`
- `mise run test:unit:note`

---

## Stage 2 - Add Neutral Parser Types Module

Files:

- `lithos-core/src/note/parser/types.rs` (new)
- `lithos-core/src/note/parser/text.rs` (new)
- `lithos-core/src/note/parser/*` imports updated

Tasks:

- [ ] Add parser-owned enums/structs:
  - [x] Add parser-owned enums/structs:
    - [x] `BlockStart`, `BlockEnd`
    - [x] `InlineDelimiterStart`, `InlineDelimiterEnd`, `InlineToken`
    - [x] `ParserEvent`
    - [x] `HeadingLevel`, `ListKind`, `FrontmatterFormat`, `LinkKind`, `LineBreakKind`, `MathKind`
    - [x] `RangedEvent` for event + source range
    - [x] `TextStyle` and `TextNode`/`TextSequence` as derived inline text model in `text.rs`
- [x] Ensure zero `pulldown_cmark::*` references in `types.rs`.
- [x] Define conversion helpers local to adapter boundary only (not in domain types).
- [x] Add unit tests for neutral type invariants and simple constructors.

Acceptance criteria:

- Neutral type layer compiles and is imported by stream/structure/block paths.
- No pulldown leakage in neutral type definitions.

Validation:

- `mise run lint`
- `cargo test -p lithos-core note::parser::types::tests -- --nocapture`
- `cargo test -p lithos-core note::parser::text::tests -- --nocapture`

---

## Stage 3 - Refactor `stream.rs` to Policy-Enforced Mapping

Files:

- `lithos-core/src/note/parser/stream.rs`

Tasks:

- [x] Replace current IR mappings to emit neutral types from `types.rs`.
- [x] Remove reverse conversion methods (`as_start_tag`, `as_end_tag`).
- [x] Replace every silent-drop path with policy-gated behavior:
  - [x] `SoftBreak`
  - [x] `HardBreak`
  - [x] `InlineMath`
  - [x] `DisplayMath`
  - [x] `FootnoteReference`
  - [x] table tags/events
  - [x] definition list tags/events
- [x] Keep reference extraction logic intact.
- [x] Preserve source range mapping for all emitted events.
- [x] Wire `EventRetentionPolicy` enforcement for unknown block/inline events; no silent unknown drop.
- [x] Improve policy violation diagnostics with concrete observed event context.
- [x] Define delimiter normalization contract for inline code/math:
  - [x] decide whether `InlineToken::InlineCode` and `InlineToken::Math` store raw payload or delimiter-stripped payload
  - [x] codify decision in mapper docs and tests
- [x] Restrict `TextMergeWithOffset` usage when policies require boundary-sensitive retention.
  - [x] Add explicit guard/tests for math boundary safety.

Acceptance criteria:

- No enabled extension is silently dropped.
- Mapping outcomes are deterministic under policy.

Validation:

- `mise run test:unit:note`
- Focused stream tests for each policy profile
- `cargo test -p lithos-core note::parser::stream::tests::event_adapter_iter_extension_drop_baseline -- --nocapture`

---

## Stage 4 - Decouple `block.rs` From Pulldown and Stream Internals

Files:

- `lithos-core/src/note/parser/block.rs`

Tasks:

- [x] Remove pulldown imports (`CowStr`, `MetadataBlockKind`).
- [x] Remove direct dependency on `stream::{ParserEvent, InlineEvent, EventWithRange}`.
- [x] Switch to parser-owned `types::{ParserEvent, InlineToken, RangedEvent}` and `text` projection as needed.
- [x] Replace leaf payload types:
  - [x] code language -> `Option<Box<str>>`
  - [x] frontmatter format -> `FrontmatterFormat`
- [x] Replace heading conversion impl from pulldown with parser-owned conversion at adapter boundary.
- [x] Introduce AST-owned inline representation strategy:
  - [x] Option A: store `InlineToken` vectors in leaf blocks (CURRENT CHOICE)
  - [ ] Option B: store materialized text payloads with optional rich inline storage
- [x] Update `block.text()` and related helpers to use AST-owned structures only.

Acceptance criteria:

- `block.rs` has zero `pulldown_cmark` imports.
- `block.rs` has zero direct `stream` IR coupling.

Validation:

- `mise run lint`
- `mise run test:unit:note`

---

## Stage 5 - Rework Structure Builder Invariants (`structure.rs`)

Precondition note (must complete before Stage 5 invariants work):

- Parser text projection is still split across legacy fragment flow and
  `text.rs`.
- Implement the canonical `text.rs` migration slice below first, then proceed
  to Stage 5 topology/invariant hardening.

### Stage 4.5 - Canonical Text Projection Migration (`text.rs`)

Files:

- `lithos-core/src/note/parser/text.rs`
- `lithos-core/src/note/parser/block.rs`
- `lithos-core/src/note/parser/structure.rs`
- `lithos-core/src/note/parser/mod.rs`
- `lithos-core/src/note/extractor.rs`
- `lithos-core/src/note/parser/context.rs` (tests/doc alignment)

Tasks:

- [x] Promote `text.rs` to canonical projection engine:
  - [x] `TextSequence::from_events(&[RangedEvent])`
  - [x] style stack handling from delimiter start/end tokens
  - [x] `as_plain_text()` and source-covering range helper
- [x] Remove legacy parser fragment path from `mod.rs`:
  - [x] remove `TextFragment`
  - [x] remove `FragmentPool`
  - [x] store and propagate parser events/ranges for leaf completion
- [x] Update sink API to consume event-derived projection input (not fragments).
- [x] Keep `text.rs` policy-agnostic; scannable filtering owned by extractor/scanner boundary.
- [x] Keep parser IR authoritative in `types.rs`; no pulldown leakage.
- [x] Remove/retire `Block::text()` and `Block::is_scannable()` if no longer
      needed by callers.
- [x] Stage 4.5b projection model cleanup:
  - [x] remove `ProjectionPolicy` from `text.rs` (consumer-owned filtering)
  - [x] keep `TextNode`/`TextSequence` names and add `TextContext`
  - [x] represent code/math as `TextStyle` variants
  - [x] move scanner/link inclusion decisions to extractor/parser boundaries
  - [x] fix empty projection fallback range to block-anchored span
  - [x] fix scanner disjoint-range lexical boundary semantics (no cross-range mode/prev_alnum carry)

Acceptance criteria:

- Single source of truth for text projection exists in `text.rs`.
- No duplicated text/scannable logic remains in parser/extractor paths.
- Heading/paragraph/list/frontmatter extraction behavior remains unchanged.

Validation:

- `mise run test:unit:note`
- `cargo test -p lithos-core note::parser:: -- --nocapture`

Files:

- `lithos-core/src/note/parser/mod.rs`
- `lithos-core/src/note/parser/structure.rs`

Tasks:

- [x] Stage 5a parser/topology correctness hardening (production path first):
  - [x] remove silent frame-role mismatch no-ops in `mod.rs` finalization
  - [x] return typed topology mismatch errors (`EventStackMismatch`) in `mod.rs`
  - [x] populate mismatch diagnostics with meaningful payload fields (`depth`, labels)
  - [x] remove silent attach-to-leaf no-op in `structure.rs`
  - [x] return typed topology/underflow errors with range context in `structure.rs`
  - [x] add focused failing-then-passing regression tests for mismatch paths
- [x] Remove pulldown imports.
- [x] Consume only neutral `types.rs` events/tags.
- [x] Keep `ProcessingContainer`, but redesign with variant payload structs:
  - [x] `ProcessingBlockQuote`
  - [x] `ProcessingList`
  - [x] `ProcessingListItem`
- [ ] Refactor `ProcessingLeaf` similarly if needed for symmetry and invariants.
      (deferred: optional, not required for Stage 5 completion)
- [x] Split depth tracking:
  - [x] `list_depth`
  - [x] ensure non-list nesting does not mutate list hierarchy depth
- [x] Move structural correctness into `ProcessingBlockTree`:
  - [x] legal `start_block` (no-start-inside-leaf enforcement)
  - [x] exact `end_block` matching (reuse `types::BlockEnd` semantics)
  - [x] legal parent-child attach rules
  - [x] no silent no-op on invalid attach
- [x] Ensure end-tag mismatch yields typed errors with range context.

Acceptance criteria:

- [x] Structure builder fails fast on topology violations.
- [x] No silent attach failures remain.

**Stage 5: COMPLETE**

Validation:

- `mise run test:unit:note`

---

## Stage 6 - Replace Visitor Trait With Traversal Iterator/Callback API`

Files:

- `lithos-core/src/note/parser/visitor.rs` (deleted)
- `lithos-core/src/note/parser/structure.rs` traversal methods

Tasks:

- [x] Implement `walk_preorder()` iterator API returning traversal events.
- [x] Add convenience traversal helpers:
  - [x] `for_each_block`
  - [x] optional typed filters
- [x] Update structure docs/examples to use iterator/callback model.
- [x] Remove pulldown types from traversal API.
- [x] Decide whether to:
  - [x] delete `BlockVisitor` trait entirely, or
  - [x] keep temporary adapter shim with deprecation note.
- [x] Port existing visitor tests to traversal event tests.

Acceptance criteria:

- Traversal semantics (pre-order, depth) preserved.
- Technical debt from default no-op trait methods removed or isolated to temporary adapter.

Validation:

- `mise run test:unit:note`

---

## Stage 7 - Structure/Text Contract Hardening

Files:

- `lithos-core/src/note/parser/structure.rs`
- `lithos-core/src/note/parser/text.rs`
- `lithos-core/src/note/parser/context.rs`

Tasks:

- [x] Add contract tests for `TextContext` + `TextStyle` filtering boundaries:
  - [x] scanner excludes link/image/code/math nodes by intended rules
  - [x] link display includes intended label/code nodes and excludes math
- [x] Add range-contract tests for `TextSequence::covering_range()` in nested/mixed inline scenarios.
- [x] Verify no duplicated projection/filtering logic drifts back into parser/extractor.
- [x] Add malformed-event topology tests that assert typed errors (no stringly fallbacks).

Acceptance criteria:

- Projection/scanning contracts are enforced by tests and remain policy-consistent.
- No silent topology/policy regressions pass parser test gates.

Validation:

- new unit tests for nested style combinations and range correctness

---

## Stage 8 - Capability Matrix and Contract Tests

Files:

- `lithos-core/src/note/parser/stream.rs` tests
- `lithos-core/src/note/parser/structure.rs` tests
- `lithos-core/src/note/parser/context.rs` tests
- `.opencode/plans/parser-capability-matrix.md` (new)

Tasks:

- [x] Add matrix doc rows for each feature:
  - [x] option enabled state
  - [x] stream IR behavior
  - [x] structure behavior
  - [x] scanner/semantic expectation
  - [x] unsupported fallback/error path
- [x] Add automated tests per matrix row.

Acceptance criteria:

- Any new enabled feature without handler fails tests.
- No future silent drops can pass CI.

Validation:

- `mise run test:unit:note`
- `mise run lint`

---

## Stage 9 - Final Integration and Quality Gates

### Phase 1: Extend BlockExtractor for DocTree

Files modified:
- `lithos-core/src/note/extractor.rs` - Rewrite to use `DocTree::for_each_block()`
- `lithos-core/src/note/parser/mod.rs` - Update `MarkdownParser::parse()` to build `DocTree`
- `lithos-core/src/note/scanner.rs` - No changes needed (scanner API stays the same)

Detailed analysis:

**How current `BlockExtractor` works:**
- Implements `ArtifactSink` trait with 3 methods: `on_container_complete()`, `on_leaf_complete()`, `on_link()`
- `process_leaf()` receives `LeafKind`, `BlockSpan`, `&[RangedEvent]`, depth
- Uses `TextSequence::from_events()` to project events into scannable text
- Scans text for tags, inline fields, block refs via `NoteScanner`
- Creates `RawSection` entries based on `LeafKind` variant
- `on_container_complete()` handles `ContainerKind::ListItem` with `ListItemPayload` (depth, parent_pos, is_checkbox)

**What must change for DocTree:**
- `DocTree::for_each_block()` passes `&Block<'source, Closed>` and `u32` depth
- New block types: `BlockKind<Closed>` with variants:
  - `Leaf(LeafBlockKind)` where `LeafBlockKind` is:
    - `Paragraph { events: Vec<RangedEvent> }` (in Open state)
    - `Heading { level, events: Vec<RangedEvent> }` (in Open state)
    - `CodeBlock { language, text: String }` (in Closed state - text is materialized)
    - `Frontmatter { format, text: String }` (in Closed state - text is materialized)
    - `ThematicBreak`
  - `Container(ContainerBlockKind)` where `ContainerBlockKind` is:
    - `BlockQuote { children }`
    - `List { kind, children }`
    - `ListItem { depth, parent_pos, is_checked, children }` (NOTE: `is_checked` not `is_checkbox`)

**Key type mapping (old → new):**
| Old Type | New Type | Notes |
|-----------|----------|-------|
| `LeafKind::Metadata(MetadataPayload { format })` | `LeafBlockKind::Frontmatter { format, text }` | `MetadataPayload.format` → `FrontmatterFormat` (same) |
| `LeafKind::Heading(HeadingPayload { level })` | `LeafBlockKind::Heading { level, events }` | `HeadingPayload.level` → `HeadingLevel` (same) |
| `LeafKind::Paragraph` | `LeafBlockKind::Paragraph { events }` | Paragraph now carries events |
| `LeafKind::ThematicBreak` | `LeafBlockKind::ThematicBreak` | Direct mapping |
| `ContainerKind::List` | `ContainerBlockKind::List { kind, children }` | List now carries children |
| `ContainerKind::ListItem(ListItemPayload { kind, depth, parent_pos, is_checkbox })` | `ContainerBlockKind::ListItem { depth, parent_pos, is_checked, children }` | **CRITICAL**: `is_checkbox` → `is_checked`, no `kind` field in new type |
| `ContainerKind::BlockQuote` | `ContainerBlockKind::BlockQuote { children }` | BlockQuote now carries children |
| `ContainerKind::CodeBlock` | `LeafBlockKind::CodeBlock { language, text }` | **NOTE**: CodeBlock is a LEAF in new model, not container |
| `BlockSpan { start, end }` | `SourceByteRange` (via `block.span`) | `Block::<Closed>.span` is `SourceByteRange` |

**Link extraction without PDA's `ActiveLink` state:**
- Old way: PDA tracked `link: Option<ActiveLink>` with `events: Vec<RangedEvent>`, then called `link_display_from_events()`
- New way: Links are extracted from `DocTree` traversal or from `ParserContext::references()`
- `RawLink` creation must happen in the new extraction path
- `RawLinkStyle::from(LinkKind)` conversion is already defined
- Reference resolution: `resolve_reference_target()` function at `mod.rs:804` must be preserved or moved

**What happens to `process_leaf()` with `LeafBlockKind::Paragraph` + `events: Vec<RangedEvent>`:**
- Same as before! The events are still `Vec<RangedEvent>` (in Open state)
- `TextSequence::from_events(&events)` still works
- `scannable_ranges_from_projection()` still works
- Only change: events come from `LeafBlockKind::Paragraph { events }` instead of `&[RangedEvent]` parameter

**What happens to tag/inline-field/block-ref scanning with `ParserEvent` tokens:**
- `NoteScanner::scan_ranges()` takes `&str`, `&[Range<usize>]`, and produces `ScannedRawArtifacts`
- Scanner does NOT use `ParserEvent` directly - it uses raw text + byte ranges
- Scanner API does NOT need to change
- Only the way we compute "scannable ranges" might change (via `TextSequence::from_events()`)

**Impact on `NoteScanner`:**
- `NoteScanner` does NOT need to change - it works on raw text + ranges
- The scanner is already decoupled from parser IR
- `scan_ranges()` method stays the same

Tasks:

- [ ] **1.1** Update `BlockExtractor` to iterate `DocTree` instead of implementing `ArtifactSink`:
  - File: `lithos-core/src/note/extractor.rs`
  - Remove `use super::parser::{ArtifactSink, BlockSpan, ContainerKind, LeafKind, ...}`
  - Add `use super::parser::structure::{Block, BlockKind, ContainerBlockKind, LeafBlockKind, DocTree}`
  - Remove `impl<'source> ArtifactSink<'source> for BlockExtractor<'source>`
  - Add new method `fn process_doc_tree(&mut self, tree: &DocTree<'source, Complete>) -> Result<(), NoteIngestError>`
  - Use `tree.for_each_block(|block, depth| { ... })` pattern
  - Handle each `BlockKind` variant:
    - `Leaf(LeafBlockKind::Paragraph { events })` → same as old `LeafKind::Paragraph` path
    - `Leaf(LeafBlockKind::Heading { level, events })` → same as old `LeafKind::Heading` path (convert `level` via `.as_u8()`)
    - `Leaf(LeafBlockKind::Frontmatter { format, text })` → use `text` directly (already materialized in Closed state)
    - `Leaf(LeafBlockKind::ThematicBreak)` → same as old `LeafKind::ThematicBreak`
    - `Leaf(LeafBlockKind::CodeBlock { language, text })` → may need section entry (currently CodeBlock is NOT in `RawSectionKind` - check this!)
    - `Container(ContainerBlockKind::ListItem { depth, parent_pos, is_checked, children })` → handle list item
    - `Container(ContainerBlockKind::List { .. })` → no-op (just a container)
    - `Container(ContainerBlockKind::BlockQuote { .. })` → push section entry
  - Downstream effect: `RawNote` section kinds updated
  - Test update: existing extractor tests must be rewritten for `DocTree` input

- [ ] **1.2** Map old `LeafKind`/`ContainerKind` variants to new `BlockKind` variants in extraction logic:
  - File: `lithos-core/src/note/extractor.rs` (inside `process_doc_tree`)
  - `LeafKind::Metadata(MetadataPayload { format })` → `LeafBlockKind::Frontmatter { format, text }`
    - `format` mapping: `FrontmatterFormat::Yaml` → `RawFrontmatterFormat::Yaml`, `FrontmatterFormat::Toml` → `RawFrontmatterFormat::Toml`
    - `text` is already `String` in Closed state
  - `LeafKind::Heading(HeadingPayload { level })` → `LeafBlockKind::Heading { level, events }`
    - Convert `level` to `u8` via `level.as_u8()` (was `payload.to_u8()` which called `HeadingPayload::to_u8()`)
    - `events` used for scanning (same as before)
  - `LeafKind::Paragraph` → `LeafBlockKind::Paragraph { events }`
    - `events` used for scanning (same as before)
  - `LeafKind::ThematicBreak` → `LeafBlockKind::ThematicBreak`
    - No change in behavior
  - `ContainerKind::ListItem(ListItemPayload { kind, depth, parent_pos, is_checkbox })` → `ContainerBlockKind::ListItem { depth, parent_pos, is_checked, children }`
    - **CRITICAL**: `is_checkbox` → `is_checked` (field rename)
    - **CRITICAL**: `kind` field is GONE in new type - `RawListKind` must be derived differently
    - `depth` in new type is `u32` (was `RawListDepth` in old - need conversion `RawListDepth::from(depth)`)
    - `parent_pos` is `Option<SourceByteOffset>` (same)
  - `ContainerKind::List` → `ContainerBlockKind::List { kind, children }`
    - No direct extraction needed (just a container)
  - `ContainerKind::BlockQuote` → `ContainerBlockKind::BlockQuote { children }`
    - Push `RawSection::new(RawSectionKind::BlockQuote, range, depth)`
  - Downstream effect: `RawSectionKind` mapping table updated
  - Test update: unit tests for each variant mapping

- [ ] **1.3** Handle `ListItemPayload` → `ContainerBlockKind::ListItem` field differences:
  - File: `lithos-core/src/note/extractor.rs`
  - Old `ListItemPayload` had: `kind: RawListKind`, `depth: RawListDepth`, `parent_pos: Option<SourceByteOffset>`, `is_checkbox: Option<bool>`
  - New `ContainerBlockKind::ListItem` has: `depth: u32`, `parent_pos: Option<SourceByteOffset>`, `is_checked: Option<bool>`, `children: Vec<Block>`
  - **Field mapping**:
    - `kind: RawListKind` - GONE (must be obtained from parent `List` block's `kind` field)
    - `depth: RawListDepth` → `depth: u32` (use `RawListDepth::from(depth)` for `RawListItem` construction)
    - `parent_pos` - same type, direct mapping
    - `is_checkbox: Option<bool>` → `is_checked: Option<bool>` (just a rename)
  - **To get `RawListKind` for list items**: Need to traverse up to parent `List` block in `DocTree`
    - Option A: Pass `RawListKind` down during traversal (complex)
    - Option B: Store `RawListKind` in `ContainerBlockKind::ListItem` (modifies `block.rs`)
    - Option C: Get `RawListKind` from the `List` ancestor during `for_each_block` traversal
    - **Recommended**: Option C - during `for_each_block`, track current list kind in traversal state
  - Downstream effect: `RawListItem::new()` signature may need update
  - Test update: list item extraction tests

- [ ] **1.4** Handle link extraction without PDA's `ActiveLink` state:
  - File: `lithos-core/src/note/extractor.rs`
  - Old way: `MarkdownParser` had `link: Option<ActiveLink>` field, tracked link events, called `link_display_from_events()`, used `resolve_reference_target()`
  - New way: Links need to be extracted from `DocTree` traversal by scanning for `InlineToken::DelimiterStart(InlineDelimiterStart::Link { .. })` and `InlineToken::DelimiterStart(InlineDelimiterStart::Image { .. })` in leaf block events
  - Need to reconstruct `RawLink` from inline events (similar to `link_display_from_events()` logic)
  - `RawLink::new()` needs: `style: RawLinkStyle`, `is_embed: bool`, `target: Cow<'source, str>`, `start: SourceByteOffset`
  - `style` from `LinkKind` via `RawLinkStyle::from(LinkKind)`
  - `is_embed` from `LinkKind::WikiLink { has_pothole }` (was `LinkKind::WikiLink` in old code at `mod.rs:228`)
  - `target` from `destination` field of `InlineDelimiterStart::Link`
  - `start` from `RangedEvent` range
  - Preserve `resolve_reference_target()` function (moved from `mod.rs` to `extractor.rs` or a shared location)
  - Downstream effect: `RawNote::links` still populated correctly
  - Test update: link extraction tests from `mod.rs` tests need to be ported

- [ ] **1.5** Update `RawSectionKind` mapping for new block types:
  - File: `lithos-core/src/note/raw/section.rs` (may need new variants)
  - Current `RawSectionKind` variants: `Heading`, `Paragraph`, `CodeBlock`, `BlockQuote`, `List`, `Frontmatter`, `ThematicBreak`
  - New mapping needed:
    - `LeafBlockKind::Paragraph { .. }` → `RawSectionKind::Paragraph` ✓ (exists)
    - `LeafBlockKind::Heading { .. }` → `RawSectionKind::Heading` ✓ (exists)
    - `LeafBlockKind::Frontmatter { .. }` → `RawSectionKind::Frontmatter` ✓ (exists)
    - `LeafBlockKind::ThematicBreak` → `RawSectionKind::ThematicBreak` ✓ (exists)
    - `LeafBlockKind::CodeBlock { .. }` → `RawSectionKind::CodeBlock` ✓ (exists)
    - `ContainerBlockKind::List { .. }` → `RawSectionKind::List` ✓ (exists)
    - `ContainerBlockKind::BlockQuote { .. }` → `RawSectionKind::BlockQuote` ✓ (exists)
    - `ContainerBlockKind::ListItem { .. }` → `RawSectionKind::List` (list items are sections of kind `List`)
  - **No new `RawSectionKind` variants needed**
  - Downstream effect: None (all mappings exist)
  - Test update: None needed

### Phase 2: Handle Type Mismatches

Files modified:
- `lithos-core/src/note/extractor.rs` - Update all pattern matches
- `lithos-core/src/note/parser/mod.rs` - Remove old types
- `lithos-core/src/note/raw/aggregate.rs` - No changes (already uses new types)

Tasks:

- [ ] **2.1** Map EVERY old `LeafKind` variant to new `LeafBlockKind`:
  - File: `lithos-core/src/note/extractor.rs`
  - `LeafKind::Metadata(MetadataPayload { format })` → `LeafBlockKind::Frontmatter { format, text }`
    - Old: `payload.format` (type `FrontmatterFormat`)
    - New: `format` (type `FrontmatterFormat`) - SAME TYPE, direct mapping
    - Old: `projection.as_plain_text()` to get text (events were `Vec<RangedEvent>`)
    - New: `text` field is already `String` (materialized in `Block::close()`)
    - Change: Use `text.clone()` instead of `projection.as_plain_text()`
  - `LeafKind::Heading(HeadingPayload { level })` → `LeafBlockKind::Heading { level, events }`
    - Old: `payload.to_u8()` to get `u8` level
    - New: `level.as_u8()` (same method on `HeadingLevel`)
    - Old: `events: &[RangedEvent]` parameter
    - New: `events: &Vec<RangedEvent>` from the struct field
    - Change: `let projection = TextSequence::from_events(events)` (same)
  - `LeafKind::Paragraph` → `LeafBlockKind::Paragraph { events }`
    - Old: `events: &[RangedEvent]` parameter
    - New: `events: &Vec<RangedEvent>` from the struct field
    - Change: `let projection = TextSequence::from_events(events)` (same)
  - `LeafKind::ThematicBreak` → `LeafBlockKind::ThematicBreak`
    - No events, no scanning, just push section
    - No change in behavior
  - Downstream effect: All `process_leaf()` pattern matches rewritten
  - Test update: All existing tests for paragraph/heading/frontmatter/thematic-break

- [ ] **2.2** Map EVERY old `ContainerKind` variant to new `ContainerBlockKind`:
  - File: `lithos-core/src/note/extractor.rs`
  - `ContainerKind::List` → `ContainerBlockKind::List { kind, children }`
    - Old: No extraction (just a container)
    - New: No extraction (just a container) - **NO CHANGE**
  - `ContainerKind::ListItem(ListItemPayload { kind, depth, parent_pos, is_checkbox })` → `ContainerBlockKind::ListItem { depth, parent_pos, is_checked, children }`
    - **CRITICAL FIELD MAPPING**:
      - `kind: RawListKind` - GONE (need to get from parent `List` block)
      - `depth: RawListDepth` → `depth: u32` (use `RawListDepth::from(depth)` for `RawListItem::new()`)
      - `parent_pos: Option<SourceByteOffset>` - SAME
      - `is_checkbox: Option<bool>` → `is_checked: Option<bool>` (RENAMED)
    - **To reconstruct `RawListKind`**: Need to get `kind` from parent `List` block
      - During `for_each_block` traversal, maintain a stack of `RawListKind` for nested lists
      - When entering `ContainerBlockKind::List { kind, .. }`, push `kind` onto stack
      - When entering `ContainerBlockKind::ListItem`, pop/get `RawListKind` from stack
    - **Extraction logic rewrite**:
      ```rust
      // Old:
      ContainerKind::ListItem(payload) => {
          let scanned = self.scan_projection(&projection)?;
          let (raw_text, text_range) = projection_text_and_range(&projection, range)?;
          let item = RawListItem::new(
              payload.kind,        // RawListKind
              payload.depth,      // RawListDepth
              payload.parent_pos,
              payload.is_checkbox, // Option<bool>
              RawListItemText::new(Cow::Owned(raw_text), text_range),
              range,
              scanned.tags,
              scanned.inline_fields,
          );
      }
      ```
      ```rust
      // New:
      ContainerBlockKind::ListItem { depth, parent_pos, is_checked, children } => {
          // Get RawListKind from traversal state
          let list_kind = /* from traversal state */;
          let scanned = self.scan_projection_from_children(children)?;
          // ... construct RawListItem with depth.into() for RawListDepth
      }
      ```
  - `ContainerKind::BlockQuote` → `ContainerBlockKind::BlockQuote { children }`
    - Old: `self.out.sections.push(RawSection::new(RawSectionKind::BlockQuote, range, depth))`
    - New: Same, but get `range` from `block.span` (type `SourceByteRange`)
    - **NO CHANGE** in extraction logic (just section push)
  - `ContainerKind::CodeBlock` → `LeafBlockKind::CodeBlock { language, text }` (NOTE: CodeBlock is LEAF in new model!)
    - CodeBlock in old model was a `ContainerKind` (?? This seems wrong - checking `mod.rs:332` shows `ContainerKind::CodeBlock`)
    - Actually in old code (`mod.rs:332`): `ContainerKind::CodeBlock` - but CodeBlock has no children!
    - In new `block.rs`: `LeafBlockKind::CodeBlock { language, text }` - correct (CodeBlock is a leaf)
    - **MIGRATION**: Old `on_container_complete(ContainerKind::CodeBlock, ...)` → New `for_each_block` with `LeafBlockKind::CodeBlock { .. }`
    - Old extraction: Just push `RawSection::new(RawSectionKind::CodeBlock, range, depth)`
    - New extraction: Same - just push section
  - Downstream effect: `RawNote::sections` updated correctly
  - Test update: List item tests, block quote tests, code block tests

- [ ] **2.3** Identify extraction logic that relies on old field names:
  - File: `lithos-core/src/note/extractor.rs`
  - Search for `payload.kind` → Change to getting `RawListKind` from traversal state
  - Search for `payload.depth` (type `RawListDepth`) → Change to `depth: u32` and convert with `RawListDepth::from(depth)`
  - Search for `payload.parent_pos` → No change (same type)
  - Search for `payload.is_checkbox` → Change to `is_checked` (rename)
  - Search for `leaf_span` or `BlockSpan` → Change to `block.span` (type `SourceByteRange`)
  - Downstream effect: All field accesses updated
  - Test update: Regression tests for each field mapping

- [ ] **2.4** Handle `ListItemPayload` fields (depth, parent_pos, is_checkbox) in `ContainerBlockKind::ListItem`:
  - File: `lithos-core/src/note/parser/block.rs` (type definition)
  - File: `lithos-core/src/note/extractor.rs` (consumption)
  - **`depth` field**:
    - Old: `ListItemPayload.depth: RawListDepth`
    - New: `ContainerBlockKind::ListItem.depth: u32`
    - Conversion for `RawListItem::new()`: `RawListDepth::from(depth)` (where `depth` is `u32`)
    - `RawListDepth::from(u32)` already exists (check `raw/` module)
  - **`parent_pos` field**:
    - Old: `ListItemPayload.parent_pos: Option<SourceByteOffset>`
    - New: `ContainerBlockKind::ListItem.parent_pos: Option<SourceByteOffset>`
    - **No conversion needed** - same type
  - **`is_checkbox` → `is_checked` field**:
    - Old: `ListItemPayload.is_checkbox: Option<bool>`
    - New: `ContainerBlockKind::ListItem.is_checked: Option<bool>`
    - **Just a rename** - update all accesses
    - Old code at `mod.rs:363`: `is_checkbox: None` → New: `is_checked: None`
    - Old code at `mod.rs:515`: `payload.is_checkbox = Some(checked)` → New: `is_checked = Some(checked)`
    - Old code at `extractor.rs:183`: `payload.is_checkbox` → New: `is_checked`
  - **`kind` field (REMOVED)**:
    - Old: `ListItemPayload.kind: RawListKind`
    - New: `ContainerBlockKind::ListItem` does NOT have `kind` field
    - Need to get `RawListKind` from parent `List` block during traversal
    - See Phase 1, Task 1.3 for solution
  - Downstream effect: `RawListItem` construction updated
  - Test update: List item tests with `RawListKind` verification

- [ ] **2.5** Update `RawSectionKind` mapping:
  - File: `lithos-core/src/note/raw/section.rs` (type definition)
  - File: `lithos-core/src/note/extractor.rs` (consumption)
  - Current `RawSectionKind` enum (in `raw/section.rs`):
    ```rust
    pub enum RawSectionKind {
        Heading,
        Paragraph,
        CodeBlock,
        BlockQuote,
        List,
        Frontmatter,
        ThematicBreak,
    }
    ```
  - **All variants still valid** - no new variants needed
  - Mapping table (new → `RawSectionKind`):
    - `LeafBlockKind::Paragraph { .. }` → `RawSectionKind::Paragraph` ✓
    - `LeafBlockKind::Heading { .. }` → `RawSectionKind::Heading` ✓
    - `LeafBlockKind::Frontmatter { .. }` → `RawSectionKind::Frontmatter` ✓
    - `LeafBlockKind::ThematicBreak` → `RawSectionKind::ThematicBreak` ✓
    - `LeafBlockKind::CodeBlock { .. }` → `RawSectionKind::CodeBlock` ✓
    - `ContainerBlockKind::List { .. }` → `RawSectionKind::List` ✓
    - `ContainerBlockKind::ListItem { .. }` → `RawSectionKind::List` ✓ (list items are sub-sections of List)
    - `ContainerBlockKind::BlockQuote { .. }` → `RawSectionKind::BlockQuote` ✓
  - **No changes needed to `RawSectionKind` enum**
  - Downstream effect: None
  - Test update: None

### Phase 3: Rewrite MarkdownParser::parse()

Files modified:
- `lithos-core/src/note/parser/mod.rs` - Rewrite `parse()` and remove `parse_with_sink()`
- `lithos-core/src/note/processor.rs` - No changes (uses `MarkdownParser::parse()` which stays)

Tasks:

- [ ] **3.1** Implement new `MarkdownParser::parse()` using `ParserContext` + `DocTree`:
  - File: `lithos-core/src/note/parser/mod.rs`
  - Current `parse()` (lines 628-643):
    ```rust
    pub fn parse(
        source: &'source str,
        task_spec: &TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let sink = BlockExtractor::new(source, scanner);
        Self::parse_with_sink(source, task_spec, sink)
            .map(BlockExtractor::finish)
    }
    ```
  - New `parse()` implementation:
    ```rust
    pub fn parse(
        source: &'source str,
        task_spec: &TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        // 1. Create ParserContext (eagerly parses and caches events)
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)?;

        // 2. Build DocTree from context
        let tree = DocTree::from_context(&ctx)?;

        // 3. Create BlockExtractor with scanner
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let mut extractor = BlockExtractor::new(source, scanner);

        // 4. Process DocTree
        extractor.process_doc_tree(&tree)?;

        // 5. Finish and return RawNote
        Ok(extractor.finish())
    }
    ```
  - **Downstream effect**: `MarkdownParser` struct can be simplified or removed
  - **API compatibility**: `MarkdownParser::parse()` signature stays the same (`source: &str, task_spec: &TaskConfigSpec`) → `Result<RawNote, NoteIngestError>`
  - Test update: `processor.rs:458` calls `MarkdownParser::parse(&self.status.content, task_spec)` - NO CHANGE needed (same signature)

- [ ] **3.2** Remove `parse_with_sink()` method:
  - File: `lithos-core/src/note/parser/mod.rs` (lines 145-162)
  - Current `parse_with_sink()`:
    ```rust
    pub(crate) fn parse_with_sink(
        source: &'source str,
        task_spec: &TaskConfigSpec,
        sink: S,
    ) -> Result<S, NoteIngestError> {
        let stream_config = config::EventStreamConfig::default();
        let (stream, ref_defs) =
            stream::MarkdownEventStream::new(source, stream_config);
        let mut parser = Self::new(source, task_spec, ref_defs, sink);
        for event in stream {
            let event = event?;
            parser.step_spanned(&event)?;
        }
        Ok(parser.sink)
    }
    ```
  - **Can be removed** after `parse()` is rewritten to use `DocTree`
  - **But**: Tests in `mod.rs` use `parse_raw()` helper which calls `MarkdownParser::parse()` (not `parse_with_sink()` directly)
  - Check: Is `parse_with_sink()` called anywhere else?
    - Search result: Only called from `parse()` at line 640
  - **Safe to remove** after `parse()` is rewritten
  - Downstream effect: `MarkdownParser` struct no longer needs `S: ArtifactSink<'source>` generic parameter
  - Test update: Remove `parse_with_sink()` tests (if any)

- [ ] **3.3** Check tests that use `parse_with_sink()`:
  - File: `lithos-core/src/note/parser/mod.rs` (tests module, lines 830-1239)
  - Search for `parse_with_sink` in tests:
    - No direct calls to `parse_with_sink()` in tests
    - Tests use `parse_raw()` helper (line 886-889) which calls `MarkdownParser::parse()`
    - `NoopSink` struct (lines 840-865) implements `ArtifactSink` - can be removed after PDA removal
    - Tests that create `MarkdownParser::new()` directly (lines 974-981, 1005-1006, 1100-1101) - these test PDA internals, must be removed
  - **Tests to update/remove**:
    - `finalize_leaf_frame_rejects_container_topology_mismatch` (line 973) - tests PDA `finalize_leaf_frame()` → REMOVE
    - `finalize_container_frame_rejects_leaf_topology_mismatch` (line 996) - tests PDA `finalize_container_frame()` → REMOVE
    - `reference_definitions_first_wins` (line 1120) - tests `parse_raw()` → KEEP (uses new `parse()`)
    - `reference_definitions_are_case_insensitive` (line 1132) - tests `parse_raw()` → KEEP
    - ... (all tests that use `parse_raw()` helper → KEEP)
    - Tests that directly manipulate `MarkdownParser` PDA fields (`stack`, `link`, etc.) → REMOVE
  - Downstream effect: Test count decreases (PDA tests removed)
  - Test update: Remove PDA-internal tests, keep behavior tests

- [ ] **3.4** Update `MarkdownParser` struct definition:
  - File: `lithos-core/src/note/parser/mod.rs` (lines 117-133)
  - Current struct:
    ```rust
    pub struct MarkdownParser<'source, S>
    where
        S: ArtifactSink<'source>,
    {
        ref_defs: references::ReferenceDefinitions,
        stack: BlockStack<'source>,
        depth: u32,
        link: Option<ActiveLink<'source>>,
        list_kinds: Vec<RawListKind>,
        open_items: Vec<usize>,
        sink: S,
    }
    ```
  - **New struct** (after PDA removal):
    - Option A: Keep `MarkdownParser` as a marker/utility struct (no fields needed for `parse()`)
    - Option B: Remove `MarkdownParser` entirely, make `parse()` a free function
    - **Recommended**: Option A - keep `MarkdownParser` as a utility struct for API compatibility, but it can be empty or contain only `PhantomData`
    ```rust
    pub struct MarkdownParser {
        _marker: std::marker::PhantomData<()>,
    }
    // OR just remove it and use free function:
    pub fn parse_markdown<'source>(
        source: &'source str,
        task_spec: &TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        // ... implementation
    }
    ```
    - **But**: `processor.rs:458` calls `MarkdownParser::parse(...)` - if we remove `MarkdownParser`, need to update `processor.rs`
    - **Decision**: Keep `MarkdownParser` as an empty struct for API compatibility, or update `processor.rs` to call free function
    - **Preferred**: Update `processor.rs` to call free function `parse_markdown()` instead of `MarkdownParser::parse()`
  - Downstream effect: `processor.rs` updated
  - Test update: None (tests use `parse_raw()` helper which will be updated)

### Phase 4: Remove Legacy PDA Code

Files modified:
- `lithos-core/src/note/parser/mod.rs` - Remove ~600 lines of PDA code
- `lithos-core/src/note/parser/block.rs` - Already migrated (no PDA code)
- `lithos-core/src/note/parser/structure.rs` - Already migrated (new DocTree)
- `lithos-core/src/note/extractor.rs` - Already migrated (no `ArtifactSink` impl)

Tasks:

- [ ] **4.1** List EVERY struct/enum/function in `mod.rs` that can be deleted:
  - File: `lithos-core/src/note/parser/mod.rs`
  - **Structs to delete**:
    - `BlockStack<'source>` (lines 686-734) - PDA stack
    - `BlockFrame<'source>` (lines 737-748) - PDA stack frame
    - `ActiveLink<'source>` (lines 572-575) - PDA link tracking
    - `MetadataPayload` (lines 770-772) - OLD leaf payload (replaced by `LeafBlockKind::Frontmatter`)
    - `HeadingPayload` (lines 776-791) - OLD leaf payload (replaced by `LeafBlockKind::Heading`)
    - `ListItemPayload` (lines 795-800) - OLD container payload (replaced by `ContainerBlockKind::ListItem`)
  - **Enums to delete**:
    - `LeafKind` (lines 752-757) - OLD enum (replaced by `LeafBlockKind`)
    - `ContainerKind` (lines 760-766) - OLD enum (replaced by `ContainerBlockKind`)
  - **Traits to delete**:
    - `ArtifactSink<'source>` (lines 648-666) - OLD trait (replaced by `DocTree::for_each_block`)
  - **Functions to delete**:
    - `MarkdownParser::new()` (lines 267-282) - PDA constructor
    - `MarkdownParser::step_spanned()` (lines 164-199) - PDA step function
    - `MarkdownParser::on_block_start()` (lines 284-336) - PDA block start handler
    - `MarkdownParser::on_list_item_start()` (lines 338-367) - PDA list item handler
    - `MarkdownParser::on_block_end()` (lines 369-418) - PDA block end handler
    - `MarkdownParser::finalize_leaf_frame()` (lines 420-449) - PDA leaf finalizer
    - `MarkdownParser::finalize_container_frame()` (lines 451-480) - PDA container finalizer
    - `MarkdownParser::record_inline_event()` (lines 482-503) - PDA inline event recorder
    - `MarkdownParser::on_task_marker()` (lines 509-517) - PDA task marker handler
    - `MarkdownParser::finalize_link()` (lines 519-524) - PDA link finalizer
    - `MarkdownParser::record_link_event()` (lines 526-537) - PDA link event recorder
    - `MarkdownParser::record_open_item()` (lines 539-547) - PDA open item recorder
    - `MarkdownParser::open_link()` (lines 549-569) - PDA link opener
    - `frame_role_mismatch_error()` (lines 577-605) - PDA error helper
    - `link_display_from_events()` (lines 607-619) - PDA link display helper (may be reused in new extractor)
    - `resolve_reference_target()` (lines 804-815) - reference resolution helper (KEEP - needed for link extraction)
    - `is_reference_link_type()` (lines 817-828) - reference link checker (KEEP - needed by `resolve_reference_target()`)
  - **Fields to remove from `MarkdownParser` struct**:
    - `ref_defs: references::ReferenceDefinitions` (moved to `ParserContext`)
    - `stack: BlockStack<'source>` (PDA stack - deleted)
    - `depth: u32` (PDA depth - replaced by `DocTree` traversal depth)
    - `link: Option<ActiveLink<'source>>` (PDA link state - deleted)
    - `list_kinds: Vec<RawListKind>` (PDA list kind stack - deleted)
    - `open_items: Vec<usize>` (PDA open items - replaced by `DocTree` state)
    - `sink: S` (PDA sink - replaced by `BlockExtractor` direct call)
  - Downstream effect: `mod.rs` reduced from ~1239 lines to ~200 lines (just `parse()` and maybe `extension_options()`)
  - Test update: All PDA tests removed

- [ ] **4.2** List EVERY import that can be removed:
  - File: `lithos-core/src/note/parser/mod.rs`
  - **Imports to remove** (lines 90-108):
    - `use std::borrow::Cow;` - may be used by remaining code? Check: `resolve_reference_target()` uses `Cow` → KEEP if that function is kept
    - `use pulldown_cmark::Options;` - used by `extension_options()` → KEEP (or remove if `extension_options()` is removed)
    - `use text::{TextContext, TextSequence};` - used by PDA `record_inline_event()` → REMOVE (new code uses `DocTree`)
    - `use types::{BlockEnd, BlockStart, FrontmatterFormat, InlineDelimiterEnd, InlineDelimiterStart, InlineToken, LinkKind, ParserEvent, RangedEvent};` - used by PDA → REMOVE (new code in `structure.rs`/`extractor.rs`)
    - `use crate::{config::task::TaskConfigSpec, note::{error::{NoteIngestError, NoteParseError}, extractor::BlockExtractor, position::{SourceByteOffset, SourceByteRange}, raw::{RawLink, RawLinkStyle, RawListDepth, RawListKind, RawNote}, scanner::NoteScanner};` - some may be kept (e.g., `BlockExtractor`, `NoteScanner`, `RawNote` are used by new `parse()`)
    - **Keep these imports for new `parse()`**:
      - `crate::config::task::TaskConfigSpec`
      - `crate::note::error::NoteIngestError`
      - `crate::note::extractor::BlockExtractor`
      - `crate::note::parser::context::ParserContext` (NEW import)
      - `crate::note::parser::structure::DocTree` (NEW import)
      - `crate::note::parser::config::EventStreamConfig` (may need)
      - `crate::note::scanner::NoteScanner`
    - **Remove these imports**:
      - `text::TextContext` (not used in new code)
      - `text::TextSequence` (not used in `mod.rs` anymore, moved to `extractor.rs`)
      - All `types::*` imports (PDA used them, new code won't)
      - `raw::{RawLink, RawLinkStyle, RawListDepth, RawListKind}` (check if needed by `resolve_reference_target()`)
  - Downstream effect: Cleaner import list
  - Test update: None

- [ ] **4.3** Identify tests in `mod.rs` that test old PDA (must be rewritten or deleted):
  - File: `lithos-core/src/note/parser/mod.rs` (tests module, lines 830-1239)
  - **Tests to DELETE** (test PDA internals):
    - `NoopSink` struct (lines 840-865) - PDA test helper → DELETE
    - `task_spec_fixture()` (lines 867-884) - used by PDA tests → KEEP (used by `parse_raw()`)
    - `parse_raw()` (lines 886-889) - calls `MarkdownParser::parse()` → KEEP (calls new `parse()`)
    - `should_extract_block_ref_from_paragraph_tail` (line 892) - tests behavior → KEEP
    - `should_capture_yaml_at_start` (line 908) - tests behavior → KEEP
    - `should_capture_tags_inside_heading` (line 920) - tests behavior → KEEP
    - `should_ignore_tags_inside_links` (line 928) - tests behavior → KEEP
    - `should_ignore_block_refs_inside_links` (line 936) - tests behavior → KEEP
    - `should_detect_tag_after_link_label_gap` (line 944) - tests behavior → KEEP
    - `should_detect_block_ref_after_link_label_gap` (line 952) - tests behavior → KEEP
    - `should_not_scan_code_or_math_for_tags` (line 961) - tests behavior → KEEP
    - `finalize_leaf_frame_rejects_container_topology_mismatch` (line 973) - tests PDA → DELETE
    - `finalize_container_frame_rejects_leaf_topology_mismatch` (line 996) - tests PDA → DELETE
    - `should_extract_bare_fields` (line 1021) - tests behavior → KEEP
    - `should_handle_wikilinks` (line 1034) - tests behavior → KEEP
    - `should_track_list_nesting` (line 1050) - tests behavior → KEEP
    - `should_capture_checkbox_state_and_marker` (line 1065) - tests behavior → KEEP
    - `should_extract_thematic_break` (line 1078) - tests behavior → KEEP
    - `should_report_event_stack_mismatch_on_finalization` (line 1090) - tests PDA → DELETE
    - `reference_definitions_first_wins` (line 1120) - tests behavior → KEEP
    - `reference_definitions_are_case_insensitive` (line 1132) - tests behavior → KEEP
    - `reference_definitions_in_frontmatter_are_ignored` (line 1146) - tests behavior → KEEP
    - `reference_definitions_in_fenced_code_are_ignored` (line 1155) - tests behavior → KEEP
    - `reference_definitions_normalize_whitespace` (line 1163) - tests behavior → KEEP
    - `reference_definitions_unescape_labels` (line 1176) - tests behavior → KEEP
    - `reference_definitions_allow_multiline_destination` (line 1189) - tests behavior → KEEP
    - `external_scheme_targets_preserve_fragments` (line 1201) - tests behavior → KEEP
    - `file_scheme_targets_preserve_fragments` (line 1214) - tests behavior → KEEP
    - `s3_scheme_targets_preserve_fragments` (line 1229) - tests behavior → KEEP
  - **Summary**: DELETE ~4 tests that directly test PDA internals, KEEP ~20 tests that test parse behavior
  - Downstream effect: Test count decreases
  - Test update: Remove PDA test code, keep behavior tests

- [ ] **4.4** Check if `ArtifactSink` trait has other consumers besides `BlockExtractor`:
  - Search result from grep: Only 2 impls:
    - `impl<'source> ArtifactSink<'source> for NoopSink` (line 843 in `mod.rs`) - test helper → DELETE
    - `impl<'source> ArtifactSink<'source> for BlockExtractor<'source>` (line 151 in `extractor.rs`) → DELETE (after migration to `DocTree`)
  - **No other consumers** - safe to delete `ArtifactSink` trait
  - **Also check**: Is `ArtifactSink` used in any other files?
    - Grep result: Only in `mod.rs` and `extractor.rs`
  - Downstream effect: `ArtifactSink` trait deleted from `mod.rs`
  - Test update: `NoopSink` deleted from tests

- [ ] **4.5** Clean up `BlockSpan` usage:
  - File: `lithos-core/src/note/parser/mod.rs` (lines 669-683)
  - `BlockSpan` struct (old PDA type):
    ```rust
    pub(crate) struct BlockSpan {
        pub start: usize,
        pub end: usize,
    }
    ```
  - New code uses `SourceByteRange` (from `block.span` in `DocTree`)
  - **Can be deleted** after PDA removal
  - Also check `extractor.rs` - it imports `BlockSpan` (line 12):
    - Change to use `SourceByteRange` from `block.span`
  - Downstream effect: `BlockSpan` deleted
  - Test update: None

### Phase 5: Clean Up and Verify

Files modified:
- `lithos-core/src/note/parser/mod.rs` - Final cleanup
- `lithos-core/src/note/parser/stream.rs` - Check for dead code
- `lithos-core/src/note/parser/structure.rs` - Check for dead code
- `lithos-core/src/note/parser/block.rs` - Check for dead code
- `lithos-core/src/note/parser/text.rs` - Check for dead code
- `lithos-core/src/note/parser/types.rs` - Check for dead code
- `lithos-core/src/note/parser/config.rs` - Check for dead code
- `lithos-core/src/note/parser/context.rs` - Already uses new API
- `lithos-core/benches/note_parsing.rs` - Update benchmarks
- `docs/` - Update any parser documentation

Tasks:

- [ ] **5.1** Update documentation:
  - File: `lithos-core/src/note/parser/mod.rs` (module doc at lines 1-62)
  - Current doc describes "Pipeline Architecture" with 5 stages including "Artifact Assembler"
  - **Update to describe new architecture**:
    - Stage 1: `ParserContext` (eager parse + cache)
    - Stage 2: `DocTree::from_context()` (build AST)
    - Stage 3: `DocTree::for_each_block()` (traverse AST)
    - Stage 4: `BlockExtractor::process_doc_tree()` (extract artifacts)
  - Remove references to "ArtifactSink", "PDA", "stack", "sink" patterns
  - Update code examples to use new API:
    ```rust
    // Old example (lines 41-59):
    // let stream = MarkdownEventStream::new(source, config);
    // for event in stream { ... }

    // New example:
    // let ctx = ParserContext::new(source, config)?;
    // let tree = DocTree::from_context(&ctx)?;
    // tree.for_each_block(|block, depth| { ... });
    ```
  - Downstream effect: Docs aligned with new code
  - Test update: None

- [ ] **5.2** Update benchmarks:
  - File: `lithos-core/benches/note_parsing.rs`
  - Current benchmark (line 423): `parser::MarkdownParser::parse(&markdown, &task_spec)`
  - **No change needed** - benchmark already calls `MarkdownParser::parse()` which is being kept (with new implementation)
  - But: Check if benchmarks import anything from `mod.rs` that will be deleted:
    - Line 226: `use lithos_core::{config::task::TaskConfigSpec, fs::FsReader, note::parser};` - uses `note::parser` module
    - Benchmark calls `parser::MarkdownParser::parse()` - this stays
  - **No update needed** - benchmark uses public API which stays the same
  - Downstream effect: Benchmarks still work
  - Test update: None

- [ ] **5.3** Check for clippy warnings after deletion:
  - File: All files in `lithos-core/src/note/parser/`
  - After deleting PDA code, run `mise run lint`
  - **Potential warnings**:
    - Unused imports (in files that previously imported PDA types)
    - Dead code (functions that were only called by PDA)
    - Missing docs (new types/functions)
  - **Files to check**:
    - `extractor.rs` - removed `ArtifactSink` impl, may have unused imports
    - `stream.rs` - check if any functions were only used by PDA
    - `types.rs` - check if all types are used (some may have been PDA-only)
    - `block.rs` - check if all types are used
    - `structure.rs` - check if all types are used
    - `text.rs` - check if all types are used
  - Downstream effect: Clean lint output
  - Test update: Fix any test code that triggers warnings

- [ ] **5.4** Verify `resolve_reference_target()` and `is_reference_link_type()` placement:
  - File: `lithos-core/src/note/parser/mod.rs` (lines 804-828)
  - These functions are used for link extraction:
    - `resolve_reference_target()` - resolves reference link targets using `ReferenceDefinitions`
    - `is_reference_link_type()` - checks if a `LinkKind` is a reference type
  - **After PDA removal**: These functions are needed by the new link extraction logic in `extractor.rs`
  - **Move to `extractor.rs`** or to a shared `parser/links.rs` module
  - Decision: Move to `extractor.rs` (since link extraction now happens there)
  - Also move `link_display_from_events()` (line 607-619) if it's still needed
  - Downstream effect: `mod.rs` no longer has link helper functions
  - Test update: None (functions moved, not changed)

- [ ] **5.5** Remove module-level `#[expect]` attributes that are no longer needed:
  - File: `lithos-core/src/note/parser/mod.rs`
  - Line 64-67: `#![expect(clippy::pattern_type_mismatch, ...)]` - was for PDA pattern matches
    - **Can be removed** if new code doesn't trigger this warning
  - Line 113-116: `#[expect(private_bounds, ...)]` for `MarkdownParser` with `S: ArtifactSink<'source>`
    - **Can be removed** after `MarkdownParser` struct is simplified
  - Line 135-138: `#[expect(private_bounds, ...)]` for `impl<'source, S> MarkdownParser`
    - **Can be removed** after PDA removal
  - Downstream effect: Cleaner code
  - Test update: None

- [ ] **5.6** Run full verification suite:
  - Command: `mise run verify`
  - This runs: fmt + lint + tests + adr:validate
  - Fix any failures
  - Downstream effect: All gates pass
  - Test update: None

- [ ] **5.7** Check for any remaining references to deleted types:
  - Grep for: `ArtifactSink`, `BlockSpan`, `BlockStack`, `BlockFrame`, `ActiveLink`, `LeafKind`, `ContainerKind`, `ListItemPayload`, `MetadataPayload`, `HeadingPayload`
  - These should NOT appear in any code files (only in git history and comments)
  - If found in comments/docs, update or remove
  - Downstream effect: No stale references
  - Test update: None

---

## Cross-Stage Watch Items (Do Not Miss)

- [ ] Minimal module wiring may still be needed for new `types.rs` declaration; treat as mechanical compile plumbing only.
- [ ] Keep source range fidelity through every transformation.
- [ ] Keep `ReferenceDefinitions` behavior unchanged unless explicitly scoped.
- [ ] Ensure `ParserContext` remains the canonical cache boundary after traversal API changes.
- [ ] Avoid introducing new string allocation anti-patterns in hot paths.

## Suggested Commit Slices

- [ ] `refactor(parser): add extension policy model`
- [ ] `refactor(parser): introduce parser neutral types module`
- [ ] `refactor(parser): enforce stream mapping policy and preserve math`
- [ ] `refactor(parser): decouple block ast from pulldown`
- [ ] `refactor(parser): enforce structure stack invariants`
- [ ] `refactor(parser): replace visitor trait with traversal iterator`
- [ ] `test(parser): add capability matrix contract coverage`
- [ ] `refactor(parser): rewrite MarkdownParser::parse() to use DocTree` (Phase 3)
- [ ] `refactor(parser): remove legacy PDA code` (Phase 4)
- [ ] `refactor(parser): clean up and verify` (Phase 5)
