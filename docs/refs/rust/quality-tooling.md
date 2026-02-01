# Rust Quality & Tooling Reference

**Primary source (paraphrased):** https://github.com/apollographql/rust-best-practices

This reference captures **tooling-facing practices** that don’t fit cleanly in:
- [docs/refs/rust/style.md](style.md) (code style + organization), or
- [docs/refs/rust/idioms.md](idioms.md) (Rust Design Patterns “Idioms”).

If there’s a conflict, prefer the more conservative / correctness-preserving rule.

## 1) Linting discipline (Clippy)

### Treat warnings as failures

- Prefer running clippy with “warnings are errors” in day-to-day work.
- In CI, prefer `cargo clippy --all-targets --all-features -- -D warnings`.

### Fix warnings; don’t silence them

- Prefer refactoring the code so the lint no longer triggers.
- If the lint is a false positive or a deliberate tradeoff, prefer a **local** override.

In this repo (and in the Apollo guidance), the preferred pattern is:

- Use `#[expect(clippy::lint_name, reason = "...")]` rather than `#[allow(...)]`.
  - The “expect” form warns you if the lint stops triggering, which helps prevent dead, cargo-cult suppressions.

### High-signal lint themes (what they usually mean)

These are the categories that routinely surface real issues:

- **Redundant clones / copies**: likely an ownership/API shape issue.
- **Needless allocations** (e.g., collecting iterators early): likely an avoidable intermediate collection.
- **Large enum variants / large types by value**: likely a layout or API decision (box a variant, pass by reference).
- **`unwrap` / `expect` usage**: needs a real error path (or a test-only justification).

Note: If you centralize lint policy in a workspace, keep it visible and version-controlled.

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

### 2.4 Unit vs integration vs doc tests

- **Unit tests**: colocated with code (`#[cfg(test)]`), can test private helpers.
- **Integration tests**: under `tests/`, exercise the public surface.
- **Doc tests**: examples in `///` docs.

Important detail:
- `nextest` does not execute doc tests.
- If you rely on `nextest`, add a separate `cargo test --doc` step so docs stay correct.

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

## 4) Performance workflow (measure first)

### Don’t guess; measure

- Prefer benchmarking/profiling before changing code for performance.
- Validate improvements with repeated runs and realistic inputs.

### Benchmarking and profiling

- Prefer `cargo bench` (and criterion) for micro-benchmarks.
- Prefer a profiler (e.g., flamegraphs) when CPU time is unclear.

### Common, low-risk performance hygiene

- Avoid obviously redundant cloning; prefer borrowing and passing views.
- Avoid intermediate collections unless you actually need materialization.
- Be mindful of stack size when working with large arrays/structs.

## 5) Errors: library vs binary ergonomics

Apollo’s guidance is:

- Prefer structured errors in libraries/crates (typed errors, `thiserror`).
- Reserve “stringly” error aggregation (e.g., `anyhow`) for binaries / CLI entrypoints.
- Use `?` for propagation; use `inspect_err`/`map_err` when you need logging or translation.
