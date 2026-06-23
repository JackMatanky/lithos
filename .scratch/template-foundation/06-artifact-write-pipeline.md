---
title: 06-artifact-write-pipeline
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-11
date_completed:
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

- [ ] `TemplateArtifact<S>` is generic over a state type parameter, with shared fields in the outer struct and per-state data in `state: S`
- [ ] `try_resolve_target` converts `TemplateArtifact<Rendered>` → `Result<TemplateArtifact<TargetResolved>, TemplateArtifactError>`; path validation uses `RelativeFilePath::try_new` (no separate `PathValidator` call)
- [ ] `try_check_conflict` converts `TemplateArtifact<TargetResolved>` → `Result<TemplateArtifact<ReadyToCommit>, TemplateArtifactError>`; creates parent directories (no existence check — `File::create_new` is the atomic guard, eliminating TOCTOU)
- [ ] Commit from `ReadyToCommit` to `Committed` uses `File::create_new` (not a pre-check + create sequence)
- [ ] Absolute target paths are rejected at the `Rendered → TargetResolved` transition
- [ ] Traversal paths (`..` components) are rejected at the `Rendered → TargetResolved` transition
- [ ] No raw `std::fs` usage — all I/O goes through the FS context
- [ ] `TemplateArtifact<Committed>` is the terminal state
- [ ] `TemplateArtifactSet` is not implemented
- [ ] Tests cover: vault-relative target success (file created), absolute path rejection, traversal path rejection, `AlreadyExists` failure from `File::create_new`, single-file creation verified end-to-end; invalid transitions are impossible by type construction (no runtime test needed — compiler enforces this)

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
