# Parsing Layer Implementation Plan

**Status**: Ready for Implementation
**Created**: 2026-04-27
**Approved By**: Jack

---

## Executive Summary

This document provides a complete implementation roadmap for refactoring the Lithos note parsing layer from a single-pass Pushdown Automaton (PDA) to a multi-phase **Parse → Cache → Structure** architecture.

**Key Goals**:

- **Strict separation**: Grammar recognition (parsing) vs Pattern matching (scanning) vs Domain validation
- **Zero-copy throughout**: Borrow from source until storage layer via `SourceByteRange`
- **LSP-ready**: Cache parsed results for incremental updates
- **Visitor pattern**: Clean traversal API for metadata extraction
- **Thick Adapter**: Centralize `pulldown-cmark` dependency in a single module (`stream.rs`)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ PARSING LAYER (This Implementation)                        │
├─────────────────────────────────────────────────────────────┤
│ Input:  Raw markdown (&str)                                 │
│ Output: DocTree (AST)                                  │
│ Scope:  Grammar recognition ONLY                            │
├─────────────────────────────────────────────────────────────┤
│ Components:                                                 │
│ 1. MarkdownEventStream (Thick Adapter) ✅ UPDATED           │
│ 2. ParserContext (Cache) ✅ EXISTS                          │
│ 3. DocTree (AST) ✅ EXISTS                             │
│ 4. BlockVisitor (Traversal) ✅ EXISTS                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ LEXICAL SCANNING LAYER (Future Work - Not This PR)         │
├─────────────────────────────────────────────────────────────┤
│ 1. MetadataScanner (Pattern matching on InlineEvents)      │
│ 2. MetadataExtractor (Visitor implementation)              │
└─────────────────────────────────────────────────────────────┘
```

---

## Terminology (CommonMark 0.31.2 Alignment)

To ensure consistency, the parsing layer uses official CommonMark terminology:

- **Block**: Structural elements of the document.
  - **Container Block**: Blocks that can contain other blocks (e.g., `BlockQuote`, `List`, `Item`).
  - **Leaf Block**: Blocks that contain inlines but not other blocks (e.g., `Paragraph`, `Heading`, `CodeBlock`, `ThematicBreak`).
- **Inline**: Content within a Leaf Block (e.g., `Text`, `CodeSpan`, `Emphasis`, `Link`).
- **Thick Adapter**: A design pattern where a wrapper module (`stream.rs`) translates external library events into internal domain events, ensuring the rest of the crate is decoupled from the library's API.

---

## Approved Design Decisions

### 1. Component Naming

- **`DocTree`**: The AST container (not `MarkdownAst`, `NoteStructure`, or `ParsedDocument`)
- **`ParserContext`**: The cache (not `NoteContext`)
- **`Block`**: AST node type
- **`BlockVisitor`**: Traversal trait

### 1.1 Block Module Placement

- **Decision**: Move all block-domain types to `lithos-core/src/note/parser/block.rs`.
- **Scope**: `Block`, container/leaf block enums, `HeadingLevel`, `ListKind`, inline-bearing leaf payload types, and block helper methods (`text()`, `is_scannable()`).
- **Rationale**: `structure.rs` currently mixes domain model + builder algorithm + tests; extracting block model improves cohesion and clarifies responsibilities.
- **Boundary**:
  - `block.rs` = pure domain model + helpers
  - `structure.rs` = `StructureBuilder` / `Processing*` build-state + tree assembly algorithm
  - `visitor.rs` = traversal contract and visitor implementations over `block.rs` types

### 2. Frontmatter Handling

- **Decision**: Store as `BlockKind::Frontmatter { format, text }`
- **Rationale**: Frontmatter is a block per CommonMark spec, not special metadata
- **Location**: Can only appear in `DocTree::blocks[0]` (root-level only)

### 3. List Depth Tracking

- **Decision**: Pre-compute during AST building and store in `BlockKind::ListItem`
- **Fields**: `depth: u32`, `parent_span: Option<SourceByteRange>`
- **Rationale**: Structural information (not metadata), used by multiple consumers

### 4. Caching Strategy

- **Decision**: Cache both events AND AST in `ParserContext`
- **Mechanism**: `OnceCell<DocTree>` for lazy initialization
- **Rationale**: LSP will request multiple passes per edit (diagnostics + autocomplete + hover)

### 5. Block Helper Methods

- **Decision**: Yes, include `text()` and `is_scannable()` helpers
- **Rationale**: Encapsulation, cleaner extractor code

### 6. Error Handling

- **Decision**: `DocTree::from_context()` returns `Result<DocTree, ParseError>`
- **Rationale**: Safety against malformed event streams (stack underflow)

### 7. Internal Representation (IR) "Chokepoint"

- **Decision**: Define a Lithos-native `ParserEvent` enum to replace raw `pulldown_cmark::Event` throughout the pipeline.
- **Rationale**: Decouples the codebase from `pulldown-cmark` version drift and simplifies downstream matching logic.
- **Components**:
  - `ParserEvent`: Signals `BlockStart`, `BlockEnd`, `Inline`, or `ThematicBreak`.
  - `InlineEvent`: Encapsulates text content and inline markers (`Link`, `Emphasis`, `CodeSpan`).

### 8. Thick Adapter Implementation (`stream.rs`)

- **Decision**: `stream.rs` is the ONLY module permitted to import `pulldown_cmark`.
- **Normalization**:
  - `SoftBreak` -> `InlineEvent::Text(" ")`
  - `HardBreak` -> `InlineEvent::Text("\n")`
  - `TaskListMarker` -> Captured by `ListItem` state during `BlockStart(Item)`.
- **Zero-Copy**: Uses `pulldown_cmark::CowStr` but prefers referencing the original source via `SourceByteRange` obtained from `into_offset_iter()`.

### 9. CommonMark 0.31.2 Taxonomy Alignment

- **Container Block**: Recursive structural elements (List, BlockQuote). They define the "skeleton" of the document.
- **Leaf Block**: Non-recursive terminal nodes (Paragraph, Heading). They hold the content and inline boundaries.
- **Inline**: Granular elements inside Leaf Blocks (Text, Link, Emphasis, CodeSpan).
- **Rationale**: Strict adherence to the spec prevents misunderstanding of component responsibilities.

### 10. Structural Storage (`DocTree`)

- **Decision**: `Paragraph` and `Heading` blocks store `Vec<InlineWithRange<'source>>`.
- **Rationale**: Sets up the lexical scanning layer to operate solely on inline streams without structural noise.

