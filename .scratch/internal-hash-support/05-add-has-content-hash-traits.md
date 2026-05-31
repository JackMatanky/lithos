---
title: 05-add-has-content-hash-traits
category: enhancement
label: ready-for-human
status: draft
date_created: 2026-05-31T00:00:00+00:00
---

# Add `HasContentHash` / `HasContentHashMut` traits

## What to build

Define `HasContentHash` and `HasContentHashMut` traits in `content_hash.rs` that capture the content-hash access pattern currently duplicated across the codebase.

`HasContentHash` should expose:
- `fn content_hash(&self) -> &Blake3Hash`
- Provided default: `fn is_content_match(&self, hash: &Blake3Hash) -> bool { self.content_hash().is_match(hash) }`

`HasContentHashMut` should extend `HasContentHash` with:
- `fn set_content_hash(&mut self, hash: Blake3Hash)`

Then evaluate which existing types should implement these traits. Candidates include:

- `Blake3Hash` (trivially)
- `HashRecord` (already has `content()` getter and `is_content_match()`)
- Version/snapshot types that implement `VersionRead` / `RawViewRead` (schema views)
- `ConfigVersionView` and related config view types

This requires a design decision: which types should implement via the trait vs. keep their own methods? The goal is to reduce duplicated `is_content_match` definitions, not to force every hash-having type into a trait.

## Acceptance criteria

- [ ] `HasContentHash` trait defined in `content_hash.rs` with `content_hash()` and default `is_content_match()`.
- [ ] `HasContentHashMut` trait defined in `content_hash.rs` extending `HasContentHash` with `set_content_hash()`.
- [ ] Types that benefit from the trait implement it (e.g., `Blake3Hash`, `HashRecord`). Types where the trait doesn't fit keep existing methods.
- [ ] No regressions in existing `is_content_match` call sites.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes.
- [ ] `cargo test -p lithos-core` passes.

## Blocked by

- `.scratch/internal-hash-support/04-split-hash-rs-into-content-hash-and-hash-index.md`
