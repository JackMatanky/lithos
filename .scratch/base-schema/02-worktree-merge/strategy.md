# Merge Strategy: feature/02-base-repository-contracts-and-storage → main

## Chosen Strategy

**Standard `git merge` (fast-forward-able, 3-way merge)**

Rationale: zero overlapping files, no schema changes on main since divergence, and no API surface crossing config ↔ schema.

## Pre-merge Steps

1. [x] Verify worktree: `.worktrees/feature/02-base-repository-contracts-and-storage`
2. [x] Confirm divergence point: `3d255769`
3. [x] Confirm no file overlap between main and feature changes
4. [x] Confirm all quality gates pass on feature branch
5. [ ] Push feature branch to origin

## Merge Sequence

### Phase 1 — Sync feature with main (optional but recommended)

```bash
# From feature worktree:
git fetch origin
git merge origin/main
# Resolve conflicts (expect 0)
mise run verify
```

### Phase 2 — Merge into main

```bash
# From main worktree:
git fetch origin
git merge origin/feature/02-base-repository-contracts-and-storage
```

### Phase 3 — Validate

```bash
mise run verify
```

### Phase 4 — Push

```bash
git push origin main
```

## Post-merge Actions

- [ ] Update issue file status to reflect merged state
- [ ] Delete feature branch (local + remote)
- [ ] Remove worktree: `git worktree remove .worktrees/feature/02-base-repository-contracts-and-storage`

## Rollback

### Before push (local only):
```bash
git reset --hard ORIG_HEAD
```

### After push:
```bash
git revert -m 1 HEAD
git push origin main
```
