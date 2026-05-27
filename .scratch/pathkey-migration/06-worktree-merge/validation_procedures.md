# Validation & Rollback Procedures

## Pre-Merge Validation

### Step 1: Verify Both Branches Independently

```bash
# 1a. Main branch — test
cd /Users/jack/Documents/41_personal/lithos
git checkout main
cargo test -p lithos-core --lib
```
- **Expected:** ✓ 1360 passing (verified 2026-05-27)
- **If fails:** Do NOT merge — fix main first

```bash
# 1b. Feature branch — test
cd /Users/jack/Documents/41_personal/lithos/.worktrees/feat/pathkey-redb-traits
cargo test -p lithos-core --lib
```
- **Expected:** ✓ 1346 passing (verified 2026-05-27)
- **If fails:** Do NOT merge — fix feature branch first

### Step 2: Verify Quality Gates

```bash
# 2a. Feature branch quality
cd /Users/jack/Documents/41_personal/lithos/.worktrees/feat/pathkey-redb-traits
cargo clippy -p lithos-core -- -D warnings
cargo fmt -p lithos-core --check
```
- **Expected:** ✓ No clippy warnings, no formatting issues (verified 2026-05-27)

### Step 3: Create Backup

```bash
cd /Users/jack/Documents/41_personal/lithos
git branch backup/main-pre-pathkey-merge main
```
- **Purpose:** Quick recovery point

### Step 4: Verify GitNexus Index Fresh

```bash
npx gitnexus status
```
- **Expected:** Index is up-to-date or within acceptable staleness
- **If stale:** Run `npx gitnexus analyze`

---

## Merge Execution

### Primary: Fast-Forward Merge

```bash
cd /Users/jack/Documents/41_personal/lithos
git checkout main
git merge --ff-only .worktrees/feat/pathkey-redb-traits
```

**If ff-only succeeds:** ✓ Clean linear history
**If ff-only fails:** (diverged) → Use `--no-ff` instead

### Fallback: No-Fast-Forward Merge

```bash
git merge --no-ff .worktrees/feat/pathkey-redb-traits \
  -m "Merge feat/pathkey-redb-traits: PathKey redb traits and table wrappers

- Implements redb::Value and redb::Key for PathKey
- Adds PathUuidTable and UuidPathTable wrappers
- Migrates vault/note/schema storage to PathKey
- Fixes filesystem layer violation in vault/processor.rs
- Removes deprecated path_key() helper

All 1346 tests passing. No file conflicts with main.
Closes Issue #06."
```

**Expected outcome:** Clean merge — no file conflicts detected during analysis

---

## Post-Merge Validation

### Gate 1: Compilation

```bash
cd /Users/jack/Documents/41_personal/lithos
cargo build -p lithos-core
```
**Expected:** ✓ Compilation succeeds (0 errors, 0 warnings)
**If fails:** Stop — investigate compilation errors

### Gate 2: Full Test Suite

```bash
cargo test -p lithos-core --lib
```
**Expected:** ✓ All tests pass (1360+ after merge)
**If fails:** Run only failing tests, investigate, fix

### Gate 3: Critical Test Modules

Target individual modules for regression verification:

```bash
# Property bank processor — HIGHEST RISK area
cargo test -p lithos-core -- schema::property_bank_processor 2>&1

# Schema discovery
cargo test -p lithos-core -- schema::discovery 2>&1

# Schema builder
cargo test -p lithos-core -- schema::builder 2>&1

# Vault storage — directly changed
cargo test -p lithos-core -- vault::storage 2>&1

# Note storage — directly changed
cargo test -p lithos-core -- note::storage 2>&1

# DB layer — new traits
cargo test -p lithos-core -- db::path 2>&1
cargo test -p lithos-core -- db::table 2>&1

# Config storage — main-only changes, verify still works
cargo test -p lithos-core -- config::storage 2>&1
```

**Expected:** Each module passes independently

### Gate 4: Integration Tests

```bash
cargo test -p lithos-core 2>&1 | tail -3
```
**Expected:** ✓ All integration tests pass

### Gate 5: Lint & Format

```bash
cargo clippy -p lithos-core -- -D warnings 2>&1
cargo fmt -p lithos-core --check 2>&1
```
**Expected:** ✓ No warnings, no formatting issues

### Gate 6: GitNexus Execution Flow Integrity

```bash
cd /Users/jack/Documents/41_personal/lithos
npx gitnexus analyze 2>&1 | tail -3
npx gitnexus detect-changes 2>&1
```
**Expected:** No unexpected execution flow breaks. Focus on property bank and schema discovery processes.

### Gate 7: ADR Validation

```bash
mise run adr:validate 2>&1
```
**Expected:** ✓ All ADRs pass validation

---

## Rollback Procedures

### Rollback Strategy A: Pre-Push Recovery

**Trigger:** Post-merge validation fails (any gate above)

```bash
# 1. Abort merge in progress (if still merging)
git merge --abort

# 2. Or reset to backup if merge completed
git reset --hard backup/main-pre-pathkey-merge
git branch -D backup/main-pre-pathkey-merge
```
**Recovery time:** < 1 minute
**Data loss:** None

### Rollback Strategy B: Post-Push Recovery

**Trigger:** Push completed but issues discovered

```bash
# Method 1: Revert (safe — preserves history)
git revert -m 1 <merge-commit-hash>
git push

# Method 2: Reset (only if team okay with force push)
git checkout main
git reset --hard backup/main-pre-pathkey-merge
git push --force-with-lease
git branch -D backup/main-pre-pathkey-merge
```
**Recovery time:** < 2 minutes
**Data loss:** None (changes can be re-applied from feature branch)

---

## Merge Acceptance Checklist

### Pre-Merge
- [x] Feature branch tests: 1346 passing
- [x] Main branch tests: 1360 passing
- [x] No file conflicts detected
- [x] Semantic overlap assessed (3 areas, MEDIUM risk max)
- [ ] Backup branch created: `backup/main-pre-pathkey-merge`

### Post-Merge
- [ ] Compilation succeeds
- [ ] Full test suite passes
- [ ] Property bank processor integration tests pass
- [ ] Schema discovery tests pass
- [ ] Schema builder tests pass
- [ ] Vault storage tests pass
- [ ] Note storage tests pass
- [ ] DB layer tests pass
- [ ] Config storage tests pass
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] GitNexus execution flows intact
- [ ] ADR validation passes

### Rollback Ready
- [ ] Backup branch exists
- [ ] Revert command documented
- [ ] Team notified of merge (if applicable)