### 11. Container vs Leaf Separation

- **Decision**: Represent container and leaf blocks as separate types under a unified `Block` wrapper.
- **Rationale**: Only container blocks can own child blocks; encoding this structurally (not by convention) improves type safety and simplifies depth/list bookkeeping in the builder.
- **Planned Shape**:
  - `Block = Container(ContainerBlock) | Leaf(LeafBlock)`
  - `ContainerBlockKind = BlockQuote | List | ListItem`
  - `LeafBlockKind = Paragraph | Heading | CodeBlock | Frontmatter | ThematicBreak`

---

## pulldown-cmark to Lithos IR Mapping

This table defines how `pulldown-cmark` types map to Lithos Internal Representation (IR) types. This is the contract that the Thick Adapter (`stream.rs`) must implement.

### Event Mapping

| `pulldown_cmark::Event`               | Lithos IR `ParserEvent`                     | Notes                                    |
| ------------------------------------- | ------------------------------------------- | ---------------------------------------- |
| `Start(Tag::Paragraph)`               | `BlockStart(BlockType::Paragraph)`          | Pushes new leaf frame                    |
| `Start(Tag::Heading { level })`       | `BlockStart(BlockType::Heading(level))`     | Level converted to Lithos `HeadingLevel` |
| `Start(Tag::BlockQuote)`              | `BlockStart(BlockType::BlockQuote)`         | Increments depth                         |
| `Start(Tag::List(start))`             | `BlockStart(BlockType::List(kind))`         | `start` → `ListKind`                     |
| `Start(Tag::Item)`                    | `BlockStart(BlockType::ListItem)`           | Captures parent span                     |
| `Start(Tag::CodeBlock(kind))`         | `BlockStart(BlockType::CodeBlock(lang))`    | \_kind → language                        |
| `Start(Tag::MetadataBlock(kind))`     | `BlockStart(BlockType::Frontmatter(kind))`  |                                          |
| `End(TagEnd::*)`                      | `BlockEnd(BlockType::*)`                    | Pops frame                               |
| `Text(CowStr)`                        | `Inline(InlineEvent::Text(Cow))`            | Raw text content                         |
| `Code(CowStr)`                        | `Inline(InlineEvent::CodeSpan(Cow))`        | Inline code                              |
| `Html(CowStr)` / `InlineHtml(CowStr)` | `Inline(InlineEvent::Html(Cow))`            | Raw HTML                                 |
| `SoftBreak`                           | `Inline(InlineEvent::Text(" ".into()))`     | Normalized per `BreakPolicy`             |
| `HardBreak`                           | `Inline(InlineEvent::Text("\n".into()))`    | Normalized per `BreakPolicy`             |
| `TaskListMarker(bool)`                | `Inline(InlineEvent::TaskListMarker(bool))` | Stored in ListItem state                 |
| `Rule`                                | `ThematicBreak`                             | Standalone block                         |

