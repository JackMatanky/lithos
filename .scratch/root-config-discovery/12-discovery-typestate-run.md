---
title: 12-discovery-typestate-run
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-13
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
- `docs/adr/discovery/0001-discovery-service-redesign.md`

## What to build

Implement the execution side of redesigned Discovery: a private typestate `DiscoveryProcessor` that runs the fixed discovery process and powers `DiscoveryService` as the concrete implementation of `DiscoveryPort`.

This slice is **additive** — legacy code (`engine.rs`, `diagnostics.rs`, `DiscoveryPolicy`, `VaultRootProbe`/`GlobalRootProbe`, flat error enum) stays untouched. No deletions. No breaking changes to existing call sites. Legacy retirement is a separate future issue.

Issue `10` introduced `DiscoveryPort` returning only `DiscoveryResult`. That is now incomplete because execution also produces `DiscoveryReport`. This issue owns updating the port and Bootstrapper pass-through seam so the app layer can receive both discovery data and report metadata.

## Process to implement

The processor must run the same phases in the same order for every invocation. A phase may produce no candidates or a skip report, but phases do not disappear.

1. **Init**: processor receives already-acquired `DiscoveryContext` from Bootstrapper plus stable `DiscoveryServiceConfig` from `DiscoveryService`.
2. **FlagOverride**: reads `ctx.flags()`. Probes flag vault dir with `FolderProbe{VAULT_MARKER_PATTERNS}` → `vault: Vec<CandidatePath>`. Parses ceilings from `ctx.env().ceiling_dirs_raw()` → `report.skipped_ceilings`.
3. **EnvOverride**: resolves flag>env>anchor for traversal anchor. Resolves config precedence (flag>env). Sets `report.local_traversal_stop_reason = ExplicitConfigFile` when config is present.
4. **AscendingTraversal**: walks from traversal anchor with `FolderProbe{VAULT_MARKER_PATTERNS}`, respects ceilings and project boundaries. Fills `vault` if not already populated by FlagOverride.
5. **GlobalResolution**: probes `config.global_directories` with `FolderProbe{GLOBAL_MARKER_PATTERNS}`. Sets `report.global_resolution_skip_reason = SuppressedByFlag` if suppressed.
6. **Finalized**: `finalize() -> (DiscoveryResult, DiscoveryReport)`.

### Branching

After EnvOverride, the branching matrix is:

| vault override | config override | path |
|---|---|---|
| yes | yes | FlagOverride probes vault → Finalized |
| yes | no | FlagOverride probes vault → GlobalResolution → Finalized |
| no | yes | AscendingTraversal → Finalized |
| no | no | AscendingTraversal → GlobalResolution → Finalized |

This is driven by `branch_strategy()` on `EnvOverride` phase data; all paths are explicit `From` transitions selected at runtime.

## Acceptance criteria

