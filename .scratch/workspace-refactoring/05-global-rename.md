---
labels: ["ready-for-agent"]
---

## Parent

PRD: `.scratch/workspace-refactoring/PRD.md`

## What to build

Execute the final, global project rename from "Lithos" to "Trace".

The crate structures and imports have already been updated to `trace-`, but many text references remain. Search the codebase for "lithos", "Lithos", and "LITHOS" and replace them with their "Trace" equivalents. This includes:
- `README.md` and `ROADMAP.md`
- `CONTEXT.md` and `CONTEXT-MAP.md` files
- Error messages, logs, tracing spans, and terminal output.
- Environment variable prefixes (e.g., if there is a `LITHOS_VAULT_PATH`, it should become `TRACE_VAULT_PATH`).
- Binary names or clap CLI descriptions.

## Acceptance criteria

- [ ] No references to "Lithos" remain in user-facing documentation or output.
- [ ] No references to "lithos" remain in environment variables or configuration keys.
- [ ] The project successfully compiles and runs under its new identity.

## Blocked by

- `.scratch/workspace-refactoring/02-migrate-cli.md`
- `.scratch/workspace-refactoring/03-consolidate-settings.md`
## Agent Brief

**Category:** enhancement
**Summary:** Perform a global text replacement to rebrand "Lithos" to "Trace" across all documents and strings.

**Current behavior:**
The project name "Lithos" appears extensively in user-facing documentation (`README.md`, `CONTEXT.md`), logs, terminal output, error messages, and environment variables.

**Desired behavior:**
All textual instances of "Lithos", "lithos", and "LITHOS" are correctly replaced with "Trace", "trace", and "TRACE", respectively. The rename is semantically correct and does not accidentally break standard Rust keywords or syntax.

**Key interfaces:**
- Markdown documentation files (`README.md`, `ROADMAP.md`, `CONTEXT.md` files).
- CLI output descriptions (e.g., `clap` documentation strings).
- Tracing spans, log output, and environment variable prefixes.

**Acceptance criteria:**
- [ ] No references to "Lithos" remain in user-facing documentation or output.
- [ ] No references to "lithos" remain in environment variables or configuration keys.
- [ ] The project successfully compiles and runs under its new identity.

**Out of scope:**
- Structural crate renaming (already handled in previous slices).
- Changing domain behavior or functionality.
