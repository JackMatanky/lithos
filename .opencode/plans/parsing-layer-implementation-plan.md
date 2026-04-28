# Parsing Layer Implementation Plan

**Status**: Ready for Implementation
**Created**: 2026-04-27
**Approved By**: Jack

---

## Executive Summary

This document provides a complete implementation roadmap for refactoring the Lithos note parsing layer from a single-pass Pushdown Automaton (PDA) to a multi-phase **Parse → Cache → Structure** architecture.

**Key Goals**:

- Strict separation: Grammar recognition (parsing) vs Pattern matching (scanning) vs Domain validation
- Zero-copy throughout: Borrow from source until storage layer
- LSP-ready: Cache parsed results for incremental updates
- Visitor pattern: Clean traversal API for metadata extraction

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ PARSING LAYER (This Implementation)                        │
├─────────────────────────────────────────────────────────────┤
│ Input:  Raw markdown (&str)                                 │
│ Output: DocStructure (AST)                                  │
│ Scope:  Grammar recognition ONLY                            │
├─────────────────────────────────────────────────────────────┤
│ Components:                                                 │
│ 1. MarkdownEventStream (Thick Adapter) ✅ UPDATED           │
│ 2. ParserContext (Cache) ✅ EXISTS                          │
│ 3. DocStructure (AST) ✅ EXISTS                             │
│ 4. BlockVisitor (Traversal) ✅ EXISTS                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ LEXICAL SCANNING LAYER (Future Work - Not This PR)         │
├─────────────────────────────────────────────────────────────┤
│ 1. MetadataScanner (Pattern matching on InlineEvents)      │
│ 2. MetadataExtractor (Visitor implementation)              │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ SEMANTIC VALIDATION LAYER (Already Exists)                 │
├─────────────────────────────────────────────────────────────┤
│ 1. Note::try_from(RawNote) ✅ EXISTS                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Approved Design Decisions

### 1. Component Naming

- **`DocStructure`**: The AST container (not `MarkdownAst`, `NoteStructure`, or `ParsedDocument`)
- **`ParserContext`**: The cache (not `NoteContext`)
- **`Block`**: AST node type
- **`BlockVisitor`**: Traversal trait

### 2. Frontmatter Handling

- **Decision**: Store as `BlockKind::Frontmatter { format, text }`
- **Rationale**: Frontmatter is a block per CommonMark spec, not special metadata
- **Location**: Can only appear in `DocStructure::blocks[0]` (root-level only)

### 3. List Depth Tracking

- **Decision**: Pre-compute during AST building and store in `BlockKind::ListItem`
- **Fields**: `depth: u32`, `parent_span: Option<SourceByteRange>`
- **Rationale**: Structural information (not metadata), used by multiple consumers

### 4. Caching Strategy

- **Decision**: Cache both events AND AST in `ParserContext`
- **Mechanism**: `OnceCell<DocStructure>` for lazy initialization
- **Rationale**: LSP will request multiple passes per edit (diagnostics + autocomplete + hover)

### 5. Block Helper Methods

- **Decision**: Yes, include `text()` and `is_scannable()` helpers
- **Rationale**: Encapsulation, cleaner extractor code

### 6. Error Handling

- **Decision**: `DocStructure::from_context()` returns `Result<DocStructure, ParseError>`
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
- **Leaf Block**: Non-recursive terminal nodes (Paragraph, Heading). They hold the "meat" of the content.
- **Inline**: Granular elements inside Leaf Blocks (Text, Link, Bold).
- **Rationale**: Strict adherence to the spec prevents misunderstanding of component responsibilities.

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

## Implementation Phases

### Phase 1: Foundation - ParserContext (Cache Layer)

**Goal**: Establish the caching infrastructure that will enable LSP features.

**Tasks**:

1. Create `lithos-core/src/note/parser/context.rs`
2. Implement `ParserContext` struct with eager event caching
3. Wire `MarkdownEventStream` into `ParserContext::new()`
4. Add tests for event caching and reference extraction

**Acceptance Criteria**:

- [ ] `ParserContext::new()` successfully caches all events from a 1000+ line markdown file
- [ ] `ParserContext::events()` returns borrowed slice without allocation
- [ ] `ParserContext::references()` correctly resolves case-insensitive link refs
- [ ] All existing parser tests still pass

**Estimated Effort**: 2-3 hours

---

### Phase 2: IR Refactoring & Thick Adapter

**Goal**: Decouple `structure.rs` and `context.rs` from `pulldown-cmark` using a domain-native IR.

**Tasks**:

1. Define `ParserEvent` and `InlineEvent` in `stream.rs`.
2. Update `EventAdapterIter` to emit `ParserEvent` instead of `pulldown_cmark::Event`.
3. Update `EventWithRange` to wrap `ParserEvent`.
4. Refactor `BlockKind` and `Block` in `structure.rs` to use `InlineEvent`.
5. Remove all `pulldown_cmark` imports from `structure.rs`, `context.rs`, and `visitor.rs`.
6. Update helper methods (`text()`, `is_scannable()`) to work with `InlineEvent`.

