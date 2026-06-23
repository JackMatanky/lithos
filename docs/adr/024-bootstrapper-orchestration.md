---
name: bootstrapper-orchestration
status: accepted
supersedes: [021]
date_proposed: 2026-06-11
date_decided: 2026-06-11
date_implemented:
stakeholders: [lithos-core maintainers, lithos-cli maintainers]
---

# ADR 024: Bootstrapper as the Single Orchestration Point

## Context

ADR 021 established `crates/app` as the composition root but left the
internal shape of orchestration undesign. Three concerns needed a home:

1. **Discovery** — locating the vault root and config file paths (owned by
   `discovery/`)
2. **Config loading** — reading discovered files, detecting staleness, merging
   layers, building the validated `Config` (owned by `config/`)
3. **Diagnostic surfacing** — process metadata (skipped overrides, ceiling
   violations) that the CLI needs for verbose output

Previously these were tangled: `config/Builder` ran discovery internally,
`discovery/` and `config/` both imported each other's types, and diagnostic
data was embedded in discovery result types that flowed to downstream
components that didn't need it. A single orchestration point is needed that:
- Owns the sequence: context acquisition → discovery → config loading
- Is the only place both `discovery/` and `config/` are imported
- Returns both the resolved config and the process metadata to the CLI

## Decision

We will introduce a `Bootstrapper` component in `app/` as the single
orchestration point for the bootstrap sequence.

**Architecture:**

- `Bootstrapper` is the only component that imports from both `discovery/` and
  `config/`.
- `discovery/` never imports from `config/`.
- `config/` imports exactly one data type from `discovery/` (`DiscoveryResult`)
  in exactly one place (`Builder::from_discovery()`).
- The CLI handler never calls `discovery/` or `config/` directly.

**Interface:**

```rust
pub struct Bootstrapper<R: Repository> {
    discovery: DiscoveryService,
    repo: R,
}

impl<R: Repository> Bootstrapper<R> {
    pub fn new(repo: R) -> Result<Self, ConfigError> {
        let discovery = DiscoveryService::default()
            .global_directories(resolve_platform_dirs())
            .build()?;
        Ok(Self { discovery, repo })
    }

    pub fn run(&self, input: InvocationInput<'_>)
        -> Result<BootstrapResult, BootstrapError>
    {
        let (result, report) = self.discovery.discover(input)?;
        let builder = Builder::from_discovery(&result, &self.repo)?;
        let config = builder.build()?;
        Ok(BootstrapResult { config, report })
    }
}

pub struct BootstrapResult {
    pub config: Config,
    pub report: DiscoveryReport,
}
```

**Responsibilities:**

1. Construct `DiscoveryService` with platform-resolved global directories.
2. Acquire per-invocation context (CWD, env vars, CLI flags).
3. Call `DiscoveryService::discover()` → `(DiscoveryResult, DiscoveryReport)`.
4. Emit `tracing::warn!` for skipped ceilings/overrides from `DiscoveryReport`.
5. Call `Builder::from_discovery()` + `Builder::build()` → `Config`.
6. Return `BootstrapResult { config, report }` to the CLI handler.

**Scope boundary:** The Bootstrapper does not construct `Config` itself, does
not run the indexer, and does not contain business logic. It wires components
that own that logic.

## Alternatives Considered

### Alternative 1: CLI-owned orchestration (pre-ADR 021 status quo)

- **Pros**: No new component; each CLI command can compose its own pipeline.
- **Cons**: Every CLI command duplicates orchestration logic. Adding a new
  executable adapter (TUI, daemon) requires duplicating the full bootstrap
  sequence. Discovery diagnostics and error handling vary per adapter for no
  architectural reason. Rejected by ADR 021.

### Alternative 2: Multiple `app/` flows (one per pipeline step)

- **Pros**: Finer granularity; components can be reused independently.
- **Cons**: Callers must still compose the sequence themselves — "which
  discovery service, then which builder?" The composition logic lives at the
  CLI layer, violating ADR 021's intent. No single place to add cross-cutting
  diagnostics.

### Alternative 3: Discovery still owned by config/Builder

- **Pros**: Single `load()` call, minimal call-site ceremony.
- **Cons**: `config/` imports discovery engine types. The bridge layer
  (`root.rs`) exists only to translate between structurally identical types.
  The hexagonal boundary between discovery and config is violated.

## Technical Validation

### Research Findings

- The Bootstrapper pattern is standard in hexagonal architecture composition
  roots: one component constructs ports, wires the pipeline, and exposes typed
  results to executable adapters.
- The architectural authority is ADR 021, which established `app/` for this
  purpose. This ADR instantiates that decision with a concrete component.
- `BootstrapResult` follows the `DiscoveryResult`/`DiscoveryReport` split
  established in the discovery redesign: process metadata (report) is
  separated from domain data (config) so downstream components never see
  orchestration diagnostics.

## Consequences

- **Positive**: `discovery/` and `config/` are fully independent hexagonal
  contexts. The only cross-import is `config/` importing `DiscoveryResult`
  as a pure data type in `from_discovery()`.
- **Positive**: CLI commands that need `DiscoveryReport` (e.g. `config where`)
  destructure `BootstrapResult` without calling discovery themselves.
- **Positive**: All bootstrap diagnostics flow through one point, making it
  easy to add cross-cutting behaviour (tracing, metrics, caching).
- **Positive**: Two-phase discovery (deferred) will add a second
  `discover()` call before `build()` without changing the CLI interface.
- **Negative**: Every CLI command that needs config goes through the
  Bootstrapper even if it doesn't need discovery (e.g. reading an already-known
  config file). This is negligible for MVP — the discovery query is cheap for
  an already-cached workspace.
- **Negative**: The `run()` method signature depends on both `DiscoveryResult`
  (from `discovery/`) and `Config` (from `config/`), meaning any change to
  either output type requires changes in three locations (the owning module,
  the Bootstrapper, and the CLI).
- **Risks**: If `run()` grows additional responsibilities (two-phase discovery,
  indexer triggering, vault health checks) without intentional guardrails, the
  Bootstrapper risks becoming a god function. Each new responsibility should
  be added as a separate method, not as a parameter to `run()`.

## References

- ADR 021 — `app` as composition root
- `docs/adr/discovery/0001-discovery-service-redesign.md` — discovery service API
- `docs/adr/config/0001-config-builder-decoupling.md` — config builder interface
- `.scratch/root-config-discovery/11-discovery-redesign-decisions.md` — full session decisions
