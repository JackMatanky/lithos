---
title: 07-template-service
category: enhancement
label: ready-for-agent
status: implemented
branch: feat/07-template-service
merge_commit:
date_created: 2026-06-11
date_completed:
---

# Template Service

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement `TemplateService` — the use-case orchestrator that wires together the repository, engine, and artifact pipeline into two primary operations:

**`load(&self)`** — Orchestrates the `TemplateProcessor` pipeline to ingest/index templates from the configured source into the repository. Returns a `HashMap<TemplateName, Template>` for downstream use by `create()`. Wires up the deferred batch deletion from issue 04.

**`create(&mut self)`** — Orchestrates a full render-to-commit flow:
1. Look up the named `Template` from the in-memory map (populated by `load()`) — fail-fast with `NotFound` before engine compilation
2. Compile all templates from the map into `self.engine` (implicit compile-time validation — broken source is caught here)
3. Call `self.engine.render(template, context)` with the supplied `HashMap<String, String>` context
4. Drive the `TemplateArtifact` typestate pipeline: `Rendered → TargetResolved → Committed`
5. Return the created vault-relative path on success

A `--dry-run` variant of `create()` performs steps 1–3 only (renders but does not commit) and returns the rendered string.

`TemplateService` owns: lookup, validation workflow, rendering orchestration, target resolution, conflict checks, and commit orchestration. None of that logic leaks into the engine port or the artifact states.

### Architecture

`TemplateService` is a **stateful struct** composing its ports via uniform generic parameters per the hexagonal architecture pattern:

```rust
pub struct TemplateService<R, W, E> {
    repository: R,
    writer: W,
    engine: E,
    config: TemplateConfigSpec,
}
```

Where:
- `R: ReadRepository + WriteRepository` — template persistence
- `W: FileWriter` — filesystem writes
- `E: TemplateEngine` — rendering engine (constructed once, templates compile/accumulate across calls)
- `config: TemplateConfigSpec` — provides vault root via `self.config.root()`, no separate `vault_root` field needed
- `load()` returns `HashMap<TemplateName, Template>` which `create()` uses for both lookup and compilation in one pass

All three generic dependencies use the same pattern: trait bounds + struct fields. No `Box<dyn ...>`, no factory closures, no double indirection. The composition root (main / test setup) injects concrete implementations:

```rust
// Production: MiniJinjaEngine injected at the composition root
let service = TemplateService::new(
    redb_repo,
    Writer::new(vault_root),
    MiniJinjaEngine::new(),
    config,
);

// Test: mock engine injected the same way
let mock = MockTemplateEngine::new();
let service = TemplateService::new(repo, writer, mock, config);
```

This refactors the existing zero-field `TemplateService` whose methods accepted `config` and `repository` as ad-hoc parameters.

### Why `&mut self` for `create()`

`TemplateEngine::compile(&mut self, ...)` takes `&mut self` because registering a template mutates engine internal state. `create()` needs `&mut self` to call `self.engine.compile()`. `load()` stays `&self` — it doesn't touch the engine.

### Error model

Two separate error layers, each with its own variants:

**Repository port** (`TemplateRepositoryError`):
- `NotFoundById(TemplateId)` — existing
- `NotFoundByPath(PathKey)` — existing
- `NotFoundByName(TemplateName)` — new, for consistency with existing `*ById`/`*ByPath` variants

**Service/use-case** (`TemplateError`):
- `NotFound { name: String }` — new. Service detects a miss from the map. This is **not** on `TemplateEngineError` because the service resolves the template *before* the engine is involved.
- `Engine(TemplateEngineError)` — existing. Wraps compile/render failures.
- `Artifact(TemplateArtifactError)` — new. Wraps artifact pipeline errors.

Engine does **not** get a `NotFound` variant — it only receives an already-fetched `&Template`. A request for a template that wasn't compiled is a programming error surfacing as `TemplateEngineError::Render`.

**Simplified `TemplateArtifactError`** (replaces the current over-named variants):
```rust
pub enum TemplateArtifactError {
    #[error(transparent)]
    Path(WriteTargetError),        // absolute, traversal, hidden, empty, current-dir
    #[error(transparent)]
    Write(WriteError),             // AlreadyExists, Io
}
```
No `TemplateWriteError` wrapper, no `AbsolutePathRejected`/`TraversalRejected`/`InvalidPath` named variants — `WriteTargetError` and `WriteError` already preserve those distinctions.

## Acceptance criteria

- [ ] `TemplateService<R, W, E>` stateful struct with uniform generics: `repository`, `writer`, `engine`, `config`
- [ ] `load(&self)` refactored to use `self.*` fields; returns `HashMap<TemplateName, Template>`
- [ ] `load()` handles batch deletion of raw template views for cached paths absent from scan (TODO from issue 04)
- [ ] `create(&mut self)` takes `&mut self` (for `engine.compile`), uses the template map directly
- [ ] Engine instance is a generic field `E: TemplateEngine`, injected at construction like repo and writer
- [ ] Missing template name returns `TemplateError::NotFound { name }` (no panic, no unwrap)
- [ ] Engine compile/render failures surface as `TemplateError::Engine(TemplateEngineError)`
- [ ] Absolute, traversal, hidden-file, empty, and current-dir output paths each return `TemplateError::Artifact(WriteTargetError::…)`
- [ ] Destination already exists returns `TemplateError::Artifact(WriteError::AlreadyExists { … })`
- [ ] No MiniJinja types appear in `TemplateService` method signatures or `TemplateError` public API
- [ ] No `unwrap()` or `panic!` in service code
- [ ] `#[allow(dead_code)]` removed from `artifact.rs` and `engine/mod.rs`
- [ ] Tests cover: `load()` orchestration with repository interactions, batch deletion, `create()` success path end-to-end, dry-run, missing template, engine failure propagation, all path rejection types, `AlreadyExists` propagation
- [ ] `mise run test` passes

## Blocked by

- `issue-03-repository-traits.md`
- `issue-04-processor-pipeline.md`
- `issue-05-engine-port-adapter.md`
- `issue-06-artifact-write-pipeline.md`

---

> *This was generated by AI during triage and refined during architectural review.*

## Agent Brief

**Category:** enhancement
**Summary:** Implement `TemplateService` — use-case orchestrator wiring repository, engine, and artifact pipeline into `load()` and `create()` operations

