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

### 2.2 Public API: Two-Phase Builder Pattern

`DiscoveryService` uses a two-phase builder as its public API:

1. **`DiscoveryServiceBuilder`** — one-time configuration: target filenames, boundary markers, precedence rules, global directory resolution, `suppress_global`. Owns its data, validates at `build()`.
2. **`DiscoveryService::discover()`** — per-invocation input: `flag_path`, `env_path`, `cwd`, `ceiling_dirs_raw`. Caller provides only what changes per call.

### 2.3 Internal Typestate Pipeline

Inside `discover()`, the 6-phase process uses a **typestate pattern** (following the Hoverbear state machine pattern). Each phase is a typed node; invalid transitions are compile errors:

```
Initialized → Preempted → Anchored → Traversed → GlobalResolved → Finalized
```

Each step is `impl From<Previous> for Next`, zero-cost at runtime. The caller sees only `discover()` — the state machine is an internal implementation detail.

### 2.6 Module Structure

| Module              | Responsibility                                                                                              | Status        |
| ------------------- | ----------------------------------------------------------------------------------------------------------- | ------------- |
| `service.rs` (new)  | `DiscoveryService` + `DiscoveryServiceBuilder` — public API for discovery                                   | New           |
| `override.rs` (new) | `OverrideResolver` — validates explicit CLI/env paths, verifies file existence vs. directory                | New           |
| `probe.rs`          | `FolderProbe` — generic: probes one directory for target filenames (replaces `VaultRootProbe` + `GlobalRootProbe`) | Refactor      |
| `walk.rs`           | `BoundedAscent` + ceiling parsing — traversal iterator, unchanged in responsibility                         | Keep          |
| `selector.rs`       | `select_candidate` + deduplication/ordering (absorbs `select_markers` from engine)                          | Keep + extend |
| `policy.rs`         | `DiscoveryPolicy` + target filenames + boundary markers + precedence rules                                  | Keep + extend |
| `error.rs`          | `DiscoveryError` with nested transparent sub-errors (see §2.8); absorbs `diagnostics.rs`                    | Refactor      |
| `diagnostics.rs`    | **Delete** — warnings move to `DiscoveryReport` (structured) or `tracing::warn!` (inline)                   | Delete        |
| `engine.rs`         | **Delete** — responsibilities redistributed to `service.rs`, `override.rs`, `selector.rs`                  | Delete        |

### 2.7 `FolderProbe` — Generalised Probe

`VaultRootProbe` and `GlobalRootProbe` in the current `probe.rs` are identical
in structure; they differ only in which `MarkerPattern` list they use.

**Decision:** Replace both with a single `FolderProbe` that accepts a
`&[MarkerPattern]` at construction time. `policy.rs` defines the two pattern
lists (`ROOT_MARKER_FILES`, `GLOBAL_MARKER_FILES`). `DiscoveryService` passes the
appropriate list when constructing the probe.

The `DiscoveryProbe` trait remains but its name may be simplified to `Probe` or
kept as-is if it aids clarity.

### 2.8 Error Structure — Layered Transparent Errors

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

### 2.9 Output Types — `DiscoveryResult` and `DiscoveryReport`

`DiscoveryService::discover()` returns two distinct types:

**`DiscoveryResult`** — pure domain data consumed by `config/Builder` and
downstream components. No process metadata here.

```rust
pub struct DiscoveryResult {
    /// The directory in which vault markers were found (the Vault Root).
    /// `None` if no vault was located.
    pub vault_root: Option<PathBuf>,
    /// All vault marker candidates found, ordered by precedence (winner first).
    /// Empty if no vault was located.
    pub vault: Box<[DiscoveredMarker]>,
    /// All global marker candidates found, ordered by precedence (winner first).
    /// Empty if no global config was located.
    pub global: Box<[DiscoveredMarker]>,
}
```

`Box<[DiscoveredMarker]>` is used because the list is built during traversal
and then frozen — it signals immutability and avoids excess heap capacity.

**`DiscoveryReport`** — process metadata for the Bootstrapper / CLI only.
Downstream components (`config/Builder`) never see this.

