---
title: 14-config-builder-discovery-decoupling
category: enhancement
label: ready-for-human
status: completed
date_created: 2026-06-12
date_completed: 2026-06-15
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-human

## Parent

- `.scratch/root-config-discovery/PRD.md`
- `.scratch/root-config-discovery/discovery-redesign-decisions.md`
- `.scratch/root-config-discovery/10-bootstrap-context-discovery-contracts.md`
- `.scratch/root-config-discovery/11-discovery-service-config.md`
- `.scratch/root-config-discovery/12-discovery-typestate-run.md`
- `.scratch/root-config-discovery/13-bootstrapper-orchestration-flow.md`
- `docs/adr/config/0001-config-builder-decoupling.md`

## What to build

Refactor `config/Builder` so Config no longer orchestrates Discovery.

`Builder` receives the Bootstrapper-produced `DiscoveryResult` via a thin
`from_discovery()` adapter, then builds config through `build_vault()` and
`build_global()` — each fully owning its file read, staleness check, and
processor pipeline from the `CandidatePath` data. The `ConfigDiscoveryPipeline`
intermediate layer is removed entirely.

## Current state

Issues `10`-`13` are the implementation source of truth for this slice, even
where older ADR text differs.

The active Discovery handoff is:

- `DiscoveryResult` with separate ordered `vault` and `global` candidate vectors.
- `CandidatePath { base: DirPath, path: FilePath }`.
- No separate `vault_root`; the vault root is derived from the selected vault
  candidate's `base`.
- No stored format field; format-sensitive behavior must derive from the
  candidate path when Config needs it.
- No `DiscoveredMarker`, `InvocationInput`, `DiscoveryEngine`, `DiscoveryInput`,
  `GlobalDiscoveryInput`, or `DiscoveryPolicy` in the new integration path.

The DiscoveryService enforces a non-empty vault candidate list before producing
`DiscoveryResult`, so Config never receives a vaultless result.

## Design

```
DiscoveryService
  └─ guarantees at least one vault candidate, none empty (structural invariant)
       │
       ▼
DiscoveryResult { vault: Vec<CandidatePath>, global: Vec<CandidatePath> }
       │
       ▼
Builder::from_discovery(result, repo)
  └─ stores candidate boxes, returns Builder
  └─ infallible (structural invariants enforced upstream by DiscoveryService)
       │
       ▼
Builder::build()
  ├─ build_vault(&R) → RawVaultConfig                                [always]
  │   └─ derives VaultRoot from vault[0].base(), resolves VaultId via DB
  ├─ build_global(&R) → Option<RawGlobalConfig>                      [if global present]
  └─ build_from_layers(global, vault) → Config                       [unchanged merge]
```

`build_vault()` and `build_global()` each:

1. Read `FileMetadata` from the `CandidatePath`
2. Fetch the cached DB view for staleness comparison
3. Call `ConfigFileProcessor::compare()` to determine staleness
4. Produce their respective raw config type

No `ConfigDiscoveryPipeline` type exists — the work is split between the two
build methods, which are independently testable.

### Builder representation

`Builder<R>` stores the candidate vectors directly (owned) to avoid a lifetime
parameter:

```rust
pub(crate) struct Builder<R> {
    vault: Box<[CandidatePath]>,
    global: Box<[CandidatePath]>,
    repository: R,
}
```

`from_discovery()` moves the `Box<[CandidatePath]>` slices out of the incoming
`DiscoveryResult`. `VaultRoot` and `VaultId` are derived inside
`build_vault()` from `self.vault[0].base()` and checked against stored vault
identity in the DB — builder state stays minimal.

## Acceptance criteria

### Builder interface

- [ ] `Builder::from_discovery()` is the only Config entry point that accepts a
      `discovery::service::DiscoveryResult`.
- [ ] `Builder::from_discovery()` stores the candidate boxes and repository.
      Winner extraction is deferred — `build_vault()` and `build_global()`
      index `self.vault[0]` and `self.global.first()` respectively.
