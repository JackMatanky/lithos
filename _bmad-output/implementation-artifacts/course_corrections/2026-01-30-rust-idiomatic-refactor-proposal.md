# Sprint Change Proposal: Rust Idiomatic Architecture Refactor

**Status:** Draft
**Date:** 2026-01-30
**Trigger:** Language-Agnostic Anti-Pattern Realization
**Scope:** MAJOR / STOP-THE-WORLD
**Author:** BMad Master (Architect Agent)

---

## 1. Issue Summary

### The "Translation Gap" Anti-Pattern

The project is currently suffering from a fundamental architectural misalignment: **treating Rust as a generic implementation detail rather than a distinct system.**
We applied high-level patterns (Hexagonal Architecture) directly from other languages (Go/Java) without translating them into Rust-specific idioms.

- **The Error:** We mapped logical boundaries to physical compilation units (Crates) instead of using Rust's native encapsulation primitives (Modules/Visibility).
- **The Symptom:** Struggles with cache implementation and ownership were merely friction caused by fighting the language's grain.
- **The Result:** A structure that actively fights against Rust's zero-cost abstractions, resulting in poor performance and high friction.

### Specific Technical Triggers

1.  **Zero-Copy Violation:** Multi-crate boundaries prevent compiler inlining, causing a **5-10x performance penalty** on zero-copy reads.
2.  **Async Abuse:** "Async Everything" default creates unnecessary complexity and runtime overhead for CPU-bound/local-IO tasks.
3.  **Ownership Hell:** Complex `Arc` wrapping required to pass types across crates, compromising safety and ergonomics.

---

## 2. Impact Analysis: The Collapse of Layers

**⚠️ CRITICAL WARNING: FUNDAMENTAL RE-ARCHITECTURE**

This change represents a **total structural reset** of the codebase. It is not a refactor; it is a migration to a new architectural paradigm. The scale of this change classifies it as a **Stop-the-World Event**—no feature work can proceed until this transition is complete and stable.

### 1. The Collapse of Layers (Architecture Shift)

We are moving from a strict **Interface-Defined Architecture** (Hexagonal) to a **Data-Flow Architecture**.

- **Old World:** Domain is isolated. Adapters depend on Domain. Explicit "App" layer orchestrates.
- **New World:** Boundaries are permeable for performance. Use Cases are concrete command pipelines. The distinct "App" layer is absorbed into Core/Commands.
- **Trade-off:** We prioritize **Zero-Copy Performance** (direct rkyv usage) over **Academic Purity** (backend pluggability).

### 2. Physical Scope: "The Great Migration"

- **Total File Displacement**: **100% of source code** in `crates/domain`, `crates/app`, and `crates/adapters` will be physically moved.
  - _Old:_ `crates/domain/src/...`
  - _New:_ `crates/lithos-core/src/domain/...` (logical module)
- **Import Apocalypse**: Every single file in the project will require import rewrites.
  - External crate imports (e.g., `use domain::...`) must become internal module imports (`use crate::...` or `use lithos_core::...`).
  - `use` statements across the entire test suite and benchmarks will break and require manual reconstruction.
- **Workspace Demolition**: The root `Cargo.toml` workspace definition will be deleted and recreated. The build graph is being fundamentally altered.

### 3. Logical Scope: Core Principles, Architecture & Patterns

- **Boundary Enforcement Shift**: We are moving from **Physical Enforcement** (compiler-enforced crate boundaries) to **Logical Enforcement** (module visibility rules).
  - _Risk:_ Strict discipline is required to prevent "spaghetti coupling" now that circular dependencies are physically possible within the same crate.
- **Dependency Injection Refactor**: All cross-crate trait bounds and injection patterns must be rewritten to work within a single crate context.
- **Sync by Default:** We reverse the "Async Everything" trend. Default is SYNC. Async is permitted ONLY for distinct concurrent/network needs (LSP, HTTP).
- **File-First Modules:** `mod.rs` is **BANNED**. We use `entity.rs` (API) + `entity/` (Implementation).

### Workflow Impact

- **Invalidation of Work-in-Progress**: Any feature branches currently in flight will likely be **unmergeable** without a total rewrite.
- **Documentation Obsolescence**: All architectural documentation referencing the 4-crate structure (including `project-context.md`) becomes immediately obsolete.
- **Tooling Updates**: CI pipelines, `mise` tasks, and test runners configured for the old crate structure (`test:unit:domain`, etc.) will fail immediately and require reconfiguration.

