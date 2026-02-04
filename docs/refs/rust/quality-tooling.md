# Rust Quality & Tooling Reference

**Primary source (paraphrased):** https://github.com/apollographql/rust-best-practices

This reference captures **tooling-facing practices** that don’t fit cleanly in:
- [docs/refs/rust/style.md](style.md) (code style + organization), or
- [docs/refs/rust/idioms.md](idioms.md) (Rust Design Patterns “Idioms”).

If there’s a conflict, prefer the more conservative / correctness-preserving rule.

Lithos note: this repo standardizes on `mise` as the entry point for quality gates.

- Prefer `mise run fmt`, `mise run lint`, `mise run test`, and `mise run verify` for day-to-day work.
- Use raw `cargo ...` commands only for one-off exploration, and keep flags aligned with what the repo expects.

## 1) Linting discipline (Clippy)

### Treat warnings as failures

- Prefer running clippy with “warnings are errors” in day-to-day work.
- In CI, prefer `cargo clippy --all-targets --all-features -- -D warnings`.

In Lithos, the preferred entry point is:

- `mise run lint`

If you need the raw clippy command (e.g., debugging a single crate), use a consistent baseline:

- `cargo clippy --all-targets --all-features --locked -- -D warnings`
  - `--locked` helps ensure CI and local match `Cargo.lock`.
  - If you’re exploring stricter lints, try `-- -W clippy::pedantic` and `-- -W clippy::nursery` locally first.

### Fix warnings; don’t silence them

- Prefer refactoring the code so the lint no longer triggers.
- If the lint is a false positive or a deliberate tradeoff, prefer a **local** override.
  - Avoid global `#[allow(...)]` at module/crate scope unless it’s a deliberate, well-documented policy.

In this repo (and in the Apollo guidance), the preferred pattern is:

- Use `#[expect(clippy::lint_name, reason = "...")]` rather than `#[allow(...)]`.
  - The “expect” form warns you if the lint stops triggering, which helps prevent dead, cargo-cult suppressions.

### High-signal lint themes (what they usually mean)

These are the categories that routinely surface real issues:

- **Redundant clones / copies**: likely an ownership/API shape issue.
- **Needless allocations** (e.g., collecting iterators early): likely an avoidable intermediate collection.
- **Large enum variants / large types by value**: likely a layout or API decision (box a variant, pass by reference).
- **`unwrap` / `expect` usage**: needs a real error path (or a test-only justification).

### Specific lints worth learning (high signal)

These tend to correlate strongly with “real” issues:

- `clippy::redundant_clone`, `clippy::clone_on_copy`: ownership/API mismatch or accidental extra work.
- `clippy::needless_collect`: an avoidable intermediate allocation.
- `clippy::large_enum_variant`: consider boxing the large variant or restructuring.
- `clippy::manual_ok_or`, `clippy::map_unwrap_or`: use standard combinators (`ok_or_else`, `_or_else`) to simplify and avoid eager allocation.
- `clippy::unnecessary_wraps`: a `Result`/`Option` return type that isn’t buying you anything.

Note: If you centralize lint policy in a workspace, keep it visible and version-controlled.

In Lithos, workspace lint policy lives in the root `Cargo.toml` under `[workspace.lints.*]`.

## 2) Testing discipline

### 2.1 Tests should read like documentation

- Name tests so the output reads like a sentence (behavior + condition).
- Prefer organizing tests with modules:
  - `mod parse { ... }` then `fn rejects_empty_input()` etc.

### 2.2 One behavior per test (and usually one assertion)

- Keep each test focused on a single behavior.
- Prefer one assertion per test when feasible (it makes failures obvious).
- If you need multiple checks, consider splitting into multiple tests with shared setup.

### 2.3 Make failures explain themselves

- Use assertion messages with useful context (actual value, parsed error, etc.).
- For error shapes, `assert!(matches!(err, MyError::Variant(_)))` is often clearer than string matching.

If an “Ok” assertion fails, prefer printing the error you would have gotten:

- `assert!(value.is_ok(), "expected Ok, got: {:?}", value.unwrap_err())`

### 2.4 Unit vs integration vs doc tests

