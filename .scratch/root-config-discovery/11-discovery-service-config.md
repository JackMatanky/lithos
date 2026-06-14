---
title: 11-discovery-service-config
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
- `docs/adr/discovery/0001-discovery-service-redesign.md`

## What to build

Add the constructed `DiscoveryService` facade and explicit stable configuration object that will later back `DiscoveryPort`.

This issue is only the service configuration/construction slice. It should make stable Discovery policy explicit and validated, but it must not run discovery, instantiate the typestate processor, probe the filesystem, traverse directories, or call Config.

## Current state

Issue `10-bootstrap-context-discovery-contracts` already introduced:

- `discovery/context.rs`: `DiscoveryContext`, `DiscoveryFlags`, `DiscoveryEnv`
- `discovery/service.rs`: `CandidatePath`, `DiscoveryResult`
- `discovery/report.rs`: `DiscoveryReport` and report enums
- `discovery/port.rs`: `DiscoveryPort`, the inbound port used by `app/Bootstrapper`
- `app/bootstrap.rs`: `Bootstrapper<D: DiscoveryPort>` and context construction
- `discovery/policy.rs`: marker pattern constants currently named `ROOT_MARKER_PATTERNS` and `GLOBAL_MARKER_PATTERNS`

This issue should build on those contracts rather than reintroducing `InvocationInput`, `DiscoveryEngine`, `DiscoveredMarker`, `SkippedOverride`, or `DiscoveryPolicy` as the new design.

## Service Contract

`DiscoveryService` is the concrete Discovery-domain service that will eventually implement `DiscoveryPort`. In this issue it only becomes constructible and validated.

Use an explicit config object, not a fluent self-builder:

```rust
let config = DiscoveryServiceConfig {
    global_directories,
    ..DiscoveryServiceConfig::default()
};

let service = DiscoveryService::new(config)?;
```

This is intentionally not `DiscoveryService::default().with_*().build()?` because the stable knobs are preset domain policy plus app-supplied global directories. A builder adds ceremony without clarifying the boundary.

## Stable Configuration Owned By `DiscoveryServiceConfig`

- Vault marker patterns: ordered path patterns for local/vault config candidates.
- Global marker patterns: ordered path patterns for global config candidates.
- Boundary marker patterns: project boundary path patterns such as `.git` and `.workspace`.
- Global directories: already-resolved global namespace directories supplied by the app layer or platform adapter.
- Traversal policy: `allow_marker_at_ceiling` or equivalent.

## Required Rename

- Rename `ROOT_MARKER_PATTERNS` to `VAULT_MARKER_PATTERNS`.
- Keep or introduce `GLOBAL_MARKER_PATTERNS`.
- Use a boundary-specific name such as `BOUNDARY_MARKER_PATTERNS` or `DEFAULT_BOUNDARY_MARKERS` for project boundary markers.

`ROOT_MARKER_PATTERNS` is too ambiguous because this domain distinguishes vault root, filesystem root, and project boundary. The local config discovery patterns establish a Vault Root, so the name must say `VAULT`.

## Per-Invocation Configuration Not Owned By `DiscoveryServiceConfig`

- `DiscoveryFlags::suppress_global`
- Explicit config file path from flags/env
- Explicit vault directory path from flags/env
- Anchor/current working directory
- Raw ceiling directory data

Those values remain in `DiscoveryContext` and are supplied per invocation.

## Acceptance criteria

