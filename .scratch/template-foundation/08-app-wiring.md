---
title: 08-app-wiring
category: enhancement
label: ready-for-agent
status: completed
branch: issue-08-app-wiring
merge_commit: 68295e2c
date_created: 2026-06-28
date_completed: 2026-06-28
---

# App Wiring — TemplateService Composition Root

Status: completed

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Wire `TemplateService` into the `traces-app` composition root so the CLI (`issue-09`) and future callers can invoke template rendering without knowing about `RedbRepository`, `FsWriter`, or `MiniJinjaEngine` construction.

Following the pattern established by `crates/app/src/index.rs` (`run_index`):

1. Add `traces-template` to `crates/app/Cargo.toml` dependencies.
2. Create `crates/app/src/template.rs` with:
   - `TEMPLATE_DB_FILENAME` re-exported from `traces_template::storage`
   - `run_template_create(config, cache_dir, input)` — the single execution flow that wires concrete adapters and calls `TemplateService`
3. Add `Template(TemplateError)` variant to `AppError` in `crates/app/src/error.rs` with `#[from]` conversion.
4. Register `pub mod template;` in `crates/app/src/lib.rs`.
5. Remove `#[allow(dead_code)]` from `RedbRepository` in `crates/template/src/storage/core.rs` (now has a production caller).

### `run_template_create` shape

```rust
pub fn run_template_create(
    config: &Config,
    cache_dir: &DirPath,
    input: &CreateTemplateInput,
) -> Result<CreateTemplateOutcome, AppError> {
    let spec = config.to_template_spec()?;
    let db_path = cache_dir.as_path().join(TEMPLATE_DB_FILENAME);
    let store = Store::open(&db_path)
        .map_err(|e| AppError::Template(
            TemplateError::Repository(
                TemplateRepositoryError::Storage(e.into()),
            ),
        ))?;
    let repo = RedbRepository::new(Arc::new(store));
    let writer = FsWriter::new(spec.root().as_path());
    let engine = MiniJinjaEngine::configured();
    let mut service = TemplateService::new(repo, writer, engine, spec);
    service.create(input).map_err(AppError::Template)
}
```

The function:
- Derives `TemplateConfigSpec` from the resolved `Config`
- Opens a `Store` at `cache_dir / templates.db`
- Constructs `RedbRepository`, `FsWriter`, `MiniJinjaEngine`
- Constructs `TemplateService`
- Runs `create()` — `verify_path` handles per-template freshness without a full tree walk

### Error mapping

`Store::open` failures (DB corruption, permissions) map through `TemplateRepositoryError::Storage` → `TemplateError::Repository` → `AppError::Template`. All other `TemplateError` variants (`NotFound`, `Engine`, `Artifact`, `Path`, `Scan`, `Repository`) propagate as `AppError::Template`.

No `Template`-specific exit codes or user-facing message formatting belongs here — that's the CLI handler's job (issue 09).

## Acceptance criteria

- [x] `traces-template` added to `crates/app/Cargo.toml` dependencies
- [x] `crates/app/src/template.rs` exists with `run_template_create()` matching the shape above
- [x] Module tests cover: `run_template_create` constructs service and returns outcome for valid inputs, propagates `TemplateError` as `AppError::Template`
- [x] `AppError::Template(TemplateError)` variant added in `crates/app/src/error.rs` with `#[from]` conversion
- [x] `crates/template/src/storage/core.rs` `#[allow(dead_code)]` removed from `RedbRepository` struct definition and `new()` method
- [x] No `unwrap()` or `panic!` in production code
- [x] `crate::storage::TEMPLATE_DB_FILENAME` is exported and used by the app wiring (no dead_code warning on the const)
- [x] `mise run test` passes

## Blocked by

- `issue-07-template-service.md`

## Implementation Notes

- Wire `TemplateService` as a composition-root flow in `traces-app`.
- **Adversarial Review Findings & Fixes**:
  - Removed app-side `spec.to_dir_path()` precheck to avoid boundary-smearing; `TemplateError::Path` now correctly propagates via `AppError::Template`.
  - Strengthened wiring tests to verify file existence and rendered content in the vault root.
  - Restored `storage::tables` visibility to `pub(crate)` to preserve storage encapsulation.
  - Updated module/variant documentation for template-rendering integration.
  - Added CLI `exit_code` mapping for template errors in `crates/cli/src/error.rs`.
- `mise run test` passed (unit/integration/e2e/doc).
- Commit: `68295e2c`

**Category:** enhancement
**Summary:** Wire `TemplateService` into the `traces-app` composition root so CLI commands can invoke template rendering without knowing adapter construction.

**Current behavior:**
`traces-app` has no dependency on the template context, no `template` module, no `Template` error variant. `RedbRepository` in the template crate carries `#[allow(dead_code)]` on its struct and constructor because no production caller exists.

**Desired behavior:**
`traces-app` exposes `run_template_create(config, cache_dir, input) -> Result<CreateTemplateOutcome, AppError>` — the single execution flow that wires concrete adapters (`Store`, `RedbRepository`, `FsWriter`, `MiniJinjaEngine`) into `TemplateService` and calls `create()`. No `process_all()` call: `create()` self-heals stale entries via internal `verify_path()`. The writer root comes from `TemplateConfigSpec::root()` (validated during spec construction), not a long config chain.

**Key interfaces:**
- `Config::to_template_spec()` — derives `TemplateConfigSpec` from resolved config
- `RedbRepository::new(Arc<Store>)` — production repository adapter
- `FsWriter::new(&Path)` — filesystem writer for rendered output
- `MiniJinjaEngine::configured()` — production rendering engine
- `TemplateService::new(repo, writer, engine, spec)` — orchestrator
- `CreateTemplateInput` — input DTO (name, output path, context, dry_run flag)
- `CreateTemplateOutcome` — result enum (Preview | Created)
- `AppError::Template(TemplateError)` — error variant with `#[from]` conversion