### Tag Classification (per CommonMark 0.31.2)

| `pulldown_cmark::Tag` | Block Type    | Can Contain   |
| --------------------- | ------------- | ------------- |
| `Paragraph`           | **Leaf**      | Inlines only  |
| `Heading`             | **Leaf**      | Inlines only  |
| `BlockQuote`          | **Container** | Blocks        |
| `List`                | **Container** | `Item` blocks |
| `Item`                | **Container** | Blocks        |
| `CodeBlock`           | **Leaf**      | Flat text     |
| `MetadataBlock`       | **Leaf**      | Raw text      |
| `FootnoteDefinition`  | **Container** | Blocks (rare) |

### Zero-Copy Rules

1. **Source Ranges**: Always use `Range<usize>` from `into_offset_iter()` to create `SourceByteRange`
2. **String Borrowing**: Prefer `Cow::Borrowed` over `Cow::Owned` where possible
3. **Conversion**: Only allocate when converting to `String` for storage in `BlockKind::CodeBlock` / `BlockKind::Frontmatter`

---

## Implementation Phases (REVISED)

### Phase 1: Foundation (COMPLETED)

- `ParserContext` eager caching implemented.
- `DocTree` and `BlockVisitor` foundations established.
- `EventWithRange` moved to `stream.rs`.

---

### Phase 2: IR Refactoring & Thick Adapter (NEXT)

**Goal**: Decouple `structure.rs` and `context.rs` from `pulldown-cmark`.

**Tasks**:

1. Define `ParserEvent` and `InlineEvent` in `stream.rs`.
2. Update `EventAdapterIter` to emit `ParserEvent` instead of `pulldown_cmark::Event`.
3. Update `EventWithRange` to wrap `ParserEvent`.
4. Refactor `BlockKind` in `structure.rs` to use `InlineEvent`.
5. Remove all `pulldown_cmark` imports from `structure.rs`, `context.rs`, and `visitor.rs`.
6. Create `block.rs` and move block-domain types out of `structure.rs`.

**Acceptance Criteria**:

- [ ] `cargo test` passes for all 829+ tests.
- [ ] `grep -r "pulldown_cmark" src/note/parser/` only shows results in `stream.rs` and `config.rs`.
- [ ] `structure.rs` no longer defines block-domain types; it imports them from `block.rs`.

**Estimated Effort**: 4-6 hours

---

### Phase 3: Structure Building - AST Construction

**Goal**: Transform flat `ParserEvent` stream into hierarchical AST.

**Tasks**:

1. Implement `DocTree` struct in `structure.rs`.
2. Implement `DocTree::from_context()` with stack-based algorithm matching on `ParserEvent`.
3. Handle all `ParserEvent` variants (`BlockStart`, `BlockEnd`, `Inline`).
4. Track list depth and parent spans during building using internal `StructureBuilder`.
5. Add explicit builder-state types (`ProcessingBlockTree`, `ProcessingContainer`, `ProcessingLeaf`) and container/leaf finalize paths.
6. Add comprehensive tests for nested structures.

**Acceptance Criteria**:

- [ ] Simple paragraph parses to `BlockKind::Paragraph { events: Vec<InlineEventWithRange> }`.
- [ ] Nested lists correctly track depth (0 for root, 1 for first level, etc.).
- [ ] List items store parent span for nested items.
- [ ] Blockquotes correctly nest child blocks.
- [ ] Code blocks store language and flattened text.
- [ ] Frontmatter at start of document is captured.
- [ ] All existing integration tests pass.
- [ ] Container and leaf finalize paths are separated in `StructureBuilder` and covered by tests.

**Estimated Effort**: 4-6 hours

---

### Phase 4: Traversal API - Visitor Pattern

**Goal**: Provide clean traversal interface for metadata extraction.

**Tasks**:

1. Create `lithos-core/src/note/parser/visitor.rs`.
2. Define `BlockVisitor` trait with `visit_*` methods.
3. Implement `DocTree::walk()` method.
4. Add depth-tracking during traversal.
5. Add tests for visitor pattern.

**Acceptance Criteria**:

- [ ] `BlockVisitor` has methods for all `BlockKind` variants.
- [ ] `DocTree::walk()` correctly traverses nested structures.
- [ ] Visitor receives correct depth parameter.
- [ ] Test visitor can collect all block types and counts.

**Estimated Effort**: 2-3 hours

---

