# Clippy Quality Standards & AI Safeguards

This document outlines the stringent clippy linting standards for the Lithos Rust project, specifically designed to maintain high code quality in an AI-assisted development environment.

## Overview

We use a highly restrictive clippy configuration to prevent common Rust anti-patterns and ensure that AI-generated code follows enterprise-grade best practices.

## Core Thresholds

- **Cognitive Complexity**: Hard limit of **25** (deny). Functions exceeding this must be refactored.
- **Function Length**: Hard limit of **100 lines** (deny).
- **MSRV**: Minimum Supported Rust Version is **1.85**.

## Mandatory Deny Lints

The following categories are strictly enforced at the `deny` level:

- `correctness`: Basic logic and type safety errors.
- `suspicious`: Code that likely contains a bug or unintended behavior.
- `pedantic`: Best practice lints for idiomatic Rust.
- `restriction`: Highly opinionated lints to prevent risky patterns (e.g., `unwrap_used`, `panic`, `todo`).

### Specific Prohibited Patterns

- **No Unwraps/Expects**: Use structured error handling (`Result`, `Option`) with `thiserror` or `anyhow`.
- **No Panics**: Avoid `panic!`, `todo!`, `unimplemented!`, and `unreachable!`.
- **No Debug Prints**: `dbg!` and `println!` are prohibited in production code. Use `tracing` for logging.
- **Safe Arithmetic**: Arithmetic that can overflow is denied; use checked/saturated arithmetic or justify the use of wrapping.
- **Safe Indexing**: Direct indexing (`[index]`) is discouraged; use `.get()` or `.first()`.

## Lint Disable Policy

Disabling a lint is an **absolute last resort**. AI agents and human developers must exhaust all refactoring options before using `#[allow(...)]`.

### Audit Trail Requirement

Every lint disable MUST be accompanied by a structured comment in the following format:

```rust
// # LINT_DISABLE_REASON: [Short reason for disable]
// | Options tried: [List of refactoring attempts that failed]
// | Justification: [Detailed explanation of why disabling is the only viable option]
#[allow(clippy::lint_name)]
```

### AI Training

AI agents are expected to:
1. Recognize these standards and proactively fix violations.
2. Suggest refactoring patterns (e.g., splitting functions, using `map_err`) instead of suggesting disables.
3. Automatically generate the audit trail if a disable is truly necessary.

## Workflow Integration

- **Mise**: Run `mise run verify` to check lints before any major change.
- **Pre-commit**: Hooks automatically run clippy on every commit. Bypassing these hooks is prohibited.
