---
title: 02-make-discovery-internal-and-linear
category: enhancement
label: ready-for-agent
status: ready
branch: issue-02-make-discovery-internal-and-linear
merge_commit: null
date_created: 2026-06-29
date_completed: null
---

# Make Discovery Internal And Linear

## Parent

.scratch/settings-redesign/PRD.md

## What to build

Move discovery behind the Settings boundary and implement the linear discovery flow: normalize options and environment into internal input, collect local candidates, collect global candidates, filter/dedupe, and return a discovery outcome. Discovery should no longer be modeled as an external port for new code.

Migrate discovery consumers to the new SettingsService API before marking old discovery service/port types unused. Do not delete old components here if any caller still needs them.

### Renaming Directives

| New Name | Old Name | Action |
|----------|----------|--------|
| `DiscoveryProcessor` (typestate, internal) | `DiscoveryService` + `DiscoveryPort` | New internal orchestrator replaces both. Delete old port/service in issue 07 |
| `SettingsEnvVars` (in `src/env_var.rs`) | `SettingsEnvVars` (in `discovery/env.rs`) | **Move file** — structural rename was handled in Issue 00. Remove duplicated XDG statics from `discovery/env.rs` (already in `src/os_dirs.rs`). Keep `discovery/env.rs` `EnvVars` struct — old `DiscoveryContext`/`DiscoveryProcessor` still construct it. New code uses `SettingsEnvVars` |
| `DiscoveryInput` (in `discovery/input.rs`) | (new — no old equivalent) | Normalized input from options + env |
| `src/os_dirs.rs` | `discovery/dirs.rs` + XDG static helpers in `discovery/env.rs` | **Consolidate and move** — `AppDirs` in `discovery/dirs.rs` and XDG statics (`HOME`, `XDG_*`) in `discovery/env.rs` fold into `src/os_dirs.rs`. Old `discovery/dirs.rs` import chain updates to use `crate::os_dirs` |
| `src/location.rs` | `discovery/policy.rs` (path constants only) | **New code only** — `MarkerPattern` struct replaced by flat `&[&str]` slices in `src/location.rs`. `BOUNDARY_MARKER_PATTERNS` moves into `src/location.rs`. `discovery/location.rs` (cache types `CacheRoot` etc.) is **not consolidated** — those stay for old callers until issue 07 |
| `CandidatePath` (in `src/candidate.rs`) | Was inline in old discovery outcome | **Extract** — standalone bridge type |
| `SettingsEnvVars` env var rename | `TRACES_VAULT_DIR` → `TRACES_DEFAULT_VAULT` | **Rename** — semantics change: `TRACES_VAULT_DIR` was an override; `TRACES_DEFAULT_VAULT` is a fallback used only when normal local traversal finds nothing. Update `src/env_var.rs` field + capture key. Old `discovery/env.rs` keeps `TRACES_VAULT_DIR` for old callers |
| `DiscoveryOutcome` (in `discovery/outcome.rs`) | Replaces old `DiscoveryResult` | **Rename** — now holds `Box<[CandidatePath]>` slices, not old candidate types. **No `cache_root` field** — cache dir moves to `AppConfig::create_cache_dir()`. Old `DiscoveryResult` retains `cache_root` for old callers until issues 08/09 |

## Acceptance criteria

