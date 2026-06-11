---
title: 11-discovery-redesign-decisions
category: architecture
label: in-progress
date_created: 2026-06-11
---

# Discovery Redesign & Bootstrap Orchestration — Grilling Session Decisions

This document captures all decisions and open questions from the grilling session
covering the `discovery/` module redesign, the `Bootstrapper` orchestration in
`app/`, and the proposed changes to the `config/` module.

---

## 1. Core Architectural Principle

The flow is a strict pipeline: **Discovery → Config → Indexer**.

Dependencies flow in one direction only:
- `app/` (Bootstrapper) orchestrates `discovery/` and `config/`, and is the
  only layer that knows about both.
- `discovery/` must never import from `config/`.
- `config/` must never orchestrate `discovery/`.

This is the canonical hexagonal architecture for Lithos's bootstrap sequence.

---

## 2. `discovery/` Module Redesign

### 2.1 Guiding Process

The full 6-phase discovery process that the `DiscoveryService` must implement:

1. **Initialization** — Context and target definitions (CWD, filenames, boundary
   markers). Context Acquisition (CWD, anchor) is the *caller's* responsibility
   (the Bootstrapper). `DiscoveryService` receives the anchor; it does not
   acquire it.
2. **Explicit Preemption** — CLI flag / env var path resolution and validation.
   Terminates the process if a valid explicit path is provided.
3. **Anchor Normalization** — Canonicalize the anchor to an absolute path.
4. **Local Traversal** — Iterative ascending walk from the anchor, probing each
   directory for target filenames, respecting ceiling and project boundary
   markers.
5. **Global Resolution (Fallback)** — Probe OS-level config directories if no
   local candidates were found (or accumulative layering is active).
6. **Finalization** — Aggregate, deduplicate, and order all candidates by
   specificity priority.

### 2.2 Proposed Module Structure

| Module              | Responsibility                                                                                   | Status          |
| ------------------- | ------------------------------------------------------------------------------------------------ | --------------- |
| `service.rs` (new)  | `DiscoveryService::run()` — orchestrates phases 2–6                                              | New             |
| `override.rs` (new) | `OverrideResolver` — validates explicit CLI/env paths, verifies file existence vs. directory     | New             |
| `probe.rs`          | `FolderProbe` — generic: probes one directory for target filenames (replaces `VaultRootProbe` + `GlobalRootProbe`) | Refactor        |
| `walk.rs`           | `BoundedAscent` + ceiling parsing — traversal iterator, unchanged in responsibility              | Keep            |
| `selector.rs`       | `select_candidate` + deduplication/ordering (absorbs `select_markers` from engine)               | Keep + extend   |
| `policy.rs`         | `DiscoveryPolicy` + target filenames + boundary markers + precedence rules                       | Keep + extend   |
| `error.rs`          | `DiscoveryError` with nested transparent sub-errors (see §2.4); absorbs `diagnostics.rs`          | Refactor        |
| `diagnostics.rs`    | **Delete** — warnings move to structured fields on results or `tracing::warn!`                    | Delete          |
| `engine.rs`         | **Delete** — responsibilities redistributed to `service.rs`, `override.rs`, `selector.rs`        | Delete          |

### 2.3 `FolderProbe` — Generalised Probe

`VaultRootProbe` and `GlobalRootProbe` in the current `probe.rs` are identical
in structure; they differ only in which `MarkerPattern` list they use.

**Decision:** Replace both with a single `FolderProbe` that accepts a
`&[MarkerPattern]` at construction time. `policy.rs` defines the two pattern
lists (`ROOT_MARKER_FILES`, `GLOBAL_MARKER_FILES`). `DiscoveryService` passes the
appropriate list when constructing the probe.

The `DiscoveryProbe` trait remains but its name may be simplified to `Probe` or
kept as-is if it aids clarity.

### 2.4 Error Structure — Layered Transparent Errors

`DiscoveryError` becomes a transparent envelope with `#[from]` conversions:

