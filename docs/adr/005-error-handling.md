---
name: error-handling-and-diagnostics-framework
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-08
date_decided: 2026-01-11
date_implemented: 2026-01-11
---

# ADR 005: Error Handling and Diagnostics Framework

## Context

Lithos Rust must provide exceptional feedback across CLI and LSP interfaces. We need to categorize failures (Domain, Infrastructure, User Diagnostics) while maximizing visual fidelity (code snippets, colors) and minimizing error erasure.

## Decision

The project will adopt **miette** as the primary diagnostic framework, layered over **thiserror** for defining structured error enums.

### Tiered Error Model

- **thiserror (v2.0)**: Used to define the underlying error types (Domain, Infrastructure) and ensure they are programmatically matchable.
- **miette (v7.6)**: Adds diagnostic metadata (codes, help, `SourceSpan` labels) for User Diagnostics.
- **anyhow**: Explicitly avoided in core library to maintain type safety; used sparingly in the CLI main loop for global panic catching.

## Alternatives Considered

### anyhow for everything

- **Pros**: Easy to use, ergonomic context chaining.
- **Cons**: Erases type information, making it impossible for the LSP to categorize errors; lacks the high-fidelity snippet rendering of miette.

### color-eyre

- **Pros**: Beautiful terminal output.
- **Cons**: Less suitable for the LSP interface; miette's structured `Diagnostic` trait maps more cleanly to LSP requirements.

## Technical Validation

### Research Findings

- **LSP Synergy**: `miette::Diagnostic` exposes fields (severity, code, labels, help) that map 1:1 to the Language Server Protocol's `Diagnostic` object, ensuring CLI/LSP consistency.
- **Visual Fidelity**: Research into `miette`'s `SourceSpan` confirmed it can render red underlines and descriptive labels automatically, which is the "golden standard" for validating user content.

### Compatibility & Performance

- **Hexagonal Alignment**: `thiserror` keeps the domain clean; `miette` metadata is applied where the diagnostic is finally rendered.
- **Actionable Remediation**: Using the `help` field allows providing clear "How to fix" instructions directly in the terminal.

## Consequences

- **Positive**: High-fidelity terminal output, seamless LSP integration, consistent error feedback, actionable help for users.
- **Negative**: Requires developers to track `SourceSpan` offsets during parsing (e.g., using `pulldown-cmark` byte-offsets).
