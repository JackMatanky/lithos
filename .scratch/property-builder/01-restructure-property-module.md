---
labels: [ready-for-agent]
---

## What to build

Restructure the `schema/property/` module layout so all property and spec code lives under a single directory. This is a pure file-move with no behaviour change — the compiler must be green after the move with no logic altered.

- Move `schema/property.rs` → `schema/property/mod.rs`
- Move `schema/property_spec/mod.rs` content → `schema/property/spec.rs`
- Move all spec files (`bool.rs`, `date.rs`, `file.rs`, `number.rs`, `string.rs`) from `schema/property_spec/` into `schema/property/`
- Update all `mod` declarations and `use` paths across the codebase to reflect the new locations
- Delete the now-empty `schema/property_spec/` directory

## Acceptance criteria

- [ ] `schema/property_spec/` directory no longer exists
- [ ] `schema/property/mod.rs` contains the former `property.rs` content
- [ ] `schema/property/spec.rs` contains the former `property_spec/mod.rs` content
- [ ] All spec files (`bool.rs`, `date.rs`, `file.rs`, `number.rs`, `string.rs`) live under `schema/property/`
- [ ] `cargo build` passes with no errors or warnings
- [ ] `cargo test` passes with no regressions

## Blocked by

None — can start immediately.