```
DiscoveryError
 ├── Override(OverrideError)         ← Phase 2: explicit/env path validation
 │    ├── Missing { path, source }
 │    └── NotADirectory { path, source }
 ├── Traversal(TraversalError)       ← Phases 3+4: anchor normalization, walk
 │    ├── AnchorCanonicalize { path, source }
 │    └── ReadDirectory { path, source }
 └── GlobalResolution(GlobalError)   ← Phase 5: global probe failures
      └── ReadDirectory { path, source }
```

Each sub-error is self-describing and scoped to its phase. `DiscoveryError`
wraps them with `#[error(transparent)]` + `#[from]`.

### 2.5 Non-Fatal Structured Information on `VaultDiscoveryResult`

The caller (Bootstrapper / CLI) needs structured access to non-fatal conditions.
These are **not** errors — they are informational fields on `VaultDiscoveryResult`.

| Field                                                                    | Phase | Reason caller needs it                                               |
| ------------------------------------------------------------------------ | ----- | -------------------------------------------------------------------- |
| `skipped_override: Option<SkippedOverride>` (path + reason)               | 2     | User gave explicit path that was invalid; CLI must report it         |
| `skipped_ceilings: Vec<SkippedCeiling>` (path + reason: empty / invalid) | 4     | Invalid ceiling segments alter traversal scope; CLI must report them |
| `traversal_stop_reason: TraversalStopReason`                             | 4     | Ceiling / ProjectBoundary / FilesystemRoot / NotStarted              |

`SkippedOverride.reason`:
- `InvalidPath` — path does not exist
- `NotADirectory` — path is a file

`SkippedCeiling.reason`:
- `EmptySegment`
- `InvalidPath` — does not exist or is not a directory

`TraversalStopReason`:
- `FilesystemRoot`
- `ProjectBoundaryMarker { marker: PathBuf }` — e.g. `.git` found
- `CeilingEnforced { ceiling: PathBuf }`
- `NotStarted` — traversal was never entered (explicit preemption fired)

**Open Question (Q1):** `GlobalDiscoveryResult` does not currently carry
non-fatal diagnostics. Are there non-fatal conditions during global resolution
that the caller should know about (e.g., a global directory candidate that was
inaccessible due to permissions)? Tentatively no — global probing silently
skips inaccessible directories — but this should be confirmed.

### 2.6 `DiscoveryService::run()` — Single Entry Point (MVP)

For the MVP, `DiscoveryService` exposes exactly one method:

```rust
pub fn run(&self, input: DiscoveryInput<'_>) -> Result<DiscoveryResult, DiscoveryError>
```

`DiscoveryResult` aggregates:
- `vault: VaultDiscoveryResult`
- `global: GlobalDiscoveryResult`

A second method (two-phase discovery after finding only a global config) is
**explicitly deferred** until the foundational pipeline is proven. It must not
be designed into the MVP interface.

### 2.7 `DiscoveryInput` — What the Bootstrapper Provides

`DiscoveryInput` carries everything the Bootstrapper resolved from the runtime
environment before calling `DiscoveryService::run()`:

- `flag_path: Option<&Path>` — from CLI `--vault` flag
- `env_path: Option<&Path>` — from `LITHOS_VAULT` env var
- `cwd: &Path` — current working directory (Context Anchor)
- `ceiling_dirs_raw: Option<&OsStr>` — raw ceiling list (env var)
- `global_directories: &[GlobalDirectoryCandidate]` — OS-resolved paths
  (XDG, UserConfig, SystemConfig); the Bootstrapper resolves platform-specific
  paths and passes them in
- `suppress_global: bool` — corresponds to `--no-global-config`

The Bootstrapper owns Context Acquisition (CWD, env vars, platform path
resolution). `DiscoveryService` receives already-gathered context.

---

## 3. `Bootstrapper` — `app/` Orchestration

### 3.1 Role

The `Bootstrapper` is the single component in `lithos-core/src/app/` that
orchestrates the bootstrap sequence. It is the *only* place in the codebase
where `discovery/` and `config/` are both imported.

