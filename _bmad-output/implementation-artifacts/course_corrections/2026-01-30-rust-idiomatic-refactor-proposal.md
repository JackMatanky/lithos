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

**Epic Affected**: ALL (Epics 1-16)
**Priority**: P0 - Critical (Blocks Implementation)
**Target Files**: Root `Cargo.toml`, all epic implementation notes

#### OLD (Current Multi-Crate Architecture)

```
lithos/
├── Cargo.toml (workspace root)
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml (separate compilation unit)
│   │   └── src/
│   │       ├── config/    (7 files - bounded context)
│   │       ├── note/      (8 files - bounded context)
│   │       ├── schema/    (7 files - bounded context)
│   │       ├── template/  (7 files - bounded context)
│   │       └── ports/     (api/ + spi/ separate folders)
│   ├── app/ (separate compilation unit)
│   ├── adapters/ (separate compilation unit)
│   └── cli/ (separate compilation unit)
```

**Architectural Pattern**: Hexagonal (Ports & Adapters) with physical crate boundaries

#### NEW (Single-Core Architecture)

```
lithos/
├── Cargo.toml (workspace root)
├── crates/
│   ├── lithos-core/
│   │   ├── Cargo.toml (single compilation unit)
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── config.rs       # Public API for Config context
│   │       ├── config/         # Private implementation details
│   │       │   ├── vault.rs
│   │       │   ├── global.rs
│   │       │   ├── merge.rs
│   │       │   ├── ports.rs    # ConfigRepository trait (co-located)
│   │       │   ├── error.rs    # ConfigError enum (co-located)
│   │       │   └── events.rs   # ConfigUpdated event (co-located)
│   │       │
│   │       ├── note.rs         # Public API for Note context
│   │       ├── note/           # Private implementation details
│   │       │   ├── aggregate.rs
│   │       │   ├── frontmatter.rs
│   │       │   ├── links.rs
│   │       │   ├── tasks.rs
│   │       │   ├── ports.rs    # NoteRepository trait (co-located)
│   │       │   ├── error.rs    # NoteError enum (co-located)
│   │       │   └── events.rs   # NoteCreated, NoteUpdated (co-located)
│   │       │
│   │       ├── schema.rs       # Public API for Schema context
│   │       ├── schema/         # Private implementation details
│   │       │   ├── property.rs
│   │       │   ├── property_bank.rs
│   │       │   ├── resolver.rs
│   │       │   ├── ports.rs    # SchemaRepository trait (co-located)
│   │       │   ├── error.rs    # SchemaError enum (co-located)
│   │       │   └── events.rs   # SchemaCreated (co-located)
│   │       │
│   │       ├── template.rs     # Public API for Template context
│   │       ├── template/       # Private implementation details
│   │       │   ├── composition.rs
│   │       │   ├── variables.rs
│   │       │   ├── ports.rs    # TemplateRepository trait (co-located)
│   │       │   ├── error.rs    # TemplateError enum (co-located)
│   │       │   └── events.rs   # TemplateCreated (co-located)
│   │       │
│   │       ├── fs/             # File system utilities (parsers, validation, vault)
│   │       ├── cache/          # Generic cache infrastructure (redb, moka, coordinator)
│   │       └── <context>/      # Context IO implementations live with contexts (see Proposal 3)
│   │
│   ├── lithos-cli/
│   │   ├── Cargo.toml (separate binary)
│   │   └── src/
│   │
│   └── lithos-lsp/            # Phase 2
│       ├── Cargo.toml (separate binary)
│       └── src/
```

**Architectural Pattern**: Dependency Flow Architecture (dependencies flow inward toward domain core)

```
Dependency Flow:
fs/ and cache/ are generic infrastructure with no domain knowledge
<context>/storage.rs depends on domain types and uses fs/ + cache/
cli depends on context storage and/or domain traits

Domain contexts (note/, schema/, etc.) depend on NOTHING internal
```

**Rationale**:

