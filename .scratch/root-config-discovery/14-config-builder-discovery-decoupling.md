---
title: 14-config-builder-discovery-decoupling
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-12
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`
- `.scratch/root-config-discovery/discovery-redesign-decisions.md`
- `.scratch/root-config-discovery/10-bootstrap-context-discovery-contracts.md`
- `.scratch/root-config-discovery/11-discovery-service-config.md`
- `.scratch/root-config-discovery/12-discovery-typestate-run.md`
- `.scratch/root-config-discovery/13-bootstrapper-orchestration-flow.md`
- `docs/adr/config/0001-config-builder-decoupling.md`

## What to build

Refactor `config/Builder` so Config no longer orchestrates Discovery.

`Builder` receives the Bootstrapper-produced `DiscoveryResult` via a thin
`from_discovery()` adapter, then builds config through `build_vault()` and
`build_global()` — each fully owning its file read, staleness check, and
processor pipeline from the `CandidatePath` data. The `ConfigDiscoveryPipeline`
intermediate layer is removed entirely.

## Current state

Issues `10`-`13` are the implementation source of truth for this slice, even
where older ADR text differs.

The active Discovery handoff is:

- `DiscoveryResult` with separate ordered `vault` and `global` candidate vectors.
- `CandidatePath { base: DirPath, path: FilePath }`.
- No separate `vault_root`; the vault root is derived from the selected vault
  candidate's `base`.
- No stored format field; format-sensitive behavior must derive from the
  candidate path when Config needs it.
- No `DiscoveredMarker`, `InvocationInput`, `DiscoveryEngine`, `DiscoveryInput`,
  `GlobalDiscoveryInput`, or `DiscoveryPolicy` in the new integration path.

The DiscoveryService enforces a non-empty vault candidate list before producing
`DiscoveryResult`, so Config never receives a vaultless result.

## Design

```
DiscoveryService
  └─ guarantees at least one vault candidate, none empty (structural invariant)
       │
       ▼
DiscoveryResult { vault: Vec<CandidatePath>, global: Vec<CandidatePath> }
       │
       ▼
Builder::from_discovery(result, repo)
  └─ stores candidate boxes, returns Builder
  └─ infallible (structural invariants enforced upstream by DiscoveryService)
       │
       ▼
Builder::build()
  ├─ build_vault(&R) → RawVaultConfig                                [always]
  │   └─ derives VaultRoot from vault[0].base(), resolves VaultId via DB
  ├─ build_global(&R) → Option<RawGlobalConfig>                      [if global present]
  └─ build_from_layers(global, vault) → Config                       [unchanged merge]
