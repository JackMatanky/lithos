---
labels: [ready-for-agent]
---

## What to build

Restructure the `crates/schema/src/property/` module layout so all property and spec code lives under a single directory. This is a pure file-move with no behaviour change — the compiler must be green after the move with no logic altered.

- Move `crates/schema/src/property.rs` → `crates/schema/src/property/mod.rs`
- Move `crates/schema/src/property_spec/mod.rs` content → `crates/schema/src/property/spec.rs`
- Move all spec files (`bool.rs`, `date.rs`, `file.rs`, `number.rs`, `string.rs`) from `crates/schema/src/property_spec/` into `crates/schema/src/property/`
- Update all `mod` declarations and `use` paths across the codebase to reflect the new locations
- Delete the now-empty `crates/schema/src/property_spec/` directory

## Acceptance criteria

- [ ] `crates/schema/src/property_spec/` directory no longer exists
- [ ] `crates/schema/src/property/mod.rs` contains the former `property.rs` content
- [ ] `crates/schema/src/property/mod.rs` declares `mod spec;` (or `pub mod spec;`) to re-export the moved spec module
- [ ] `crates/schema/src/property/spec.rs` contains the former `property_spec/mod.rs` content
- [ ] All spec files (`bool.rs`, `date.rs`, `file.rs`, `number.rs`, `string.rs`) live under `crates/schema/src/property/`
- [ ] All references to `super::property_spec::*` or `crate::property_spec::*` across the codebase are updated to `super::spec::*` or `crate::property::spec::*`
- [ ] `crates/schema/src/lib.rs` has `pub mod property_spec;` removed (replaced by `pub mod property;` which nests spec)
- [ ] `cargo build` passes with no errors or warnings
- [ ] `cargo test` passes with no regressions

## Blocked by

None — can start immediately.
