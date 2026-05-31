# Worktree Merge Plan - Issue 06

## Goal

Merge `feat/has-hash-index-traits` into `main` while preserving all post-divergence changes on both sides, identifying overlap/conflicts early, and validating safe integration with rollback options.

## Scope

- Compare changes from divergence base to `HEAD` (worktree branch) and to `main`.
- Build a merge sequence with explicit conflict handling.
- Define validation and rollback procedures.
- Planning only in this phase (no merge execution yet).

## Phase Status

| Phase | Description | Status |
|---|---|---|
| 1 | Confirm isolated worktree and source-of-truth issue state | complete |
| 2 | Compute divergence and changed-file sets | complete |
| 3 | Identify overlap/conflict surfaces + required interventions | complete |
| 4 | Build recommended merge sequence | complete |
| 5 | Define validation + rollback procedures | complete |
| 6 | Present approval package (no merge yet) | in_progress |

## Divergence Snapshot

- Merge base: `f1dbbdb0fa1ef964399e6a6b2818d8b9f89124c3`
- Branch-only commits (`base..HEAD`):
  - `4613ab67` docs(scratch): record implementation notes for issue 06
  - `6bf78550` feat(core): implement HasHashIndex traits on wrapper types
  - `b7f7847f` feat(support): define HasHashIndex and HasHashIndexMut traits
  - `c5db922c` docs(scratch): add approved design decisions and agent brief for issue 06
- Main-only commits (`base..main`):
  - `c928e542` refactor(config): merge configbuilder metadata threading
  - `1b4ac5bd` chore(config): finalize merge planning and validation for 04-refactor-configbuilder-metadata
  - `7800255b` chore(config): merge main into refactor branch
  - `82ff6e88` docs(scratch): add worktree merge planning artifacts for 04-refactor-configbuilder-metadata
  - `3ff0d12b` docs(scratch): mark 04-refactor-configbuilder-metadata as complete with updated plan
  - `ed338c5d` refactor(config): decouple traversal from IO and thread discovery metadata
  - `a11bd954` docs(scratch): split event foundation into issues
  - `9b85ef33` docs: tighten event sourcing foundation PRD
  - `619227b7` docs(scratch): update refactor plan for configbuilder metadata threading

## Changed Files Since Divergence

- Branch side:
  - `.scratch/internal-hash-support/06-add-has-hash-index-traits.md`
  - `lithos-core/src/config/processor.rs`
  - `lithos-core/src/schema/views/hashes.rs`
  - `lithos-core/src/support/hash_index.rs`
  - `lithos-core/src/support/mod.rs`
- Main side:
  - `.scratch/event-sourcing-foundation/*` (PRD + issue docs)
  - `.scratch/fs-inode-architecture/17-*`
  - `.scratch/fs-reader-scanner-split/04-*`
  - `AGENTS.md`
  - `lithos-core/src/config/builder.rs`
  - `lithos-core/src/config/discovery.rs`
  - `lithos-core/src/fs/format.rs`
  - `lithos-core/src/fs/mod.rs`
  - `lithos-core/src/schema/discovery.rs`
- Overlap (exact same path modified on both sides): none

## Risks / Interventions

1. **Unstaged local drift in worktree `AGENTS.md`**
   - Current worktree has unstaged `AGENTS.md` edits from GitNexus analyze refresh metadata.
   - Main also changed `AGENTS.md` since divergence.
   - Intervention: before merge execution, decide whether to discard local unstaged `AGENTS.md` drift or intentionally keep and commit it as part of merge changes.

2. **Config area adjacency risk (semantic, not file overlap)**
   - Branch modifies `config/processor.rs`; main modifies `config/builder.rs` + `config/discovery.rs`.
   - Low textual conflict risk, moderate integration risk in config workflows.

3. **GitNexus detect_changes compare limitation**
   - `gitnexus_detect_changes(scope:"compare", base_ref:"main")` returned no changes despite known divergence.
   - Use git-native diff/log as source of truth for merge set.

## Proposed Merge Sequence

1. Pre-merge hygiene
   - Confirm still in dedicated worktree.
   - Resolve `AGENTS.md` unstaged drift decision.
   - Ensure branch is clean before merge attempt.
2. Sync and merge
   - Fetch latest refs.
   - Merge `main` into `feat/has-hash-index-traits` (not rebasing to preserve both histories clearly).
3. Conflict handling (if any)
   - Resolve conflicts file-by-file with preservation policy: retain both sides unless redundant.
   - For docs/scratch artifacts, preserve both issue histories.
4. Validation
   - Run format/lint/tests per repo standards.
   - Re-run targeted checks for touched config/hash support surfaces.
5. Finalize
   - Commit merge resolution changes with clear message.
   - Summarize resulting commit graph and changed files.

## Validation Procedure

Primary:
- `cargo fmt`
- `cargo clippy -p lithos-core --all-targets -- -D warnings`
- `cargo test -p lithos-core`

Recommended additional confidence checks:
- Targeted tests in hash and config areas if failures/flake appear.
- `git diff --name-status main...HEAD` to confirm intended merge surface.

## Rollback Procedure

- If merge not committed: `git merge --abort`
- If merge committed but not pushed:
  - Identify pre-merge commit SHA
  - `git reset --hard <pre-merge-sha>` (only with explicit approval at execution time)
- If conflicts become non-trivial or requirements diverge:
  - Stop
  - Update issue/planning artifacts
  - Request decision before continuing

## Notes

- This is a planning-only artifact set. No merge execution occurs until explicit approval.
