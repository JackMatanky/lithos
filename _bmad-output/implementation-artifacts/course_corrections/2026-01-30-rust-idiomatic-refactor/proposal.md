# Sprint Change Proposal: Rust Idiomatic Architecture Refactor

**Status:** Approved
**Date:** 2026-01-30
**Approved:** 2026-02-01
**Approver:** Jack (Product Owner)
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

| Aspect              | Trait Approach                       | Concrete Type Approach                | Winner   |
| ------------------- | ------------------------------------ | ------------------------------------- | -------- |
| **stdlib pattern**  | Not used (std::fs::File is concrete) | Matches std::fs::File                 | Concrete |
| **Zero-copy**       | Requires GATs and complex lifetimes  | Natural with ArchivedGuard            | Concrete |
| **IDE support**     | Trait objects hide implementations   | F12 goes directly to impl             | Concrete |
| **Error messages**  | "trait bound not satisfied"          | Clear concrete type errors            | Concrete |
| **Mocking**         | Easy with trait objects              | Wrap in trait or use in-memory redb   | Trait    |
| **Future backends** | Easy to swap                         | Locked to redb (but ADR 0002 decided) | Trait    |

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

**Decision**: **Option A (Explicit CQRS Traits)** with dedicated implementation files.

**Rationale**:

1. **Architecture Compliance**: Enforces the ports & adapters pattern defined in the architecture.
2. **Testability**: Allows mocking of data access without spinning up a full database.
3. **Clarity**: Separates data access logic from domain logic completely.
4. **Organization**: Co-located `command.rs` and `query.rs` files keep contexts self-contained.

**Rules**:

1. **Define Traits in Ports**:
   - `note/ports.rs` defines `Command` and `Query` traits.

2. **Implement in Dedicated Files**:
   - `note/command.rs` implements `Command`.
   - `note/query.rs` implements `Query`.

3. **Use Database Reference**:
   - Implementations hold `&'db Database`.

**File Structure**:

```
note/
├── aggregate.rs         # Pure domain logic
├── ports.rs             # Command/Query trait definitions
├── command.rs           # Command implementation (NoteCommand<'db>)
├── query.rs             # Query implementation (NoteQuery<'db>)
├── frontmatter.rs
├── error.rs
└── events.rs
```

**Migration Path (Phase 2 - LSP)**:

LSP can simply use the traits for dependency injection, allowing for easy mocking or alternative backends if needed.

**Tradeoffs**:

- **Pros**:
  - ✅ Compiler-enforced boundary
  - ✅ Excellent for testing
  - ✅ Clear separation of concerns
  - ✅ Future-proof

- **Cons**:
  - ❌ Slightly more boilerplate than static methods
  - ❌ Requires struct wrappers (`NoteCommand`, `NoteQuery`)

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

This section provides a concrete execution roadmap aligned with the 10 detailed change proposals. Each phase maps directly to specific proposals and includes acceptance criteria.

**Critical Phase Reordering**: The Database Layer (Phase 6) is the most complex and risky component. To minimize friction, we will first restructure the workspace and domain (Phases 2-5) using a **stubbed Database interface**, then implement the concrete Redb/Rkyv logic in Phase 6.

---

### Phase 1: Foundation & Documentation (Architect)

**Status**: ✅ Partially Complete (Task 1.2 Done)
**Duration**: 1-2 days
**Blocking**: Must complete before Phase 2
**Proposals**: 1, 2, 3, 7, 10

#### Tasks

**1.1 ADR Audit (Proposal 7) - ✅ DONE**

- [x] Move non-decision ADRs to `docs/guides/`
- [x] Archive outdated ADRs
- [x] Create/Rename ADRs (ADR 0002, 0009, etc.)
- [x] Renumber remaining ADRs sequentially
- [x] Update all ADR references in epics/docs

**1.2 Architecture Documentation Updates (Proposals 1, 2, 3) - ✅ DONE**

- [x] Update `_bmad-output/project-context.md`:
  - Replace "Hexagonal Architecture" with "Dependency Flow Architecture"
  - Update structure diagrams (single-crate, db.rs)
  - Add "File First, Folder Second" rule (Proposal 3)
  - Add "Zero-Copy Primitives" section (Proposal 4)
