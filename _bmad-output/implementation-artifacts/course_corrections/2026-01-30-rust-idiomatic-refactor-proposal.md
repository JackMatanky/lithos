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
│   │       ├── db.rs           # Zero-copy database layer (see Proposals 4 & 5)
│   │       └── <context>/      # Domain contexts with static methods
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
fs/ is generic infrastructure with no domain knowledge
db.rs provides zero-copy primitives with no domain knowledge
<context>/ contains domain types with static methods that use Database
lithos-cli orchestrates db + contexts

Domain contexts (note/, schema/, etc.) depend on NOTHING internal except db.rs
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
  - Replace `crates/adapters/src/spi/cache/` with `lithos-core/src/db.rs`
  - NO cache/ folder (see Proposals 4 & 5)
  - Database layer provides zero-copy primitives

- Epic 9 (Storage):
  - Integrate into domain contexts as static methods
  - Use `Database` type from `db.rs` (no separate storage module)

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
lithos-core/src/db/error.rs

OPTIONAL (Top-level composition)
lithos-core/src/lib.rs (LithosError via #[from])
```

**Note**: Later proposals refine portions of this structure. See Proposals 4 & 5 for database layer and CQRS pattern decisions.

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
- Co-locate error types with their context (`note/error.rs`, `schema/error.rs`, `config/error.rs`, `db/error.rs`)
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
db/error.rs
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
**Epic Affected**: Epic 3 (Domain), all epics with domain contexts
**Priority**: P1 - High (Foundation for file organization)
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
├── adapters/                         # DELETE (replaced by db.rs)
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
├── db.rs                     # NEW (zero-copy database layer, see Proposals 4 & 5)
└── fs/                       # NEW (file system utilities)
    ├── parsers/
    │   ├── config.rs         # TOML/JSON/YAML parsing
    │   └── markdown.rs       # Markdown parsing (deferred)
    ├── validator.rs
    ├── vault.rs
    └── walker.rs
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

fs/ and db.rs are generic infrastructure with no domain knowledge
<context>/ contains domain types with static methods that use Database
lithos-cli depends on context types and Database

Domain contexts (note/, schema/, config/, template/) depend on NOTHING internal except db.rs
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

**Note**: Later proposals refine portions of this structure. See Proposals 4 & 5 for database layer and CQRS pattern decisions.

### 4. Database Layer: Zero-Copy Infrastructure (CLI MVP)

**Target**: `project-structure-boundaries.md`, `implementation-patterns-consistency-rules.md`, Epics 4, 5, 9, 10
**Epic Affected**: Epic 5 (Cache), Epic 9 (Storage), Epic 10 (Indexing)
**Priority**: P0 - Critical (Foundation for all IO operations)

**Problem**: The current multi-crate architecture prevents zero-copy optimization and creates unnecessary abstraction layers. We need a Rust-idiomatic database layer that:

- Exposes ALL redb/rkyv zero-copy primitives (AccessGuard lifetimes, insert_reserve, MultimapTable)
- Avoids duplicating serialization logic across contexts
- Uses concrete types (not traits) matching `std::fs::File` pattern
- Defers LSP-specific features (Moka, Coordinator) to Phase 2

**Idiomatic Rust Evidence**:

**Pattern 1: Concrete Types Over Traits (stdlib pattern)**
- `std::fs::File` provides concrete methods (`open`, `read`, `write`), NOT a `FileSystem` trait
  - Source: https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/fs.rs
- `std::collections::HashMap` provides concrete methods, NOT a `Map<K,V>` trait
- **Lesson**: Generic methods on concrete types are MORE idiomatic than trait hierarchies

**Pattern 2: Generic Methods (not macros)**
- Rust API Guidelines C-GENERIC: "Use generics to enable callers to reuse code"
- stdlib uses generic functions (`fs::read<P: AsRef<Path>>`) NOT macros
- **Lesson**: Generics provide better IDE support and error messages than macros

**Decision**: **Concrete `Database` type with generic zero-copy methods**

**Architecture**:

```
lithos-core/src/
├── db.rs                       # Zero-copy database layer (NO traits)
│   ├── pub struct Database     # Concrete type (not trait)
│   ├── pub struct ArchivedGuard<'txn, V>  # Zero-copy Deref wrapper
│   ├── get_archived<K,V>()     # Hot path: returns ArchivedGuard
│   ├── get<K,V>()              # Cold path: full deserialization
│   ├── put_reserve<K,V,F>()    # Zero-copy write (insert_reserve)
│   ├── put<K,V>()              # Convenience wrapper
│   ├── multimap_insert<K,V>()  # 1:N indexes (tags, backlinks)
│   ├── multimap_get<K,V>()     # Returns iterator of ArchivedGuard
│   └── batch_write<F>()        # Durability::None for bulk ops
│
├── fs/                         # File system utilities (NO db dependency)
│   ├── parsers/
│   │   ├── config.rs           # TOML/JSON/YAML parsing
│   │   └── markdown.rs         # Markdown parsing (deferred to Epic 10)
│   ├── validator.rs            # Path validation
│   ├── vault.rs                # Vault directory utilities
│   └── walker.rs               # Vault traversal
│
└── note/
    ├── aggregate.rs            # Pure domain (NO db import)
    ├── indexing.rs             # Tag/backlink index logic
    └── ...
```

**Key Implementation: Zero-Copy Primitives**

```rust
// db.rs

pub struct Database {
    inner: redb::Database,
}

/// Zero-copy guard that provides Deref access to archived data
pub struct ArchivedGuard<'txn, V> {
    guard: redb::AccessGuard<'txn, &'static [u8]>,
    _phantom: PhantomData<V>,
}

