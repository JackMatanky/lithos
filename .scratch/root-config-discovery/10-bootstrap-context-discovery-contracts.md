---
title: 10-bootstrap-context-discovery-contracts
category: enhancement
label: closed
status: done
date_created: 2026-06-13
date_closed: 2026-06-13
---

## Type

AFK

## Labels

- root-config-discovery
- closed

## Parent

- `.scratch/root-config-discovery/PRD.md`
- `.scratch/root-config-discovery/discovery-redesign-decisions.md`
- `docs/adr/024-bootstrapper-orchestration.md`
- `docs/adr/discovery/0001-discovery-service-redesign.md`

## What to build

Create the first vertical slice for the redesigned discovery flow: discovery input/output contracts plus minimal Bootstrapper context acquisition.

This slice should not implement the discovery processor, traversal, global resolution, Config integration, or CLI commands. Its purpose is to establish the hexagonal boundary: `app/Bootstrapper` acquires runtime context, and `discovery/` declares the data contract it needs.

## Implementation status

**Implemented and verified.** All acceptance criteria met with the deviations and additions documented below.

Files added or modified:
- `lithos-core/src/discovery/context.rs` — new
- `lithos-core/src/discovery/service.rs` — new
- `lithos-core/src/discovery/report.rs` — new
- `lithos-core/src/discovery/error.rs` — extended (new variants and sub-error types)
- `lithos-core/src/discovery/policy.rs` — extended (`ROOT_MARKER_PATTERNS`, `GLOBAL_MARKER_PATTERNS`)
- `lithos-core/src/discovery/mod.rs` — module declarations added
- `lithos-core/src/app/bootstrap.rs` — new
- `lithos-core/src/app/mod.rs` — module declaration added
- `lithos-core/src/discovery/engine.rs` — updated to use new `FlagOverrideError` / `EnvironmentOverrideError`
- `lithos-core/src/discovery/port.rs` — new (post-initial; see §DiscoveryPort below)

Post-initial passes (committed separately):
- **rust-best-practices pass** (commit `779b5c2a`): eliminated double `to_path_buf()` allocations at all 5 path-validation sites in `context.rs`; fixed `anchor` variable shadow (`anchor_dir` rename); split all multi-assertion tests into single-assertion tests; added `# Errors` doc sections to the three fallible constructors; improved struct-level doc comments. 1832 tests after this pass.
- **Builder + port refactor** (commit `1c06560a`): `DiscoveryContext::new` signature changed (see §Builder below); `DiscoveryFlags` and `DiscoveryEnv` gained `Default`; `Bootstrapper` redesigned (see §Bootstrapper below); `discovery/port.rs` added (see §DiscoveryPort below). 1847 tests after this pass.

All 1847 unit tests pass (baseline 1813 at initial implementation; grew to 1832 after rust-best-practices pass; grew to 1847 after builder + port refactors). Lint, format, deny, and ADR validation clean.

## Acceptance criteria

- [x] `discovery/context.rs` defines `DiscoveryContext<'a>` as the Discovery-owned input contract that the Bootstrapper fills.
- [x] `DiscoveryContext` groups invocation data into `DiscoveryFlags` and `DiscoveryEnv<'a>` rather than a flat bag of options.
- [x] `DiscoveryFlags` contains CLI-derived invocation fields: explicit config file path, explicit vault directory path, and `suppress_global`.
- [x] `DiscoveryEnv` contains environment-derived invocation fields: explicit config file path, explicit vault directory path, and raw ceiling directory data.
- [x] `DiscoveryContext` contains the active context anchor path supplied by the Bootstrapper.
- [x] The contract names use `DiscoveryContext`, not `InvocationInput`.
- [x] The Bootstrapper, not Discovery, owns context acquisition: current working directory, CLI flags, environment variables, and platform global-directory discovery.
- [x] `app/Bootstrapper` can build a `DiscoveryContext` from injected or testable context sources without running discovery.
- [x] `suppress_global` is per-invocation state on `DiscoveryFlags`, not stable `DiscoveryService` builder config.
- [x] `discovery/service.rs` defines the boundary output types without executing discovery: `CandidatePath` and `DiscoveryResult`.
- [x] `CandidatePath` contains `base: DirPath` and `path: FilePath`; it does not store a file format.
- [x] `DiscoveryResult` keeps separate `vault` and `global` candidate collections. (**Deviation**: uses `Vec<CandidatePath>`, not `Box<[CandidatePath]>` — kept mutable during discovery phase; Bootstrapper can freeze to `Box<[T]>` later if needed. Adds `into_parts()` for ownership transfer.)
- [x] `DiscoveryResult` does not store a separate `vault_root`; vault root is derivable from the selected vault candidate's `base`.
- [x] `discovery/report.rs` defines report-only process metadata: skipped ceilings, local traversal stop reason, and global resolution skip reason.
- [x] Invalid explicit config/vault overrides are modeled as fatal errors, not as skipped overrides on `DiscoveryReport`.
- [x] `DiscoveryReport` includes explicit global suppression (`--no-global-config`) while inaccessible global directories remain `tracing::warn!` only.
- [x] `LocalTraversalStopReason` can represent local traversal skipped because an explicit config file was supplied.
- [x] `discovery/error.rs` defines the fatal error taxonomy needed by the contracts, including invalid explicit config file and invalid explicit vault directory cases.
- [x] `discovery/policy.rs` uses names that describe path patterns, not files: `ROOT_MARKER_PATTERNS` and `GLOBAL_MARKER_PATTERNS` or equivalent.
- [x] Target patterns and boundary marker definitions are declared as Discovery policy contracts, but traversal/probing logic is out of scope.
- [x] Unit tests cover construction of `DiscoveryContext`, `DiscoveryFlags`, `DiscoveryEnv`, `CandidatePath`, `DiscoveryResult`, and `DiscoveryReport`.
- [x] Unit tests prove the Bootstrapper context-acquisition seam can build a `DiscoveryContext` without invoking DiscoveryService or Config.