- [ ] `SettingsEnvVars` field `vault_dir` renamed to `default_vault_dir`, env key `TRACES_VAULT_DIR` → `TRACES_DEFAULT_VAULT`.
- [ ] `SettingsEnvVars` field `config_file` renamed to `global_config`, env key `TRACES_CONFIG_FILE` → `TRACES_GLOBAL_CONFIG`.
- [ ] `discovery/env.rs` XDG statics (`HOME`, `XDG_*`) removed — already exist in `src/os_dirs.rs`.
- [ ] `discovery/env.rs` `EnvVars` struct **kept** — old `DiscoveryContext`/`DiscoveryProcessor` still need it. `SettingsEnvVars` is used only by new code.
- [ ] `discovery/dirs.rs` imports updated from `crate::discovery::env::XDG_*` to `crate::os_dirs::XDG_*`. `AppDirs` struct **kept** — `BootstrapRunner::from_platform()` still uses it until issue 08/09.
- [ ] `DiscoveryInput` is internal and constructed from `DiscoveryOptions` plus internally-read settings environment variables.
- [ ] `DiscoveryProcessor` uses explicit transition methods for local collection, global collection, and finish.
- [ ] Discovery flow is linear and has no old branch/cache-resolution phase model.
- [ ] Local collection returns candidates in outer-ancestor to nearest-ancestor order.
- [ ] `TRACES_DEFAULT_VAULT` (renamed from `TRACES_VAULT_DIR`) is only used as fallback when normal local collection finds no local candidate.
- [ ] Global collection follows suppress, flag, env, platform-dir precedence.
- [ ] Exact filename slices in `src/location.rs` replace marker-pattern/extension iteration for new discovery code.
- [ ] `CandidatePath` lives at `src/candidate.rs`, not in `discovery/outcome.rs` — discovery produces them, config consumes them.
- [ ] Dedupe/desymlink and ignored-path filtering happen before returning `DiscoveryOutcome`.
- [ ] `DiscoveryInput` carries the exact fields from the PRD: `anchor`, `flag_global`, `flag_vault`, `env_global`, `env_default_vault`, `ceiling_dirs`, `suppress_global`.
- [ ] `CandidatePath` is imported from `src/candidate.rs` — not defined in `discovery/outcome.rs`.
- [ ] `DiscoveryOutcome` has **no `cache_root` field**. Cache dir is derived from `CandidatePath::base` by `AppConfig::create_cache_dir()`, not by discovery. Old `DiscoveryResult` retains `cache_root` for old callers until issues 08/09.

## Out of Scope

- Config file reading, parsing, or domain type construction (issue 03)
- Config builder typestate or merge logic (issue 04)
- Tracker or trust modules (issues 05, 06)
- Removing old DiscoveryService/DiscoveryPort callers (issue 07)
- BootstrapRunner or CLI migration (issues 08, 09)

## Blocked by

- .scratch/settings-redesign/01-define-settings-service-boundary.md

## Triage Notes

> *This was generated by AI during triage.*

**Verdict**: `enhancement` + `ready-for-agent`.

This is ready once the service boundary exists. The issue is intentionally a migration slice: move discovery behind SettingsService for new code, but do not delete old discovery entry points until app/CLI consumers have moved.

### Agent Brief

- Implement the linear internal discovery path: options/env normalization, local collection, global collection, filter/dedupe, finish.
- Preserve behavior for existing callers until later migration slices replace them.
- Keep collector filesystem work outside the typestate orchestrator; processor states should sequence, not perform all logic inline.
- Use exact filename constants from `location.rs`; do not recreate `MarkerPattern`.
- `DiscoveryOutcome` has **no `cache_root`** field. Cache dir is `<base>/.traces/cache/` derived from `CandidatePath::base`, not returned by discovery.
- Consolidate `discovery/env.rs`: remove its XDG static duplicates (`HOME`, `XDG_*`) — already exist in `src/os_dirs.rs`. **Keep its `EnvVars` struct** — old `DiscoveryContext`/`DiscoveryProcessor` still construct it.
- Consolidate `discovery/dirs.rs`: `AppDirs` imports are updated to use `crate::os_dirs` statics instead of `discovery/env` ones.
- Rename `SettingsEnvVars` fields: `vault_dir` → `default_vault_dir` (env key `TRACES_VAULT_DIR` → `TRACES_DEFAULT_VAULT`); `config_file` → `global_config` (env key `TRACES_CONFIG_FILE` → `TRACES_GLOBAL_CONFIG`). These are different semantics — override → fallback.

### GitNexus Context

- Current CLI config flows call `Bootstrapper.run_discovery_only`, which calls the discovery port.
- Current architecture tests still assert discovery-service dependencies; expect those to change in the final docs/test slice, not necessarily here.
- `discovery/env.rs` `EnvVars` is the old env capture — its callers (`discovery/context.rs` via `DiscoveryEnv::from_env`, `discovery/dirs.rs` for XDG statics) stay on the old struct. `app/src/bootstrap.rs` already uses `SettingsEnvVars`.
- `BootstrapRunner::build_context` in `app/src/bootstrap.rs` still constructs `DiscoveryContext` with old `DiscoveryEnv`/`DiscoveryFlags`. This issue does NOT migrate `BootstrapRunner` (issues 08/09), but its old imports should continue to compile.

### Rust Best-Practices Notes

- Typestate is justified here because ordering matters; keep it linear and avoid runtime branch enums unless a real branch exists.
- Use `Result` and `?` for filesystem/env errors; no production `unwrap`/`expect`.
- Unit-test ordering, fallback behavior, global precedence, and ignored filtering with tempdirs.

## TDD Plan

Approved plan (13 cycles, vertical RED→GREEN tracer bullets per cycle). All tests follow Structure A (submodules) per `docs/engineering/testing/unit-naming.md`. Use `pretty_assertions` + `tempfile`.

