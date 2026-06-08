---
title: 07-phase-2-environment-config-discovery
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`

## What to build

Implement mechanical Environment Config path discovery in Phase 2 with deterministic source precedence, exact lowercase multi-format candidate checks, and non-fatal missing behavior.

This slice implements the previously stubbed **`GlobalConfigProbe`** and **`DiscoveryEngine::find_global()`** method. It covers mechanical Environment Config path resolution (`LITHOS_CONFIG_FILE`, XDG, user, system), returning a **`GlobalDiscoveryResult`** containing the mechanical winner and non-winning same-tier alternatives, while keeping parsing, format stability, and Config-owned classification out of Discovery. For the MVP, recognized config filenames are exact lowercase conventions only; Discovery does not scan for, correct, or accept mis-cased filenames.

## Acceptance criteria

- [ ] **Global Discovery Implementation**:
    - [ ] Implement `GlobalConfigProbe` in `discovery/probe.rs` to find Environment Config path candidates without importing Config-owned types.
    - [ ] Replace the stubbed `DiscoveryEngine::find_global()` in `discovery/engine.rs` with an implementation returning `GlobalDiscoveryResult`.
    - [ ] Add `GLOBAL_MARKER_FILES` or an equivalent Discovery-owned marker pattern constant for `lithos.{toml,json,yaml,yml}` under generated global base directories.
- [ ] Environment Config source precedence is implemented using ranked **`GlobalSourceType`**: `EnvVar(0)` > `XdgConfig(1)` > `UserConfig(2)` > `SystemConfig(3)`. The environment override name is **`LITHOS_CONFIG_FILE`**.
- [ ] Each tier supports structured format candidates (`toml`, `json`, `yaml`, `yml`) with deterministic selection via **`discovery::selector::select_candidate()`**.
- [ ] Missing Environment Config at any tier is treated as a non-error and discovery continues to the next tier.
- [ ] Result populates `alternatives: Vec<FoundRootMarker>` with non-winning candidates found at the winning tier; the selected `marker` is not duplicated in `alternatives`.
- [ ] Core suppression input for `--no-global-config` suppresses Environment Config lookup entirely. Prefer a dedicated `GlobalDiscoveryInput<'_>` over overloading vault-specific `DiscoveryInput<'_>`.
- [ ] Mis-cased recognized filenames are not corrected, accepted, or scanned for. The MVP places filename casing correctness on the user and only recognizes exact lowercase conventional names.
- [ ] Explicit missing or invalid user-provided paths (`--vault`, `LITHOS_VAULT`, `LITHOS_CONFIG_FILE`) include a helpful hint that Lithos config filenames must use lowercase conventional names such as `lithos.toml`, `lithos.json`, `lithos.yaml`, or `lithos.yml`.
- [ ] `GlobalDiscoveryResult` does not carry case-correction warnings; normal global tier misses remain soft non-errors.
- [ ] Config-side mapping converts `GlobalDiscoveryResult` into `ConfigDiscoveryResult.global` using Config-owned global file classification in `config/root.rs` or another Config-owned module. Discovery must not perform this classification.
- [ ] Unit/integration tests cover source precedence, suppression behavior, no-config behavior, exact lowercase candidate behavior, helpful explicit-path error hints, same-tier alternatives, and Config-side global mapping.

## Agent Brief

**Category:** enhancement
**Summary:** Implement Environment Config discovery with deterministic tier precedence and soft-miss behavior.

**Current behavior:**
Environment Config lookup is partially hardcoded and does not expose consistent precedence semantics across all supported locations/formats.

**Desired behavior:**
Mechanical Environment Config path discovery checks tiers in documented order, finds exact lowercase candidates at the highest-priority tier, and picks a mechanical winner using `StructuredFileFormat::PRECEDENCE` through `discovery::selector::select_candidate()`. Discovery does not perform case-correction scans; if an explicit user-provided path is missing or invalid, the user-facing error should remind the user that Lithos config filenames must use lowercase conventional names.

**Key interfaces:**
- `discovery::engine::DiscoveryEngine::find_global()`
- `discovery::policy::GlobalSourceType` (ranked)
- `discovery::engine::GlobalDiscoveryResult` (winner + alternatives)
- `discovery::selector::select_candidate()`
- `GlobalDiscoveryInput<'_>` or equivalent core suppression input for `--no-global-config`
- Config-owned mapping in `config/root.rs` from `GlobalDiscoveryResult` to `ConfigDiscoveryResult.global`

