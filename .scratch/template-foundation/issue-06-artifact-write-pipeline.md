# Template Artifact Write Pipeline

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement `TemplateArtifact<State>` — a typestate write pipeline that enforces ordering of target resolution, conflict checking, and file commit using the hoverbear generic state machine pattern.

States and valid transitions:
- `TemplateArtifact<Rendered>` → `TemplateArtifact<TargetResolved>` via `From` impl (path validation: rejects absolute paths and traversal paths using `PathValidator`)
- `TemplateArtifact<TargetResolved>` → `TemplateArtifact<ReadyToCommit>` via `From` impl (conflict check)
- `TemplateArtifact<ReadyToCommit>` → `TemplateArtifact<Committed>` via `From`/commit method (file write)

The outer struct carries shared data (rendered content, template name, etc.); the `state: S` field carries per-state data. Invalid transitions (e.g. `Rendered → Committed` directly) are impossible by type construction — no runtime guard needed.

Commit behavior:
- Uses `File::create_new` (stable since Rust 1.77.0) to atomically create the file, failing with `AlreadyExists` if the destination exists. No separate existence pre-check — this eliminates the TOCTOU race.
- Writes to a vault-safe target path resolved from the vault root.
- Does not use raw `std::fs` — uses the FS context.

Path validation (in `Rendered → TargetResolved` transition):
- Rejects absolute paths
- Rejects paths containing `..` traversal components
- Wraps existing `PathValidator` logic

`TemplateArtifact<Committed>` is the terminal state; no further transitions are defined.

`TemplateArtifactSet<State>` for multi-file packs is explicitly out of scope.

## Acceptance criteria

- [ ] `TemplateArtifact<S>` is generic over a state type parameter, with shared fields in the outer struct and per-state data in `state: S`
- [ ] `From<TemplateArtifact<Rendered>> for TemplateArtifact<TargetResolved>` is implemented and performs path validation
- [ ] `From<TemplateArtifact<TargetResolved>> for TemplateArtifact<ReadyToCommit>` is implemented and performs conflict/existence check
- [ ] Commit from `ReadyToCommit` to `Committed` uses `File::create_new` (not a pre-check + create sequence)
- [ ] Absolute target paths are rejected at the `Rendered → TargetResolved` transition
- [ ] Traversal paths (`..` components) are rejected at the `Rendered → TargetResolved` transition
- [ ] No raw `std::fs` usage — all I/O goes through the FS context
- [ ] `TemplateArtifact<Committed>` is the terminal state
- [ ] `TemplateArtifactSet` is not implemented
- [ ] Tests cover: vault-relative target success (file created), absolute path rejection, traversal path rejection, `AlreadyExists` failure from `File::create_new`, single-file creation verified end-to-end; invalid transitions are impossible by type construction (no runtime test needed — compiler enforces this)

## Blocked by

- `issue-01-domain-models.md`
