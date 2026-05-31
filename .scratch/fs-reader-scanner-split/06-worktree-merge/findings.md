## Worktree and Divergence Facts

- Current branch: `refactor/fsreader-purge-methods`
- Current HEAD: `f58706ace8ed321a447c72fdccdfb6ef3fd9f5c4`
- Main HEAD: `8830ce961e5d6da2e40c00304c3f08abfe2a6f81`
- Merge-base: `8830ce961e5d6da2e40c00304c3f08abfe2a6f81`

Interpretation:

- `main` has **0 commits** since divergence.
- Feature branch has **4 commits** since divergence:
  - `37e97e8f9` docs: update agent brief and tdd plan for 06-rename-fsreader-purge-methods
  - `483266f60` docs: correct 06 issue wording and approved findings
  - `1902b271a` refactor(fs): rename FsReader and purge traversal APIs
  - `f58706ace` docs(fs): rename FsReader references to FileReader

## File-Level Overlap Analysis

- Files changed on feature branch since merge-base: **78**
- Files changed on `main` since merge-base: **0**
- Overlapping changed paths between `main` and feature: **0**

Current conflict outlook (at analysis time):

- Textual merge conflicts: **none expected** (fast-forward eligible if `main` unchanged).
- Semantic conflicts: low, because `main` has no divergent edits.

## GitNexus + Rust-Oriented Impact Findings

GitNexus observations:

- Query around `FileReader` and dependent flows shows usage concentrated in:
  - `lithos-core/src/config/builder.rs`
  - `lithos-core/src/schema/property_bank_processor.rs`
  - `lithos-core/src/fs/reader.rs`
- Affected process families include config and schema loading flows where `read_to_string`/structured parsing is consumed.

Rust best-practices implications for merge safety:

- API narrowing (removing traversal + metadata methods) should be guarded by full compile + tests + doctests.
- Doc comments/examples must match public API (already observed as a practical failure mode and fixed in this branch).
- Keep non-panicking error flow and avoid reintroducing removed legacy methods through conflict resolution.

## Required Migrations / Manual Interventions

Even with no current overlap, execute these checks as mandatory safeguards:

1. **Pre-merge freshness check**
   - Re-check merge-base right before merge; if `main` moved, recompute overlap and conflict map.
2. **If `main` advanced and conflicts appear**
   - Prioritize preserving both sides by resolving symbol/API changes to `FileReader` naming and narrowed method surface.
   - Do not resurrect removed methods:
     - `filter_entries`, `filter_file_entries`, `filter_dir_entries`
     - `filter_paths`, `filter_file_paths`, `filter_dir_paths`
     - `std_metadata`, `metadata`, `created_at`, `modified_at`
   - Keep `exists(&self, path: &Path)` intact.
3. **Doctest/doc synchronization**
   - Resolve any stale docs to `FileReader` API shape.

## Recommended Merge Sequence

1. Verify working directory and branch safety (must be dedicated worktree, clean index).
2. Fetch and re-evaluate divergence (`main` may have moved).
3. If no new `main` commits:
   - Merge via fast-forward from `main` to feature tip (or merge feature into `main` with FF).
4. If `main` has new commits:
   - Perform non-FF merge in integration branch.
   - Resolve conflicts preserving both sides; prefer feature’s FS API narrowing while manually integrating `main` logic changes.
5. Run validation gates.
6. Commit merge result.

## Validation Procedure

Minimum required:

- `mise run verify`
- `cargo test -p lithos-core --doc`
- `gitnexus_detect_changes(scope="all")` to inspect affected flows after merge

Target outcomes:

- No compile, clippy, test, or doctest regressions.
- No references to removed `FileReader` methods.
- Merge commit/tree captures both sides’ intent since divergence.

## Rollback Procedure

If merge result fails validation:

1. Capture failing command and error context in `progress.md`.
2. Abort merge if still in-progress (`git merge --abort`).
3. If merge commit already created locally but not pushed:
   - `git reset --hard HEAD~1` only after explicit user approval (destructive).
   - Preferred safer path: create a revert commit instead of history rewrite.
4. Re-open plan, add required manual resolutions, and retry.

## Approval Gate

This plan is ready for approval. No merge actions should be executed before explicit user approval.
