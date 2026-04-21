# Lithos Note Pipeline - Greenfield Architecture Design

**Status**: Research & Planning Phase
**Created**: 2026-04-21
**Last Updated**: 2026-04-21

---

## Executive Summary

This document outlines a complete redesign of the Lithos note parsing pipeline based on:
1. General parsing pipeline best practices research
2. Rust-specific parsing idioms and patterns
3. Analysis of production Rust markdown projects (rumdl, comrak, pulldown-cmark)
4. Critical review of current architecture (28 flaws identified)

**Key Goals**:
- Strict phase separation (Parse → Extract → Validate → Store)
- Zero-copy throughout pipeline (borrow until storage)
- Dependency injection (no component creates dependencies)
- Single responsibility per component
- LSP/IDE-ready (cached parsing via NoteContext)

---

## Research Findings Summary

### General Parsing Best Practices

**Pipeline Stages**: Lexical → Syntactic → Semantic → Output

**Key Principles**:
- Separation of concerns (each phase owns one aspect)
- Pull-based parsing (parser drives)
- Error recovery (panic mode + synchronization)
- Zero-copy (return slices, not owned strings)
- Minimal state (prefer call stack)

### Rust-Specific Patterns

- Lifetime-based zero-copy: `Output<'a>` borrows from `Input<'a>`
- Type-driven validation: Raw types (syntax) vs Domain types (semantics)
- Closure-based access for zero-copy (already in Lithos!)
- `Cow<'a, str>` for conditional allocation
- Indices over references for graph structures

### Real-World Projects

**rumdl** (Obsidian linting):
- NoteContext pattern: Parse once, cache events (30-50% speedup)
- Content characteristics: Pre-scan to skip irrelevant validators
- Two-phase validation

**pulldown-cmark**:
- Event-driven Iterator of `Event<'a>` with offsets
- Zero-copy: All text borrows from source

---

## Current Architecture - 28 Critical Flaws

