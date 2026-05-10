---
name: utils-support-boundary-governance
status: accepted
date_proposed: 2026-05-10
date_decided: 2026-05-10
date_implemented: 2026-05-10
stakeholders: [Core Maintainers]
---

# ADR 015: Utils and Support Boundary Governance

## Context

`lithos-core` historically exposed `support` as a public module while also using it for internal implementation details. As UUID and hashing primitives spread across contexts, this made it unclear which surfaces were stable contracts versus internal helpers and increased long-term coupling risk.

## Decision

We will treat `utils` as the outward-facing utility contract surface and `support` as crate-private internals.

We will enforce the following governance rules:
- `support` is crate-private and not part of the external API surface.
- `utils` provides stable, reusable contract types (for example `UuidV7`).
- `utils` dependencies are allowlisted; additions must satisfy all admission criteria by default.
- Exceptions to allowlist policy require a dedicated ADR.
- `utils` public APIs avoid direct third-party type exposure by default; direct exposure requires explicit architectural justification.

## Alternatives Considered

### Alternative 1: Keep `support` publicly exposed and continue adding shared primitives there
- **Pros**: Minimal immediate migration work.
- **Cons**: Blurs contract boundaries, invites accidental external coupling to internals, and makes future refactors riskier.

### Alternative 2: Create a standalone workspace `utils` crate immediately
- **Pros**: Strongest long-term decoupling and explicit reuse boundary.
- **Cons**: Higher immediate churn and repository-shape decisions were intentionally deferred.

## Technical Validation

### Research Findings
- The selected boundary reflects implemented code changes: `support` is crate-private and UUID contract types now live in `utils`.
- Existing project quality gates validated the migration with format, lint, and test hooks passing on boundary-change commits.

### Benchmarks & Prototypes
- Not applicable for this ADR; this is a boundary/governance decision validated through compile/test and repository policy checks.

## Consequences

- **Positive**: Clear contract vs internals boundary; safer refactoring posture; explicit policy for dependency/API surface growth.
- **Negative**: Additional governance overhead for dependency additions and exceptions.
- **Risks**: `lithos-core`-scoped `utils` may still accrue broad responsibilities if policy discipline is not maintained.

## References

- `.scratch/utils-support-boundary/01-utils-uuidv7-contract-surface-and-cutover.md` - Related implementation slice.
- `.scratch/utils-support-boundary/02-enforce-support-crate-private-internals.md` - Related implementation slice.
