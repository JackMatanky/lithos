# Merge Strategy: issue-08/absolutepath-removal → main

## Overlap Summary

| Branch Pair | Overlapping Files | Conflict Risk |
|-------------|------------------|---------------|
| main ↔ issue-08 | **0** | None |
| main ↔ root-config-discovery | **0** | None |
| issue-08 ↔ root-config-discovery | **0** | None |

**Dry-run confirmed:** `Automatic merge went well` — zero conflicts.

## Merge Sequence

Since there is no inter-branch dependency or overlap, the merges are independent. Recommended order:

```
1. issue-08/absolutepath-removal → main   (current task)
2. root-config-discovery → main           (independent, not in scope)
```

## Merge Command

From the **main** worktree (`/Users/jack/Documents/41_personal/lithos`):

```bash
git merge --no-ff issue-08/absolutepath-removal -m "feat: merge issue-08 — remove AbsolutePath from codebase"
```

`--no-ff` is required because `main` has advanced past the merge base (3 additional commits). A fast-forward would be impossible anyway since `main` has diverged.

## Post-Merge Validation

After merge, run in the merged main worktree:

1. `mise run fmt` — formatting check
2. `mise run lint` — clippy clean
3. `mise run test:unit` — 1436 tests passing
4. `git status` — verify only merge commit + planning artifacts are uncommitted
5. `rg "AbsolutePath" --type rust src/` — confirm zero references

## Rollback Procedures

### Before committing the merge (if validation fails):
```bash
git merge --abort
```

### After committing the merge:
```bash
# Option A: Reset to pre-merge state
git reset --hard ORIG_HEAD

# Option B: Revert the merge commit
git revert -m 1 <merge-commit-sha>
```

## Required Migrations or Manual Interventions

**None.** All changes are purely additive or type-deletion refactors:
- Deleted: `AbsolutePath` struct, `AbsolutePathError` variant
- Changed: `TrustedVaultPath` inner type from `AbsolutePath` → `Box<str>`
- Migrated: 5 `AbsolutePathError` refs → `RestrictedPathError`
- Added: `TrustedVaultPath::to_dir_path()`, `TrustedVaultPath::as_str()`

No database migrations, no config format changes, no protocol changes.
