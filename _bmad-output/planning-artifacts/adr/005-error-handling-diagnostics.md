# ADR 005: Error Handling and Diagnostics Framework

## Status
Accepted

## Context
Lithos Rust must provide exceptional feedback to users across two primary interfaces:
1.  **CLI:** Human-readable, colorized output with code snippets highlighting the exact location of schema or syntax violations in Markdown and YAML files.
2.  **LSP:** Structured diagnostic objects compatible with the Language Server Protocol's `Diagnostic` type (range, severity, message, and related information).

We need to categorize failures into three distinct classes:
*   **Domain Errors:** Internal logical failures (e.g., `SchemaNotFound`, `CircularInheritance`).
*   **Infrastructure Errors:** System-level failures (e.g., `DiskFull`, `PermissionDenied`).
*   **User Diagnostics:** Actionable feedback on user-provided content (e.g., `InvalidFrontmatter`, `MissingRequiredField`, `BrokenLink`).

The project requires a solution that minimizes "error erasure" (losing type information) while maximizing "visual fidelity" for the user.

## Decision
The project will adopt **miette** as the primary diagnostic framework, layered over **thiserror** for defining structured error enums.

### Tiered Error Model
- **thiserror (v2.0):** Used to define the underlying error types and derive `std::error::Error` and `Display`. This ensures the domain layer remains pure and its errors are programmatically matchable.
- **miette (v7.6):** Used to add diagnostic metadata via `#[derive(Diagnostic)]`. This includes error codes (e.g., `L001`), help text, documentation URLs, and `SourceSpan` labels.
- **anyhow:** Explicitly avoided in the core library crates to maintain strict type safety and structured diagnostics. It may be used sparingly in the `main` CLI entry point for catching unexpected global panics.

## Rationale

### 1. High-Fidelity Diagnostics (The "Golden Standard")
`miette` is the industry leader for "fancy" terminal diagnostics. It allows us to define `SourceSpan`s that point to exact byte offsets in a file. `miette` handles the rendering of these snippets—complete with red underlines and descriptive labels—automatically. This is essential for a tool that validates user-written Markdown.

### 2. LSP Synergy
The `miette::Diagnostic` trait exposes fields like `severity()`, `code()`, `labels()`, and `help()`. These map 1:1 to the requirements of the Language Server Protocol's `Diagnostic` object. By implementing `miette`, we ensure our CLI and LSP provide identical, high-quality feedback without redundant mapping logic.

### 3. Actionable Remediation
`miette` encourages adding a `help` field to errors. We will use this to provide clear "How to fix" instructions (e.g., "Add 'type: contact' to your frontmatter to resolve this"), which is a key usability requirement for Lithos.

## Consequences
- **Developer Responsibility:** Developers must track `SourceSpan` offsets during parsing (e.g., using `pulldown-cmark`'s byte-offsets) to provide high-quality snippets.
- **Dependency Profile:** `miette` and `thiserror` are standard Rust crates with good performance profiles, aligning with our "Mechanical Sympathy" goals.
- **Consistency:** Error feedback will be consistent across all subcommands and the LSP, as they will all share the same structured diagnostic types.