**Current behavior:**
`TemplateService` exists as a zero-field struct with `load()` taking `config` and `repository` as method parameters. Returns `Vec<Template>`. No `create()` method exists. The processor, engine, and artifact pipeline from issues 04–06 are complete as isolated components but have no production caller wiring them together.

**Desired behavior:**
`TemplateService<R, W, E>` lives in `crates/template/src/service.rs` and is a stateful struct composing its ports via uniform generics:

```rust
pub struct TemplateService<R, W, E> {
    repository: R,
    writer: W,
    engine: E,
    config: TemplateConfigSpec,
}
```

Where `R: ReadRepository + WriteRepository`, `W: FileWriter`, `E: TemplateEngine`.

**`load(&self) -> Result<HashMap<TemplateName, Template>, TemplateError>`**
- Constructs and runs the `TemplateProcessor` pipeline (Discovery → Completed) using `self.config`
- Persists the resulting `Template` aggregates and `RawTemplateView`s via `self.repository`
- After all templates are processed, performs batch deletion of cached `RawTemplateView` paths absent from the current scan (wires the TODO at `service.rs:62-63`)
- Returns a `HashMap<TemplateName, Template>` keyed by derived template name — `create()` uses this map for both lookup and compilation, avoiding separate `find_template_by_name` + `list_templates` repository calls

**`create(&mut self, input: CreateTemplateInput) -> Result<CreatedTemplatePath, TemplateError>`**
where `CreateTemplateInput` holds: template name (`TemplateName`), output path (vault-relative `&str` or `PathBuf`), and context (`HashMap<String, String>`)

Steps:
1. Look up the named `Template` from the map; return `TemplateError::NotFound { name }` if absent
2. Compile all templates from the map into `self.engine` (implicit compile-time validation — broken source is caught here via `compile`)
3. Call `self.engine.render(template, &context)`; map `TemplateEngineError` → `TemplateError::Engine`
4. Drive `TemplateArtifact` pipeline: `Rendered → TargetResolved → Committed`
5. Map `TemplateArtifactError` → `TemplateError::Artifact(TemplateArtifactError)`
6. Return the committed vault-relative path

**`create_dry_run(&mut self, input: DryRunInput) -> Result<String, TemplateError>`** (or a flag on `CreateTemplateInput`):
- Performs steps 1–3 of `create()` only (no artifact pipeline)
- Returns the rendered string without writing any file

**What changes in the error types:**

| Error | Variant | Layer |
|-------|---------|-------|
| `TemplateRepositoryError` | `NotFoundByName(TemplateName)` (new) | Repository port |
| `TemplateError` | `NotFound { name: String }` (new) | Service / use-case |
| `TemplateError` | `Artifact(TemplateArtifactError)` (new) | Service / use-case |
| `TemplateArtifactError` | `Path(WriteTargetError)` (simplified) | Artifact pipeline |
| `TemplateArtifactError` | `Write(WriteError)` (simplified) | Artifact pipeline |

Removed: `TemplateWriteError` wrapper, `AbsolutePathRejected`, `TraversalRejected`, `InvalidPath` named variants.

**Key interfaces:**
- `TemplateService<R, W, E>` — stateful struct; all three generics use the same direct-injection pattern
- `CreateTemplateInput` — new request type; all fields Traces-owned
- `CreatedTemplatePath` — newtype/alias for the committed vault-relative path
- `TemplateEngine` — generic `E` field, compiled with all templates from map inside `create()`
- Tests use `InMemoryRepository` + mock engine or `MiniJinjaEngine` directly

**Wire deferred batch deletion:**
At `crates/template/src/service.rs:62-63`, replace the TODO to iterate `_deleted_paths` and call `repository.delete_raw_template_view(path)?`. Template aggregate deletion needs ID resolution — the agent should decide whether to add `find_template_id_by_path` calls or batch-delete views only.

**Remove `#[allow(dead_code)]`:**
1. `crates/template/src/artifact.rs:9-14` — `create()` exercises the pipeline
2. `crates/template/src/engine/mod.rs:8` — `create()` calls `compile` and `render`

**Acceptance criteria:**
- [ ] `TemplateService<R, W, E>` with `repository`, `writer`, `engine`, `config` fields
- [ ] `new()` constructor taking all four dependencies
- [ ] `load(&self)` refactored to use `self.*` fields; returns `HashMap<TemplateName, Template>`
- [ ] `load()` wires batch deletion of orphaned raw template views
- [ ] `create(&mut self)` uses map for lookup + compilation, drives artifact pipeline
- [ ] `create_dry_run(&mut self)` renders without writing any file
- [ ] Missing template name returns `TemplateError::NotFound { name }`
- [ ] Engine compile/render failures return `TemplateError::Engine(TemplateEngineError)`
- [ ] Absolute path → `TemplateError::Artifact(WriteTargetError::Absolute(…))`
- [ ] Traversal path → `TemplateError::Artifact(WriteTargetError::Traversal(…))`
- [ ] Hidden-file path → `TemplateError::Artifact(WriteTargetError::Hidden(…))`
- [ ] Empty path → `TemplateError::Artifact(WriteTargetError::Empty)`
- [ ] Current-dir path → `TemplateError::Artifact(WriteTargetError::CurrentDir(…))`
- [ ] Already exists → `TemplateError::Artifact(WriteError::AlreadyExists { … })`
- [ ] No `minijinja` types in `TemplateService` method signatures or `TemplateError` public API
- [ ] No `unwrap()` or `panic!` in service code
- [ ] `#[allow(dead_code)]` removed from `artifact.rs` — pipeline has production caller
- [ ] `#[allow(dead_code)]` removed from `engine/mod.rs` — engine has production caller
- [ ] `TemplateRepositoryError::NotFoundByName` added
- [ ] `TemplateError::Artifact(TemplateArtifactError)` and `TemplateError::NotFound { name }` added
- [ ] `TemplateArtifactError` simplified to `Path(WriteTargetError)` + `Write(WriteError)` — no `TemplateWriteError` wrapper, no named validation variants
- [ ] Tests cover: `load()` orchestration and repository interactions, batch deletion, `create()` end-to-end, dry-run, missing template, engine failure, all 5 path rejection types, `AlreadyExists`
- [ ] `mise run test` passes

**Out of scope:**
- `TemplateDiagnostic` or any diagnostic framework
- Interactive prompts, declared inputs, or `inputs.*` namespace
- Multi-file packs or `TemplateArtifactSet`
- redb storage adapter
- CLI adapter (issue 08)

---

## Deferred from issue 06 (artifact write pipeline review)