## Implementation details

### Lifetimes

The original spec used `DiscoveryFlags<'a>` and `DiscoveryEnv<'a>` with borrowed `&'a Path` fields. In the implementation, `DiscoveryFlags` drops its lifetime parameter entirely because its path fields are now owned `FilePath` and `DirPath` values, validated at construction. `DiscoveryEnv<'a>` retains `'a` only for `ceiling_dirs_raw: Option<&'a OsStr>`, which remains borrowed. `DiscoveryContext<'a>` retains `'a` to thread the env borrow.

### Fallible constructors

All three context constructors (`DiscoveryFlags::new`, `DiscoveryEnv::new`, `DiscoveryContext::new`) are now `Result`-returning. Each validates its path arguments into `FilePath` / `DirPath` at construction time, converting filesystem path validation errors into domain-level discovery errors immediately. This eliminates deferred raw-path checks in the discovery engine for inputs known at context build time.

`Bootstrapper::discovery_context` returns `Result<DiscoveryContext<'_>, DiscoveryError>`.

### Error taxonomy (extensions to the original spec)

The error module was restructured with two separate sub-error types embedded in `DiscoveryError`:

- **`FlagOverrideError`** — fatal errors from explicit CLI flag validation:
  - `GlobalConfigPathNotFile { path, source }` — explicit config path does not resolve to a file.
  - `VaultPathNotDirectory { path, source }` — explicit vault path does not resolve to a directory.
  - Embedded as `DiscoveryError::Flag(#[from] FlagOverrideError)`.

- **`EnvironmentOverrideError`** — fatal errors from environment variable validation:
  - `GlobalConfigPathNotFile { path, source }` — env config path does not resolve to a file.
  - `VaultPathMissing { path }` — env vault path is empty / does not exist.
  - `VaultPathNotDirectory { path }` — env vault path exists but is not a directory.
  - `VaultPathInvalid { path, source }` — env vault path fails other path validation.
  - Embedded as `DiscoveryError::Env(#[from] EnvironmentOverrideError)`.
  - `EnvironmentOverrideError::from_vault_path_error(path, PathError)` maps `PathError` variants to the correct env error variant.

- **`DiscoveryError::InvalidAnchorDirectory { path, source }`** — anchor directory (cwd at invocation time) fails `DirPath` validation.

The flat `ExplicitPathMissing`, `ExplicitPathNotDirectory`, `EnvironmentPathMissing`, `EnvironmentPathNotDirectory` variants from the earlier engine were removed and replaced by the sub-error hierarchy. `engine.rs` was updated accordingly.

Flag override non-existence and not-a-directory cases are collapsed into a single variant (`VaultPathNotDirectory`) because `DirPath::try_new` returns `PathError::NotADirectory` for both missing and non-directory paths — the two conditions are indistinguishable at the `DirPath` validation level.

### `DiscoveryResult` Vec vs Box

`DiscoveryResult` stores `Vec<CandidatePath>` rather than the `Box<[CandidatePath]>` specified in the original issue. Rationale: during discovery the list is built up incrementally; freezing to `Box<[T]>` is a Bootstrapper concern for the boundary crossing to Config, deferred until that integration slice.

