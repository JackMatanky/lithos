---
title: 06-discovery-cleanup-and-integration
category: refactor
label: ready-for-agent
status: open
date_created: 2026-06-03
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`

## What to build

Refactor the Discovery context into a "Modular Engine" architecture and integrate it into the Config orchestration layer.

This transformation breaks the monolithic `resolver.rs` into specialized components for **Traversal**, **Detection**, **Policy**, and **Orchestration**. It prioritizes a "Dumb Discovery" design where path-finding remains decoupled from Config-side classification taxonomy.

## Acceptance criteria

- [ ] **Core Discovery Architecture (The Modular Engine)**:
    - [ ] **Traversal (`discovery/walk.rs`)**:
        - [ ] **`AscendingWalker`**: Implement as an IO-aware iterator with the following attributes:
            - `current: Option<PathBuf>`: Current traversal state.
            - `visited: HashSet<PathBuf>`: Physical paths (canonicalized) to prevent symlink cycles.
            - `ceilings: HashSet<PathBuf>`: Physical paths (canonicalized) that terminate the walk.
        - [ ] **`DiscoveryBoundaries`**: Resolved context containing `start_dir` (canonicalized) and `ceilings` (HashSet of canonicalized physical paths). `pub(crate)` fields with accessor methods.
    - [ ] **Detection (`discovery/probe.rs`)**:
        - [ ] **`DiscoveryProbe<Output>` Trait**: Interface for directory-level probing.
        - [ ] **`VaultRootProbe`**: Uses internal `MarkerPattern` templates to find root markers.
        - [ ] **`GlobalConfigProbe` (stub)**: Minimal declaration only; full implementation deferred to Issue 07.
        - [ ] **`MarkerPattern` (Internal)**: `{ prefix: &'static str, is_nested: bool }` - used to generate candidate filenames across all `StructuredFileFormat` variants.
        - [ ] **`ROOT_MARKER_FILES`** & **`GLOBAL_MARKER_FILES`**: Define as `pub(crate)` constants used by probes.
    - [ ] **Policy (`discovery/policy.rs`)**:
        - [ ] **`VaultSourceType`**: `ExplicitFlag`, `EnvVar`, `AscendingWalk`. Derives `PartialOrd, Ord` for declaration-order precedence.
        - [ ] **`GlobalSourceType`**: `EnvVar`, `XdgConfig`, `UserConfig`, `SystemConfig`. Derives `PartialOrd, Ord` for declaration-order precedence.
        - [ ] Implement `rank() -> u8` for both to drive deterministic tier-precedence.
        - [ ] **`DiscoveryPolicy`**: Precedence (`Vec<VaultSourceType>`), `allow_marker_at_ceiling: bool`, and `strict_overrides: bool`.
    - [ ] **Selector (`discovery/selector.rs`)**:
        - [ ] **`select_candidate()`**: Pure function that picks a winner from `&[FoundRootMarker]` using `StructuredFileFormat::PRECEDENCE`.
        - [ ] **`promote_alternative()`**: Pure function that swaps the winner for an alternative if it matches a provided `StructuredFileFormat` (Stability Hint).
    - [ ] **Engine (`discovery/engine.rs`)**:
        - [ ] **`DiscoveryEngine`**: Implement with the following interface:
            ```rust
            pub struct DiscoveryEngine { policy: DiscoveryPolicy }
            impl DiscoveryEngine {
                pub fn new(policy: DiscoveryPolicy) -> Self { ... }
                pub fn find_vault(&self, input: DiscoveryInput<'_>) -> Result<VaultDiscoveryResult, DiscoveryError> {
                    // 1. Resolution: Transform DiscoveryInput + Policy -> DiscoveryBoundaries
                    // 2. Precedence: Iterate through Policy.precedence (Explicit -> Env -> Walk)
                    // 3. Traversal: For AscendingWalk, drive AscendingWalker + VaultRootProbe
                    // 4. Selection: Use select_candidate() to pick winner and populate alternatives
                    // 5. Result: Assemble and return VaultDiscoveryResult
                }
                pub fn find_global(&self, input: DiscoveryInput<'_>) -> Result<GlobalDiscoveryResult, DiscoveryError> {
                    // STUB: Returns None. Full implementation in Issue 07.
                    // 1. Precedence: Iterate GlobalSourceType tiers
                    // 2. Detection: Use GlobalConfigProbe at each tier
                    // 3. Selection: Use select_candidate() to pick winner and populate alternatives
                    // 4. Result: Assemble and return GlobalDiscoveryResult
                }
            }
            ```
        - [ ] **`DiscoveryInput` (Raw)**: `flag_path`, `env_path`, `cwd`, and `ceiling_dirs_raw` (OsStr).
        - [ ] **`VaultDiscoveryResult`**: `root: Option<PathBuf>`, `marker: Option<FoundRootMarker>`, `alternatives: Vec<FoundRootMarker>`, `source: Option<VaultSourceType>`, `warnings: Vec<DiscoveryWarning>`.
        - [ ] **`GlobalDiscoveryResult`**: `marker: Option<FoundRootMarker>`, `alternatives: Vec<FoundRootMarker>`, `source: Option<GlobalSourceType>`, `warnings: Vec<DiscoveryWarning>`.
- [ ] **Context Cleanup & Normalization**:
    - [ ] **Delete `discovery/resolver.rs`** entirely after migrating logic.
    - [ ] Create `config/diagnostics.rs` with `ConfigWarning` enum wrapping `LocalDiscoveryWarning` and `FormatDiscoveryWarning` (moved from `discovery/diagnostics.rs`).
    - [ ] `discovery::diagnostics::DiscoveryWarning` loses its `Local(...)` / `Format(...)` variants. Keeps only Discovery-owned variants: `RootResolution(VaultDiscoveryWarning)`, `CaseCorrection { .. }`.
    - [ ] Rename `RootResolutionWarning` to `VaultDiscoveryWarning` in `discovery/diagnostics.rs`.
    - [ ] **No cross-context imports**: `discovery::diagnostics` must NOT import from `config::diagnostics`, and vice versa.
- [ ] **Config Integration**:
    - [ ] Rename `config/discovery.rs::DiscoveryEngine` to `ConfigDiscoveryPipeline`.
    - [ ] Refactor `ConfigDiscoveryPipeline` to accept `config::ConfigDiscoveryResult` as input instead of `VaultRoot`.
    - [ ] `find_global_config()` / `find_vault_config()` remain private helpers inside `ConfigDiscoveryPipeline`.
    - [ ] Implement mapping from Discovery outputs to `config::DiscoveredConfigFile` in `config/root.rs`.
    - [ ] **Delete** legacy manual scanning and `find_vault_config`/`find_global_config` in `config/discovery.rs`.

## Migration Plan (from `resolver.rs`)

| Component | New Name | New Location |
| :--- | :--- | :--- |
| `RootResolver` | `DiscoveryEngine` | `engine.rs` |
| `RootResolverInput` | `DiscoveryInput` | `engine.rs` |
| `RootResolutionResult` | `VaultDiscoveryResult` | `engine.rs` |
| `RootResolutionSource` | `VaultSourceType` | `policy.rs` |
| `RootResolutionPolicy` | `DiscoveryPolicy` | `policy.rs` |
| `RootResolutionError` | `DiscoveryError` | `error.rs` |
| `resolve_ascending` | `AscendingWalker` | `walk.rs` |
| `discover_marker` | `VaultRootProbe` | `probe.rs` |

## Agent Brief

**Category:** refactor
**Summary:** Transform Discovery into a Modular Engine (Traversal, Detection, Policy) and wire it into Config.

**Architecture Note:**
Maintain strict context boundaries. Discovery handles path-finding via un-classified probes (using internal helpers like `MarkerPattern`); Config handles the rich domain classification (`LocalConfigLocation`). `07-phase-2-environment-config-discovery.md` will later introduce `GlobalConfigProbe` and `find_global`.

## Updated Agent Brief (2026-06-03 Triage Review)

**Category:** refactor
**Summary:** Transform monolithic `resolver.rs` into Modular Engine components and wire into Config.

**Architecture decisions (resolved):**

1. **`DiscoveryEngine` naming collision**: The new engine in `discovery/engine.rs` is named `DiscoveryEngine`. The existing Config-side orchestrator in `config/discovery.rs` is **renamed to `ConfigDiscoveryPipeline`**.
2. **`find_global` / `GlobalConfigProbe`**: **Stubbed.** Full implementation deferred to Issue 07. `find_global()` returns `None` (noop). `GlobalConfigProbe` type declared but minimal.
3. **`DiscoveryError`**: New enum preserving all `RootResolutionError` variants (renamed). `RootResolutionError` deleted after migration.
4. **`selector.rs` vs `candidates.rs`**: `discovery/selector.rs` operates on raw `FoundRootMarker` only (path+format level). `config/candidates.rs::select_config_candidate` stays on `DiscoveredConfigFile` (Config-classified). Different domain levels.
5. **Warning type ownership — NO CROSS-CONTEXT IMPORTS**: `LocalDiscoveryWarning` / `FormatDiscoveryWarning` move to `config/diagnostics.rs` under `ConfigWarning`. `discovery::diagnostics::DiscoveryWarning` keeps only Discovery-owned variants. Contexts NEVER import each other's diagnostics modules.
6. **`VaultSourceType`**: No explicit discriminants. Uses `#[derive(PartialOrd, Ord)]` on declaration order.
7. **`select_candidate()`**: Operates on `&[FoundRootMarker]`, not `Vec`.
8. **`DiscoveryBoundaries`**: `pub(crate)` with accessor methods.

**Acceptance criteria additions:**
- [ ] `RootResolutionError` → `DiscoveryError`: Preserve all variants (`ExplicitPathMissing`, `ExplicitPathNotDirectory`, `EnvironmentPathMissing`, `EnvironmentPathNotDirectory`, `CurrentDirectoryCanonicalize`, `CanonicalizePath`).
- [ ] `config/discovery.rs::DiscoveryEngine` renamed to `ConfigDiscoveryPipeline`. Receives `ConfigDiscoveryResult` instead of `VaultRoot`.
- [ ] `find_global()` in `discovery::engine::DiscoveryEngine` is a stub returning `None`.
- [ ] `discovery/selector.rs::select_candidate` operates on `&[FoundRootMarker]`.
- [ ] All `resolver.rs` tests (~630 lines) split across new module files, preserving coverage.
- [ ] `discovery/CONTEXT.md` updated: reference `DiscoveryEngine`, `AscendingWalker`, `VaultRootProbe`, `MarkerPattern` (not `RootResolver`).
- [ ] `VaultSourceType` / `GlobalSourceType`: no numeric discriminants; `#[derive(PartialOrd, Ord)]`.
- [ ] `discovery::diagnostics::DiscoveryWarning` removes `Local(...)` and `Format(...)` variants; keeps `RootResolution(VaultDiscoveryWarning)` and `CaseCorrection`.
- [ ] `config/diagnostics.rs` created with `ConfigWarning` wrapping `LocalDiscoveryWarning` + `FormatDiscoveryWarning`.

**Out of scope:**
- Full `GlobalConfigProbe` / `find_global` implementation (Issue 07).
- Config content parsing, validation, hashing.
- CLI command wiring (Issue 10).
- `vault_path` removal from `RawVaultConfig` (Issue 09).
- `config/candidates.rs::select_config_candidate` contract changes.

## TDD Plan

Vertical-slice tracer-bullet approach (RED → GREEN → REFACTOR per cycle). Three phases.

### Phase 1: Foundation — Extract New Modules (no behavior change, no Config touch)

| Cycle | RED test | GREEN implementation |
|-------|----------|---------------------|
| 1 | `discovery/error.rs`: `DiscoveryError` preserves all `RootResolutionError` variants via rename | Create `DiscoveryError` enum, migrate error formatting tests |
| 2 | `discovery/walk.rs`: `AscendingWalker` walks upward, stops at ceiling, detects symlink loops | Extract `resolve_ascending` + `parse_ceilings` logic, add `visited` set, migrate 8 tests |
| 3 | `discovery/walk.rs`: `DiscoveryBoundaries` accessors | Create `DiscoveryBoundaries` struct with `start_dir()` / `ceilings()` accessors |
| 4 | `discovery/probe.rs`: `VaultRootProbe` matches markers by prefix × format | Extract `discover_marker` logic with `MarkerPattern`, `ROOT_MARKER_FILES`, migrate 5 tests |
| 5 | `discovery/probe.rs`: `DiscoveryProbe<Output>` trait | Define trait with `probe(&self, dir: &Path) -> Result<Option<Output>>` |
| 6 | `discovery/policy.rs`: `VaultSourceType` + `GlobalSourceType` + `DiscoveryPolicy` | Extract from `RootResolutionSource` + `RootResolutionPolicy`, add `rank()`, migrate tests |
| 7 | `discovery/selector.rs`: `select_candidate(&[FoundRootMarker])` picks highest-precedence | Pure function using `StructuredFileFormat::PRECEDENCE`, new tests for selection |
| 8 | `discovery/selector.rs`: `promote_alternative()` swaps winner on stability hint | Pure function preserving alternatives order, new tests |
| 9 | `discovery/engine.rs`: `DiscoveryEngine::find_vault()` orchestrates walk → probe → select | Wire AscendingWalker → VaultRootProbe → select_candidate, migrate 5 resolve tests |

### Phase 2: Context Cleanup

| Cycle | Action | Verification |
|-------|--------|-------------|
| 10 | Create `config/diagnostics.rs`: `ConfigWarning` wrapping `LocalDiscoveryWarning` + `FormatDiscoveryWarning`. Remove these variants from `discovery::diagnostics::DiscoveryWarning`. | `discovery::diagnostics` has zero imports from `config`; all compile |
| 11 | Rename `RootResolutionWarning` → `VaultDiscoveryWarning` in `discovery/diagnostics.rs` | Compiles, tests pass |
| 12 | Delete `discovery/resolver.rs` | `git rm`, verify everything still compiles from new modules |
| 13 | Update `discovery/CONTEXT.md` with new terminology | Review |
| 14 | Update `discovery/mod.rs` to declare new module files | Compiles |

### Phase 3: Config Integration

| Cycle | Action | Verification |
|-------|--------|-------------|
| 15 | Rename `config/discovery.rs::DiscoveryEngine` → `ConfigDiscoveryPipeline` | config module compiles |
| 16 | Refactor `ConfigDiscoveryPipeline::run()` to accept `ConfigDiscoveryResult` instead of `VaultRoot` | Builder compiles, builder tests pass |
| 17 | Wire `discovery::engine::DiscoveryEngine` output → `config::DiscoveredConfigFile` mapping in `config/root.rs` | Pipeline integration test passes |
| 18 | Delete legacy `find_global_config` / `find_vault_config` from `config/discovery.rs` | Config tests pass |
| 19 | `ConfigDiscoveryResult.warnings: Vec<ConfigWarning>` (Config-owned). Remove `DiscoveryWarning` import from Config side. | Clean compile, no cross-context imports |

### Per-cycle checklist

- [ ] RED: test describes behavior through public interface
- [ ] GREEN: minimal implementation to pass
- [ ] REFACTOR: no duplication, no speculative features
- [ ] `mise run test:unit` passes for affected crate
- [ ] `cargo clippy` no new warnings

### Module visibility

All new types are `pub(crate)` unless explicitly needed wider.

### Context boundary enforcement

- `discovery/` must NOT import from `config/`
- `config/` may import Discovery output types (`FoundRootMarker`, `DiscoveryWarning`, `DiscoveryError`) but NOT implementation modules
- `discovery::diagnostics` and `config::diagnostics` are mutually exclusive — NO cross-context imports

## Blocked by

- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`

## Implementation Log

### Phase 1 (Cycles 1-9): Foundation — Extract from `resolver.rs`

**Files created:** `discovery/error.rs`, `discovery/walk.rs`, `discovery/probe.rs`, `discovery/selector.rs`, `discovery/policy.rs`, `discovery/engine.rs`
**Files modified:** `discovery/diagnostics.rs`, `discovery/marker.rs`, `discovery/mod.rs`, `discovery/CONTEXT.md`
**Files deleted:** `discovery/resolver.rs`

**Key decisions:**
- `#[allow(dead_code)]` preserved on all Phase-1 seam types pending orchestration wiring
- `AscendingWalker` yields directories; ceiling policy enforced by `DiscoveryEngine`
- `LocalDiscoveryWarning` / `FormatDiscoveryWarning` temporarily retained in `discovery/diagnostics.rs` to preserve compilation while Phase 2 migrates them

**Spec deviations resolved:**
- `DiscoveryWarning::Local`/`Format` variants retained for Phase 2 migration (compile-safe)
- `rank()` implemented on `VaultSourceType`/`GlobalSourceType` enums (correct placement)

**Quality gate review findings:**
- Fixed `pub mod diagnostics` → `pub(crate) mod diagnostics`, `pub mod marker` → `pub(crate) mod marker` (inconsistent visibility)
- Added `#[allow(dead_code)]` to `FormatDiscoveryWarning` (missing annotation)
- Fixed stale doc link in `marker.rs` (referenced deleted `resolver` module)
- 30 new tests, 1663/1633 baseline pass → 1663 total pass

### Phase 2 (Cycles 10-14): Context Cleanup

**Files created:** `config/diagnostics.rs`
**Files modified:** `discovery/diagnostics.rs`, `config/root.rs`, `config/candidates.rs`, `config/mod.rs`, `discovery/CONTEXT.md`

**Key decisions:**
- `LocalDiscoveryWarning` + `FormatDiscoveryWarning` moved to `config/diagnostics.rs` under `ConfigWarning` wrapper
- `discovery::diagnostics::DiscoveryWarning` keeps only Discovery-owned variants: `RootResolution(VaultDiscoveryWarning)`, `CaseCorrection`
- Zero cross-context imports: `discovery::diagnostics` does not import from `config`, `config::diagnostics` does not import from `discovery`
- `ConfigDiscoveryResult.warnings` changed from `Vec<DiscoveryWarning>` to `Vec<ConfigWarning>`
- `VaultDiscoveryWarning` renaming was pre-existing from Phase 1

**Quality gate review findings:** All clean. 1665 tests pass.

### Phase 3 (Cycles 15-18): Config Integration

**Files modified:** `config/discovery.rs`, `config/builder.rs`, `config/root.rs`, `config/mod.rs`

**Key decisions:**
- `config/discovery.rs::DiscoveryEngine` renamed to `ConfigDiscoveryPipeline`
- `ConfigDiscoveryPipeline::run()` signature: `(discovery_result: &ConfigDiscoveryResult, vault_id: Option<VaultId>, repo: &R)` — extra `vault_id` param needed for DB query routing
- `Builder.load()` uses `flag_path: Some(vault_root.as_path())` (not `cwd`) to skip ascending walk when vault root is already known
- Legacy `find_global_config`, `find_vault_config`, `scan_filesystem`, `global_config_paths` deleted from `config/discovery.rs`
- `ConfigDiscoveryResult::from_vault_discovery()` added to `config/root.rs` with path-component-safe `classify_local_config_location()`

**Quality gate review findings (Critical bugs fixed):**
1. `builder.rs`: `_entry` → `entry`; vault config path was hardcoded to `.lithos/lithos.toml` — fixed to use discovered `entry_path`
2. `builder.rs`: `DiscoveryEngine::find_vault()` was called with `cwd=vault_root` — fixed to use `flag_path=Some(vault_root.as_path())` to signal that vault root is already known
3. `root.rs`: `classify_local_config_location` used string matching — replaced with path component matching for cross-platform safety
4. `config/discovery.rs`: Added doc comment explaining `vault_id: None` branch semantics

### Verification

| Gate | Status |
| :--- | :----- |
| `cargo fmt --check` | ✅ clean |
| `cargo clippy --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo nextest run --workspace` | ✅ 1665/1665 pass |
| `mise run verify` | ✅ all gates pass |

**Test delta:** 1633 baseline → 1665 final (+32 new discovery tests)
**Files added:** 6 (`discovery/error.rs`, `walk.rs`, `probe.rs`, `selector.rs`, `policy.rs`, `engine.rs`, `config/diagnostics.rs`)
**Files deleted:** 2 (`discovery/resolver.rs`, `discovery/marker.rs`)
**Files modified:** 10

### Post-Implementation Review Fixes

After the initial implementation and quality gate checks, the following fixes were applied to adhere fully to Rust best practices and ACs:

1. **VaultDiscoveryWarning Location**: Moved `VaultDiscoveryWarning` from `error.rs` to `diagnostics.rs`, matching the exact specification from the Acceptance Criteria.
2. **Method Refactoring**: Refactored `validate_override()` into an associated method on `DiscoveryEngine` (`DiscoveryEngine::validate_override`) and `parse_ceilings()` into an associated method on `DiscoveryBoundaries` (`DiscoveryBoundaries::parse_ceilings`).
3. **Iterator Chains over Loops**: Refactored the manual `for` loops in `VaultRootProbe::probe` to use a cleaner, idiomatic Rust iterator chain (`.flat_map`, `.find_map`, `.transpose()`).
4. **Const Precedence**: Added `const PRECEDENCE` arrays to both `VaultSourceType` and `GlobalSourceType` to allow for idiomatic iteration over ordered source types, similar to `StructuredFileFormat`.
5. **FoundRootMarker Relocation**: Moved the `FoundRootMarker` struct out of `marker.rs` and directly into `engine.rs` alongside the `*DiscoveryResult` types, and deleted `marker.rs` to reduce module fragmentation for tightly coupled discovery result types.