- [ ] `discovery/service.rs` defines `DiscoveryService` alongside the existing `CandidatePath` and `DiscoveryResult` contract types.
- [ ] `discovery/service.rs` defines `DiscoveryServiceConfig` or an equivalent explicit stable configuration object.
- [ ] `DiscoveryServiceConfig::default()` uses `VAULT_MARKER_PATTERNS`, `GLOBAL_MARKER_PATTERNS`, boundary marker defaults, empty global directories, and default traversal policy.
- [ ] `DiscoveryService::new(config: DiscoveryServiceConfig) -> Result<Self, DiscoveryError>` constructs and validates the service.
- [ ] The issue does not introduce or require `DiscoveryService::default().with_*().build()?`.
- [ ] `ROOT_MARKER_PATTERNS` is renamed to `VAULT_MARKER_PATTERNS` everywhere in the active discovery code.
- [ ] `DiscoveryServiceConfig` stores ordered vault marker patterns using `VAULT_MARKER_PATTERNS` as the default.
- [ ] `DiscoveryServiceConfig` stores ordered global marker patterns using `GLOBAL_MARKER_PATTERNS` as the default.
- [ ] `DiscoveryServiceConfig` stores project boundary marker patterns using a named default constant such as `BOUNDARY_MARKER_PATTERNS` or `DEFAULT_BOUNDARY_MARKERS`.
- [ ] `DiscoveryServiceConfig` stores resolved global directories or equivalent global namespace directories supplied by the app/platform layer.
- [ ] `DiscoveryServiceConfig` stores `allow_marker_at_ceiling` or equivalent traversal policy.
- [ ] `DiscoveryServiceConfig` does not store `suppress_global`; that remains `DiscoveryFlags` per issue 10.
- [ ] `DiscoveryService` does not acquire CWD, parse CLI flags, read environment variables, validate `DiscoveryContext`, run traversal, probe files, sort candidates, or call Config.
- [ ] `DiscoveryService` does not implement `DiscoveryPort::discover()` in this issue unless the implementation can do so without executing discovery. If a trait impl would require a fake or placeholder run method, defer the impl to issue `12-discovery-typestate-run`.
- [ ] `DiscoveryService` does not expose a trusted-path second-pass discovery method.
- [ ] Existing `DiscoveryPort` remains the app-facing inbound port for Bootstrapper; this issue prepares `DiscoveryService` to be the concrete implementation in the next issue.
- [ ] Unit tests cover `DiscoveryServiceConfig::default()`.
- [ ] Unit tests cover constructing `DiscoveryServiceConfig` with explicit global directories and traversal policy.
- [ ] Unit tests cover successful `DiscoveryService::new(config)`.
- [ ] Unit tests cover any `DiscoveryService::new` validation failures, such as empty marker pattern lists or invalid global directory configuration if those states are representable.
- [ ] Unit tests prove `suppress_global` is not part of `DiscoveryServiceConfig`.

## Blocked by

- `.scratch/root-config-discovery/10-bootstrap-context-discovery-contracts.md`

## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Add explicit `DiscoveryServiceConfig` and `DiscoveryService::new(config)` without running discovery.

**Current behavior:**
The app layer now depends on `DiscoveryPort`, and discovery has context/result/report contracts. There is no concrete service object with validated stable Discovery configuration.

**Desired behavior:**
`DiscoveryServiceConfig` makes stable Discovery policy explicit: vault/global marker patterns, boundary markers, global directories, and traversal policy. `DiscoveryService::new(config)` validates and stores that configuration. The service is ready for issue `12` to add the typestate processor and `DiscoveryPort` execution behavior.

**Key interfaces:**
- `DiscoveryServiceConfig` — explicit stable configuration object.
- `DiscoveryService::new(config)` — validates and returns the configured service.
- `DiscoveryPort` — existing inbound port that `DiscoveryService` will implement when execution exists.
- `VAULT_MARKER_PATTERNS`, `GLOBAL_MARKER_PATTERNS`, boundary marker constants — stable target definitions.

**Acceptance criteria:**
- [ ] `DiscoveryServiceConfig` is implemented and tested.
- [ ] `DiscoveryService::new(config)` is implemented and tested.
- [ ] Stable service config is clearly separated from per-invocation `DiscoveryContext`.
- [ ] `ROOT_MARKER_PATTERNS` is renamed to `VAULT_MARKER_PATTERNS`.
- [ ] `suppress_global` remains on `DiscoveryFlags`.
- [ ] No discovery execution, processor state machine, or Config interaction is implemented.

**Out of scope:**
- Typestate processor implementation.
- `DiscoveryPort::discover()` execution behavior.
- Preemption, probing, traversal, global search, finalization, or candidate ordering.
- Bootstrapper full orchestration.
- Config builder decoupling.