1. Performance: Enables within-crate inlining -> 5-10x faster zero-copy reads (Critical for ADR 0002 sub-50ms LSP target)
2. Compilation: Eliminates quadratic monomorphization across crates -> 1.5-2x faster builds
3. File Organization: Module pattern: entity.rs (public API) + entity/ folder (private implementation)
4. Rust Idioms: Aligns with ecosystem patterns (tokio, clap, rust-analyzer use single-crate cores)
5. Dependency Control: Enforced via pub(crate) visibility + module structure (compiler-enforced, dependencies flow inward)
6. Port Co-location: Ports live in context/ports.rs within each bounded context (easier discovery, better cohesion per proposal section 2)

#### Research-Backed Design Decision: Co-Located Errors and Events

**Critical Question**: Should errors.rs and events.rs be centralized or co-located with contexts?
**Answer**: CO-LOCATE

**Evidence and Sources**:

1. Rust API Guidelines (Official Standard)
   - Source: https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err
   - "Error types are meaningful and well-behaved (C-GOOD-ERR)"
   - Key principle: Each module should define context-specific error types that are meaningful for its domain.

2. Nick Groenen - "Rust Error Handling in 2020"
   - Source: https://nick.groenen.me/posts/rust-error-handling/
   - Libraries should produce meaningful, structured error types; applications consume errors.

3. Tokio (Real-World Pattern)
   - Evidence: tokio/src/io/mod.rs (re-exports std::io::Error)
   - Pattern: Tokio co-locates errors with functionality, not centralized.

4. Clap (Real-World Pattern)
   - Evidence: clap_builder/src/error/mod.rs
   - Centralized errors are appropriate because Clap is an error-presentation library (not a domain library).

**Counter-Argument Addressed**: "We need a top-level error type"

- Solution: Compose with `thiserror` and `#[from]`:

```
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LithosError {
    #[error(transparent)]
    Note(#[from] note::NoteError),

    #[error(transparent)]
    Schema(#[from] schema::SchemaError),

    #[error(transparent)]
    Cache(#[from] cache::CacheError),
}
```

**Events**: Co-locate by default; centralize ONLY if event bus architecture requires it.

#### Epic-Level Impact

- Epic 3 (Domain):
  - Move from `crates/domain/src/config/` -> `lithos-core/src/config.rs` + `config/` folder
  - Add `config/error.rs`, `config/events.rs` (co-located, not centralized)
  - Remove centralized `domain/src/errors.rs`, `domain/src/events.rs`

- Epic 5 (Cache):
  - Move from `crates/adapters/src/spi/cache/` -> `lithos-core/src/cache/`
  - Add `cache/error.rs` (co-located CacheError enum)

- Epic 9 (Storage):
  - Integrate into `lithos-core/src/cache/storage.rs` (extends cache module)
  - Reuse `cache::CacheError` (no separate StorageError)

- ALL Epics:
  - Update imports from `use domain::config::...` to `use lithos_core::config::...`
  - Replace "Hexagonal Architecture" references with "Dependency Flow Architecture (dependencies point inward)"
  - Replace centralized error references with co-located per-context errors

#### Migration Path for Errors

```text
OLD (Centralized - incorrect)
crates/domain/src/errors.rs

NEW (Co-located - correct)
lithos-core/src/note/error.rs
lithos-core/src/schema/error.rs
lithos-core/src/cache/error.rs

OPTIONAL (Top-level composition)
lithos-core/src/lib.rs (LithosError via #[from])
```

**Note**: Later proposals refine portions of this structure (e.g., IO/CQRS placement and commands folder usage). See Proposal 3 for the finalized IO/CQRS structure.

| Feature        | Current (Multi-Crate)   | Proposed (Single-Core)                     |
| :------------- | :---------------------- | :----------------------------------------- |
| **Root**       | Workspace with 4 crates | Workspace with `lithos-core`, `lithos-cli` |
| **Boundaries** | Physical (Crates)       | Logical (Modules + Visibility)             |
| **Layers**     | Physical Crates         | Logical Modules                            |
| **CLI**        | `crates/cli`            | `crates/lithos-cli` (remains separate)     |