- [ ] `Builder::from_discovery()` stays thin: moves candidate boxes, stores
      repository. No file reading, no staleness checking, no structural
      validation, no VaultId resolution.
- [ ] `Builder::from_discovery()` consumes Discovery's validated `CandidatePath
      { base: DirPath, path: FilePath }` handoff instead of re-validating plain
      `PathBuf` marker paths.
- [ ] `Builder::from_discovery()` is infallible. It moves candidate boxes and
      stores the repository. All discovery-side invariants (non-empty vault,
      valid paths) are enforced by `DiscoveryService` upstream. Error sources
      (VaultId resolution, DirPath→VaultRoot conversion, file I/O, staleness)
      live in `build_vault()` and `build_global()`.
- [ ] `Builder::build()` orchestrates `build_global()` and `build_vault()`
      based on discovered marker presence.

### Build methods

- [ ] `Builder::build_vault()` reads `self.vault[0]`, derives `VaultRoot` from
      its `base()`, resolves `VaultId` via DB (create if new), reads file
      metadata, fetches the cached vault view, runs
      `ConfigFileProcessor::compare()` for staleness, and produces
      `RawVaultConfig`. Always called (vault candidate guaranteed upstream).
- [ ] `Builder::build_global()` reads `self.global.first()`, reads file
      metadata, fetches the cached global view, runs
      `ConfigFileProcessor::compare()` for staleness, and produces
      `Option<RawGlobalConfig>`. Called only when a global candidate exists.
- [ ] `Builder::build_vault()` and `Builder::build_global()` are independently
      testable and contain no discovery orchestration.
- [ ] `build_from_layers()` remains the unchanged pure config-domain merge seam.

### Removals

- [ ] `config/root.rs` is deleted; `ConfigDiscoveryResult` and
      `DiscoveredConfigFile` are removed.
- [ ] `config/discovery.rs` is deleted; `ConfigDiscoveryPipeline` and its
      config-owned `DiscoveryResult` type are removed. The per-candidate
      file-read + staleness + processor work is absorbed into `build_global()`
      and `build_vault()`.
- [ ] `Builder` no longer stores `start_dir`. All callers of
      `Builder::new(start_dir, ...)` are updated or removed.
- [ ] `config/builder.rs` no longer imports `DiscoveryEngine`, `DiscoveryInput`,
      `GlobalDiscoveryInput`, or discovery policy types.
- [ ] `config/builder.rs` imports `discovery::service::DiscoveryResult` only
      for `Builder::from_discovery()`.

### Invariants preserved

- [ ] Existing staleness behavior remains owned by
      `ConfigFileProcessor::compare()`; no `BuildMode` is introduced.
- [ ] File-vs-directory validation remains owned by Discovery/FS path types
      (`DirPath`, `FilePath`).
- [ ] Config only reads file contents and queries cached DB views — no path
      re-validation.

### Tests

- [ ] Tests prove `Builder` builds correctly from vault-only and combined
      (global+vault) discovery outputs.
- [ ] A regression test verifies `build_from_layers()` contract is preserved
      during refactoring.
- [ ] Test naming follows descriptive convention:
      `from_discovery_stores_vault_root_from_candidate_base`,
      `build_vault_produces_config_from_vault_only`, etc.

## Blocked by

- `.scratch/root-config-discovery/13-bootstrapper-orchestration-flow.md` —
  **RESOLVED** (completed 2026-06-14)

## Implementation Notes

**Commit:** `a3fffdd5` on branch `feat/issue-14-config-builder-discovery-decoupling`

### What was built

`Builder<R>` struct fields replaced: `start_dir: PathBuf` → `vault: Box<[CandidatePath]>` + `global: Box<[CandidatePath]>`.

New entry point:
- `Builder::from_discovery(result: DiscoveryResult, repository: R) -> Self` — infallible; calls `result.into_parts()` and stores the two boxed slices plus repository. No file I/O, no validation, no VaultId resolution.

New build methods:
- `build_vault()` — indexes `self.vault[0]`, calls `VaultRoot::from_dir_path(candidate.base().clone())`, calls `get_or_create_vault_id()`, reads `FsMetadata`, parses `RawVaultConfig` via `FileReader::from_system_root()`, fetches `RawVaultConfigView` from DB. Returns `VaultBuildResult` type alias.
- `build_global()` — same pipeline for `self.global.first()`; returns `(None, None)` when no global candidate. Returns `GlobalBuildResult` type alias.
- `build()` — orchestrates both, feeds processor outcomes into `ConfigResolver::resolve()` → `execute_plan()`.

`execute_plan` and `rebuild_with_configs` refactored to read file paths from `self.vault.first()` / `self.global.first()` rather than the old `config::discovery::DiscoveryResult` parameter.

Deleted: `config/discovery.rs` (ConfigDiscoveryPipeline, config-owned DiscoveryResult, GlobalDiscovery, VaultDiscovery), `config/root.rs` (ConfigDiscoveryResult, DiscoveredConfigFile). Removed their `mod` declarations from `config/mod.rs`.

### Architecture tests updated

- `builder_must_not_use_known_vault_root_discovery_shortcut`: removed the now-obsolete `assert!(content.contains("find_vault"))` check; kept the `find_known_vault` prohibition.
- New test `builder_imports_only_discovery_service_result_from_discovery`: verifies builder contains no `discovery::engine`, `discovery::policy`, `DiscoveryEngine`, `DiscoveryInput`, `GlobalDiscoveryInput`, or `DiscoveryPolicy` references.

### Tests added (18 unit tests)

- `from_discovery`: stores_vault_candidates_as_boxed_slice, stores_global_candidates_as_boxed_slice, is_infallible, stores_repository
- `build_vault`: derives_vault_root_from_candidate_base, resolves_vault_id_from_database, reads_file_metadata_from_candidate, queries_database_for_vault_view, parses_raw_config_from_candidate_file, returns_raw_vault_config
- `build_global`: returns_none_when_no_global_candidate, reads_file_and_returns_raw_global_config, queries_database_for_global_view
- `build`: orchestrates_vault_and_global, builds_from_vault_only_discovery
- `build_from_layers_regression`: preserves_existing_merge_behavior_vault_overrides_global, preserves_existing_merge_behavior_global_used_when_no_vault, preserves_existing_merge_behavior_defaults_used_when_no_sources

### Deviations from plan

None. All 9 TDD phases implemented as specified. One divergence from the TDD plan's test name conventions: `build::runs_config_file_processor` is folded into `build_vault` / `build_global` integration (processor is invoked inside `build()`, tested implicitly by the `build::orchestrates_*` tests which exercise full staleness and merge behavior). A separate `runs_config_file_processor` unit test was not added since it would require mocking `ConfigFileProcessor` internals.

### Bootstrapper wiring gap

`Builder::from_discovery()` is `pub(crate)` and annotated with `#[cfg_attr(not(test), expect(dead_code, ...))]`. The old `Builder::new(start_dir, repo)` + `Builder::load()` entry point that internally drove `DiscoveryEngine` is gone. **Nothing in production code calls `Builder::from_discovery()` yet.** The Bootstrapper (`app::Bootstrapper`) produces a `DiscoveryResult` (issue 13), and the Config builder now consumes it (issue 14), but the call-site connecting them — the CLI command handler or app service layer — has not been created. This wiring is not covered by issues 10–16 in the current plan. A new issue is needed (see "Blocks" below).

