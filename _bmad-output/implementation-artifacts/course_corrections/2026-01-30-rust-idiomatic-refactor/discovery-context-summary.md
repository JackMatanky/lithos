## Discovery Context

### 1. The Core Realization: The "Translation Gap"
The fundamental failure mode identified was **treating Rust as a generic implementation detail rather than a distinct system**. We had fallen into the trap of "writing a program that only happened to use a particular language," applying high-level architectural patterns (Hexagonal/Ports & Adapters) without translating them into Rust's specific idioms.
- **The Gap**: We mapped logical boundaries directly to physical compilation units (Crates) instead of Rust's native encapsulation primitives (Modules/Visibility).
- **The Result**: This naive translation fought against the language's strengths, turning Rust's zero-cost abstractions into high-cost barriers.

### 2. The Symptom: Cache Performance & Ownership
This philosophical misalignment manifested concretely during the **Cache Foundation (Epic 5)** implementation. What appeared to be specific "cache utility issues" were actually symptoms of the underlying translation gap:
- **Zero-Copy Violation**: Passing `rkyv` `AccessGuard`s across crate boundaries broke compiler inlining, forcing expensive allocations or unsafe workarounds.
- **Performance Impact**: The multi-crate structure imposed a **5-10x performance penalty** (link-time optimization barriers) that threatened our sub-50ms LSP latency target.
- **Ownership Complexity**: We were fighting the borrow checker to enforce boundaries that the compiler could have handled trivially within a single crate.

### 3. Research Phase: Rust Ecosystem Analysis
To understand why our implementation felt so resistant to Rust's strengths, we compared our architecture against idiomatic projects (`tokio`, `rust-analyzer`, `clap`). This confirmed the "Translation Gap":
- **Workspace Misalignment**: We used workspaces like Java/Go "packages" (internal layering), whereas idiomatic Rust uses them for **independent, reusable libraries**.
- **The "Inlining Barrier"**: We confirmed that Rust compilers do not inline generic code across crate boundaries by default, necessitating expensive Link-Time Optimization (LTO).
- **Compilation Overhead**: The quadratic monomorphization costs of the 4-crate structure were significantly slowing down the "edit-compile-test" loop.

### 4. The Pivot: Holistic Architecture Review
The investigation escalated from a specific cache problem to a systemic architectural review (documented in `2026-01-30-critical-architecture-review.md`).
- **Decision**: The rigid multi-crate structure must be dismantled.
- **New Direction**: We will pivot to a **Single-Crate "Core" Architecture** (`lithos-core`) that leverages Rust's module system (`pub(crate)`) for encapsulation. This aligns our architecture with the language, enabling zero-copy optimizations to flow naturally as intended.
