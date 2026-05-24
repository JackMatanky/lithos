# Task Plan - Merge `feat/db-testing-seam` back into `main`

## Goal

Merge the feature worktree branch back into the main worktree branch without
losing any work already completed on `main` since the worktrees diverged.

## Current Divergence Snapshot

- Merge base: `be4493f6`
- Feature-only commits: schema/db-testing seam + Issue 06 docs and test hardening
- Main-only commits: fs/path refactors + PRD/ADR updates and doc corrections
- Primary risk: same markdown issue files changed in both branches

## Constraints

- No destructive git operations (`reset --hard`, history rewrite, force push).
- Preserve both streams of work.
- Keep Rust quality gates green after merge (`fmt`, `lint`, `test`).
- Resolve conflicts manually with semantic review, not blind conflict marker removal.

## Merge Strategy

### Phase 1 - Pre-merge safety checks

- [ ] Ensure both worktrees are clean (`git status`).
- [ ] Tag/save safety points:
  - [ ] `git branch backup/main-before-merge main`
  - [ ] `git branch backup/feat-db-testing-seam-before-merge feat/db-testing-seam`
- [ ] Record comparison artifacts:
  - [ ] `git log --left-right --cherry-pick --oneline main...feat/db-testing-seam`
  - [ ] `git diff --name-status main...feat/db-testing-seam`

### Phase 2 - Integrate feature into main worktree

- [ ] In main worktree (`/Users/jack/Documents/41_personal/lithos`), update refs:
  - [ ] `git fetch --all --prune`
  - [ ] `git switch main`
- [ ] Start non-fast-forward merge for traceability:
  - [ ] `git merge --no-ff feat/db-testing-seam`
- [ ] If conflicts occur, stop and resolve file-by-file.

### Phase 3 - Conflict resolution protocol (lossless)

- [ ] For each conflicted file:
  - [ ] Inspect both sides: `git checkout --ours <file>` and `git checkout --theirs <file>` into temp views if needed.
  - [ ] Reconstruct combined intent manually.
  - [ ] Re-open acceptance criteria files in `.scratch/db-refactor/*` and ensure both:
    - [ ] main branch PRD/doc fixes remain
    - [ ] feature branch Issue 06/07/08/09 criteria updates remain
- [ ] Rust code files: preserve behavior and avoid regressions.
- [ ] Re-stage only resolved files.

### Phase 4 - Verify merged state

- [ ] Run quick diff sanity:
  - [ ] `git diff --check`
  - [ ] `git diff --stat HEAD~1..HEAD` (or staged equivalent before commit)
- [ ] Run project quality tasks (mise-first):
  - [ ] `mise run fmt`
  - [ ] `mise run lint`
  - [ ] `mise run test`
- [ ] If failures appear, fix in follow-up commits (no amend requirement unless requested).

### Phase 5 - Finalize and clean up

- [ ] Commit merge with explicit message if conflict resolution changed content.
- [ ] Validate no work loss:
  - [ ] `git log --oneline --decorate -20`
  - [ ] check key feature commits still reachable
  - [ ] check main-only commits still reachable
- [ ] Optional cleanup after confirmation:
  - [ ] remove backup branches
  - [ ] remove feature worktree/branch only when user confirms

## Rust Best Practices Guardrails During Merge

- Prefer minimal semantic edits while resolving conflicts in Rust code.
- Keep error propagation idiomatic (`?`) and avoid reintroducing unwrap/panic.
- Avoid unnecessary cloning introduced by manual conflict merges.
- Keep lint suppressions narrow and justified.
- Maintain test naming/structure conventions already applied in schema storage tests.

## Conflict Hotspots (Expected)

- `.scratch/db-refactor/06-db-testing-seam-and-in-memory-alignment.md`
- `.scratch/db-refactor/07-note-storage-migration-and-testing-repo-update.md`
- `.scratch/db-refactor/08-template-storage-migration-and-testing-repo-update.md`
- `.scratch/db-refactor/09-config-storage-migration-and-testing-repo-update.md`

## Errors Encountered

| Error | Attempt | Resolution |
|---|---:|---|
| None yet | - | - |
