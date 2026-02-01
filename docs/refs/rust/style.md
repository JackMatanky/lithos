# Rust Style Reference

**Primary sources (paraphrased):**
- Rust official style guide (default Rust style)
- rust-analyzer contributor style guide

This document paraphrases a subset of rust-analyzer’s style guidance into reviewable rules.
It focuses on reviewable, repeatable practices for readability, maintainability, and performance.

When there’s any ambiguity about formatting, treat **`rustfmt` output as authoritative**.

This is intentionally separate from [docs/refs/rust/idioms.md](idioms.md), which tracks the Rust Design Patterns “Idioms” chapter.

For linting/testing/tooling workflow guidance (Clippy discipline, doc tests, benchmarking), see [docs/refs/rust/quality-tooling.md](quality-tooling.md).

## Scope and how to use this

- Treat this as **style + code-organization guidance** (how to structure code and changes).
- Prefer these rules for application and library code.
- For “pure” modules (no I/O), favor explicit invariants and minimal coupling.

## 0) Default Rust formatting (let rustfmt do it)

The Rust style guide’s core thesis is: formatting is mostly mechanical, so let tools do it.

- **Indentation**: spaces only; indent in multiples of 4.
- **Line width**: 100 columns for code is the default target.
- **Block indent over visual indent**: prefer
  - `foo(\n    a,\n    b,\n)`
  - over aligning arguments under the opening paren.
- **Trailing commas**: include them when an item list is split across lines.
- **Blank lines**: separate items and statement blocks with zero or one blank line.
- **Trailing whitespace**: never.

Comments and attributes (high leverage, even with rustfmt):

- Prefer `//` over `/* ... */`.
- Put a single space after `//`.
- Prefer a comment on its own line; if it trails code, use a single preceding space.
- Comments should usually be full sentences (capitalize; end with a period).
- Prefer naming “non-obvious” comments with a prefix like `SAFETY:`, `PERF:`, or `CONTEXT:` so reviewers know what to look for.
- Keep comment-only lines reasonably short (the official guide targets 80 columns for full-line comments).
- Prefer `///` doc comments over `/** ... */`.
- Prefer outer doc comments (`///`) and reserve inner docs (`//!`) for module/crate docs.
- Put doc comments before attributes.
- Put each attribute on its own line; keep `#[derive(...)]` as a single attribute.
- For `#[attr = value]`, keep single spaces around `=`.

## 1) Scale of changes (boundaries first)

When reviewing or authoring a change, identify which category it falls into:

1. **Internal change:** no public surface change (no new `pub`, no new re-exports).
2. **API expansion:** adds/changes a `pub` API.
3. **New dependency edge:** adds a new dependency or introduces a new re-export that couples components.

Rules of thumb:

- For (1): merge is mostly about correctness, tests, and “no panics for unhappy paths”.
- For (2): minimize churn; keep the diff small; consider splitting “API change” and “implementation change”.
- For (3): be conservative—dependency edges are hard to undo.

## 2) Dependencies: conservative by default

rust-analyzer is conservative with crates.io dependencies to keep compile times low.
Lithos already has a curated stack; follow the same spirit:

- Avoid adding “tiny helper crates” without a strong justification.
- Prefer writing small utilities locally (or in an internal utility module/crate if the project uses one).
- Be especially careful about adding dependencies that might violate the hexagonal boundaries.

## 3) Tests: minimal, explicit, and never panic-oriented

### Minimal fixtures

- Test fixtures should be **minimal** and trimmed down.
- For multiline fixtures, prefer raw string literals that start at column 0 for easier copy/paste and offset reasoning.

### Tests should read like documentation

- Prefer test names that describe **behavior + condition** (so failures are self-explanatory in output).
- Use modules to group a unit-of-work: `mod parse { ... }`, then tests like `rejects_empty_input`.

### Keep tests narrow

- Prefer **one behavior per test**.
- Prefer very few assertions per test (ideally one); split tests rather than piling assertions.

### Doc tests and nextest (Lithos note)

- `nextest` doesn’t execute doc tests.
- If you use `nextest`, make sure doc tests are still run (e.g., `cargo test --doc` in CI).

### Avoid `#[should_panic]`

- Prefer explicit checks (`assert!(result.is_err())`, match the error, etc.).
- The goal is “no panics even on invalid input”, not “panics in the right way”.

### Avoid `#[ignore]`

- Don’t hide broken tests.
- If behavior is currently wrong, assert the wrong behavior and add a focused explanation of why it is wrong and what should change.

## 4) Preconditions and control flow: push decisions outward

### Encode preconditions in types

Prefer signatures that require the precondition, rather than accepting `Option<T>` and silently doing nothing.

- Good: `fn frobnicate(walrus: Walrus)`
- Avoid: `fn frobnicate(walrus: Option<Walrus>)` with an early `return` on `None`

### Push control flow to callers

Don’t hide control flow in a helper that returns early on a global condition.
Make the condition visible where the helper is called.

### Early returns are good

Prefer guard clauses; they reduce indentation and make the “happy path” stand out.

Also, prefer `return Err(err)` over `Err(err)?` for early error exits.

## 5) Public fields vs getters (and never setters)

- If a field can take any value without breaking invariants, consider making it public.
- If there is an invariant:
  - document it,
  - enforce it in construction,
  - keep the field private,
  - add a getter.
- Don’t provide setters; they tend to spread invariant enforcement across the codebase.

Getter guidance:

- Return borrowed views (`&str`, `Option<&T>`, slices) rather than owned clones.

## 6) Prefer “useful types” (types on the left)

Prefer more general borrowed types:

