# Task Plan: Worktree Merge - pathkey-09-relativepath

## Goal
Safely merge the `pathkey-09-relativepath` worktree back into the `main` branch, ensuring all changes are preserved and the resulting codebase is stable.

## Analysis
- **Divergence Point**: `1e0d4e68`
- **Worktree Head**: `f4971efc`
- **Main Head**: `63d6c7d9`
- **Overlapping Files**: None detected in source code. Some overlap in `.scratch/` (if any, will be checked).
- **Semantic Conflicts**: Low risk. `lithos-core/src/config/discovery/` does not appear to use the modified path types yet.

## Phases

### 1. Planning & Analysis [in_progress]
- [x] Identify divergence point and heads.
- [x] Check for file-level overlaps.
- [x] Check for semantic conflicts in new code on `main`.
- [ ] Create detailed findings and merge strategy.

### 2. Preparation [pending]
- [ ] Ensure both worktrees are clean (already verified).
- [ ] Synchronize `main` with `origin/main` (if applicable, but `main` is ahead).

### 3. Execution [pending]
- [ ] Perform merge of `pathkey-09-relativepath` into `main`.
- [ ] Resolve any unexpected conflicts.
- [ ] Run verification suite (`mise run verify`).

### 4. Validation & Finalization [pending]
- [ ] Confirm all tests pass.
- [ ] Check clippy and fmt.
- [ ] Finalize issue tracker state.

## Decisions
- Merge `pathkey-09-relativepath` INTO `main`.
- Use the main worktree for the merge operation to ensure a clean environment.

## Errors Encountered
(None)
