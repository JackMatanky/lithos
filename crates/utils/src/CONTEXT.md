# Utils

The Utils context defines stable, outward-facing utility contracts shared across `lithos-core` contexts. It exists to provide reusable primitives without exposing crate-internal implementation internals.

## Language

**Utility Contract**:
A stable, reusable type-level contract exposed for cross-context consumption.
_Avoid_: helper internals, convenience-only utility

**Allowlist Dependency**:
A dependency explicitly approved for use in `utils` because it meets all policy criteria.
_Avoid_: ad hoc dependency, default dependency

**Allowlist Exception**:
A dependency policy exception that is permitted only through an ADR-recorded decision.
_Avoid_: reviewer-only override, silent exception

**Third-Party Type Exposure**:
Direct use of external crate types in `utils` public API surfaces.
_Avoid_: implicit passthrough, accidental exposure

**Contract Wrapper**:
A project-owned type that encodes invariants while encapsulating an external type.
_Avoid_: raw external type alias, thin re-export

## Relationships

- **Utils** provides **Utility Contracts** to **Note**, **Schema**, **Template**, **Config**, and **DB**.
- **Utils** enforces **Allowlist Dependency** policy for its implementation surface.
- **Allowlist Exception** decisions for **Utils** are recorded via ADRs.
- **Contract Wrapper** usage in **Utils** reduces direct **Third-Party Type Exposure** in public APIs.

## Example dialogue

> **Dev:** "Can we expose `uuid::Uuid` directly from this new utility API?"
> **Domain expert:** "Only if we explicitly accept that lock-in; otherwise publish a **Contract Wrapper** like `UuidV7` and keep the external type behind it."

## Flagged ambiguities

- "utils" was used to mean both public contracts and private helpers — resolved: **Utils** refers only to outward-facing contracts; private helpers belong to **Support**.
