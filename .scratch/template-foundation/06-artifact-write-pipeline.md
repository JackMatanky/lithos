---
title: 06-artifact-write-pipeline
category: enhancement
label: ready-for-agent
status: completed
branch:
merge_commit:
date_created: 2026-06-11
date_completed: 2026-06-24
---

# Template Artifact Write Pipeline

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement `TemplateArtifact<State>` — a typestate write pipeline that enforces ordering of target resolution, conflict checking, and file commit using the hoverbear generic state machine pattern.

States and valid transitions:
- `TemplateArtifact<Rendered>` → `TemplateArtifact<TargetResolved>` via `try_resolve_target` (fallible; path validation via `RelativeFilePath::try_new`, which rejects absolute paths and `..` traversal)
- `TemplateArtifact<TargetResolved>` → `TemplateArtifact<ReadyToCommit>` via `try_check_conflict` (fallible; ensures parent directory exists)
- `TemplateArtifact<ReadyToCommit>` → `TemplateArtifact<Committed>` via `commit` (file write; uses `File::create_new`)

The outer struct carries shared data (rendered content, template name, etc.); the `state: S` field carries per-state data. Invalid transitions (e.g. `Rendered → Committed` directly) are impossible by type construction — no runtime guard needed.

Commit behavior:
- Uses `File::create_new` (stable since Rust 1.77.0) to atomically create the file, failing with `AlreadyExists` if the destination exists. No separate existence pre-check — this eliminates the TOCTOU race.
- Writes to a vault-safe target path resolved from the vault root.
- Does not use raw `std::fs` — uses the FS context.

Path validation (in `Rendered → TargetResolved` transition):
- Rejects absolute paths
- Rejects paths containing `..` traversal components
- Uses `RelativeFilePath::try_new` (construction is the validation — `PathError::NotRelative` is absolute path, `PathError::ParentTraversal` is `..` traversal)

`TemplateArtifact<Committed>` is the terminal state; no further transitions are defined.

`TemplateArtifactSet<State>` for multi-file packs is explicitly out of scope.

## Acceptance criteria

- [x] `TemplateArtifact<S>` is generic over a state type parameter, with shared fields in the outer struct and per-state data in `state: S`
- [x] `try_resolve_target` converts `TemplateArtifact<Rendered>` → `Result<TemplateArtifact<TargetResolved>, TemplateArtifactError>`; path validation uses `WriteTarget::try_new` (no separate `PathValidator` call)
- [x] Commit from `TargetResolved` to `Committed` uses `File::create_new` via the `FileWriter` port (not a pre-check + create sequence, parent directories are created here)
- [x] Absolute target paths are rejected at the `Rendered → TargetResolved` transition
- [x] Traversal paths (`..` components) are rejected at the `Rendered → TargetResolved` transition
- [x] Hidden files and current-dir components are rejected at the `Rendered → TargetResolved` transition
- [x] No raw `std::fs` usage — all I/O goes through the FS context (`FileWriter` trait)
- [x] `TemplateArtifact<Committed>` is the terminal state
- [x] `TemplateArtifactSet` is not implemented
- [x] Tests cover: vault-relative target success (file created), absolute path rejection, traversal path rejection, hidden path rejection, `AlreadyExists` failure from `File::create_new`, generic I/O failures, single-file creation verified end-to-end; invalid transitions are impossible by type construction (no runtime test needed — compiler enforces this)

## Blocked by

- `issue-01-domain-models.md`

---

## TDD Plan

### Error design

Follows existing wrapping pattern (`TemplateReadError` wraps `trace_fs::ReadError`):

| Layer | Type | Location |
|-------|------|----------|
| FS crate | `trace_fs::error::WriteError` | `crates/fs/src/error.rs` — `AlreadyExists { path }`, `Io { path, source }` |
| Template crate | `TemplateWriteError` | `crates/template/src/error.rs` — wraps `WriteError` via `#[from]` |
| Template crate | `TemplateArtifactError` | `crates/template/src/error.rs` — `AbsolutePathRejected(PathBuf)`, `TraversalRejected(PathBuf)`, `Write(TemplateWriteError)` |