- [x] Update `project-structure-boundaries.md`:
  - Document `<context>.rs` + `<context>/` pattern (NO mod.rs)
  - Document co-located errors/events/ports (Proposal 2)
  - Document dependency flow rules (domain → db.rs → cli)
- [x] Update `implementation-patterns-consistency-rules.md`:
  - Add "Database Access Rules" from Proposal 4
  - Add "CQRS Naming Conventions" from Proposal 5
  - Add "Sync-First Rules" from Proposal 6
  - Remove cache/ trait references

**1.3 Error Strategy Documentation (Proposal 2) - ✅ DONE**

- [x] Document co-located error pattern:
  - `note/error.rs`, `schema/error.rs`, etc.
  - NO centralized `errors.rs`
  - Optional top-level `LithosError` composition
- [x] Update epic stories to reference context-specific errors

**1.4 Serde Feature Flag Strategy (Proposal 10) - ✅ DONE**

- [x] Document feature flag pattern in architecture docs
- [x] Add to domain type guidelines:
  - `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`
  - Feature-gated, not required

#### Acceptance Criteria

- [x] All ADRs renumbered and validated (`mise run adr:validate` passes)
- [x] New ADR 0017 approved and merged
- [x] Architecture docs reflect single-crate structure
- [x] No references to old multi-crate structure remain in docs
- [x] All new architectural patterns (Zero-copy, sync-first) documented

---

### Phase 2: Workspace Restructuring (Dev)

**Duration**: 1 day
**Blocking**: Must complete before domain migration
**Proposals**: 1

#### Tasks

**2.1 Workspace Definition (Proposal 1)**

- [ ] Update `Cargo.toml` workspace definition
  - Remove: `crates/domain`, `crates/app`, `crates/adapters`
  - Add: `crates/lithos-core`, `crates/lithos-cli`
- [ ] Create `crates/lithos-core/Cargo.toml` with dependencies:
  - `redb` (persistence)
  - `rkyv` (zero-copy serialization)
  - `thiserror` (structured errors)
  - `serde` (optional, feature-gated per Proposal 10)
  - NO `async-trait`, NO `moka`, NO `tokio` (defer to Phase 2)
- [ ] Update `mise.toml` task definitions:
  - Replace `test:unit:domain` → `test:unit:core`
  - Remove separate crate test tasks
  - Keep `test:arch` for boundary enforcement

**2.2 Directory Structure Creation**

- [ ] Create `crates/lithos-core/src/` directory structure:
  ```
  lithos-core/src/
  ├── lib.rs
  ├── db.rs (stub only)
  ├── fs/ (utilities)
  ├── config.rs
  ├── config/
  ├── note.rs
  ├── note/
  ├── schema.rs
  ├── schema/
  ├── template.rs
  └── template/
  ```

#### Acceptance Criteria

- [ ] `cargo check` passes with new workspace structure
- [ ] `crates/lithos-core` exists and compiles (with empty lib.rs)

---

### Phase 3: Domain Migration & Module Restructure (Dev)

**Duration**: 2-3 days
**Blocking**: Must complete before CQRS
**Proposals**: 2, 3, 9

**⚠️ CRITICAL: One Context at a Time**

Migrate contexts **individually**, starting with `config`, then `note`, `schema`, `template`. After **each** context migration:

1. Stage the files (`git add`)
2. Run pre-commit hooks (`git commit` or `mise run verify`)
3. **All hooks must pass before proceeding to next context**

This incremental approach prevents accumulation of errors and makes debugging easier.

#### Tasks

**3.1 Module Migration (Proposal 3) - Per Context**

For **each** context (config → note → schema → template), in order:

1. **Create `<context>.rs` entry file**:
   - Merge `_legacy/crates/domain/src/<context>/mod.rs` + `aggregate.rs` content
   - Module-level documentation
   - `mod` declarations for submodules
   - Public re-exports (`pub use`)
   - Aggregate root implementation
   - Add `#[expect(clippy::module_name_repetitions)]` on error enums
   - Add `#[non_exhaustive]` on public structs/enums