- `&[T]` over `&Vec<T>`
- `&str` over `&String`
- `Option<&T>` over `&Option<T>`
- `&Path` over `&PathBuf`

(These overlap with idioms in [docs/refs/rust/idioms.md](idioms.md) but are repeated here because they are central to style and API shape.)

## 7) Construction guidance: `Default` beats `new()` (when sensible)

- Prefer `#[derive(Default)]` (or manual `Default`) over a zero-arg `new()`.
- Avoid inventing “dummy default states” just to satisfy `Default`—if there’s no sensible empty value, make callers choose.

Small consistency rule:

- Prefer `Vec::new()` over `vec![]` when the empty vector is the intent.

## 8) Functions over “doer objects”

Avoid outward APIs that force callers to allocate or construct a temporary object just to invoke one method:

- Prefer `do_thing(arg1, arg2)`
- Avoid `ThingDoer::new(arg1, arg2).do()`

It can still be fine to create an internal “context struct” as an implementation detail.

## 9) Many parameters: use a config struct (but don’t over-default)

- If a function takes many optional/bool params, prefer a `Config` struct.
- Don’t automatically derive `Default` for every config—callers often know better defaults.
- Don’t store config inside long-lived state; pass it explicitly.

If the variation also changes return type/shape, consider a command/query type.

## 10) Prefer two functions over `Option`/`bool` parameters (when literal)

If a function is always called with literal `true/false` or `Some/None`, split it:

- `foo()` and `foo_with_bar(Bar)`

This removes “false sharing” where two unrelated code paths become entangled inside one function.

## 11) Import organization and item order

### Import groups

- Separate `use` groups with blank lines.
- Prefer one `use` per crate (keeps dependencies visually obvious).
- Keep brace imports tight (no spaces inside braces): `use foo::{bar, baz};`.
- When an import list is split across lines, use block indentation and trailing commas.

Apollo-derived convention (compatible with rustfmt):

- Prefer grouping as: `std` / external crates / workspace crates / `super::` / `crate::`.

Ordering guidance (pragmatic):

- Keep import ordering stable and let `rustfmt` handle intra-group sorting.
- Avoid large “style-only” reorder diffs unless you’re already editing the file.
- If a file has both `use` and `mod`, prefer following the existing local convention in that file.

### Prefer crate-root paths

Prefer `use crate::foo::bar` over `use super::bar` in most cases for consistency.

### Item order: optimize for first-time readers

- Put the main public entry points first.
- Put types before impls; order type declarations top-down (parent before children).
- If you have nested helper functions, place them at the end of the enclosing function and keep nesting shallow.

## 12) Naming: boring, explicit, consistent

- Prefer longer, “boring” names; they work well with editor completion.
- Default local name can follow the type name (`global_state: GlobalState`).
- Use established acronyms consistently (`db`, `ctx`, `acc`).

Default casing conventions:

- Types, traits, enum variants: `UpperCamelCase`.
- Modules, functions, methods, locals, struct fields: `snake_case`.
- Macros: `snake_case`.
- Constants (`const` and immutable `static`): `SCREAMING_SNAKE_CASE`.

Reserved words:

- If a name collides with a keyword, prefer a raw identifier (`r#type`) or a trailing underscore (`type_`).
- Avoid misspelling the word just to dodge the keyword.

## 13) Control-flow micro-style (readability)

- Prefer `<` / `<=` comparisons over `>` / `>=` when expressing bounds.
- Prefer `match` over `if let ... else ...` when both branches are present.
- Don’t use the `ref` keyword in matches (match ergonomics makes it redundant).
- If a match arm is intentionally empty, prefer `=> (),`.

Combinators:

- Use `map`/`and_then`/etc. when natural.
- If the chain becomes “clever” or hard to scan, switch to `if`/`match`/`for`.

Types:

- Prefer explicit type ascription on bindings over turbofish on `collect()`.

## 14) Helpers: blocks and variables are cheap; avoid single-use helpers

- Prefer a local block with access to surrounding context over a single-use helper function.
- Exception: a helper can be worthwhile if it enables `return` or `?` in a cleaner way.

Introduce helper variables freely, especially to name complex conditions.

## 15) Performance style (“avoid pessimization”)

- Avoid allocations where an iterator will do.
- If allocation is inevitable, prefer pushing the allocation responsibility to the caller rather than allocating internally.
- Avoid intermediate collections; use accumulators for recursive “build a set/list” patterns.
- Be mindful of monomorphization costs at boundaries; type parameters everywhere can slow compilation.

Lithos note: Balance this with “don’t optimize before profiling” — avoid obviously wasteful patterns, but don’t contort code for micro-optimizations.

## 16) Documentation style

- Inline comments should read like sentences (capitalized, ending with a period).
- For Markdown files, prefer one sentence per line (diff-friendly).

## 17) Cargo.toml style (when you touch manifests)

- Use the same indentation and line width spirit as Rust code.
- Put exactly one space around `=`.
- Don’t indent keys.
- Use blank lines *between sections*, not between keys within a section.
- Prefer stable ordering: keep `[package]` first; keep `name` and `version` at the top of `[package]`.
- For arrays that don’t fit on one line, use a block form with one item per line and trailing commas.
- Prefer inline tables when short; otherwise split into a dedicated `[dependencies.crate_name]` section.

## RA-specific guidance (do not copy blindly)

The rust-analyzer guide includes rules tailored to rust-analyzer’s architecture.

- Prefer `&Path` / `PathBuf` for filesystem paths.
- Don’t adopt project-specific path wrapper types unless you standardize them.
- Don’t adopt collection-type conventions wholesale without a clear team standard.