The adversarial review of issue 06 intentionally left work for this issue, because it requires `TemplateService` to exist. Issue 06 ships the typestate pipeline behind a `FileWriter` port with a `WriteTarget` newtype, but with **no production caller** (`#[allow(dead_code)]` on `artifact.rs`). Issue 07 is where the pipeline gets wired and the allow is removed.

### Work this issue must do (in addition to the existing AC above)

1. **Refactor `TemplateService` to stateful composition with uniform generics.** Change from zero-field struct to `TemplateService<R, W, E>` holding `repository`, `writer`, `engine`, `config`. All three ports use the same generic-field pattern.
2. **Change `load()` to return `HashMap<TemplateName, Template>`** instead of `Vec<Template>`. Update existing tests.
3. **Change `create()` to `&mut self`** (engine `compile` needs `&mut self`). Look up target template from the map before touching the engine. Compile all templates from the map. The engine instance persists across `create()` calls — templates accumulate (re-compiling an already-registered name updates it in-place).
4. **Drive the collapsed pipeline.** `artifact.try_resolve_target(output_path)?.commit(&self.writer)?`.
5. **Add `TemplateError::NotFound { name }`** for missing template.
6. **Add `TemplateRepositoryError::NotFoundByName(TemplateName)`** for port-layer consistency.
7. **Add `TemplateError::Artifact(TemplateArtifactError)`** wrapping the simplified error.
8. **Simplify `TemplateArtifactError`** — drop `TemplateWriteError`, drop named validation variants, use `Path(WriteTargetError)` and `Write(WriteError)`.
9. **Wire batch deletion in `load()`.** After processing all discovered templates, iterate `_deleted_paths` and call `repository.delete_raw_template_view(path)?`.
10. **Extend path-rejection ACs** for hidden-file, empty, and current-dir output paths.
11. **`CreatedTemplatePath`** returns a `WriteTarget` or `&Path`, not a raw `PathBuf`.
12. **Remove `#[allow(dead_code)]`** from both `artifact.rs` and `engine/mod.rs`.

### Updated acceptance criteria (supersede/extend the list above)

- [ ] `TemplateService<R, W, E>` with `repository`, `writer`, `engine`, `config` fields
- [ ] `load(&self)` returns `HashMap<TemplateName, Template>`; existing tests updated
- [ ] `load()` wires batch deletion for orphaned raw template views
- [ ] `create(&mut self)` uses map for lookup + compilation, renders, drives artifact pipeline
- [ ] Engine injected as generic `E: TemplateEngine` at construction — same pattern as repo and writer
- [ ] All 5 path rejection types return `TemplateError::Artifact(WriteTargetError::…)`
- [ ] `AlreadyExists` returns `TemplateError::Artifact(WriteError::AlreadyExists { … })`
- [ ] Missing template returns `TemplateError::NotFound { name }`
- [ ] `TemplateArtifactError` simplified: `Path(WriteTargetError)` + `Write(WriteError)`; no `TemplateWriteError`, no named variants
- [ ] `TemplateRepositoryError::NotFoundByName(TemplateName)` added
- [ ] `#[allow(dead_code)]` removed from both `artifact.rs` and `engine/mod.rs`
- [ ] No `minijinja` types and no concrete `Writer` type in `TemplateService` signatures or `TemplateError` public API

---

## TDD Plan

### Design Decisions (from architectural review)

| Decision | Choice |
|---|---|
| Input struct | Single `CreateInput` with `dry_run: bool` (matches Indexer pattern) |
| Return type | `WriteTarget` — already validated, `as_path()` for CLI display |
| `Committed` state | Holds `WriteTarget`, exposed as `committed_path() -> &WriteTarget` |
| Batch deletion | Delete both `RawTemplateView` and `Template` aggregates |
| Dry run gate | `if !input.dry_run { commit }` |

### Target interface

```rust
pub struct CreateInput {
    pub name: TemplateName,
    pub output_path: String,
    pub context: HashMap<String, String>,
    pub dry_run: bool,
}

pub struct TemplateService<R, W, E> {
    repository: R,
    writer: W,
    engine: E,
    config: TemplateConfigSpec,
}

impl<R: ReadRepository + WriteRepository, W: FileWriter, E: TemplateEngine>
    TemplateService<R, W, E>
{
    pub fn new(
        repository: R,
        writer: W,
        engine: E,
        config: TemplateConfigSpec,
    ) -> Self;

    pub fn load(&self)
        -> Result<HashMap<TemplateName, Template>, TemplateError>;

    pub fn create(&mut self, input: CreateInput)
        -> Result<WriteTarget, TemplateError>;
}
```

### Vertical slices (red-green-refactor)

Each cycle: write one failing test → implement minimum code to pass → move on.

| # | Slice | RED test | GREEN work |
|---|---|---|---|
| **0a** | `NotFoundByName` error | `not_found_by_name_displays_name` | Add `NotFoundByName(TemplateName)` to `TemplateRepositoryError` |
| **0b** | `NotFound` + `Artifact` errors | `not_found_displays_name`, `artifact_variant_wraps` | Add `NotFound { name }` and `Artifact(TemplateArtifactError)` to `TemplateError` |
| **0c** | Simplify `TemplateArtifactError` | `path_variant_wraps_write_target_error`, `write_variant_wraps_write_error` | Replace named variants with `Path(WriteTargetError)` + `Write(WriteError)`. Remove `TemplateWriteError`. Update `map_path_error()`. Update `lib.rs` exports. |
| **1** | Tracer bullet: stateful struct + `new()` | `constructs_with_all_fields` | `TemplateService<R, W, E>` with fields, `new()` constructor. Trait bounds on `impl` block. |
| **2** | `load()` refactored to use fields, returns `HashMap` | `load_returns_empty_map_when_empty_dir` | Move `config`/`repository` to `self.*`. Return `HashMap<TemplateName, Template>`. Unused `W`+`E` params. |
| **3** | `load()` batch deletion | `load_deletes_orphaned_templates_and_views` | Replace TODO: for each `_deleted_path`, `find_template_id_by_path`, `delete_template`, `delete_raw_template_view`. |
| **4** | `Committed` holds `WriteTarget` | `committed_holds_write_target`, `commit_preserves_target` | Change `Committed` to `Committed(WriteTarget)`. `commit()` clones target into new state. |
| **5** | `create()` — not found | `create_returns_not_found_when_template_missing` | Define `CreateInput`. Lookup from map. Return `TemplateError::NotFound`. |
| **6** | `create()` — render + commit | `create_renders_and_commits_file` | `engine.compile` all map templates, `engine.render`, `try_resolve_target().commit()`. Extract `WriteTarget`. |
| **7** | `create()` — dry run | `dry_run_renders_without_writing` | Gate: `if !input.dry_run { commit }`. Return rendered string. |
| **8** | `create()` — engine errors | `create_propagates_engine_error` | Map `TemplateEngineError` → `TemplateError::Engine`. |
| **9** | `create()` — 5 path rejections | `rejects_absolute`, `traversal`, `hidden`, `empty`, `current_dir` | Each returns `TemplateError::Artifact(WriteTargetError::…)` |
| **10** | `create()` — AlreadyExists | `rejects_existing_file` | `WriteError::AlreadyExists` → `TemplateError::Artifact(WriteError::…)` |
| **11** | Dead code removal | No new clippy dead_code warnings | Remove `#[allow(dead_code)]` from `artifact.rs:9-14` and `engine/mod.rs:8` |
| **12** | Update existing tests | All `load` tests pass | Adapt for stateful struct, HashMap return, generic params |
| **13** | Quality gate | — | `mise run fmt && mise run lint && mise run test` |

