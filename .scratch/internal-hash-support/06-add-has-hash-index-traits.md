---
title: 06-add-has-hash-index-traits
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-31T00:00:00+00:00
date_approved: 2026-05-31T00:00:00+00:00
---

# Add `HasHashIndex` / `HasHashIndexMut` traits

## What to build

Define `HasHashIndex` and `HasHashIndexMut` traits in `hash_index.rs` that capture the keyed-hash-index access pattern currently used by wrapper types.

`HasHashIndex` should expose:
- Associated type `Key` (each implementor binds to one key type)
- `fn hash_index(&self) -> &Blake3HashIndex<Self::Key>`

`HasHashIndexMut` should extend `HasHashIndex` with:
- `fn hash_index_mut(&mut self) -> &mut Blake3HashIndex<Self::Key>`

Implement these traits on the following types:

- `Blake3HashIndex<K>` — trivially: `hash_index()` returns `&self`, `Key = K`
- `HashRecord` — delegates through `properties()` (read-only; no `HasHashIndexMut`)
- `ConfigFieldHashes` — delegates to `inner` field (read + mut)
- `RawPropertyHashIndex` — delegates to inner `Blake3HashIndex<PropertyName>` (read + mut)

Types explicitly NOT implementing the traits:
- `ArchivedHashRecord` / `ArchivedBlake3HashIndex` — rkyv archived types; `is_match_by` stays as-is
- `PropertiesView` — composes `RawPropertyHashIndex`; not a direct consumer

## Design decisions (approved HITL)

| # | Question | Decision |
|---|----------|----------|
| 1 | Generic over K vs. associated type for key? | **Associated type** — each implementor binds to one key type; mirrors `HasContentHash` pattern |
| 2 | Method shape: whole-index vs. key-level access? | **Whole-index** — `hash_index()` returning `&Blake3HashIndex<Key>` is the correct analog to `HasContentHash::content_hash()` |
| 3 | Implement on `Blake3HashIndex<K>` itself? | **Yes** — enables generic functions accepting `impl HasHashIndex`; follows `HasContentHash` on `Blake3Hash` |
| 4 | Implement on `HashRecord`? | **Yes** — read-only (`HasHashIndex` only; no `Mut`); delegates via `properties()` |
| 5 | Implement on `ConfigFieldHashes`? | **Yes** — `HasHashIndex` + `HasHashIndexMut` (has `insert()`) |
| 6 | Implement on `RawPropertyHashIndex`? | **Yes** — `HasHashIndex` + `HasHashIndexMut` (has `insert()`) |
| 7 | Implement on `PropertiesView`? | **No** — composes `RawPropertyHashIndex`; not a direct hash index consumer |
| 8 | Archived type implementations? | **No** — follows issue 05 precedent |

## Acceptance criteria

- [ ] `HasHashIndex` trait defined in `hash_index.rs` with associated `Key` type and `hash_index(&self)` method.
- [ ] `HasHashIndexMut` trait defined in `hash_index.rs` extending `HasHashIndex` with `hash_index_mut(&mut self)`.
- [ ] `Blake3HashIndex<K>` implements both traits.
- [ ] `RawPropertyHashIndex` implements both traits.
- [ ] `ConfigFieldHashes` implements both traits.
- [ ] `HashRecord` implements `HasHashIndex` (read-only).
- [ ] `support/mod.rs` facade re-exports updated if needed.
- [ ] No regressions in existing hash-index call sites.
- [ ] Tests for each trait implementation follow `has_content_hash` / `has_content_hash_mut` submodule pattern.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes.
- [ ] `cargo test -p lithos-core` passes.

## Implementation notes (2026-05-31)

- [x] `HasHashIndex` added in `support/hash_index.rs` with associated `Key` and `hash_index(&self)`.
- [x] `HasHashIndexMut` added in `support/hash_index.rs` extending `HasHashIndex` with `hash_index_mut(&mut self)`.
- [x] `Blake3HashIndex<K>` implements both traits.
- [x] `RawPropertyHashIndex` implements both traits.
- [x] `ConfigFieldHashes` implements both traits.
- [x] `HashRecord` implements `HasHashIndex` only (read-only), per approved scope.
- [x] `support/mod.rs` re-exports updated: `HasHashIndex`, `HasHashIndexMut`.
- [x] Tests added in `has_hash_index` / `has_hash_index_mut` submodules across touched modules.
- [x] Validation passed: `cargo test -p lithos-core`, `cargo clippy -p lithos-core --all-targets -- -D warnings`, `cargo fmt`.

