# Template Engine Port + MiniJinja Adapter

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define the `TemplateEngine` port trait and implement `MiniJinjaEngine` as its build-once adapter.

**Port trait (`TemplateEngine`):**

Two methods, both taking `&self`:
- `compile` — checks engine-level source validity by calling `env.get_template(name)` against sources already loaded at construction. Does not mutate the engine. Returns `Result<(), TemplateEngineError>`.
- `render` — renders an already-supplied `Template` with an already-supplied `HashMap<String, String>` context. Returns `Result<String, TemplateEngineError>`.

The port is Lithos-shaped: no MiniJinja types in method signatures. No `Clone + Send + Sync + 'static` bounds on the trait — this is a CLI tool, not a web server.

**Adapter (`MiniJinjaEngine`):**

Build-once: all template sources are registered at construction via `set_loader` or `add_template_owned`. The engine is never mutated after construction. Holds `env: minijinja::Environment<'static>` as a plain owned field — no `Arc`, `Mutex`, or `RwLock` needed (`Environment<'static>` is already `Send + Sync`).

Configuration:
- `UndefinedBehavior::Strict` (render fails on undefined variables)
- `AutoEscape::None` (no Markdown auto-escape)
- MiniJinja built-ins only (no custom extensions)

The adapter converts `HashMap<String, String>` context values to `minijinja::Value` internally, keeping all `minijinja` types out of method signatures.

`TemplateEngineError` preserves the source `minijinja::Error` in its error chain.

## Acceptance criteria

- [ ] `TemplateEngine` trait is defined with `compile(&self, name: &str)` and `render(&self, template: &Template, context: &HashMap<String, String>)` (exact signatures may vary slightly per implementation pressure, but MiniJinja types must not appear)
- [ ] `TemplateEngine` has no `Clone + Send + Sync + 'static` bounds
- [ ] `MiniJinjaEngine` is constructed with a set of template sources and is never mutated after construction
- [ ] `MiniJinjaEngine` uses `UndefinedBehavior::Strict`
- [ ] `MiniJinjaEngine` uses `AutoEscape::None`
- [ ] `TemplateEngineError` preserves `minijinja::Error` as its source
- [ ] MiniJinja types (`Environment`, `Value`, etc.) do not appear in the `TemplateEngine` trait or `TemplateEngineError` public API surface
- [ ] Tests cover: compile success, compile failure (invalid syntax) with preserved source error, render success with variable substitution, render failure (undefined variable under strict mode), no auto-escape of Markdown characters, build-once source registration (all templates available after construction without mutation)

## Blocked by

- `issue-01-domain-models.md`