Its responsibilities:
1. Acquire runtime context (CWD, env vars, platform paths).
2. Call `DiscoveryService::run(input)` to get `DiscoveryResult`.
3. Log non-fatal diagnostic fields from `DiscoveryResult` (skipped ceilings,
   skipped override, traversal stop reason).
4. Pass `VaultDiscoveryResult` and `GlobalDiscoveryResult` directly to
   `config::Builder`.
5. Return the resolved `Config` to the caller (CLI command handler).

The Bootstrapper does **not** construct a `Config` itself — it delegates that
entirely to `config::Builder`.

### 3.2 Naming

"Bootstrap Orchestration" is the agreed term for what `app/` performs at
program startup before any business logic runs. The Bootstrapper is the
component that runs this sequence.

### 3.3 Future: Two-Phase Discovery

After the MVP is established, the Bootstrapper will support a second pass:
1. Run `DiscoveryService::run()` (single-pass).
2. If only global config is found (no vault), parse the global config to extract
   `trusted_paths`.
3. Call a second discovery method (not yet designed) with those paths as
   additional search anchors.

This is deferred. The Bootstrapper's interface must not anticipate it yet.

---

## 4. `config/` Module Changes

### 4.1 `ConfigBuilder` Input

**Decision:** `ConfigBuilder` is refactored to accept
`(VaultDiscoveryResult, GlobalDiscoveryResult)` directly, replacing the
current arrangement where `Builder` internally constructs a `DiscoveryEngine`
and runs discovery itself.

`ConfigBuilder` is a pure config-domain component. It does not know about
discovery orchestration. It receives resolved discovery outputs and uses them
only to locate file paths for config ingestion.

### 4.2 `config/root.rs` and `config/discovery.rs`

**Decision:** `config/root.rs` is deleted. `DiscoveredConfigFile` and
`ConfigDiscoveryResult` are redundant wrappers around `DiscoveredMarker` from
`discovery/engine.rs`. The `ConfigDiscoveryResult::from_discovery()` conversion
is removed along with the file.

`config/discovery.rs` (`ConfigDiscoveryPipeline`) is refactored: instead of
accepting `ConfigDiscoveryResult`, it accepts `VaultDiscoveryResult` and
`GlobalDiscoveryResult` directly.

**Open Question (Q2):** Should `ConfigDiscoveryPipeline` be renamed to reflect
its revised role? It is now clearly a "file metadata reader" that translates
discovered marker paths into `FsEntry` objects with database view lookup. A
name like `ConfigFileLoader` or `ConfigIngestionPipeline` may be more precise.

### 4.3 Coupling Removal

The import `use crate::discovery::engine::{DiscoveryEngine, DiscoveryInput, GlobalDiscoveryInput}` is removed from `config/builder.rs`. After the refactor, `config/` imports only from `discovery::engine` the result types (`VaultDiscoveryResult`, `GlobalDiscoveryResult`, `DiscoveredMarker`). These are pure data types with no behaviour; the dependency is one-way and acceptable.

---

## 5. Open Questions

| ID  | Question                                                                                           | Status      |
| --- | -------------------------------------------------------------------------------------------------- | ----------- |
| Q1  | Are there non-fatal conditions during global resolution the caller should know about?               | Unanswered  |
| Q2  | Should `ConfigDiscoveryPipeline` be renamed to reflect its revised role as a file metadata reader? | Unanswered  |
| Q3  | What is the exact public interface of `DiscoveryService`? (input/output type signatures)            | Unanswered  |
| Q4  | Should `DiscoveryResult` be a flat struct with `vault` and `global` fields, or two separate return values? | Unanswered |
| Q5  | Does `DiscoveryPolicy` own the target filename lists and boundary marker definitions, or should those be separate from precedence rules? | Unanswered |

---

## 6. Deferred / Out of Scope for This Session

- Two-phase discovery (global config `trusted_paths` → second vault search).
- CLI subcommands (`config where`, `config list-sources`, `config check`) — tracked in issue `10-cli-discovery-subcommands.md`.
- `GlobalDiscoveryResult` non-fatal diagnostics (pending Q1).
- Renaming `ConfigDiscoveryPipeline` (pending Q2).