### Phase 5: Integration - Wire Into Pipeline

**Goal**: Replace legacy `MarkdownParser` with new architecture.

**Tasks**:

1. Create compatibility adapter: `DocTree` -> `BlockExtractor` callback.
2. Update `MarkdownParser::parse()` to use `ParserContext` + `DocTree`.
3. Run full test suite and fix regressions.
4. Update benchmarks to measure parsing vs extraction separately.

**Acceptance Criteria**:

- [ ] All 1103 lines of existing parser tests pass.
- [ ] No performance regression (within 5% of baseline).
- [ ] Benchmarks show clear phase separation.

**Estimated Effort**: 3-4 hours

---

### Phase 6: Cleanup - Remove Legacy Code

**Goal**: Delete obsolete components now that new pipeline is proven.

**Tasks**:

1. Delete `BlockSpan` (replaced by `SourceByteRange`).
2. Delete `TextFragment` (replaced by `Vec<SpannedEvent>`).
3. Delete `BlockFrame` (replaced by `StackFrame`).
4. Delete `LeafKind` and `ContainerKind` (replaced by `BlockKind`).
5. Delete `BlockStack` and `FragmentPool` (replaced by `DocTree` internal stack).
6. Delete `ArtifactSink` trait (replaced by `BlockVisitor`).
7. Update module exports in `parser/mod.rs`.

**Acceptance Criteria**:

- [ ] All legacy types removed from `parser/mod.rs`.
- [ ] No dead code warnings.
- [ ] All tests still pass.
- [ ] Documentation updated.

**Estimated Effort**: 1-2 hours

## Detailed Component Specification

### 1. `ParserEvent` (The Chokepoint)

```rust
pub(crate) enum ParserEvent<'source> {
    BlockStart(BlockType<'source>),
    BlockEnd(BlockType<'source>),
    Inline(InlineEvent<'source>),
    ThematicBreak,
}
```

### 2. `InlineEvent` (Content)

```rust
pub(crate) enum InlineEvent<'source> {
    Start(InlineTag),
    End(InlineTag),
    Text(Cow<'source, str>),
    CodeSpan(Cow<'source, str>),
    Html(Cow<'source, str>),
    TaskListMarker(bool),
}
```

### 3. `block.rs` Components (Post-Refactor)

`block.rs` will contain the complete block-domain model:

- `Block<'source>`
- `ContainerBlock<'source>` and `ContainerBlockKind`
- `LeafBlock<'source>` and `LeafBlockKind<'source>`
- `HeadingLevel`, `ListKind`
- helper methods: `text()`, `is_scannable()`

`structure.rs` will no longer own these domain types; it will only assemble them.

### 4. `structure.rs` Components (Post-Refactor)

`structure.rs` will focus on build-time orchestration:

- `DocTree<'source>`
- `StructureBuilder<'source>`
- `ProcessingBlockTree<'source>`
- `ProcessingContainer<'source>` / `ProcessingLeaf<'source>`
- finalize and nesting/depth bookkeeping logic

---

## Dependencies & Interaction

### pulldown-cmark Interaction

- **`into_offset_iter()`**: Used to get absolute byte ranges for every event.
- **`CowStr`**: Used for efficient string handling, but mapped to `std::borrow::Cow` in our IR to avoid library leakage.
- **`Options`**: Defined in `config.rs` but passed solely to `Parser::new_ext` in `stream.rs`.

---

## Success Criteria (Updated)

- [ ] **Zero Coupling**: No `pulldown_cmark` types in `structure.rs`.
- [ ] **CommonMark Fidelity**: Nested lists and blockquotes correctly modeled via `ParserEvent::BlockStart/End`.
- [ ] **Performance**: IR conversion overhead remains <2% of total parse time.
- [ ] **Clean Pipeline**: `DocTree::from_context` only matches on `ParserEvent`.

---

## Total Estimated Effort

**15-22 hours** across 6 phases

---

## Components Requiring Design Clarity (RESOLVED)

### Original Questions (Previously BLOCKING):

1. **`Block`** - Final AST node (has complete `span: SourceByteRange`)
2. **`PartialBlockKind`** - Incomplete block during building (only has `span_start: usize`)
3. **`StackFrame`** - Builder state (wraps `PartialBlockKind` + accumulates events/children)

**Questions to Answer**:

- Why do we need both `PartialBlockKind` AND `StackFrame`?
- Should `PartialBlockKind` be an enum or struct?
- How does `StackFrame` differ from `Block` structurally?
- Can we simplify by having `Block` use `Option<usize>` for `span.end`?

**Design Goals**:

