# Findings - Worktree vs Main (Issue 06 Merge Planning)

## Worktree Safety Verification

- `git rev-parse --git-dir` => `.git/worktrees/has-hash-index-traits`
- `git rev-parse --git-common-dir` => `.git`
- `pwd` => `.worktrees/feat/has-hash-index-traits`
- Result: operating in dedicated linked worktree, not main checkout.

## Source-of-Truth Issue State

- Issue: `.scratch/internal-hash-support/06-add-has-hash-index-traits.md`
- Status updated to `completed` with implementation notes and dead-code lint nuance.
- Latest issue-doc commit: `4613ab67`.

## Divergence Analysis

- Merge base between branch and main: `f1dbbdb0fa1ef964399e6a6b2818d8b9f89124c3`
- Branch has 4 commits after base; main has 9 commits after base.

### Branch-side thematic changes

- Support traits (`HasHashIndex`, `HasHashIndexMut`) and re-exports
- Wrapper impls in schema/config (`RawPropertyHashIndex`, `HashRecord`, `ConfigFieldHashes`)
- Issue documentation updates

### Main-side thematic changes

- Config builder/discovery refactor thread
- FS-related updates
- Planning/scratch documentation expansion
- AGENTS GitNexus metadata path/table adjustments

## File Overlap / Conflict Surface

- Exact file overlap since divergence: **none**.
- Expected textual conflicts during merge: **low probability**.
- Expected semantic integration risk: **low-to-moderate**, concentrated in config context due adjacent (not same-file) refactors.

## GitNexus + Rust Best Practices Lens

### GitNexus observations

- `gitnexus_query` confirms main-side activity centered in config builder/discovery flows.
- Branch-side query confirms touched symbols are in support/schema/config hash-index path.
- `gitnexus_detect_changes(compare)` did not reflect expected divergence; treated as tool limitation for this compare mode and not used as authoritative merge-set source.

### Rust best-practices implications for merge

- Preserve trait API shape consistency and avoid accidental broadening of visibility.
- Keep generic/associated-type contracts untouched during conflict resolution.
- Re-run strict clippy gate post-merge to catch dead code and trait-usage regressions.
- Ensure no unnecessary clones/allocations introduced while manually resolving conflicts.

## Special Finding: Local AGENTS.md Drift

- Unstaged `AGENTS.md` modifications are present in worktree (index stats/path table refresh side-effects).
- Main also changed `AGENTS.md` since divergence.
- Must decide at merge-execution time:
  - **Option A (recommended):** discard local unstaged drift before merge, then adopt merged/desired AGENTS state intentionally.
  - **Option B:** keep and include as explicit merge resolution change (higher noise risk).

## Recommended Strategy Summary

1. Clean local unstaged drift decision first.
2. Merge `main` into feature worktree branch.
3. Resolve any conflicts preserving both histories and functional behavior.
4. Validate with fmt/clippy/tests.
5. Commit merge result.