2. **Create supporting files in `<context>/` folder**:
   - Move: `types.rs`, `vault.rs`, `global.rs` (config), etc.
   - Move: `events.rs` (keep co-located, Proposal 2)
   - Create: `error.rs` (extract from `_legacy/crates/domain/src/errors.rs`)
   - Optional: `ports.rs` (only if traits needed, Proposal 5)

3. **Verify and Commit**:
   - Run `cargo check -p lithos-core` - must pass
   - Run `cargo clippy -p lithos-core` - must pass (no errors)
   - Stage: `git add lithos-core/src/<context>.rs lithos-core/src/<context>/`
   - Commit: `git commit -m "feat(config): migrate config context to lithos-core"`
   - **All pre-commit hooks must pass**

4. **Proceed to next context only after clean commit**

**Delete obsolete files after ALL contexts migrated**:

- ❌ `_legacy/crates/domain/src/config/mod.rs` (merged into `config.rs`)
- ❌ `_legacy/crates/domain/src/config/aggregate.rs` (merged into `config.rs`)
- ❌ `_legacy/crates/domain/src/errors.rs` (split into context errors)
- ❌ `_legacy/crates/domain/src/ports/` (moved to context/ports.rs)

**3.2 Error Migration (Proposal 2) - Per Context**

For **each** context during its migration:

- [ ] Create `<context>/error.rs`:
  - Extract relevant variants from `_legacy/crates/domain/src/errors.rs`
  - Use `thiserror::Error` derivation
  - Add `#[expect(clippy::module_name_repetitions, reason = "Context-specific error")]`
  - Context-specific error types

- [ ] Update all `Result<T, DomainError>` → `Result<T, <Context>Error>` within the context

**3.3 Naming Cleanup (Proposal 9) - Per Context**

During each context migration:

- [ ] Remove suffixes in that context:
  - `VaultWriterPort` → `VaultWriter`
  - `VaultFileDto` → `VaultFile`
  - `ConfigPort` → Config (use module scoping instead)

**3.4 Import Rewrites - Per Context**

During each context migration:

- [ ] Update imports within the migrated context:
  - `use crate::...` (internal) instead of `use domain::...`
  - Remove all `async_trait` imports (sync-first)
  - Remove all cache trait imports (replaced by `db.rs`)

#### Migration Order

1. **Config** (simplest context, ~6 files) → Commit
2. **Note** (core entity, ~8 files) → Commit
3. **Schema** (~7 files) → Commit
4. **Template** (~6 files) → Commit

#### Acceptance Criteria

- [ ] All domain contexts migrated to `lithos-core/src/`
- [ ] NO `mod.rs` files remain in `lithos-core/src/`
- [ ] All contexts have `error.rs` co-located
- [ ] Each context has its own commit with passing pre-commit hooks
- [ ] `cargo check -p lithos-core` passes after each context
- [ ] `cargo clippy -p lithos-core` passes (no errors) after each context

---

### Phase 4: CQRS Implementation with Stubs (Dev)

**Duration**: 1-2 days
**Blocking**: Must complete before CLI refactor
**Proposals**: 4, 5

#### Tasks

**4.1 Database Placeholder (Stub)**

- [ ] Create stub `crates/lithos-core/src/db.rs`:
  ```rust
  pub struct Database;
  impl Database {
      pub fn open(_path: &std::path::Path) -> Result<Self, DbError> { Ok(Self) }
      // Add stub methods needed for compilation
  }
  ```

**4.2 CQRS Implementation (Proposal 5)**

For each context:

- [ ] Add static methods to aggregate (in `<context>.rs`) using the stubbed Database:

  ```rust
  // Queries (read-only)
  impl Note {
      pub fn find_by_id(_db: &Database, _id: Uuid) -> Result<Option<Self>, NoteError> {
          todo!("Implement in Phase 6")
      }
      pub fn find_by_path(_db: &Database, _path: &str) -> Result<Option<Self>, NoteError> {
          todo!("Implement in Phase 6")
      }
      pub fn list_all(_db: &Database) -> Result<Vec<Self>, NoteError> {
          todo!("Implement in Phase 6")
      }
  }

  // Commands (write)
  impl Note {
      pub fn save(&self, _db: &Database) -> Result<(), NoteError> {
          todo!("Implement in Phase 6")
      }
      pub fn delete(_db: &Database, _id: Uuid) -> Result<(), NoteError> {
          todo!("Implement in Phase 6")
      }
  }
  ```