### 2. Error Type Consolidation (Co-Located Errors)

**Epic Affected**: Epic 3 (Domain), Epic 5 (Cache), Epic 9 (Storage), Epic 14 (CLI)
**Priority**: P1 - High (Error clarity and boundary enforcement)
**Target Files**: `implementation-patterns-consistency-rules.md`, domain error definitions

**Problem**: A single `DomainError` obscures context-specific failures and leaks across boundaries, making errors harder to match and violating Rust error idioms.

**Change**:

- Replace `DomainError` with context-specific error enums
- Co-locate error types with their context (`note/error.rs`, `schema/error.rs`, `config/error.rs`, `cache/error.rs`)
- Use `thiserror` with `#[from]` conversions
- Optional top-level error type only for orchestration

**OLD**

```
domain/src/errors.rs
```

```rust
pub enum DomainError {
    Validation(String),
    Storage(String),
    Parse(String),
}
```

**NEW**

```
note/error.rs
schema/error.rs
config/error.rs
cache/error.rs
```

```rust
#[derive(thiserror::Error, Debug)]
pub enum NoteError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error(transparent)]
    Parse(#[from] NoteParseError),
}
```

**Optional orchestration error**:

```rust
#[derive(thiserror::Error, Debug)]
pub enum LithosError {
    #[error(transparent)]
    Note(#[from] note::NoteError),

    #[error(transparent)]
    Schema(#[from] schema::SchemaError),
}
```

**Rationale**:

- Aligns with Rust API Guidelines (C-GOOD-ERR)
- Improves error clarity and matching at call sites
- Enforces context boundaries by type

**Note**: This proposal formalizes the co-located error decision described in Proposal 1 and removes `DomainError` entirely.

### 3. Module Organization Strategy

**Target**: `project-structure-boundaries.md`
**Principle**: File First, Folder Second.
**Constraint**: `mod.rs` is strictly banned.

**Summary**: The current modularization is correct in scope (files are already 200-900 lines), but the entry-point files (`mod.rs`) should be replaced with `<context>.rs` files that merge the aggregate root and public re-exports. The aggregate root becomes the primary entry point for each context.

#### Proposed Directory Structure

##### OLD (Current - Multi-Crate with mod.rs pattern)

```
crates/
├── domain/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── errors.rs                 # Centralized errors (will be split)
│       ├── patterns.rs
│       ├── validation.rs
│       │
│       ├── config/
│       │   ├── mod.rs                # DELETE (pre-2018 pattern)
│       │   ├── aggregate.rs          # DELETE (merge into config.rs)
│       │   ├── types.rs              # KEEP
│       │   ├── vault.rs              # KEEP
│       │   ├── global.rs             # KEEP
│       │   └── events.rs             # KEEP (already co-located)
│       │
│       ├── note/
│       │   ├── mod.rs                # DELETE
│       │   ├── aggregate.rs          # DELETE (merge into note.rs)
│       │   ├── frontmatter.rs        # KEEP
│       │   ├── link.rs               # KEEP
│       │   ├── tag.rs                # KEEP
│       │   ├── structure.rs          # KEEP
│       │   ├── task.rs               # KEEP
│       │   └── events.rs             # KEEP (already co-located)
│       │
│       ├── schema/
│       │   ├── mod.rs                # DELETE
│       │   ├── aggregate.rs          # DELETE (merge into schema.rs)
│       │   ├── property.rs           # KEEP
│       │   ├── property_spec.rs      # KEEP
│       │   ├── graph.rs              # KEEP
│       │   ├── resolver.rs           # KEEP
│       │   ├── raw.rs                # KEEP
│       │   └── events.rs             # KEEP (already co-located)
│       │
│       ├── template/
│       │   ├── mod.rs                # DELETE
│       │   ├── aggregate.rs          # DELETE (merge into template.rs)
│       │   ├── variable.rs           # KEEP
│       │   ├── composition.rs        # KEEP
│       │   ├── syntax.rs             # KEEP
│       │   ├── validation.rs         # KEEP
│       │   └── events.rs             # KEEP (already co-located)
│       │
│       └── ports/
│           ├── mod.rs                # DELETE (centralized ports)
│           ├── config.rs             # MOVE to config/ports.rs
│           ├── note.rs               # MOVE to note/ports.rs
│           ├── schema.rs             # MOVE to schema/ports.rs
│           ├── template.rs           # MOVE to template/ports.rs
│           └── spi/
│               └── mod.rs            # DELETE (empty)
│
├── app/                              # DELETE (merge into lithos-core)
├── adapters/                         # MOVE to lithos-core/cache/
└── cli/                              # RENAME to lithos-cli/
```

