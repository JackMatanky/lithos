---
name: indexer-read-repository-cross-context-access
status: accepted
date_proposed: 2026-06-08
date_decided: 2026-06-08
date_implemented:
stakeholders: [traces-core maintainers, schema maintainers, note maintainers, template maintainers]
---

# ADR 022: Cross-context read access through `IndexerReadRepository`

## Context

The Filesystem Indexer owns two canonical redb tables — `FILES` and `DIRS` —
which hold `FileNode` and `DirNode` records for every indexed filesystem node.
Downstream context processors (Schema, Note, Template) need to resolve paths
and `FsNodeId` values to `FileNode` and `DirNode` records during ingestion.

Two approaches were considered:

1. **Port-based access**: downstream contexts take a compile-time dependency on
   the Indexer's `ReadRepository` port and call it at the application-service
   level.
2. **Shared table schema**: `FILES` / `DIRS` table definitions are promoted to
   a shared schema layer that allows read-only table accessors from any context
   adapter.

redb provides no joins, so any cross-context lookup that needs a filesystem
node must traverse `FILES` / `DIRS` regardless of approach. The question is
where the traversal logic lives and who controls the key schema.

## Decision

Downstream context processors access `FileNode` and `DirNode` records by
calling the Indexer's `ReadRepository` port, not by accessing `FILES` / `DIRS`
table definitions directly.

- The Indexer is the sole writer of `FILES` and `DIRS`.
- Other contexts receive an `IndexerReadRepository` reference at the
  application-service level (injected via `traces-core::app`).
- `FILES` / `DIRS` table definitions and key schema remain private to the
  Indexer's storage adapter.
- If a downstream context adapter needs a combined traversal that the port
  does not expose, the escape hatch is promoting `FILES` / `DIRS` definitions
  to a shared schema layer — that is an explicit opt-in, not a default.

## Alternatives Considered

### Alternative: shared table schema layer

Expose `FILES` and `DIRS` as public table constants in a shared module so
downstream adapters can build their own read-only accessors.

- **Pros**: marginally simpler for combined traversals; no port call overhead.
- **Cons**: couples every downstream adapter to the raw key schema of `FILES`
  and `DIRS`. A key layout change in the Indexer breaks every adapter that
  reads those tables directly. This violates the port-ownership rule from
  ADR 016 and removes the Indexer's ability to evolve its storage layout
  independently.

## Technical Validation

### Research Findings

- ADR 016 establishes that each context owns its repository ports. Routing
  cross-context reads through a port rather than a shared table is a direct
  application of that rule.
- ADR 018 establishes that redb table definitions stay private to each
  adapter. Exposing `FILES` / `DIRS` as shared constants would violate that
  invariant.
- The hexagonal architecture pattern (`docs/refs/rust/howtocodeit-hexagonal`)
  identifies driven ports as the correct seam for any infrastructure dependency
  that a domain service must cross. `IndexerReadRepository` is that seam.

### Benchmarks & Prototypes

- No runtime cost difference: the port call is a thin trait dispatch. redb
  table reads are identical in both approaches; only the call site differs.

## Consequences

- **Positive**: the Indexer can change its key schema, table layout, or
  serialization format without touching downstream context adapters.
- **Positive**: dependency direction is enforced at the storage level, not
  just the application level — consistent with the architectural invariant that
  `Indexer → Context processors`.
- **Positive**: downstream context services remain testable without a real
  redb database; tests inject a mock `IndexerReadRepository`.
- **Negative**: downstream context services carry a compile-time dependency on
  the Indexer's `ReadRepository` trait. This is coupling, albeit to a port
  rather than a concrete adapter.
- **Escape hatch**: if the port proves too rigid for a combined traversal need,
  promote `FILES` / `DIRS` definitions to a shared schema layer and add
  read-only table accessors. That change should be made deliberately with a
  superseding ADR, not as a shortcut.

## References

- ADR 016 — segregated repository traits (port ownership rule)
- ADR 018 — explicit redb adapter seam (adapter injection precedent)
- ADR 021 — `traces-core::app` composition root (injection site for `IndexerReadRepository`)
- `.scratch/filesystem-indexer/PRD.md` Section 10 — `FILES`/`DIRS` table ownership
