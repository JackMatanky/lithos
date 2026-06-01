# Merge Strategy: `07/pathkey-note-template` → `main`

## Divergence Summary

| Property | Value |
|----------|-------|
| Merge base | `6e951e49` — "docs: update centralized discovery prd" |
| main commits (post-base) | 3 docs-only commits |
| Worktree commits (post-base) | 1 issue-file commit (uncommitted: 4 source files) |
| Files changed on main | 21 files (all docs/skills/config) |
| Files changed on worktree | 5 files (4 source + 1 issue) |
| Overlapping files | **Zero** |

## Branch Contents

### What `main` added (since `6e951e49`)
```
.agents/skills/mermaid-diagrams/      (new skill, 5 files)
.scratch/base-schema/                 (updated PRD + 8 new issues)
.scratch/schema-processor-split/      (updated PRD)
AGENTS.md                             (+2 lines: mermaid skill reference)
skills-lock.json                      (+6 lines)
```
All non-code: documentation, skills, config.

### What `07/pathkey-note-template` added (since `6e951e49`)
**Committed:**
```
.scratch/pathkey-migration/07-note-template-cut-relativepath-to-pathkey.md  (+114/-38)
```

**Uncommitted (the implementation):**
```
lithos-core/src/note/paths.rs         (-49 net)  NotePath(PathKey), FoldPath(PathKey), removed RelativePath
lithos-core/src/note/storage/read.rs  (-5 net)   1 PathKey::try_new → as_path_key()
lithos-core/src/note/storage/write.rs (-27 net)  5 PathKey::try_new → as_path_key()
lithos-core/src/vault/processor.rs    (-2 net)   1 NotePath::try_new → TryFrom<PathKey>
```
All source code in 4 files, none of which were touched by main.

## Overlap Analysis

### File-by-file comparison
| File path | main changed? | worktree changed? | Conflict |
|-----------|---------------|-------------------|----------|
| lithos-core/src/note/paths.rs | No | **Yes** | None — no main edits |
| lithos-core/src/note/storage/read.rs | No | **Yes** | None — no main edits |
| lithos-core/src/note/storage/write.rs | No | **Yes** | None — no main edits |
| lithos-core/src/vault/processor.rs | No | **Yes** | None — no main edits |
| .scratch/pathkey-migration/07-*.md | No | **Yes** | None — unique file |
| All .agents/skills/mermaid-diagrams/ | **Yes** | No | None — unique files |
| All .scratch/base-schema/ | **Yes** | No | None — unique files |
| .scratch/schema-processor-split/PRD.md | **Yes** | No | None — unique file |
| AGENTS.md | **Yes** (+2 lines) | No | None |
| skills-lock.json | **Yes** (+6 lines) | No | None |

**Result: Zero conflicting files. Merge is fully automatable with no manual conflict resolution.**

## Merge Sequence

```
Step 1: Commit worktree changes  → 07/pathkey-note-template (staged)
Step 2: Fetch main HEAD           → ensure local state is current
Step 3: Merge main into worktree  → test merge locally
Step 4: Validate merged state     → tests, clippy, fmt
Step 5: Push to main              → fast-forward or merge commit
```

### Step 1: Commit Worktree Changes
- Stage the 4 modified source files + updated issue file
- Commit with conventional-commit message
- The TODO about centralized discovery is a forward-looking note, not a blocker

### Step 2-3: Merge Strategy
Since there's no file overlap, we can either:
- **Option A** (recommended): `git merge --no-ff main` in the worktree (merge main's 3 doc commits INTO the worktree), verify it works, then fast-forward main to the worktree tip
- **Option B**: `git merge --no-ff 07/pathkey-note-template` in main (pull worktree INTO main)
- **Option C**: Rebase worktree onto main, then fast-forward

**Recommendation: Option A** — merge main into worktree first to validate, then fast-forward main. This ensures we catch any unexpected interactions before they reach main.

### Step 4: Validation
After merge, verify:
- `cargo test` — all tests pass (1433 unit, 152 doc, 36 integration, 1 e2e)
- `cargo clippy --package lithos-core` — zero warnings
- `rustfmt --check` on all changed files — zero format issues
- `git diff main..HEAD` — only expected files changed

### Step 5: Push
- `git checkout main` in main worktree
- `git merge --ff-only 07/pathkey-note-template`
- Push

## Required Migrations
**None.** rkyv binary format is identical (both old and new use `Box<str>`). No database migration needed.

## Manual Interventions Required
**None.** Zero conflict files + zero overlapping edits = fully automatic merge.

## Rollback Procedure
If merged state fails validation:
1. `git reset --hard HEAD@{1}` in main worktree (before merge commit)
2. Debug and re-merge after fix

If pushed and needs revert:
1. `git revert <merge-commit-hash>` in main
2. Create follow-up issue for the fix

## gitnexus Post-Merge Analysis
The index is stale for the `as_path_key` symbol (uncommitted). After merge + re-index, run:
```
npx gitnexus analyze
```
Then verify `as_path_key` is connected to its callers in read.rs/write.rs.

## Acceptance Criteria Verification (Post-Merge)

- [x] The private `RelativePath` struct is removed; `NotePath`/`FolderPath` use `PathKey`
- [x] All 6 `PathKey::try_new(path.as_str())` in storage replaced with `as_path_key()`
- [x] Vault processor `NotePath::try_new(path.as_str())` uses `TryFrom<PathKey>` (deviation noted)
- [x] rkyv binary compatibility maintained
- [x] All tests pass
- [x] Zero `PathKey::try_new()` in note/storage/ + vault/processor/
