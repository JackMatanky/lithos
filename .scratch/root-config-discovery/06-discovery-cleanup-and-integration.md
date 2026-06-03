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
        - [ ] **`DiscoveryBoundaries`**: Resolved context containing `start_dir` (canonicalized) and `ceilings` (HashSet of canonicalized physical paths).
    - [ ] **Detection (`discovery/probe.rs`)**:
        - [ ] **`DiscoveryProbe<Output>` Trait**: Interface for directory-level probing.
        - [ ] **`VaultRootProbe`**: Uses internal `MarkerPattern` templates to find root markers.
        - [ ] **`GlobalConfigProbe`**: Uses `GLOBAL_MARKER_FILES` to find environment-level config.
        - [ ] **`MarkerPattern` (Internal)**: `{ prefix: &'static str, is_nested: bool }` - used to generate candidate filenames across all `StructuredFileFormat` variants.
        - [ ] **`ROOT_MARKER_FILES`** & **`GLOBAL_MARKER_FILES`**: Define as `pub(crate)` constants used by probes.
    - [ ] **Policy (`discovery/policy.rs`)**:
        - [ ] **`VaultSourceType`**: `ExplicitFlag(0)`, `EnvVar(1)`, `AscendingWalk(2)`.
        - [ ] **`GlobalSourceType`**: `EnvVar(0)`, `XdgConfig(1)`, `UserConfig(2)`, `SystemConfig(3)`.
        - [ ] Implement `rank() -> u8` for both to drive deterministic tier-precedence.
        - [ ] **`DiscoveryPolicy`**: Precedence (`Vec<VaultSourceType>`), `allow_marker_at_ceiling: bool`, and `strict_overrides: bool`.
    - [ ] **Selector (`discovery/selector.rs`)**:
        - [ ] **`select_candidate()`**: Pure function that picks a winner from `Vec<FoundRootMarker>` using `StructuredFileFormat::PRECEDENCE`.
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
    - [ ] Move `LocalDiscoveryWarning` and `FormatDiscoveryWarning` to `config/diagnostics.rs`.
    - [ ] Rename `RootResolutionWarning` to `VaultDiscoveryWarning` in `discovery/diagnostics.rs`.
- [ ] **Config Integration**:
    - [ ] Refactor `config/discovery.rs` into a "Consolidator" that is **passed** a `discovery::ConfigDiscoveryResult` (from Phase 2).
    - [ ] `config/discovery.rs` orchestrates behavior **after** path selection: querying the `Repository` for cached views and combining them with the discovered paths.
    - [ ] Implement mapping from Discovery outputs to `config::DiscoveredConfigFile` in `config/root.rs`.
    - [ ] **Delete** legacy manual scanning and `find_vault_config`/`find_global_config` in `config/discovery.rs`.

## Migration Plan (from `resolver.rs`)

| Component | New Name | New Location |
| :--- | :--- | :--- |
| `RootResolver` | `DiscoveryEngine` | `engine.rs` |
| `RootResolverInput` | `DiscoveryInput` | `engine.rs` |
| `RootResolutionResult` | `VaultDiscoveryResult` | `engine.rs` |
| `RootResolutionSource` | `SourceType` | `policy.rs` |
| `RootResolutionPolicy` | `DiscoveryPolicy` | `policy.rs` |
| `RootResolutionError` | `DiscoveryError` | `error.rs` |
| `resolve_ascending` | `AscendingWalker` | `walk.rs` |
| `discover_marker` | `VaultRootProbe` | `probe.rs` |

## Agent Brief

**Category:** refactor
**Summary:** Transform Discovery into a Modular Engine (Traversal, Detection, Policy) and wire it into Config.

**Architecture Note:**
Maintain strict context boundaries. Discovery handles path-finding via un-classified probes (using internal helpers like `MarkerPattern`); Config handles the rich domain classification (`LocalConfigLocation`). `07-phase-2-environment-config-discovery.md` will later introduce `GlobalConfigProbe` and `find_global`.

## Blocked by

- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