**Acceptance Criteria**:

- [ ] `BlockKind` has variants for all CommonMark block types
- [ ] `BlockKind::ListItem` stores depth and parent span
- [ ] `BlockKind::Frontmatter` captures YAML/Pluses metadata
- [ ] `Block::text()` correctly extracts text from `InlineEvent` sequence
- [ ] No `pulldown_cmark` imports outside `stream.rs` and `config.rs`

**Estimated Effort**: 4-6 hours

---

### Phase 3: Structure Building - AST Construction

**Goal**: Transform flat `ParserEvent` stream into hierarchical AST.

**Tasks**:

1. Implement `DocStructure` struct in `structure.rs`
2. Implement `DocStructure::from_context()` with stack-based algorithm matching on `ParserEvent`.
3. Handle all `ParserEvent` variants (`BlockStart`, `BlockEnd`, `Inline`).
4. Track list depth and parent spans during building using internal `StructureBuilder`.
5. Add comprehensive tests for nested structures.

**Acceptance Criteria**:

- [ ] Simple paragraph parses to `BlockKind::Paragraph { events: Vec<InlineEventWithRange> }`
- [ ] Nested lists correctly track depth (0 for root, 1 for first level, etc.)
- [ ] List items store parent span for nested items
- [ ] Blockquotes correctly nest child blocks
- [ ] Code blocks store language and flattened text
- [ ] Frontmatter at start of document is captured
- [ ] All existing integration tests pass

**Estimated Effort**: 4-6 hours

---

### Phase 4: Traversal API - Visitor Pattern

**Goal**: Provide clean traversal interface for metadata extraction.

**Tasks**:

1. Create `lithos-core/src/note/parser/visitor.rs`
2. Define `BlockVisitor` trait with `visit_*` methods
3. Implement `DocStructure::walk()` method
4. Add depth-tracking during traversal
5. Add tests for visitor pattern

**Acceptance Criteria**:

- [ ] `BlockVisitor` has methods for all `BlockKind` variants
- [ ] `DocStructure::walk()` correctly traverses nested structures
- [ ] Visitor receives correct depth parameter
- [ ] Test visitor can collect all block types and counts

**Estimated Effort**: 2-3 hours

---

### Phase 5: Integration - Wire Into Pipeline

**Goal**: Replace legacy `MarkdownParser` with new architecture.

**Tasks**:

1. Create compatibility adapter: `DocStructure` → `BlockExtractor` callback
2. Update `MarkdownParser::parse()` to use `ParserContext` + `DocStructure`
3. Run full test suite and fix regressions
4. Update benchmarks to measure parsing vs extraction separately

**Acceptance Criteria**:

- [ ] All 1103 lines of existing parser tests pass
- [ ] No performance regression (within 5% of baseline)
- [ ] Benchmarks show clear phase separation

**Estimated Effort**: 3-4 hours

---

### Phase 6: Cleanup - Remove Legacy Code

**Goal**: Delete obsolete components now that new pipeline is proven.

**Tasks**:

1. Delete `BlockSpan` (replaced by `SourceByteRange`)
2. Delete `TextFragment` (replaced by `Vec<SpannedEvent>`)
3. Delete `BlockFrame` (replaced by `StackFrame`)
4. Delete `LeafKind` and `ContainerKind` (replaced by `BlockKind`)
5. Delete `BlockStack` and `FragmentPool` (replaced by `DocStructure` internal stack)
6. Delete `ArtifactSink` trait (replaced by `BlockVisitor`)
7. Update module exports in `parser/mod.rs`

**Acceptance Criteria**:

- [ ] All legacy types removed from `parser/mod.rs`
- [ ] No dead code warnings
- [ ] All tests still pass
- [ ] Documentation updated

**Estimated Effort**: 1-2 hours

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
- ✅ `DocStructure` (`parser/structure.rs`) - Uses `ParserEvent`
- ✅ `BlockVisitor` (`parser/visitor.rs`)

### External Crates

- ✅ `pulldown-cmark` (already in `Cargo.toml`)
- ❌ No new dependencies required

---

## Testing Strategy

### Unit Tests (Per Component)

- `ParserContext`: Event caching, reference resolution
- `Block`: Helper methods (`text()`, `is_scannable()`)
- `DocStructure::from_context()`: AST building for each block type

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

## tags: [test]

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
3. **Implement Phase 1 (ParserContext)** ❌ (COMPLETED - Phase 1 done)
4. **Implement Phase 2 (IR Refactoring)** ⬅️ **NEXT**
5. **Implement Phase 3 (DocStructure building)**
6. **Continue through remaining phases**

---

## Open Questions for Stakeholder

1. **Performance target**: Is ±5% acceptable, or do we need stricter bounds?
2. **Incremental delivery**: Should we merge after each phase, or ship all 6 phases together?
3. **Backward compatibility**: Do we need to support the old `MarkdownParser` API during transition?

---

**Document Status**: ✅ Ready for IR Implementation

**Blocking Issue**: Resolved - IR "Chokepoint" design centralizes pulldown-cmark dependency in stream.rs
