# Module Structure Lessons for File-Based, Read-Heavy Rust Systems
## Research Report: Ports/Adapters and Submodule Boundaries

**Date:** March 11, 2026
**Focus:** How successful Rust systems structure modules, where to place boundaries, and when to use ports/adapters

---

## Executive Summary

Based on external research (rust-analyzer architecture, matklad’s ARCHITECTURE.md guidance, Rust API design articles, and Rust book guidance on modules), the dominant approach in Rust for file-based, read-heavy systems is:

1. **Use module boundaries over trait boundaries** for business logic.
2. **Use ports/adapters only at I/O boundaries** (filesystem, database, network).
3. **Split submodules by responsibility and stability**, not by “layer purity.”
4. **Keep data flows linear and explicit** (file → parse → transform → cache).
5. **Treat API boundaries as separate crates or modules** with different rules (serialization, exposure, stability).

---

## Sources (External)

1. **matklad: ARCHITECTURE.md**
   https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html

2. **rust-analyzer Architecture Guide**
   https://rust-analyzer.github.io/book/contributing/architecture.html

3. **Elegant APIs in Rust (Pascal Hertleif)**
   https://deterministic.space/elegant-apis-in-rust.html

4. **Rust Book: Modules and Privacy**
   https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html

---

## 1. What Real Rust Systems Do for Module Structure

### 1.1 rust-analyzer: Strong Module/Crate Boundaries

**Key takeaways (from rust-analyzer’s architecture guide):**

- **Crates act as API boundaries.** The `syntax`, `hir`, and `ide` crates explicitly act as different boundary layers with different constraints.
- **Input vs derived state separation** (base-db holds input facts; everything else is derived).
- **I/O is pushed to the outermost layer** (the `rust-analyzer` crate handles LSP and serialization).
- **Invariants are documented explicitly** (“this crate never does I/O”, “this crate is tree-independent”).

**Lesson for Lithos:** For read-heavy systems, **separate input ingestion (FS/IO) from pure domain logic**, and clearly document boundaries. Treat each module boundary as an invariant contract.

### 1.2 matklad: ARCHITECTURE.md as a Structural Tool

**Key takeaways:**

- Teams lose time **finding where to change code**, not understanding the code itself.
- A small `ARCHITECTURE.md` is the fastest way to fix that.
- The doc should include:
  - **Bird’s-eye overview**
  - **Codemap** (“where’s the thing that does X?”)
  - **Invariants** (often defined by *absence* of something)
  - **Boundaries** (where rules change)

**Lesson for Lithos:** Use the doc as the authoritative guide for module layout. Keep it stable and short, and encode “what is NOT allowed” (e.g., “note does not import schema”).

### 1.3 Elegant APIs: Traits for I/O Boundaries, Not Business Logic

**Key takeaways:**

- Traits are best used where you need **polymorphism for I/O**, not for domain logic.
- Rust favors **simple, ergonomic APIs** with minimal abstraction layers.
- Builders and explicit constructors are preferred over heavy frameworks.

**Lesson for Lithos:** Ports/Adapters should exist only for **filesystem, database, and external services**. Domain logic should be plain functions and structs.

### 1.4 Rust Book: Modules for Privacy and Structure

**Key takeaways:**

- Modules are the **primary tool** for grouping related code.
- The **filesystem layout mirrors the module tree**.
- Privacy is the default; public APIs are explicit.

**Lesson for Lithos:** Organize contexts by module with **public API at the top** and private internal submodules beneath.

---

## 2. Ports and Adapters: When to Use Them

### 2.1 Use Ports/Adapters When

✅ **You need multiple I/O implementations** (real FS vs in-memory FS, Redb vs test DB).
✅ **You want deterministic testing** without OS or network dependencies.
✅ **You need to swap providers** (e.g., different storage engines).
✅ **You need to enforce architectural boundaries** (domain must not depend on I/O).

**Typical ports for file-based systems:**

- `FsReader` (real FS vs in-memory)
- `Repository` (Redb vs fake)
- `Clock` (real time vs fixed time for tests)
- `Hasher` (real vs deterministic test)

### 2.2 Avoid Ports/Adapters When

❌ There is only **one concrete implementation** and no test benefit.
❌ The abstraction hides domain complexity rather than I/O complexity.
❌ The trait becomes a “god interface” (generic CRUD or 10+ methods).

**Rule of thumb:** If the trait is not used by tests or alternative backends, **don’t introduce it**.

---

## 3. When to Create Submodules Within a Context

### 3.1 Split Into Submodules When

✅ **Responsibility is distinct and stable** (e.g., parsing vs resolution).
✅ **Types belong to a separate conceptual layer** (raw vs domain).
✅ **The file exceeds ~300–500 lines** and is no longer scannable.
✅ **There are multiple independent algorithms** (e.g., resolver vs expander).
✅ **Tests would otherwise become tangled** (each submodule can carry focused tests).

### 3.2 Keep Flat When

❌ The module is still under ~300 lines.
❌ All functions operate on the same data shape and lifecycle.
❌ Splitting would introduce circular imports or false layering.

---

## 3.3 Context-Internal Submodules (Concrete Guidance)

**Goal:** Isolate dependencies, parsing complexity, or data-shape variants without creating new architectural layers.

### Note Context: `parser` Submodule (pulldown-cmark Isolation)

**Why:** Markdown parsing depends on pulldown-cmark specifics and AST walking, which should not leak into domain types.

**Structure:**

