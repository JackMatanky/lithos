# Canonical Rust Best Practices (Alignment Notes)

**Source (read-only reference):** https://canonical.github.io/rust-best-practices/

This document maps Canonical’s guidance into **reviewable rules** for Lithos.
It is written in our own words (no verbatim copying) and is intended to complement:
- [docs/refs/rust/style.md](style.md) (style + organization)
- [docs/refs/rust/idioms.md](idioms.md) (Rust Design Patterns idioms)
- [docs/refs/rust/quality-tooling.md](quality-tooling.md) (tooling + quality gates)
- [docs/refs/rust/module-system.md](module-system.md) (module organization)

## Coverage

Canonical’s guide is structured into these areas (with sub-sections). This file aims to align with each of them:

- Preconditions
- Cosmetic discipline (spacing, grouping, hex)
- Naming discipline (content, pattern bindings, generics, lifetimes, builders)
- Import discipline (no globs, grouping, nested imports, `self::` re-exports)
- Pattern matching discipline (exhaustiveness as a maintenance alarm)
- Code discipline (construction, mutability scoping, shadowing, bounds/annotations, early drop, local helper/wire types, `format!` args)
- Error and panic discipline (message shape, typed errors, early conversion, panic policy)
- Function discipline (unit-return clarity, hiding generics, unused params, builder ergonomics)
- Ordering discipline (file + impl ordering, derive/declaration ordering, field ordering)
- Unsafe discipline (`unsafe` minimization + `SAFETY:` contracts)
- Structural discipline (module root as manifest + placement of `Error`/`Result`)
- Comment discipline (first sentence, definite language)

## 1) Preconditions (quality gates)

Before merging or calling a change “done”, ensure:
- Formatting is clean (`mise run fmt`).
- Lints are clean (`mise run lint`).
- Tests pass (`mise run test`).
- Prefer the full gate when in doubt (`mise run verify`).

If code is feature-gated, ensure lints and tests cover all relevant feature combinations (Canonical calls this out explicitly; Lithos’s `mise` tasks are the authoritative entry point).

## 2) Cosmetic discipline (spacing + grouping)

Use whitespace to communicate structure:
- Avoid “decorative” blank lines; prefer semantic grouping.
- Keep strongly-associated statements together (e.g., `let x = ...;` followed immediately by its validation guard).

High-signal heuristics:
- If a local is declared and only used in the immediately-following block, keep them adjacent (no blank line).
- If a local is used across multiple later blocks, visually separate it from any one block (insert a blank line).
- Keep “declare then validate” adjacent; if validation grows into a multi-step block, insert a blank line *after* the validation block.

Grouping:
- Don’t interleave unrelated work; group computations into readable stages.
- Avoid helper closures that sit far from their use site (especially if they capture nothing). Prefer a small private `fn` or inline logic near where it’s used.
- If a closure captures values, declare it near its use to reduce “things to remember”.

Literals:
- Prefer lowercase hex unless a local convention requires otherwise.

## 3) Naming discipline

Naming should reduce ambiguity:
- Prefer names that imply roles (`haystack`/`needle`, `input`/`output`, `src`/`dst`).
- Keep word-order consistent across APIs (e.g., `parse_*`, `render_*`, `validate_*`).
- Avoid encoding types in names unless needed to disambiguate.

More Canonical-aligned guidance:
- Prefer a single term per concept; avoid synonyms that imply false distinctions.
- Prefer concise names that still stand alone (avoid obscure abbreviations).
- If you can’t find a good name, treat it as a smell: the API/abstraction may need refactoring.
- Be extra conservative with public names; renames are breaking changes.

Pattern binding naming:
- When destructuring, keep variable names aligned with the source field name.

Generic + lifetime naming:
- Generic type parameters are typically single-letter (e.g., `T`, `K`, `V`) so they don’t masquerade as concrete types.
- Lifetimes should be meaningful when they matter to the user; avoid defaulting to `'a`/`'b` when a semantic name communicates relationships.
- Lifetime names should reflect what is borrowed (the source of the reference), not the destination type.
- Avoid numbering lifetimes.

Builders:
- If a type offers a builder, prefer `Type::builder() -> TypeBuilder`.
- Prefer `build()` to be fallible when invariants can fail.

## 4) Import discipline