impl<'txn, V> Deref for ArchivedGuard<'txn, V>
where
    V: rkyv::Archive,
    rkyv::Archived<V>: for<'a> rkyv::bytecheck::CheckBytes<...>,
{
    type Target = rkyv::Archived<V>;

    fn deref(&self) -> &Self::Target {
        // Validation happens once in get_archived()
        unsafe { rkyv::access_unchecked(self.guard.value()) }
    }
}

impl Database {
    /// Zero-copy read (HOT PATH for LSP)
    /// Returns guard with lifetime tied to transaction
    pub fn get_archived<K, V>(
        &self,
        table: &str,
        key: &K,
    ) -> Result<ArchivedGuard<'_, V>, DbError>
    where
        K: rkyv::Serialize,
        V: rkyv::Archive,
        rkyv::Archived<V>: for<'a> rkyv::bytecheck::CheckBytes<...>,
    {
        let txn = self.inner.begin_read()?;
        let table = txn.open_table(TableDefinition::new(table))?;
        let key_bytes = rkyv::to_bytes(key)?;
        let guard = table.get(key_bytes.as_ref())?.ok_or(DbError::NotFound)?;

        // Validate alignment ONCE
        let alignment = std::mem::align_of::<rkyv::Archived<V>>();
        if guard.value().as_ptr().align_offset(alignment) != 0 {
            return Err(DbError::Misaligned);
        }

        // Validate bytes ONCE (per ADR 0002 requirement)
        rkyv::check_archived_root::<V>(guard.value())?;

        Ok(ArchivedGuard { guard, _phantom: PhantomData })
    }

