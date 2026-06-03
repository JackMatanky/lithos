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
        - [ ] Implement `AscendingWalker` iterator.
        - [ ] **`DiscoveryBoundaries`**: Resolved context containing `start_dir` (canonicalized) and `ceilings` (HashSet of canonicalized physical paths).
    - [ ] **Detection (`discovery/probe.rs`)**:
        - [ ] **`DiscoveryProbe<Output>` Trait**: Interface for directory-level probing.
        - [ ] **`VaultRootProbe`**: Uses internal `MarkerPattern` templates to find root markers.
        - [ ] **`MarkerPattern` (Internal)**: `{ prefix: &'static str, is_nested: bool }` - used to generate candidate filenames across all `StructuredFileFormat` variants.
    - [ ] **Policy (`discovery/policy.rs`)**:
        - [ ] **`SourceType`**: `ExplicitFlag`, `EnvironmentVariable`, `AscendingWalk`.
        - [ ] **`DiscoveryPolicy`**: Precedence (`Vec<SourceType>`), `allow_marker_at_ceiling: bool`, and `strict_overrides: bool`.
    - [ ] **Engine (`discovery/engine.rs`)**:
        - [ ] **`DiscoveryEngine`**: Orchestrator that transforms `DiscoveryInput` + `DiscoveryPolicy` into `DiscoveryBoundaries` and drives the search.
        - [ ] **`DiscoveryInput` (Raw)**: `flag_path`, `env_path`, `cwd`, and `ceiling_dirs_raw` (OsStr).
        - [ ] **`VaultDiscoveryResult`**: `root`, `marker`, `source`, `warnings`.
- [ ] **Context Cleanup & Normalization**:
    - [ ] **Delete `discovery/resolver.rs`** after migrating all logic to the modular structure.
    - [ ] Move `LocalDiscoveryWarning` and `FormatDiscoveryWarning` to `config/diagnostics.rs`.
    - [ ] **`ROOT_MARKER_FILES`**: Define as a public `&[&str]` constant in `discovery/mod.rs` containing all literal filenames that can trigger root discovery.
- [ ] **Config Integration**:
    - [ ] Refactor `config/discovery.rs` (`DiscoveryEngine`) to call the new `discovery::engine::DiscoveryEngine`.
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

## Blocked by

- `.scratch/root-config-discovery/05-move-discovery-module-boundary.md`
