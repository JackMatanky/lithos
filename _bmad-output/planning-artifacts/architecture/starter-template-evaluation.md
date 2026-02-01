---
title: "Starter Template Evaluation"
description: "Evaluation of starter templates and selection of workspace-based hexagonal architecture"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Architecture Evaluation"
---

# Starter Template Evaluation

## Primary Technology Domain

CLI Tool (Rust) - Complex vault templating system with 50 functional requirements requiring hexagonal architecture, CQRS patterns, and async operations.

## Technical Preferences Confirmed

Based on project requirements analysis: Rust 1.70+, async runtime, hexagonal ports/adapters, CQRS for vault operations, embedded storage. Research of Rust ecosystem patterns confirms these as optimal for complex CLI applications with performance requirements and concurrent operations.

## Starter Options Evaluated

**Generic CLI Templates**: Keats/rust-cli-template and similar provide basic clap setup but lack the sophisticated hexagonal organization, CQRS separation, async infrastructure, and domain modeling patterns required for complex vault operations.

**Custom Single-Crate Setup**: Traditional approach but doesn't scale for 50-FR requirements or enable the semi-microservices development pattern you established in Go.

**Resources Reviewed**: Rust-Trends/example_project_structure provides basic layout. Djamware guide offers organizational principles but lacks the architectural depth of your implementation. Your Go source tree demonstrates the gold standard for hexagonal organization.

## Selected Starter: Single-Crate Architecture (Performance Pivot)

**Rationale for Pivot:**
Initially, a 4-crate Hexagonal Workspace (`domain`, `app`, `adapters`, `cli`) was selected. However, implementation analysis revealed that physical crate boundaries prevent **zero-copy optimizations** (specifically `rkyv` inlining) and introduce significant boilerplate for `redb` transactions.

We have pivoted to a **Single-Crate Core Architecture** to prioritize:
1.  **Performance:** 5-10x faster zero-copy reads via compiler inlining.
2.  **Simplicity:** Reduced boilerplate, simpler ownership, and faster compilation.
3.  **Idiomatic Rust:** Aligns with ecosystem standards (tokio, clap) where "library + binary" is the preferred pattern.

**Revised Structure:**

- **`lithos-core`:** Single library crate containing Domain, App, and Infrastructure logic.
  - Boundaries enforced via `pub(crate)` visibility.
  - Dependencies flow INWARD (`db.rs` -> `domain/`).
- **`lithos-cli`:** Thin binary driver for terminal UI.

**Initialization Commands:**

```bash
# Create workspace root
mkdir lithos && cd lithos
# Core logic (Domain + Infra)
cargo new crates/lithos-core --lib
# CLI driver
cargo new crates/lithos-cli --bin
```

**Workspace Cargo.toml:**

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
# ... dependencies ...
```

**Architectural Decisions (Revised):**

- **Module-based Hexagonal:** Logical separation via modules (`note/`, `schema/`, `db.rs`) instead of physical crates.
- **Sync-First:** Core logic is synchronous; async is restricted to CLI/LSP edges.
- **Zero-Copy Native:** `db.rs` exposes `rkyv` types directly to the domain.

**Development Benefits:**

- **Performance:** Zero-copy read paths are compile-time optimized.
- **Velocity:** Faster builds (less monomorphization overhead) and easier refactoring.
- **Focus:** Less time fighting the borrow checker across crate boundaries.
