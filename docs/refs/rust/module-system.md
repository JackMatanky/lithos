# Rust Module System Guide (2018+ Edition)

**Status**: Reference Guide
**Date**: 2026-01-30
**Purpose**: Document modern Rust module organization patterns for Rust 2018+ projects

---

## Table of Contents

1. [Module Declaration Patterns](#module-declaration-patterns)
2. [Visibility Control](#visibility-control)
3. [Organization Principles](#organization-principles)
4. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
5. [Examples from Real Projects](#examples-from-real-projects)

---

## Module Declaration Patterns

### Pattern 1: Single-File Module (Small Modules)

**When to use:** Module has <200 lines of code

```rust
// src/config.rs
pub struct Config { }
impl Config { }
```

**Declaration in parent:**
```rust
// src/lib.rs
mod config;
pub use config::Config;
```

---

### Pattern 2: File + Folder (Modern Style - Rust 2018+)

**When to use:** Module has multiple submodules or >200 lines

```rust
// src/note.rs (module declaration + public API)
pub mod aggregate;
pub mod events;
pub mod frontmatter;

// Re-export public types
pub use aggregate::Note;
pub use events::{NoteCreated, NoteEvents};
pub use frontmatter::Frontmatter;

// src/note/ (implementation folder)
├── aggregate.rs
├── events.rs
└── frontmatter.rs
```

**✅ MODERN (2018+):** Module declaration is in `note.rs`
**❌ OLD (pre-2018):** Module declaration was in `note/mod.rs`

---

### Pattern 3: Folder with mod.rs (Old Style - Still Valid)

**Status:** Still valid but **discouraged** in new code

```rust
// src/note/mod.rs (OLD STYLE)
pub mod aggregate;
pub use aggregate::Note;

// src/note/
├── mod.rs          ← Declaration here (old style)
├── aggregate.rs
└── events.rs
```

**Why avoid:** Creates many files named `mod.rs`, confusing in editors

---

## Visibility Control

### Public Visibility Levels

| Keyword | Visibility | Use Case |
|---------|-----------|----------|
| `pub` | Public to all | External API |
| `pub(crate)` | Public within crate | Internal API |
| `pub(super)` | Public to parent module | Helper functions |
| `pub(in crate::path)` | Public to specific path | Rare, specific needs |
| (none) | Private to module | Implementation details |

### Hexagonal Architecture via Visibility

**Instead of separate crates, use visibility:**

```rust
// lithos-core/src/lib.rs
pub mod note {          // Public module
    pub use note_impl::Note;  // Public API
}

pub(crate) mod storage {  // Internal only
    pub(crate) struct RedbStorage { }
}

mod internal_utils {      // Private
    fn helper() { }
}
```

**This replaces:**
```toml
# ❌ OLD APPROACH: Separate crates
[dependencies]
lithos-domain = { path = "../domain" }
lithos-adapters = { path = "../adapters" }
```

**With:**
```rust
// ✅ NEW APPROACH: Visibility control
pub(crate) mod adapters {
    pub(crate) struct RedbStorage { }
}
```

---

## Organization Principles

### Principle 1: Organize by Feature/Context, Not Layer

**✅ GOOD (Feature-based):**
```
src/
  note/           # Note feature
  schema/         # Schema feature
  template/       # Template feature
  storage/        # Storage infrastructure (alongside features)
```

**❌ BAD (Layer-based):**
```
src/
  domain/         # "Layer" wrapper
    note/
    schema/
  infrastructure/ # "Layer" wrapper
    storage/
```

**Evidence:** Every major Rust project (tokio, rust-analyzer, clap) uses feature-based.

---

### Principle 2: Flat > Nested

**From Matklad's "Large Rust Workspaces":**
> "Even comparatively large lists are easier to understand at a glance than even small trees."

**✅ GOOD (Flat):**
```
src/
  note.rs
  note/
  schema.rs
  schema/
  template.rs
  template/
  storage.rs
  storage/
```

**❌ BAD (Nested):**
```
src/
  contexts/
    note/
      models/
        aggregate.rs
```

**Why:** `ls src/` gives immediate project overview. Nested requires mental tree traversal.

---

### Principle 3: Module File = Public API Declaration

**Modern pattern (file + folder):**

```rust
// src/note.rs (public API boundary)
//! Note bounded context.

// ONLY declare submodules and re-exports here
pub mod aggregate;
pub mod events;

pub use aggregate::Note;
pub use events::NoteCreated;

// NO implementation code in this file!
```

**Implementation goes in folder:**
```rust
// src/note/aggregate.rs
pub struct Note { }
impl Note { }  // ✅ Implementation here
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Deep Nesting

**❌ BAD:**
```
src/
  domain/
    bounded_contexts/
      note/
        entities/
          aggregate/
            note.rs  ← 6 levels deep!
```

**✅ GOOD:**
```
src/
  note/
    aggregate.rs  ← 2 levels, clear path
```

---

### Anti-Pattern 2: Using `mod.rs` in New Code

**❌ OLD (pre-2018):**
```
src/
  note/
    mod.rs          ← Avoid in new code
    aggregate.rs
```

**✅ NEW (2018+):**
```
src/
  note.rs           ← Modern declaration style
  note/
    aggregate.rs
```

**Exception:** Already written code can stay `mod.rs` (don't refactor just for this).

---

### Anti-Pattern 3: Cargo.toml for Architectural Boundaries

**❌ BAD (Using crates for layers):**
```toml
# crates/domain/Cargo.toml
[dependencies]
# No dependencies ← enforced by Cargo
```

**✅ GOOD (Using visibility):**
```rust
// src/lib.rs
pub(crate) mod domain {  // ← enforced by compiler
    pub struct Note { }
}
```

**Why:** Visibility is faster to compile and more idiomatic Rust.

---

## Examples from Real Projects

### rust-analyzer (200k LOC, 32 crates)

**crates/hir/src/lib.rs:**
```rust
mod attrs;
mod from_id;
mod semantics;

pub mod db;           // Public module
pub mod diagnostics;

pub use semantics::Semantics;
```

**Pattern:** Flat organization, `pub mod` for public APIs, private `mod` for internal.

---

### tokio (Runtime library)

**tokio/src/lib.rs:**
```rust
pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;
pub mod sync;
pub mod task;

mod loom;     // Internal testing utilities
```

**Pattern:** Feature-based modules at root level, no "domain/" or "infrastructure/" wrappers.

---

### serde (Serialization framework)

**serde uses separate crates, but for TECHNICAL reasons:**
```
serde/           # Core traits
serde_derive/    # Proc macro (MUST be separate crate)
serde_core/      # Implementation
```

**Reason:** `proc-macro = true` requires separate crate (Rust limitation).

**NOT** because "serialization" and "deserialization" are different "layers".

---

## Migration Checklist

When converting from old to new style:

- [ ] Move `src/foo/mod.rs` → `src/foo.rs`
- [ ] Keep only declarations/re-exports in `foo.rs`
- [ ] Keep implementation in `src/foo/` folder
- [ ] Update `mod` statements in parent modules
- [ ] Run `cargo check` to verify
- [ ] Update `ARCHITECTURE.md` if exists

---

## Quick Reference

| Question | Answer |
|----------|--------|
| **File or folder?** | File if <200 lines, folder if larger |
| **mod.rs or file.rs?** | Use `file.rs` (modern style) |
| **Where is public API?** | In the module declaration file (`note.rs`) |
| **Where is implementation?** | In the folder (`note/aggregate.rs`) |
| **How to enforce boundaries?** | Use `pub(crate)`, not separate crates |
| **How to organize?** | By feature/context, not by layer |

---

## Sources

1. **Rust Book - Module System:**
   https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

2. **Rust Reference - Visibility:**
   https://doc.rust-lang.org/reference/visibility-and-privacy.html

3. **Matklad - Large Rust Workspaces:**
   https://matklad.github.io/2021/08/22/large-rust-workspaces.html

4. **Matklad - Fast Rust Builds:**
   https://matklad.github.io/2021/09/04/fast-rust-builds.html

5. **rust-analyzer source code:**
   https://github.com/rust-lang/rust-analyzer

6. **tokio source code:**
   https://github.com/tokio-rs/tokio

---

## Revision History

- **2026-01-30:** Initial version (architect agent + Jack)
