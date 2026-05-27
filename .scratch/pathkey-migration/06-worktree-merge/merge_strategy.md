# Merge Strategy: feat/pathkey-redb-traits → main

## Executive Summary

**Merge Type:** Fast-forward merge or rebase recommended
**Risk Level:** LOW-MEDIUM
**File Conflicts:** NONE detected
**Semantic Conflicts:** 3 low-medium risk areas
**Recommended Approach:** Merge feature → main with comprehensive validation

## Risk Assessment

### Overall Risk: LOW-MEDIUM

| Category | Risk | Justification |
|----------|------|---------------|
| File conflicts | NONE | No overlapping file edits |
| API compatibility | LOW | Storage layer changes are transparent |
| Test coverage | LOW | Feature branch: 1346 passing |
| Semantic conflicts | MEDIUM | Property bank processor uses modified storage |
| Rollback complexity | LOW | Clean branch divergence, easy to revert |

### Affected Areas

**Low Risk:**
1. **DB layer** - New PathKey traits, isolated change
2. **Table wrappers** - Additive change, no API breakage
3. **Vault storage** - Internal implementation only

**Medium Risk:**
1. **Property bank processor** - Main refactored business logic; feature changed data layer
2. **Schema discovery** - Main changed entry point; feature changed storage layer
3. **Config storage** - Main did major refactor; feature changed schema storage (similar pattern)

## Merge Sequence

### Recommended: Fast-Forward Merge

```bash
# In main worktree
cd /Users/jack/Documents/41_personal/lithos
git checkout main
git merge --ff-only feat/pathkey-redb-traits

# If ff fails (main has diverged):
git merge --no-ff feat/pathkey-redb-traits
```

**Rationale:**
- No file conflicts detected
- Feature branch is self-contained
- Changes are additive (new traits, wrappers)
- Storage layer modifications are transparent to consumers

### Alternative: Rebase Feature onto Main

```bash
# In feature worktree
cd /Users/jack/Documents/41_personal/lithos/.worktrees/feat/pathkey-redb-traits
git rebase main

# Then merge in main
cd /Users/jack/Documents/41_personal/lithos
git merge --ff-only feat/pathkey-redb-traits
```

**When to use:**
- If you want linear history
- If main has important fixes feature needs
- If you want to test feature on top of latest main

**NOT recommended** because:
- Main has 28 commits, rebase will rewrite feature history
- Feature branch is complete, tested, and documented
- Merge commit preserves feature branch context

## Pre-Merge Validation

### 1. Verify Main Branch Test Status

```bash
cd /Users/jack/Documents/41_personal/lithos
git checkout main
mise run test
```

**Expected:** All tests pass
**If fails:** Document failing tests, assess if related to merge

### 2. Verify Feature Branch Test Status

```bash
cd /Users/jack/Documents/41_personal/lithos/.worktrees/feat/pathkey-redb-traits
git checkout feat/pathkey-redb-traits
mise run test
```

**Expected:** 1346 tests passing (verified)
**Status:** ✅ COMPLETE

### 3. Run GitNexus Impact Analysis

```bash
# From feature worktree
npx gitnexus analyze
```

**Purpose:** Ensure index is current for execution flow tracking

### 4. Verify No Uncommitted Changes

```bash
git status
# Both worktrees should be clean
```

## Merge Execution Plan

### Step 1: Pre-Merge Backup

```bash
cd /Users/jack/Documents/41_personal/lithos
git branch backup/main-pre-pathkey-merge main
```

**Purpose:** Easy rollback point if merge causes issues

### Step 2: Attempt Fast-Forward Merge

```bash
git checkout main
git merge --ff-only feat/pathkey-redb-traits
```

**If succeeds:** Continue to Step 4
**If fails:** Continue to Step 3

### Step 3: Manual Merge (if FF fails)

```bash
git merge --no-ff feat/pathkey-redb-traits -m "Merge feat/pathkey-redb-traits: PathKey redb traits and table wrappers

Implements:
- redb::Value and redb::Key traits for PathKey
- PathUuidTable and UuidPathTable wrappers
- Type-safe DB boundary across vault/note/schema contexts
- Filesystem layer compliance in vault/processor.rs

All 1346 tests passing.
Closes Issue #06 PathKey migration."
```

**Conflict resolution:**
- If conflicts occur, use GitNexus to analyze affected execution flows
- Prefer feature branch changes for storage layer
- Prefer main branch changes for business logic
- Consult findings.md for conflict analysis

