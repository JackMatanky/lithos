# Domain docs

This repository uses a multi-context domain-doc layout.

## Layout contract

- Root contains `CONTEXT-MAP.md`.
- `CONTEXT-MAP.md` points to one or more per-context `CONTEXT.md` files.
- Each context may define its own ADR location (for example, `docs/adr/` or context-local ADR directories).

## Consumer rules for engineering skills

When a skill needs domain language or architecture history:

1. Read `CONTEXT-MAP.md` first.
2. Resolve the relevant context(s) for the task.
3. Read the mapped `CONTEXT.md` file(s) before making domain decisions.
4. Read ADRs for the same context as referenced by the map.
5. If context selection is ambiguous, prefer the narrowest matching context and cite assumptions in output.

## Fallback behavior

- If `CONTEXT-MAP.md` is missing or incomplete, pause and request clarification before applying cross-context assumptions.