##### NEW (Single-Crate with Rust 2018+ pattern)

```
crates/
└── lithos-core/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── patterns.rs               # KEEP (shared patterns)
        ├── validation.rs             # KEEP (shared validation utilities)
        │
        ├── config.rs                 # NEW (mod.rs + aggregate.rs merged)
        ├── config/
        │   ├── types.rs              # KEEP
        │   ├── vault.rs              # KEEP
        │   ├── global.rs             # KEEP
        │   ├── events.rs             # KEEP (co-located)
        │   ├── ports.rs              # NEW (moved from ports/config.rs)
        │   └── error.rs              # NEW (extracted from errors.rs)
        │
        ├── note.rs                   # NEW (mod.rs + aggregate.rs merged)
        ├── note/
        │   ├── frontmatter.rs        # KEEP
        │   ├── link.rs               # KEEP
        │   ├── tag.rs                # KEEP
        │   ├── structure.rs          # KEEP
        │   ├── task.rs               # KEEP
        │   ├── events.rs             # KEEP (co-located)
        │   ├── ports.rs              # NEW (moved from ports/note.rs)
        │   └── error.rs              # NEW (extracted from errors.rs)
        │
        ├── schema.rs                 # NEW (mod.rs + aggregate.rs merged)
        ├── schema/
        │   ├── property.rs           # KEEP
        │   ├── property_spec.rs      # KEEP
        │   ├── graph.rs              # KEEP
        │   ├── resolver.rs           # KEEP
        │   ├── raw.rs                # KEEP
        │   ├── events.rs             # KEEP (co-located)
        │   ├── ports.rs              # NEW (moved from ports/schema.rs)
        │   └── error.rs              # NEW (extracted from errors.rs)
        │
        ├── template.rs               # NEW (mod.rs + aggregate.rs merged)
        ├── template/
        │   ├── variable.rs           # KEEP
        │   ├── composition.rs        # KEEP
        │   ├── syntax.rs             # KEEP
        │   ├── validation.rs         # KEEP
        │   ├── events.rs             # KEEP (co-located)
        │   ├── ports.rs              # NEW (moved from ports/template.rs)
        │   └── error.rs              # NEW (extracted from errors.rs)
        │
        └── cache/                    # MOVED (from adapters/src/spi/cache/)
            ├── mod.rs
            ├── moka.rs
            ├── redb.rs
            ├── coordinator.rs
            ├── error.rs              # CacheError (co-located)
            └── storage.rs
```

#### File Structure Rules

**Rule 1: Context Module Pattern (Rust 2018+)**

Each bounded context follows this structure:

```
<context>.rs                  # Aggregate root + module declarations + re-exports
<context>/
    <submodule>.rs            # Supporting types and logic
    ports.rs                  # Context-specific repository traits (co-located)
    error.rs                  # Context-specific error enum (co-located)
    events.rs                 # Context-specific domain events (co-located)
```

**What goes in `<context>.rs`:**

1. Module-level documentation
2. Submodule declarations (`mod types;`, `mod vault;`, etc.)
3. Public API re-exports (`pub use`)
4. Aggregate root implementation

**What goes in `<context>/` folder:**

- Supporting files (types, value objects, focused logic)
- Co-located concerns (ports, errors, events)

**Rule 2: Co-Located Concerns**

- Ports in `<context>/ports.rs`
- Errors in `<context>/error.rs`
- Events in `<context>/events.rs`

