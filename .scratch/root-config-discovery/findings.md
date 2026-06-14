# Design Research Findings

## Constraint

Redesign is additive only. Old code (`engine.rs`, `diagnostics.rs`, `DiscoveryPolicy`, `VaultRootProbe`/`GlobalRootProbe`, flat error enum) stays untouched. No deletions, no breaking changes to existing call sites. Legacy retirement is a separate future issue.

## Processor model

```
DiscoveryProcessor<'ctx, P> {
    config: &DiscoveryServiceConfig,   // from DiscoveryService
    ctx: &DiscoveryContext,            // from Bootstrapper
    vault: Vec<CandidatePath>,         // accumulator
    global: Vec<CandidatePath>,        // accumulator
    report: DiscoveryReport,           // accumulator
    phase: P,                          // phase-specific data
}
```

6 phases: `Init → FlagOverride → EnvOverride → AscendingTraversal → GlobalResolution → Finalized`.

Phase structs carry minimal data — most state read from `ctx`/`config` directly.

## Port shape

```rust
fn discover(&self, ctx: &DiscoveryContext<'_>)
    -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>;
```

Tuple return. No `DiscoveryOutput` wrapper struct needed.

## FolderProbe

```rust
struct FolderProbe { patterns: &'static [MarkerPattern] }
fn probe(&self, dir: &DirPath) -> Vec<CandidatePath>
```

Infallible (paths pre-validated). Added alongside existing `VaultRootProbe`/`GlobalRootProbe` without modifying them.

## Branching strategy

`EnvOverride` phase exposes a `branch_strategy()` method returning:

```rust
enum Branch {
    VaultProbedSkipGlobal,   // vault override + config override
    VaultProbedRunGlobal,    // vault override only
    AscendSkipGlobal,        // config override only
    AscendThenGlobal,        // neither
}
```

Driven by `(has_vault_override, has_config_override)` — determined during `FlagOverride → EnvOverride` transition from flag/env precedence.

## FolderProbe marker path construction

```
non-nested (prefix = "lithos"):  dir.join("lithos").set_extension("toml") → dir/lithos.toml
nested (prefix = ".lithos/config"): dir.join(".lithos/config").set_extension("toml") → dir/.lithos/config.toml
```

Same logic as existing probes.

## Ceiling handling

Parsed in `FlagOverride` from `ctx.env().ceiling_dirs_raw()` using `env::split_paths`. Valid directories → `valid_ceilings: Vec<DirPath>` carried in phase data. Invalid entries → `report.skipped_ceilings`. Used by `AscendingTraversal` via `BoundedAscent` from walk.rs.

## Boundary (project marker) handling

AscendingTraversal checks `config.boundary_markers` at each directory (probe-then-stop). Uses `dir.join(marker).exists()` — infallible check. `BoundedAscent` from walk.rs handles ceiling bounds; boundary markers checked in the loop body.