Avoid imports that obscure provenance:
- Avoid glob imports (`use foo::*`) in production code.
- Avoid depending on “prelude” modules for application/library code.
- Exception: `use super::*;` is acceptable inside `#[cfg(test)] mod tests { ... }`.

More specifics:
- Avoid glob-importing enum variants (hides the enum type in control flow).
- If an enum name is long in a tight scope, prefer a local alias scoped to that function.

Grouping:
- Keep imports grouped as `std`/`core`/`alloc`, then external crates, then workspace/self/super/crate.

Import form:
- Prefer nested imports (`use foo::{a, b};`) over repeating the full path on many lines.

Explicit module paths:
- When re-exporting from child modules, prefer `pub use self::child::Type;` to avoid future name collisions.

## 5) Pattern-matching discipline

Use pattern matching as a maintenance alarm:
- Prefer destructuring `self`/structs in a way that forces compiler errors when new fields appear and must be considered.

Avoid patterns that hide what’s happening:
- Prefer explicit dereferencing over `|&x| x`-style patterns when mapping references.
- Avoid numeric tuple indexing (`.0`, `.1`) when a named `let (x, y) = point;` is clearer.
- Avoid pattern-matching in function parameters; destructure as the first statement so signatures stay focused on the API.

Exception:
- Pattern-matched parameters are common and acceptable in closures (local scope + inferred types).

## 6) Code discipline (small but high-leverage rules)

`Self`:
- Prefer `Self` in `impl` blocks to reduce noise, except when constructing associated types where naming the concrete type is clearer.

Struct population:
- Prefer “field init shorthand” when variable names match fields.
- Keep struct field initialization ordered the same as the struct definition.
- Avoid mixing “trivial bindings” and “big computations” inside a struct literal; compute complex fields with `let` bindings first.

Tuple population:
- If a tuple literal becomes multi-line, prefer `let` bindings for components and then `(a, b)`.

Collections:
- Prefer `Vec::new()` for “intentionally empty”.
- Prefer `Vec::with_capacity(n)` when you have a reasonable estimate.
- Avoid `vec![expr; 0]` patterns; they still evaluate `expr`.

Mutability:
- Keep `mut` scopes tight (construct with a scoped `let mut`, then return an immutable value).

Avoid “declare then assign later” locals:
- Prefer expression-oriented Rust (`if`/`match`/`loop { break ... }`) over `let x; ... x = ...;`.

Ownership clarity:
- Prefer `let v = expr; f(&v);` over binding a reference to a temporary and passing it around.
- As a rule of thumb, only start a `let` binding with `&` when you’re slicing/indexing.

Shadowing:
- Same-scope shadowing with the same type can be okay.
- If shadowing is “mutability in disguise”, use a scoped-mutable construction block instead.
- Same-scope shadowing that changes types should be rare and typically limited to a single step.

Generic constraints:
- Keep bounds unified; if any constraint needs `where`, move all constraints to `where`.

Type annotations:
- Provide only the minimum annotation required; prefer annotating a `let` binding over turbofish; treat fully-qualified syntax as a last resort.

Avoid explicit `drop`:
- Prefer scoping over calling `drop(x)` explicitly.

Exceptions:
- If intentionally discarding an `Error`, make it obvious (e.g., `.ok()` on a `Result`) rather than a generic ignore mapping.
- At the end of a `Result<()>` function, prefer `call()?; Ok(())`.

Method chaining after delimiters:
- Avoid calling methods directly on an expression that ends with `}` (and often `]`/`)`) when formatting would hide the call chain on a visually surprising new line; bind to a local first.

Boundary types (serde / external schemas):
- Define `serde`-annotated “wire” structs locally near the boundary and convert into internal/domain types.
- Avoid spreading `serde` attributes onto core/internal types unless the type truly is the wire format.

Formatting macros:
- Prefer format-string inlining (`format!("{path}/{file}")`) when supported and it improves readability.

## 7) Error and panic discipline

Error design:
- Define your crate’s “error language” early (error type + conventions) so messages stay consistent.
- Prefer typed errors for libraries; reserve type-erased errors for top-level binaries.
- Convert foreign errors into your local error type near the boundary to keep call chains readable.

