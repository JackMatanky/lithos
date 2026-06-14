---
title: 11-discovery-service-config
category: enhancement
label: ready-for-agent
status: closed
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

- [x] `discovery/service.rs` defines `DiscoveryService` alongside the existing `CandidatePath` and `DiscoveryResult` contract types.
- [x] `discovery/service.rs` defines `DiscoveryServiceConfig` or an equivalent explicit stable configuration object.
- [x] `DiscoveryServiceConfig::default()` uses `VAULT_MARKER_PATTERNS`, `GLOBAL_MARKER_PATTERNS`, boundary marker defaults, empty global directories, and default traversal policy.
- [x] `DiscoveryService::new(config: DiscoveryServiceConfig) -> Result<Self, DiscoveryError>` constructs and validates the service.
- [x] The issue does not introduce or require `DiscoveryService::default().with_*().build()?`.
- [x] `ROOT_MARKER_PATTERNS` is renamed to `VAULT_MARKER_PATTERNS` everywhere in the active discovery code.
- [x] `DiscoveryServiceConfig` stores ordered vault marker patterns using `VAULT_MARKER_PATTERNS` as the default.
- [x] `DiscoveryServiceConfig` stores ordered global marker patterns using `GLOBAL_MARKER_PATTERNS` as the default.
- [x] `DiscoveryServiceConfig` stores project boundary marker patterns using `BOUNDARY_MARKER_PATTERNS`.
- [x] `DiscoveryServiceConfig` stores resolved global directories or equivalent global namespace directories supplied by the app/platform layer.
- [x] `DiscoveryServiceConfig` stores `allow_marker_at_ceiling` or equivalent traversal policy.
- [x] `DiscoveryServiceConfig` does not store `suppress_global`; that remains `DiscoveryFlags` per issue 10.
- [x] `DiscoveryService` does not acquire CWD, parse CLI flags, read environment variables, validate `DiscoveryContext`, run traversal, probe files, sort candidates, or call Config.
- [x] `DiscoveryService` does not implement `DiscoveryPort::discover()` in this issue — deferred to issue 12.
- [x] `DiscoveryService` does not expose a trusted-path second-pass discovery method.
- [x] Existing `DiscoveryPort` remains the app-facing inbound port for Bootstrapper.
- [x] Unit tests cover `DiscoveryServiceConfig::default()`.
- [x] Unit tests cover constructing `DiscoveryServiceConfig` with explicit global directories and traversal policy.
- [x] Unit tests cover successful `DiscoveryService::new(config)`.
- [x] Unit tests cover `DiscoveryService::new` validation failures for empty marker pattern lists.
- [x] Unit tests prove `suppress_global` is not part of `DiscoveryServiceConfig`.

## Implementation Notes

### Files Changed

- `lithos-core/src/discovery/policy.rs` — Renamed `ROOT_MARKER_PATTERNS` → `VAULT_MARKER_PATTERNS`, added `BOUNDARY_MARKER_PATTERNS` (`&[&str]`), derived `Clone`/`Debug`/`Eq`/`PartialEq` on `MarkerPattern`.
- `lithos-core/src/discovery/error.rs` — Added `ServiceConfigError` with variants `VaultMarkerPatterns`, `GlobalMarkerPatterns`, `BoundaryMarkerPatterns` and `DiscoveryError::Config(#[from] ServiceConfigError)`.
- `lithos-core/src/discovery/service.rs` — Added `DiscoveryServiceConfig` with `&'static` fields for marker data (zero alloc), `Vec<DirPath>` for dynamic global directories, `validate()` method, and `DiscoveryService::new(config)` delegating to validate.

### Design Decisions

- **`&'static [T]` over `Box<[T]>`**: The three marker pattern collections are fixed, immutable policy constants. `&'static` avoids allocation entirely and matches the constant types directly, eliminating all `.into_boxed_slice()` ceremony.
- **Separate `ServiceConfigError`**: Structured error variants (`VaultMarkerPatterns`, `GlobalMarkerPatterns`, `BoundaryMarkerPatterns`) rather than a generic `{ details: String }` — enables pattern matching on the exact failure.
- **`validate()` on config, not service**: `DiscoveryServiceConfig::validate()` is the validation authority; `DiscoveryService::new(config)` calls `config.validate()?`. This keeps the service constructor a one-liner and makes validation testable independently.
- **No builder pattern**: Per requirements — `DiscoveryServiceConfig` uses direct struct construction with `..Default::default()` spread.

### Verification

- 179 discovery tests pass.
- `cargo clippy` — zero warnings.
- `cargo fmt` — clean.

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
- [x] `DiscoveryServiceConfig` is implemented and tested.
- [x] `DiscoveryService::new(config)` is implemented and tested.
- [x] Stable service config is clearly separated from per-invocation `DiscoveryContext`.
- [x] `ROOT_MARKER_PATTERNS` is renamed to `VAULT_MARKER_PATTERNS`.
- [x] `suppress_global` remains on `DiscoveryFlags`.
- [x] No discovery execution, processor state machine, or Config interaction is implemented.

**Out of scope:**
- Typestate processor implementation.
- `DiscoveryPort::discover()` execution behavior.
- Preemption, probing, traversal, global search, finalization, or candidate ordering.
- Bootstrapper full orchestration.
- Config builder decoupling.