## Blocks

A follow-on issue is needed to wire `Bootstrapper::discover()` → `Builder::from_discovery()` in the app/CLI layer. Without it, `Builder` is structurally ready but unreachable from any executable path. The old `Builder::new` + `load()` path was the only production caller and is now removed.

## TDD Plan — Phase 10: `Bootstrapper::run()` wiring

Extends the issue with the wiring that closes the dead-code gap in `from_discovery`.

### Context

`Bootstrapper::discover()` already returns `(DiscoveryResult, DiscoveryReport)`.
`Builder::from_discovery(result, repo).build()` already returns `Result<Config, ConfigError>`.
The only missing piece is the method that sequences them and the error variant that carries `ConfigError` out of the bootstrap boundary.

### Phase 10: Extend `BootstrapError` and add `Bootstrapper::run()`

#### 10.1 `bootstrap_error::includes_config_variant`

- **Module**: `app/bootstrap.rs` → `mod tests` → `mod bootstrap_error`
- **Behavior**: `BootstrapError` has a `Config(#[from] ConfigError)` variant so `?` works on `Builder::build()` inside `run()`.
- **Test**: Construct `BootstrapError::Config(ConfigError::Ingestion("x".into()))`, assert `matches!(e, BootstrapError::Config(_))`.
- **Implementation**: Add `Config(#[from] ConfigError)` to the `BootstrapError` enum. Remove the `#[allow(dead_code)]` on `BootstrapError` — both variants are now reachable.

