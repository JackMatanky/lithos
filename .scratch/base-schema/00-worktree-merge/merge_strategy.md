# Merge Strategy: chore/base-schema-phase0-prereq-checkpoint -> main

## Recommended Merge Sequence
1. **Fetch and Sync**: Ensure `main` and `chore/base-schema-phase0-prereq-checkpoint` are both up to date.
2. **Merge into Branch**: Merge `main` into `chore/base-schema-phase0-prereq-checkpoint` first.
   - Resolve `AGENTS.md` conflict by combining both GitNexus stats updates or accepting `main`'s latest and re-running analysis.
3. **Verify in Worktree**: Run full verification suite on the merged branch.
4. **Fast-forward Main**: Once verified, merge the branch into `main` (should be fast-forward).

## Manual Interventions
- **`AGENTS.md` Conflict Resolution**:
  - The conflict occurs in the "This project is indexed by GitNexus as **lithos**..." line.
  - **Resolution**: Use `main`'s version of the stats line as it is more recent, then run `npx gitnexus analyze` to update to the latest combined state.

## Required Migrations
- **Code**: Consumers of `RawSchema.extends` or `SchemaVersion.extends` must now handle `Vec<SchemaName>` instead of `Option<SchemaName>`.
  - All known consumers in `lithos-core` have been updated in this branch.
- **Data**: No persistent data migrations required; schema files on disk remain compatible (YAML sequence of 1 is compatible with previous optional single).

## Validation Plan
1. **Compilation**: `mise run build`
2. **Unit Tests**: `mise run test:unit`
3. **Full Quality Gate**: `mise run verify`
4. **GitNexus**: `npx gitnexus analyze` and verify symbols are correctly linked.

## Rollback Procedure
1. **Before Commit**: `git merge --abort`
2. **After Commit**: `git reset --hard HEAD~1` (on `main`)
3. **Notify**: Inform team if `main` was unstable for any duration.
