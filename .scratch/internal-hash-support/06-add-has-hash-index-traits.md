---
title: 06-add-has-hash-index-traits
category: enhancement
label: ready-for-human
status: draft
date_created: 2026-05-31T00:00:00+00:00
---

# Add `HasHashIndex` / `HasHashIndexMut` traits

## What to build

Define `HasHashIndex` and `HasHashIndexMut` traits in `hash_index.rs` that capture the keyed-hash-index access pattern currently used by wrapper types.

`HasHashIndex` should expose generic read access. Exact shape needs design — candidates include:
- `fn hash_index(&self) -> &Blake3HashIndex<K>`
- Key-level access: `fn entry_hash(&self, key: &K) -> Option<&Blake3Hash>`

`HasHashIndexMut` should extend `HasHashIndex` with:
- `fn hash_index_mut(&mut self) -> &mut Blake3HashIndex<K>`

Then evaluate which existing types should implement these traits. Candidates:

- `Blake3HashIndex<K>` itself (trivially)
- `HashRecord` (exposes property hashes)
- `ConfigFieldHashes` (wraps `Blake3HashIndex<ConfigField>`)
- `RawPropertyHashIndex` (wraps `Blake3HashIndex<PropertyName>`)

The key question: should the trait be generic over `K`, or should each implementor define its own key type? This must be resolved during the HITL review.

## Acceptance criteria

- [ ] `HasHashIndex` trait defined in `hash_index.rs` with read access to the indexed hashes.
- [ ] `HasHashIndexMut` trait defined in `hash_index.rs` extending `HasHashIndex` with mutable access.
- [ ] Generic parameter approach resolved (trait generic vs. associated type for key).
- [ ] Wrapper types that benefit implement the traits.
- [ ] No regressions in existing hash-index call sites.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes.
- [ ] `cargo test -p lithos-core` passes.

## Blocked by

- `.scratch/internal-hash-support/04-split-hash-rs-into-content-hash-and-hash-index.md`