**Rule 3: Dependency Flow**

```
Dependencies flow INWARD (toward domain contexts):

fs/ and cache/ are generic infrastructure with no domain knowledge
<context>/storage.rs depends on domain types and uses fs/ + cache/
lithos-cli depends on context storage and/or domain traits

Domain contexts (note/, schema/, config/, template/) depend on NOTHING internal
```

#### Migration Operations

| Operation                               | Files Affected     | Action                                            |
| --------------------------------------- | ------------------ | ------------------------------------------------- |
| Merge mod.rs + aggregate.rs             | 4 contexts         | Create config.rs, note.rs, schema.rs, template.rs |
| Co-locate ports                         | 4 files            | Move ports/config.rs -> config/ports.rs, etc.     |
| Co-locate errors                        | 4 new files        | Extract from errors.rs -> config/error.rs, etc.   |
| Keep events as-is                       | 4 files            | Already co-located                                |
| Physical relocation                     | ALL files          | Move to lithos-core/src/                          |
| Update imports                          | ALL consuming code | use domain::config:: -> use lithos_core::config:: |
| Remove centralized ports/ and errors.rs | 2 folders          | Delete after migration                            |

#### File Count Summary

| Context  | Before (Current)                        | After (Proposed)                      | Net Change |
| -------- | --------------------------------------- | ------------------------------------- | ---------- |
| Config   | 6 files (mod.rs, aggregate.rs, 4 files) | 7 files (config.rs, 6 files)          | +1         |
| Note     | 8 files (mod.rs, aggregate.rs, 6 files) | 9 files (note.rs, 8 files)            | +1         |
| Schema   | 7 files (mod.rs, aggregate.rs, 5 files) | 8 files (schema.rs, 7 files)          | +1         |
| Template | 6 files (mod.rs, aggregate.rs, 4 files) | 7 files (template.rs, 6 files)        | +1         |
| Shared   | 2 files (errors.rs, ports/)             | 0 files (errors/ports now co-located) | -2         |
| Total    | 29 files                                | 31 files                              | +2         |

**Net +2 files**: Eliminated 2 centralized files (errors.rs, ports/) but added 4 co-located ports.rs + 4 error.rs - 6 deleted mod.rs/aggregate.rs pairs.

#### Navigation Improvement

**Before (Current):**

```
Q: Where is the Config aggregate?
A: crates/domain/src/config/mod.rs (re-exports)
   -> config/aggregate.rs (actual implementation)

Q: Where is ConfigRepository trait?
A: crates/domain/src/ports/config.rs
```

**After (Proposed):**

```
Q: Where is the Config aggregate?
A: lithos-core/src/config.rs (direct access - main entry point)

Q: Where is ConfigRepository trait?
A: lithos-core/src/config/ports.rs (co-located with Config context)
```

**Note**: Later proposals refine portions of this structure (e.g., IO/CQRS placement and commands folder usage). See Proposal 3 for the finalized IO/CQRS structure.

### 4. IO and CQRS Integration in a Flat Architecture

**Target**: `project-structure-boundaries.md`, `implementation-patterns-consistency-rules.md`, Epics 4, 5, 9, 10

**Problem**: Removing the adapters layer makes IO placement and CQRS boundaries ambiguous. We need a Rust-idiomatic pattern that:

- Maintains dependency flow inward
- Avoids duplicating persistence logic per context
- Keeps IO implementations discoverable
- Avoids reintroducing an adapter layer under a new name

**Idiomatic Rust Evidence (co-located impls with type)**:

- `std::fs::File` defines the type and implements `Read` in the same module (`std/src/fs.rs`).
  - Source: https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/fs.rs
- `serde_json::Value` defines the type in `value/mod.rs` and implements `Serialize`/`Deserialize` in `value/ser.rs` and `value/de.rs` (same module folder).
  - Source: https://github.com/serde-rs/json/tree/master/src/value

**Evaluated Options**:

**Option A: Fully co-located IO per context (no shared infra)**

