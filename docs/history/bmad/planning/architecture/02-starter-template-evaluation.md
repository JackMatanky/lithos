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

CLI Tool (Rust) - Complex vault templating system with 50 functional requirements requiring modular architecture with isolated business contexts, unified Repository traits, and async operations.

## Technical Preferences Confirmed

Based on project requirements analysis: Rust 1.70+, async runtime, modular architecture with isolated business contexts, embedded storage with files as source of truth. Research of Rust ecosystem patterns (Cargo, mdBook, Zola, rust-analyzer) confirms these as optimal for file-based CLI applications with performance requirements.

## Starter Options Evaluated

**Generic CLI Templates**: Keats/rust-cli-template and similar provide basic clap setup but lack the sophisticated modular organization, context isolation, async infrastructure, and domain modeling patterns required for complex vault operations.

**Custom Single-Crate Setup**: Traditional approach but doesn't scale for 50-FR requirements or enable the semi-microservices development pattern you established in Go.

**Resources Reviewed**: Rust-Trends/example_project_structure provides basic layout. Djamware guide offers organizational principles but lacks the architectural depth of our implementation. Research into Rust ecosystem file-based projects (Cargo, mdBook, Zola, rust-analyzer) informed our files-as-source-of-truth approach with unified Repository traits.

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
  - **Cross-Cutting Context:** config (user-configurable business rules)
  - **Pure Infrastructure:** db, fs, patterns (generic utilities)
  - **Boundaries:** Enforced via:
    - Visibility modifiers (`pub(crate)`, `pub`)
    - Unified Repository traits (single trait per context combining reads and writes)
    - Architecture tests (validate no forbidden cross-context imports)
  - **Dependencies flow INWARD:** Infrastructure (db, fs, config) ← Business Contexts (note, schema, template) ← CLI
  - **Repository Pattern:** Each context defines unified `Repository` trait with multiple implementations (Redb, InMemory, Fake)
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

- **Module-based Boundaries:** Logical separation via modules and isolated contexts (`note/`, `schema/`, `db/`) instead of physical crates.
- **Unified Repository Traits:** Each context defines single `Repository` trait combining reads and writes (no CQRS split).
- **Files as Source of Truth:** Markdown files in vault are authoritative; database is rebuildable projection/cache.
- **Sync-First:** Core logic is synchronous; async is restricted to CLI/LSP edges.
- **Zero-Copy via Closures:** Repository traits use closure-based `with_archived()` for zero-copy reads.
- **Optional View Pattern:** Introduce `*View` types only when domain shape is inefficient for storage (per ADR 003).

**Repository Pattern for Decoupling:**

While the single-crate structure enables zero-copy performance, we maintain architectural boundaries through:

1. **Unified Repository Traits:** Each context defines `<context>::Repository` trait with reads (`get`, `list`, `with_archived`) and writes (`save`, `delete`)
2. **Multiple Implementations:** `RedbStorage`, `InMemoryStorage`, `FakeStorage` per context
3. **Closure-Based Access:** `with_archived<F, R>(&self, id, f: F)` provides zero-copy without lifetime leakage
4. **Test Substitution:** `FakeStorage` for unit tests without real database

This provides:
- ✅ Decoupling (can swap backends via trait implementations)
- ✅ Zero-copy performance (via closure-based archived reads)
- ✅ Testability (trait substitution with InMemory/Fake implementations)
- ✅ Static dispatch (concrete implementations, no `dyn Trait` overhead)

**Development Benefits:**

- **Performance:** Zero-copy read paths are compile-time optimized.
- **Velocity:** Faster builds (less monomorphization overhead) and easier refactoring.
- **Focus:** Less time fighting the borrow checker across crate boundaries.
- **Testability:** Port-based design enables easy test substitution.
