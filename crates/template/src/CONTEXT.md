# Template

The Template context defines template assets and rendering constraints for note generation. It separates render-engine behavior from use-case orchestration so template rendering stays adapter-backed without leaking engine details into domain workflows.

## Language

**Template**:
A renderable template asset stored as vault-relative source text with a stable identity and derived name.
_Avoid_: script, snippet, generator file

**Template Engine**:
A rendering boundary that checks and renders an already-supplied Template using an already-supplied render context.
_Avoid_: template service, template workflow, template repository

**Template Service**:
The use-case orchestrator for finding, indexing, validating, rendering, resolving, and committing templates.
_Avoid_: template engine, MiniJinja wrapper

**Template Artifact**:
The rendered output item moving through the write pipeline from rendered content to committed vault file.
_Avoid_: generated note, output file handle

## Invariants

- Template Engine behavior is limited to engine-level source checking/loading and rendering.
- Template Service owns repository lookup, indexing, validation workflow, target resolution, conflict checks, and commit orchestration.
- Template Service may report compile health after template processing for tracing; this does not make engine compilation a Template Processor state.
- Template use cases report failures through `TemplateError`; engine failures are embedded as `TemplateEngineError`.
- MiniJinja types do not appear in Template domain models, repositories, service requests, or service responses.
- MiniJinja may be used by an adapter inside `traces-core`; the boundary is the Template domain/service public API, not the crate dependency graph.
- Foundation rendering is non-interactive and single-output.

## Not Owned Here

- Filesystem path policy and storage transaction mechanics.
- Terminal prompting and interactive input collection.
- Schema property validation semantics.

## Example Dialogue

Developer: "Should the Template Engine load templates from the repository?"

Domain expert: "No. The Template Service finds the Template, then asks the Template Engine to compile or render that supplied Template."

Developer: "Should the Template Engine choose the output path?"

Domain expert: "No. Target resolution and commit behavior belong to the Template Service and FS context."
