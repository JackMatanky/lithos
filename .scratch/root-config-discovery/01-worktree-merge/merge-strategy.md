# Merge Strategy - Worktree Merge: root-config-discovery-01-discovery-type-contracts

## Recommended Merge Sequence
1.  **Sync planning artifacts**: Ensure this directory is committed to `main` (or a temporary branch) first to preserve the analysis. (Actually, I'll just merge the worktree branch which has the changes).
2.  **Execute Merge**:
    ```bash
    git checkout main
    git merge issue/root-config-discovery-01-discovery-type-contracts
    ```
3.  **Resolve Conflicts**:
    - Conflict expected: `lithos-core/src/config/discovery.rs` vs `lithos-core/src/config/discovery/mod.rs`.
    - Resolution: `git rm lithos-core/src/config/discovery.rs` (it's now in `mod.rs`).
    - Keep all new files in `lithos-core/src/config/discovery/`.
4.  **Validation**:
    - `mise run verify` (runs fmt, lint, tests, ADR validation).
    - `mise run test:unit` specifically for `lithos-core`.
5.  **Finalize**:
    - Commit merge resolution.
    - Delete worktree: `git worktree remove .worktrees/root-config-discovery-01-discovery-type-contracts`.
    - Delete branch: `git branch -d issue/root-config-discovery-01-discovery-type-contracts`.

## Manual Interventions
- **File removal**: Manual `git rm lithos-core/src/config/discovery.rs` if git doesn't handle the rename/move cleanly.

## Validation Procedure
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo fmt --check`

## Rollback Procedure
1.  `git merge --abort` (if in progress).
2.  `git reset --hard HEAD~1` (if merge completed but failed validation).
