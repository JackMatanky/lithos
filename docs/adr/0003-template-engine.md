# ADR 0003: Selection of Template Engine for Markdown-based Templates

## Status
Accepted

## Context
Lithos templates are primarily Markdown files containing YAML frontmatter and body content with embedded logic. The engine must support high-frequency rendering for a Command Line Interface (CLI) and a future Language Server Protocol (LSP).

Key technical challenges for Markdown templates include:
1.  **Frontmatter Integrity**: Template logic must render within YAML blocks without breaking indentation or syntax (strict whitespace control).
2.  **Markdown Body Safety**: Special characters (`#`, `*`, `[`, `]`, `<`, `&`) must be handled without accidental HTML escaping that would break Markdown rendering.
3.  **Complex Logic in Docs**: Rendering loops and conditionals inside whitespace-sensitive structures like tables and lists.
4.  **Performance**: The LSP requires sub-50ms rendering for thousands of small snippets during real-time interaction.

## Decision
We will use **MiniJinja** as the primary template engine for Lithos Rust.

### 1. Engine Specifics for Markdown
- **Configurable Auto-Escaping**: Unlike Tera, which defaults to HTML escaping, MiniJinja allows us to configure the `Environment` with a custom auto-escape callback. We will disable escaping for `.md` and `.yaml` templates by default while maintaining the ability to mark specific data as `Safe`.
- **Whitespace Control**: Full support for Jinja2-style whitespace stripping (`{%- ... -%}`), essential for preserving YAML frontmatter alignment and Markdown table pipes.
- **Low-Overhead Snippets**: Optimized for fast compilation and rendering, making it ideal for the real-time feedback loops required by the LSP.

### 2. Evaluated Alternatives

| Feature | **MiniJinja** | **Tera** | **Handlebars** |
| :--- | :--- | :--- | :--- |
| **Performance (LSP)** | **Ultra-High** | High | Medium-High |
| **Binary Size** | **Minimal (~50KB)** | Moderate | Moderate |
| **Markdown Escaping** | **Native/Custom** | Strict (HTML) | Helper-Dependent |
| **Whitespace Control** | **Excellent** | Good | Limited |
| **Logic Capability** | High (Jinja2) | High (Jinja2) | Low (Logic-less) |
| **Ergonomics** | **Excellent** | Excellent | Good |

## Rationale

### Why not Tera?
Tera's strict HTML-centric defaults are cumbersome for a Markdown tool. Disabling auto-escaping in Tera is globally handled and less flexible than MiniJinja's environment-level callbacks. Furthermore, MiniJinja’s smaller binary footprint and faster rendering align with the **ADR 0002** goal of "Mechanical Sympathy."

### Why not Handlebars?
Handlebars' "logic-less" philosophy requires excessive custom helpers for common PKM operations, such as conditionally formatting list items or complex schema-driven table generation. This leads to a fragmented and verbose template codebase.

### The "Markdown Safety" Advantage
MiniJinja allows defining "Safe" vs "Raw" strings at the application level. Lithos can pass schema-validated Markdown snippets into a template and ensure they aren't mangled by the engine, while still protecting against accidental injection in other contexts.

## Consequences
- **Jinja2 Familiarity**: Users coming from the Go implementation or other PKM tools (like Templater/Nunjucks) will find the syntax intuitive.
- **Environment Management**: We must implement a custom `Environment` manager in the `adapters` crate to handle the switching of escaping rules based on file extensions.
- **Crate Size**: Using MiniJinja keeps our final binary size significantly smaller than using Tera, aiding in rapid CLI distribution and toolchain integration.