**Boundary Note:**
Phase 2 is "Dumb" inside `lithos-core/src/discovery/`. Discovery may return only path/source/format metadata (`FoundRootMarker`, `GlobalDiscoveryResult`) and fatal/explicit-path diagnostics. It must not import `GlobalConfigLocation`, `ConfigLocation`, `DiscoveredConfigFile`, `ConfigDiscoveryResult`, `ConfigWarning`, or any other Config-owned type. Discovery must not perform case-correction scans or return case-correction warning diagnostics in the MVP.

Config classification happens after Discovery returns. The mapping from `GlobalDiscoveryResult` into `ConfigDiscoveryResult.global` belongs in Config-owned code, preferably `config/root.rs`; `config/discovery.rs::ConfigDiscoveryPipeline` should continue to consume already-classified `ConfigDiscoveryResult` values and read/query the selected files.

**Out of scope:**
- Local (vault) config discovery
- Format stability (History-aware promotion)
- Config parsing, validation, merging, and hashing
- CLI command wiring beyond defining the core suppression input consumed by CLI later

## Completed dependencies

- `.scratch/root-config-discovery/02-local-candidate-generation.md`
- `.scratch/root-config-discovery/03-candidate-selection-format-stability.md`
- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`

## Triage Update (2026-06-05)

> *This was generated by AI during triage.*

The prerequisite issues are completed. This issue is ready for an AFK agent after
the architecture clarifications above: Discovery remains a mechanical path/source
finder, while Config owns global location classification and selected-file
consumption.

## Agent Brief Addendum (2026-06-05)

> *This was generated by AI during triage.*

**Category:** enhancement
**Summary:** Complete Environment Config path discovery and Config-owned global mapping without expanding into parsing, CLI wiring, or local discovery.

**Current behavior:**
`DiscoveryEngine::find_global()` returns an empty stub result. `GlobalConfigProbe` exists as an unwired seam. `GlobalDiscoveryResult.warnings` still uses the vault-specific warning type, and `GlobalSourceType` documentation still references `LITHOS_CONFIG` even though the approved Environment Config override is `LITHOS_CONFIG_FILE`. Config loading currently builds `ConfigDiscoveryResult` from only vault discovery, so `ConfigDiscoveryPipeline` receives no selected Environment Config path.

**Desired behavior:**
Discovery should mechanically check Environment Config candidates in strict tier order: `LITHOS_CONFIG_FILE`, XDG config home, user config fallback, then system config fallback. Missing files at every tier are non-fatal and produce no selected global marker. If one or more candidates exist at the first tier with any matches, Discovery selects the highest-precedence structured format through `discovery::selector::select_candidate()`, reports non-winning same-tier candidates as alternatives, and does not inspect lower-priority tiers. Mis-cased recognized filenames should emit Discovery-owned corrective warnings. Suppression input for `--no-global-config` should bypass Environment Config lookup entirely.

**Key interfaces:**
- `DiscoveryEngine::find_global()` should accept global-specific input, preferably `GlobalDiscoveryInput<'_>`, rather than overloading vault-specific `DiscoveryInput<'_>`.
- `GlobalConfigProbe` should enumerate `lithos.{toml,json,yaml,yml}` under generated global base directories and should remain Discovery-owned.
- `GlobalDiscoveryResult` should contain `marker`, `alternatives`, `source`, and Discovery-owned warnings, not `VaultDiscoveryWarning`.
- `GlobalSourceType` should use documented ranks `EnvVar(0)`, `XdgConfig(1)`, `UserConfig(2)`, `SystemConfig(3)` and document `LITHOS_CONFIG_FILE`.
- Config-owned mapping should convert `GlobalDiscoveryResult` into `ConfigDiscoveryResult.global` with `ConfigLocation::Global(...)`; Discovery must not import Config-owned location types.
- `ConfigDiscoveryPipeline` should continue consuming already-classified `ConfigDiscoveryResult` values and should not perform mechanical path discovery.

**Acceptance criteria:**
- [ ] `find_global()` returns `None`/empty alternatives/non-fatal warnings when no Environment Config candidates exist.
- [ ] `LITHOS_CONFIG_FILE` candidates outrank XDG, user, and system candidates.
- [ ] XDG candidates outrank user and system candidates; user candidates outrank system candidates.
- [ ] Same-tier multi-format candidates are selected with `select_candidate()` and non-winning same-tier candidates are returned in `alternatives` without duplicating the winner.
- [ ] Missing files in one tier do not error and do not prevent checking lower-priority tiers.
- [ ] Global suppression input bypasses all Environment Config lookup and returns no marker or alternatives.
- [ ] Mis-cased recognized global filenames emit Discovery-owned corrective warnings.
- [ ] Config-owned mapping classifies the selected global marker as `ConfigLocation::Global(...)` and preserves path, base, and format metadata.
- [ ] Unit tests follow the repository unit test naming standards and cover the discovery and mapping behaviors above.

**Out of scope:**
- Local (vault) config discovery changes.
- Format stability/history-aware promotion.
- Config parsing, validation, merging, hashing, and storage migrations.
- CLI flag wiring beyond defining core suppression input for later consumption.
- Adding new external dependencies unless separately justified through the dependency registry.

## Approved Implementation Notes (2026-06-05)

> *This was generated by AI during triage.*

**Resolved design decisions:**
- `GlobalSourceType::EnvVar` must not carry a `PathBuf`; it should mirror `VaultSourceType::EnvVar` as a source classification only. The selected path is already carried by `GlobalDiscoveryResult.marker: Option<FoundRootMarker>`.
- Config-owned `GlobalConfigLocation` variants should not carry `PathBuf` for this slice. `DiscoveredConfigFile` already embeds the selected `path`, `base`, `format`, and `location`; duplicating the path inside `ConfigLocation::Global(...)` is unnecessary.
- `GlobalDiscoveryResult.warnings` should use a Discovery-owned warning shape. A dedicated `GlobalDiscoveryWarning` wrapped by `DiscoveryWarning` is acceptable and preferred for parity with the existing vault-specific diagnostic channel.
- Discovery must remain a mechanical path/source/format finder. Config-owned modules perform all `ConfigLocation::Global(...)` classification after `GlobalDiscoveryResult` is returned.

**Approved TDD execution plan:**
- Start with `DiscoveryEngine::find_global()` behavior tests in `discovery::engine` using Structure A unit test organization and verb-first names.
- First tracer bullet: `find_global::returns_none_when_no_global_config_exists` should fail against the stub, then pass with minimal no-config behavior.
- Add `GlobalDiscoveryInput<'_>` with explicit, testable inputs for `env_config_file`, XDG base, user base, system base, and `suppress_global`. Do not read process environment directly in unit tests.
- Add suppression behavior: `find_global::returns_none_when_global_lookup_is_suppressed`.
- Add `GlobalConfigProbe` lookup behavior in `discovery::probe` for `lithos.{toml,json,yaml,yml}` under a supplied base directory.
- Add source precedence behavior in vertical slices: env var over XDG/user/system, XDG over user/system, user over system.
- Add same-tier format selection behavior using `select_candidate()` and assert alternatives contain only non-winning same-tier candidates.
- Add missing-tier continuation behavior: missing env/XDG/user candidates continue to lower tiers without error.
- Add case-correction diagnostics with `GlobalDiscoveryWarning` or `DiscoveryWarning::Global(...)`, using deterministic local filesystem setup.
- Add Config-owned mapping tests in `config::root` proving global markers become `ConfigDiscoveryResult.global` with `ConfigLocation::Global(...)` while preserving path, base, and format.
- Wire `ConfigDiscoveryResult` construction so Config can combine vault and global discovery outputs without making Discovery import Config-owned types.

