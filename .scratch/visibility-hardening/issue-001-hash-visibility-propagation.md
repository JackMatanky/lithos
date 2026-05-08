# Visibility Hardening: Propagate `pub(crate)` for hash surface

- Label: `needs-triage`
- Type: `AFK`
- Category: `enhancement`
- State: `needs-triage`

## Parent

General visibility hardening refactor for the codebase.

## What to build

Constrain the hash surface to crate-internal visibility by keeping all components in `lithos-core/src/support/hash.rs` at `pub(crate)` and propagating that visibility through all APIs that currently expose `Blake3Hash` in public signatures. Ensure schema/config/view contracts remain coherent after visibility tightening, with no `private_interfaces` or dead-code regressions in strict lint mode.

## Acceptance criteria

- [ ] All items in `lithos-core/src/support/hash.rs` are `pub(crate)` or more restrictive, with no public leakage of hash internals.
- [ ] Any API in `lithos-core` that exposes `Blake3Hash` (or hash-index internals) is visibility-aligned (`pub(crate)` where appropriate), including trait methods and view contracts.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes without `private_interfaces` errors related to hash visibility.
- [ ] Targeted schema/config/hash tests pass after propagation, plus full `mise run verify` for regression safety.

## Blocked by

None - can start immediately.

## Progress log

### 2026-05-08

- Ran strict lint gate: `cargo clippy -p lithos-core --all-targets -- -D warnings`.
- Propagated `pub(crate)` through hash-exposing APIs in config/schema views:
  - `config::processor::ConfigFieldHashes::{insert,get,iter}`.
  - `config::views::RawFileVersion::{content_hash,is_content_match,matches}`.
  - `schema::views::hashes::{HashRecord,RawPropertyMapHash}` hash-facing methods.
  - Internalized schema contracts export path: `schema::views::contracts` traits remain crate-internal and no longer publicly re-exported from `schema::views::mod`.
- Cleaned hash utility lint fallout while preserving crate-internal visibility:
  - Removed unfulfilled lint expectation on `Blake3Hash` type.
  - Merged duplicate match arms in `Blake3Hash::compute`.
  - Scoped test-only helpers (`Blake3Hash::new`, `hash_bytes`) and removed unused `hash_text`.
  - Added targeted `#[expect(dead_code, reason = ...)]` where API is intentionally staged and crate-internal.
- Verification status:
  - `cargo clippy -p lithos-core --all-targets -- -D warnings` ✅
  - `cargo test -p lithos-core support::hash::tests` ✅
  - `cargo test -p lithos-core config::views::tests` ✅
  - `cargo test -p lithos-core schema::views::hashes::tests` ✅

## Acceptance criteria status

- [x] All items in `lithos-core/src/support/hash.rs` are `pub(crate)` or more restrictive, with no public leakage of hash internals.
- [x] Any API in `lithos-core` that exposes `Blake3Hash` (or hash-index internals) is visibility-aligned (`pub(crate)` where appropriate), including trait methods and view contracts.
- [x] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes without `private_interfaces` errors related to hash visibility.
- [ ] Targeted schema/config/hash tests pass after propagation, plus full `mise run verify` for regression safety.