#### Acceptance Criteria

- [ ] Static methods defined (find\_\*, save, delete)
- [ ] Code compiles with stubbed Database
- [ ] No functional implementation yet (todo! macros acceptable)

---

### Phase 5: CLI & Sync Refactor (Dev)

**Duration**: 2-3 days
**Blocking**: Must complete before Database Layer
**Proposals**: 6, 8

#### Tasks

**5.1 CLI Restructure**

- [ ] Rename `crates/cli` → `crates/lithos-cli`
- [ ] Update CLI `Cargo.toml`:
  - Depend on `lithos-core`
  - Add `miette` for error reporting (Proposal 8)
  - Remove `async-trait`, `tokio` (unless needed for concurrency)

**5.2 De-Async Core Logic (Proposal 6)**

- [ ] Remove `async` from database layer stubs (sync by default)
- [ ] Remove `#[async_trait]` from domain
- [ ] CLI orchestration:
  - Use sync calls directly: `Note::find_by_id(&db, id)?`
  - IF async needed (e.g., parallel indexing):
    - Use `tokio::task::spawn_blocking`
    - Keep async at edges only

**5.3 Error Reporting (Proposal 8)**

- [ ] CLI uses `miette` for user-facing errors
- [ ] Core uses `thiserror` (already done in Phase 3)

**5.4 Remove Cache Infrastructure**

- [ ] Delete obsolete files:
  - ❌ `crates/adapters/src/spi/cache/` (entire folder)
  - ❌ `crates/app/` (entire crate)
  - ❌ All async trait implementations
  - ❌ Coordinator/Moka/Backfiller (defer to LSP Phase 2)

**5.5 Architecture Tests**

- [ ] Create `tests/arch/boundary_tests.rs`:
  - Ensure `note/aggregate.rs` does NOT import `db`
  - Ensure domain modules are pure (no IO)
  - Ensure dependency flow is enforced

#### Acceptance Criteria

- [ ] CLI compiles
- [ ] NO async in core domain logic
- [ ] Error reporting uses miette (CLI) and thiserror (core)
- [ ] All obsolete cache/ code deleted
- [ ] `mise run test:arch` passes

---

### Phase 6: Database Layer Implementation (Dev)

**Duration**: 3-4 days
**Blocking**: Must complete before Verification
**Proposals**: 4

#### Tasks

**6.1 Implement db.rs Infrastructure**

- [ ] Update `crates/lithos-core/src/db.rs` (replace stub):
- [ ] Implement `Database` struct:
  - Wraps `redb::Database`
  - Constructor: `Database::open(path: &Path) -> Result<Self>`
  - Configuration: `set_cache_size()`, `set_page_size()` per ADR 0002
- [ ] Implement `ArchivedGuard<'txn, V>` wrapper:
  - `Deref<Target = rkyv::Archived<V>>`
  - Lifetime tied to transaction
  - Validation in constructor

**6.2 Implement Zero-Copy Read Primitives**

- [ ] `get_archived<K, V>() -> Result<ArchivedGuard<'_, V>>`
  - Validates alignment (ADR 0002 requirement)
  - Validates bytes with `rkyv::check_archived_root`
  - Returns guard with transaction lifetime
- [ ] `get<K, V>() -> Result<Option<V>>`
  - Full deserialization
  - Delegates to `get_archived()` + `deserialize()`

**6.3 Implement Zero-Copy Write Primitives**

- [ ] `put_reserve<K, V, F>()` using `insert_reserve`
  - Accepts closure to write directly to DB page
  - Zero intermediate allocation
- [ ] `put<K, V>()` convenience wrapper
  - Uses `put_reserve()` internally

**6.4 Implement MultiMap for Indexes**

- [ ] `multimap_insert<K, V>()`
  - For 1:N relations (tags → notes, backlinks)
  - Uses `MultimapTable` per ADR 0002
