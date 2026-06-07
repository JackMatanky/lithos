# Progress: Phase 2 Environment Discovery Architecture Refactor

## 2026-06-07

- Loaded `planning-with-files` as requested.
- Loaded `receiving-code-review` to process technical corrections rigorously.
- Ran session catchup in dedicated worktree; no prior planning context reported.
- Created persistent planning files in dedicated worktree:
  - `task_plan.md`
  - `findings.md`
  - `progress.md`
- Revised plan around corrected architecture:
  - Config receives selected path data only.
  - Discovery warnings stay in Discovery/reporting flow.
  - `GLOBAL_MARKER_FILES` keeps name and gains both base and nested marker patterns.
  - `GlobalConfigProbe` should be renamed to remove Config from Discovery naming.
  - Avoid short-term fixes that preserve boundary violations.
- Incorporated follow-up corrections:
  - Keep `GlobalSourceType` name.
  - Use `GlobalSourceDirectory` for the directory enum.
  - Reuse `DiscoveredConfigFile` as the path-only handoff by removing `location`, instead of adding `SelectedConfigPath`.
  - Rename `GlobalConfigProbe` to `GlobalRootProbe`.
  - Do not split `probe.rs`.
  - Rename `FoundRootMarker` to `DiscoveredMarker`.
  - Move global warning handling into `DiscoveryEngine` flow, keeping `GlobalRootProbe::probe()` plain like `VaultRootProbe::probe()`.

## Current Status

- Latest review corrections are being implemented.
- New target: shared case-correction helper for vault/global, shared marker selection assembly, `global_precedence: Vec<GlobalSourceType>`, remove `find_known_vault()` workaround from Builder, and make Builder obtain vault root through Discovery.
- Previous verification remains useful as baseline only; fresh verification required after this implementation pass.

## Discovery Architecture Review

- Read `lithos-core/src/discovery/CONTEXT.md`.
- Listed Discovery source files.
- Queried GitNexus for Discovery concepts; results were weak/stale for branch-level review, so source inspection is primary.
- Read every Discovery module: `mod.rs`, `engine.rs`, `probe.rs`, `policy.rs`, `walk.rs`, `selector.rs`, `diagnostics.rs`, `error.rs`.
- Read ADR 009 for config/discovery historical context.
- Dispatched an independent explore subagent over `discovery/` and adjacent config call sites.
- Recorded cross-check findings in `findings.md`.

## Implementation Notes

- Added red tests for explicit vault roots discovering marker files, nested global markers at `lithos/config.*`, and Config retaining path-only global handoff without Discovery source identity.
- Renamed `FoundRootMarker` to `DiscoveredMarker`.
- Renamed `GlobalConfigProbe` to `GlobalRootProbe` and kept it in `probe.rs`.
- Expanded `GLOBAL_MARKER_FILES` to include both `lithos` and `lithos/config` marker patterns.
- Added `GlobalSourceDirectory` and `DiscoveryPolicy::global_precedence` while keeping `GlobalSourceType` unchanged.
- Renamed `DiscoveryPolicy::precedence` to `vault_precedence`.
- Replaced fixed global XDG/user/system fields with caller-provided `GlobalDirectoryCandidate` values.
- Added `DiscoveryEngine::find_known_vault()` and updated `Builder::load()` to use it.
- Moved global case-correction warning orchestration into `DiscoveryEngine`.
- Fixed explicit/env vault override resolution so a known vault root is still probed for marker files.
- Removed `location` and `warnings` from `ConfigDiscoveryResult`/`DiscoveredConfigFile` handoff into Config.
- Deleted `config/location.rs` and `config/candidates.rs`; Config no longer owns marker candidate path generation or filesystem candidate probing.
- Added architecture tests locking the Config/Discovery boundary and preventing `/etc/lithos` hardcoding in `Builder`.

## Latest Implementation Pass

- Updated `task_plan.md` and `findings.md` with the latest review decisions before production edits.
- TDD required for: vault case-correction warning, global env path as root directory, shared selection behavior, and Builder using Discovery rather than a known root.
- RED: `cargo test -p lithos-core --test architecture builder_must_not_use_known_vault_root_discovery_shortcut` failed because Builder still called `find_known_vault()`.
- RED: `cargo test -p lithos-core prefers_environment_root_over_global_base_directories` failed to compile because production still used `env_file` and `Vec<GlobalSourceDirectory>`.
- Implemented shared `case_correction_markers(patterns, base)` and used it for vault and global discovery.
- Added shared selector helpers for dedupe and selected-marker/alternatives splitting; no `DiscoverySelector` trait was introduced.
- Changed `DiscoveryPolicy::global_precedence` to `Vec<GlobalSourceType>`.
- Changed `GlobalDiscoveryInput` from `env_file` to `env_path`; global env source now probes a root directory.
- Removed `DiscoveryEngine::find_known_vault()` and changed `Builder::load()` to call `find_vault()` from its discovery start directory.
- Renamed Builder's private input field from `vault_root` to `start_dir` to avoid claiming Builder already knows the root.
- Verification: focused builder/discovery/policy/architecture tests passed after implementation.
- Verification: `mise run fmt && mise run lint && mise run test:unit:core` passed; core run reported 1555 unit tests and 201 doc tests.
- Verification: `mise run test` passed; full suite reported 1556 unit tests, 201 doc tests, 42 integration tests, and 1 e2e test.
- Verification: boundary grep found no `find_known_vault`, `env_file`, directory-only global precedence, `GlobalRootProbe::dedupe`, `resolve_override`, or old global case helpers in source.
- Verification: `git diff --check` produced no whitespace errors.
- GitNexus: `gitnexus_detect_changes(scope: all)` again reported no changes despite the real git diff; treat as stale/incomplete graph data for this worktree.

## Verification

- RED observed: `mise run test:unit:core` failed on `returns_global_file_without_source_identity` before implementation.
- GREEN: `mise run test:unit:core` passed with 1553 core unit tests and 201 doc tests after implementation.
- GREEN after follow-up edits: `mise run test:unit:core` passed with 1556 core unit tests and 201 doc tests.
- GREEN: `cargo test -p lithos-core --test architecture` passed with 8 tests.
- Lint: `mise run lint` passed after fixing clippy warnings.
- Format: `mise run fmt` passed after the final edits.
- Full suite: `mise run test` passed with 1557 unit tests, 201 doc tests, 41 integration tests, and 1 e2e test.
- GitNexus: `gitnexus_detect_changes(scope: all)` returned low risk but stale/incomplete changed-symbol data.
