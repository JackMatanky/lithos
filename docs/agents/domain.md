# Domain Docs

This repository uses a **single-context** domain documentation layout.

## Layout

- Domain context: `CONTEXT.md` at repository root (when present).
- Architectural decisions: `docs/adr/`.

## Consumer rules for skills

- Skills that need project language and boundaries should read `CONTEXT.md` first (if present).
- Skills that need architecture history should read relevant ADRs in `docs/adr/`.
- If `CONTEXT.md` is missing, use `AGENTS.md` plus ADRs as the best available domain context.
- Treat ADR decisions as constraints unless explicitly superseded by a newer ADR.