### Phase 0 — Prerequisites (tracer bullet 1)

Enable cross-crate `Writer` access and add `create_new`.

| Step | File | Change | Test module | Test functions |
|------|------|--------|-------------|----------------|
| 0.1 | `crates/fs/src/error.rs` | Add `WriteError` enum: `AlreadyExists { path: PathBuf }`, `Io { path: PathBuf, source: io::Error }` + `#[from] FsError` | `write_error` | `already_exists_displays_path`, `io_displays_path_and_source`, `implements_error_trait` |
| 0.2 | `crates/fs/src/writer.rs` | Add `create_new(&self, path: &Path, contents: &[u8]) -> Result<(), WriteError>` — validates, resolves, creates parent dirs via `create_dir_all`, calls `std::fs::File::create_new`, writes content via `write_all` | `create_new` | `creates_file_when_not_exists`, `rejects_existing_file`, `rejects_invalid_path` |
| 0.3 | `crates/fs/src/lib.rs` | Add `pub use writer::Writer as FsWriter;` + re-export `WriteError` | — | Compile check |

### Phase 1 — Template error types (tracer bullet 2)

| Step | File | Change | Test module | Test functions |
|------|------|--------|-------------|----------------|
| 1.1 | `crates/template/src/error.rs` | Add `TemplateWriteError` wrapping `trace_fs::error::WriteError` | `template_write_error` | `preserves_already_exists_variant`, `preserves_io_variant` |
| 1.2 | `crates/template/src/error.rs` | Add `TemplateArtifactError`: `AbsolutePathRejected(PathBuf)`, `TraversalRejected(PathBuf)`, `Write(TemplateWriteError)` | `template_artifact_error` | `absolute_path_rejected_displays_path`, `traversal_rejected_displays_path`, `write_preserves_source`, `implements_error_trait` |
| 1.3 | `crates/template/src/lib.rs` | Re-export `TemplateArtifactError`, `TemplateWriteError` | — | Compile check |

### Phase 2 — State types & artifact refactor (tracer bullet 3)

Replace `PhantomData<State>` with `state: S`; add typed state markers.

| Step | File | Change | Test module | Test functions |
|------|------|--------|-------------|----------------|
| 2.1 | `crates/template/src/artifact.rs` | Add state structs: `TargetResolved(RelativeFilePath)`, `ReadyToCommit(RelativeFilePath)`, `Committed` (unit) | `state` | `target_resolved_holds_path`, `ready_to_commit_holds_path`, `committed_is_zero_sized` |
| 2.2 | `crates/template/src/artifact.rs` | Change `state: PhantomData<State>` → `state: S`; update `TemplateArtifact<Rendered>::rendered()` to set `state: Rendered` | `constructor`, `accessors` | `stores_template_and_content`, `maintains_partial_eq`, `rendered_state_is_rendered` |

### Phase 3 — Transition methods (tracer bullets 4–6)

One fallible transition per tracer bullet.

**Step 3.1 — `try_resolve_target`**

| Change | Test module | Test functions |
|--------|-------------|----------------|
| `impl TemplateArtifact<Rendered> { fn try_resolve_target(&self, path: &str) -> Result<TemplateArtifact<TargetResolved>, TemplateArtifactError> }` — calls `RelativeFilePath::try_new`, maps `PathError::NotRelative` → `AbsolutePathRejected`, `PathError::ParentTraversal` → `TraversalRejected` | `try_resolve_target` | `returns_target_resolved_when_path_valid`, `rejects_absolute_path`, `rejects_traversal_path` |

**Step 3.2 — `try_check_conflict`**

