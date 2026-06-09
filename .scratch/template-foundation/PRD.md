# Template Foundation PRD

Status: ready-for-agent

## Problem Statement

Lithos does not yet have a Template context that can ingest renderable template assets, validate them through a configured Template Engine, render them with a minimal context, and safely commit a single rendered output into the vault. Users need a service-first foundation before richer template extensions, prompts, query helpers, or multi-file packs can be designed safely.

Without this foundation, template work risks mixing engine behavior, filesystem writes, repository persistence, and CLI orchestration into one shallow module. That would make future extension registry and interaction features hard to test and easy to couple to MiniJinja internals.

## Solution

Build the minimal Template foundation as a non-interactive, single-output vertical slice. The foundation introduces Template domain models, raw file-backed views, segregated repository traits, a Template Processor ingestion pipeline, a Lithos-shaped Template Engine port backed by MiniJinja, a Template Service for use-case orchestration, a typestate Template Artifact write pipeline, and a minimal CLI render command.

The foundation deliberately excludes Lithos custom extensions, prompts, query/runtime objects, multi-file template packs, rich conflict policies, and custom diagnostics. MiniJinja built-ins are allowed, with Lithos-specific engine configuration for owned template sources, strict undefined behavior, and no Markdown auto-escape.

## User Stories

1. As a Lithos user, I want to list available Templates, so that I can discover what note-generation assets are available.
2. As a Lithos user, I want Templates to be indexed from configured template sources, so that rendering does not depend on ad hoc file reads.
3. As a Lithos user, I want a named Template to be validated before rendering, so that syntax/source problems can be caught early.
4. As a Lithos user, I want Lithos to report whether a processed Template can compile, so that template health can be traced after ingestion.
5. As a Lithos user, I want to render a named Template with simple variables, so that I can generate Markdown from reusable source text.
6. As a Lithos user, I want repeated `--var key=value` CLI flags to provide a minimal render context, so that foundation rendering is usable without prompts or declared inputs.
7. As a Lithos user, I want rendered output to be written to a vault-relative path, so that generated content stays inside the vault boundary.
8. As a Lithos user, I want Lithos to reject absolute output paths, so that rendering cannot write outside the vault.
9. As a Lithos user, I want Lithos to reject traversal paths, so that a Template cannot escape the vault through path manipulation.
10. As a Lithos user, I want rendering to fail if the destination already exists, so that the foundation cannot overwrite notes accidentally.
11. As a Lithos user, I want successful render commits to print the created path, so that I know where the generated note was written.
12. As a Lithos user, I want structured errors when rendering fails, so that CLI failures are actionable.
13. As a developer, I want Template domain models to be validated at construction, so that invalid Template state is not persisted.
14. As a developer, I want Template identity to use `TemplateId`, so that Templates align with existing `NoteId`, `SchemaId`, and `FileId` identity patterns.
15. As a developer, I want Template paths to use `PathKey`, so that Template assets use existing vault-relative path semantics.
16. As a developer, I want `TemplateName` derived from the file path stem, so that users can refer to Templates by a stable name.
17. As a developer, I want `TemplateBody` to wrap renderable source text, so that source validation has a clear boundary.
18. As a developer, I want `RawTemplate` to remain a thin raw-content DTO, so that ingestion stages are explicit without over-modeling metadata.
19. As a developer, I want `RawTemplateView` to store content hash and file metadata, so that freshness checks can avoid unnecessary parsing and persistence.
20. As a developer, I want `RawTemplateView` to implement content-hash traits, so that it can participate in existing hash support patterns.
21. As a developer, I want Template repositories to follow segregated `ReadRepository`, `WriteRepository`, and `Repository` traits, so that persistence stays isolated behind Template-owned ports.
22. As a developer, I want batch raw-view operations, so that template discovery can compare multiple paths efficiently.
23. As a developer, I want the Template Processor to stop at `Completed`, so that engine compilation does not become an ingestion state.
24. As a developer, I want Template Engine `compile` to mean engine-level source checking/loading only, so that service-level validation does not leak into the adapter port.
25. As a developer, I want Template Engine `render` to accept an already-supplied Template and context, so that the engine does not own lookup or context assembly.
26. As a developer, I want Template Service to own lookup, validation workflow, rendering orchestration, target resolution, conflict checks, and commit orchestration, so that use-case logic stays in one place.
27. As a developer, I want Template Engine errors to preserve MiniJinja source errors, so that Rust error chains remain useful for debugging.
28. As a developer, I want Template use cases to return `TemplateError`, so that missing Templates, load failures, and engine failures share one template-level error surface.
29. As a developer, I want no custom `TemplateDiagnostic` in foundation, so that diagnostics do not become a speculative framework.
30. As a developer, I want `TemplateArtifact<State>` to enforce the write pipeline, so that content cannot be committed before target resolution and conflict checks.
31. As a developer, I want terminal write state to remain `TemplateArtifact<Committed>`, so that the typestate API stays consistent.
32. As a developer, I want future multi-file generation to use `TemplateArtifactSet<State>`, so that foundation does not overbuild for packs.
33. As a maintainer, I want MiniJinja types kept out of Template domain models, repository traits, service requests, and service responses, so that Template APIs stay Lithos-shaped.
34. As a maintainer, I want MiniJinja allowed in an adapter module inside `lithos-core`, so that dependency boundaries are based on API leakage rather than crate-level absolutism.
35. As a maintainer, I want FS reads and writes to use the FS context rather than raw `std::fs`, so that filesystem isolation remains enforced.
36. As a maintainer, I want the initial freshness seam to be small, so that a future vault filesystem indexer can take over general file freshness.
37. As a maintainer, I want custom extensions deferred, so that the foundation does not decide extension registry shape prematurely.
38. As a maintainer, I want prompt interaction deferred, so that non-interactive rendering can be tested before blocking UI behavior exists.
39. As a maintainer, I want query/frontmatter behavior deferred, so that Template source ingestion does not absorb schema/query semantics too early.
40. As a maintainer, I want the minimal CLI vertical slice, so that the Template module proves end-to-end behavior before richer UX is added.