```rust
pub struct DiscoveryReport {
    /// An explicit CLI/env path that was provided but failed validation.
    /// `None` if no override was given or the override succeeded.
    pub skipped_override: Option<SkippedOverride>,
    /// Ceiling path segments that were skipped during traversal setup.
    pub skipped_ceilings: Box<[SkippedCeiling]>,
    /// Why local traversal stopped (or did not start).
    pub traversal_stop_reason: TraversalStopReason,
}
```

Supporting types:

```rust
pub struct SkippedOverride {
    pub path: PathBuf,
    pub reason: SkippedOverrideReason,
}
pub enum SkippedOverrideReason {
    /// Path does not exist on the filesystem.
    InvalidPath,
    /// Path exists but is a file, not a directory.
    NotADirectory,
}

pub struct SkippedCeiling {
    pub segment: PathBuf,
    pub reason: SkippedCeilingReason,
}
pub enum SkippedCeilingReason {
    /// Segment was empty or whitespace.
    EmptySegment,
    /// Segment does not exist or is not a directory.
    InvalidPath,
}

pub enum TraversalStopReason {
    /// Traversal was never started — explicit preemption fired first.
    NotStarted,
    /// Walk reached the filesystem root (/ or C:\).
    FilesystemRoot,
    /// Walk stopped at a project boundary marker (e.g. `.git`).
    ProjectBoundaryMarker { marker: PathBuf },
    /// Walk stopped at an enforced ceiling directory.
    CeilingEnforced { ceiling: PathBuf },
}
```

**Rationale for separation:** Non-fatal process metadata does not belong on
`DiscoveryResult` because it is not needed by any downstream component
(`config/Builder`, Indexer). Placing it on a separate `DiscoveryReport`
keeps `DiscoveryResult` clean, avoids polluting the domain type with
orchestration concerns, and makes it clear that the Bootstrapper — not
downstream components — is responsible for acting on diagnostics.

This also eliminates the old `VaultDiscoveryResult` / `GlobalDiscoveryResult`
split which perpetuated an incorrect dual-discovery design. Discovery is one
multi-phase process; its output is one `DiscoveryResult`.

### 2.6 `DiscoveryService::discover()` — Single Entry Point (MVP)

For the MVP, `DiscoveryService` exposes exactly one method:

```rust
pub fn run(&self, input: DiscoveryInput<'_>)
    -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>
```

A second method (two-phase discovery after finding only a global config) is
**explicitly deferred** until the foundational pipeline is proven. It must not
be designed into the MVP interface.

### 2.7 `DiscoveryInput` — What the Bootstrapper Provides

`DiscoveryInput` carries everything the Bootstrapper resolved from the runtime
environment before calling `DiscoveryService::discover()`:

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
2. Call `DiscoveryService::run(input)` to get `(DiscoveryResult, DiscoveryReport)`.
3. Act on `DiscoveryReport`: emit `tracing::warn!` for skipped ceilings/overrides,
   surface diagnostics for CLI verbose output.
4. Pass `DiscoveryResult` to `config::Builder`.
5. Return the resolved `Config` to the CLI command handler.

The Bootstrapper does **not** construct a `Config` itself — it delegates that
entirely to `config::Builder`.

**Open Question (Q-B1):** Does the Bootstrapper return just `Config` to the CLI
handler, or a `BootstrapResult { config: Config, report: DiscoveryReport }`?
The CLI commands `config where` and `config list-sources` need access to the
`DiscoveryReport` to display verbose output. This must be decided before the
`Bootstrapper` interface is finalised.

### 3.2 Naming

"Bootstrap Orchestration" is the agreed term for what `app/` performs at
program startup before any business logic runs. The Bootstrapper is the
component that runs this sequence.

### 3.3 Future: Two-Phase Discovery

After the MVP is established, the Bootstrapper will support a second pass:
1. Run `DiscoveryService::discover()` (single-pass).
2. If only global config is found (no vault), parse the global config to extract
   `trusted_paths`.
3. Call a second discovery method (not yet designed) with those paths as
   additional search anchors.

This is deferred. The Bootstrapper's interface must not anticipate it yet.

---

## 4. `config/` Module Changes

### 4.1 `ConfigBuilder` Input