---

## 3. Recommended Approach

**Selected Option: Option 3 - Strategic Pivot (Single-Crate Architecture)**

We must pivot to a **Single-Crate "Core" Architecture** (`lithos-core`) + separate Binary Crates (`lithos-cli`, `lithos-lsp`).

### Rationale

1.  **Performance**: Enables within-crate inlining, restoring 10x performance for zero-copy reads.
2.  **Velocity**: Reduces compilation times (no quadratic monomorphization) and simplifies navigation.
3.  **Idiomatic Rust**: Aligns with ecosystem standards (tokio, clap), reducing cognitive load for Rust developers.
4.  **Simplicity**: Replaces complex physical boundaries with standard module visibility (`pub(crate)`).

### Trade-offs

- **Cost**: High immediate effort (1-2 sprints of pure refactoring).
- **Risk**: Temporary instability during the merge.
- **Benefit**: Long-term sustainability, performance, and developer happiness.

---

## 4. Detailed Change Proposals

### 1. Workspace Structure

**Target**: `project-structure-boundaries.md`

| Feature        | Current (Multi-Crate)   | Proposed (Single-Core)                     |
| :------------- | :---------------------- | :----------------------------------------- |
| **Root**       | Workspace with 4 crates | Workspace with `lithos-core`, `lithos-cli` |
| **Boundaries** | Physical (Crates)       | Logical (Modules + Visibility)             |
| **Layers**     | Physical Crates         | Logical Modules                            |
| **CLI**        | `crates/cli`            | `crates/lithos-cli` (remains separate)     |

### 2. Module Organization Strategy

**Target**: `project-structure-boundaries.md`
**Principle**: File First, Folder Second.
**Constraint**: `mod.rs` is strictly banned.

- **API File (`entity.rs`)**: Sits _outside_ the folder. Defines public interface.
- **Impl Folder (`entity/`)**: Contains private logic (`validation.rs`, `util.rs`).
- **Visibility**: Use `pub(crate)` to enforce internal boundaries.
- **Ports**: Co-locate ports with their domains (e.g., `NoteRepository` trait lives in `note.rs`).
- **Benefit**: Eliminates "recursive confusion" and aligns with modern Rust module system.

### 3. Async vs. Sync Model

**Principle**: Synchronous by Default.
**Constraint**: Async requires explicit justification.

- **Default**: Core domain, file system (SSD), data processing = **SYNC**.
- **Exception**: LSP Server, Network I/O = **ASYNC**.
- **Bridge**: Use `tokio::task::spawn_blocking` at edges only.

### 4. ADR Audit & Cleanup

**Action**: Clean up `docs/adr/`. Separate "Decisions" from "Guides".

- **Move to `docs/guides/`**: Testing patterns (0008-0011), Benchmarking (0012), Metrics (0017).
- **Archive**: Rename Detection (0014).
- **Keep**: Core decisions (Redb, Rkyv, Event Bus).

### 5. Error Handling

**Target**: `implementation-patterns-consistency-rules.md`

- **Library (Core)**: Use `thiserror` exclusively. No `anyhow`. No type erasure.
- **Binary (CLI)**: Use `miette` for rich error reporting.
- **Rule**: Library code must never erase types (`anyhow::Result`).

### 6. Naming & Style

**Target**: `implementation-patterns-consistency-rules.md`

- **Remove Suffixes**: `VaultWriterPort` -> `VaultWriter`. `VaultFileDto` -> `VaultFile`.
- **No Hungarian Notation**: Names should reflect capability, not implementation pattern.

---

## 5. Implementation Handoff

**Scope Classification**: **MAJOR (Fundamental Replan)**

### Execution Plan

1.  **Phase 1: Documentation (Architect)**
    - Execute ADR Audit (Move files).
    - Update Architecture Docs with "Sync First" & "File First" rules.
2.  **Phase 2: Demolition (Dev)**
    - Delete root workspace. Create `lithos-core`.
    - Move files to new `file.rs` + `file/` structure.
3.  **Phase 3: Reconstruction (Dev)**
    - De-async core logic.
    - Fix imports.
4.  **Phase 4: Verification (Tea)** - Repair tests. Verify zero-copy benchmarks.

### Required Agents

- **Architect**: To supervise the transition and update docs.
- **Dev**: To execute the file moves and refactoring.
- **Tea**: To repair the test suite.

**Approval Required**: Explicit user approval to proceed with this destructive change.
