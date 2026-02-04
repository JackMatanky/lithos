---
title: "Starter Template Evaluation"
description: "Evaluation of starter templates and selection of single-crate port-based architecture"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-04"
section: "Architecture Evaluation"
---

# Starter Template Evaluation

## Primary Technology Domain

CLI Tool (Rust) - Complex vault templating system with 50 functional requirements requiring port-based CQRS patterns, bounded contexts, and async operations.

## Technical Preferences Confirmed

Based on project requirements analysis: Rust 1.70+, async runtime, port-based CQRS with bounded contexts, embedded storage. Research of Rust ecosystem patterns confirms these as optimal for complex CLI applications with performance requirements and concurrent operations.

## Starter Options Evaluated

**Generic CLI Templates**: Keats/rust-cli-template and similar provide basic clap setup but lack the sophisticated bounded context organization, port-based CQRS separation, async infrastructure, and domain modeling patterns required for complex vault operations.

**Custom Single-Crate Setup**: Traditional approach but doesn't scale for 50-FR requirements or enable the semi-microservices development pattern you established in Go.

**Resources Reviewed**: Rust-Trends/example_project_structure provides basic layout. Djamware guide offers organizational principles but lacks the architectural depth of your implementation. Research into Rust ecosystem (rust-analyzer, diesel, Polars) informed the port-based CQRS approach.

## Selected Starter: Single-Crate Architecture (Performance Pivot)

**Rationale for Pivot:**
Initially, a traditional multi-crate workspace with separate `domain`, `app`, `adapters`, `cli` crates was considered. However, implementation analysis revealed that physical crate boundaries prevent **zero-copy optimizations** (specifically `rkyv` inlining) and introduce significant boilerplate for `redb` transactions.

We have pivoted to a **Single-Crate Core Architecture** to prioritize:
1.  **Performance:** 5-10x faster zero-copy reads via compiler inlining.
2.  **Simplicity:** Reduced boilerplate, simpler ownership, and faster compilation.
3.  **Idiomatic Rust:** Aligns with ecosystem standards (tokio, clap) where "library + binary" is the preferred pattern.

**Revised Structure:**

- **`lithos-core`:** Single library crate containing Domain, Infrastructure, and Storage logic.
  - **Business Contexts:** note, schema, template (isolated from each other)
  - **Cross-Cutting:** config, db, fs, patterns (available to all contexts)
  - **Boundaries:** Enforced via:
    - Visibility modifiers (`pub(crate)`, `pub`)
    - Port-based CQRS (traits with GATs)
    - Architecture tests (validate no forbidden cross-context imports)
  - **Dependencies flow INWARD:** Infrastructure (db, fs, config, patterns) ← Business Contexts (note, schema, template) ← CLI
  - **Port Pattern:** CQRS types generic over storage port for decoupling + performance
- **`lithos-cli`:** Thin binary driver for terminal UI and orchestration.

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

- **Module-based Boundaries:** Logical separation via modules and bounded contexts (`note/`, `schema/`, `db/`) instead of physical crates.
- **Port-Based CQRS:** Each context defines storage port trait, CQRS types generic over port.
- **Sync-First:** Core logic is synchronous; async is restricted to CLI/LSP edges.
- **Zero-Copy via GATs:** Port traits use GATs for closure-based archived reads without leaking transaction lifetimes.
- **Storage DTOs:** `Stored*` types (per ADR 0009 Appendix A) isolate rkyv coupling from domain.

**Port-Based Decoupling:**

While the single-crate structure enables zero-copy performance, we maintain architectural boundaries through:

1. **Storage Port Traits:** Each context defines `<Context>Store` trait with GATs
2. **Generic CQRS:** `Query<S: SchemaStore>` decouples from concrete database
3. **Type Aliases:** `RedbSchemaQuery<'db>` hides generic complexity for callers
4. **Test Substitution:** `FakeSchemaStore` for unit tests without real database

This provides:
- ✅ Decoupling (can swap backends)
- ✅ Zero-copy performance (via GAT-based archived reads)
- ✅ Testability (trait substitution)
- ✅ Static dispatch (when using concrete type aliases)

**Development Benefits:**

- **Performance:** Zero-copy read paths are compile-time optimized.
- **Velocity:** Faster builds (less monomorphization overhead) and easier refactoring.
- **Focus:** Less time fighting the borrow checker across crate boundaries.
- **Testability:** Port-based design enables easy test substitution.