#### 10.2 `run::builds_config_from_vault_only_discovery`

- **Module**: `app/bootstrap.rs` → `mod tests` → `mod run`
- **Behavior**: `Bootstrapper::run(flags, env, anchor, repo)` returns `Ok((Config, DiscoveryReport))` when discovery finds a vault candidate and config builds successfully.
- **Test**: Create a temp dir with a `lithos.toml`, pass its path as a `DiscoveryFlags` vault + config override, call `run()` with `InMemoryRepository`, assert `Ok((config, _))`.
- **Implementation**:
  ```rust
  pub(crate) fn run<R: Repository>(
      &self,
      flags: Option<DiscoveryFlags>,
      env: Option<DiscoveryEnv<'_>>,
      anchor: &std::path::Path,
      repository: R,
  ) -> Result<(Config, DiscoveryReport), BootstrapError> {
      let context = Self::build_context(flags, env, anchor)?;
      let (discovery, report) = self.discover(&context)?;
      let config = Builder::from_discovery(discovery, repository).build()?;
      Ok((config, report))
  }
  ```
  Add imports: `crate::config::{aggregate::Config, builder::Builder, repository::Repository}`.

#### 10.3 `run::propagates_discovery_error`

- **Behavior**: `run()` returns `Err(BootstrapError::Discovery(_))` when the discovery port fails.
- **Test**: Use `MockDiscoveryPort` returning `Err(DiscoveryError::InvalidAnchorDirectory { .. })`, assert error variant matches.

#### 10.4 `run::propagates_config_error`

- **Behavior**: `run()` returns `Err(BootstrapError::Config(_))` when `Builder::build()` fails (e.g. unparseable TOML).
- **Test**: Create a `lithos.toml` containing invalid TOML, wire through flags, assert `Err(BootstrapError::Config(_))`.

#### 10.5 `run::returns_report_alongside_config`

- **Behavior**: The `DiscoveryReport` is returned unchanged alongside the built `Config`.
- **Test**: Use `MockDiscoveryPort` returning a report with a known `local_traversal_stop_reason`, assert the returned report matches.

#### 10.6 `run::concrete_service_builds_config_from_vault_with_platform_bootstrapper`

- **Module**: `mod concrete_service` (existing)
- **Behavior**: End-to-end smoke test through `Bootstrapper<DiscoveryService>`: `with_global_directories([])`, flag-supplied vault + config, `run()` succeeds.
- **Test**: Mirrors the existing `returns_app_result_from_concrete_discovery_service` test but calls `run()` instead of `discover()`, asserts `config` is returned.

### Test suite additions