    /// Full deserialization (COLD PATH for mutations)
    pub fn get<K, V>(&self, table: &str, key: &K) -> Result<Option<V>, DbError>
    where
        K: rkyv::Serialize,
        V: rkyv::Archive + rkyv::Deserialize,
    {
        match self.get_archived::<K, V>(table, key) {
            Ok(archived) => Ok(Some(archived.deserialize()?)),
            Err(DbError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Zero-copy write using insert_reserve (per ADR 0002)
    pub fn put_reserve<K, V, F>(
        &self,
        table: &str,
        key: &K,
        value_size: usize,
        write_fn: F,
    ) -> Result<(), DbError>
    where
        K: rkyv::Serialize,
        F: FnOnce(&mut [u8]) -> Result<(), DbError>,
    {
        let mut txn = self.inner.begin_write()?;
        let mut table = txn.open_table(TableDefinition::new(table))?;
        let key_bytes = rkyv::to_bytes(key)?;

        // Zero-copy: get mutable slice directly to DB page
        let mut reserved = table.insert_reserve(key_bytes.as_ref(), value_size)?;
        write_fn(reserved.as_mut())?;

        txn.commit()?;
        Ok(())
    }

    /// Convenience wrapper (allocates temp buffer)
    pub fn put<K, V>(&self, table: &str, key: &K, value: &V) -> Result<(), DbError>
    where
        K: rkyv::Serialize,
        V: rkyv::Serialize,
    {
        let value_bytes = rkyv::to_bytes::<_, 256>(value)?;
        let value_size = value_bytes.len();

        self.put_reserve(table, key, value_size, |buf| {
            rkyv::to_bytes_in(value, buf)?;
            Ok(())
        })
    }

    /// MultimapTable for 1:N relations (per ADR 0002)
    pub fn multimap_insert<K, V>(
        &self,
        table: &str,
        key: &K,
        value: &V,
    ) -> Result<(), DbError>
    where
        K: rkyv::Serialize,
        V: rkyv::Serialize,
    {
        let mut txn = self.inner.begin_write()?;
        let mut table = txn.open_multimap_table(
            MultimapTableDefinition::new(table)
        )?;

        let key_bytes = rkyv::to_bytes(key)?;
        let value_bytes = rkyv::to_bytes(value)?;

        table.insert(key_bytes.as_ref(), value_bytes.as_ref())?;
        txn.commit()?;
        Ok(())
    }

    /// Bulk write with Durability::None (per ADR 0002)
    pub fn batch_write<F>(&self, batch_fn: F) -> Result<(), DbError>
    where
        F: FnOnce(&mut WriteTransaction) -> Result<(), DbError>,
    {
        let mut txn = self.inner.begin_write()?;
        txn.set_durability(Durability::None);  // Defer fsync

        batch_fn(&mut txn)?;

        txn.commit()?;

        // Final fsync for entire batch
        let mut final_txn = self.inner.begin_write()?;
        final_txn.set_durability(Durability::Immediate);
        final_txn.commit()?;

        Ok(())
    }
}
```

**Usage Patterns (Context-Specific)**:

```rust
// note.rs (static methods, NOT traits)
impl Note {
    /// Zero-copy read for LSP hot paths
    pub fn find_by_id_archived(
        db: &Database,
        id: Uuid,
    ) -> Result<impl Deref<Target = rkyv::Archived<Note>> + '_, NoteError> {
        db.get_archived("notes", &id).map_err(Into::into)
    }

    /// Full load for mutations
    pub fn find_by_id(db: &Database, id: Uuid) -> Result<Option<Self>, NoteError> {
        db.get("notes", &id).map_err(Into::into)
    }

    /// Save with index updates
    pub fn save(&self, db: &Database) -> Result<(), NoteError> {
        db.put("notes", &self.id, self)?;

        // Update secondary indexes using MultimapTable
        for tag in &self.tags {
            db.multimap_insert("tags_to_notes", tag, &self.id)?;
        }

        for link in &self.outbound_links {
            db.multimap_insert("backlinks", &link.target, &self.id)?;
        }

        Ok(())
    }

    /// Find by tag (using multimap index)
    pub fn find_by_tag(db: &Database, tag: &str) -> Result<Vec<Uuid>, NoteError> {
        db.multimap_get("tags_to_notes", &tag)?
            .map(|guard| *guard.deref())  // Zero-copy read Uuid
            .collect()
    }
}

// schema.rs (same pattern, different types)
impl Schema {
    pub fn find_by_name(db: &Database, name: &str) -> Result<Option<Self>, SchemaError> {
        db.get("schemas", &name).map_err(Into::into)
    }

    pub fn save(&self, db: &Database) -> Result<(), SchemaError> {
        db.put("schemas", &self.name, self).map_err(Into::into)
    }
}
```

**Rules for Database Layer**:

1. **NO cache/ folder with traits** - Use concrete `Database` type
2. **NO macros** - Use generic methods on `Database`
3. **Pass `&Database` as parameter** - Match `std::fs::File` pattern
4. **Expose zero-copy primitives**:
   - `get_archived()` returns `ArchivedGuard` (hot path)
   - `put_reserve()` uses `insert_reserve` (zero-copy write)
   - `multimap_insert/get()` for 1:N indexes
   - `batch_write()` for bulk operations
5. **Validation once at boundary** - `bytecheck` in `get_archived()`, then unsafe access
6. **Defer LSP features to Phase 2**:
   - NO Moka (in-memory L1 cache)
   - NO Coordinator (L1/L2 orchestration)
   - NO backfiller (async background population)

**Deferred Components (Phase 2 - LSP)**:

When LSP is implemented, `Database` gains:
```rust
pub struct Database {
    inner: redb::Database,
    l1_cache: Option<moka::Cache<Vec<u8>, Vec<u8>>>,  // Added in Phase 2
}
```

The API remains the same; caching becomes transparent internal optimization.

**Tradeoffs**:

- **Pros**:
  - ✅ Zero abstraction overhead (concrete type)
  - ✅ Full zero-copy support (AccessGuard, insert_reserve, MultimapTable)
  - ✅ Idiomatic Rust (matches std::fs pattern)
  - ✅ Simple for CLI (no trait soup)
  - ✅ Future-ready for LSP (add cache field internally)

- **Cons**:
  - ❌ Less polymorphic (but we only have one backend: redb per ADR 0002)
  - ❌ Harder to mock in tests (but can use in-memory redb or wrap in trait for tests)

**Why NOT traits?**

| Aspect           | Trait Approach                       | Concrete Type Approach               | Winner   |
| ---------------- | ------------------------------------ | ------------------------------------ | -------- |
| **stdlib pattern**   | Not used (std::fs::File is concrete) | Matches std::fs::File                | Concrete |
| **Zero-copy**        | Requires GATs and complex lifetimes  | Natural with ArchivedGuard           | Concrete |
| **IDE support**      | Trait objects hide implementations   | F12 goes directly to impl            | Concrete |
| **Error messages**   | "trait bound not satisfied"          | Clear concrete type errors           | Concrete |
| **Mocking**          | Easy with trait objects              | Wrap in trait or use in-memory redb  | Trait    |
| **Future backends**  | Easy to swap                         | Locked to redb (but ADR 0002 decided) | Trait    |

**Decision**: Concrete type wins 5/6 criteria, and the "locked to redb" is already decided in ADR 0002.

**Dependency Flow**:

```
fs/ (NO dependencies, pure utilities)
  ↑
db.rs (uses redb + rkyv, NO domain knowledge)
  ↑
note/aggregate.rs (pure domain, NO db import)
note/indexing.rs (uses Database, implements save/find methods)
  ↑
lithos-cli (orchestrates db + contexts)
```

---

### 5. CQRS Pattern: Optional Trait Boundaries

**Target**: `implementation-patterns-consistency-rules.md`, Epics 3, 4, 5, 9
**Epic Affected**: Epic 3 (Domain), Epic 9 (Storage), Epic 14 (CLI)
**Priority**: P2 - Medium (Design pattern, not blocking)

**Problem**: CQRS (Command-Query Responsibility Segregation) is mentioned in the architecture, but the single-crate refactor removes the natural boundary enforcement of separate crates. Do we need explicit CQRS traits, or can we rely on naming conventions?

**Context**: CQRS separates:
- **Commands** (writes): `save()`, `delete()`, `update()`
- **Queries** (reads): `find_by_id()`, `find_by_tag()`, `list_all()`

**Evaluated Options**:

**Option A: Explicit CQRS Traits (Strong Boundaries)**

```rust
// note/commands.rs
pub trait NoteCommands {
    fn save(&self, note: &Note) -> Result<(), NoteError>;
    fn delete(&self, id: Uuid) -> Result<(), NoteError>;
}

// note/queries.rs
pub trait NoteQueries {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;
    fn find_by_tag(&self, tag: &str) -> Result<Vec<Note>, NoteError>;
}

// note/storage.rs
pub struct NoteStorage {
    db: Database,
}

impl NoteCommands for NoteStorage { ... }
impl NoteQueries for NoteStorage { ... }
```

**Pros**:
- ✅ Explicit architectural boundary (can't accidentally mix read/write)
- ✅ Easy to mock for testing
- ✅ Clear documentation of capabilities

**Cons**:
- ❌ Extra abstraction layer
- ❌ Trait object overhead if using `&dyn NoteCommands`
- ❌ Not idiomatic Rust (std doesn't use CQRS traits)

**Option B: Static Methods with Naming Convention (Lightweight)**

```rust
// note.rs
impl Note {
    // Queries (by convention)
    pub fn find_by_id(db: &Database, id: Uuid) -> Result<Option<Self>, NoteError> { ... }
    pub fn find_by_tag(db: &Database, tag: &str) -> Result<Vec<Self>, NoteError> { ... }

    // Commands (by convention)
    pub fn save(&self, db: &Database) -> Result<(), NoteError> { ... }
    pub fn delete(db: &Database, id: Uuid) -> Result<(), NoteError> { ... }
}
```

**Pros**:
- ✅ Idiomatic Rust (matches std::fs pattern)
- ✅ Zero abstraction overhead
- ✅ Simple and direct

**Cons**:
- ❌ No compiler-enforced CQRS boundary
- ❌ Harder to mock (need to pass real Database or trait-wrap it)

**Option C: Hybrid (Traits Only Where Needed)**

```rust
// note.rs - Default: static methods
impl Note {
    pub fn find_by_id(db: &Database, id: Uuid) -> Result<Option<Self>, NoteError> { ... }
    pub fn save(&self, db: &Database) -> Result<(), NoteError> { ... }
}

// note/ports.rs - OPTIONAL: Define traits for testing/mocking
pub trait NoteRepository {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;
    fn save(&self, note: &Note) -> Result<(), NoteError>;
}

// Blanket impl for Database
impl NoteRepository for Database {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        Note::find_by_id(self, id)
    }
    fn save(&self, note: &Note) -> Result<(), NoteError> {
        note.save(self)
    }
}
```

**Pros**:
- ✅ Idiomatic by default (static methods)
- ✅ Traits available when needed (testing)
- ✅ Best of both worlds

**Cons**:
- ❌ Duplication between static methods and trait impls

**Decision**: **Option B (Static Methods)** for CLI MVP, with Option C available if testing demands it.

**Rationale**:
1. **YAGNI**: CLI doesn't need polymorphism (single Database implementation)
2. **Idiomatic**: Matches Rust stdlib patterns
3. **Simple**: Fewer files, less cognitive load
4. **Testable**: Can use in-memory redb for tests without traits

**Rules**:

1. **Default: Static methods on domain types**
   ```rust
   impl Note {
       pub fn find_by_id(db: &Database, id: Uuid) -> Result<Option<Self>, NoteError>;
       pub fn save(&self, db: &Database) -> Result<(), NoteError>;
   }
   ```

2. **Naming Convention (CQRS by name)**:
   - **Queries**: `find_*`, `get_*`, `list_*`, `count_*`
   - **Commands**: `save`, `delete`, `update`, `create`

3. **OPTIONAL: Add traits later if needed**:
   - When testing requires mocking
   - When LSP requires different storage backend
   - Define in `note/ports.rs` (deferred)

4. **NO separate commands.rs/queries.rs files for MVP**
   - Keep methods in main module (`note.rs` or `note/aggregate.rs`)
   - Split only if file exceeds 500 lines

**File Structure**:

```
note/
├── aggregate.rs         # Note struct + all methods (find_*, save, etc.)
├── frontmatter.rs
├── indexing.rs          # Helper: update_tag_index(), update_backlink_index()
├── error.rs
└── events.rs

# NO commands.rs or queries.rs for MVP
# NO ports.rs unless testing requires it
```

**Migration Path (Phase 2 - LSP)**:

If LSP requires trait-based polymorphism (e.g., mock storage for tests):

```rust
// note/ports.rs (added in Phase 2)
pub trait NoteRepository {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;
    fn save(&self, note: &Note) -> Result<(), NoteError>;
}

// Static methods remain primary API
impl Note {
    pub fn find_by_id(db: &Database, id: Uuid) -> Result<Option<Self>, NoteError> {
        db.get("notes", &id).map_err(Into::into)
    }
}

// Trait impl delegates to static methods
impl NoteRepository for Database {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        Note::find_by_id(self, id)
    }
}
```

**Tradeoffs**:

- **Pros**:
  - ✅ Simpler for CLI (no trait soup)
  - ✅ Idiomatic Rust (matches std pattern)
  - ✅ Zero overhead (no trait objects)
  - ✅ Future-ready (can add traits later)

- **Cons**:
  - ❌ Harder to mock (but in-memory redb works)
  - ❌ No compiler-enforced CQRS boundary (rely on naming)

---

### 6. Async vs. Sync Model: Sync‑First Execution Model (Async Only at Edges)

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

### 7. ADR Audit & Cleanup

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

### 8. Error Handling Standardization

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

### 9. Naming & Style Cleanup

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

### 10. Domain Serialization Strategy (Idiomatic Rust Default)

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