| Change | Test module | Test functions |
|--------|-------------|----------------|
| `impl TemplateArtifact<TargetResolved> { fn try_check_conflict(&self, fs_writer: &FsWriter) -> Result<TemplateArtifact<ReadyToCommit>, TemplateArtifactError> }` — calls `fs_writer.create_dir_all` on parent of resolved path (no existence check — `File::create_new` is the atomic TOCTOU-free guard) | `try_check_conflict` | `returns_ready_to_commit_when_dir_created`, `returns_ready_to_commit_when_dir_exists` (idempotent) |

**Step 3.3 — `commit`**

| Change | Test module | Test functions |
|--------|-------------|----------------|
| `impl TemplateArtifact<ReadyToCommit> { fn commit(&self, fs_writer: &FsWriter) -> Result<TemplateArtifact<Committed>, TemplateArtifactError> }` — calls `fs_writer.create_new(target, content.as_bytes())`, maps `WriteError { AlreadyExists, Io }` → `TemplateArtifactError::Write(TemplateWriteError(...))` | `commit` | `creates_file_with_content`, `rejects_existing_file` |

### Phase 4 — End-to-end integration (tracer bullet 7)

| Step | File | Test module | Test functions |
|------|------|-------------|----------------|
| 4.1 | `crates/template/src/artifact.rs` | `pipeline` | `writes_file_end_to_end`, `rejects_absolute_path`, `rejects_traversal_path`, `rejects_existing_file` |

### Phase 5 — Quality gate

- [ ] `mise run fmt`
- [ ] `mise run lint`
- [ ] `mise run test:unit`
- [ ] No `unwrap()` / `expect()` / `panic!` in production code
- [ ] `TemplateWriteError` follows `TemplateReadError` conventions (wrapping, `#[error(transparent)]`, `#[from]`)
- [ ] All tests use Structure A (submodules), verb-first naming (`returns_*`, `rejects_*`)
- [ ] Equality assertions via `pretty_assertions`
- [ ] `#[cfg(test)] mod tests` within each implementation file

### Out of scope

- `TemplateArtifactSet<State>` for multi-file packs
- Overwrite, skip, rename, append, or other conflict policies
- Template service, processor, or CLI (issues 04, 07, 08)
- Any engine integration

---

## Adversarial Review (post-implementation)

The first implementation landed, compiles, lints clean, formats clean, and passes
the full suite (2149 nextest + doctests). An adversarial review against
`rust-best-practices`, the FS/Template `CONTEXT.md` boundaries, and
`docs/engineering/testing/unit*.md` found the suite green but the design unsound
in several places. Severities and `file:line` below are against the initial
implementation.

### Findings

**Critical — architecture / boundaries**

- **C1. Domain model depends on a concrete FS adapter.** `artifact.rs` imports
  `trace_fs::FsWriter` and passes `&FsWriter` into transitions
  (`artifact.rs:18,149,188`). No port/trait. This is a hexagonal inversion and
  contradicts the Template `CONTEXT.md` invariant that the **Template Service**
  owns "target resolution, conflict checks, and commit orchestration".
- **C2. An FS adapter was promoted to `pub` solely to serve dead code.**
  `fs/lib.rs:82` plus `writer.rs:22,41,61,121` flipped `Writer`/`new`/
  `create_new`/`create_dir_all` from `pub(crate)` to `pub`. The entire pipeline
  has **no production caller** (only `engine/mini_jinja.rs:83` constructs
  `TemplateArtifact<Rendered>`); the `pub` surface exists only for tests.

**Important**

- **I1. `&self` + `.clone()` is the wrong ownership model for a typestate.**
  Transitions take `&self` and clone `template`/`content` at every stage
  (`artifact.rs:123-124,164-165,198-199`). This defeats single-use linearity and
  heap-copies the rendered body up to 3×. Should consume `self` by value.