### Files changed

| File | Changes |
|---|---|
| `crates/template/src/error.rs` | Add `NotFoundByName`, `NotFound`, `Artifact`. Simplify `TemplateArtifactError`. Remove `TemplateWriteError`. |
| `crates/template/src/lib.rs` | Remove `TemplateWriteError` export. |
| `crates/template/src/artifact.rs` | Remove `#[allow(dead_code)]`. `Committed` holds `WriteTarget`. Update `map_path_error` and `commit`. |
| `crates/template/src/engine/mod.rs` | Remove `#[allow(dead_code)]`. |
| `crates/template/src/service.rs` | Stateful struct with generics. `CreateInput`. `create()`. `load()` returns `HashMap`. Wire batch deletion. |

### Test coverage matrix

| Behavior | How verified |
|---|---|
| Stateful struct constructs | `new()` test |
| `load()` uses `self.*` fields | No `config`/`repository` params |
| `load()` returns `HashMap` | Return type assertion, downstream lookup |
| `load()` deletes orphaned views + aggregates | Repo read returns `None` after load |
| `create()` returns `NotFound` for missing name | Name not in map → error |
| `create()` compiles all templates | Mock engine compile count |
| `create()` renders + writes file | File exists on disk with content |
| `create()` dry-run renders only | File absent, string matches |
| `create()` propagates engine errors | Mock error → `TemplateError::Engine` |
| 5 path rejection types | Each returns typed `WriteTargetError` |
| `AlreadyExists` propagation | Pre-created file → typed error |
| Dead code allowances removed | Warning-free build |
| Existing `load()` tests pass | Adapted suite |
| `mise run test` passes | Full suite |

---

## Implementation Notes

Implemented on branch `feat/07-template-service` (worktree `.worktrees/07-template-service`). 2161/2161 workspace tests pass; `cargo clippy --all-targets --all-features --locked -- -D warnings` is clean.

### Public API surface (lib.rs re-exports)

```rust
pub use aggregate::{Template, TemplateBody, TemplateId, TemplateName};
pub use engine::{TemplateEngine, TemplateEngineError, mini_jinja::MiniJinjaEngine};
pub use error::{TemplateArtifactError, TemplateBodyError, TemplateError,
                TemplateNameError, TemplateRepositoryError};
pub use raw::RawTemplate;
pub use repository::{ReadRepository, Repository, WriteRepository};
pub use service::{CreateInput, CreateTemplateOutcome, RenderedTemplate, TemplateService};
pub use views::RawTemplateView;
```

### Deviations from the original TDD plan (approved during implementation)

1. **`create()` return type — `CreateTemplateOutcome` enum (not `WriteTarget`).** The plan's design table specified `Result<WriteTarget, TemplateError>` while slice 7 said "dry run returns rendered string" — these conflict for a single method. Approved resolution:
    ```rust
    #[non_exhaustive]
    pub enum CreateTemplateOutcome {
        Preview { output_path: WriteTarget, rendered: RenderedTemplate },
        Created { output_path: WriteTarget, bytes_written: u64 },
    }
    ```
    `create()` signature: `fn create(&mut self, templates: &HashMap<TemplateName, Template>, input: &CreateInput) -> Result<CreateTemplateOutcome, TemplateError>`. Note `templates` is passed in rather than stored, so a single `load()` map can drive many `create()` calls without re-querying the repository.

2. **`TemplateEngine::render` returns `String`, not `TemplateArtifact<Rendered>`.** Keeps the artifact typestate (`Rendered`/`TargetResolved`/`Committed`) `pub(crate)`. The service constructs the artifact internally via `TemplateArtifact::rendered(template.name().clone(), rendered_text)` before driving `try_resolve_target` → `commit`. Reduces public surface; engine port stays narrow.

3. **Repository gains two batch methods beyond the plan** (driven by review feedback after first pass):
    - `find_template_ids_by_paths(paths: &[PathKey]) -> Result<Vec<Option<TemplateId>>, _>` — same-length contract, mirrors `find_raw_template_views_by_paths`.
    - `delete_many_templates(paths: &[PathKey]) -> Result<(), _>` — single-transaction batch delete of both `Template` aggregates (resolved via path index) and matching `RawTemplateView` rows. Idempotent on missing entries.

   `load()`'s orphan cleanup collapses from a per-path loop of three round-trips (`find_template_id_by_path` + `delete_template` + `delete_raw_template_view`) to a single `delete_many_templates(&deleted_paths)` call.

4. **Renamed `ReadRepository::list_raw_template_view_paths` → `list_template_path_keys`.** The previous name leaked the storage-table identity into the port API. Returned values are vault-relative template path keys; one per persisted template.

5. **`Committed` state holds a `WriteTarget`.** Per slice 4 of the plan; exposed via `committed_path() -> &WriteTarget`. Service clones it into `CreateTemplateOutcome::Created.output_path`.

### Visibility decisions