**Implementation guardrails:**
- Every implementation step must run in `.worktrees/root-config-discovery/07-phase-2-environment-config-discovery`, never the main worktree.
- Every implementation and review subagent must verify the dedicated worktree before reading or editing files.
- Use GitNexus impact analysis before editing Rust symbols.
- Follow `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md` for all new tests.
- If implementation reveals a material deviation from this issue or plan, stop and update this issue before continuing.

## Implementation Notes Update (2026-06-08)

> *This was generated during acceptance review and implementation cleanup.*

**Clarified MVP decisions:**

- `LITHOS_CONFIG_FILE` is treated as an Environment Config directory/base for
  this MVP, mirroring `LITHOS_VAULT` source semantics. Discovery probes that
  base for exact lowercase conventional filenames. Supporting both file and
  directory inputs may be revisited later with an `FsPath`-style abstraction,
  but is out of scope for this slice.
- Config only needs path, base, and format handoff data from Discovery for this
  slice. Reintroducing Config-owned source/location taxonomy in `config/` would
  duplicate Discovery responsibilities and risks drifting outside Config's
  domain boundary.
- The top-level acceptance criteria are authoritative for casing behavior:
  Discovery recognizes exact lowercase conventional filenames only and does not
  scan for, correct, accept, or warn about mis-cased filenames. Older addendum
  references to case-correction diagnostics are superseded.

**Acceptance review follow-up:**

- Discovery implementation behavior is accepted under the clarified MVP
  decisions above.
- Discovery module test suites should be cleaned up to fully align with
  `docs/engineering/testing/unit.md`,
  `docs/engineering/testing/unit-naming.md`, and Rust best-practices testing
  guidance: Structure A for multi-unit files, focused one-behavior tests,
  canonical module names, verb-first names, and explicit Arrange/Act/Assert
  discipline.