`into_parts() -> (Vec<CandidatePath>, Vec<CandidatePath>)` was added to allow consuming the result by value.

### Engine update

`discovery/engine.rs` `validate_override` was updated to produce `FlagOverrideError::VaultPathNotDirectory` for explicit paths and `EnvironmentOverrideError::VaultPathMissing` / `EnvironmentOverrideError::VaultPathNotDirectory` for environment paths, using `.into()` to coerce into `DiscoveryError`.

### Builder pattern on `DiscoveryContext` (post-initial)

The original `DiscoveryContext::new(flags, env, anchor)` three-argument constructor was replaced with a builder:

```rust
DiscoveryContext::new(anchor: &Path) -> Result<Self, DiscoveryError>
fn with_flags(self, flags: DiscoveryFlags) -> Self
fn with_env(self, env: DiscoveryEnv<'a>) -> Self
```

`anchor` is the only required field. `DiscoveryFlags` and `DiscoveryEnv` both derive `Default` (all-`None` / `false`); the context defaults to empty flags and env when neither builder method is called. This eliminates the forced construction of empty sub-structs (`DiscoveryFlags::new(None, None, false)`) at call sites where no overrides are present.

### `Bootstrapper` redesign (post-initial)

`Bootstrapper` became generic over `D: DiscoveryPort`:

```rust
pub(crate) struct Bootstrapper<D: DiscoveryPort> { port: D }
```

`discovery_context()` was renamed `build_context()`, made a static method, and now accepts `Option<DiscoveryFlags>` and `Option<DiscoveryEnv>` (calling `with_flags` / `with_env` only when `Some`).

A new `discover()` instance method delegates to `self.port.discover(context)`, making the bootstrap layer testable with a mock `D` without touching the filesystem.

### `DiscoveryPort` inbound port (post-initial)

`lithos-core/src/discovery/port.rs` defines:

```rust
pub(crate) trait DiscoveryPort {
    fn discover(
        &self,
        context: &DiscoveryContext<'_>,
    ) -> Result<DiscoveryResult, DiscoveryError>;
}
```

The trait lives in `discovery/` (domain owns its inbound port). `Bootstrapper<D: DiscoveryPort>` calls through it. `DiscoveryEngine` will implement this trait when the full discovery pipeline lands. In tests, `MockPort` implements `DiscoveryPort` returning a fixed result, proving the orchestration layer is independent of the filesystem.

## Blocked by

None - can start immediately.

## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Establish the Discovery contracts and minimal Bootstrapper context-acquisition seam before redesigning discovery internals.

**Current behavior:**
The existing discovery redesign issue starts with a full DiscoveryService/processor rewrite before the application layer can build the runtime context Discovery needs. That made the implementation scope too large and blurred the boundary between app context acquisition and Discovery execution.

**Desired behavior:**
`discovery/` declares the input/output/report/error contracts for the redesigned process, while `app/Bootstrapper` acquires runtime context and builds `DiscoveryContext`. No discovery execution happens yet. Later issues can implement phase helpers, the typestate processor, the service facade, and Config integration against these stable contracts.

**Key interfaces:**
- `DiscoveryContext<'a>` — Discovery-owned input contract filled by Bootstrapper.
- `DiscoveryFlags` — CLI-derived config path, vault path, and `suppress_global`. Owns validated `FilePath`/`DirPath`.
- `DiscoveryEnv<'a>` — env-derived config path, vault path, and ceiling directory raw data. Owns validated `FilePath`/`DirPath`; borrows `OsStr` for ceiling dirs.
- `CandidatePath` — validated candidate with `base: DirPath` and `path: FilePath`.
- `DiscoveryResult` — separate ordered `vault` and `global` candidate vectors. Provides `into_parts()`.
- `DiscoveryReport` — non-fatal phase metadata only.
- `Bootstrapper` context-acquisition seam — builds `DiscoveryContext` without invoking Discovery or Config. Returns `Result`.

**Acceptance criteria:**
- [x] Context acquisition is app-owned and Discovery execution is not implemented in this slice.
- [x] `InvocationInput` is not introduced; `DiscoveryContext` is the canonical input contract.
- [x] Explicit config file and explicit vault directory overrides are represented separately.
- [x] Invalid explicit overrides are fatal errors, not report entries.
- [x] Global suppression is represented as per-invocation flag state and report metadata.
- [x] Discovery result has `vault` and `global` lists, no separate `vault_root`, and no stored format field.

**Out of scope:**
- Discovery traversal, probing, selection, global resolution, or finalization logic.
- Typestate processor implementation.
- `DiscoveryService::discover()` implementation.
- Config builder decoupling.
- CLI discovery commands.
