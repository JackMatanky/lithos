# PRD: DocumentParser Extraction

**Status**: drafting
**Created**: 2026-05-26
**Context**: The `template` and `note` modules require robust parsing to split YAML frontmatter from Markdown bodies. Currently, parsing logic (like `parse_structured` and `classify_path`) is glued to `FileReader`, which mixes pure data transformation with I/O. This PRD outlines the extraction of a pure-data `DocumentParser` seam.

---

## Problem Statement

File parsing and format classification are currently tightly coupled to `FileReader`. This creates several issues:
1. **I/O Coupling**: Pure string parsing cannot easily be done without bringing in the file system reader.
2. **Missing Markdown Extraction**: The current parser doesn't handle splitting YAML frontmatter from Markdown bodies, which is a hard requirement for the new template architecture (as defined in `.scratch/template-module-refactor/PRD.md`) and the note ingestion pipeline.
3. **Bloated Reader**: `FileReader` is taking on too many responsibilities, violating the single-responsibility principle.

## Solution

Extract parsing and classification into a dedicated `DocumentParser` module (e.g., in `src/fs/parser.rs` or `src/parser/`).

The parser will act as a pure-data seam:
- It takes a `&str` (the file contents).
- It uses `pulldown-cmark` to cheaply slice the text, extracting the YAML frontmatter block without building a heavy AST for the entire document.
- It returns structured frontmatter and a raw Markdown body string.

---

## User Stories

1. As a template author, I want my YAML frontmatter cleanly separated from the Markdown body, so that `minijinja` is handed a pure Markdown string without YAML interference.
2. As a parser developer, I want format classification and structured parsing completely decoupled from I/O, so I can parse in-memory strings easily in tests.
3. As a system architect, I want a single, robust markdown extraction utility shared by both the `template` and `note` modules, avoiding duplicated parsing logic.

---

## Implementation Decisions

### Phase 1: Decoupling Existing Parsing
- Extract `parse_structured`, `parse_structured_from_str`, and `classify_path` from `FileReader`.
- Move format classification logic into a pure module (`src/fs/parser.rs` or `src/parser/`).

### Phase 2: DocumentParser Implementation
- Implement a `DocumentParser` that uses `pulldown-cmark`.
- Configure the parser to consume only the initial YAML block.
- Yield the parsed frontmatter and the untouched raw Markdown body as a string slice.
- Ensure the parser is pure (no I/O, no path validation).

## Blast Radius & Impact

- **Direct Callers**: Any component currently calling `FileReader::parse_structured` (e.g., `ConfigBuilder`, `SchemaProcessor`).
- **Risk**: LOW/MEDIUM. The parsing logic itself isn't changing for structured formats (JSON/TOML), but the API is moving. The Markdown extraction is net-new functionality that will unblock the template refactor.

## Testing Strategy
- **Pure Parsing**: Write unit tests for `DocumentParser` that pass raw strings containing YAML frontmatter and Markdown bodies. Verify that the frontmatter is correctly extracted and the body perfectly preserves whitespace and Markdown formatting.
- **Classification**: Verify `classify_path` still correctly identifies formats based on extensions and content sniffing.