- **I2. Validation failures are laundered as `WriteError::Io`.**
  `writer.rs:108-112` maps every `Validator` rejection (traversal/hidden/absolute)
  into `io::Error(InvalidInput)` → `WriteError::Io`, so a forbidden target reads
  as a disk I/O error. Dishonest on a security-relevant path.
- **I3. `try_check_conflict` is redundant and misnamed.** `Writer::create_new`
  already creates parent dirs (`writer.rs:73-80`), so the middle stage's
  `create_dir_all` (`artifact.rs:158-160`) is repeated by commit anyway, and by
  its own docstring it performs **no** conflict check (`artifact.rs:135`).
- **I4. Path taxonomy round-trips through raw `&str`/`Path`.** The validated
  newtype is discarded via `as_str()` → `Path::new` (`artifact.rs:152-153,192`)
  and re-validated by the writer (ADR-020 violation; double validation).
- **I5. Test modules ignore the canonical matrix.** `try_resolve_target` →
  `validation`, `commit` → `create` (`unit-naming.md`); the new `write_error`
  module (`fs/error.rs:647`) is flat where its neighbours use
  `formatting`/`conversions` submodules.
- **I6. Failure-path coverage gaps.** No test for dir-creation failure, the
  `commit` non-`AlreadyExists` Io branch, `InvalidPath` via `./x` current-dir, or
  a hidden-file target.

**Minor**

- **M1. `WriteError::Fs(#[from] FsError)` is unused** (constructed only in the
  test at `fs/error.rs:690`) — speculative public surface (YAGNI).
- **M2. `AbsolutePathRejected`/`TraversalRejected` sever the error source chain**
  (`error.rs:144,147`; `artifact.rs:227-232` discards the originating
  `PathError`).
- **M3. Act-phase `expect`** in `creates_file_with_content` /
  `writes_file_end_to_end` (`artifact.rs:466,534`) — documented anti-pattern.
- **M4. Non-end-to-end `pipeline` tests.** `pipeline::rejects_absolute_path` /
  `rejects_traversal_path` (`artifact.rs:546,561`) duplicate the single-step
  resolve tests and fail at the first transition.
- **M5. `accessors` module mislabeled** — holds an equality test and a private
  `state` field test (`artifact.rs:610`).
- **M6. Stale docs.** `writer.rs:5,19-20` and `fs/lib.rs:14,57` still describe
  only "atomic replace semantics", not `create_new`. Variant docs missing on
  `TemplateReadError::Read`/`TemplatePathError::Path`/`TemplateDirScanError::Scan`
  (`error.rs:94,108,168`).

### Remediation plan

Context: the project is deleting `PathValidator` in favour of a newtype design,
so path policy must move into the target newtype or it is lost. Approved design
decisions for this remediation:

- The FS write port is a trait named **`FileWriter`**.
- The validated write-target newtype is named **`WriteTarget`**, wraps
  `Path`/`PathBuf` (not `Box<str>`), and is **not** normalized — it preserves
  the caller's separators and works with the writer as-is.
- The pipeline collapses to **`Rendered → TargetResolved → Committed`**:
  drop the `ReadyToCommit` state and `try_check_conflict`; **fold parent-dir
  creation into `commit`**.

**A. `WriteTarget` owns all path policy** (fixes I2, I4, M-security; required
before `PathValidator` is deleted).
- Add `WriteTargetError` in `crates/fs/src/error.rs` as a standalone enum with `Empty`, `Absolute(PathBuf)`, `Traversal(PathBuf)`, `CurrentDir(PathBuf)`, and `Hidden(PathBuf)` variants. Keep this separate from `WriteError` (which is I/O only).
- Add `WriteTarget` in `crates/fs/src/path.rs` wrapping `PathBuf`, with
  `try_new(impl AsRef<Path>) -> Result<Self, WriteTargetError>` rejecting **absolute**, **`..` traversal**,
  **hidden components** (leading `.`), and **empty**. No normalization;
  `as_path() -> &Path`.

