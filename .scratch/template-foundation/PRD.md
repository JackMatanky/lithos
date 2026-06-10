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
4. As a Lithos user, I want to check whether a named Template can compile via a CLI command, so that template health can be verified on demand without triggering a render.
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
16. As a developer, I want `TemplateName` derived from the subdirectory-qualified path stem relative to the configured template directory, so that users can refer to Templates by a stable, collision-free name.
17. As a developer, I want `TemplateBody` to wrap renderable source text with structural invariants only (non-empty, UTF-8), so that syntax validation remains the engine's responsibility and future frontmatter can be added.
18. As a developer, I want `RawTemplate` to remain a thin raw-content DTO, so that ingestion stages are explicit without over-modeling metadata.
19. As a developer, I want `RawTemplateView` to store content hash and file metadata, so that freshness checks can avoid unnecessary parsing and persistence.
20. As a developer, I want `RawTemplateView` to implement `HasContentHash` and `HasContentHashMut` from `support::content_hash`, so that it participates in the existing crate-private hash support patterns.
21. As a developer, I want `RawTemplateView` and `Template` to derive `rkyv::Archive`, `rkyv::Serialize`, and `rkyv::Deserialize`, so that zero-copy persistence is consistent with all other persisted aggregates.
22. As a developer, I want Template repositories to follow segregated `ReadRepository`, `WriteRepository`, and `Repository` traits, so that persistence stays isolated behind Template-owned ports.
23. As a developer, I want batch raw-view operations, so that template discovery can compare multiple paths efficiently.
24. As a developer, I want the Template Processor to stop at `Completed`, so that engine compilation does not become an ingestion state.
25. As a developer, I want Template Engine `compile` to mean engine-level source checking only (calling `env.get_template` against an already-loaded source), so that service-level validation does not leak into the adapter port and no mutation of the engine occurs at call time.
26. As a developer, I want Template Engine `render` to accept an already-supplied Template and context, so that the engine does not own lookup or context assembly.
27. As a developer, I want Template Service to own lookup, validation workflow, rendering orchestration, target resolution, conflict checks, and commit orchestration, so that use-case logic stays in one place.
28. As a developer, I want Template Engine errors to preserve MiniJinja source errors, so that Rust error chains remain useful for debugging.
29. As a developer, I want Template use cases to return `TemplateError`, so that missing Templates, load failures, and engine failures share one template-level error surface.
30. As a developer, I want no custom `TemplateDiagnostic` in foundation, so that diagnostics do not become a speculative framework.
31. As a developer, I want `TemplateArtifact<State>` to enforce the write pipeline using the hoverbear generic state machine pattern (`impl From<TemplateArtifact<S1>> for TemplateArtifact<S2>`), so that content cannot be committed before target resolution and conflict checks, and compile-time errors list valid transitions.
32. As a developer, I want terminal write state to remain `TemplateArtifact<Committed>`, so that the typestate API stays consistent.
33. As a developer, I want future multi-file generation to use `TemplateArtifactSet<State>`, so that foundation does not overbuild for packs.
34. As a maintainer, I want MiniJinja types kept out of Template domain models, repository traits, service requests, and service responses, so that Template APIs stay Lithos-shaped.
35. As a maintainer, I want MiniJinja allowed in an adapter module inside `lithos-core`, so that dependency boundaries are based on API leakage rather than crate-level absolutism.
36. As a maintainer, I want FS reads and writes to use the FS context rather than raw `std::fs`, so that filesystem isolation remains enforced.
37. As a maintainer, I want the initial freshness seam to be small, so that a future vault filesystem indexer can take over general file freshness.
38. As a maintainer, I want custom extensions deferred, so that the foundation does not decide extension registry shape prematurely.
39. As a maintainer, I want prompt interaction deferred, so that non-interactive rendering can be tested before blocking UI behavior exists.
40. As a maintainer, I want query/frontmatter behavior deferred, so that Template source ingestion does not absorb schema/query semantics too early.
41. As a maintainer, I want the minimal CLI vertical slice, so that the Template module proves end-to-end behavior before richer UX is added.

## Implementation Decisions

### Domain Models

