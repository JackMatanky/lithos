---
name: config-builder-decoupling
status: accepted
supersedes: [009, 021]
date_proposed: 2026-06-11
date_decided: 2026-06-11
date_implemented:
stakeholders: [traces-core maintainers]
---

# ADR 0001: Config Builder Decoupling

## Context

`config/Builder` was performing two roles simultaneously: config-domain
composition (merging raw configs, validating, persisting) and discovery
orchestration (running the `DiscoveryEngine` internally to locate vault root
and config files). The latter role required importing
`DiscoveryEngine`, `DiscoveryInput`, `GlobalDiscoveryInput`, and
`DiscoveryPolicy` — creating an architectural wire crossed.

A bridge layer in `config/root.rs` (`ConfigDiscoveryResult`,
`DiscoveredConfigFile`) converted discovery output types into config-native
types, but these were structurally identical to `DiscoveredMarker` from
`discovery/` — pure redundancy with an `into()` conversion that added no
meaning.

The resulting coupling meant any change to the discovery process (adding a new
source, changing the phase order, restructuring error types) required changes
in three modules: the discovery site of the change, the bridge layer, and the
config consumer.

## Decision

We will refactor `config/Builder` to a three-phase interface that isolates
discovery imports to a single adapter method:

**Phase 1 — `from_discovery()`**: The only place `config/` touches a
`discovery/` data type. Accepts `&DiscoveryResult`, validates into domain
types (`VaultRoot`, `VaultId`), stores winning markers internally.

**Phase 2 — `build_global()` / `build_vault()`**: Individual source processing.
Each reads its marker file through `ConfigFileProcessor`, returns
`Option<RawConfig>`. No awareness of the other source.

**Phase 3 — `build()`**: Orchestrator that calls phase 2 methods based on
which markers exist. Merges via `build_from_layers()`. Returns `Config`.

Additional changes:

- `config/root.rs` is deleted. `DiscoveredConfigFile` and
  `ConfigDiscoveryResult` are redundant with `DiscoveredMarker` from
  `discovery/`.
- `config/discovery.rs` (`ConfigDiscoveryPipeline`) keeps its name but
  receives winner marker paths from `from_discovery()` instead of the bridge
  type. Its purpose is unchanged: read file metadata and query database views.
- `Builder` no longer holds `start_dir`. That is now the Bootstrapper's
  concern.

## Alternatives Considered

### Alternative 1: Keep orchestration in config/Builder (status quo)

- **Pros**: No refactoring cost. Single `load()` method.
- **Cons**: `config/` imports discovery engine types, creating a cross-module
  coupling that violates the hexagonal architecture. Any change to discovery
  propagates to config. The bridge layer (`root.rs`) is pure overhead.

### Alternative 2: Single `load(&DiscoveryResult)` method

- **Pros**: Simpler than the three-phase design. One method, one call.
- **Cons**: `config/` still imports `DiscoveryResult` in its core orchestration
  method. The method must handle both extraction of vault_root/vault_id and the
  file processing pipeline in one body — mixing concerns.

### Alternative 3: Decompress at the Bootstrapper level

- **Pros**: `config/` imports nothing from `discovery/`.
- **Cons**: The Bootstrapper must extract individual paths from
  `DiscoveryResult` and pass 5+ parameters to the builder — a "Long Parameter
  List" that wants a parameter object of its own. This pushes the coupling
  problem up a layer instead of solving it.

## Technical Validation

### Research Findings

- The three-phase pattern mirrors the `from_`/`build` idiom used by `serde`'s
  deserializers and `nom`'s parsers: an initial extraction/adapter step,
  independent component processing, and final orchestration.
- ADR 016 established the segregated repository pattern. This ADR extends the
  same principle to the discovery-config boundary: each context imports only
  the data types it needs, never the orchestration logic.
- ADR 021 established `app/` as the composition root. This refactoring moves
  discovery orchestration from `config/` to `app/Bootstrapper`, implementing
  ADR 021's intent.

## Consequences

- **Positive**: `config/` imports exactly one data type from `discovery/`
  (`DiscoveryResult`) in exactly one place (`from_discovery()`).
- **Positive**: `build_global()` and `build_vault()` are independently
  testable — no need to mock discovery.
- **Positive**: The bridge layer (`root.rs`) disappears, removing the
  `ConfigDiscoveryResult::from_discovery()` conversion that existed only to
  bridge types that were structurally equivalent.
- **Positive**: Removing `start_dir` from `Builder` clarifies that the builder
  is a pure config-domain component — it needs only a repository and
  discovery results.
- **Negative**: Three methods instead of one for the basic case. The
  Bootstrapper must call `from_discovery()` then `build()` — two calls instead
  of one.
- **Risks**: If `from_discovery()` grows validation logic beyond what belongs
  in an adapter, it risks becoming the new bridge layer. Validation should
  remain thin: type conversion + ID creation only.

## References

- ADR 021 — `app` as composition root (this ADR implements the config side)
- ADR 016 — segregated repository traits (precedent for boundary discipline)
- `.scratch/root-config-discovery/11-discovery-redesign-decisions.md` — full session decisions
