---
title: 05-engine-port-adapter
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-11
date_completed:
---

# Template Engine Port + MiniJinja Adapter

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define the `TemplateEngine` port trait, the initial rendered artifact state, and implement `MiniJinjaEngine` as the MiniJinja-backed adapter.

**Port trait (`TemplateEngine`):**

Two methods, both accepting Lithos domain types and exposing no MiniJinja types:
- `compile(&mut self, template: &Template)` — checks engine-level source validity and loads the supplied `Template` source into the configured engine. This is adapter-local engine state mutation, not service orchestration. Returns `Result<(), TemplateEngineError>`.
- `render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>)` — renders an already-supplied `Template` with an already-supplied flat context. Returns `Result<TemplateArtifact<Rendered>, TemplateEngineError>`.

The port is Lithos-shaped: no MiniJinja types in method signatures. No `Clone + Send + Sync + 'static` bounds on the trait — this is a CLI tool, not a web server, and runtime injection pressure should drive future bounds.

**Adapter (`MiniJinjaEngine`):**

Encapsulates MiniJinja setup and source registration behind the `TemplateEngine` port. Holds `env: minijinja::Environment<'static>` as a plain owned field — no `Arc`, `Mutex`, or `RwLock` needed.

`MiniJinjaEngine::configured()` creates an environment configured for foundation rendering. `compile(&mut self, template)` registers/checks the supplied template using `add_template_owned(template.name().as_ref().to_owned(), template.body().as_ref().to_owned())`. This keeps MiniJinja lifetime and source-registration mechanics inside the adapter while allowing `TemplateService` to pass domain `Template` values through the port.

Configuration:
- `UndefinedBehavior::Strict` (render fails on undefined variables)
- `AutoEscape::None` (no Markdown auto-escape)
- MiniJinja built-ins only (no custom extensions)

`render` uses `template.name().as_ref()` as the lookup key into the environment and does not add or replace `template.body()` at render time. Callers that need strict compile-before-render semantics should call `compile` first; `render` may surface a load/render error if the template was not compiled/loaded.

**Rendered artifact (`TemplateArtifact<Rendered>`):**

Issue 05 owns the initial rendered artifact type because rendering produces domain state, not a raw string. `TemplateArtifact<Rendered>` carries at minimum the source `TemplateName` and rendered content. It is not writeable yet and has no resolved output target.

Issue 06 owns only the post-render write pipeline transitions: `Rendered -> TargetResolved -> ReadyToCommit -> Committed`.

`TemplateEngineError` preserves the source `minijinja::Error` in its error chain without exposing `minijinja::Error` in trait method signatures. MiniJinja source errors may be stored privately inside the engine adapter/error internals; public API should remain Lithos-shaped.

## Acceptance criteria

- [ ] `TemplateEngine` trait is defined with `compile(&mut self, template: &Template)` and `render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>` (exact signatures may vary slightly per implementation pressure, but MiniJinja types must not appear)
- [ ] `TemplateEngine` has no `Clone + Send + Sync + 'static` bounds
- [ ] `TemplateArtifact<Rendered>` is defined as the rendered-output domain state with shared data needed by issue 06 (`TemplateName`, rendered content)
- [ ] `MiniJinjaEngine::configured()` creates a strict/no-escape MiniJinja environment and keeps `Environment<'static>` behind the adapter boundary
- [ ] `compile(&mut self, template: &Template)` registers/checks the supplied source using `template.name()` and `template.body()` and does not perform repository/filesystem lookup, target resolution, conflict checks, CLI context assembly, or artifact commit behavior
- [ ] `render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>)` looks up the template by `template.name()` and does not add or replace `template.body()` at render time
- [ ] `MiniJinjaEngine` uses `UndefinedBehavior::Strict`
- [ ] `MiniJinjaEngine` uses `AutoEscape::None`
- [ ] `TemplateEngineError` preserves `minijinja::Error` as its source via `std::error::Error::source()`
- [ ] MiniJinja types (`Environment`, `Value`, etc.) do not appear in the `TemplateEngine` trait or `TemplateEngineError` public API surface
- [ ] Tests cover: compile success, compile failure (invalid syntax) with preserved source error, render success with variable substitution, render failure (undefined variable under strict mode), no auto-escape of Markdown characters, render returns `TemplateArtifact<Rendered>` with template name and content, and render does not re-register/replace source at render time