### Scanner (8 flaws)
1. ScannerContext is config, not context
2. Hard-coded rules (fake extensibility)
3. Cursor partial public fields
4. ScanMode for one rule (BareFieldRule)
5. ScanRule trait inadequate (multi-byte issue)
6. BareFieldRule privileged (doesn't implement trait)
7. EmojiFieldRule.can_start_with() broken
8. 5 allocations per scan

### Extractor (8 flaws)
9. Naming: builder not extractor
10. Magic number capacities
11. finish() sorts (why out of order?)
12. process_leaf duplication
13. Arbitrary domain routing logic
14. Inconsistent depth tracking
15. Needless allocations
16. Wrong signatures (String vs Cow)

### Parser (12 flaws)
17. Generic facade (fake polymorphism)
18. Knows about TaskConfigSpec
19. 7 responsibilities (should be 2)
20. Silently ignores events
21. Inconsistent frame creation
22. Complex list item logic
23. 118 lines of copy-paste in on_end
24. Clones fragments unnecessarily
25. Stateful link handling
26. 70 lines embedded LinkRefResolver
27. Hot path metrics pollution
28. Wasteful normalize_breaks

---

## Greenfield Architecture

### Data Flow

```
File → MarkdownEvents → NoteContext → ArtifactBuilder → RawNote → Note → Storage
       (pulldown-cmark)  (cache)      (construct)      (syntax)  (semantic)
```

### Core Principles

1. **Strict Phase Separation**: File I/O → Parsing → Extraction → Validation → Storage
2. **Zero-Copy**: Borrow from source until final storage
3. **Functional Composition**: Direct function calls with `Result<T, E>`
4. **Type-Driven Validation**: `RawNote` (syntax) vs `Note` (semantics)
5. **Dependency Injection**: No component creates dependencies
6. **Single Responsibility**: One job per component

### Components

#### 1. MarkdownEvents (New)
**Purpose**: Wrap pulldown-cmark with Lithos config
**Responsibilities**:
- Configure extensions (tasklists, wikilinks, frontmatter)
- Normalize breaks to text
- Merge adjacent text events
- Provide reference definitions

```rust
pub struct MarkdownEvents<'a> {
    source: &'a str,
    inner: TextMergeWithOffset<...>,
}

impl Iterator for MarkdownEvents<'a> {
    type Item = (Event<'a>, Range<usize>);
}
```

#### 2. NoteContext (New - rumdl pattern)
**Purpose**: Cache parsed structures for reuse
**Responsibilities**:
- Parse markdown once
- Cache events for multiple validators
- Detect content features (for filtering)
- Provide source reference

```rust
pub struct NoteContext<'a> {
    source: &'a str,
    events: Vec<(Event<'a>, Range<usize>)>,
    ref_defs: HashMap<&'a str, &'a str>,
    characteristics: ContentFeatures,
}

pub struct ContentFeatures {
    has_frontmatter: bool,
    has_wikilinks: bool,
    has_tags: bool,
    has_tasks: bool,
    has_block_refs: bool,
}
```

**Benefits**: 30-50% performance gain (parse once, use many times)

#### 3. ArtifactBuilder (Needs Refinement)
**Purpose**: Build RawNote from event stream
**Current Issue**: Name suggests just builds RawNote, but does more
**Responsibilities** (to be refined after scanner research):
- Process event stream
- Build block structure
- Track nesting (via extracted component?)
- Trigger metadata scanning
- Produce RawNote

#### 4. MetadataScanner (To Be Researched)
**Purpose**: Scan text for Obsidian metadata
**Open Questions**:
- When to scan? Before Raw* types or after?
- Work on pulldown-cmark events or structured Raw* objects?
- Policy-based design: how to make extensible?

**Action**: Subagent researching scanner design

#### 5. TryFrom<RawNote> for Note (Use Existing)
**Purpose**: Semantic validation and conversion
**Implementation**: Already exists in aggregate.rs (lines 614-689)
**Responsibilities**:
- Validate references exist
- Check cycles, depth limits
- Convert borrowed → owned
- Collect artifacts from various sources

---

## Open Questions & Research Tasks

### 1. Scanner Design (IN PROGRESS)
**Subagent researching**:
- Policy-based scanner patterns
- When to work on text vs Raw* types
- Performance vs extensibility tradeoffs
- Best practices from production scanners

### 2. Structural Components (PENDING)
**Need to critique**:
- BlockFrame (Leaf vs Container)
- LeafKind (Heading, Paragraph, ListItem, Metadata)
- ContainerKind (List, BlockQuote, CodeBlock)
- Are these right abstractions?
- Do they reflect markdown structure or implementation?

### 3. Lexical vs Semantic Separation (PENDING)
**Questions**:
- Should we differentiate lexical from semantic concerns?
- Where does this boundary fall in our pipeline?
- How does it affect component design?

### 4. Component Naming & Responsibilities
**Issues**:
- "NestingTracker" → "ListStack" (more accurate)
- "ArtifactBuilder" → too broad, needs refinement
- What are precise boundaries?

---

## User Decisions

### Confirmed
1. **Scope**: Just parsing/extraction pipeline
2. **Note type**: Use existing in aggregate.rs
3. **Caching**: Implement NoteContext now
4. **API**: Redesign from scratch
5. **LSP**: Plan for future LSP/IDE features
6. **Performance**: Clean design first, optimize later

### To Validate
- TryFrom instead of NoteValidator ✓
- ListStack instead of NestingTracker
- Scanner design (awaiting research)
- Structural components review

---

## Next Steps

1. ✅ Complete scanner design research (subagent running)
2. ⏳ Review structural components (BlockFrame, etc.)
3. ⏳ Define lexical vs semantic boundaries
4. ⏳ Refine ArtifactBuilder responsibilities
5. ⏳ Create detailed component specs
6. ⏳ Write implementation plan with phases

---

## References

### Current Codebase
- Parser: `lithos-core/src/note/parser.rs` (1041 lines)
- Extractor: `lithos-core/src/note/extractor.rs` (228 lines)
- Scanner: `lithos-core/src/note/scanner.rs` (655 lines)
- Note aggregate: `lithos-core/src/note/aggregate.rs` (981 lines)

### Key Insights
- rumdl NoteContext: 30-50% gain from caching
- Type-driven: RawNote (syntax) → Note (semantics)
- Zero-copy: Lifetime-based borrowing
- Dependency injection: Receive, don't create
