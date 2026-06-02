# Merge Strategy: Worktree 05 to Main

## 1. Overview
Merge the `05-move-discovery-module-boundary` worktree into `main`, incorporating logical updates for the `Discovery -> Config -> Indexer` boundary and `rust-best-practices`.

## 2. Recommended Sequence
1.  **Worktree Sync**: Merge `main` into `05-move-discovery-module-boundary` within the dedicated worktree.
2.  **Conflict Resolution**:
    - **`discovery/mod.rs`**: Prefer worktree's full boundary documentation.
    - **`config/location.rs`**: Prefer worktree's `#[expect]` and `#[allow]` attributes with specific reason strings.
    - **`config/mod.rs`**: Prefer worktree's documentation distinguishing local `discovery` from `crate::discovery`.
3.  **Validation**:
    - `mise run fmt`
    - `mise run lint`
    - `mise run test`
4.  **Integration Merge**: Merge `05-move-discovery-module-boundary` back into `main`.

## 3. Required Migrations / Manual Interventions
- No database migrations required.
- **Logical migration**: `schema/discovery.rs` and `config/discovery.rs` in `main` continue to follow the old pattern; they are NOT yet refactored to use `discovery::resolver::RootResolver`. This is deferred to Phase 2 (Issue 06/07) to maintain vertical slice scope.

## 4. Validation Procedures
- **Compilation**: `cargo check --all-targets`
- **Quality**: `mise run quality` (fmt + lint + adr:validate)
- **Tests**: `mise run test` (unit + integration + e2e)

## 5. Rollback Procedures
- **Merge failure**: `git merge --abort`
- **Post-merge failure**: `git reset --hard HEAD~1` on `main` (if not pushed) or revert commit.
