# Note Module Analysis & Refactoring Strategy

## 1. Introduction and Goal

The `lithos-core/src/note/` module currently feels unnecessarily large, highly fragmented, and overly complex for its purpose. The primary goal of this document is to outline a refactoring strategy that significantly reduces the size of the codebase, improves performance, and achieves partial architectural parity with the data shapes defined in `@docs/refs/obsidian/api-reference.md` and `@docs/refs/obsidian/dataview-reference.md`.

Crucially, the ultimate domain aggregate representing an ingested markdown file must mirror Obsidian's `CachedMetadata` and be named **`Note`**. Because Lithos is focused purely on data for the MVP, features that pertain primarily to rendering text (such as `footnotes` and `footnoteRefs`) are deliberately excluded.

## 2. Research Findings

To determine the most efficient design, several other Rust markdown projects and Obsidian ecosystem models were analyzed:

### 2.1. Markdown Parsing Strategies (Rust Ecosystem)
1. **`basalt` (AST Construction):** `basalt` is an excellent TUI application for Obsidian. Because it must physically render markdown elements on a terminal screen, it builds a complete Abstract Syntax Tree (AST) using `pulldown-cmark`. It does this efficiently in a single pass, but the core requirement is maintaining a nested structural tree (`MarkdownNode::Paragraph`, `MarkdownNode::Heading`, etc.). This is a great pattern for *rendering*, but heavy for data extraction.
2. **`Zola` vs. `mdBook`:** `Zola` (a static site generator) streams `pulldown-cmark` events directly into an HTML string via a flat event loop, avoiding AST construction entirely. `mdBook` builds an intermediate AST (`ego_tree::Tree<Node>`) because it must apply complex plugin-driven structural transformations to the document tree before rendering it.
3. **`obsidian-parser` (Frontmatter):** This project correctly recognizes that spinning up a full markdown parser just to find frontmatter is wasteful. It uses a string-slicing approach (`raw_text.lines().next() == "---"`) to separate the YAML block from the markdown body before processing the rest of the text.

### 2.2. Obsidian Data Models
1. **Obsidian's Natively Cached Data (`CachedMetadata`):** Obsidian natively **does not cache a full AST**. Instead, it performs a fast scan and emits flat arrays of metadata (`links`, `tags`, `headings`, `sections`, `listItems`, `frontmatter`). Notably, `sections` are merely root-level block boundaries (e.g. a top-level list, blockquote, or paragraph) tagged with their byte positions, not deeply nested trees. It also tracks `blocks` (`^block-id`s) mapped to specific sections or list items.
2. **Dataview's Index:** Dataview does not cache a full AST either. It extracts and persists only queryable metadata: frontmatter, inline fields (`Key:: Value`), links, tags, and heavily parsed list/task items. For tasks, it caches hierarchy (parent/child line numbers) but stores them in flat arrays. It completely ignores standard paragraph text.

## 3. Analysis: The Source of the Bloat in Lithos

The current `note` module suffers from four primary architectural flaws:

### Flaw 1: The "Double Pass" AST Anti-Pattern (The Basalt Trap)
Lithos is an indexer/extractor, not a renderer. Yet, it currently mimics `basalt` and `mdBook` by building a massive intermediate AST.
1. `parser/mod.rs` iterates over the `pulldown-cmark` event stream to build a deeply nested `ParsedNote` containing thousands of heap-allocated `Node` objects.
2. `parser/extract.rs` instantly recurses over that exact same tree to flatten it into vectors (`headings`, `tags`, `links`).

Building an AST just to flatten it is a massive waste of code, memory allocations, and CPU cycles. This introduces ~600 lines of boilerplate code in `ast.rs` and `note.rs` defining hierarchical structures that are discarded before reaching the domain layer.

### Flaw 2: Over-engineered Frontmatter Extraction
Currently, Lithos uses `pulldown-cmark`'s `ENABLE_YAML_STYLE_METADATA_BLOCKS` to find frontmatter, creating a complex `MetadataBlock` in the AST, which is then parsed in `extract.rs`. This forces the markdown parser state machine to track metadata events and adds dozens of lines of boundary code (`parser/frontmatter.rs`).

### Flaw 3: Hyper-Fragmentation in the `raw` Layer
The `note/raw/` directory contains 13 separate files (e.g., `tags.rs`, `headings.rs`, `links.rs`). These are not complex domain objects with behavior; they are simple Plain Old Java Objects (POJOs) used to transfer data from the parser to the domain aggregate. Scattering these lightweight structs across 13 files makes the module feel unnecessarily huge and highly fragmented.

