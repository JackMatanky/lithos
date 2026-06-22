# Support

The Support context defines crate-private implementation support used inside `lithos-core`. It exists to centralize internal building blocks that are shared internally but intentionally excluded from the public contract surface.

## Language

**Support Internal**:
A crate-private helper used by multiple internal modules but not exposed as public API.
_Avoid_: public utility, shared contract

**Internal Hash Primitive**:
A crate-private hashing type or operation used for change detection, indexing, or staleness checks.
_Avoid_: exported hash API, external hashing contract

**Crate-Private Boundary**:
The visibility rule that allows use within `lithos-core` while disallowing external consumption.
_Avoid_: soft private convention, public-by-default

**Support Facade**:
Crate-private re-exports in `support/mod.rs` for internal ergonomics.
_Avoid_: public facade, external compatibility layer

## Relationships

- **Support** provides **Support Internals** to internal modules in **DB**, **Schema**, and **Config**.
- **Support** enforces the **Crate-Private Boundary** and is not imported by downstream crates.
- **Support Facade** re-exports **Internal Hash Primitives** for internal call-site ergonomics.
- Public contracts that emerge from **Support** migrate to **Utils** when designated stable and outward-facing.

## Example dialogue

> **Dev:** "Should this shared hash helper go into `utils`?"
> **Domain expert:** "Not if it is only internal infrastructure; keep it as a **Support Internal** behind the **Crate-Private Boundary**."

## Flagged ambiguities

- "support" was previously interpreted as a public shared module — resolved: **Support** now means crate-private internals only.
