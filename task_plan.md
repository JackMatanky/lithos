# Task Plan: Template Module Redesign

## Goal
Produce a rigorous research-driven redesign proposal for the `template` module in `@lithos-core/src/template/`, conforming to the new Lithos architecture (file-centric, unified repositories, Markdown-first, high-performance, lean, idiomatic Rust).

## Phases

### Phase 1: Inspect Existing Module [complete]
- Read all files in `@lithos-core/src/template/`.
- Diagnosis: Wrapping minijinja and implementing own inheritance/topological sort. Memory leaks with static strings. Leftovers of CQRS events.

### Phase 2: Research MiniJinja [complete]
- Evaluated native inheritance, source loaders, objects, state, values.

### Phase 3: Research Similar Rust Projects [complete]
- Zola/Tera, mdBook/Handlebars.

### Phase 4: Research Obsidian Templater [complete]
- Read templater-reference.md.
- Extracted `tp.*` to `li.*` mappings.

### Phase 5: Draft the Redesign Proposal [in_progress]
- Executive Diagnosis.
- Research Findings.
- Design Principles.
- Full Template Module Redesign (Module structure, Pipeline).
- Rust Types and Data Model.
- `li` Runtime API Design and Interactive Runtime Features.
- `li.vault` Deep Design.
- InputSpec Redesign.
- Caching and Stale Detection.
- Markdown-Specific Concerns.
- Error Handling and Diagnostics.
- Migration and Testing Strategy.
- Final Recommendation.

## Errors Encountered
*(None yet)*
