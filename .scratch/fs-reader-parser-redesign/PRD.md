# PRD: FsReader Redesign and DocumentParser Extraction

**Status**: drafting
**Created**: 2026-05-26
**Context**: `FsReader` has become a shallow god object that mixes file I/O, directory traversal, and structured data parsing. This PRD outlines the strategy to deepen the architecture by splitting these responsibilities into distinct seams: `FileReader`, `DirScanner`, and `DocumentParser`.

---

## Problem Statement

Currently, `FsReader` provides an interface that is nearly as complex as its implementation. It violates single-responsibility by managing:
1. **File I/O**: Reading bytes/strings and fetching metadata.
2. **Directory Scanning**: Glob-based filtering and traversal (`filter_entries`, etc.).
3. **Format Parsing**: Sniffing formats and parsing structured data (`parse_structured`, `classify_path`).

This creates significant architectural friction:
- **Poor Leverage**: Consumers like `VaultProcessor` and `SchemaProcessor` must depend on the entire `FsReader` surface area even if they only need one capability.
- **Poor Locality**: Parsing bugs, traversal bugs, and I/O bugs all funnel back to `fs/reader.rs`.
- **Blocked Refactoring**: As defined in the template-module-refactor PRD, the template and note modules require a robust Markdown parser (via `pulldown-cmark`) to split YAML frontmatter from bodies without I/O. Leaving parsing mixed with `FsReader` prevents this clean extraction.

## Solution

We will decouple I/O, traversal, and parsing into three distinct boundaries:

1. **`FileReader` (I/O)**: A stripped-down version of `FsReader` that only handles reading file contents and extracting metadata scoped to a validated root.
2. **`DirScanner` (Traversal)**: A standalone component dedicated to discovering files and directories (already partially implemented, but needs to be explicitly injected).
3. **`FsContext` (Application Service)**: A bundle struct passed to processors (`VaultProcessor`, `SchemaProcessor`) containing both `FileReader` and `DirScanner` to prevent excessive dependency injection boilerplate.
4. **`DocumentParser` (Parsing)**: A pure-data seam that consumes a `&str` and returns structured frontmatter + raw Markdown body. It uses `pulldown-cmark` for zero-IO parsing and format classification.

---

## User Stories

1. As a system architect, I want file reading and directory scanning to be separate adapters, so that I can mock them independently in tests.
2. As a system architect, I want `VaultProcessor` and `SchemaProcessor` to accept an `FsContext` application service, so that injecting multiple FS adapters is ergonomic.
3. As a parser developer, I want format classification and structured parsing completely decoupled from I/O, so I can parse in-memory strings easily.
4. As a template author, I want frontmatter cleanly separated from the Markdown body via `pulldown-cmark`, so that `minijinja` is handed a pure Markdown string without YAML interference.

---

## Implementation Decisions

### Phase 1: Injection & Traversal Decoupling (FsContext)
- Strip all `filter_*` methods (`filter_entries`, `filter_file_paths`, etc.) from `FsReader`.
- Rename `FsReader` to `FileReader`.
- Create an `FsContext` struct (likely in `src/fs/context.rs`) that composes `FileReader` and `DirScanner`.
- Refactor `VaultProcessor`, `SchemaProcessor`, and `DiscoveryEngine` to accept `FsContext` instead of `FsReader`.

### Phase 2: Parsing Decoupling (DocumentParser)
- Extract `parse_structured`, `parse_structured_from_str`, and `classify_path` from `FileReader`.
- Move format classification logic into a pure module (`src/fs/parser.rs` or `src/parser/`).
- Implement a `DocumentParser` that uses `pulldown-cmark` to split YAML frontmatter from Markdown body, satisfying the `template-module-refactor` PRD.
- Ensure the parser is pure (no I/O, no path validation).

## Blast Radius & Impact

- **Direct Callers**: `scan_views` in `vault/processor.rs`, `parse` in `schema/schema_processor.rs`, and configuration discovery engines.
- **Risk**: MEDIUM. The logic isn't changing fundamentally, but the interfaces are. We must ensure `FsContext` provides the exact same leverage that `FsReader` previously provided, just across a cleaner seam.

## Testing Strategy
- **Deletion Test**: Verify that replacing `FsReader` with `FileReader` + `DirScanner` doesn't leak traversal complexity into the caller. The `FsContext` should absorb this.
- **Pure Parsing**: Write unit tests for `DocumentParser` that pass raw strings containing YAML frontmatter and Markdown bodies, ensuring pure data transformation without temporary files.
