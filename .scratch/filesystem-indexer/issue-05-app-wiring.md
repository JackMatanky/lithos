# Issue 05: `lithos-core::app` wiring — `IndexCommand` and `run_index` flow

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Flesh out `lithos-core::app` (placeholder introduced in commit `2f644fdc`)
with the typed command and execution flow for the Indexer use case. The `app`
module is the composition root: it constructs concrete adapters, wires ports,
and exposes a typed execution flow to executable adapters (CLI, future TUI,
future daemon). No business logic lives here — only wiring.

Modules to introduce inside `lithos-core::app`:

- `commands`: define `IndexCommand` carrying `IndexScope` and `IndexOptions`.
- `flows` (or `services`): implement `run_index(cmd: IndexCommand, ...) ->
  IndexResult` — runs Discovery, then Config, then constructs the walkdir
  adapter and redb adapter, injects them into the Indexer service, and returns
  the result.
- `composition`: construction of concrete adapters from runtime resources
  (redb store handle, vault root path, etc.).
- `diagnostics`: app-level result summaries for executable adapters to render
  (counts, failure descriptions) — no CLI formatting here.

The `app` module must stay within its intended scope: typed commands, flows,
composition, diagnostics. It must not become a domain logic home.

## Acceptance criteria

- [ ] `IndexCommand` is defined with `IndexScope` and `IndexOptions` fields.
- [ ] `run_index` constructs the walkdir `ScannerPort`, the redb
      `Repository`, and the Indexer service, then delegates to the service.
- [ ] `run_index` runs Discovery and Config before constructing Indexer
      adapters (correct pipeline order: Discovery → Config → Indexer).
- [ ] `lithos-core::app` exposes no redb, walkdir, or adapter-specific types
      in its public surface.
- [ ] Integration test: calling `run_index` with a real (temp-dir) vault root
      produces an `IndexResult` with correct counts and no panics.
- [ ] `app` module-level documentation describes its four sub-modules and
      their growth guardrails (per ADR 021).
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- issue-04-application-service.md
