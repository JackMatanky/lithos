# Discovery & Findings

## Issue Scope
- Build `DiscoveryContext<'a>`, `DiscoveryFlags<'a>`, and `DiscoveryEnv<'a>` as the discovery-owned input contract.
- Build boundary output/report/error/policy contracts without traversal, probing, selection, Config integration, or CLI commands.
- Add a minimal app/Bootstrapper seam that acquires invocation context and can be tested without running DiscoveryService or Config.

## Existing Planning Context
- Previous root `task_plan.md`, `findings.md`, and `progress.md` were for a completed filesystem/indexer rename and are being replaced for this session.

## Graph Context
- `graphify-out/GRAPH_REPORT.md` exists and identifies `Discovery Builder Handle` and `Vault Global From` communities relevant to config/discovery boundaries.

## Initial Code Search
- Existing discovery files are under `lithos-core/src/discovery/`.
- Existing result names include `VaultDiscoveryResult` and `GlobalDiscoveryResult` in `discovery/engine.rs`.
- `DiscoveryResult` also exists in config and schema modules, so new discovery result naming may need careful module qualification.
- Initial search did not find a current Rust `Bootstrapper` symbol by name.

## Domain and ADR Notes
- `app/` exists as a composition-root module but currently contains documentation only.
- Current `config::Builder::load()` still constructs and invokes `DiscoveryEngine` directly; this issue should not move that pipeline yet.
- ADR and scratch decisions say Bootstrapper owns context acquisition, while this issue narrows the first slice to building `DiscoveryContext` without running discovery or config.
- Existing `probe.rs` defines `ROOT_MARKER_FILES` and `GLOBAL_MARKER_FILES`; the issue wants pattern-oriented names in `policy.rs`, so add policy aliases/contracts rather than refactor traversal/probing.

## Impact Analysis
- `discovery/mod.rs`: LOW, 0 direct dependents, 0 affected processes.
- `DiscoveryError`: LOW, 0 direct dependents, 0 affected processes.
- `DiscoveryPolicy`: LOW, 0 direct dependents, 0 affected processes.
- `app/mod.rs`: LOW, 0 direct dependents, 0 affected processes.
