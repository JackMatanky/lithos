---
title: 01-hash-index-visibility-propagation
category: enhancement
label: ready-for-human
status: completed
date_created: 2026-05-08T14:28:32+03:00
date_completed: 2026-05-08T13:47:11+03:00
---

# Hash index visibility propagation

## Parent

Hash-index refactor stream.

## Related

- `.scratch/visibility-hardening/issue-001-crate-visibility-policy-and-rollout.md` (broader crate visibility policy and rollout)

## What to build

Constrain hash/index surfaces used by the hash-index refactor to crate-internal visibility. Keep `Blake3Hash` and `Blake3HashIndex` internals `pub(crate)` and propagate visibility through schema/config/view seams so no public API leaks crate-private hash internals.

## Acceptance criteria

- [x] All items in `lithos-core/src/support/hash.rs` are `pub(crate)` or more restrictive, with no public leakage of hash internals.
- [x] Any API in `lithos-core` that exposes `Blake3Hash` (or hash-index internals) is visibility-aligned (`pub(crate)` where appropriate), including trait methods and view contracts.
- [x] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes without `private_interfaces` errors related to hash visibility.
- [ ] Full `mise run verify` rerun on latest `main` for final regression safety.

## Implementation notes

- Implemented as part of hash-index refactor commits:
  - `4d2d456f` (`refactor(hash): harden crate-private visibility`)
  - `63f1cda5` (`refactor(schema): split raw hash index from view wrapper`)
- Visibility was propagated through config/schema view contracts to remove crate-private type leakage in public-facing signatures.
- Schema contracts export path was internalized where required to keep hash internals crate-scoped.

## Verification evidence

- `cargo clippy -p lithos-core --all-targets -- -D warnings` ✅
- `cargo test -p lithos-core support::hash::tests` ✅
- `cargo test -p lithos-core config::views::tests` ✅
- `cargo test -p lithos-core schema::views::hashes::tests` ✅

## Changed files

- `lithos-core/src/support/hash.rs`
- `lithos-core/src/config/processor.rs`
- `lithos-core/src/config/views.rs`
- `lithos-core/src/schema/views/hashes.rs`
- `lithos-core/src/schema/views/mod.rs`

## Follow-up validation

- [ ] Run `mise run verify` on latest `main` to close the remaining acceptance criterion.
- [ ] Spot-check rustdoc/public signatures for accidental hash-internal exposure after future refactors.

## Blocked by

None - implementation completed; awaiting final verification closeout.
