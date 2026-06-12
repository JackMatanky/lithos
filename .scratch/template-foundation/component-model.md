# Template Foundation Component Model

This file is a working design surface for the foundation grilling session. It captures accepted foundation component decisions before implementation planning.

## Design Order

1. Domain models.
2. DTOs and raw views.
3. Repository traits.
4. Typestate processor.
5. Template service orchestration.
6. Single-output artifact commit pipeline.
7. Minimal CLI vertical slice.
8. Storage adapter tables and redb/rkyv details.

## Domain Models To Design

### `Template`

The primary domain aggregate for a renderable template asset.

Foundation shape:

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Template {
    id: TemplateId,
    path: PathKey,
    name: TemplateName,
    body: TemplateBody,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}
```

- `TemplateId(UuidV7)` — stable identity, matching `NoteId`, `SchemaId`, `FileId`.
- `path: PathKey` — vault-relative canonical location (not a separate newtype).
- `TemplateName` — validated field derived from path stem at construction time.
- `TemplateBody` — validated wrapper around the renderable template source string.
- `recorded_at` — private ingestion timestamp with `#[rkyv(with = AsUnixTime)]`, matching `Schema`.
- `#[non_exhaustive]` — foundation shape is intentionally extensible for future frontmatter/query fields.

Decisions:
- `Template` starts as the minimal validated aggregate needed to identify and render a template asset.
- The foundation shape is intentionally extensible; richer user-facing metadata can be added in later phases once frontmatter/query semantics are designed.
- Extension registry is not part of foundation and is not stored on the aggregate.
- Frontmatter postponed to `template-query`.
- Both `TemplateName` and path are derived from `FilePath` at construction time.
- MiniJinja receives `(name, source)` pairs at the adapter boundary, never a filesystem path directly.

### `RawTemplate`

The raw file-backed DTO produced from reading a template file before parsing/validation.

Foundation shape:
- `RawTemplate { content: String }` — a thin named wrapper for pipeline clarity.
- No path (ingestion context assigns it), no metadata (on `RawTemplateView`).
- Processor takes `(content, PathKey, FileMetadata)` from FS adapter and produces `Template` directly.

### `RawTemplateView`

The lightweight cache view for freshness and staleness checks.