- [ ] `multimap_get<K, V>() -> Iterator<ArchivedGuard>`

**6.5 Implement Batch Operations**

- [ ] `batch_write<F>()`
  - Sets `Durability::None` for batch
  - Calls closure with `&mut WriteTransaction`
  - Final commit with `Durability::Immediate`

**6.6 Connect Domain to Real Implementation**

- [ ] Update `Note::find_by_id` etc. to use real `db` methods
- [ ] Remove `todo!` macros

**6.7 Testing**

- [ ] Unit tests for each primitive
- [ ] Zero-copy benchmarks

#### Acceptance Criteria

- [ ] All db.rs primitives implemented and tested
- [ ] Zero-copy benchmarks pass (10x improvement over deserialize)
- [ ] Domain methods function correctly against real DB
- [ ] `mise run test:unit:core` passes

---

### Phase 7: Frontmatter Serialization Solution (Dev)

**Status**: CRITICAL - BLOCKING MVP
**Duration**: 1-2 days
**Blocking**: Must complete before verification
**Proposals**: 4 (Database Layer)
**Problem**: Recursive `FieldValue` enum causes trait solver overflow in rkyv derives

#### Background

During Phase 6 implementation, we discovered that `Frontmatter` contains a recursive `FieldValue` enum:

```rust
pub enum FieldValue {
    Array(Vec<FieldValue>),      // Recursion
    Object(HashMap<String, FieldValue>),  // Recursion
    Boolean(bool),
    String(String),
    Number(f64),
    Date(i64),
}
```

This causes rustc trait solver to overflow when deriving rkyv traits, preventing Note serialization.

**Current Workaround**: Note aggregate skips frontmatter field using `#[rkyv(with = rkyv::with::Skip)]`, resulting in data loss (frontmatter always `None` after deserialization).

**Impact**:

- ❌ **BLOCKING**: Frontmatter is critical metadata for MVP (titles, aliases, file classes, custom fields)
- ❌ Note serialization is incomplete without frontmatter
- ❌ Users lose all YAML frontmatter data on round-trip

#### Solution Options

**Option A: Separate Frontmatter Storage (RECOMMENDED)**

Store frontmatter separately from Note aggregate using JSON/TOML serialization:

```rust
// db.rs additions
impl Database {
    /// Store frontmatter as JSON (serde-based, not rkyv)
    pub fn put_json<K, V>(&self, table: &str, key: &K, value: &V) -> Result<(), DbError>
    where
        K: rkyv::Serialize,
        V: serde::Serialize,
    {
        let json = serde_json::to_vec(value)?;
        let key_bytes = rkyv::to_bytes(key)?;

        let mut txn = self.inner.begin_write()?;
        let mut table = txn.open_table(TableDefinition::new(table))?;
        table.insert(key_bytes.as_ref(), json.as_slice())?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_json<K, V>(&self, table: &str, key: &K) -> Result<Option<V>, DbError>
    where
        K: rkyv::Serialize,
        V: serde::de::DeserializeOwned,
    {
        let txn = self.inner.begin_read()?;
        let table = txn.open_table(TableDefinition::new(table))?;
        let key_bytes = rkyv::to_bytes(key)?;

        match table.get(key_bytes.as_ref())? {
            Some(guard) => Ok(Some(serde_json::from_slice(guard.value())?)),
            None => Ok(None),
        }
    }
}

// note/command.rs - save frontmatter separately
impl Command for NoteCommand<'_> {
    fn create(&self, path: String) -> Result<Note, NoteError> {
        let note = Note::new(Uuid::now_v7(), path)?;

        // Save note (rkyv, zero-copy, frontmatter skipped)
        self.db.put("notes", &note.id.to_string(), &note)?;

        // Save frontmatter separately (serde JSON)
        if let Some(ref fm) = note.frontmatter {
            self.db.put_json("frontmatter", &note.id.to_string(), fm)?;
        }

        Ok(note)
    }
}

// note/query.rs - load and merge
impl Query for NoteQuery<'_> {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        let id_str = id.to_string();

        // Load note (zero-copy rkyv)
        let mut note: Option<Note> = self.db.get_owned("notes", &id_str)?;

        // Load frontmatter separately (serde)
        if let Some(ref mut n) = note {
            n.frontmatter = self.db.get_json("frontmatter", &id_str)?;
        }

        Ok(note)
    }
}
```