## Blocked by

- None. `01-domain-models.md` is completed and provides the required `Template`, `TemplateName`, and `TemplateBody` types.

---

> *This was generated by AI during triage.*

## Agent Brief

**Category:** enhancement
**Summary:** Define the `TemplateEngine` port trait, rendered artifact state, and implement `MiniJinjaEngine` as the MiniJinja adapter with strict/no-escape config

**Current behavior:**
No template rendering port or adapter exists. There is no Lithos-shaped API surface for compile-checking or rendering templates. `minijinja` is already available as a `lithos-core` dependency, but the Template context does not yet expose a rendering boundary or adapter implementation.

**Desired behavior:**
Four artifacts are produced:

**1. `TemplateEngine` trait** (lives in `lithos-core/src/template/engine.rs` or similar):
- `fn compile(&mut self, template: &Template) -> Result<(), TemplateEngineError>` — checks engine-level source validity and loads the supplied template source into the configured engine; does not do repository lookup
- `fn render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>` — renders the loaded template selected by `template.name()` with the supplied context and returns rendered domain state
- `compile` intentionally accepts a `Template`. The service owns lookup; the engine owns engine-level source loading/checking for the supplied domain aggregate.
- No `Clone + Send + Sync + 'static` bounds on the trait itself
- No `minijinja` types in any method signature

**2. `MiniJinjaEngine` struct** (lives in an adapter module, e.g. `lithos-core/src/template/engine/minijinja.rs`):
- Holds `env: minijinja::Environment<'static>` as a plain owned field; no `Arc`, `Mutex`, or `RwLock`
- `configured()` constructs the environment with foundation MiniJinja settings
- `compile(&mut self, template)` registers/checks the supplied template with `add_template_owned`, using `template.name().as_ref()` as the MiniJinja key and `template.body().as_ref()` as the source
- `compile` maps MiniJinja parse/load failures into `TemplateEngineError::Compile` (or equivalent opaque compile failure)
- Configured with `UndefinedBehavior::Strict` and `AutoEscape::None`
- MiniJinja built-ins only; no custom filters/globals/extensions
- `render` calls `env.get_template(template.name().as_ref())` + `tmpl.render(context)`; it returns `TemplateArtifact<Rendered>` containing the template name and rendered content
- Implements `TemplateEngine`

**3. `TemplateArtifact<Rendered>` type**:
- Lives with the template engine/artifact boundary (e.g. `lithos-core/src/template/artifact.rs` or `template/engine.rs` until issue 06 expands it)
- Generic `TemplateArtifact<State>` shape starts here with `Rendered` as the only state required by issue 05
- Carries rendered content and the source `TemplateName`
- Has no target path and cannot commit to disk in issue 05
- Issue 06 extends this type with `TargetResolved`, `ReadyToCommit`, and `Committed` transitions

**4. `TemplateEngineError` type**:
- Covers at minimum compile and render failure kinds.
- `minijinja::Error` is the source in the error chain but must not appear in trait method signatures. Prefer keeping MiniJinja source fields private or otherwise confined to the adapter/error internals so the port remains Lithos-shaped.
- Implements `std::error::Error` with `source()` returning the inner `minijinja::Error`

**Key interfaces:**
- `TemplateEngine` — the port trait; must be import-clean of `minijinja`
- `MiniJinjaEngine` — the adapter; `minijinja` imports are confined to its module
- `TemplateEngineError` — the error type; `minijinja::Error` appears only as a private/internal source field
- `TemplateArtifact<Rendered>` — the domain render output returned by the engine before issue 06 resolves a target or writes to disk
- `Template` from issue-01 — `compile` uses `TemplateName` and `TemplateBody`; `render` uses `TemplateName` as the lookup key and does not mutate the registered source body
- Architecture test (issue-09) will verify that `minijinja` does not appear outside the adapter module — design the module boundary accordingly