Field naming conventions (consistency + chaining):
- Prefer the field name `source` for wrapped underlying errors.
- Prefer the field name `reason` for human-facing explanation strings when a typed error needs a detailed explanation.
- Avoid exposing dependency error types in a public error enum; wrap or re-home them behind an internal error type/variant.

Error messages:
- Keep them concise, consistent in tone, and written for the intended user.
- Prefer a consistent phrasing shape across the codebase.
- If an error might be wrapped, prefer starting messages lowercase.
- Avoid repeating the same context at multiple layers as an error bubbles up.

Panics:
- Don’t panic on user input.
- If a panic is truly unavoidable (bug/unreachable state), make it obvious it’s an internal fault.
- `unwrap()`/`expect()` are acceptable in tests/setup and narrow “programmer-only failure” contexts; otherwise prefer typed error handling.
- `Mutex` poisoning can justify propagating the panic rather than continuing in a potentially-invalid state.

## 8) Function discipline

Unit-return clarity:
- When a statement/expression is intended to yield no information (`()` or `!`), use semicolons deliberately.
- For `Result<()>` functions, prefer `some_call()?; Ok(())` over returning the last `Result<()>` expression implicitly.

Match-as-control-flow:
- If a `match` is used purely for side effects/control flow, prefer empty blocks (`=> {}`) over `=> ()`.

API simplicity:
- Hide generics when possible (use elision, `'_`, and `impl Trait`) to keep signatures approachable.

Lifetime visibility:
- If a type carries a lifetime parameter, don’t hide that fact (prefer `Type<'_>` over writing a lifetime-less name that looks owned).

Unused parameters in default implementations:
- If a default trait method ignores a parameter, use `let _ = param;` in the body (avoid `_param` names that leak into docs, or blanket `#[allow]`).

Builders:
- Prefer builders that consume `self` (fluent move-based builder) unless mutability is required.
- Prefer `Type::builder()` so callers don’t need to import the builder type.
- Prefer builders constructed via `Type::builder()` rather than a public `TypeBuilder::new()`.

## 9) Ordering discipline

Files should read top-to-bottom:
- Define important public types and entrypoints before helpers.
- Don’t put an `impl` before its type/trait definition.

Impl ordering:
- Prefer inherent impls first, then trait impls. Keep ordering consistent so scanning is predictable.

Derives and declarations:
- Keep derive lists stable and intentionally ordered.
- Keep declaration ordering consistent (`const`, `static`, `let`, `let mut`).

Struct fields:
- Consider ordering fields by visibility (`pub`, then `pub(crate)`, then private) unless there’s a stronger local convention.
- Don’t reorder fields just to influence derived ordering; write manual impls when ordering semantics matter.

## 10) Unsafe discipline

Lithos policy: unsafe is forbidden in production code (workspace-level lints).

If a narrow, future exception is ever considered:
- Minimize `unsafe` scopes.
- Document preconditions precisely using a `SAFETY:` comment.
- Require tests that exercise the unsafe boundary.
- Document the decision in an ADR.
- Don’t justify unsafe purely with “it should be faster”; require profiling evidence.

## 11) Structural and comment discipline

Module roots:
- Canonical prefers `mod.rs` as the module root for multi-file modules and treats module roots as “manifest-like” (module declarations + re-exports, minimal glue).

Lithos divergence (intentional):
- Lithos generally prefers Rust 2018+ “file + folder” layout (a `foo.rs` module root alongside a `foo/` directory) as documented in [docs/refs/rust/module-system.md](module-system.md).

How to reconcile (recommended):
- Treat whatever is the module root (`mod.rs` or `foo.rs`) as manifest-like: declarations + re-exports, minimal glue.
- Keep implementations in submodules (`foo/…`) so the root remains scannable.
- If you do have a `mod.rs`, keep cfg-gated blocks grouped at the end.

`Error` and `Result` placement (findability):
- Canonical recommends defining a library crate’s `Error` and crate `Result<T>` alias near the top of `lib.rs`.
- In binaries, keep `Error` and `Result` in dedicated root files (e.g., `error.rs` and `result.rs`) for consistency.

Doc comments:
- Make the first sentence describe the “golden path” purpose clearly.
- Prefer concrete language that references parameters by name.
- Prefer definite language (“the …”) over vague indefinite language (“a …”) for parameters/values.