| Type / item                                                                          | Visibility | Reason                                                                                                                  |
| ------------------------------------------------------------------------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------- |
| `TemplateService<R, W, E>`, `CreateInput`, `CreateTemplateOutcome`, `RenderedTemplate` | `pub`        | Public API surface — CLI / composition root feed in and consume                                                          |
| `TemplateEngine` trait, `MiniJinjaEngine`                                              | `pub`        | Engine port and adapter; composition root injects `MiniJinjaEngine`                                                       |
| `TemplateArtifact<S>`, `Rendered`, `TargetResolved`, `Committed`                         | `pub(crate)` | Typestate pipeline lives inside the service. Engine returns `String`; the service wraps internally.                       |
| `TemplateArtifact::{rendered, try_resolve_target, commit, target_path, into_content, content_len, committed_path}` | `pub(crate)` | Internal pipeline drivers                                                                                                |
| `TemplateArtifact::{template, content}` (on `Rendered`)                                 | `pub(crate)` + `#[cfg(test)]` | Test-only accessors                                                                                          |

### Production-code constraints honored

- No `unwrap()` / `expect()` / `panic!` / `unreachable!` outside `#[cfg(test)]`.
- No `minijinja` types appear in `TemplateService` signatures or `TemplateError` public API (enforced by the existing policy tests in `lib.rs`).
- Dry-run path never touches `self.writer`.
- `#[allow(dead_code)]` removed from `artifact.rs`, `engine/mod.rs`, `engine/mini_jinja.rs`.
- `#[cfg_attr(test, allow(dead_code))]` removed from `check_batch_existence` (had a production caller via `load()`).

### Test coverage delta

- 5 baseline files (`error.rs`, `lib.rs`, `artifact.rs`, `engine/mod.rs`, `service.rs`) plus 4 storage/engine sibling files modified (`storage/read.rs`, `storage/write.rs`, `storage/testing.rs`, `engine/mini_jinja.rs`, `repository.rs`).
- Net 220 unit tests in `trace-template` (up from 200 baseline).
- New service tests: `create::{returns_not_found_when_template_name_missing_from_map, renders_and_commits_file_to_disk, dry_run_returns_preview_without_writing_file, propagates_engine_error_when_template_source_is_invalid, rejects_{absolute,traversal,hidden,empty,current_dir}_output_path, rejects_existing_destination_file, returns_loaded_map_with_template_name_keys}`, plus `load::deletes_only_orphaned_paths_when_one_template_remains` and `construction::new_constructs_with_all_fields`.
- New repository tests: `find_template_ids_by_paths_{preserves_order_with_nones, empty_slice}` and `delete_many_templates_{removes_aggregate_and_view_for_each_path, is_idempotent_on_missing_paths, removes_view_when_aggregate_absent, empty_slice}`.

### Follow-up work (not done here)

- `RAW_TEMPLATE_VIEWS` should be a `UuidTable` to match `RAW_SCHEMA_VIEWS`. Out of scope for this issue — belongs in a dedicated storage-layer issue alongside any related migration concerns.

---

## Adversarial review (post-merge critique pass)

Independent reviewer ran the full six-dimension critique (rust-best-practices conformance, module/item docs, hexagonal design, test quality, verified-working) against commit `f3643dd5` on `feat/07-template-service`. All verification gates re-ran green:

| Command | Result |
| ------- | ------ |
| `mise run fmt` | clean |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 warnings |
| `cargo nextest run -p trace-template` | 220 / 220 |
| `cargo nextest run --workspace` | 2161 / 2161 |
| `mise run verify` | full quality gate green |

### Top-line risk: LOW–MEDIUM

Nothing breaks the build, tests, or hexagonal boundary in a way that should block merge. The findings below are improvement work — some architectural, most discipline.

### Findings table