**Decision:** `ConfigBuilder` is refactored to accept `DiscoveryResult` directly,
replacing the current arrangement where `Builder` internally constructs a
`DiscoveryEngine` and runs discovery itself.

`ConfigBuilder` is a pure config-domain component. It does not know about
discovery orchestration. It receives `DiscoveryResult` and uses the
`vault`/`global` marker lists and `vault_root` only to locate file paths for
config ingestion.

### 4.2 `config/root.rs` and `config/discovery.rs`

**Decision:** `config/root.rs` is deleted. `DiscoveredConfigFile` and
`ConfigDiscoveryResult` are redundant wrappers around `DiscoveredMarker` from
`discovery/`. The `ConfigDiscoveryResult::from_discovery()` conversion is
removed along with the file.

`config/discovery.rs` (`ConfigDiscoveryPipeline`) is refactored: instead of
accepting `ConfigDiscoveryResult`, it accepts `DiscoveryResult` directly and
uses `vault[0]` / `global[0]` (the winner, first element) for file ingestion.

**Open Question (Q2):** Should `ConfigDiscoveryPipeline` be renamed to reflect
its revised role? It is now clearly a "file metadata reader" that translates
discovered marker paths into `FsEntry` objects with database view lookup. A
name like `ConfigFileLoader` or `ConfigIngestionPipeline` may be more precise.

### 4.3 Coupling Removal

The import of `DiscoveryEngine`, `DiscoveryInput`, `GlobalDiscoveryInput` from
`config/builder.rs` is removed entirely. After the refactor, `config/` imports
only `DiscoveryResult` and `DiscoveredMarker` from `discovery/`. These are pure
data types; the dependency is one-way and acceptable.

---

## 5. Open Questions

### 5.1 Resolved

| ID   | Question                                                                                                    | Resolution |
| ---- | ----------------------------------------------------------------------------------------------------------- | ---------- |
| Q1   | Are there non-fatal conditions during global resolution the caller should know about?                        | **Silent skip.** Inaccessible global directories are silently skipped, continue to next candidate. `tracing::warn!` used for debugging. No `DiscoveryReport` field. |
| Q2   | Should `ConfigDiscoveryPipeline` be renamed (e.g. `ConfigFileLoader`)?                                      | **No rename.** Keeps existing name; will be integrated into processors/builder as it lands. |
| Q5   | Does `DiscoveryPolicy` own the target filename lists and boundary marker definitions, or separate?           | **Move to `policy.rs`.** `ROOT_MARKER_FILES`, `GLOBAL_MARKER_FILES`, and boundary markers are configuration about *what to look for* and *when to stop* — they belong in `DiscoveryPolicy`. |
| Q6   | Where is the project boundary marker list (`.git`, `.workspace`) defined? Can it be overridden?             | **Fixed const in `policy.rs`.** Not user-overridable. Ceiling env vars already provide escape-hatch walk control. Boundary markers follow **probe-then-stop** semantics (probe dir for target markers first, then stop ascending), governed by `allow_marker_at_ceiling`. |

### 5.2 Still Open

| ID   | Question                                                                                                    | Status     |
| ---- | ----------------------------------------------------------------------------------------------------------- | ---------- |
| Q-B1 | Does the Bootstrapper return `Config` or `BootstrapResult { config, report }` to the CLI handler?          | **`BootstrapResult { config: Config, report: DiscoveryReport }`.** Most ergonomic; CLI ignores report in hot path; subcommands destructure it. |

Previously open questions now resolved:
- **Q3** (DiscoveryService interface) — resolved: `run() -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>`.
- **Q4** (DiscoveryResult shape) — resolved: flat struct with `vault_root`, `vault: Box<[DiscoveredMarker]>`, `global: Box<[DiscoveredMarker]>`.

---

## 6. Deferred / Out of Scope for This Session

- Two-phase discovery (global config `trusted_paths` → second vault search).
- CLI subcommands (`config where`, `config list-sources`, `config check`) —
  tracked in issue `10-cli-discovery-subcommands.md`.
- `GlobalDiscoveryResult` non-fatal diagnostics (pending Q1).
- Renaming `ConfigDiscoveryPipeline` (pending Q2).
- `BootstrapResult` vs bare `Config` return (pending Q-B1).
