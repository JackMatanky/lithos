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

This slice should align code around each phase of the process directly. It should not wire together legacy `engine.rs` behavior as a black box. Legacy code can be mined for tests or algorithms, but the new processor must be organized around the agreed phases.

Issue `10` introduced `DiscoveryPort` returning only `DiscoveryResult`. That is now incomplete because execution also produces `DiscoveryReport`. This issue owns updating the port and Bootstrapper pass-through seam so the app layer can receive both discovery data and report metadata.

## Process to implement

The processor must run the same phases in the same order for every invocation. A phase may produce no candidates or a skip report, but phases do not disappear.

1. Initialization / `Init` state: processor receives already-acquired `DiscoveryContext` from Bootstrapper plus stable `DiscoveryServiceConfig` from `DiscoveryService`.
2. Explicit preemption: apply already-validated explicit config file and vault directory inputs from flags/env, preserving precedence.
3. Anchor normalization: confirm the active anchor from `DiscoveryContext` is the traversal anchor for non-explicit-vault discovery.
4. Local traversal: search ordered vault marker patterns from the anchor or explicit vault directory, respecting ceilings and project boundaries.
5. Global resolution: always evaluate global resolution; if `DiscoveryFlags::suppress_global` is set, report explicit global suppression and produce no global candidates.
6. Finalization: deduplicate and order candidates, returning `DiscoveryResult` and `DiscoveryReport`.

## Acceptance criteria

- [ ] `discovery/processor.rs` defines a private typestate processor with states aligned to the process phases.
- [ ] The initial processor state can be named `Init` and represents already-acquired context plus service config; it does not acquire context itself.
- [ ] The processor uses typed transitions following the fixed process order, following the Hoverbear-style typestate pattern.
- [ ] `DiscoveryPort::discover` is updated to return both `DiscoveryResult` and `DiscoveryReport`, either as `Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>` or a clearly named equivalent output struct.
- [ ] `app::Bootstrapper::discover` is updated to preserve the same DiscoveryPort output shape without discarding `DiscoveryReport`.
- [ ] `DiscoveryService` implements `DiscoveryPort`.
- [ ] `DiscoveryService::discover(context: &DiscoveryContext<'_>)` or the `DiscoveryPort` implementation delegates to the processor.
- [ ] Explicit config file paths preempt local traversal but do not suppress global resolution.
- [ ] Explicit vault directory paths preempt anchor traversal but still search vault marker patterns from that vault directory.
- [ ] Invalid explicit config file or vault directory inputs remain fatal `DiscoveryError`s from context construction; the processor must not reintroduce skipped override report entries.
- [ ] Local traversal records `LocalTraversalStopReason::ExplicitConfigFile` when skipped by explicit config file.
- [ ] Local traversal respects raw ceiling data from `DiscoveryEnv` and reports skipped ceiling segments.
- [ ] Local traversal respects project boundary markers with probe-then-stop semantics.
- [ ] Global resolution always runs as a phase; `suppress_global` produces `GlobalResolutionSkipReason::SuppressedByFlag`.
- [ ] Inaccessible global directories are skipped with `tracing::warn!` and do not add report fields.
- [ ] Candidate paths use `CandidatePath { base: DirPath, path: FilePath }` and do not store format.
- [ ] Candidate ordering derives structured format precedence from the candidate path extension when needed, using `StructuredFileFormat::from_path` / `StructuredFileFormat::PRECEDENCE` or equivalent.
- [ ] `DiscoveryResult` returns separate ordered `vault` and `global` candidate vectors.
- [ ] `vault_root` is not reintroduced; it remains derivable from the winning vault candidate's `base`.
- [ ] The implementation removes or retires legacy `engine.rs` and `diagnostics.rs` responsibilities rather than routing through them.
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
`DiscoveryService` implements `DiscoveryPort`. Calling the port constructs a private typestate processor and returns `DiscoveryResult` plus `DiscoveryReport`. The processor code is organized by the agreed process phases, not by legacy engine structure.

**Key interfaces:**
- `DiscoveryProcessor` — private typestate processor in `processor.rs`.
- `DiscoveryPort` — inbound port used by Bootstrapper; updated to carry report output.
- `DiscoveryService` — concrete implementation of `DiscoveryPort`.
- `CandidatePath` — validated candidate output without stored format.
- `DiscoveryReport` — skipped ceilings, local traversal stop reason, and global suppression metadata.

**Acceptance criteria:**
- [ ] Typestate phases are explicit and phase-aligned.
- [ ] `DiscoveryService` implements `DiscoveryPort` and the port returns report metadata as well as discovery data.
- [ ] Explicit config preempts local traversal only; global phase still runs.
- [ ] Explicit vault preempts anchor traversal only; local target search still runs.
- [ ] Global resolution is a fixed phase and can be suppressed by per-invocation flags.
- [ ] Legacy engine/diagnostics responsibilities are replaced by phase-aligned code.
- [ ] Tests cover the process branches and finalization behavior.

**Out of scope:**
- Bootstrapper full orchestration beyond preserving the updated `DiscoveryPort` output shape.
- Config builder decoupling.
- CLI commands.