```

`build_vault()` and `build_global()` each:

1. Read `FileMetadata` from the `CandidatePath`
2. Fetch the cached DB view for staleness comparison
3. Call `ConfigFileProcessor::compare()` to determine staleness
4. Produce their respective raw config type

No `ConfigDiscoveryPipeline` type exists — the work is split between the two
build methods, which are independently testable.

### Builder representation

`Builder<R>` stores the candidate vectors directly (owned) to avoid a lifetime
parameter:

```rust
pub(crate) struct Builder<R> {
    vault: Box<[CandidatePath]>,
    global: Box<[CandidatePath]>,
    repository: R,
}
```

`from_discovery()` moves the `Box<[CandidatePath]>` slices out of the incoming
`DiscoveryResult`. `VaultRoot` and `VaultId` are derived inside
`build_vault()` from `self.vault[0].base()` and checked against stored vault
identity in the DB — builder state stays minimal.

## Acceptance criteria

### Builder interface

- [ ] `Builder::from_discovery()` is the only Config entry point that accepts a
      `discovery::service::DiscoveryResult`.
- [ ] `Builder::from_discovery()` stores the candidate boxes and repository.
      Winner extraction is deferred — `build_vault()` and `build_global()`
      index `self.vault[0]` and `self.global.first()` respectively.
- [ ] `Builder::from_discovery()` stays thin: moves candidate boxes, stores
      repository. No file reading, no staleness checking, no structural
      validation, no VaultId resolution.
- [ ] `Builder::from_discovery()` consumes Discovery's validated `CandidatePath
      { base: DirPath, path: FilePath }` handoff instead of re-validating plain
      `PathBuf` marker paths.
- [ ] `Builder::from_discovery()` is infallible. It moves candidate boxes and
      stores the repository. All discovery-side invariants (non-empty vault,
      valid paths) are enforced by `DiscoveryService` upstream. Error sources
      (VaultId resolution, DirPath→VaultRoot conversion, file I/O, staleness)
      live in `build_vault()` and `build_global()`.
- [ ] `Builder::build()` orchestrates `build_global()` and `build_vault()`
      based on discovered marker presence.

### Build methods

- [ ] `Builder::build_vault()` reads `self.vault[0]`, derives `VaultRoot` from
      its `base()`, resolves `VaultId` via DB (create if new), reads file
      metadata, fetches the cached vault view, runs
      `ConfigFileProcessor::compare()` for staleness, and produces
      `RawVaultConfig`. Always called (vault candidate guaranteed upstream).
- [ ] `Builder::build_global()` reads `self.global.first()`, reads file
      metadata, fetches the cached global view, runs
      `ConfigFileProcessor::compare()` for staleness, and produces
      `Option<RawGlobalConfig>`. Called only when a global candidate exists.
- [ ] `Builder::build_vault()` and `Builder::build_global()` are independently
      testable and contain no discovery orchestration.
- [ ] `build_from_layers()` remains the unchanged pure config-domain merge seam.

### Removals

- [ ] `config/root.rs` is deleted; `ConfigDiscoveryResult` and
      `DiscoveredConfigFile` are removed.
- [ ] `config/discovery.rs` is deleted; `ConfigDiscoveryPipeline` and its
      config-owned `DiscoveryResult` type are removed. The per-candidate
      file-read + staleness + processor work is absorbed into `build_global()`
      and `build_vault()`.
- [ ] `Builder` no longer stores `start_dir`. All callers of
      `Builder::new(start_dir, ...)` are updated or removed.
- [ ] `config/builder.rs` no longer imports `DiscoveryEngine`, `DiscoveryInput`,
      `GlobalDiscoveryInput`, or discovery policy types.
- [ ] `config/builder.rs` imports `discovery::service::DiscoveryResult` only
      for `Builder::from_discovery()`.

### Invariants preserved

- [ ] Existing staleness behavior remains owned by
      `ConfigFileProcessor::compare()`; no `BuildMode` is introduced.
- [ ] File-vs-directory validation remains owned by Discovery/FS path types
      (`DirPath`, `FilePath`).
- [ ] Config only reads file contents and queries cached DB views — no path
      re-validation.

### Tests

- [ ] Tests prove `Builder` builds correctly from vault-only and combined
      (global+vault) discovery outputs.
- [ ] A regression test verifies `build_from_layers()` contract is preserved
      during refactoring.
- [ ] Test naming follows descriptive convention:
      `from_discovery_stores_vault_root_from_candidate_base`,
      `build_vault_produces_config_from_vault_only`, etc.

## Blocked by

- `.scratch/root-config-discovery/13-bootstrapper-orchestration-flow.md` —
  **RESOLVED** (completed 2026-06-14)


## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Refactor Config builder so Config consumes discovery results
without orchestrating Discovery. Remove the `ConfigDiscoveryPipeline`
intermediate layer entirely — each build method owns its full pipeline from
`CandidatePath` to raw config.

**Current behavior:**
Config builder orchestrates discovery internally (`DiscoveryEngine`,
`DiscoveryInput`, `GlobalDiscoveryInput`, `DiscoveryPolicy`), then feeds
results through `config/root.rs` bridge types and `ConfigDiscoveryPipeline` —
creating unnecessary coupling and indirection.

**Desired behavior:**
Config receives the Bootstrapper-produced, FS-validated `DiscoveryResult`
through a narrow `from_discovery()` adapter that extracts winning
`CandidatePath` values, derives `VaultRoot`, and resolves `VaultId`.
`Builder::build()` then delegates to `build_vault()` (always) and
`build_global()` (conditionally), each of which owns its file read, staleness
check, and processor pipeline. The `ConfigDiscoveryPipeline` layer,
`config/discovery.rs`, and `config/root.rs` bridge types are deleted.

Discovery-side structural invariants (non-empty vault, valid paths) are
enforced by `DiscoveryService` before `DiscoveryResult` reaches Config.
`from_discovery()` never validates discovery assumptions — it only converts and
stores.

**Key interfaces:**
- `Builder::from_discovery(DiscoveryResult, R)` — moves candidate boxes,
  stores repository. Infallible.
- `Builder::build_vault(&R)` → `RawVaultConfig` — derives VaultRoot, resolves
  VaultId, reads file, checks staleness, processes. Always called.
- `Builder::build_global(&R)` → `Option<RawGlobalConfig>` — same, called
  conditionally from `build()`.
- `Builder::build()` — orchestrates build methods and merge seam.
- `build_from_layers()` — unchanged merge seam.
- `ConfigFileProcessor::compare()` — unchanged staleness owner.

**Deleted files:**
- `config/root.rs` — bridge types removed.
- `config/discovery.rs` — pipeline removed.

**Acceptance criteria:**
- [ ] `from_discovery()` moves candidate boxes and stores repository — no file
      I/O, no staleness, no VaultRoot/VaultId derivation, no structural
      validation. Infallible.
- [ ] `build_vault()` indexes `self.vault[0]`, derives `VaultRoot` and resolves
      `VaultId`, reads file, fetches view, checks staleness, processes.
      `build_global()` indexes `self.global.first()` for the same pipeline.
- [ ] No `ConfigDiscoveryPipeline` — `config/discovery.rs` deleted.
- [ ] No `config/root.rs` — bridge types deleted.
- [ ] Builder imports no discovery engine/input/policy types.
- [ ] `from_discovery()` does not validate structural invariants (non-empty
      vault) — DiscoveryService guarantees those upstream.
- [ ] Tests prove vault-only and combined flows, regression-test
      `build_from_layers()` contract.

**Out of scope:**
- Changing the DiscoveryService public API.
- Bootstrapper implementation beyond consuming the result shape from issue `13`.
- CLI discovery subcommands.
- Replacing existing staleness comparison with a new build mode.