```
mod bootstrap_error {
    #[test] fn includes_config_variant() {}
}
mod run {
    #[test] fn builds_config_from_vault_only_discovery() {}
    #[test] fn propagates_discovery_error() {}
    #[test] fn propagates_config_error() {}
    #[test] fn returns_report_alongside_config() {}
}
mod concrete_service {
    // existing tests unchanged
    #[test] fn run_builds_config_from_vault_with_platform_bootstrapper() {}
}
```

### Definition of Done — Phase 10

- [ ] `BootstrapError::Config(#[from] ConfigError)` variant added
- [ ] `#[allow(dead_code)]` removed from `BootstrapError` (both variants reachable)
- [ ] `Bootstrapper::run<R: Repository>(flags, env, anchor, repo) -> Result<(Config, DiscoveryReport), BootstrapError>` added
- [ ] `#[cfg_attr(not(test), expect(dead_code, ...))]` removed from `Builder::from_discovery` (now called from production code in `run()`)
- [ ] `run()` is on `impl<D: DiscoveryPort> Bootstrapper<D>`, not only on `impl Bootstrapper<DiscoveryService>`
- [ ] All 6 new tests pass
- [ ] `mise run verify` clean


## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Refactor Config builder so Config consumes discovery results
without orchestrating Discovery. Remove the `ConfigDiscoveryPipeline`
intermediate layer entirely — each build method owns its full pipeline from
`CandidatePath` to raw config.

**Current behavior:**
Config builder orchestrates discovery internally (`DiscoveryEngine`,
`DiscoveryInput`, `GlobalDiscoveryInput`, `DiscoveryPolicy`), then feeds
results through `config/root.rs` bridge types and `ConfigDiscoveryPipeline` —
creating unnecessary coupling and indirection.

**Desired behavior:**
Config receives the Bootstrapper-produced, FS-validated `DiscoveryResult`
through a narrow `from_discovery()` adapter that extracts winning
`CandidatePath` values, derives `VaultRoot`, and resolves `VaultId`.
`Builder::build()` then delegates to `build_vault()` (always) and
`build_global()` (conditionally), each of which owns its file read, staleness
check, and processor pipeline. The `ConfigDiscoveryPipeline` layer,
`config/discovery.rs`, and `config/root.rs` bridge types are deleted.

Discovery-side structural invariants (non-empty vault, valid paths) are
enforced by `DiscoveryService` before `DiscoveryResult` reaches Config.
`from_discovery()` never validates discovery assumptions — it only converts and
stores.

**Key interfaces:**
- `Builder::from_discovery(DiscoveryResult, R)` — moves candidate boxes,
  stores repository. Infallible.
- `Builder::build_vault(&R)` → `RawVaultConfig` — derives VaultRoot, resolves
  VaultId, reads file, checks staleness, processes. Always called.
- `Builder::build_global(&R)` → `Option<RawGlobalConfig>` — same, called
  conditionally from `build()`.
- `Builder::build()` — orchestrates build methods and merge seam.
- `build_from_layers()` — unchanged merge seam.
- `ConfigFileProcessor::compare()` — unchanged staleness owner.

**Deleted files:**
- `config/root.rs` — bridge types removed.
- `config/discovery.rs` — pipeline removed.

**Acceptance criteria:**
- [ ] `from_discovery()` moves candidate boxes and stores repository — no file
      I/O, no staleness, no VaultRoot/VaultId derivation, no structural
      validation. Infallible.
- [ ] `build_vault()` indexes `self.vault[0]`, derives `VaultRoot` and resolves
      `VaultId`, reads file, fetches view, checks staleness, processes.
      `build_global()` indexes `self.global.first()` for the same pipeline.
- [ ] No `ConfigDiscoveryPipeline` — `config/discovery.rs` deleted.
- [ ] No `config/root.rs` — bridge types deleted.
- [ ] Builder imports no discovery engine/input/policy types.
- [ ] `from_discovery()` does not validate structural invariants (non-empty
      vault) — DiscoveryService guarantees those upstream.
