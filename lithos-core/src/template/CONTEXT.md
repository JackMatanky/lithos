# Template

The Template context defines reusable generation assets and rendering contracts for producing note content.

## Language

**Template Asset**:
A stored template definition that can be rendered into note content.
_Avoid_: snippet, script file

**Render Context**:
The structured input payload available during template rendering.
_Avoid_: globals, ambient state

**Rendered Output**:
The final note content produced by rendering a template asset.
_Avoid_: draft output, raw text blob

## Invariants

- Template assets are validated before use in generation workflows.
- Rendering behavior is deterministic for the same template asset and render context.
- Template usage is constrained by configuration and schema semantics.

## Not Owned Here

- Note extraction logic (tasks, links, tag parsing).
- Schema graph resolution and property-rule authoring.
- Filesystem root safety policy and persistence transaction internals.