### Lint nuance: dead_code annotations

- Trait-level `#[allow(dead_code)]` on `HasHashIndex` and `HasHashIndexMut` are currently required for strict clippy in `lib` builds.
- Reason: non-test code currently defines impls but does not yet consume the traits via trait bounds/dynamic dispatch/generic trait-based call sites.
- Impl-level `#[allow(dead_code)]` on `impl HasHashIndex for Blake3HashIndex<K>` and `impl HasHashIndexMut for Blake3HashIndex<K>` is not required.

## Blocked by

- `.scratch/internal-hash-support/04-split-hash-rs-into-content-hash-and-hash-index.md`

## TDD plan

Following `docs/engineering/testing/unit.md` and `unit-naming.md` — Structure A with submodules, verb-first naming.

| Phase | File | Tracer bullets |
|-------|------|----------------|
| 1 | `support/hash_index.rs` | 5 RED→GREEN cycles: define `HasHashIndex`, test it, define `HasHashIndexMut`, test it, diff tests verify no breakage |
| 2 | `schema/views/hashes.rs` | 2 RED→GREEN cycles: implement `HasHashIndex` + `HasHashIndexMut` on `RawPropertyHashIndex`; implement `HasHashIndex` on `HashRecord` |
| 3 | `config/processor.rs` | 2 RED→GREEN cycles: implement `HasHashIndex` + `HasHashIndexMut` on `ConfigFieldHashes` |
| 4 | Quality gate | `clippy -D warnings`, `cargo test -p lithos-core`, `cargo fmt` |

## Test structure

```rust
// In hash_index.rs:
#[cfg(test)]
mod tests {
    mod has_hash_index {
        #[test]
        fn returns_self_for_blake3_hash_index() {}
        #[test]
        fn provides_read_access_to_indexed_hashes() {}
    }
    mod has_hash_index_mut {
        #[test]
        fn returns_mut_self_for_blake3_hash_index() {}
        #[test]
        fn provides_write_access_to_indexed_hashes() {}
    }
}
```

## Agent Brief

**Category:** enhancement
**Summary:** Define `HasHashIndex`/`HasHashIndexMut` traits in `hash_index.rs` and implement on wrapper types

**Current behavior:**
Wrapper types (`RawPropertyHashIndex`, `ConfigFieldHashes`) each independently delegate methods to an inner `Blake3HashIndex`. `HashRecord` accesses property hashes only through `properties()`. No shared trait captures the "has a typed hash index" pattern.

**Desired behavior:**
`HasHashIndex`/`HasHashIndexMut` traits in `hash_index.rs` capture the access pattern. Implementors include `Blake3HashIndex<K>` (trivially), `RawPropertyHashIndex`, `ConfigFieldHashes`, and `HashRecord` (read-only). The key type is an associated type, matching the single-key-per-type constraint.

**Key interfaces:**
- `HasHashIndex` trait (associated type `Key`): `fn hash_index(&self) -> &Blake3HashIndex<Self::Key>`
- `HasHashIndexMut` trait extends `HasHashIndex`: `fn hash_index_mut(&mut self) -> &mut Blake3HashIndex<Self::Key>`
- `Blake3HashIndex<K>` impl: `Key = K`, `hash_index()` returns `&self`
- `RawPropertyHashIndex` impl: `Key = PropertyName`, `hash_index()` returns `&self.0`
- `ConfigFieldHashes` impl: `Key = ConfigField`, `hash_index()` returns `&self.inner`
- `HashRecord` impl: `Key = PropertyName`, `hash_index()` returns `self.properties().hash_index()`

**Acceptance criteria:**
- [ ] `HasHashIndex` trait defined with associated `Key` type and `hash_index(&self)` method
- [ ] `HasHashIndexMut` trait defined extending `HasHashIndex` with `hash_index_mut(&mut self)`
- [ ] `Blake3HashIndex<K>` implements both traits
- [ ] `RawPropertyHashIndex` implements both traits
- [ ] `ConfigFieldHashes` implements both traits
- [ ] `HashRecord` implements `HasHashIndex` (read-only, no mut)
- [ ] `support/mod.rs` re-exports updated if needed
- [ ] Tests for each implementation follow the existing submodule pattern
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes
- [ ] `cargo test -p lithos-core` passes with no regressions

**Out of scope:**
- Archived type implementations (follows issue 05 precedent)
- `PropertiesView` implementation
- Renaming or removing existing wrapper methods
- Changing the `Blake3HashIndex` data structure or hashing algorithm