- [ ] Tests prove vault-only and combined flows, regression-test
      `build_from_layers()` contract.

**Out of scope:**
- Changing the DiscoveryService public API.
- Bootstrapper implementation beyond consuming the result shape from issue `13`.
- CLI discovery subcommands.
- Replacing existing staleness comparison with a new build mode.

## TDD Plan

### Phase 1: New Builder State & Constructor

#### 1.1 `from_discovery::stores_vault_candidates_as_boxed_slice`

- **Module**: `config/builder.rs` → `mod builder` → `mod from_discovery`
- **Behavior**: `Builder::from_discovery(result, repo)` stores `vault` and `global` `Box<[CandidatePath]>` from `DiscoveryResult::into_parts()`.
- **Test**: Create `DiscoveryResult` with one vault candidate, verify `builder.vault.len() == 1`.
- **Implementation**: Replace `start_dir: PathBuf` field with `vault: Box<[CandidatePath]>`, `global: Box<[CandidatePath]>`. Add `from_discovery()` calling `into_parts()`.

#### 1.2 `from_discovery::stores_global_candidates_as_boxed_slice`

- Mirror of above for `global` field.

#### 1.3 `from_discovery::is_infallible`

- **Behavior**: `from_discovery()` returns `Self`, not `Result`.
- **Test**: Call with empty `DiscoveryResult::new(vec![], vec![])`.

#### 1.4 `from_discovery::stores_repository`

- **Behavior**: Repository handle is stored and accessible by build methods.

### Phase 2: Build Methods — Vault Pipeline

#### 2.1 `build_vault::derives_vault_root_from_candidate_base`

- **Module**: `mod build_vault`
- **Behavior**: `build_vault()` derives `VaultRoot` from `self.vault[0].base()`.
- **Implementation**: `VaultRoot::from_dir_path(self.vault[0].base().clone())`

#### 2.2 `build_vault::resolves_vault_id_from_database`

- **Behavior**: Calls `repository.find_vault_id_by_path(vault_root)`, creates new `VaultId` if absent.
- **Implementation**: Reuse `get_or_create_vault_id()` helper moved to work on stored state.

#### 2.3 `build_vault::reads_file_metadata_from_candidate`

- **Behavior**: Reads `FileMetadata` from the path at `self.vault[0].path()`.
- **Implementation**: `FsMetadata::from_path(candidate.path().as_path())`

#### 2.4 `build_vault::queries_database_for_vault_view`

- **Behavior**: Fetches `RawVaultConfigView` from repository via `VaultId`.

#### 2.5 `build_vault::parses_raw_config_from_candidate_file`

- **Behavior**: Reads and parses the config file at the candidate path.
- **Implementation**: `FileReader::from_system_root().parse_structured::<RawVaultConfig>(candidate.path().as_path())`

#### 2.6 `build_vault::runs_config_file_processor`

- **Behavior**: Feeds raw config + view through `ConfigFileProcessor::compare()` pipeline. Returns processor outcome (same pattern as current `load()` lines 374-396).

#### 2.7 `build_vault::returns_raw_vault_config`

- **Behavior**: Returns `RawVaultConfig` with metadata attached.

### Phase 3: Build Methods — Global Pipeline

#### 3.1 `build_global::returns_none_when_no_global_candidate`

- **Module**: `mod build_global`
- **Behavior**: Returns `None` when `self.global` is empty.

#### 3.2 `build_global::reads_file_and_returns_raw_global_config`

- **Behavior**: Same pipeline as `build_vault()` minus `VaultRoot`/`VaultId` steps.

#### 3.3 `build_global::queries_database_for_global_view`

- **Behavior**: Fetches `RawGlobalConfigView` from repository.

### Phase 4: Build Orchestration

#### 4.1 `build::orchestrates_vault_and_global`