**B. `FileWriter` port that accepts `WriteTarget`** (fixes C2, I4, I2).
- Define `pub trait FileWriter { fn create_new(&self, target: &WriteTarget, contents: &[u8]) -> Result<(), WriteError>; }`
  in `crates/fs/src/writer.rs`.
- **Strip the `Writer` God Object**: Remove the embedded `Validator` (since `WriteTarget` now owns validation) and delete all speculative dead code (`atomic_write`, `rename`, `remove_file`).
- **Implement the port directly**: Implement `FileWriter` directly on the simplified `pub struct Writer { root: PathBuf }`, eliminating the need for a wrapper adapter.
- With validity guaranteed by `WriteTarget`, `Writer::create_new` stops
  validating; it can no longer emit a validation-as-`Io` error.

**C. Collapse the pipeline + ownership** (fixes C1, I1, I3).
- States: `Rendered → TargetResolved(WriteTarget) → Committed`. Remove
  `ReadyToCommit` and `try_check_conflict`.
- `try_resolve_target(self, &str) -> Result<TemplateArtifact<TargetResolved>, TemplateArtifactError>`
  constructs `WriteTarget::try_new` and maps its error (absolute →
  `AbsolutePathRejected`, traversal → `TraversalRejected`, catch-all incl.
  hidden/empty/current-dir → `InvalidPath`).
- `commit(self, writer: &impl FileWriter) -> Result<TemplateArtifact<Committed>, TemplateArtifactError>`
  calls `writer.create_new(target, content.as_bytes())`; parent-dir creation
  happens inside `create_new`.
- All transitions **consume `self`** (no clones). The artifact depends only on
  the `FileWriter` port, never on the concrete adapter.

**D. Error-model cleanup** (fixes M1, M2).
- Remove `WriteError::Fs(#[from] FsError)` and its test.
- Restore the source chain on `AbsolutePathRejected`/`TraversalRejected`
  (carry `#[source] WriteTargetError`).

**E. Tests** (fixes I5, I6, M3, M4, M5).
- Rename modules to canonical (`validation`, `create`; split `accessors` into
  `equality`/`state`); give `write_error` `formatting`/`conversions` submodules.
- Add failure-path tests: dir-creation failure, `commit` non-`AlreadyExists` Io,
  `InvalidPath` via `./x`, hidden-file target rejected at `try_resolve_target`.
- Remove the two non-end-to-end `pipeline::rejects_*` duplicates; fix Act-phase
  `expect`.
- Use `CreateFileWriter::new` in `artifact.rs` tests.

**F. Docs** (fixes M6).
- Update `writer.rs` / `fs/lib.rs` headers to mention `create_new` and `CreateFileWriter`; add variant
  docs on the template wrapper enums.

### Split: fix in 06 vs defer to 07

**Fix in this issue (06):** A, B, C (the typestate shape, `FileWriter`,
`WriteTarget`, consume-`self`, collapsed states), D, E, F. These are about the
quality and boundary of the code 06 ships and do not need the Service.

**Defer to issue 07 (see that issue's "Deferred from issue 06" section):** the
production caller that drives the pipeline, removing `#![allow(dead_code)]` from
`artifact.rs`, mapping `TemplateArtifactError` → `TemplateError`, and injecting
the `FileWriter` port + vault root into `TemplateService`.

> The litmus test: do **not** merge `Writer` flipped to `pub` for dead code. If
> the port/`WriteTarget` work is not done in 06, keep `Writer` `pub(crate)` and
> hold the pipeline's FS dependency until 07 builds the port.

### Implementation Notes
- **Interface Segregation**: By implementing the narrow `FileWriter` trait, the Template domain is protected from full filesystem capabilities, while avoiding the "cheap wrapper" anti-pattern.
- **YAGNI**: Speculative methods from `Writer` were removed rather than hidden. If features like atomic replacement are needed later, they should be implemented via trait ports rather than bloating a single `Writer` struct.
