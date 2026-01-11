# ADR 0004: High-Performance Markdown Parsing with pulldown-cmark

## Status
Accepted

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

- **Native Extensions:** We will enable the following `Options`:
    - `ENABLE_WIKILINKS`: Direct support for `[[target|alias]]` resolution.
    - `ENABLE_METADATA_BLOCKS`: Native support for YAML/TOML frontmatter blocks.
    - `ENABLE_TASKLISTS`: For checklist tracking in the graph.
    - `ENABLE_STRIKETHROUGH`, `ENABLE_TABLES`, `ENABLE_HEADING_ATTRIBUTES`.

- **Custom Implementations:**
    - **Tags:** Since `pulldown-cmark` does not natively support `#tag` detection (treating them as text), we will implement a custom scanner or regex-based event filter to extract tags for the knowledge graph.
    - **Callouts:** Handled as standard BlockQuotes via the parser and transformed into Obsidian-style callouts at the rendering/LSP layer based on the `[!type]` prefix.
    - **Backlinks:** Populated by querying the `links_back` table in Redb, which is filled during the initial parse pass.

## Evaluated Alternatives

| Feature | **pulldown-cmark** | **comrak** | **markdown-rs** |
| :--- | :--- | :--- | :--- |
| **Parsing Model** | **Pull Event Stream** | Full AST | Full AST (mdast) |
| **Indexing Speed** | **Ultra-High** | High | Moderate |
| **Memory Overhead** | **Minimal** | Moderate | High |
| **Zero-Copy Support**| **Native** | Limited | Limited |
| **Extensibility** | Wrapper-based | Built-in extensions | Plugin-based (Unified) |

## Rationale

### Why not Comrak?
While `comrak` provides a rich AST that makes deep document manipulation (like link renaming) easier, its memory and CPU overhead are significantly higher. For a tool like Lithos, which spends 99% of its time reading and indexing, the raw speed of a stream-based parser is a better trade-off. We can always perform surgical string replacements for refactoring using the byte-offsets provided by `pulldown-cmark`.

### Why not markdown-rs?
`markdown-rs` is a faithful port of the JavaScript `unified.js` ecosystem. While robust, it is significantly heavier and slower than `pulldown-cmark`. The complexity of the `unified` ecosystem is unnecessary for the focused CLI and LSP goals of Lithos.

## Consequences
-   **Custom Logic for Wikilinks:** We must maintain a small "interceptor" layer in the `adapters` crate to handle Obsidian-specific syntax that falls outside the CommonMark spec.
-   **Surgical Refactoring:** Instead of manipulating an AST to rename links, we will use the byte-offsets from `pulldown-cmark` to perform precise string slices and replacements in the original source, which is actually more efficient and preserves user formatting (like whitespace and comments) better than an AST-to-String roundtrip.