| # | File:Line | Problem | Sev | Fix |
| - | --------- | ------- | --- | --- |
| F1 | `service.rs:500`, `service.rs:498-501` | `FsNode::Dir(_) \| _` wildcard defeats clippy's exhaustiveness check; when `FsNode` gains a third variant the compiler will not flag it. | Med | Drop the `\| _` arms; `FsNode::Dir(_) => None` is exhaustive. |
| F2 | `service.rs:159-203` `create()` | Compiles every entry in `templates` on every call — O(N) wasted compile work per render. | Med | Compile only the requested template; engine `add_template_owned` is idempotent on re-registration. See AR-5 in the solution plan. |
| F3 | `service.rs:164-168` | `TemplateError::NotFound { name: String }` allocates from `input.name.as_str().to_owned()` while `TemplateName` already carries the canonical text. `TemplateRepositoryError::NotFoundByName(TemplateName)` already uses the domain type — the two layers disagree. | Low | Change variant to `NotFound { name: TemplateName }`. |
| F4 | `service.rs:170-172` | First-error-wins compile loop discards subsequent broken templates' names. Acceptable but undocumented. | Low | Document the first-error-wins behaviour in `create`'s docstring. |
| F5 | `service.rs:225-226` vs `service.rs:292` | `Vec<PathKey>` is built twice from the same `scanned` slice (once in `load`, once in `check_batch_existence`). | Low | Build once in `load()` and pass into `check_batch_existence`, or return the path set from the helper. |
| F6 | `artifact.rs:136-138` | `u64::try_from(self.content.len()).unwrap_or(u64::MAX)` silently saturates `bytes_written` on a 128-bit target — worst of both worlds (false safety + bad telemetry). | Low | `#[expect(clippy::cast_possible_truncation, reason = "usize → u64 always widens on supported platforms")]` direct cast, or `unwrap_or_else(\|_\| unreachable!())`. |
| F7 | `lib.rs:50` | `#[allow(clippy::panic, ...)]` should be `#[expect(...)]` per Apollo Ch.2. | Low | `s/allow/expect/`. |
| F8 | `storage/tables.rs:21,32,43,54`; `storage/mod.rs:71,97` | Five `#[allow(dead_code, reason = "...")]` should be `#[expect(...)]`. | Low | `s/allow/expect/` at the listed sites. |
| F9 | `lib.rs:1` | `#![feature(trivial_bounds)]` at crate root — nightly-only, no `// CONTEXT:` comment explaining why. | Med | Add a `// CONTEXT:` comment naming the type/trait that needs it, with an ADR or issue link. |
| F10 | `lib.rs:7-14` `//!` doc | Module-level doc list omits the new public surface (`TemplateService`, `CreateInput`, `CreateTemplateOutcome`, `RenderedTemplate`, `TemplateArtifactError`, `TemplateEngine`, `MiniJinjaEngine`). | Med | Update the `//!` doc list. |
| F11 | `service.rs` `TemplateService` | No `# Examples` doc-test on `TemplateService::{new, load, create}`. Issue acceptance criteria called for `# Examples` on non-trivial public APIs. | Med | Add at least one no-run or `#[doc = include_str!(…)]` example anchoring the API. |
| F12 | `service.rs:118-137` `new()` | Doc mentions "compiled templates accumulate in the engine" without forward-linking `create()`/`load()`. | Low | Cross-link via ``[`Self::create`]``. |
| F13 | `repository.rs:212-215` `delete_many_templates` | Trait doc does not state the atomicity guarantee. The redb impl wraps in one transaction; the trait does not promise it, so callers can't rely on it across adapters. | Med | Add `# Atomicity` note: "all paths processed in a single consistent view; either all deletions commit or none do." |
| F14 | `repository.rs:80-83` `find_template_ids_by_paths` | Doc says "single transaction" but transactions are an adapter concept. | Low | Reword to talk about consistency/atomic view, not transactions. |
| F15 | `storage/write.rs:189-235` `delete_many_templates` (redb) | Opens four redb tables even when `paths.is_empty()`. | Low | Early-return `Ok(())` when `paths.is_empty()`. |
| F16 | `storage/write.rs:99-117` `delete_template` | Opens name+path index tables inside the `if let Some(...)` block. Unidiomatic for redb; clearer to open all tables once. | Low | Open `name_index` / `path_index` unconditionally; `remove` is a no-op on absent keys. |
| F17 | `service.rs:864-871` `mod construction` | `construction` is not in the canonical vocabulary (`docs/engineering/testing/unit-naming.md`). | Low | Rename `mod construction` → `mod constructor`. |
| F18 | `service.rs:1225-1264` `renders_and_commits_file_to_disk` | Test name combines two behaviours; body verifies rendering + disk write + byte count simultaneously. | Med | Split into `returns_created_outcome_when_template_renders` and `writes_rendered_content_to_disk_when_dry_run_is_false`. |
| F19 | `service.rs:1219,1247,1289,…` | `assert!(matches!(result, ...))` without diagnostic message — CI failures will show no actual value. | Med | Convert to `assert!(matches!(result, ...), "expected X, got: {result:?}")` throughout the create/artifact test modules. |
| F20 | `service.rs:497-510` `fixtures::scanned_metadata` | Final `.find(...).map(...).unwrap()` panics with no context if no matching markdown file exists. | Low | `.expect("scanned_metadata: no markdown file named '{file_name}' under temp/templates")`. |
| F22 | `lib.rs:62-72` policy test | Hand-maintained file list. New `*.rs` files under `src/` (e.g. `engine/error.rs`) are not in the list, so future leaks of forbidden imports into new files slip past the policy test. `engine/error.rs` legitimately imports `minijinja::Error` so its absence is correct today — but the discovery mechanism is fragile. | High | Replace the hand-maintained list with a `WalkDir` glob over `src/*.rs` excluding `engine/mini_jinja.rs` and `engine/error.rs`. |
| F23 | `engine/error.rs:11-30` `TemplateEngineError` | `minijinja::Error` appears as the `source:` field in two public variants. CONTEXT.md says "MiniJinja types do not appear in Template domain models, repositories, service requests, or service responses." Error variants are a grey area — arguably the engine boundary. | Med | Add an inline CONTEXT.md exception documenting that engine error is the documented boundary (recommended), or box the source as `Box<dyn std::error::Error + Send + Sync>`. |
| F24 | `service.rs:105-110` `TemplateService<R, W, E>` | Not `Send + Sync + 'static`-bound. Hexagonal architecture guide says service implementations should be axum-injectable. | Med | Acknowledged deferral: the bounds are axum/tower bounds, not hexagonal-architecture bounds. Add a doc comment recording the intentional deferral. Bounds get added at the composition root when a runtime needs them. |
| F25 | `service.rs:159-203` `create()` | Takes `templates: &HashMap<TemplateName, Template>` from the caller. Nothing prevents a hand-rolled map from bypassing the processor pipeline. | Med | See AR-1 below (resolved at the architecture level, not by a newtype wrapper). |
| F26 | `engine/mod.rs:43-47` `TemplateEngine::render` | Returns `String`. `RenderedTemplate` newtype lives one crate level up in `service.rs`. Port speaks a primitive while the domain has a newtype — worst of both. | Med | See AR-2. |
| F27 | `error.rs:24-57` `TemplateError` | 9 variants. `TemplatePathError` / `TemplateReadError` / `TemplateDirScanError` are thin newtype wrappers around `trace_fs::error::*`. ADR-017 endorses the wrapper-per-category pattern. | Low | Hold. Owner indicated the three categories surface in different code paths and a single `TemplateFsError` would mash unrelated cases together. ADR-017 stands. Revisit only if a future change makes the boundaries blur. |
| F28 | `service.rs:185-191` `dry_run` branch | Resolves the target but never checks destination collisions. `AlreadyExists` only surfaces at commit time, so dry-run silently passes for paths that would later fail. | Med | Document explicitly in `create`'s doc that `dry_run` does NOT check for existing destination files. Or add a `target_exists` check in dry-run mode. |

### Design critique

Two architectural smells dominate. First, the **load-then-create handoff** is the wrong shape. `load()` reads like "fetch from DB" but the method walks the FS, runs the processor pipeline, writes to the repo, garbage-collects orphans, and incidentally returns the result. That's an indexer, not a loader. The downstream consequence is that `create(&HashMap<TemplateName, Template>)` requires the caller to remember to invoke the indexer first, and nothing prevents a unit test or misguided caller from constructing a bare `HashMap` and skipping the processor pipeline entirely.

Second, the **engine port speaks in primitives**. `TemplateEngine::render -> Result<String, _>` while `RenderedTemplate` lives in `service.rs`. The newtype is decorative rather than load-bearing. Either delete it (a `String` field on the outcome is honest) or push it into the port. The current arrangement is the worst of both.

The other findings are correctness and discipline items: a hidden `\| _` wildcard that defeats clippy's exhaustiveness check, a `unwrap_or(u64::MAX)` that should commit to one strategy or the other, `#[allow]` instead of `#[expect]` in several places, and a hand-maintained file list in the policy test that lets new source files slip past the boundary check.

### What the review did NOT find