Foundation shape:
- No `TemplatePath` newtype — use `PathKey` directly.
- `FileMetadata` replaces separate `created_at`/`modified_at` fields.
- Single `Blake3Hash` for content identity (no `Blake3HashIndex` until frontmatter/query phase).
- Must implement `HasContentHash` and `HasContentHashMut` from `lithos-core/src/support/content_hash.rs`.
- Seam kept small for future vault filesystem indexer takeover.

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawTemplateView {
    path: PathKey,
    content_hash: Blake3Hash,
    metadata: FileMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl HasContentHash for RawTemplateView { ... }
impl HasContentHashMut for RawTemplateView { ... }
```

## Template Rendering Boundary

Following hexagonal architecture, MiniJinja must not leak into domain models, repository traits, or service requests/responses.

Foundation includes `TemplateEngine` as the primary rendering port. The port is Lithos-shaped and must not mirror MiniJinja's API.

### Domain layer (lithos-core/src/template/)

```rust
/// Primary rendering port — no MiniJinja types exposed.
pub trait TemplateEngine: Send + Sync {
    /// Compile or load a template into the configured engine.
    ///
    /// This is engine-level syntax/source checking and owned template loading,
    /// not service-level validation or workflow orchestration.
    fn compile(
        &mut self,
        template: &Template,
    ) -> Result<(), TemplateEngineError>;

    /// Render a compiled or loadable template with the supplied context.
    fn render(
        &self,
        template: &Template,
        context: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>;
}
```

### Adapter layer (where MiniJinja lives — e.g., a template adapter module)

```rust
pub struct MiniJinjaEngine {
    env: Environment<'static>,
}

impl TemplateEngine for MiniJinjaEngine {
    fn configured() -> Self { ... }
    fn compile(&mut self, template: &Template) -> Result<(), TemplateEngineError> { ... }
    fn render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>) -> Result<TemplateArtifact<Rendered>, TemplateEngineError> { ... }
}
```

### Design decisions

- `Environment<'static>` + `add_template_owned()` avoids lifetime coupling between source strings and the environment.
- `compile` stays on `TemplateEngine` in foundation, but its scope is narrow: check/load the supplied `Template` source for the configured engine.
- `compile` must not perform repository lookup, template discovery, indexing, target resolution, conflict checks, filesystem writes, or CLI context assembly.
- `TemplateService` owns use-case orchestration and may call `compile` as one step of `validate_template` or render preparation.
- Template correctness under the configured engine is reported by `TemplateService`, not encoded as a terminal `TemplateProcessor` state.
- Foundation rendering uses MiniJinja built-ins only.
- Foundation configures MiniJinja for Lithos semantics: no Markdown auto-escape, strict undefined behavior, owned template sources.
- Foundation has no `TemplateExtension`, no `ExtensionRegistry`, and no Lithos custom modules (`date.*`, `str.*`, `path.*`, `file.*`, `prompt.*`, query helpers).
- Context is a flat `serde_json::Map<String, serde_json::Value>` assembled from minimal CLI `--var key=value` flags.
- The flat context is a foundation proving tool, not the long-term UX. Declared inputs, namespaces, prompts, and richer context construction belong to later phases.
- Adapter setup remains localized so a later template extension registry can replace or extend it cleanly.
- Do not split `TemplateEngine` into smaller capability traits in foundation unless implementation pressure proves a concrete need. This avoids trait proliferation while preserving the primary adapter boundary.

## TemplateProcessor (Typestate)

A dual-typestate pipeline for template ingestion, mirroring `PropertyBankProcessor`.

### Stages (in logical order)

| Stage          | Purpose                                                    |
|----------------|------------------------------------------------------------|
| `Discovery`    | FS read produces `(content, metadata, path)`               |
| `Comparison`   | Check against `RawTemplateView` (timestamps, content hash) |
| `Parsed`       | Content validated into `RawTemplate` / `Template` body     |
| `Refresh`      | Early-commit when only metadata changed                    |
| `Construction` | Build and persist `Template` aggregate                     |
| `Completed`    | Terminal — `Template` owned                                |

### Flow

```text
Discovery ─┬─ no view
           │     → Comparison (Missing)
           │     → Parsed (Missing) → parse content → Construction (New) → Completed
           └─ view found
                 → Comparison (Present) → check timestamps
                       ├─ match → Construction (Fresh) → fetch cached → Completed
                       └─ mismatch → Comparison (Suspect, content loaded)
                             └─ check content hash
                                   ├─ match → Refresh (StaleTimestamps, sync metadata)
                                   │        → Construction (Fresh) → fetch cached → Completed
                                   └─ mismatch → Parsed (Stale)
                                               → Construction (New) → Completed
```

Individual paths skip stages (fresh path goes Comparison → Construction, skipping Parsed and Refresh).

`TemplateProcessor<Phase, Status>` stops at `Completed`. It is responsible for ingesting valid Lithos template assets, not proving those assets compile under a configured render engine. Engine compilation/check results belong to `TemplateService` validation reporting.

## TemplateService (Foundation Scope)

`TemplateService` is the application/use-case orchestrator for the limited foundation vertical slice. It is not a MiniJinja wrapper and should not decide extension registry shape.

Foundation use cases:
- List available templates.
- Ingest/index templates from configured template sources.
- Validate a named template without rendering.
- Check whether a processed template can compile, for post-ingestion tracing.
- Render a named template to Markdown in memory.
- Build a single-output `TemplateArtifact` for a vault-safe target.
- Commit that artifact with basic safe filesystem behavior.

Boundary with `TemplateEngine`:
- `TemplateService` finds the template and assembles the render context before invoking the engine.
- `TemplateService` owns named-template validation workflow; `TemplateEngine::compile` only reports engine-level source/syntax/load failures for the supplied `Template`.
- Missing templates are `TemplateError::NotFound`, not engine errors.
- Foundation does not introduce a custom `TemplateDiagnostic` type. Invalid existing templates carry a Lithos-owned `TemplateEngineError` with source chaining.
- `TemplateService` reports whether a template compiles under the configured engine; this is not a `TemplateProcessor<Phase, Status>` terminal state.
- `TemplateService` may expose `can_compile` or `is_compilable` so callers can run a boolean compile check after `TemplateProcessor` reaches `Completed` and emit tracing about template health.
- `TemplateService` owns all target resolution, conflict checks, and filesystem commit orchestration after rendering.
- `TemplateEngine` must not know about repositories, config lookup, filesystem paths, CLI flags, or commit policy.

Foundation error shape:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("template `{name}` was not found")]
    NotFound { name: String },

    #[error("failed to load template from `{path}`")]
    Load {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Engine(#[from] TemplateEngineError),
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    #[error("failed to compile template `{name}`")]
    Compile {
        name: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("failed to render template `{name}`")]
    Render {
        name: String,
        #[source]
        source: minijinja::Error,
    },
}
```

Error boundary:
- `TemplateError` is the primary template use-case error type.
- `TemplateError::Engine` embeds `TemplateEngineError` transparently.
- `TemplateEngineError` exposes MiniJinja source errors because MiniJinja is the selected foundation engine adapter.
- The dependency boundary is not crate-wide: `lithos-core` may depend on MiniJinja for an adapter module. The protected boundary is the Template domain/service/port API surface.
- MiniJinja types must not appear in Template domain models, repository traits, service requests, or service responses. If an error type is returned by a domain port, keep its public contract Lithos-shaped even when the implementation preserves MiniJinja as a source error.

Accepted validation/compile-check shape:
- The service may expose `validate(...) -> Result<(), TemplateError>` for detailed compile validation that preserves error chains.
- The service may also expose `can_compile(...) -> Result<bool, TemplateError>` or `is_compilable(...) -> Result<bool, TemplateError>` for post-ingestion tracing after `TemplateProcessor` completes.
- The boolean compile-check method should log/tracing-report compile failures without replacing the detailed validation path.
- Do not add `TemplateValidation` or `TemplateValidationStatus` unless the service needs richer non-error validation reporting.

Out of scope for foundation:
- Lithos custom extensions and extension packs.
- Prompt interaction and declared template inputs.
- Query/runtime objects such as `li.*`.
- Multi-file template packs.
- Overwrite, skip, rename, append, or merge-frontmatter conflict policies.
- Arbitrary hooks or script execution.

## TemplateArtifact (Write Typestate)

Foundation uses singular `TemplateArtifact<State>` for the write pipeline. The terminal state remains `TemplateArtifact<Committed>` rather than a separate committed type.

```rust
pub struct TemplateArtifact<State> {
    template: TemplateName,
    content: String,
    target: Option<PathKey>,
    _state: PhantomData<State>,
}

pub struct Rendered;
pub struct TargetResolved;
pub struct ReadyToCommit;
pub struct Committed;
```

Accepted state meaning:
- `Rendered`: MiniJinja has produced content, but the artifact is not writeable.
- `TargetResolved`: the requested output target has been normalized into a safe vault-relative `PathKey`.
- `ReadyToCommit`: the target passed conflict checks and can be written.
- `Committed`: the filesystem write succeeded.

Foundation commit behavior:
- Create one file under a vault-safe target path.
- Reject absolute paths and traversal.
- Fail if the destination already exists.
- No overwrite, skip, rename, append, merge, or multi-file operations.
- Use the existing FS context writer/path validation rather than raw `std::fs`.

Planned multi-file evolution:
- Keep `TemplateArtifact<State>` for one output item.
- Add `TemplateArtifactSet<State>` when template packs introduce multi-file generation.

## Repository Traits

Template repositories must follow ADR 016:

- `template::ReadRepository`: reads templates, raw views, and identity mappings.
- `template::WriteRepository`: saves templates, raw views, and identity mappings.
- `template::Repository`: marker trait extending read and write traits.

Foundation trait methods:

```rust
pub trait ReadRepository {
    fn find_template_by_id(&self, id: TemplateId)
        -> Result<Option<Template>, TemplateRepositoryError>;
    fn find_template_by_path(&self, path: &PathKey)
        -> Result<Option<Template>, TemplateRepositoryError>;
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError>;
    fn get_raw_template_view(&self, path: &PathKey)
        -> Result<Option<RawTemplateView>, TemplateRepositoryError>;
    fn get_raw_template_views(&self, paths: &[PathKey])
        -> Result<Vec<Option<RawTemplateView>>, TemplateRepositoryError>;
}

pub trait WriteRepository {
    fn save_template(&self, path: &PathKey, template: &Template)
        -> Result<(), TemplateRepositoryError>;
    fn save_many_templates(&self, entries: &[(PathKey, Template)])
        -> Result<(), TemplateRepositoryError>;
    fn delete_template(&self, id: TemplateId)
        -> Result<(), TemplateRepositoryError>;
    fn save_raw_template_view(&self, path: &PathKey, view: &RawTemplateView)
        -> Result<(), TemplateRepositoryError>;
    fn save_many_raw_template_views(&self, entries: &[(PathKey, RawTemplateView)])
        -> Result<(), TemplateRepositoryError>;
    fn delete_raw_template_view(&self, path: &PathKey)
        -> Result<(), TemplateRepositoryError>;
}
```

Method rationale:
- `get_raw_template_views` (batch) added for simple batch discovery, matching schema pattern.
- `save_many_raw_template_views` added for atomic batch cache updates.
- `save_template` takes `(path, template)` to atomically write template + path index.
- FS materialization is not a repository concern; the processor reads via FS adapters and persists via repository.

## Minimal CLI

Foundation includes a minimal CLI vertical slice so the template module has usable output behavior.

Proposed command shape:

```text
lithos template --input <template-name> --output <vault-relative-path> --var key=value
```

Shortened forms `lithos template -i <template-name> -o <vault-relative-path> --var key=value` accepted.

CLI behavior:
- Load config/vault context.
- Ensure templates are indexed or load the named template through `TemplateService`.
- Accept `--input`/`-i` for template name and `--output`/`-o` for vault-relative path.
- Convert repeated `--var key=value` flags into a flat render context.
- Render, resolve target, conflict-check, and commit through the foundation service.
- Print the created path or a structured error.

## Interaction Ports

Interaction ports are not template repository traits.

Planned location:
- `lithos-core/src/interact/` or `lithos-core/src/prompt/`.

Planned ports:
- `InputProvider` for text input.
- `SelectionProvider` for single and multi-select input.

Adapter decision:
- `lithos-cli` owns the `inquire` dependency and implements these ports.

## Foundation Extensions

The full planned extension registry is captured in `planned-extensions.md`.

Foundation includes no Lithos custom extensions. MiniJinja built-ins are allowed.

Later phases include:
- A dedicated `template-extension-registry` phase for extension registry design.
- `prompt.*` in `template-user-interaction`.
- Query and frontmatter handling in `template-query`.