## Implementation Decisions

- Build the Template context as a service-first foundation, not as a MiniJinja wrapper.
- Define Template domain models before DTOs, repositories, processors, service orchestration, artifact commit pipeline, CLI behavior, and storage adapter details.
- Model `Template` as the primary renderable asset with stable identity, `PathKey`, derived `TemplateName`, validated `TemplateBody`, and recorded ingestion time.
- Keep `Template` non-exhaustive for later frontmatter, query, and metadata evolution.
- Model `RawTemplate` as a thin raw-content DTO.
- Model `RawTemplateView` as the freshness/cache view with `PathKey`, content hash, file metadata, and recorded time.
- Use existing content-hash support traits for raw views.
- Define Template repository traits using the segregated repository pattern: read, write, and unified marker traits.
- Include batch raw-view read/write methods for simple batch discovery and atomic cache updates.
- Keep filesystem materialization outside repository traits.
- Implement a Template Processor typestate pipeline with Discovery, Comparison, Parsed, Refresh, Construction, and Completed states.
- Stop Template Processor at `Completed`; do not add `Compiled` or `Validated` terminal states.
- Define `TemplateEngine` as the primary rendering port with `compile` and `render`.
- Keep `compile` narrow: engine-level source checking and owned template loading for an already-supplied Template.
- Keep `render` narrow: render an already-supplied Template with an already-supplied context.
- Keep `TemplateEngine` Lithos-shaped; do not mirror MiniJinja registration, loader, filter, global, or environment APIs.
- Implement `MiniJinjaEngine` with owned template sources and foundation engine configuration.
- Use MiniJinja built-ins only in foundation.
- Configure MiniJinja for strict undefined behavior and no Markdown auto-escape.
- Let `TemplateService` own repository lookup, indexing, validation workflow, render context assembly, render orchestration, target resolution, conflict checks, and commit orchestration.
- Allow `TemplateService` to expose `validate` for detailed compile validation and `can_compile` or `is_compilable` for post-ingestion tracing.
- Use `TemplateError` as the primary template use-case error type.
- Use `TemplateEngineError` for compile/render engine failures and preserve `minijinja::Error` as source.
- Defer `TemplateDiagnostic`; rely on well-written Rust errors and source chains in foundation.
- Model single-output write flow with `TemplateArtifact<State>` and states `Rendered`, `TargetResolved`, `ReadyToCommit`, and `Committed`.
- Commit behavior creates one file under a vault-safe target path, rejects absolute paths and traversal, and fails if the destination already exists.
- Use the FS context for path validation, reads, and writes; do not use raw `std::fs` in Template use cases.
- Add a minimal CLI shape: `lithos template render <template-name> --output <vault-relative-path> --var key=value`.
- Treat repeated `--var key=value` flags as flat raw MiniJinja context for foundation only.
- Defer declared inputs, namespaces, prompt UX, query helpers, custom extension modules, multi-file packs, and rich conflict policies.

