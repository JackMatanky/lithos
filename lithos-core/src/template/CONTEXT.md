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

**Template Runtime**:
The imperative execution environment exposed to the template during rendering. Provides interactive capabilities (prompts, suggesters) and data access (structured queries).
_Avoid_: minijinja env, god object

**Template Frontmatter**:
The declarative configuration block at the top of a template file defining static schema dependencies and inputs.
_Avoid_: yaml config, script metadata

**Structured Query**:
An engine-agnostic, SQL-like builder representation for data retrieval.
_Avoid_: query string, raw db query

## Invariants

- Template assets are validated before use in generation workflows.
- Rendering behavior is deterministic for the same template asset and render context.
- Template usage is constrained by configuration and schema semantics.
- Runtime schema discovery is strictly limited to child schemas of those explicitly declared in the template frontmatter.

## Interfaces
- Defines segregated `Repository` interfaces (Read, Write, and Unified) for persistence operations.

## Not Owned Here
- Note extraction logic (tasks, links, tag parsing).
- Schema graph resolution and property-rule authoring.
- Filesystem root safety policy and persistence transaction internals.

## Resources
- **[Templater](https://silentvoid13.github.io/Templater/)** provides conceptual reference material for programmable note-generation workflows.
  - Relevant concepts: template files, runtime functions, interactive prompts, user scripts
  - GitHub: <https://github.com/SilentVoid13/Templater>
  - Source Digest: `docs/refs/digests/obsidian_silentvoid13-templater-src-digest.txt`
  - Docs: `docs/refs/digests/obsidian_silentvoid13-templater-docs-digest.txt`
  - Internal Reference: `docs/refs/obsidian/templater-reference.md`
