# Merge Strategy: 03-candidate-selection-format-stability -> main

## Recommended Merge Sequence
1. **Switch to `main` branch** in the primary worktree.
2. **Merge `03-candidate-selection-format-stability`** into `main` using `--no-ff` to preserve the merge point.
   ```bash
   git checkout main
   git merge --no-ff 03-candidate-selection-format-stability
   ```
3. **Resolve conflicts**: None expected based on current analysis.
4. **Validation**: Run full test suite and quality gates.

## Preservation of Changes
- All commits from `03-candidate-selection-format-stability` will be incorporated.
- Changes in `main` (mostly `.scratch/` file updates) are disjoint from the worktree's code changes and will be preserved by the merge.

## Overlapping Edits & Conflicts
- No overlapping edits were found in code files.
- Potential conflict: None identified.

## Required Migrations / Manual Interventions
- No database or configuration schema migrations are required.
- No manual code interventions are expected.

## Validation Procedures
1. **Unit Tests**: `mise run test:unit` must pass (all 1463 tests).
2. **Quality Gates**: `mise run quality` (includes fmt, lint, and ADR validation) must pass.
3. **Functional Verification**: Verify `select_config_candidate` behavior manually if needed (already covered by extensive unit tests in `candidates.rs`).

## Rollback Procedures
In case of failure during or after merge:
1. **Abort Merge**: If conflicts are unresolvable: `git merge --abort`.
2. **Undo Merge**: If validation fails after merge: `git reset --hard HEAD~1` (assuming `main` was the HEAD before merge).
3. **Isolate**: Re-examine the divergence point and identify the cause of the failure.