## Testing Decisions

- Tests should assert external behavior and invariants, not private implementation details.
- Domain tests should cover Template construction, name derivation, body validation, identity behavior, raw view hashing, and serialization boundaries.
- Repository contract tests should cover read/write methods, path identity mappings, raw view persistence, batch operations, delete behavior, and missing-entity behavior.
- Processor tests should cover fresh, missing, stale timestamp, stale content, metadata-only refresh, and deleted-cache scenarios.
- Template Engine adapter tests should cover compile success, compile failure with preserved source error, render success, render failure, strict undefined behavior, no Markdown auto-escape, and owned source registration.
- Template Service tests should cover list, ingest/index, validate, compile health checks, render in memory, artifact creation, commit orchestration, missing Template errors, repository errors, and engine errors.
- Artifact typestate tests should cover legal transitions and externally observable write behavior; invalid transitions should be impossible by type construction rather than runtime tests.
- Commit pipeline tests should cover vault-relative target success, absolute target rejection, traversal rejection, existing destination failure, and single-file creation.
- CLI tests should cover the minimal render command, repeated `--var` flags, output path reporting, and structured failure paths.
- Architecture tests should continue enforcing FS isolation and context import boundaries.
- Prior art includes Schema repository/error tests, Schema discovery/processor patterns, FS path validation behavior, and existing architecture tests for ports and filesystem isolation.

## Out of Scope

- Lithos custom extension modules such as `date.*`, `str.*`, `path.*`, `file.*`, `num.*`, `prompt.*`, query helpers, and frontmatter handlers.
- `TemplateExtension` and `ExtensionRegistry` implementation.
- Prompt interaction, suggesters, declared template inputs, and interactive UX.
- Query/runtime objects such as `li.*`.
- Multi-file template packs and `TemplateArtifactSet<State>` implementation.
- Overwrite, skip, rename, append, merge-frontmatter, or other conflict policies.
- Arbitrary hooks, script execution, or side-effectful template execution beyond the single safe output commit.
- Rich custom diagnostics, diagnostic codes, snippets, suggestions, or pretty-rendering frameworks.
- `inputs.*` namespacing or long-term user-friendly context construction.
- Moving general file freshness ownership into the vault filesystem indexer.

## Further Notes

- The foundation should remain small enough to implement and verify as a vertical slice.
- The design intentionally leaves the exact `can_compile` versus `is_compilable` method name open.
- The design intentionally leaves exact Template Artifact fields and transition method names open.
- The design intentionally leaves exact Template Engine method signatures and error field types open where implementation pressure may refine names.
- The follow-up phases are expected in this order: extension registry, interactive prompt extension, and query/frontmatter extension.
