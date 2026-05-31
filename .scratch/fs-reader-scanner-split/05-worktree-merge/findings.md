# Findings: Worktree Merge Analysis

## Divergence Point
- Merge base: `8bb70a2ff254931468d80a8d840349aa7dbce60d` (`chore(merge): bring has-hash-index work into main`)

## Changes on `main`
- None. `main` is at the merge base.

## Changes on `fs-reader-scanner-split-05`
- **Committed**: `1bdaffc4` Docs update for TDD plan and review findings.
- **Uncommitted**: Implementation of the GREEN step for NoteProcessor staleness checks.
  - Modifies `Note` aggregate to hold `FileMetadata`.
  - Updates `NoteFileInfo` and `NoteProcessor` to use `FileMetadata`.
  - Fixes `check_metadata` to perform pure data comparisons.
  - Updates `VaultProcessor` to pass the `FileMetadata`.
  - Fixes tests and benchmarks to accommodate the new `Note` aggregate signature.

## Overlapping Edits and Merge Conflicts
- None. Since `main` has not advanced, the merge will be a fast-forward or a clean merge with no conflicts.

## Migrations and Manual Interventions
- The `Note` aggregate storage format has changed (added `FileMetadata`). Since Lithos uses the database as a cache (`NOTES` table), existing cached notes in the `main` environment may need to be invalidated or re-indexed. The `rkyv` serialization structure changed.
