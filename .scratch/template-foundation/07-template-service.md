---
title: 07-template-service
category: enhancement
label: ready-for-agent
status: open
branch:
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