```
note/
├── mod.rs           # Public API (Note, Tag, Link, Task, etc.)
├── raw.rs           # RawNote syntax representation
├── parser/          # pulldown-cmark boundary
│   ├── mod.rs
│   ├── events.rs    # Parser event stream helpers
│   ├── headings.rs  # Heading extraction logic
│   ├── links.rs     # Link extraction logic
│   ├── tags.rs      # Tag extraction logic
│   └── tasks.rs     # Task extraction logic
├── loader.rs        # Orchestrates file → raw → domain
└── error.rs
```

**Rule:** The `note::parser` submodule should only expose **pure functions** that take raw markdown and return structured `Raw*` or intermediate parsed types. Domain types (`Note`, `Task`, `Tag`) should not depend on pulldown-cmark types.

### Schema Context: `property_spec` Submodule

**Why:** Property specs are a distinct language/shape that evolve independently of schema resolution and storage.

**Structure:**

```
schema/
├── mod.rs            # Public API (Schema, Property, etc.)
├── raw.rs            # RawSchema input
├── property_spec/    # Spec parsing + validation
│   ├── mod.rs
│   ├── raw.rs        # RawPropertySpec (serde-only)
│   ├── parser.rs     # Spec parsing helpers
│   └── error.rs
├── resolver.rs       # Inheritance and reference resolution
├── loader.rs         # Orchestrates pipeline
└── error.rs
```

**Rule:** `property_spec` is allowed to define its own raw/domain types if the spec language is complex. Keep them scoped to the submodule to avoid global type fan-out.

### Template Context: `compiler` or `render` Submodule

**Why:** Template compilation/rendering (MiniJinja) often has specific dependency and error types.

**Structure:**

```
template/
├── mod.rs
├── raw.rs
├── compiler/         # MiniJinja integration
│   ├── mod.rs
│   ├── env.rs
│   └── render.rs
├── loader.rs
└── error.rs
```

**Rule:** Keep MiniJinja types isolated to avoid leaking them into domain APIs.

---

## 4. Submodule Patterns for File-Based Systems

### Pattern A: Parse → Resolve → Store Pipeline

```
context/
├── mod.rs           # Public API, re-exports
├── raw.rs           # Raw* types (serde parsing)
├── domain.rs        # Validated domain types
├── parser.rs        # Syntax parsing functions
├── resolver.rs      # Cross-file resolution
├── loader.rs        # Orchestrates pipeline
├── error.rs         # Context-specific errors
└── repository.rs    # Trait for cache/storage I/O
```

**Use this when:** the context has a multi-phase pipeline (schema, templates).

### Pattern B: Simple Context (Minimal Submodules)

```
context/
├── mod.rs
├── raw.rs
├── domain.rs
├── loader.rs
└── error.rs
```

**Use this when:** parsing and validation are straightforward (config).

### Pattern C: Read-Heavy Indexing Context

```
context/
├── mod.rs
├── raw.rs
├── domain.rs
├── index.rs         # Query helpers and lookup structures
├── loader.rs
└── error.rs
```

**Use this when:** indexing and lookup structures are a major concern (notes).

---

## 5. Boundary Rules for Read-Heavy Systems

### 5.1 Input vs Derived State

From rust-analyzer:
- **Input facts are stored separately** from derived data.
- Derived data should be recomputable and **not depend on I/O**.

**For Lithos:**
- Raw files are input facts.
- Domain types are derived but stable.
- Database is a cache (derived data).

### 5.2 API Boundary Rules

From rust-analyzer:
- **Serialization belongs at the edge** (LSP layer).
- Inner modules should avoid serialization constraints.

**For Lithos:**
- JSON serialization should live in CLI only.
- Core domain types should be pure, serializable only if needed for caching.

---

## 6. Decision Matrix: Ports/Adapters vs Modules

| Need | Use Module | Use Port/Adapter |
|------|------------|------------------|
| Organize domain logic | ✅ | ❌ |
| Abstract file I/O | ❌ | ✅ |
| Abstract database | ❌ | ✅ |
| Multiple storage backends | ❌ | ✅ |
| Single concrete implementation | ✅ | ❌ |
| Avoiding god objects | ✅ (split functions) | ❌ |
| Testing without I/O | ❌ | ✅ |

---

## 7. Practical Rules for Lithos (Derived From Research)

1. **Modules are the first boundary.** Start by splitting responsibilities into modules, not traits.
2. **Ports only for I/O.** Use traits for `FsReader`, `Repository`, and similar boundaries.
3. **Submodules map to pipeline phases.** Raw → Domain → Resolve → Store should each be a file.
4. **Keep the public API thin.** Re-export from `mod.rs`, hide internals by default.
5. **Document invariants in one place.** Use ARCHITECTURE.md and context-specific README files.
6. **Avoid data-model fanout.** Do not create `Aggregate`, `Stored`, `View`, `Projection` types unless profiling demands it.

---

## 8. Open Questions for Lithos (To Address Later)

1. Should schema resolution live in its own submodule or be part of loader?
2. Which contexts need true indexing submodules (note vs schema)?
3. Should config parsing share a common raw/validate utility with other contexts?
4. When introducing a `Repository` trait, should it live per-context or in a shared `db` module?

---

## 9. Summary

The external research consistently emphasizes **module structure first, ports/adapters second**. File-based, read-heavy Rust systems keep business logic in plain modules and reserve trait abstraction for I/O seams. Submodules should reflect stable responsibilities and pipeline phases, not theoretical layers.

**Actionable outcome for Lithos:**
- Use context modules with internal submodules for raw/domain/loader.
- Add ports/adapters only where the system touches the outside world.
- Keep the module graph simple, enforce invariants in docs, and optimize only when profiling demands it.