- **Clear names**: Each type's purpose is obvious from the name
- **Single responsibility**: No overlapping concerns
- **Minimal duplication**: Don't repeat the same fields across types
- **Type safety**: Impossible to use an incomplete node as a complete one

---

### Resolution (IR "Chokepoint" Design):

The IR "Chokepoint" design resolves previous confusion:

1. **`Block`** - Now stores `Vec<InlineWithRange>` via Lithos IR
2. **`PartialBlockKind`** - Replaced by internal `ProcessingBlockKind` during building
3. **`StackFrame`** - Internal implementation detail of `StructureBuilder`

**Resolution Summary**:

- **IR "Chokepoint"**: All `pulldown-cmark` knowledge centralized in `stream.rs`
- **ParserEvent**: Unified event type for StructureBuilder
- **InlineEvent**: Content type for Leaf Blocks (future scanner will use this)
- **Clear separation**: Parsing (IR) vs Scanning (InlineEvent) vs Validation

---

## Dependencies

### Existing Components (Updated for IR Design)

- ✅ `MarkdownEventStream` & `EventAdapterIter` (`parser/stream.rs`) - Thick Adapter
- ✅ `EventStreamConfig` (`parser/config.rs`)
- ✅ `EventWithRange` (`parser/stream.rs`) - Now wraps IR `ParserEvent`
- ✅ `ReferenceDefinitions` (`parser/references.rs`)
- ✅ `SourceByteRange` (`note/position.rs`)
- ✅ `DocTree` (`parser/structure.rs`) - Uses `ParserEvent`
- ✅ `BlockVisitor` (`parser/visitor.rs`)

### External Crates

- ✅ `pulldown-cmark` (already in `Cargo.toml`)
- ❌ No new dependencies required

---

## Testing Strategy

### Unit Tests (Per Component)

- `ParserContext`: Event caching, reference resolution
- `Block`: Helper methods (`text()`, `is_scannable()`)
- `DocTree::from_context()`: AST building for each block type

### Integration Tests (End-to-End)

- Parse markdown with all block types (headings, lists, code, blockquotes)
- Nested structures (lists in blockquotes, paragraphs in list items)
- Edge cases (empty blocks, malformed nesting)

### Regression Tests

- All 1103 existing parser tests must pass
- Existing benchmarks must not regress >5%

### New Test Fixtures

```markdown
# Test: Nested Lists with Depth Tracking
- Root item (depth=0)
  - Nested item (depth=1, parent=Root)
    - Deep item (depth=2, parent=Nested)

# Test: Frontmatter Capture
---
tags: [test]
---
Content here

# Test: Mixed Containers
> Blockquote
>
> - List in quote
>   - Nested in quote
```

---

## Success Criteria

### Functional

- [ ] All existing tests pass
- [ ] New AST correctly models nesting (list items, blockquotes)
- [ ] Depth and parent tracking matches legacy behavior
- [ ] Frontmatter is captured and accessible

### Non-Functional

- [ ] No performance regression (±5%)
- [ ] Memory usage within 10% of baseline
- [ ] Code coverage >85% for new components

### Architectural

- [ ] Clear separation: Parsing vs Scanning vs Validation
- [ ] Visitor pattern enables extensibility
- [ ] Zero-copy maintained (borrows from source)
- [ ] LSP-ready caching infrastructure

---

## Risks & Mitigations

| Risk                                   | Impact | Mitigation                                             |
| -------------------------------------- | ------ | ------------------------------------------------------ |
| Breaking existing tests                | High   | Run full suite after each phase                        |
| Performance regression                 | Medium | Benchmark each phase, optimize hot paths               |
| Complexity in nested structure builder | Medium | Comprehensive unit tests for each nesting scenario     |
| Incomplete event handling              | Low    | Exhaustive match on `Event` enum, fail fast on unknown |

---

## Next Steps

1. **Review and approve this plan** ✅
2. **Design IR Chokepoint & Thick Adapter** ✅
3. **Implement Phase 1 (ParserContext)** ✅
4. **Implement Phase 2 (IR Refactoring)** ⬅️ **NEXT**
5. **Implement Phase 3 (DocTree building)**
6. **Continue through remaining phases**

---

## Open Questions for Stakeholder

1. **Performance target**: Is ±5% acceptable, or do we need stricter bounds?
2. **Incremental delivery**: Should we merge after each phase, or ship all 6 phases together?
3. **Backward compatibility**: Do we need to support the old `MarkdownParser` API during transition?

---

**Document Status**: ✅ Ready for IR Implementation

**Blocking Issue**: Resolved - IR "Chokepoint" design centralizes pulldown-cmark dependency in stream.rs
