# Template Service

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement `TemplateService` — the use-case orchestrator that wires together the processor, engine, and artifact pipeline into two primary operations:

**`load()`** — Orchestrates the `TemplateProcessor` pipeline to ingest/index templates from the configured source into the repository. Ensures templates are available for subsequent `create()` calls.

**`create()`** — Orchestrates a full render-to-commit flow:
1. Look up the named `Template` from the repository
2. Build a `MiniJinjaEngine` from all persisted templates (implicit compile-time validation — if the source is broken, `compile` catches it here)
3. Call `engine.render(template, context)` with the supplied `HashMap<String, String>` context
4. Drive the `TemplateArtifact` typestate pipeline: `Rendered → TargetResolved → ReadyToCommit → Committed`
5. Return the created vault-relative path on success

A `--dry-run` variant of `create()` performs steps 1–3 only (renders but does not commit) and returns the rendered string.

`TemplateService` owns: lookup, validation workflow, rendering orchestration, target resolution, conflict checks, and commit orchestration. None of that logic leaks into the engine port or the artifact states.

Error types:
- `TemplateError` — primary use-case error surface (missing template, load failure, engine failure, path validation failure, write failure)
- `TemplateEngineError` embedded as a variant or cause within `TemplateError`
- No `TemplateDiagnostic` — rely on well-written Rust error chains

## Acceptance criteria

- [ ] `TemplateService` exposes `load()` and `create()` (and a dry-run variant)
- [ ] `create()` returns the created vault-relative path on success
- [ ] Dry-run returns the rendered string without writing any file
- [ ] Missing template name returns a `TemplateError` (not a panic or unwrap)
- [ ] Engine compile/render failures surface as `TemplateError` wrapping `TemplateEngineError`
- [ ] Absolute path and traversal path rejections surface as `TemplateError`
- [ ] Destination already exists surfaces as `TemplateError`
- [ ] No MiniJinja types appear in `TemplateService` method signatures or `TemplateError` variants
- [ ] No `unwrap()` or `panic!` in service code
- [ ] Tests cover: `load()` orchestration with repository interactions, `create()` success path end-to-end (rendered content written, correct path returned), dry-run returns rendered string without file creation, missing template error, engine failure propagation, target path validation errors, `AlreadyExists` error propagation

## Blocked by

- `issue-03-repository-traits.md`
- `issue-04-processor-pipeline.md`
- `issue-05-engine-port-adapter.md`
- `issue-06-artifact-write-pipeline.md`