### Step 4: Post-Merge Validation

```bash
# Run full test suite
mise run verify

# Expected results:
# - All tests pass (1346+)
# - No clippy warnings
# - Code formatted
# - ADR validation passes
```

### Step 5: Verify Execution Flows

```bash
# Use GitNexus to verify no processes broken
npx gitnexus analyze
# Check critical flows: property bank, schema discovery, config storage
```

### Step 6: Commit Merge (if manual)

```bash
# Only if Step 3 was used
git commit
```

## Conflict Resolution Procedure

### If File Conflicts Occur

**Despite analysis showing no conflicts, if they occur:**

1. **Identify conflict type:**
   ```bash
   git status
   git diff --name-only --diff-filter=U
   ```

2. **Analyze each conflict:**
   ```bash
   # For each conflicted file:
   git diff <file>
   ```

3. **Resolution strategy:**
   - **DB layer conflicts:** Prefer feature branch (new traits)
   - **Storage layer conflicts:** Prefer feature branch (PathKey implementation)
   - **Business logic conflicts:** Prefer main (recent refactors)
   - **Test conflicts:** Merge both, ensure coverage

4. **Validate after resolution:**
   ```bash
   # After resolving each file
   git add <file>
   mise run test -p lithos-core --lib <affected_module>
   ```

### If Semantic Conflicts Occur

**Symptoms:**
- Tests fail after merge
- Compilation errors
- Runtime errors in integration tests

**Resolution:**

1. **Identify failing tests:**
   ```bash
   mise run test 2>&1 | grep -E "(FAILED|error)"
   ```

2. **Analyze with GitNexus:**
   ```bash
   npx gitnexus detect-changes --scope=compare --base-ref=main~1
   ```

3. **Fix pattern:**
   - Property bank: Ensure storage layer API unchanged
   - Schema discovery: Verify PropertyBankDiscovery consumes storage correctly
   - Config storage: Check parallel pattern application

## Rollback Procedure

### Immediate Rollback (Before Push)

```bash
# If merge completed but tests fail:
git reset --hard backup/main-pre-pathkey-merge
```

**Recovery:** All changes reverted, back to pre-merge state

### Rollback After Push

```bash
# Create revert commit
git revert -m 1 <merge-commit-hash>

# Or hard reset (if no one pulled)
git reset --hard backup/main-pre-pathkey-merge
git push --force-with-lease
```

**Warning:** Coordinate with team if main is shared

## Validation Checklist

### Pre-Merge
- [ ] Main branch tests pass
- [ ] Feature branch tests pass (1346 ✓)
- [ ] No uncommitted changes in either worktree
- [ ] GitNexus index up to date
- [ ] Backup branch created

### Post-Merge
- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] Code formatted (`mise run fmt`)
- [ ] ADR validation passes
- [ ] Property bank integration tests pass
- [ ] Schema discovery tests pass
- [ ] Config storage tests pass
- [ ] Vault storage tests pass
- [ ] Note storage tests pass

### Final Verification
- [ ] GitNexus execution flows intact
- [ ] No new compilation warnings
- [ ] Documentation updated (if needed)
- [ ] Merge commit message accurate

## Manual Interventions

### None Expected

**Reason:** No file conflicts detected, storage layer changes transparent

**If needed:**
- Document in `progress.md`
- Update this file with resolution details
- Create ADR if architectural decision made

## Migration Requirements

### None Required

**Reason:** Changes are backward compatible
- New redb traits don't break existing code
- Table wrappers are transparent replacements
- Storage layer changes internal only

**If breaking changes discovered:**
- Document in findings.md
- Create migration guide
- Update CHANGELOG.md

## Success Criteria

Merge is successful when:
1. ✅ All 1346+ tests pass
2. ✅ No clippy warnings
3. ✅ Code formatted correctly
4. ✅ GitNexus execution flows intact
5. ✅ Property bank processor works correctly
6. ✅ Schema discovery works correctly
7. ✅ Config storage works correctly
8. ✅ No runtime errors in integration tests

## Timeline Estimate

- **Pre-merge validation:** 15 minutes
- **Merge execution:** 5 minutes
- **Post-merge validation:** 20 minutes
- **Total:** ~40 minutes

**Contingency:**
- Add 30 minutes if conflicts occur
- Add 1 hour if semantic conflicts require fixes

## Next Steps

1. Verify main branch test status
2. Create backup branch
3. Attempt fast-forward merge
4. Run validation suite
5. Verify execution flows with GitNexus
6. Document results in progress.md