- Each context (`note/`, `schema/`, `config/`) contains full persistence logic.
- **Pros**: Maximum locality, easy discovery
- **Cons**: Duplicated persistence logic, inconsistent error handling, hard to share transactions/metrics

**Option B: Co-located thin adapters + shared cache infrastructure (RECOMMENDED)**

- Contexts define CQRS traits and implement them in `context/storage.rs`, delegating to generic cache primitives.
- Cache provides generic persistence building blocks (redb/moka/coordinator), with no domain knowledge.
- **Pros**: Idiomatic co-location + shared infra, minimal duplication, consistent IO behavior
- **Cons**: Thin glue files per context

**Option C: Centralized implementations in `cache/` (adapter-like)**

- Cache implements all domain traits (e.g., `cache/redb/note.rs`).
- **Pros**: Consolidated by backend
- **Cons**: Recreates adapter layer, harms discoverability, cache folder balloons

**Option D: Replace traits with functions (`ops.rs`)**

- Use concrete functions instead of CQRS traits.
- **Pros**: Minimal abstraction
- **Cons**: Harder testing/mocking, weaker CQRS boundaries, less flexible for multiple backends

**Option E: Generic Repository Trait (`Repository<T>`)**

- Define a unified `trait Repository<T> { fn get(&self, id: K) -> Option<T>; fn save(&self, entity: T); }`.
- **Pros**: Uniform API, "Don't Repeat Yourself" (DRY) for basic CRUD.
- **Cons**:
  - **The Index Problem**: `save(Note)` isn't just a Key-Value put; it requires updating secondary indexes (backlinks, tags). A generic `Repository<T>` hides this transactional complexity or requires complex `Indexable` traits.
  - **Query Anemia**: Generic repos provide `get(id)` but not `find_by_path(path)`. We would immediately need `trait NoteRepository: Repository<Note> { ... }`, creating "Trait Soup" (inheritance hierarchy).
  - **Type Erasure**: Obscures the specific capabilities of a context (e.g., `Config` is read-heavy/cached, `Note` is write-heavy/indexed).

**Decision**: **Option B** (Co-located Specific Traits + Generic Adapter Implementation)

**Key Distinction**:
- **The Port (Interface)** is *Specific* (`NoteQueries`) to clearly define Domain capabilities.
- **The Adapter (Implementation)** is *Generic* (`RedbCache<K,V>`) to reuse infrastructure code.

**Key Rules**:

1. **CQRS traits live with the context**
   - `note/commands.rs`, `note/queries.rs` (or `note/ports.rs` for small contexts)
2. **Context IO implementations live with the context**
   - `note/storage.rs` implements `NoteCommands` and `NoteQueries`
3. **Cache is generic infrastructure only**
   - `cache/` provides generic `RedbCache<K,V>`, `MokaCache<K,V>` with `put/get/delete/scan`
   - Cache has **no domain types** and **no domain trait impls**
4. **IO shared utilities live in `fs/`**
   - Config parsing + path validation in `fs/`
   - Vault directory utilities in `fs/vault.rs` and `fs/walker.rs`
5. **Vault indexing is cache orchestration**
   - `cache/indexer.rs` uses `fs/` + domain logic + cache primitives to populate storage

**Proposed Structure**:

```
crates/lithos-core/src/
├── note/
│   ├── aggregate.rs
│   ├── frontmatter.rs
│   ├── link.rs
│   ├── commands.rs        # CQRS traits (write)
│   ├── queries.rs         # CQRS traits (read)
│   ├── storage.rs         # IO impls (thin, delegate to cache)
│   ├── error.rs
│   └── events.rs
│
├── schema/
│   ├── aggregate.rs
│   ├── property.rs
│   ├── commands.rs
│   ├── queries.rs
│   ├── storage.rs
│   └── error.rs
│
├── config/
│   ├── aggregate.rs
│   ├── ports.rs           # CQRS traits (combined, small context)
│   ├── storage.rs
│   └── error.rs
│
├── fs/
│   ├── mod.rs
│   ├── parsers/
│   │   ├── mod.rs
│   │   ├── config.rs      # TOML/JSON/YAML parsing (Epic 4)
│   │   └── markdown.rs    # Markdown parsing (Epic 10, when needed)
│   ├── validator.rs       # Path validation (Epic 4)
│   ├── vault.rs           # Vault directory utilities
│   └── walker.rs          # Vault traversal
│
└── cache/
    ├── mod.rs
    ├── reader.rs          # CacheReader trait (generic)
    ├── writer.rs          # CacheWriter trait (generic)
    ├── redb.rs            # RedbCache<K,V> generic persistence
    ├── moka.rs            # MokaCache<K,V> generic cache
    ├── coordinator.rs     # Coordinator<K,V> generic orchestration
    ├── codec.rs           # rkyv/serde helpers
    └── indexer.rs         # Vault indexing (cache population)
```