**Pros**:

- ✅ Preserves all frontmatter data
- ✅ No trait solver issues (serde handles recursion)
- ✅ Note aggregate remains zero-copy for hot paths
- ✅ Frontmatter can evolve independently
- ✅ Simpler than custom rkyv implementation

**Cons**:

- ❌ Two database operations per note (one for note, one for frontmatter)
- ❌ Frontmatter not zero-copy (but rarely accessed in LSP hot paths)
- ❌ Slightly more complex query logic

**Option B: Manual rkyv Implementation**

Implement `Archive`, `Serialize`, `Deserialize` manually for `FieldValue`:

```rust
// frontmatter.rs - manual impl (100+ lines)
impl rkyv::Archive for FieldValue { /* ... */ }
impl rkyv::Serialize for FieldValue { /* ... */ }
impl rkyv::Deserialize for FieldValue { /* ... */ }
```

**Pros**:

- ✅ Single storage location
- ✅ Fully zero-copy

**Cons**:

- ❌ Complex manual implementation (error-prone)
- ❌ Breaks with rkyv API changes
- ❌ High maintenance burden
- ❌ Attempted during Phase 6 - resulted in 50+ compilation errors

**Option C: Flatten FieldValue (Breaking Change)**

Redesign frontmatter to avoid recursion:

```rust
pub enum FieldValue {
    Boolean(bool),
    String(String),
    Number(f64),
    Date(i64),
    // No Array or Object - store as JSON string instead
}
```

**Pros**:

- ✅ rkyv derives work

**Cons**:

- ❌ Loses structured data (arrays, nested objects)
- ❌ Breaking change to domain model
- ❌ Less useful for complex frontmatter

#### Decision

**CHOOSE Option A: Separate Frontmatter Storage**

**Rationale**:

1. **Proven Solution**: serde handles recursive types reliably
2. **Low Risk**: Small, isolated change to command/query implementations
3. **Performance**: Frontmatter is cold data (not accessed in LSP hover/completion hot paths)
4. **Maintainability**: No manual rkyv implementations to maintain
5. **Completeness**: Preserves all frontmatter data for MVP

#### Tasks

**7.1 Extend Database Layer**

- [ ] Add `put_json<K, V>()` method to `db.rs`
- [ ] Add `get_json<K, V>()` method to `db.rs`
- [ ] Add `DbError` variants for JSON serialization errors
- [ ] Add `serde_json` dependency to lithos-core

**7.2 Update Note CQRS Implementations**

- [ ] Update `NoteCommand::create()`:
  - Save note with rkyv
  - Save frontmatter with `put_json()` if present
- [ ] Update `NoteCommand::update()`:
  - Update note with rkyv
  - Update frontmatter with `put_json()` or delete if None
- [ ] Update `NoteCommand::delete()`:
  - Delete note
  - Delete frontmatter entry
- [ ] Update `NoteQuery::find_by_id()`:
  - Load note with rkyv
  - Load frontmatter with `get_json()` and merge
- [ ] Update `NoteQuery::find_by_path()`:
  - Same merge logic
- [ ] Update `NoteQuery::list()`:
  - Load all notes, merge frontmatter for each

**7.3 Update Note Aggregate**

- [ ] Remove `#[rkyv(with = rkyv::with::Skip)]` from frontmatter field
- [ ] Change to: `#[rkyv(skip)] #[serde(skip)]` with documentation:
  ```rust
  /// Frontmatter is stored separately using JSON serialization.
  /// This field is populated during query operations.
  #[rkyv(skip)]
  #[serde(skip)]
  pub frontmatter: Option<Frontmatter>,
  ```

**7.4 Testing**

- [ ] Unit test: `put_json` / `get_json` roundtrip
- [ ] Unit test: Frontmatter with nested objects and arrays
- [ ] Integration test: Note CRUD with frontmatter preservation
- [ ] Integration test: Frontmatter survives multiple updates

#### Acceptance Criteria