### Cycle 1: `SettingsEnvVars` field renames
**File:** `crates/settings/src/env_var.rs`
Rename `vault_dir` → `default_vault_dir` (env key `TRACES_VAULT_DIR` → `TRACES_DEFAULT_VAULT`). Rename `config_file` → `global_config` (env key `TRACES_CONFIG_FILE` → `TRACES_GLOBAL_CONFIG`). Update accessors, constructor, capture keys.
**Test:** `mod capture` / `mod constructor` — capture from new keys, construct with all/none.
**Callers to update:** `discovery/dirs.rs` (3 `SettingsEnvVars::new(...)` call sites).

### Cycle 2: Remove XDG static duplicates from `discovery/env.rs`
**File:** `crates/settings/src/discovery/env.rs`
Delete `HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` statics + their tests. Keep `EnvVars` struct + its tests.
**File:** `crates/settings/src/discovery/dirs.rs`
Update import: `crate::discovery::env::{XDG_CACHE_HOME, XDG_CONFIG_HOME}` → `crate::os_dirs::{XDG_CACHE_HOME, XDG_CONFIG_HOME}`.
**Test:** `EnvVars` capture tests still pass. `AppDirs` tests still pass using new import.

### Cycle 3: Remove `cache_root` from `DiscoveryOutcome`
**File:** `crates/settings/src/service.rs`
Remove `cache_root` field from struct, `new()` param, accessor. Remove `CacheRoot` import. Update `service_boundary.rs` test.
**File:** `crates/settings/tests/service_boundary.rs`
Update existing outcome-construction test — verify only `local` + `global` fields.

### Cycle 4: Expand `location.rs`
**File:** `crates/settings/src/location.rs`
Add `BOUNDARY_MARKERS: &[&str] = &[".git", ".workspace"]`. Add `GLOBAL_CONFIG_TARGETS: &[&str]` (same filenames as `MARKERS`). Add tracking/trust subdir names, cache subdir. Keep existing `MARKERS`.
**Test:** `mod constants` — verify each constant is non-empty and paths are relative.

### Cycle 5: `DiscoveryInput` struct
**New file:** `crates/settings/src/discovery/input.rs`
Fields: `anchor: DirPath`, `flag_vault: Option<DirPath>`, `flag_global: Option<FilePath>`, `env_global: Option<FilePath>`, `env_default_vault: Option<DirPath>`, `ceiling_dirs: Box<[PathBuf]>`, `suppress_global: bool`. Constructor `from_options(options: DiscoveryOptions, env: &SettingsEnvVars)`. Accessors.
**Test:** `mod constructor` — `from_options_merges_flag_and_env`; `mod accessors`.

### Cycle 6: Walker
**New file:** `crates/settings/src/discovery/walk.rs`
`AncestorEnumerator` — iterate from anchor up to ceiling dirs, yield `DirPath` per ancestor in outer→nearest order.
**Test:** `mod ancestor_enumeration` — starts at anchor, stops at ceiling, returns outer→nearest.

### Cycle 7: Prober
**New file:** `crates/settings/src/discovery/probe.rs` (replaces old internal one)
`exact_probe(dir: &DirPath, markers: &[&str]) -> Vec<CandidatePath>` — check only exact filenames from `location.rs`.
**Test:** `mod exact_filenames` — returns candidate when marker exists, returns empty when none, ignores non-marker files.

### Cycle 8: Global collector
**New file:** `crates/settings/src/discovery/global.rs`
`global_collect(suppress, flag, env, platform_dirs) -> Vec<CandidatePath>`. Precedence: flag → env → platform dirs.
**Test:** `mod precedence` — flag overrides env, env overrides platform, suppressed returns empty.

### Cycle 9: Filter
**New file:** `crates/settings/src/discovery/filter.rs`
`dedupe(candidates) -> Vec<CandidatePath>` — canonicalize, remove duplicates. `filter_ignored(candidates) -> Vec<CandidatePath>` — ignored-path check (stub for issue 06).
**Test:** `mod dedupe` — removes symlink duplicates, preserves first occurrence. `mod ignored` — filters ignored paths.

### Cycle 10: New linear `DiscoveryProcessor`
**New file:** `crates/settings/src/discovery/processor.rs` (after renaming old one)
Typestate: `Init` → `collect_local()` → `LocalCollected` → `collect_global()` → `GlobalCollected` → `finish()` → `DiscoveryOutcome`. Sequences walker + prober + global collector + filter.
**Test:** `mod state` — transitions; `mod finish` — returns outcome with correct ordering.