**Key design decisions:**
- No `process_all()` — `create()` calls `verify_path()` per-template (AR-1 option 3)
- DB path follows `cache_dir / "templates.db"` convention (same as index)
- `Config` is the parameter (not `TemplateConfigSpec`) — keeps app decoupled from settings internals
- No `TemplateCommand` wrapper — `CreateTemplateInput` is the input type directly

**Acceptance criteria:**
- [ ] `traces-template` dependency added to `crates/app/Cargo.toml`
- [ ] `pub fn run_template_create` exported from the `template` module with the correct signature
- [ ] `AppError::Template(TemplateError)` variant with `#[from]` conversion
- [ ] `crates/app/src/lib.rs` registers `pub mod template`
- [ ] `RedbRepository` struct and `new()` have `#[allow(dead_code)]` removed
- [ ] `TEMPLATE_DB_FILENAME` const consumed by app wiring (remove dead_code annotation)
- [ ] Tests cover: happy path, `Store::open` failure → `AppError::Template`, `Config::to_template_spec` failure → `AppError::Config`
- [ ] No `unwrap()` or `panic!()` in production code
- [ ] `mise run test` passes

**Out of scope:**
- CLI command handler (deferred to issue 09)
- Error-to-exit-code mapping (belongs in CLI handler)
- User-facing error message formatting (belongs in CLI handler)

---

## TDD Plan

### ⚠️ Critical Gap Found During Triage

Adding `AppError::Template(TemplateError)` breaks the exhaustive `exit_code()` match in `crates/cli/src/error.rs:203-208`. The match destructures `AppError::Discovery`, `Config`, and `Indexer` explicitly — the new variant causes a **compilation error**.

**Fix** (required for AC `mise run test` to pass — deferred error-to-exit-code refinement stays in issue 09):

```rust
// Add to exit_code() match after the Indexer arm:
Self::Bootstrap(_) => 2,
```

### Blast Radius (Impact Analysis)

| Symbol | Change | Risk | Dependents |
|---|---|---|---|
| `AppError` enum (`crates/app/src/error.rs:13`) | Add `Template(TemplateError)` variant | LOW | Consumed by `crates/cli/src/error.rs:21` (exit code match) |
| `RedbRepository` struct + `new()` (`crates/template/src/storage/core.rs:56,83`) | Remove `#[allow(dead_code)]` | LOW | None yet |
| `TEMPLATE_DB_FILENAME` (`crates/template/src/storage.rs:41`) | Remove `#[allow(dead_code)]` | LOW | Cosmetic (pub item) |
| `crates/app/src/lib.rs:24` | Add `pub mod template;` | LOW | — |
| `crates/app/Cargo.toml` | Add `traces-template` dep | LOW | — |
| NEW `crates/app/src/template.rs` | Create `run_template_create` | LOW | — |

### Phase 0 — Compiler-level Changes

1. **`crates/app/Cargo.toml`** — Add `traces-template = { path = "../template" }`
2. **`crates/app/src/error.rs`** — Add variant:
   ```rust
   /// Template pipeline failed.
   #[error(transparent)]
   Template(#[from] traces_template::TemplateError),
   ```
3. **`crates/app/src/lib.rs`** — Add `pub mod template;`
4. **`crates/template/src/storage/core.rs`** — Remove both `#[allow(dead_code)]` annotations
5. **`crates/template/src/storage.rs`** — Remove `#[allow(dead_code)]` on `TEMPLATE_DB_FILENAME`
6. **`crates/cli/src/error.rs:227`** — Add `Self::Bootstrap(_) => 2` to `exit_code()` match

### Phase 1 — Implement `run_template_create`

7. **NEW `crates/app/src/template.rs`** — Function matching the shape in § What to build. Re-exports:
   ```rust
   pub use traces_template::storage::TEMPLATE_DB_FILENAME;
   pub use traces_template::{CreateTemplateInput, CreateTemplateOutcome};
   ```

### Phase 2 — RED → GREEN: Tests

**8. In-module tests (`crates/app/src/template.rs`):**

| Test | How | Module |
|---|---|---|
| `returns_created_outcome_for_valid_input` | Integration w/ `TestDb` + temp vault + `Config` | `run_template_create` |
| `returns_app_error_when_store_fails` | Dir where DB file should be (same trick as `run_index` test) | `run_template_create` |
| `returns_app_error_when_config_is_invalid` | `Config` with no template dir → `to_template_spec()` fails | `run_template_create` |

**9. Conversion test (`crates/app/src/error.rs`):**
```rust
mod conversions {
    fn converts_template_error_to_app_error()
}
```

Following the existing pattern (`converts_indexer_error_to_app_error`).

**10. Integration test (optional):**
`crates/app/tests/template.rs` — full end-to-end happy path using `TestDb` + temp vault.

### Phase 3 — Verify

- `mise run lint` — clippy passes
- `mise run test` — all tests pass

### Files to Create/Modify

| Action | File |
|---|---|
| MODIFY | `crates/app/Cargo.toml` |
| CREATE | `crates/app/src/template.rs` |
| MODIFY | `crates/app/src/error.rs` |
| MODIFY | `crates/app/src/lib.rs` |
| MODIFY | `crates/template/src/storage/core.rs` |
| MODIFY | `crates/template/src/storage.rs` |
| MODIFY | `crates/cli/src/error.rs` |
| CREATE (optional) | `crates/app/tests/template.rs` |
