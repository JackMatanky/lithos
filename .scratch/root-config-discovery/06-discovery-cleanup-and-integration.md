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
        - [ ] **`MarkerPattern` (Internal)**: `{ prefix: &'static str, is_nested: bool }` - used to generate candidate filenames across all `StructuredFileFormat` variants.
        - [ ] **`ROOT_MARKER_FILES`**: Define as a `pub(crate)` constant used by `VaultRootProbe`.
    - [ ] **Policy (`discovery/policy.rs`)**:
        - [ ] **`SourceType`**: `ExplicitFlag`, `EnvironmentVariable`, `AscendingWalk`.
        - [ ] **`DiscoveryPolicy`**: Precedence (`Vec<SourceType>`), `allow_marker_at_ceiling: bool`, and `strict_overrides: bool`.
    - [ ] **Engine (`discovery/engine.rs`)**:
        - [ ] **`DiscoveryEngine`**: Implement with the following interface:
            ```rust
            pub struct DiscoveryEngine { policy: DiscoveryPolicy }
            impl DiscoveryEngine {
                pub fn new(policy: DiscoveryPolicy) -> Self { ... }
                pub fn discover_vault(&self, input: DiscoveryInput<'_>) -> Result<VaultDiscoveryResult, DiscoveryError> {
                    // 1. Resolution: Transform DiscoveryInput + Policy -> DiscoveryBoundaries
                    // 2. Precedence: Iterate through Policy.precedence (Explicit -> Env -> Walk)
                    // 3. Traversal: For AscendingWalk, drive AscendingWalker + VaultRootProbe
                    // 4. Result: Assemble and return VaultDiscoveryResult
                }
            }
            ```
        - [ ] **`DiscoveryInput` (Raw)**: `flag_path`, `env_path`, `cwd`, and `ceiling_dirs_raw` (OsStr).
        - [ ] **`VaultDiscoveryResult`**: `root: Option<PathBuf>`, `marker: Option<FoundRootMarker>`, `source: Option<SourceType>`, `warnings: Vec<DiscoveryWarning>`.
- [ ] **Context Cleanup & Normalization**:
    - [ ] **Delete `discovery/resolver.rs`** entirely after migrating logic.
    - [ ] Move `LocalDiscoveryWarning` and `FormatDiscoveryWarning` to `config/diagnostics.rs`.
    - [ ] Rename `RootResolutionWarning` to `VaultDiscoveryWarning` in `discovery/diagnostics.rs`.
- [ ] **Config Integration**:
    - [ ] Refactor `config/discovery.rs` (`DiscoveryEngine`) to call the new `discovery::engine::DiscoveryEngine::discover_vault`.
    - [ ] Implement mapping from Discovery outputs to `config::DiscoveredConfigFile` in `config/root.rs`.
    - [ ] **Delete** legacy manual scanning in `config/discovery.rs`.

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
Maintain strict context boundaries. Discovery handles the "how to find" (using dumb constants like `ROOT_MARKER_FILES`); Config handles the "what it means" (using rich enums like `LocalConfigLocation`). Mapping occurs at the `config/root.rs` seam.

Maintain strict context boundaries. Discovery handles path-finding via un-classified probes; Config handles the rich domain classification (`LocalConfigLocation`). `07-phase-2-environment-config-discovery.md` will later introduce `GlobalConfigProbe` and `discover_global`.

## Blocked by

- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
