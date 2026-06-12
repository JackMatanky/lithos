---
name: discovery-service-redesign
status: accepted
supersedes: [021]
date_proposed: 2026-06-11
date_decided: 2026-06-11
date_implemented:
stakeholders: [lithos-core maintainers]
---

# ADR 0001: Discovery Service Redesign

## Context

The `discovery/` module had accumulated structural debt that made it hard to
reason about and hard to test. `engine.rs` was a monolithic orchestrator with
dual vault/global discovery paths, `VaultRootProbe` and `GlobalRootProbe` were
structurally identical but separate, `diagnostics.rs` held non-fatal warnings
that mixed into discovery results, and `DiscoveryInput` bundled all parameters
into a single struct regardless of which phase of the 6-step process needed them.

The output types perpetuated an incorrect dual-discovery model:
`VaultDiscoveryResult` and `GlobalDiscoveryResult` implied two independent
discovery processes, when in reality there is one multi-phase process that
produces one result.

## Decision

We will redesign the `discovery/` module with the following architecture:

1. **Self-builder public API**: `DiscoveryService` IS its own builder
   (following `bincode::Config`). One-time config (filenames, boundary markers,
   global directory resolution, `suppress_global`) is set via chainable setters.
   `build()` validates invariants and returns `Result<Self>`.

2. **Per-invocation `InvocationInput`**: A small struct with only the
   per-call parameters (flag_path, env_path, cwd, ceiling_dirs_raw). Global
   directory resolution is owned by the service config, not the invocation
   input.

3. **Internal typestate pipeline (`DiscoveryMachine`)**: The 6-phase process
   (Initialized → Preempted → Anchored → Traversed → GlobalResolved →
   Finalized) is encoded as typed nodes. Each phase implements `From<Previous>`.
   Invalid transitions are compile errors. The caller sees only `discover()`.

4. **Unified output types**: `DiscoveryResult` replaces
   `VaultDiscoveryResult`/`GlobalDiscoveryResult`. `DiscoveryReport` carries
   non-fatal process metadata (skipped overrides, skipped ceilings, traversal
   stop reason). Downstream components receive only `DiscoveryResult`.

5. **`FolderProbe`**: Replaces `VaultRootProbe` and `GlobalRootProbe`. A
   single generic probe parameterised by `&[MarkerPattern]` at construction
   time.

6. **Layered error types**: `DiscoveryError` wraps phase-specific sub-errors
   (`OverrideError`, `TraversalError`, `GlobalError`) with transparent
   `#[from]` conversions.

7. **`policy.rs` as constants + types module**: Precedence enums, marker
   pattern lists (`ROOT_MARKER_FILES`, `GLOBAL_MARKER_FILES`), and boundary
   markers (`.git`, `.workspace`) live here. No separate `DiscoveryPolicy`
   struct — the builder absorbs policy fields directly.

8. **Deleted files**: `engine.rs` (responsibilities redistributed to
   `service.rs`, `override.rs`, `selector.rs`), `diagnostics.rs` (non-fatal
   conditions move to `DiscoveryReport` or `tracing::warn!`).

## Alternatives Considered

### Alternative 1: Keep monolithic `DiscoveryEngine` (status quo)

- **Pros**: No refactoring cost.
- **Cons**: Dual-discovery output types force consumers to handle two result
  types for what is conceptually one process. Single `DiscoveryInput` bundles
  all parameters regardless of phase. Adding probe types requires duplicating
  probe structs. Non-fatal diagnostics mixed into discovery results propagate
  to downstream components that don't need them.

### Alternative 2: Typestate at the public API level

- **Pros**: Compiler-enforced phase ordering visible to callers.
- **Cons**: The 6 internal phases are an implementation detail — callers want
  `discover()` not `initialize().preempt().normalize().traverse().globalize().finalize()`.
  Typestate at the API level forces every caller to navigate a type-level state
  machine for no ergonomic gain.

### Alternative 3: Flat builder with separate `DiscoveryPolicy` struct

- **Pros**: Clear separation between policy configuration and service config.
- **Cons**: An extra abstraction layer with no benefit — every field has a
  sensible default, policy never changes independently of the service, and
  the builder already enforces validation at `build()`.

## Technical Validation

### Research Findings

- The two-phase builder pattern matches `reqwest::ClientBuilder`/`reqwest::Client`,
  `bincode::Config`, and `std::process::Command` — all Rust libraries where
  one-time configuration is separated from per-invocation parameters.
- The Hoverbear typestate pattern (`cliffle.com/blog/rust-typestate/`) validates
  that internal typestate is a well-established zero-cost abstraction for phase
  ordering in Rust.
- The `DiscoveryResult`/`DiscoveryReport` split follows the CQRS principle:
  process metadata is not domain data.
- Boundary markers with probe-then-stop semantics align with the existing
  `allow_marker_at_ceiling` policy, avoiding a new mechanism.

## Consequences

- **Positive**: `config/` imports exactly one data type from `discovery/`
  (`DiscoveryResult`) in exactly one place (`Builder::from_discovery()`).
- **Positive**: The typestate pipeline makes invalid phase transitions (e.g.
  normalization before preemption) a compile error, not a runtime check.
- **Positive**: `FolderProbe` eliminates the structural duplication between
  vault and global probes. Adding a new marker pattern list requires adding
  a constant, not a new probe struct.
- **Positive**: CLI commands can destructure `DiscoveryReport` for verbose
  output without polluting the domain data path.
- **Negative**: Internal typestate adds ~100 lines of boilerplate (`From`
  impls per transition) that a simple procedural implementation wouldn't need.
- **Risks**: `DiscoveryMachine` could grow beyond its typestate boundaries if
  phase logic becomes too complex. Each phase should remain stateless beyond
  its typed input.

## References

- [Hoverbear — Rust State Machine Pattern](https://hoverbear.org/blog/rust-state-machine-pattern/#generically-sophistication) — internal typestate pattern
- [Cliffle — Typestate in Rust](https://cliffle.com/blog/rust-typestate/) — typestate design guidance
- [reqwest::ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) — two-phase builder precedent
- ADR 021 — `app` as composition root (this ADR implements the discovery side)
- `.scratch/root-config-discovery/11-discovery-redesign-decisions.md` — full session decisions
