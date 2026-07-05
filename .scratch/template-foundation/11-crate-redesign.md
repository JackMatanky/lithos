---
title: 11-crate-redesign
category: refactor
label: ready-for-review
status: completed
branch: refactor/template-crate-redesign
merge_commit: 7c79e9a9
date_created: 2026-07-04
date_completed: 2026-07-05
---

# Template Crate Redesign

Status: completed

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Strip the template crate from ~8,000 LOC (17 files) to ~500 LOC (6 files + 1
module dir) by removing the database-backed repository, processor pipeline,
views, raw template DTO, and Template aggregate. MiniJinja becomes the sole
rendering runtime — owning loading, compilation, caching, and dependency
resolution.

### Architectural changes

- **Before:** Repository-backed with `TemplateId` (UUID), `Template` aggregate,
  `TemplateProcessor` ingestion pipeline, `RawTemplate`/`RawTemplateView` DTOs,
  `RedbRepository` persistence, `InMemoryRepository` test double, typestate
  artifact pipeline (`Rendered → TargetResolved → ReadyToCommit → Committed`),
  and a 1,700-line `TemplateService`.

- **After:** Filesystem-backed. The configured template directory is the source
  of truth. `MiniJinjaEngine` owns a `Environment<'static>` with `path_loader`.
  `TemplateService` orchestrates rendering, artifact validation, and file
  writing. No database, no caching layer, no UUIDs, no content hashing, no
  typestate pipeline.

### File map — after

| File                                              | Responsibility                     | LOC  |
| ------------------------------------------------- | --------------------------------- | ---- |
| `crates/template/Cargo.toml`                      | Dependencies (trimmed to 4 deps)  | 15   |
| `crates/template/src/lib.rs`                      | Crate-level doc + public exports  | 30   |
| `crates/template/src/name.rs`                     | `TemplateName` newtype             | 60   |
| `crates/template/src/error.rs`                    | Simplified error types             | 100  |
| `crates/template/src/engine.rs`                   | `TemplateEngine` trait             | 25   |
| `crates/template/src/engine/rendered.rs`          | `RenderedTemplate` newtype         | 78   |
| `crates/template/src/engine/mini_jinja.rs`        | `MiniJinjaEngine` adapter          | 140  |
| `crates/template/src/service.rs`                  | `TemplateService` + artifact fns  | 325  |

### Modified consumers

| File                              | Change                                        |
| --------------------------------- | --------------------------------------------- |
| `crates/app/src/template.rs`        | Remove DB/repo wiring, simplify to engine+writer |
| `crates/app/src/error.rs`           | Remove `TemplateRepositoryError` handling       |
| `crates/cli/src/commands/template.rs` | Remove Repository/Scan error mapping           |
| `crates/cli/src/error.rs`           | Remove `TemplateRepositoryError` test references |
| `Cargo.toml`                        | Remove unused workspace deps                   |

### Deleted files (11)

`aggregate.rs`, `processor.rs`, `views.rs`, `raw.rs`, `storage.rs`,
`storage/core.rs`, `storage/read.rs`, `storage/write.rs`, `storage/tables.rs`,
`storage/testing.rs`, `CONTEXT.md`.

## Acceptance criteria

- [x] Template crate compiles standalone with no warnings
- [x] All 25 `traces-template` unit tests pass
- [x] Full `cargo test --workspace` passes (35 crates, all tests)
- [x] `cargo clippy --workspace` clean (zero warnings)
- [x] `cargo fmt` clean
- [x] All pre-commit hooks pass (conventional commits, gitleaks, format, lint)
- [x] Doc comments on all public API items
- [x] Edge-case tests: empty name via `unchecked("")`, empty `--var` value,
      nested directory commit
- [x] `TemplateError::Config` variant for config-path errors
- [x] `TemplateService::from_spec(&TemplateConfigSpec)` constructor
- [x] `TemplateError::Config` mapped to `ConfigInvalid` in CLI error handling
- [x] `commit()` takes `&WriteTarget` (no move/clone) in service integration
- [x] No output-path round-trip — uses `input.output_path.clone()` directly
