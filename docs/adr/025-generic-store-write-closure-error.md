---
name: generic-store-write-closure-error
status: accepted
date_proposed: 2026-06-28
date_decided: 2026-06-28
date_implemented: 2026-06-28
stakeholders: [traces-core maintainers, db maintainers, indexer maintainers, vault maintainers, template maintainers, schema maintainers]
---

# ADR 025: Generalize `Store::write` over the closure error type

## Context

`Store::write` (the DB context's scoped write-transaction API) commits the
transaction when the closure returns `Ok` and rolls back on `Err`. Its signature
hardwired the closure error to the DB context's own `DbError`:

```rust
pub fn write<R, F>(&self, f: F) -> Result<R, DbError>
where
    F: FnOnce(&mut WriteTx) -> Result<R, DbError>,
```

A repository adapter often needs to reject a write for a **domain** reason that
`DbError` cannot express. The indexer's duplicate-path check is the motivating
case: when a different record already owns a path, the adapter must surface
`IndexerRepositoryError::DuplicatePath`. Because the closure could only return
`DbError`, the adapter was forced to signal rejection out-of-band — it set a
captured `Option<PathKey>` and returned `Ok(())`, which **committed an empty
write transaction** to express "reject and change nothing", then translated the
captured value into the domain error after `write` returned.

Committing a transaction to express a no-op rejection is backwards, costs a
redundant fsync per rejected write, and is morally the same sentinel-`DbError`
hack that earlier review (06.5) explicitly rejected. The DB context's
`CONTEXT.md` forbids context-specific errors inside `DbError` ("callers branch on
stable Error Kind"; business semantics are "Not Owned Here"), so the fix cannot
be a new `DbError::DuplicatePath` variant — the rejection is a domain concern
that must travel in the adapter's own error type.

## Decision

We will generalize `Store::write` over the closure error type:

```rust
pub fn write<R, F, E>(&self, f: F) -> Result<R, E>
where
    F: FnOnce(&mut WriteTx) -> Result<R, E>,
    E: From<DbError>,
```

- The transaction still commits on closure `Ok` and rolls back on closure `Err`.
- Infrastructure failures (`begin_write`, `commit`) are produced as `DbError`
  and lifted into the caller's `E` via the `E: From<DbError>` bound.
- A repository adapter may now return its **own** error type directly from the
  closure (e.g. `IndexerRepositoryError`), so a domain rejection rolls back the
  transaction instead of committing a no-op and round-tripping through a
  side-channel.
- `Store::read` is intentionally left unchanged; no read path needs a
  domain-typed error today (YAGNI). The same generalization may be applied later
  under a superseding note if a read path requires it.

This stays within the DB context boundary: `DbError` gains no domain variants,
and callers that do not need a domain error keep returning `Result<R, DbError>`
(the `E = DbError` case, where `From<DbError> for DbError` is the reflexive impl,
is inferred — no call-site change).

## Alternatives Considered

### Alternative 1: add `DuplicatePath` to `DbError`

- **Pros**: no signature change; the closure keeps returning `DbError`.
- **Cons**: puts a context-specific domain concept into the shared DB error type,
  violating `db/CONTEXT.md` ("Backend-specific errors are wrapped transparently
  … callers branch on stable Error Kind"; business semantics Not Owned Here).
  Every other context would inherit an irrelevant `DuplicatePath` arm. Rejected
  by the same reasoning 06.5 used against the sentinel-`DbError` hack.

### Alternative 2: keep the `Option<PathKey>` side-channel

- **Pros**: zero DB-crate churn.
- **Cons**: commits an empty transaction to express rejection (extra fsync,
  surprising semantics), and smuggles a domain error around the port boundary in
  a mutable capture. Retained only as the documented fallback if DB churn were
  out of scope; it is not.

## Technical Validation

### Research Findings

- The change is backward-compatible by type inference: all existing
  `store.write(|tx| … Result<_, DbError>)` callers resolve `E = DbError` with the
  reflexive `From<DbError> for DbError`, so they compile unchanged.
- `impact` on `Store::write` reports 86 direct callers across `vault`,
  `template`, `schema`, and `indexer` (CRITICAL blast radius). The
  backward-compatible generalization neutralizes that radius: no caller signature
  changes. This was verified by a full `cargo test --workspace` +
  `cargo clippy --workspace -- -D warnings` run after the change.
- RBP ch.4 (model errors in the type, do not smuggle them around) and ch.6
  (generics for zero-cost flexibility) both favor the generic bound over the
  side-channel.

### Benchmarks & Prototypes

- No runtime cost: the closure is monomorphized per `E`; the `From<DbError>`
  conversion is a move/no-op for the common `E = DbError` case. The rejected-write
  path no longer performs a commit, removing one fsync per rejection.

## Consequences

- **Positive**: repository adapters express domain rejections in their own error
  type and get a true rollback; the duplicate-path no-op commit is gone.
- **Positive**: `DbError` stays free of context-specific concepts; the DB
  context boundary is preserved.
- **Positive**: the pattern is reusable by any future adapter that needs a
  domain-typed write rejection.
- **Negative**: `Store::write` now has a third type parameter; the signature is
  marginally more complex to read.
- **Risks**: a closure whose error type cannot be inferred may need an
  annotation. Existing callers are unaffected (inference picks `DbError`); only
  new domain-typed callers must name `E`.

## References

- ADR 002 — storage pattern (Store/WriteTx scoping precedent)
- ADR 005 — error handling (typed errors, no smuggling)
- ADR 018 — explicit redb adapter seam (adapter owns its error translation)
- `crates/db/CONTEXT.md` — "Error Kind" invariant; business semantics Not Owned Here
- `.scratch/filesystem-indexer/foundation/06.5-comprehensive-review.md` — finding R4
- `.scratch/filesystem-indexer/foundation/06.7-tdd-plan.md` — Slice 2.3
