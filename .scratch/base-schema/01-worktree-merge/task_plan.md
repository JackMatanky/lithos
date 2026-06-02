# Task Plan - Worktree Merge: feat/base-schema-01-base-domain-and-deltas -> main

## Goal
Safely merge all changes from the dedicated worktree `.worktrees/base-schema-01-base-domain-and-deltas` (branch `feat/base-schema-01-base-domain-and-deltas`) back into the `main` branch, ensuring no regressions and resolving any conflicts.

## Phases
- [x] Phase 1: Analysis & Impact Assessment <!-- id: 0 -->
- [x] Phase 2: Merge Strategy Formulation <!-- id: 1 -->
- [/] Phase 3: Review & Approval <!-- id: 2 -->
- [ ] Phase 4: Execution <!-- id: 3 -->
- [ ] Phase 5: Verification & Cleanup <!-- id: 4 -->

## Phase 1: Analysis & Impact Assessment
- [x] Identify divergence point (merge-base) between `main` and `feat/base-schema-01-base-domain-and-deltas`.
- [x] List all files changed in `feat/base-schema-01-base-domain-and-deltas`.
- [x] List all files changed in `main` since divergence.
- [x] Use `gitnexus_detect_changes` on both branches to identify affected execution flows.
- [x] Identify overlapping file edits (physical conflicts).
- [x] Identify potential semantic conflicts (e.g., changes to shared traits/structs).

## Phase 2: Merge Strategy Formulation
- [x] Define the merge sequence (rebase vs merge).
- [x] Document required migrations or manual interventions.
- [x] Define validation procedures (unit tests, clippy, fmt).
- [x] Define rollback plan.

**Merge Strategy:**
- **Sequence**:
    1. Checkout `main` in the primary workspace.
    2. Merge branch `feat/base-schema-01-base-domain-and-deltas`.
    3. Since there are no conflicts, this will be a clean merge.
- **Manual Interventions**: None required.
- **Validation**:
    - `mise run verify` (full suite).
    - Manual inspection of `.scratch/base-schema/01-base-domain-and-deltas.md` to ensure frontmatter is correctly merged (though no conflict exists).
- **Rollback**: `git reset --hard ORIG_HEAD` in case of validation failure.

## Phase 3: Review & Approval
- [ ] Present findings and strategy to the user.
- [ ] Obtain explicit approval to proceed.

## Phase 4: Execution
- [ ] Stage and commit planning artifacts.
- [ ] Execute the approved merge strategy.

## Phase 5: Verification & Cleanup
- [ ] Run `mise run verify` in the merged state.
- [ ] Verify that the `closed` status of the issue in `.scratch` is preserved.
- [ ] Stage and commit merge-related changes.
- [ ] (Optional) Remove the worktree if requested.