**Acceptance criteria:**
- [ ] `TemplateEngine` trait compiles with `compile(&mut self, template: &Template)` and `render(&self, template: &Template, context: &serde_json::Map<String, serde_json::Value>) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>` and no `minijinja` types in signatures
- [ ] `TemplateEngine` has no `Clone + Send + Sync + 'static` bounds on the trait definition
- [ ] `TemplateArtifact<Rendered>` exists and carries the source `TemplateName` plus rendered content
- [ ] `MiniJinjaEngine::configured()` constructs a strict/no-escape environment with MiniJinja contained behind the adapter boundary
- [ ] `compile` registers/checks the supplied template source using `add_template_owned`; it takes `&mut self`, accepts a `Template`, and does not perform repository/filesystem lookup
- [ ] `render` looks up the source by `template.name()` and does not add or replace sources during rendering
- [ ] `MiniJinjaEngine` uses `UndefinedBehavior::Strict`
- [ ] `MiniJinjaEngine` uses `AutoEscape::None`
- [ ] `TemplateEngineError` preserves `minijinja::Error` as the error source (accessible via `std::error::Error::source()`)
- [ ] `minijinja` types do not appear in `TemplateEngine` method signatures, `TemplateEngineError` public fields, or public constructors
- [ ] Compile success test passes
- [ ] Compile failure test: invalid Jinja syntax returns `TemplateEngineError` with `minijinja::Error` accessible as source
- [ ] Render success test: `{{ name }}` rendered with `{"name": "Alice"}` produces a `TemplateArtifact<Rendered>` whose content is `"Alice"`
- [ ] Render failure test: undefined variable under strict mode returns `TemplateEngineError`
- [ ] No auto-escape test: Markdown characters (`*`, `_`, `#`, `[`) are not escaped in rendered output
- [ ] Render does not re-register or replace `template.body()` at render time
- [ ] `mise run test` passes

**Out of scope:**
- Custom MiniJinja filters, globals, or extension modules
- `TemplateExtension` or `ExtensionRegistry`
- Repository access or template lookup inside the engine (the service owns that)
- `Clone + Send + Sync + 'static` bounds
- Target resolution, conflict checks, file commit, CLI, and service orchestration

---

> *This was generated by AI during triage.*

## Triage Review

**Recommendation:** keep `category: enhancement` and `label: ready-for-agent`.

**Codebase context:**
- The Template context already exists with domain models, repository traits, processor, storage adapter, and ingestion-oriented `TemplateService`.
- `TemplateService` currently leaves engine compilation/rendering outside ingestion, matching the requested engine boundary.
- `minijinja` is already a workspace and `lithos-core` dependency, but there is no engine adapter module yet.
- Existing policy tests currently forbid rendering-engine imports in core template domain files; this issue should add the adapter boundary without weakening the domain/service API invariant.

**Rust/API review findings:**
- The issue was realigned with `component-model.md` and the hexagonal architecture guide: the port accepts domain `Template` values, hides MiniJinja mechanics in the adapter, and returns a domain `TemplateArtifact<Rendered>` instead of a primitive string.
- The earlier name-based/build-once issue shape was rejected because it forced source registration outside the port flow and weakened the domain render boundary.
- `TemplateArtifact<Rendered>` belongs in this issue because rendering produces domain state. Issue 06 should extend that type with target resolution, conflict checks, and commit transitions.
- `compile(&mut self, template: &Template)` intentionally mutates adapter-local engine state via MiniJinja source registration/checking; this is not service orchestration and does not violate the engine boundary.
- `render(&Template, ...)` explicitly uses `template.name()` as the environment lookup key and does not re-register `template.body()` during rendering.
- `serde_json::Map<String, serde_json::Value>` remains the foundation context type from `component-model.md`; the CLI can still assemble it from flat `--var key=value` inputs.
- No `Arc`, `Mutex`, or `RwLock` should be introduced unless implementation pressure proves shared mutable ownership is necessary.

**Risk assessment:** LOW. This is a greenfield adapter/port addition with clear domain boundaries and focused tests. The main risk is accidentally leaking MiniJinja through public error types or expanding the engine into service/repository responsibilities.
