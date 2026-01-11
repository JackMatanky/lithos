# ADR 0004: High-Performance Markdown Parsing with pulldown-cmark

*   **Status**: Accepted
*   **Date**: 2026-01-11
*   **Stakeholders**: Jack (Developer), Architects

## Context

Lithos requires a Markdown parser that serves as the primary "scanner" for the vault. The system's performance is tied directly to how quickly it can process thousands of files to:
1.  **Extract Links:** Identify Wikilinks and Markdown links to populate the Redb knowledge graph.
2.  **LSP Rendering:** Convert Markdown snippets to HTML or formatted text for real-time hover documentation.
3.  **Frontmatter Extraction:** Separate YAML blocks from the body content without redundant scanning.
4.  **Performance:** Maintain a minimal memory footprint and leverage zero-copy operations wherever possible.

## Decision

We will use **pulldown-cmark** as the core Markdown processing engine for Lithos Rust.

### 1. Performance: Pull-based Event Streaming
Unlike AST-based parsers (Comrak, markdown-rs), `pulldown-cmark` uses a pull-based event stream. This allows us to:
-   **Scan in a Single Pass:** Extract links, headings, and metadata without ever building a full in-memory tree representation of the document.
-   **Zero-Copy Potential:** The parser returns events that hold references (string slices) to the original source text, minimizing allocations during the critical indexing phase.

### 2. LSP and Mechanical Sympathy
In alignment with **ADR 0002 (Redb/rkyv)** and **ADR 0003 (MiniJinja)**, `pulldown-cmark` is optimized for speed and low overhead. Its ability to render small snippets of Markdown to HTML in sub-millisecond time is essential for the fluid user experience required by the LSP.

### 3. Obsidian Compatibility Strategy
Lithos leverages `pulldown-cmark`'s native extension support while implementing custom handlers for Obsidian-specific syntax:

- **Native Extensions:**
    - `ENABLE_WIKILINKS`: Direct support for `[[target|alias]]` resolution.
    - `ENABLE_METADATA_BLOCKS`: Native support for YAML/TOML frontmatter blocks.
    - `ENABLE_TASKLISTS`: For checklist tracking in the graph.
    - `ENABLE_STRIKETHROUGH`, `ENABLE_TABLES`, `ENABLE_HEADING_ATTRIBUTES`.

- **Custom Implementations:**
    - **Tags:** Handled via a custom scanner or regex-based event filter.
    - **Callouts:** Transformed from BlockQuotes based on the `[!type]` prefix.
    - **Backlinks:** Populated by querying the `links_back` table in Redb.

## Alternatives Considered

| Feature | **pulldown-cmark** | **comrak** | **markdown-rs** |
| :--- | :--- | :--- | :--- |
| **Parsing Model** | **Pull Event Stream** | Full AST | Full AST (mdast) |
| **Indexing Speed** | **Ultra-High** | High | Moderate |
| **Memory Overhead** | **Minimal** | Moderate | High |
| **Zero-Copy Support**| **Native** | Limited | Limited |

### Why not Comrak?
While `comrak` provides a rich AST that makes deep document manipulation easier, its memory and CPU overhead are significantly higher. For Lithos, which spends 99% of its time reading and indexing, the raw speed of a stream-based parser is a better trade-off.

### Why not markdown-rs?
`markdown-rs` is significantly heavier and slower than `pulldown-cmark`. The complexity of the `unified` ecosystem is unnecessary for the focused CLI and LSP goals of Lithos.

## Technical Validation

### Research Findings
- **Pull vs AST**: Research shows that pull-based parsers avoid the allocation of thousands of small nodes required for a tree, which is the primary bottleneck in large vault indexing.
- **Surgical Refactoring**: Instead of manipulating an AST to rename links, we will use the byte-offsets from `pulldown-cmark` to perform precise string slices in the original source, preserving user formatting (whitespace/comments) better than an AST-to-String roundtrip.

### Compatibility & Performance
- **Hexagonal Alignment**: Isolated in the `adapters/spi/markdown` layer.
- **Performance Impact**: Critical for the <2s 1000-file indexing target and sub-millisecond hover rendering.

## Consequences

*   **Positive**: Ultra-fast indexing, zero-copy possible, small memory footprint, formatting preservation.
*   **Negative**: Surgical refactoring requires byte-offset math instead of simple AST node updates.

## Status Tracking

*   **Proposed**: 2026-01-08
*   **Accepted**: 2026-01-11
*   **Implemented**: 2026-01-11
