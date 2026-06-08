---
name: app-composition-root
status: accepted
date_proposed: 2026-06-08
date_decided: 2026-06-08
date_implemented: 2026-06-08
stakeholders: [lithos-core maintainers, lithos-cli maintainers]
---

# ADR 021: `lithos-core::app` as the composition root

## Context

The Filesystem Indexer PRD introduces the first multi-step execution flow in
`lithos-core`: Discovery → Config → Indexer → routing. Wiring these steps
together requires a composition root — a place where concrete adapters are
constructed, ports are injected, and execution flows are exposed to executable
adapters.

Three candidate locations were considered: inside `lithos-cli` (the executable
adapter), inside a new `main.rs` added to `lithos-core`, or inside a dedicated
`lithos-core::app` module.

The question matters because the composition root is hard to move once
downstream adapters (CLI, future TUI, future daemon) start calling into it.

## Decision

We will introduce `lithos-core::app` as the canonical composition root for
`lithos-core`.

- `lithos-core` remains a library crate with no `main.rs`.
- `lithos-core::app` composes core ports and adapters and exposes typed
  execution flows (e.g. `run_index`) to executable adapters.
- `lithos-cli` remains a thin executable adapter: it parses user intent,
  constructs typed command values (e.g. `IndexCommand`), delegates to
  `lithos-core::app`, and renders diagnostic output.
- No business logic lives in `lithos-core::app`; it only wires components
  that own that logic.

The planned internal structure of `app`:

- `commands`: typed app-level command structs.
- `flows` (or `services`): execution flows that orchestrate the pipeline.
- `composition`: construction of concrete adapters from runtime resources.
- `diagnostics`: app-level result summaries for executable adapters to render.

## Alternatives Considered

### Alternative 1: CLI-owned composition

`lithos-cli` constructs all adapters and wires the execution flow directly.

- **Pros**: No extra module; simple for the first use case.
- **Cons**: Every future executable adapter (TUI, daemon, WASM host) must
  duplicate adapter construction and flow orchestration. Business-rule changes
  (e.g. routing logic) would require updating each adapter independently.
  Violates the principle that CLI is a thin adapter with no domain knowledge.

### Alternative 2: `main.rs` in `lithos-core`

Add `[[bin]]` to `lithos-core` and embed composition logic in `main.rs`.

- **Pros**: Slightly fewer crate boundaries.
- **Cons**: Turns `lithos-core` into a mixed library-binary crate, which
  conflicts with its role as the shared domain library. Compilation behaviour
  changes (library and binary targets compete). The deprecated
  `lithos-core::application` module was heading in this direction and was
  explicitly rejected.

## Technical Validation

### Research Findings

- The hexagonal architecture guide (`docs/refs/rust/howtocodeit-hexagonal`)
  calls `main` a bootstrapping-only location. For library crates, the
  equivalent is a composition module that wires ports before handing control
  to the caller.
- Every other context in `lithos-core` already follows the pattern of
  domain-owned ports with adapter injection. `lithos-core::app` extends that
  pattern to the top-level execution flow.
- The deprecated `lithos-core::application` module is proof that an unguided
  composition root grows complex quickly. `lithos-core::app` is intentionally
  scoped to typed commands, flows, composition, and diagnostics only.

### Benchmarks & Prototypes

- `lithos-core/src/app/mod.rs` placeholder added in commit `2f644fdc`.
  Compiles cleanly, passes all existing tests with no changes required.

## Consequences

- **Positive**: A single composition root means adapter construction and
  flow wiring are defined once. Future executable adapters call
  `lithos-core::app` without duplicating pipeline knowledge.
- **Positive**: `lithos-cli` can remain a thin adapter focused on CLI
  ergonomics and diagnostic rendering, with no knowledge of port wiring.
- **Positive**: The `app` module boundary is a natural place to add
  integration tests for full pipeline flows without a real CLI invocation.
- **Negative**: `lithos-core` now has a module (`app`) that is not a domain
  context. Reviewers unfamiliar with this ADR may be surprised to find
  composition logic in a library crate.
- **Risks**: If `app` grows beyond its intended scope (commands, flows,
  composition, diagnostics), it risks becoming the new `application` module.
  The module-level doc and this ADR serve as the guardrail; scope creep should
  be caught in PR review.

## References

- [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust) — composition root pattern
- `lithos-core::application` (deprecated) — prior art and cautionary example
- `.scratch/filesystem-indexer/PRD.md` — Indexer PRD that triggered this decision
- ADR 016 — segregated repository traits (port ownership pattern this extends)
- ADR 018 — explicit redb adapter seam (adapter injection precedent)
