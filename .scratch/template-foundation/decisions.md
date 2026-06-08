# Architectural & Component Decisions

This file captures decisions made so far. It is not exhaustive; unresolved items are listed under **Open Decisions**.

## Hexagonal Architecture
- **Template ports**: Template-specific repository traits are defined in `lithos-core/src/template/` and must follow ADR 016's segregated repository pattern: `ReadRepository`, `WriteRepository`, and `Repository`.
- **Interaction ports**: `InputProvider` and `SelectionProvider` do not belong in `template`. They are planned for a new top-level `lithos-core/src/interact/` or `lithos-core/src/prompt/` context/module.
- **Adapters (Implementations)**: Defined at the edges. Example: `InquireAdapter` in `lithos-cli` using the `inquire` crate.
- **Core Purity**: `lithos-core` remains free of terminal-specific or UI-specific dependencies.
- **Service-first design**: `TemplateService` use cases are defined before deciding optional ports such as `TemplateExtension` or `ExtensionRegistry`.
- **MiniJinja isolation**: MiniJinja types remain localized to the rendering adapter/factory. Domain models, repositories, service requests, and service responses do not expose `minijinja` types.

## Foundation Rendering Boundary
- **Primary engine port**: Foundation defines `TemplateEngine` as the primary rendering port implemented by `MiniJinjaEngine`.
- **Lithos-shaped API**: `TemplateEngine` exposes `compile` and `render`; it does not mirror MiniJinja registration, filter, global, loader, or environment APIs.
- **Compile meaning**: `TemplateEngine::compile` is engine-level syntax/source checking and owned template loading for an already-supplied `Template`. Service-level validation remains a `TemplateService` responsibility.
- **Compile boundary**: `compile` must not perform repository lookup, template discovery, indexing, target resolution, conflict checks, filesystem writes, or CLI context assembly.
- **Service boundary**: `TemplateService` owns use-case orchestration and may call `compile` inside named-template validation or render preparation.
- **Validation reporting**: Template correctness under the configured engine is modeled as a `TemplateService` validation/check result, not as a terminal `TemplateProcessor<State>` state.
- **Built-ins only**: `template-foundation` uses MiniJinja built-ins only. No Lithos custom extension modules are included.
- **Engine configuration**: Foundation configures MiniJinja for Lithos semantics: owned template sources, strict undefined behavior, and no Markdown auto-escape.
- **No extension registry**: `TemplateExtension` and `ExtensionRegistry` are explicitly out of scope for foundation.
- **No capability trait split yet**: Foundation keeps `TemplateEngine` as one trait instead of splitting `TemplateCompiler` and `TemplateRenderer`; smaller traits can be introduced later only with concrete pressure.
- **Follow-up phase**: A dedicated `template-extension-registry` phase should immediately follow foundation to design the extension model for prompt, query, file, path, date, string, and numeric modules.

## TemplateService Foundation Scope
- **Use cases**: Foundation covers listing templates, ingesting/indexing templates, validating a named template, rendering Markdown in memory, creating a single-output artifact, and committing that artifact.
- **Non-interactive**: Foundation excludes prompts, suggesters, query/runtime objects, declared template inputs, and extension packs.
- **Single-output**: Foundation handles one rendered output file. Multi-file template packs are planned later.
- **Minimal context**: Foundation CLI accepts flat `--var key=value` entries as a raw MiniJinja context. Namespaced inputs and UX-friendly collection are deferred.
- **Minimal CLI**: Foundation includes a CLI vertical slice such as `lithos template render <template-name> --output <vault-relative-path> --var key=value`.

## TemplateProcessor Boundary
- **Ingestion responsibility**: `TemplateProcessor<State>` is responsible for reading, comparing, parsing, constructing, and persisting valid Lithos template assets.
- **Terminal state**: `TemplateProcessor<State>` stops at `Completed`; it does not add `Compiled` or `Validated` as final states in foundation.
- **Engine validation**: `TemplateService` reports whether persisted templates compile under the configured engine by orchestrating repository lookup and `TemplateEngine::compile`.

## TemplateArtifact Typestate
- **Artifact pipeline**: Rendered output moves through `TemplateArtifact<State>` states before write.
- **States**: Accepted foundation states are `Rendered`, `TargetResolved`, `ReadyToCommit`, and `Committed`.
- **Terminal form**: The committed artifact remains `TemplateArtifact<Committed>`; no separate `CommittedTemplateArtifact` type.
- **State meanings**: `Rendered` has content only; `TargetResolved` carries a safe vault-relative target; `ReadyToCommit` has passed conflict checks; `Committed` means the write succeeded.

## Persistence & Caching
- **Database**: Using `redb` for identity and caching.
- **Zero-Copy**: Using `rkyv` for efficient serialization/deserialization.
- **Change Detection**: Using `Blake3Hash` from `lithos-core/src/support/` for `RawTemplateView`. `Blake3HashIndex` is deferred until frontmatter/template sections require keyed sub-hashes.
- **Design order**: Domain models and DTOs must be designed before the typestate processor, because processor stages should move typed domain data rather than ad-hoc strings/maps.

## Future-Proofing
- **Vault Move**: The initial template processor is designed with the intent that it will eventually migrate to the `vault` module as part of the filesystem indexer. This ensures file freshness logic is consistent across the project.
- **Filesystem freshness**: Template-specific freshness checks should be a small, replaceable seam. The future vault filesystem indexer is expected to own general file freshness.

## Project Phases
1. **template-foundation**: Minimal domain models, raw views, repository traits, ingestion typestate processor, non-interactive `TemplateService`, single-output `TemplateArtifact` commit pipeline, and minimal CLI. (Current focus)
2. **template-extension-registry**: Extension registry design, pure vs effectful extension classification, namespaces/signatures, render-mode capabilities, and first-party modules such as date/string/path/file/numeric.
3. **template-user-interaction**: Parity with Templater system modules (prompt, suggester) using `inquire`.
4. **template-query**: Structured query builder (Dataview-style) and the `frontmatter` handler.

## Open Decisions
- Exact location name for interaction ports: `lithos-core/src/interact/` vs `lithos-core/src/prompt/`.
- Exact shape of `TemplateArtifact<State>` fields and transition methods.
- Exact `TemplateEngine` method signatures and error type names.
- Whether template file reads and writes call existing FS context ports directly or through template-specific ports that delegate to FS adapters.
- Exact minimal CLI command shape and argument naming.