- No `unwrap()`, `expect()`, `panic!`, `unreachable!`, or `todo!` in production code paths (only inside `#[cfg(test)]` modules, the policy test, and the documented `#[expect(clippy::expect_used)]` lock-poison helpers).
- No `use minijinja` outside `engine/mini_jinja.rs` and `engine/error.rs` (the latter is the documented engine boundary).
- All public items modified in this commit have `///` doc comments. Missing items are `# Examples` and a few `# Errors` sections, not entirely-undocumented APIs.
- All eight `TemplateError` variants have an `#[error(...)]` Display and are reachable from the public API.
- Test fixtures live in `mod fixtures` per the standard; no assertions inside `fixtures`.
- `pretty_assertions::assert_eq` is used in every test module performing equality assertions in the new modules.
- The redb adapter's `delete_many_templates` correctly handles the orphan-view case (path-without-template), the orphan-template case (template-without-view via the path-index lookup), and the missing-everything case (silent skip).

---

## Solution plan (post-review architectural revisions)

These five revisions resolve F2, F23 partially, F24, F25, F26, F28 and reshape the service API. They are not a re-do of the implementation — they are the agreed shape for the next pass.

### AR-1 — Rename `load()` → `process_all()`; switch storage model to "DB is the source of truth"

`load()` is mis-named. It is an indexer: it scans the FS, runs the processor pipeline, persists `Template` aggregates and `RawTemplateView`s, and garbage-collects orphans. The returned `HashMap` is incidental.

**New shape:**

```rust
pub fn process_all(&mut self) -> Result<ProcessSummary, TemplateError>;
```

- Walks the FS, runs the processor, persists to the repo, deletes orphans.
- Does **not** return a `HashMap`. Returns a `ProcessSummary` (counts of created / updated / deleted / unchanged) for observability and tests.
- Acts as the safeguard that brings the DB into sync with the filesystem.

`create()` then fetches the requested template from the repo by name, eliminating the `&HashMap` parameter and the F25 footgun entirely.

**Safeguard requirement (per owner):** the DB must remain in perfect sync with the filesystem. Options to enforce this:

1. `create()` calls `process_all()` internally before fetching the requested template. Pro: zero chance of stale DB. Con: every render walks the FS — heavy.
2. `create()` trusts the caller to have called `process_all()` recently; the composition root invokes `process_all()` on a schedule (startup, file-watch events, explicit user "reindex" command). Pro: cheap renders. Con: requires a coordination policy outside the service.
3. Hybrid: `create()` calls a lighter `verify_path(name)` that just checks the single requested path is fresh (mtime / hash compare), and reprocesses only that one template if stale. Pro: cheap renders that self-heal. Con: more code in the service.

Recommend **(3) hybrid** for the next pass: foundation render latency stays bounded, but a stale single entry never reaches the engine. Implementation can lift the existing `TemplateProcessor` pipeline for the single-template path. (1) is the safest if (3) proves too complex; (2) is the wrong choice for a foundation tool that users will inevitably edit templates in mid-session.

### AR-2 — Move `RenderedTemplate` into the engine module

`RenderedTemplate` becomes `engine::RenderedTemplate` (or `engine/rendered.rs`). The port becomes:

```rust
pub trait TemplateEngine {
    fn compile(&mut self, template: &Template) -> Result<(), TemplateEngineError>;
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError>;
}
```

The service unwraps for `CreateTemplateOutcome::Preview { rendered }` and feeds `rendered.into_inner()` (or a new `as_str()` accessor) into `TemplateArtifact::rendered`. The newtype now carries information; it is load-bearing.

### AR-3 — Error handling: hold

`TemplatePathError`, `TemplateReadError`, `TemplateDirScanError` stay as separate wrappers. Owner correctly observed that the three categories surface in different code paths and a single collapsed `TemplateFsError` would mash unrelated cases together. ADR-017 stands. No change.

### AR-4 — `Send + Sync + 'static` bounds: deferred, documented

The hexagonal architecture guide's "service implementations must be `Clone + Send + Sync + 'static`" requirement is axum/tower-specific, not hexagonal-architecture-intrinsic. The template service has no async surface in the foundation, so the bounds are not required.

Add a single doc comment on `TemplateService<R, W, E>`:

```rust
/// Orchestrates template ingestion (process_all) and rendering-to-commit (create).
///
/// `TemplateService` is intentionally not bound `Send + Sync + 'static`. Those
/// bounds are runtime-specific (axum / tokio injection sites). When a runtime
/// needs them, the composition root adds them at the injection point. The
/// service is otherwise free of runtime assumptions.
```

No other change required.

### AR-5 — `create()` compiles only what it needs

Combined with AR-1, `create()`:

1. Fetches the requested `Template` from the repo by name (or returns `TemplateError::NotFound`).
2. Calls `self.engine.compile(&template)` for **only that template**. MiniJinja's `add_template_owned` is idempotent on re-registration, so repeat calls update the source in place without rebuilding state.
3. Renders, drives the artifact pipeline, commits (or previews when `dry_run`).

The `&HashMap<TemplateName, Template>` parameter is removed.

```rust
pub fn create(
    &mut self,
    input: &CreateInput,
) -> Result<CreateTemplateOutcome, TemplateError>;
```

Template inheritance / partials are explicitly out of scope for the foundation. When that lands, `create()` will need to compile the dependency set, and the engine will need a port method for declaring template dependencies. Until then, one-template-per-render is the contract.

### AR-6 — Document dry-run's collision-check behaviour (F28)

In `create()`'s docstring, add:

```
# Dry-run semantics
///
/// When `input.dry_run` is `true`, the service renders the template and
/// validates the output target path syntactically, but does **not** check
/// whether the destination file already exists on disk. Destination-collision
/// errors (`WriteError::AlreadyExists`) surface only at commit time. A
/// successful preview is therefore not a guarantee that a subsequent non-dry
/// `create()` call will succeed.
```

If a future change requires collision-aware previews, add a `target_exists` check that consults the writer's `exists` predicate (or equivalent) before returning `Preview`.

### Action list (ordered by severity)

| # | Action | Resolves | Severity |
| - | ------ | -------- | -------- |
| 1 | Replace hand-maintained policy test file list with a `WalkDir` glob. | F22 | High |
| 2 | Rename `load()` → `process_all()`; return `ProcessSummary`; remove `&HashMap` from `create()`. Decide on the (1)/(2)/(3) safeguard option. | AR-1, F25 | Med (arch) |
| 3 | Move `RenderedTemplate` into `engine`; port returns it. | AR-2, F26 | Med (arch) |
| 4 | Compile only the requested template in `create()`. | AR-5, F2 | Med |
| 5 | Add a `# Dry-run semantics` section to `create()`'s docstring. | AR-6, F28 | Med |
| 6 | Add `Send + Sync + 'static` deferral doc-comment on `TemplateService`. | AR-4, F24 | Med |
| 7 | Drop `\| _` wildcards in `FsNode` matches (`service.rs:500, 498-501`). | F1 | Med |
| 8 | Add `// CONTEXT:` comment justifying `#![feature(trivial_bounds)]`. | F9 | Med |
| 9 | Split `renders_and_commits_file_to_disk` into one-behaviour tests; add diagnostic messages to all `assert!(matches!(…))` calls. | F18, F19 | Med |
| 10 | Document atomicity guarantee in `delete_many_templates` trait doc. | F13 | Med |
| 11 | Update `lib.rs` `//!` to list the new public items. | F10 | Med |
| 12 | Add `# Examples` doc-tests to `TemplateService::{new, process_all, create}`. | F11 | Med |
| 13 | Add inline CONTEXT.md exception for `engine/error.rs` `minijinja::Error` source field. | F23 | Med |
| 14 | Change `TemplateError::NotFound { name: String }` → `{ name: TemplateName }`. | F3 | Low |
| 15 | Replace `u64::try_from(...).unwrap_or(u64::MAX)` with `#[expect(clippy::cast_possible_truncation)]` direct cast. | F6 | Low |
| 16 | Convert `#[allow(...)]` to `#[expect(...)]` (4 sites in `tables.rs` / `storage/mod.rs`, 1 in `lib.rs`). | F7, F8 | Low |
| 17 | Early-return when `paths.is_empty()` in redb `delete_many_templates`; open redb tables unconditionally in `delete_template`. | F15, F16 | Low |
| 18 | Rename `mod construction` → `mod constructor`. | F17 | Low |
| 19 | Add context message to `fixtures::scanned_metadata`'s final `.unwrap()`. | F20 | Low |
| 20 | Document first-error-wins compile-loop behaviour, single-Vec construction, cross-links, trait wording. | F4, F5, F12, F14 | Low |