- [ ] Frontmatter roundtrips correctly (no data loss)
- [ ] All FieldValue variants (Array, Object, etc.) work
- [ ] Note CRUD operations preserve frontmatter
- [ ] `mise run test:unit:core` passes
- [ ] No rkyv trait solver errors
- [ ] Documentation updated explaining separate storage

#### Migration Note

This is a **temporary solution** for CLI MVP. In Phase 2 (LSP), we can revisit:

- Option: Implement manual rkyv for FieldValue if zero-copy frontmatter becomes critical
- Option: Keep separate storage (current solution is simple and works)

For now, **separate storage is the pragmatic path to MVP completion**.

---

### Phase 8: Verification & Validation (Tea)

**Duration**: 1-2 days
**Blocking**: Final gate before merge
**Proposals**: All

#### Tasks

**8.1 Test Suite Repair**

- [ ] Fix all broken unit tests:
  - Update imports
  - Remove async from test functions
  - Update error types
  - Replace mock cache with in-memory Database

- [ ] Integration tests:
  - Test full flows: CLI → Database → Domain
  - Test index updates (tags, backlinks)
  - Test batch operations (vault indexing)

**8.2 Performance Validation**

- [ ] Run zero-copy benchmarks
- [ ] Compare against baseline

**8.3 Quality Gates**

- [ ] `mise run verify` passes (full suite)
- [ ] `mise run test:arch` passes
- [ ] Coverage check

**8.4 Final Checklist**

- [ ] NO `mod.rs` files in codebase
- [ ] NO `cache/` folder exists
- [ ] NO `async_trait` in core
- [ ] All errors co-located
- [ ] All static methods use `&Database` parameter
- [ ] Full test suite passes

#### Acceptance Criteria

- [ ] `mise run verify` returns 100% green
- [ ] All 10 proposals fully implemented
- [ ] No regressions in existing functionality

---

### Required Agents

| Phase   | Agent     | Responsibilities                                       |
| ------- | --------- | ------------------------------------------------------ |
| Phase 1 | Architect | ADRs, docs, structure planning                         |
| Phase 2 | Dev       | Workspace restructuring                                |
| Phase 3 | Dev       | Domain migration, module restructure                   |
| Phase 4 | Dev       | CQRS stubs, module entry points                        |
| Phase 5 | Dev       | CLI refactor, de-async                                 |
| Phase 6 | Dev       | Database layer implementation (complex)                |
| Phase 7 | Dev       | Frontmatter Serialization Solution                     |
| Phase 8 | Tea       | Test repair, benchmarks, quality gates, final sign-off |

---

### Rollback Strategy

If critical issues arise:

1. **Phase 1-2**: Revert commits (documentation and workspace)
2. **Phase 3-6**: Create rollback branch from pre-migration commit
   - Multi-crate structure still exists
   - Can continue development while investigating issues
3. **Post-Phase 7**: If regressions found:
   - Fix forward (don't rollback after merge)
   - Create hotfix branch
   - Cherry-pick fixes to main

---

### Success Metrics

| Metric                       | Target                              | Validation                   |
| ---------------------------- | ----------------------------------- | ---------------------------- |
| **Zero-copy speedup**        | 5-10x faster reads                  | Benchmark comparison         |
| **Build time**               | 1.5-2x faster compilation           | `cargo build --timings`      |
| **Test coverage**            | Critical paths 100% covered         | Manual review (not % target) |
| **Code reduction**           | -1000+ lines (removed abstractions) | `git diff --stat`            |
| **Vault indexing**           | <2s for 1000 notes                  | Integration test             |
| **CLI operations**           | <500ms per command                  | End-to-end test              |
| **Quality gates**            | `mise run verify` 100% green        | CI pass                      |
| **Documentation compliance** | All ADRs validated                  | `mise run adr:validate` pass |

---

**⚠️ APPROVAL REQUIRED**: Explicit user approval to proceed with this destructive change.

**Estimated Total Duration**: 10-15 days (assuming sequential execution)

**Risk Level**: HIGH (total structural reset, 100% code displacement)

**Mitigation**: Phased approach, stub-first implementation, comprehensive testing, rollback strategy documented