- Build the Template context as a service-first foundation, not as a MiniJinja wrapper.
- Define Template domain models before DTOs, repositories, processors, service orchestration, artifact commit pipeline, CLI behavior, and storage adapter details.
- Model `Template` as the primary renderable asset with stable identity, `PathKey`, derived `TemplateName`, validated `TemplateBody`, and recorded ingestion time. Derive `rkyv::Archive + Serialize + Deserialize` on `Template`.
- Keep `Template` non-exhaustive for later frontmatter, query, and metadata evolution.
- Model `RawTemplate` as a thin raw-content DTO.
- `TemplateId` follows the `NoteId`/`SchemaId` pattern: a newtype wrapping `UuidV7`. It is assigned during the processor's Construction stage — resolved from the repository if the template already exists, or generated fresh for new templates.
- `TemplateName` is derived from the vault-relative path stem of the template file, qualified by any subdirectory structure within the configured template directory. For example, a file at `templates/daily/standup.md` produces `TemplateName("daily/standup")`. The `/` separator is permitted in `TemplateName`. This scheme is unique across the template directory tree without collision. `TemplateName` is derived at construction time from the file path, not from a separate field.
- `TemplateBody` wraps renderable source text with structural invariants only: non-empty and valid UTF-8. Jinja syntax validity is the engine's responsibility, not the domain's. This boundary is intentional and preserves the ability to add template frontmatter in a future phase without requiring engine validation at construction.
- Model `RawTemplateView` as the freshness/cache view with `PathKey`, content hash (`Blake3Hash`), file metadata, and recorded time. Derive `rkyv::Archive + Serialize + Deserialize`. Implement `HasContentHash` and `HasContentHashMut` from `support::content_hash`.
- `RawTemplateView` follows the version-history pattern of `RawSchemaView`, not a simpler flat struct.

### Template Configuration

- Define a `TemplateConfigSpec` following the same pattern as `SchemaConfigSpec`, exposing only the values the Template context requires from `Config`:
  ```rust
  pub struct TemplateConfigSpec {
      /// Vault root directory.
      root: DirPath,
      /// Relative path to the template directory from vault root.
      directory: RelativeDirPath,
  }
  ```
- Template discovery reads from the directory resolved by `TemplateConfigSpec`. All template files must be Markdown (`.md`) files directly in or in subdirectories of the configured template directory.

### Repository

- Define Template repository traits using the segregated repository pattern: read, write, and unified marker traits (`ReadRepository`, `WriteRepository`, `Repository`).
- Include batch raw-view read/write methods for simple batch discovery and atomic cache updates.
- Keep filesystem materialization outside repository traits.

### Processor

- Implement a Template Processor typestate pipeline with Discovery, Comparison, Parsed, Refresh, Construction, and Completed states.
- Stop Template Processor at `Completed`; do not add `Compiled` or `Validated` terminal states. Compilation health is a live, on-demand check via the CLI, not a stored ingestion state.
- `TemplateId` is resolved once in the Construction stage and carried through downstream states, eliminating redundant repository lookups.

### Template Engine Port and MiniJinjaEngine Adapter

- Define `TemplateEngine` as the primary rendering port with `compile` and `render`. Both methods take `&self`.
- `compile` is narrow: it checks engine-level source validity by calling `env.get_template(name)` against sources already loaded into the engine at construction. It does not mutate the engine at call time.
- `render` is narrow: render an already-supplied `Template` with an already-supplied `TemplateContext`.
- Keep `TemplateEngine` Lithos-shaped; do not mirror MiniJinja registration, loader, filter, global, or environment APIs.
- `TemplateEngine` does not require `Clone + Send + Sync + 'static` bounds. This is a CLI tool, not a web server. Those bounds are axum-specific and do not apply here.
- Implement `MiniJinjaEngine` as a build-once adapter: all template sources are registered at construction via `set_loader` or `add_template_owned`. The engine is never mutated after construction. `Environment<'static>` already implements `Send + Sync` natively via its internal `memo-map` loader cache, so no `Mutex` or `RwLock` is required.
- `MiniJinjaEngine` holds `env: minijinja::Environment<'static>` as a plain owned field. No `Arc`, `Mutex`, or `RwLock` needed for the foundation.
- Use MiniJinja built-ins only in foundation.
- Configure MiniJinja for strict undefined behavior (`UndefinedBehavior::Strict`) and no Markdown auto-escape (`AutoEscape::None`).

### Template Service

- Let `TemplateService` own repository lookup, indexing, validation workflow, render context assembly, render orchestration, target resolution, conflict checks, and commit orchestration.
- `TemplateService` exposes `validate` for detailed compile validation (used before rendering) and `can_compile` (or `is_compilable`) as an explicit, on-demand live check that calls the engine's `compile` method. Compilability is not stored on `Template` or in any registry; it is always a live engine call.
- The render context passed to the engine is `HashMap<String, String>`. The `MiniJinjaEngine` adapter converts `String → minijinja::Value::String` internally. This keeps all `minijinja` types out of the Service request and response types.
- Use `TemplateError` as the primary template use-case error type.
- Use `TemplateEngineError` for compile/render engine failures and preserve `minijinja::Error` as source.
- Defer `TemplateDiagnostic`; rely on well-written Rust errors and source chains in foundation.

### Template Artifact Write Pipeline