- **Unit tests**: colocated with code (`#[cfg(test)]`), can test private helpers.
- **Integration tests**: under `lithos-core/tests/` (when added), exercise the public surface.
- **Doc tests**: examples in `///` docs.

Important detail:
- `nextest` does not execute doc tests.
- If you rely on `nextest`, add a separate `cargo test --doc` step so docs stay correct.

Doc-test attributes worth knowing (use deliberately):

- `no_run`: compiles the example but doesn’t run it (useful for examples with side effects).
- `should_panic`: asserts the example panics.
- `compile_fail`: asserts the example does not compile (useful for misuse examples).
- `ignore`: avoid unless you have no alternative; prefer `no_run` when possible.

Ergonomics (optional, but high leverage):

- Consider `pretty_assertions` for more readable diffs when comparing large strings/structures.

## 3) Snapshot testing (optional technique)

Snapshot testing is useful when correctness is primarily **structural** or **human-reviewed**:
- rendered templates,
- CLI output,
- serialized structures.

Guidelines (from Apollo, adapted):
- Keep snapshots **small** and reviewable.
- Don’t snapshot core business logic that should be asserted precisely.
- Redact unstable fields (timestamps, UUIDs) if you choose to snapshot.

Note: If you adopt a snapshot crate (e.g., `insta`), do it deliberately—snapshots become part of your review surface.

If you do adopt `insta`:

- Prefer YAML snapshots for structured data (review-friendly diffs).
- Prefer named snapshots (stable filenames/paths).
- Use redactions for unstable fields (timestamps, UUIDs, random IDs).
- Avoid snapshotting huge objects; snapshot a focused sub-structure instead.

## 4) Performance workflow (measure first)

### Don’t guess; measure

- Prefer benchmarking/profiling before changing code for performance.
- Validate improvements with repeated runs and realistic inputs.

### Benchmarking and profiling

- Prefer `cargo bench` (and criterion) for micro-benchmarks.
- Prefer a profiler (e.g., flamegraphs) when CPU time is unclear.

Lithos note:

- Prefer `mise run test:bench` for benchmarks (Criterion is already part of the workspace).

Practical reminders:

- Measure and profile in `--release`; debug builds mislead.
- If you’re investigating performance regressions, consider running `cargo clippy -- -D clippy::perf`.
- On macOS, `samply` is often a smoother profiling workflow than flamegraphs.

### Common, low-risk performance hygiene

- Avoid obviously redundant cloning; prefer borrowing and passing views.
- Avoid intermediate collections unless you actually need materialization.
- Be mindful of stack size when working with large arrays/structs.

Iterator + logging note:

- If you need side-effect logging in an iterator chain, prefer `inspect` / `inspect_err` rather than switching to a loop purely for logging.

Sizing notes:

- Avoid passing very large types by value; prefer `&T` / `&mut T`.
- Be careful with large fixed-size arrays: they live on the stack unless you explicitly choose a heap-backed representation.

Inlining note:

- Avoid adding `#[inline]` / `#[inline(always)]` unless a benchmark shows it helps; Rust is already good at inlining without hints.

When an API needs “owned or borrowed” inputs, consider `std::borrow::Cow` to avoid cloning in the common borrowed case.

## 5) Errors: library vs binary ergonomics

Apollo’s guidance is:

- Prefer structured errors in libraries/crates (typed errors, `thiserror`).
- Reserve “stringly” error aggregation (e.g., `anyhow`) for binaries / CLI entrypoints.
- Use `?` for propagation; use `inspect_err`/`map_err` when you need logging or translation.

Error-shaping reminders:

- Prefer layered errors with `#[from]` for composition and `#[error(transparent)]` when you’re intentionally re-wrapping an upstream error.
- Prefer `inspect_err` when you need diagnostics but don’t want to change the error type.

Additional guidance:

- Avoid `unwrap()` / `expect()` in production code. Prefer `?`, `ok_or_else`, `unwrap_or_else`, or `let PATTERN = expr else { ... }` for early exits.
- In async code and spawned tasks, prefer error types that are `Send + Sync + 'static` when required by the runtime.

Testing error paths:

- If your error type doesn’t implement `Eq`/`PartialEq`, assert on `err.to_string()` (or key fields) so error behavior is still exercised.