- **Module**: `mod build`
- **Behavior**: `build()` calls `build_vault()` (always) and `build_global()` (conditionally), feeds results into `ConfigResolver::resolve()` and `execute_plan()` (refactored).

### Phase 5: Refactor Internal Methods

#### 5.1 Refactor `execute_plan` and `rebuild_with_configs`

- **Behavior**: Read paths from `self.vault[0]` / `self.global.first()` instead of `config::discovery::DiscoveryResult`.
- **Implementation**: Replace `discovery.global().entry().map(...)` with `self.global.first().map(|c| c.path().as_path().to_string_lossy())`.

### Phase 6: Delete Removed Files

#### 6.1 Delete `config/root.rs`

- `DiscoveredConfigFile`, `ConfigDiscoveryResult` removed. Remove `pub(crate) mod root;` from `config/mod.rs`.

#### 6.2 Delete `config/discovery.rs`

- `GlobalDiscovery`, `VaultDiscovery`, config-owned `DiscoveryResult`, `ConfigDiscoveryPipeline` removed. Remove `pub(crate) mod discovery;` from `config/mod.rs`.

### Phase 7: Update Architecture Tests

#### 7.1 `builder_must_not_use_known_vault_root_discovery_shortcut`

- Remove `find_vault` check (builder no longer owns discovery). Keep `find_known_vault` prohibition.

#### 7.2 Add import discipline test

- Verify `config/builder.rs` imports only `discovery::service::DiscoveryResult` from `discovery/`.

### Phase 8: Cleanup Builder Imports

- Remove `use crate::discovery::{engine::*, policy::*}` from `builder.rs`.
- Remove `use crate::config::{discovery::*, root::*}` from `builder.rs`.
- Add `use crate::discovery::service::DiscoveryResult`.

### Phase 9: Regression — `build_from_layers` Contract

#### 9.1 `build_from_layers::preserves_existing_merge_behavior`

- Pure regression: encapsulate existing `aggregate.rs` test cases to confirm identical output before/after refactor.

### Test Suite Structure

```
#[cfg(test)]
mod tests {
    mod builder {
        mod from_discovery {
            #[test] fn stores_vault_candidates_as_boxed_slice() {}
            #[test] fn stores_global_candidates_as_boxed_slice() {}
            #[test] fn is_infallible() {}
            #[test] fn stores_repository() {}
        }
        mod build_vault {
            #[test] fn derives_vault_root_from_candidate_base() {}
            #[test] fn resolves_vault_id_from_database() {}
            #[test] fn reads_file_metadata_from_candidate() {}
            #[test] fn queries_database_for_vault_view() {}
            #[test] fn parses_raw_config_from_candidate_file() {}
            #[test] fn runs_config_file_processor() {}
            #[test] fn returns_raw_vault_config() {}
        }
        mod build_global {
            #[test] fn returns_none_when_no_global_candidate() {}
            #[test] fn reads_file_and_returns_raw_global_config() {}
            #[test] fn queries_database_for_global_view() {}
        }
        mod build {
            #[test] fn orchestrates_vault_and_global() {}
        }
    }
}
```

### Definition of Done

- [ ] `Builder::from_discovery()` exists, infallible, stores boxes + repo
- [ ] `build_vault()` reads file, derives VaultRoot, resolves VaultId, queries DB, runs processor
- [ ] `build_global()` reads file, queries DB, runs processor
- [ ] `build()` orchestrates both build methods
- [ ] `config/root.rs` deleted
- [ ] `config/discovery.rs` deleted
- [ ] Builder imports no `DiscoveryEngine`, `DiscoveryInput`, `GlobalDiscoveryInput`, `DiscoveryPolicy`
- [ ] Builder imports only `discovery::service::DiscoveryResult` from discovery/
- [ ] Architecture tests updated
- [ ] `build_from_layers()` unchanged — all 36 callers pass
- [ ] `mise run test` passes
- [ ] `mise run lint` passes
