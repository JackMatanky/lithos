---
title: 04-split-hash-rs-into-content-hash-and-hash-index
category: enhancement
label: ready-for-agent
status: draft
date_created: 2026-05-31T00:00:00+00:00
---

# Split `hash.rs` into `content_hash.rs` and `hash_index.rs`

## What to build

Mechanically split `lithos-core/src/support/hash.rs` into two files with no behavioral change:

- **`content_hash.rs`**: `Blake3Hash`, `ArchivedBlake3Hash`, `HashInput` enum, all `From` impls for `HashInput`, `hash_bytes()`, `hash_structured()` helpers, and their tests. Module doc updated to say "content hashing primitives" instead of "centralised hashing utilities."
- **`hash_index.rs`**: `Blake3HashIndex`, `ArchivedBlake3HashIndex`, diff helpers (`changed_keys`, `removed_keys`), `From`/`Default` impls, and their tests. Module doc added describing the keyed hash map for change detection.

Update `support/mod.rs` facade to `pub(crate) mod content_hash; pub(crate) mod hash_index;` and adjust the re-exports block accordingly.

Update all import paths across the workspace that currently reference `support::hash::*` to the correct new submodule (`support::content_hash::*` or `support::hash_index::*`).

## Acceptance criteria

- [ ] `hash.rs` removed; `content_hash.rs` and `hash_index.rs` exist under `support/`.
- [ ] `support/mod.rs` facade re-exports the same symbols with updated paths; no public API change.
- [ ] All callers updated: `support::hash::Blake3Hash` → `support::content_hash::Blake3Hash`, `support::hash::Blake3HashIndex` → `support::hash_index::Blake3HashIndex`, etc.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes.
- [ ] `cargo test -p lithos-core` passes with no regressions.

## Blocked by

None — can start immediately.