**Tradeoffs (Explicit)**:

- **Pros**: Co-location preserved, IO logic shared, no cache bloat, strong CQRS boundary
- **Cons**: Thin storage glue files per context, requires discipline to keep cache generic

**Dependency Flow Enforcement (Required)**:

- Domain modules (`note/aggregate.rs`, `schema/aggregate.rs`, etc.) must not import `cache/` or `fs/`
- Context IO lives in `context/storage.rs` and is the only place where domain types touch infrastructure
- Cache must remain generic (no domain types, no domain trait impls)
- Add architecture tests to fail if domain modules reference `crate::cache` or `crate::fs`

---

### 5. Async vs. Sync Model: Sync‑First Execution Model (Async Only at Edges)

**Principle**: Synchronous by Default.
**Constraint**: Async requires explicit justification.

- **Default**: Core domain, file system (SSD), data processing = **SYNC**.
- **Exception**: LSP Server, Network I/O = **ASYNC**.
- **Bridge**: Use `tokio::task::spawn_blocking` at edges only.

**Epic Affected**: Epic 5 (Cache), Epic 8 (Event Bus), Epic 10 (Indexing), Epic 14 (CLI), Epic 15 (Tests)
**Priority**: P1 – High (Performance + complexity control)
**Target Files**: `implementation-patterns-consistency-rules.md`, architecture docs, and affected epic stories

**Problem**
The current plan assumes async everywhere, which increases complexity and couples core logic to an async runtime. In Rust, async is most valuable at the edges (I/O multiplexing, LSP, network), not inside CPU‑bound or local filesystem logic.

**Proposed Rule (New Standard)**
Core logic is synchronous by default.
Async is allowed **only** for:

- LSP server operations
- Network I/O
- Explicit concurrency requirements (e.g., parallel indexing)

If a core operation must perform blocking work in an async context, it should use `tokio::task::spawn_blocking` at the boundary.

**OLD (Current Pattern)**

- async_trait used broadly across domain and cache traits
- Command/query traits are async by default
- Cache operations modeled async even though redb/moka are synchronous
  **NEW (Proposed Pattern)**
- Sync traits for core domain + cache interfaces
- Async only in CLI/LSP orchestration layers
- spawn_blocking used only at async boundaries

**Example Transformation**

_OLD (Async Everywhere)_

```rust
#[async_trait]
pub trait NoteQueries: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;
}
```

_NEW (Sync Core, Async at Edge)_

```rust
pub trait NoteQueries: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;
}
// async edge (CLI/LSP)
let note = tokio::task::spawn_blocking(move || repo.find_by_id(id)).await??;
```

**Epic‑Level Impact**

- Epic 5 (Cache): Use synchronous CacheReader/CacheWriter traits
- Epic 8 (Event Bus): Defer async event bus; start with direct calls
- Epic 10 (Indexing): Indexing can be sync; parallelism is optional and explicit
- Epic 14 (CLI): Async only if CLI needs concurrent operations
- Epic 15 (Tests): Simplify tests (no runtime required for core)

**Rationale**

- Reduces runtime coupling
- Simplifies error handling and trait signatures
- Better performance for local IO + CPU-bound flows
- Matches your "sync-first" architecture intent

### 6. ADR Audit & Cleanup