- [ ] `discovery/processor.rs` defines a private typestate processor with states aligned to the process phases.
- [ ] The initial processor state can be named `Init` and represents already-acquired context plus service config; it does not acquire context itself.
- [ ] `DiscoveryProcessor` stores `config: &DiscoveryServiceConfig`, `ctx: &DiscoveryContext`, and phase-specific data — `vault: Vec<CandidatePath>` and `global: Vec<CandidatePath>` as accumulators, plus `report: DiscoveryReport`.
- [ ] The processor uses typed transitions following the fixed process order, following the Hoverbear-style typestate pattern.
- [ ] `DiscoveryPort::discover` is updated to return `Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>`.
- [ ] `app::Bootstrapper::discover` is updated to preserve the same tuple shape without discarding `DiscoveryReport`.
- [ ] `DiscoveryService` implements `DiscoveryPort`.
- [ ] `DiscoveryService::discover` delegates directly to the processor chain.
- [ ] Explicit config file paths preempt local traversal but do not suppress global resolution.
- [ ] Explicit vault directory paths preempt anchor traversal but still search vault marker patterns from that vault directory.
- [ ] Invalid explicit config file or vault directory inputs remain fatal `DiscoveryError`s from context construction; the processor does not reintroduce skipped override report entries.
- [ ] `LocalTraversalStopReason::ExplicitConfigFile` is set when local traversal is skipped by explicit config file.
- [ ] Ceiling parsing happens during the FlagOverride phase using `ctx.env().ceiling_dirs_raw()`, producing `report.skipped_ceilings`.
- [ ] Local traversal respects project boundary markers with probe-then-stop semantics.
- [ ] Global resolution always runs as a phase; `suppress_global` produces `GlobalResolutionSkipReason::SuppressedByFlag`.
- [ ] Inaccessible global directories are skipped with `tracing::warn!` and do not add report fields.
- [ ] Candidate paths use `CandidatePath { base: DirPath, path: FilePath }` and do not store format.
- [ ] Candidate ordering derives structured format precedence from the candidate path extension when needed, using `StructuredFileFormat::from_path` / `StructuredFileFormat::PRECEDENCE` or equivalent.
- [ ] `DiscoveryResult` returns separate ordered `vault` and `global` candidate vectors.
- [ ] `vault_root` is not reintroduced; it remains derivable from the winning vault candidate's `base`.
- [ ] `FolderProbe { patterns: &'static [MarkerPattern] }` is added in `probe.rs` alongside existing `VaultRootProbe`/`GlobalRootProbe` without modifying them.
- [ ] FolderProbe is infallible (paths pre-validated before reaching it) and iterates patterns × format precedence.
- [ ] Active policy names use `VAULT_MARKER_PATTERNS`, not `ROOT_MARKER_PATTERNS`.
- [ ] Unit tests cover explicit config file, explicit vault directory, env config file, env vault directory, anchor traversal, ceiling diagnostics, boundary marker stopping, global suppression, global discovery, deduplication, and precedence ordering.
- [ ] Unit tests cover `DiscoveryService` through the `DiscoveryPort` trait so Bootstrapper can be tested against the port seam.

## Blocked by

- `.scratch/root-config-discovery/11-discovery-service-config.md`

## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Implement `DiscoveryPort` for `DiscoveryService` using a phase-aligned typestate processor.

**Current behavior:**
Discovery contracts, `DiscoveryPort`, and `DiscoveryService::new(config)` exist, but no redesigned execution pipeline runs the discovery process. `DiscoveryPort` currently lacks report output and must be completed here.

**Desired behavior:**
`DiscoveryService` implements `DiscoveryPort`. Calling the port constructs a private typestate processor and returns `(DiscoveryResult, DiscoveryReport)`. The processor code is organized by the agreed process phases, not by legacy engine structure.

**Key interfaces:**
- `DiscoveryProcessor` — private typestate processor in `processor.rs`; phases: `Init`, `FlagOverride`, `EnvOverride`, `AscendingTraversal`, `GlobalResolution`, `Finalized`.
- `DiscoveryPort` — inbound port used by Bootstrapper; updated to return `Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>`.
- `DiscoveryService` — concrete implementation of `DiscoveryPort`.
- `FolderProbe { patterns }` — infallible marker probe, added alongside existing probes.
- `CandidatePath` — validated candidate output without stored format.
- `DiscoveryReport` — skipped ceilings, local traversal stop reason, and global suppression metadata.

**Acceptance criteria:**
- [ ] Typestate phases are explicit and phase-aligned.
- [ ] `DiscoveryService` implements `DiscoveryPort` and the port returns report metadata as well as discovery data.
- [ ] Explicit config preempts local traversal only; global phase still runs.
- [ ] Explicit vault preempts anchor traversal only; vault marker search from that directory still runs.
- [ ] Global resolution is a fixed phase and can be suppressed by per-invocation flags.
- [ ] Branching matrix covers all four combinations of (vault override, config override).
- [ ] Legacy code (`engine.rs`, `diagnostics.rs`, `DiscoveryPolicy`, `VaultRootProbe`/`GlobalRootProbe`, flat error enum) stays untouched — additive changes only.
- [ ] Tests cover the process branches and finalization behavior.

**Out of scope:**
- Bootstrapper full orchestration beyond preserving the updated `DiscoveryPort` output shape.
- Config builder decoupling.
- CLI commands.
- Deleting or modifying legacy discovery code.
