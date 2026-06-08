# Issue 06: `lithos index` CLI command

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Add the `lithos index` command to `lithos-cli`. The CLI is a thin executable
adapter: it parses user intent, constructs a typed `IndexCommand`, delegates
to `lithos-core::app::run_index`, and renders diagnostic output. No domain
logic, freshness rules, or routing semantics live in the CLI.

Command surface to implement:

- `lithos index` — run a full incremental index, print summary output.
- `lithos index --reindex` — discard all persisted state and re-index from
  scratch (`IndexOptions { reindex: true }`).
- `lithos index --path <path>` — partial scan for one file or directory
  subtree (`IndexScope::Partial`).
- `lithos index --context <schema|note|template>` — partial scan scoped to
  the configured context boundary path (`IndexScope::Partial`).
- `lithos index --dry-run` — classify without persisting
  (`IndexOptions { dry_run: true }`).
- `lithos index --format <human|json>` — choose diagnostic output format.
- `lithos index status` — report persisted index summary (node counts by
  kind/format, last indexed timestamp) without scanning.
- `lithos index explain <path>` — show how a path resolves to a node and its
  classification inputs.

Diagnostic output for `lithos index` must report: scanned, new, fresh, stale,
deleted, and failed node counts. Errors map to actionable messages (invalid
path, permission denied, scan failure) — not internal traces.

## Acceptance criteria

- [ ] `lithos index` runs end-to-end against a real vault directory and prints
      a summary with correct counts.
- [ ] `--reindex` produces an `IndexOptions { reindex: true }` and all nodes
      are classified `New`.
- [ ] `--path` and `--context` produce an `IndexScope::Partial` with the
      correct root.
- [ ] `--dry-run` classifies nodes without modifying the persisted index (no
      redb writes).
- [ ] `--format json` outputs machine-readable JSON; `--format human` (default)
      outputs a human-readable summary.
- [ ] CLI mapping tests prove each flag maps to the expected `IndexScope`
      and/or `IndexOptions` without duplicating domain rules.
- [ ] CLI diagnostic tests prove human and JSON output report summary counts
      and actionable errors.
- [ ] `lithos index status` reports stored counts without scanning.
- [ ] `lithos index explain <path>` resolves the path and reports
      classification inputs.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- issue-05-app-wiring.md