### Cycle 11: Rename old processor + move `DiscoveryOutcome`
Rename `discovery/processor.rs` → `discovery/processor_old.rs`. Move `DiscoveryOutcome` from `service.rs` to `discovery/outcome.rs` (no re-export, update lib.rs). Update old import paths.
**Test:** Old callers still compile. New `DiscoveryOutcome` only from `discovery::outcome`.

### Cycle 12: Wire `Service::discover()`
Replace stub with real impl: capture `SettingsEnvVars` → `DiscoveryInput::from_options()` → `DiscoveryProcessor::new(input).collect_local()?.collect_global()?.finish()`.
**Test:** `service_boundary.rs` `mod discover` — returns outcome for valid options.

### Cycle 13: Visibility + module wiring
Update `discovery/mod.rs` with new modules. Update `lib.rs` exports. Run `mise run verify`.
**Test:** `cargo check` + `cargo clippy` + `mise run test`.

## Implementation Notes

Status: implemented on branch `issue-02-discovery-linear`.

Implementation commits:

- `0c6711e4 refactor(settings): linearize discovery`
- `304fc43f chore(gitnexus): refresh index metadata`
- `c0b2dd62 fix(settings): expose discovery report`
- `555eb9b2 chore(gitnexus): refresh index metadata`

Key changes:

- Renamed new `SettingsEnvVars` fields and env keys to `default_vault_dir` / `TRACES_DEFAULT_VAULT` and `global_config` / `TRACES_GLOBAL_CONFIG`.
- Removed duplicate XDG statics from `discovery/env.rs`; old `EnvVars` remains for old discovery callers.
- Moved public `DiscoveryOutcome` to `discovery/outcome.rs` and removed `cache_root` from the new settings-service outcome.
- Added exact marker/location constants, internal `DiscoveryInput`, ancestor enumeration, exact probing, global collection, dedupe, and ignored-path filtering helpers.
- Preserved old discovery processor as `discovery/processor_old.rs`; old `DiscoveryService` still compiles against it.
- Replaced new `discovery/processor.rs` with the linear internal typestate processor and wired `Service::discover()` through it.
- Follow-up review fixes added `DiscoveryReport` back to the new `DiscoveryOutcome` shape, strengthened env-key capture tests for `TRACES_DEFAULT_VAULT` / `TRACES_GLOBAL_CONFIG`, added env-suppression edge coverage, and corrected stale discovery docs.

Decisions and deviations:

- `TRACES_DEFAULT_VAULT` validation is lazy: invalid fallback paths only error when no local marker is found and the fallback is actually used.
- Suppressed global lookup ignores invalid flag/env global paths because those inputs are not consumed when global discovery is suppressed.
- Exact marker names include `traces.{toml,json,yaml,yml}` and `.traces/config.{toml,json,yaml,yml}` to match the PRD.
- Ignored-path filtering is implemented as a helper but receives an empty list until trust-store ignored paths are wired in a later issue.
- `AGENTS.md` changed only because GitNexus index metadata was refreshed after implementation.
- The new linear report is wired through the processor and records global suppression; full skipped-ceiling and local traversal stop diagnostics remain deferred.

Verification:

- `mise run verify` passed after implementation.
- Commit hooks passed on the implementation commit, including Rust format, clippy, tests, gitleaks, and conventional commit validation.
- GitNexus staged change detection before implementation commit reported medium risk with two expected affected old discovery/cache-resolution flows.
- GitNexus index refreshed after implementation; worktree index at `304fc43fc0848db809905d67ccd891041c23bc46` on `2026-06-30T21:17:29.820Z`.
- Follow-up fix verification ran `mise run verify` from the worktree: fmt, clippy, deny, unit/doc, integration, and e2e passed. Unit summary: `2226` passed. Integration summary: `58` passed. E2E summary: `102` passed.
- GitNexus worktree index refreshed after follow-up fix; index at `555eb9b2f5eb40995246863605170bd66e9e6efa` on `2026-07-01T13:10:41.802Z`.

Review focus:

- Confirm new discovery semantics match the PRD and this issue's acceptance criteria.
- Check old/new discovery coexistence around `processor.rs` and `processor_old.rs`.
- Check that ignored-path filtering being a temporary empty-input stub is acceptable for this slice.
- Check test coverage for fallback precedence, boundary markers, exact marker names, and suppressed global inputs.
- Check whether the remaining deferred `DiscoveryReport` diagnostics should move before the diagnostics migration.
