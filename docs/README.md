# Docs Home

This is the canonical documentation entrypoint for the Lithos repository.

## Target docs structure (approved)

- `docs/adr/` - immutable architectural decision records
- `docs/architecture/` - current architecture narratives and boundaries (non-ADR)
- `docs/engineering/` - active engineering workflows and operations
  - `docs/engineering/testing/`
  - `docs/engineering/ci/`
  - `docs/engineering/tooling/`
  - `docs/engineering/runbooks/`
- `docs/refs/` - external/reference material only
- `docs/specs/` - forward-looking design specs for upcoming work
- `docs/agents/` - agent and automation configuration
- `docs/history/` - strict historical provenance only
- `docs/legacy/` - superseded technical docs split by context

## Governance rules (anti-bloat)

- Each folder has one bounded purpose and explicit inclusion criteria.
- `docs/history/` is strict historical-only; no active guidance.
- `docs/history/` allows only `README.md` at root; content must live in subfolders.
- `docs/legacy/` stores superseded technical docs, not current source-of-truth.
- Distill relevant content into active docs before moving files to `history` or `legacy`.
- Every moved historical/legacy file should include a backlink to distilled active paths.
- `docs/refs/` contains external references only, not internal policy/process docs.

## Authoritative sources

1. `CONTEXT-MAP.md` at repo root and module-local `CONTEXT.md` files
2. `docs/adr/` for architectural decisions
3. `AGENTS.md` for agent operating rules
4. `docs/agents/` for skill configuration

## Navigation

- Context map: `../CONTEXT-MAP.md`
- Documentation index: `./index.md`
- Documentation matrix: `./documentation-matrix.md`
- Active testing docs: `./engineering/testing/README.md`
