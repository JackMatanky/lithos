# Rename and adapt view hash index wrapper

- Label: `needs-triage`
- Type: `AFK`
- Category: `enhancement`
- State: `needs-triage`

## Blocked by

- `.scratch/hash-index-refactor/issue-001-raw-property-map-emits-hash-index.md`

## What to build

Rename `RawPropertyMapHash` to `RawPropertyHashIndex` in the schema views context, and keep it as the view-seam adapter over `Blake3HashIndex<PropertyName>`. Update all call sites and docs to use the new domain term while preserving behavior.

## Acceptance criteria

- [ ] Type renamed from `RawPropertyMapHash` to `RawPropertyHashIndex` across schema views and dependents.
- [ ] Wrapper remains a thin adapter over `Blake3HashIndex<PropertyName>` with minimal duplicated map mechanics.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` passes.
- [ ] Targeted schema view/delta/raw property tests pass.
