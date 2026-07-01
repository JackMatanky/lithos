---
name: template-service-send-at-composition-root
status: accepted
supersedes: []
date_proposed: 2026-07-01
date_decided: 2026-07-01
date_implemented: TBD
stakeholders: [Jack (Developer), Architecture Team]
---

# ADR 0002: TemplateService Send Bounds at the Composition Root

## Context

The indexer → context-processor integration (`.scratch/filesystem-indexer/integration/PRD.md`) spawns one OS thread per downstream context service in `crates/app/src/sync.rs::run_sync`, each running `service.sync(receiver)`. `std::thread::spawn` requires the moved closure — and therefore the service — to be `Send + 'static`.

`crates/template/src/service.rs:132-136` documents `TemplateService<R, W, E>` as *intentionally not* bound `Send + Sync + 'static`, on the grounds that those bounds are "runtime-specific (axum / tokio injection sites), not hexagonal-architecture-intrinsic." That decision predates the integration work, which now needs template to cross a thread boundary exactly like `NoteService` and `SchemaService`.

**Evaluation Criteria**:

1. **Uniformity**: the integration's "one OS thread per service" orchestration should hold for all three contexts, not carve out template as a special case.
2. **Hexagonal integrity**: the service must remain generic over its ports (`R: ReadRepository + WriteRepository`, `W: FileWriter`, `E: TemplateEngine`) so tests can inject in-memory doubles.
3. **No intrinsic non-`Send` state**: adding thread bounds must not paper over a genuine non-thread-safe field.

**Forces at Play**:

- The concrete production triple is `TemplateService<RedbRepository, Writer, MiniJinjaEngine>`.
- Every concrete port is `Send + Sync`: `RedbRepository` wraps `Arc<Store>`; `Writer` is a `PathBuf` (`crates/fs/src/writer.rs:34`); `MiniJinjaEngine` wraps `minijinja::Environment<'static>` (`crates/template/src/engine/mini_jinja.rs:20`), which is `Send + Sync`. `TemplateConfigSpec` and `Arc<AppConfig>` are likewise `Send + Sync`.
- The documented "no marker bounds" stance was a deliberate omission, not a reflection of any intrinsic constraint — there is no `Rc`, `RefCell`, or thread-local anywhere in the service's field set.

## Decision

Template runs on its own consumer thread, matching note and schema. The `Send + 'static` bounds are re-added at the **composition root** — the spawn site in `run_sync` — by pinning the concrete `TemplateService<RedbRepository, Writer, MiniJinjaEngine>` before `thread::spawn`. The generic `sync` method is **not** rewritten to a concrete type and carries no added marker bounds:

```rust
impl<R, W, E> TemplateService<R, W, E>
where
    R: ReadRepository + WriteRepository,
    W: FileWriter,
    E: TemplateEngine,
{
    pub fn sync(&self, rx: Receiver<IndexEvent>) -> Result<TemplateSyncReport, TemplateSyncError> { /* ... */ }
}
```

The `Send + 'static` requirement is satisfied structurally by the concrete port types at the spawn site. The doc comment at `service.rs:132-136` is updated to record that thread bounds are supplied by the composition root rather than the service definition, superseding its "intentionally not `Send + Sync`" phrasing.

## Alternatives Considered

### Alternative 1: Run template inline on the orchestrator thread

Process template's flush inline in `run_sync` after joining the other consumers, so no `Send` bound is needed and `service.rs:132-136` stays verbatim.

**Why Rejected**: Breaks the uniform "one OS thread per service" claim, serialises template behind the other contexts, and forgoes concurrency for no real gain — nothing in the concrete triple is non-`Send`, so the constraint it preserves is imaginary.

### Alternative 2: Erase the generics to a concrete `TemplateService` internally

Add a concrete type alias and make `sync` non-generic on the fixed triple.

**Why Rejected**: Discards the hexagonal testability the generics exist for (in-memory doubles for `R`/`W`/`E`) to solve a problem that only exists at the spawn site, not inside the service.

### Alternative 3: Wrap the service in a `Send` shim

Box or `Arc`-wrap the service at the boundary to launder thread-safety.

**Why Rejected**: Adds indirection to compensate for marker bounds that the concrete types already satisfy directly. Pure ceremony.

## Technical Validation

The Send chain was verified against source:

| Port / field | Concrete type | `Send + Sync` |
| --- | --- | --- |
| `R` | `RedbRepository` (`Arc<Store>`) | ✓ |
| `W` | `Writer` (`PathBuf`, `crates/fs/src/writer.rs:34`) | ✓ |
| `E` | `MiniJinjaEngine` (`minijinja::Environment<'static>`, `crates/template/src/engine/mini_jinja.rs:20`) | ✓ |
| `config` | `TemplateConfigSpec` (validated value) | ✓ |

No field carries `Rc`, `RefCell`, `Cell`, or a thread-local. `process_all` (`crates/template/src/service.rs:344`) is `&self` and touches only `repository` + `config` during ingestion — the engine is used solely in the render path — so `sync(&self, rx)` is the correct receiver shape and forces no `&mut`. The bounds therefore compile against the concrete triple with no state change to the service.

## Consequences

### Positive

1. Uniform thread-per-service orchestration across note, schema, and template.
2. The service stays generic and hexagonally testable; bounds live only where they are needed.
3. The correction is a doc/composition-root change, not a rewrite of the service body.

### Negative

1. Reverses a previously documented decision; the `service.rs:132-136` comment must be updated so future readers don't re-remove the bounds.

### Risks

1. **Risk**: a future non-`Send` port implementation is injected and the composition root fails to compile.
   - **Mitigation**: the failure is a compile error at the spawn site, caught immediately — the desired fail-closed behaviour.