### Flaw 4: Redundant Text Scanning Passes
After building the AST and extracting sections, Lithos currently runs `BlockRefScanner::new(source).collect()?` which iterates over the *entire raw source file string line-by-line* a completely separate time just to find `^block-refs`.

## 4. Architectural Parity with Obsidian

To reach MVP parity with the Obsidian and Dataview ecosystems, the main aggregate—**`Note`**—must mirror the flat memory model of Obsidian's `CachedMetadata`.

Obsidian's `CachedMetadata` looks like this:
```typescript
interface CachedMetadata {
  links: LinkCache[];
  tags: TagCache[];
  headings: HeadingCache[];
  sections: SectionCache[]; // Root-level blocks only
  listItems: ListItemCache[];
  frontmatter: FrontMatterCache;
  blocks?: Record<string, BlockCache>;
}
```

By adopting a flat, positional architecture (arrays tied to `SourceByteOffset`), we perfectly align with how Obsidian natively caches file features.
*   **Parity Gap - Block Refs:** Currently, `BlockRef`s are extracted but not explicitly tied to the `Section` or `ListItem` they terminate in. This must be resolved to match Obsidian's model.
*   **Deliberate Omission:** Because Lithos focuses purely on data metadata extraction for the MVP, we intentionally omit `footnotes` and `footnoteRefs`.

## 5. Suggested Solutions

To dramatically shrink the `note` module, eliminate intermediate allocations, and lock in exact Obsidian API parity, we will execute the following four solutions:

### Solution A: The "Flat Event Sink" (Eliminate the AST)
We will rewrite the `pulldown-cmark` ingestion phase to act as a **SAX-style Event Sink**, exactly mirroring the `Zola` approach but for extraction rather than HTML rendering.
Instead of building `Node` objects, the `into_offset_iter()` loop will match directly on `Event::Start`, `Event::End`, and `Event::Text`, pushing directly into `Vec<RawHeading>`, `Vec<RawLink>`, `Vec<RawListItem>`, etc.
*   **Hierarchy tracking:** We maintain simple state trackers (like a lightweight integer stack for active list depths, and `text_accumulators` for headings/links) to determine parent/child relationships on the fly, without actually nesting structs in memory.
*   **Impact:** We will completely **delete** `parser/ast.rs` and `parser/note.rs` (~600 lines). Ingestion becomes a lightning-fast $O(N)$ single pass, cutting memory allocations in half.

### Solution B: Keep `pulldown-cmark` for Frontmatter
While `obsidian-parser`'s string-slicing method is conceptually neat, we **must keep** `pulldown-cmark`'s native `ENABLE_YAML_STYLE_METADATA_BLOCKS` instead of string-slicing.
*   **Why:** Lithos relies on absolute byte offsets (`SourceByteOffset`) for zero-copy operations. Slicing the string before passing it to `pulldown-cmark` resets the parser's byte offsets to `0`. We would have to manually perform arithmetic (`offset + frontmatter_length`) on every single event to restore absolute positions, introducing massive risk for offset-corruption bugs.
*   **How:** Let the parser handle the `MetadataBlock` natively to ensure flawless offset integrity, but dump the content directly into `RawFrontmatter` upon the `Event::End` tag rather than building an AST node.

### Solution C: Collapse the `raw` Module
We will consolidate all 13 files inside the `note/raw/` directory into a single `note/raw.rs` (or `note/raw_types.rs`) file. Since these are simple Data Transfer Objects lacking complex logic, they belong in one cohesive location. This eliminates 12 files from the project structure and dramatically simplifies imports across the codebase.

### Solution D: Elevate the Target Aggregate to `Note` and Unify Scanning
1. **The Target Aggregate:** The final domain aggregate representing the parsed file will be named **`Note`**. It will contain the flat vectors extracted directly from the raw phase, perfectly mirroring the `CachedMetadata` structure.
2. **Unified Scanning:** Instead of scanning the whole file line-by-line for block references later, the `BlockRefScanner` will be integrated into the main `pulldown-cmark` event loop. When the loop receives an `Event::Text` or completes a block, it scans *just that text fragment* for `^block-refs` and attaches them directly to the active `Section` or `ListItem` state, closing the final parity gap with Obsidian's API.

---

### Conclusion
By implementing these solutions, the `note` module will shrink by roughly 40%, a dozen files will be deleted, and redundant file scanning will be eliminated. Ingestion will become a highly performant, single-pass $O(N)$ operation yielding a flat data structure identical to Obsidian's native cache model.
