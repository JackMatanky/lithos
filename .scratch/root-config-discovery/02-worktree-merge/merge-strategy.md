# Merge Strategy - 02-local-candidate-generation → main

**Date:** 2026-06-01
**Branch:** `root-config-discovery/02-local-candidate-generation`
**Target:** `main`
**Common Ancestor:** `1ec3c6d5`

---

## Summary

`main` has received **zero commits** since the worktree was created. A **fast-forward merge** is the
correct strategy. There are no merge conflicts, no overlapping edits, and no manual interventions required.

---

## Pre-Merge Checklist

- [x] Divergence point confirmed: `1ec3c6d5`
- [x] `main` commits since divergence: 0
- [x] Worktree commits since divergence: 4
- [x] Dry-run merge result: clean (exit 0)
- [x] All tests pass in worktree: 1447 unit + 36 integration
- [x] `mise run quality` clean in worktree
- [x] No conflicts in any changed file

---

## Merge Sequence

**Step 1 — Commit planning artifacts (in worktree)**
```
git add .scratch/root-config-discovery/02-worktree-merge/
git commit -m "docs(scratch): add worktree merge analysis artifacts for 02-local-candidate-generation"
```

**Step 2 — Switch to `main` and fast-forward merge**
```bash
# In the main worktree root
git merge --ff-only root-config-discovery/02-local-candidate-generation
```

Fast-forward is safe and preferred here because:
- `main` is a direct ancestor of the worktree branch.
- No divergent commits exist on either side.
- Keeps the commit history linear.

**Step 3 — Update GitNexus index**
```bash
npx gitnexus analyze
```
Required because `AGENTS.md` contains stale stat counts. Running analysis regenerates accurate numbers
and should be committed as a follow-up.

**Step 4 — Run full verification suite on `main`**
```bash
mise run verify
```

**Step 5 — Commit AGENTS.md stat update (if gitnexus generates one)**
```bash
git add AGENTS.md
git commit -m "chore(agents): update gitnexus index stats after merge"
```

---

## Changed Files Summary

| File | Change | Conflict Risk |
|------|--------|---------------|
| `lithos-core/src/config/discovery/candidates.rs` | **Added** — new Phase-2 discovery seam | None |
| `lithos-core/src/config/discovery/mod.rs` | `pub(crate) mod candidates;` added | None |
| `.scratch/root-config-discovery/02-local-candidate-generation.md` | Agent brief + TDD plan + criteria marked complete | None |
| `AGENTS.md` | GitNexus stat count updated | None (cosmetic) |

---

## Manual Interventions

None required. All changes are additive or cosmetic.

---

## Validation Procedure

After `git merge --ff-only`:

1. **Confirm HEAD matches worktree tip:**
   ```bash
   git log --oneline -5
   # Should show cfdd01bc as HEAD
   ```
2. **Confirm new file exists:**
   ```bash
   ls lithos-core/src/config/discovery/candidates.rs
   ```
3. **Confirm mod.rs registers the new module:**
   ```bash
   grep "candidates" lithos-core/src/config/discovery/mod.rs
   ```
4. **Run full test suite:**
   ```bash
   mise run test
   ```
5. **Run full quality gate:**
   ```bash
   mise run quality
   ```

---

## Rollback Procedure

If verification fails after merge:

```bash
# Hard-reset main to the pre-merge tip
git reset --hard 1ec3c6d58280603f842997c3d0ffbfe7d207aae8
```

The worktree branch remains intact and is not deleted by the merge. It can be re-merged at any time
after resolving any issues found during verification.

---

## Post-Merge Cleanup (Optional)

Once `main` is verified and the worktree is no longer needed:

```bash
# Remove the worktree and delete the branch
git worktree remove .worktrees/root-config-discovery/02-local-candidate-generation
git branch -d root-config-discovery/02-local-candidate-generation
```