---

## Implementation log (review-fix pass)

Implemented on `feat/07-template-service` in worktree
`.worktrees/07-template-service`, base commit `d3641d2e`. Five
Conventional-Commit batches; every commit passed `mise run fmt`,
`cargo clippy --all-targets --all-features --locked -- -D warnings`,
`cargo nextest run -p trace-template`, `cargo nextest run --workspace`,
and `mise run verify`. Test counts grew from 220/220 (trace-template) and
2161/2161 (workspace) at `f3643dd5` to 225/225 and 2166/2166 — a net
increase, no regressions.

AR-1 freshness safeguard: owner selected **option 3** (per-template
`verify_path`), implemented as `TemplateService::verify_path(&TemplateName)`.

| # | Commit | Note |
| - | ------ | ---- |
| 1 | `982fa0bf` | Policy tests discover `src/*.rs` via `WalkDir`, excluding `engine/mini_jinja.rs` + `engine/error.rs`; verified it flags an injected import. walkdir added as dev-dep; DEPENDENCIES.md updated. |
| 2 | `d1bcbf51` | `load()` → `process_all()` returning `ProcessSummary`; `create()` drops `&HashMap`, fetches by name, adds `verify_path` (AR-1 opt 3). |
| 3 | `d1bcbf51` | `RenderedTemplate` moved to `engine::rendered`; `TemplateEngine::render` returns it. |
| 4 | `d1bcbf51` | `create()` compiles only the requested template (no batch loop). |
| 5 | `d1bcbf51` | `# Dry-run semantics` section added to `create()` docstring. |
| 6 | `d1bcbf51` | `Send + Sync + 'static` deferral doc-comment added to `TemplateService`. |
| 7 | `220e8dcf` | Dropped `FsNode::Dir(_) \| _` collapse. `FsNode` is `#[non_exhaustive]`, so a single commented `_ => None` is required; F1's "`Dir(_) => None` is exhaustive" premise does not hold for a foreign `#[non_exhaustive]` enum. |
| 8 | `220e8dcf` | **Deviation:** `#![feature(trivial_bounds)]` was dead across lib/test/all-features builds, so it was *removed* rather than documented — a CONTEXT comment justifying a feature nothing needs would itself be misleading (Apollo Ch.8). |
| 9 | `5565385c` | Split `renders_and_commits_file_to_disk` into `returns_created_outcome_when_template_renders` + `writes_rendered_content_to_disk_when_dry_run_is_false`; added diagnostics to bare `assert!(matches!())` calls. |
| 10 | `a0141a54` | `# Atomicity` section added to `delete_many_templates` trait doc. |
| 11 | `d1bcbf51` | `lib.rs` `//!` updated to list the new public surface. |
| 12 | `d1bcbf51` | `# Examples` doc-tests added to `new`, `process_all`, `create`. |
| 13 | `a0141a54` | Inline `CONTEXT:` comment on `TemplateEngineError` documenting the engine error boundary exception. |
| 14 | `d1bcbf51` | `TemplateError::NotFound { name }` changed `String` → `TemplateName`. |
| 15 | `220e8dcf` | **Deviation:** the crate forbids bare `as` (`as_conversions`) and `expect_used`/`unreachable` in production, and `cast_possible_truncation` does not fire on a widening cast. `content_len` now uses `self.content.len() as u64` under `#[expect(clippy::as_conversions)]` — lossless, non-saturating, lint-clean. |
| 16 | `220e8dcf` | `lib.rs` policy `#[allow(clippy::panic)]` → `#[expect]`. **Deviation:** the six redb storage `#[allow(dead_code)]` sites were *not* converted — those items are dead only in the lib profile and live in the test profile, so `#[expect]` is unfulfillable across cfgs; retained `#[allow(..., reason=...)]` per Apollo Ch.2 §2.4 (documented false-positive fallback). |
| 17 | `a0141a54` | redb `delete_many_templates` early-returns on empty input; `delete_template` opens all tables unconditionally. Added empty-slice + single-path tests. |
| 18 | `5565385c` | `mod construction` → `mod constructor`. |
| 19 | `5565385c` | `fixtures::scanned_metadata` terminal lookup uses a contextual `.expect(...)` (static message; the crate forbids `panic!`/`unreachable!` and interpolating `expect`). |
| 20 | `220e8dcf` / `d1bcbf51` | F4 first-error-wins is moot — `create()` no longer loops (Action 4). F5 single-`Vec` build: `check_batch_existence` returns the path set (`BatchExistence` alias) for reuse. F12 cross-links added on `new`. F14 `find_template_ids_by_paths` doc reworded to atomic-view, not transactions. |
