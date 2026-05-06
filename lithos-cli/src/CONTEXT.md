# CLI

The CLI context defines user-facing command semantics and orchestrates core contexts without owning domain rules.

## Language

**Command Intent**:
The user-level action requested through the CLI.
_Avoid_: raw function call, internal op

**Execution Flow**:
The ordered orchestration path from command intent to context operations.
_Avoid_: ad-hoc call chain, side-effect script

**Diagnostic Output**:
Actionable user-facing feedback for success, failure, or recovery.
_Avoid_: stack dump, internal trace text

## Invariants

- CLI remains a thin orchestration layer over core contexts.
- Command behavior maps deterministically from input to execution flow.
- Diagnostics prioritize user actionability over internal detail.

## Not Owned Here

- Domain invariants for note, schema, template, or config contexts.
- Filesystem safety policy and persistence mechanics.
- Internal data-modeling rules of core contexts.