**Epic Affected**: Epic 16 (Documentation), all epics referencing ADRs
**Priority**: P2 – Medium
**Target Files**: `docs/adr/`, `docs/guides/`, all ADR references in docs/epics/architecture

**Additions Required**

1. Renumber ADRs to remain consecutive after moves/archival
2. Update all references to any ADR that moves or changes number
   - Architecture docs
   - Epics/stories
   - PRD and any references in `_bmad-output/`

**Actions**

- Move non‑decisions to `docs/guides/`
- Archive outdated ADRs
- Renumber remaining ADRs sequentially
- Update all references (file paths + ADR numbers)

**Rationale**

- Prevents broken references after ADR moves
- Keeps ADR list clean and authoritative
- Avoids confusion in planning artifacts

### 7. Error Handling Standardization

**Epic Affected**: Epic 3 (Domain), Epic 5 (Cache), Epic 14 (CLI), Epic 15 (Tests)
**Priority**: P1 – High
**Target Files**: `implementation-patterns-consistency-rules.md`, error usage across crates

**Problem**

Error handling is inconsistent (mix of anyhow, color-eyre, and planned miette). In Rust, library crates should expose structured errors while binaries format/report errors.

**Proposed Rule**

- Library/Core (lithos-core): `thiserror` only
- CLI (lithos-cli): `miette` for reporting
- No `anyhow` or `color-eyre` in library crates to avoid type erasure in library APIs
- `anyhow` allowed only in main.rs if needed for quick prototyping

**Rationale**

- Preserves typed error handling
- Avoids type erasure in library APIs
- Aligns with Rust ecosystem best practices

### 8. Naming & Style Cleanup

**Epic Affected**: Epic 3 (Domain), Epic 5 (Cache), Epic 9 (Storage), Epic 14 (CLI)
**Priority**: P2 – Medium
**Target Files**: `implementation-patterns-consistency-rules.md`, domain models, ports, adapters

**Problem**

Current naming uses suffixes like Port, Dto, WriterPort, which is a Hungarian‑style pattern and non‑idiomatic in Rust. It adds noise and leaks implementation detail.

**Proposed Rule**

- Remove Port/DTO suffixes
- Use semantic names
  - `VaultWriterPort` → `VaultWriter`
  - `VaultFileDto` → `VaultFile`
- If a type must be disambiguated, use module scoping instead:
  - `dto::VaultFile` (not `VaultFileDto`)

**Rationale**

- Aligns with Rust naming conventions
- Improves readability and API clarity
- Reduces cognitive load

### 9. Domain Serialization Strategy (Idiomatic Rust Default)

**Epic Affected**: Epic 3 (Domain), Epic 5 (Cache), Epic 7 (Schema), Epic 9 (Storage)
**Priority**: P0 - Critical (API design + dependency choice)
**Target Files**: Architecture docs, ADR 0013

**Evidence**:

- Rust API Guidelines recommend that data structures implement Serde traits.
  - Source: https://rust-lang.github.io/api-guidelines/interoperability.html#c-serde
- Serde explicitly supports feature-gated derives to avoid forcing dependencies.
  - Source: https://serde.rs/feature-flags.html

**Decision**: Prefer idiomatic Rust. Allow Serde on domain data structures behind a `serde` feature flag.

**Rule**:

- Domain types MAY derive `Serialize`/`Deserialize` when they represent data structures.
- Derives are **feature-gated** (`cfg(feature = "serde")`) to keep dependency optional.
- Non-data-structure types (behaviors, services) should not implement Serde.

**Pattern**:

```toml
[dependencies]
serde = { version = "1.0", optional = true, features = ["derive"] }
```

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note {
    // fields
}
```

**Rationale**:

- Aligns with Rust API Guidelines (C-SERDE)
- Keeps serde optional for consumers
- Avoids premature DTO mapping layers

**Note**: This intentionally prioritizes idiomatic Rust over the prior "zero external deps in domain" policy. If strict purity is still desired, it must be justified as a project-specific deviation from Rust idioms.

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