- Model single-output write flow with `TemplateArtifact<State>` using the hoverbear generic state machine pattern. The struct carries shared data in its outer fields and per-state data inside the `state: S` field. `From` implementations define valid transitions:
  - `TemplateArtifact<Rendered>` → `TemplateArtifact<TargetResolved>` (path validation)
  - `TemplateArtifact<TargetResolved>` → `TemplateArtifact<ReadyToCommit>` (conflict check)
  - `TemplateArtifact<ReadyToCommit>` → `TemplateArtifact<Committed>` (file write)
- Invalid transitions (e.g., `Rendered` → `Committed` directly) are impossible by type construction. The `From` impl pattern produces compiler errors that list valid options.
- Commit behavior creates one file under a vault-safe target path using `File::create_new` (stable since Rust 1.77.0), which atomically fails with `ErrorKind::AlreadyExists` if the destination exists. No separate existence pre-check is needed, eliminating the TOCTOU race condition.
- Absolute path rejection and traversal rejection happen in the `resolve_target` transition (`Rendered → TargetResolved`), wrapping `PathValidator` logic.
- Use the FS context (`FileWriter`, i.e., `fs::Writer`) for path validation and writes; do not use raw `std::fs` in Template use cases. `fs::Writer` is renamed to `FileWriter` for consistency with `FileReader`.

### CLI

- Add a minimal CLI shape with two commands:
  - `lithos template --input <template-name> --output <vault-relative-path> --var key=value` (shortened forms `-i`, `-o` accepted)
  - `lithos template check --input <template-name>` for on-demand compile health check
- Treat repeated `--var key=value` flags as a `HashMap<String, String>` render context. The `=` in values is handled by splitting on the first `=` only.
- The CLI adapter maps `TemplateError` variants to user-facing messages explicitly; it does not forward raw `TemplateError::to_string()` output to users.
- Defer declared inputs, namespaces, prompt UX, query helpers, custom extension modules, multi-file packs, and rich conflict policies.

## Testing Decisions

- Tests should assert external behavior and invariants, not private implementation details.
- Domain tests should cover Template construction, name derivation (including subdirectory qualification), body validation (structural invariants only), identity behavior, raw view hashing, `HasContentHash`/`HasContentHashMut` implementation, and rkyv serialization round-trips.
- Repository contract tests should cover read/write methods, path identity mappings, raw view persistence, batch operations, delete behavior, and missing-entity behavior.
- Processor tests should cover fresh, missing, stale timestamp, stale content, metadata-only refresh, and deleted-cache scenarios.
- Template Engine adapter tests should cover compile success, compile failure with preserved source error, render success, render failure, strict undefined behavior, no Markdown auto-escape, and build-once source registration.
- Template Service tests should cover list, ingest/index, validate, `can_compile` live checks, render in memory, artifact creation, commit orchestration, missing Template errors, repository errors, and engine errors.
- Artifact typestate tests should cover legal transitions using the `From`-based API and externally observable write behavior; invalid transitions should be impossible by type construction rather than runtime tests.
- Commit pipeline tests should cover vault-relative target success, absolute target rejection, traversal rejection, existing destination failure (via `File::create_new` `AlreadyExists`), and single-file creation.
- CLI tests should cover the render command, the check command, repeated `--var` flags with `=` in values, output path reporting, and structured failure paths.
- Architecture tests should continue enforcing FS isolation and context import boundaries, extended to cover the Template context's own isolation (no cross-imports from `note` or `schema`; MiniJinja only in the adapter module).
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
- Storing compilability state on `Template` or in any registry structure; compilability is always a live engine check.

## Further Notes

- The foundation should remain small enough to implement and verify as a vertical slice.
- The design intentionally leaves the exact `can_compile` versus `is_compilable` method name open.
- The design intentionally leaves exact Template Artifact state field names and `From` transition implementation details open where implementation pressure may refine them.
- The design intentionally leaves exact Template Engine method signatures and error field types open where implementation pressure may refine names.
- The follow-up phases are expected in this order: extension registry, interactive prompt extension, and query/frontmatter extension.
- `File::create_new` is stable since Rust 1.77.0 and handles the "fail if exists" commit requirement atomically without a separate TOCTOU-unsafe pre-check.
- `MiniJinjaEngine` does not require `Mutex`/`RwLock` because `Environment<'static>: Send + Sync` natively via its internal `memo-map` loader cache. The `!Freeze` marker on `Environment` reflects this interior mutability, which is already thread-safe.
- The hoverbear generic state machine pattern is documented at https://hoverbear.org/blog/rust-state-machine-pattern/#generically-sophistication — use `impl From<TemplateArtifact<S1>> for TemplateArtifact<S2>` for transitions; the outer struct carries shared fields, the `state: S` field carries per-state data.
