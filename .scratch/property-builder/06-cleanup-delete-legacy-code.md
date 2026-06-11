---
labels: [ready-for-agent]
---

## What to build

Remove all legacy code that has been superseded by the new builders and differ. This is a pure deletion pass — no new logic, no behaviour change.

- Delete `schema/expander.rs` entirely (logic absorbed into `PropertyBuilder`)
- Delete the four `TryFrom<RawPropertyMap>` implementations in `schema/property/mod.rs` (`TryFrom<RawPropertyMap<RawPropertyInline>>`, `TryFrom<RawPropertyMap<RawPropertyBankEntry>>`, `TryFrom<HashMap<PropertyName, RawPropertyInline>>`, `TryFrom<HashMap<PropertyName, RawPropertyBankEntry>>`)
- Remove any now-dead imports and dead code warnings that surface after the deletions
- If `PropertyDeltaEngine` in `delta.rs` has been fully replaced, remove the engine struct and its `diff_schema` / `diff_property_bank` methods (keep `ExcludesDelta`, `ExtendsDelta`, `PropertyDelta` which are unrelated)

## Acceptance criteria

- [ ] `schema/expander.rs` no longer exists
- [ ] The four `TryFrom<RawPropertyMap>` implementations are gone from `schema/property/mod.rs`
- [ ] `PropertyDeltaEngine` struct and its `diff_schema` / `diff_property_bank` methods are removed from `delta.rs`
- [ ] No dead-code or unused-import warnings remain
- [ ] `cargo build` passes with no errors or warnings
- [ ] `cargo test` passes with no regressions

## Blocked by

- `04-wire-base-schema-processor.md`
- `05-wire-property-bank-processor.md`
