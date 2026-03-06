# Schema Refactor: Quick Start Guide

**🚀 Ready to start? Follow this checklist!**

---

## Pre-Flight Checklist (5 min)

```bash
# 1. Verify clean state
cd /Users/jack/Documents/41_personal/lithos
git status  # Should be clean

# 2. Run full verification
mise run verify  # Should pass 100%

# 3. Create feature branch
git checkout -b refactor/schema-file-centric

# 4. Create progress tracker
cp SCHEMA_REFACTOR_MIGRATION.md MIGRATION_PROGRESS.md

# 5. Ready!
echo "✅ Ready to start Phase 1"
```

---

## Phase 1: Day 1 (8 hours)

**Goal**: Add raw file storage infrastructure

### Morning (4 hours)

**Step 1: Dependencies** (30 min)
```bash
cd lithos-core
cargo add blake3
cargo add zstd
cargo add hex  # For Blake3Hash display/parse
cargo add base64  # For zstd compression
cargo check
git add Cargo.toml Cargo.lock
git commit -m "build: add blake3, zstd, hex, base64 dependencies"
```

**Step 2: Blake3Hash type** (1 hour)
- Create `lithos-core/src/schema/hash.rs`
- Copy code from `SCHEMA_REFACTOR_MIGRATION.md` Step 1.2
- Update `lithos-core/src/schema/mod.rs`: add `pub mod hash;`
- Test: `cargo test --package lithos-core --lib schema::hash`
- Commit: `git add . && git commit -m "feat(schema): add Blake3Hash type with rkyv support"`

**Step 3: RingBuffer<T, N>** (1.5 hours)
- Create `lithos-core/src/schema/ring_buffer.rs`
- Copy code from `SCHEMA_REFACTOR_MIGRATION.md` Step 1.3
- Update `lithos-core/src/schema/mod.rs`: add `pub mod ring_buffer;`
- Test: `cargo test --package lithos-core --lib schema::ring_buffer`
- Commit: `git add . && git commit -m "feat(schema): add RingBuffer<T, N> for versioned storage"`

**Step 4: Zstd compression** (1 hour)
- Create `lithos-core/src/schema/compression.rs`
- Copy code from `SCHEMA_REFACTOR_MIGRATION.md` Step 1.4
- Update `lithos-core/src/schema/mod.rs`: add `pub mod compression;`
- Test: `cargo test --package lithos-core --lib schema::compression`
- Commit: `git add . && git commit -m "feat(schema): add zstd compression for rkyv fields"`

---

### Afternoon (4 hours)

**Step 5-11: Raw file types + tables** (4 hours)
- Follow detailed steps in `SCHEMA_REFACTOR_MIGRATION.md` Steps 1.5-1.11
- Create `RawFileVersion`, `RawSchemaFile`, `RawPropertyBankFile`
- Add database tables
- Update `Ingestor` to compute hashes
- Update `Command` adapter to save raw files
- Commit after each major addition

**End of Day 1**:
```bash
# Verify everything works
cargo test --package lithos-core --lib schema
mise run lint

# Merge to feature branch
git checkout refactor/schema-file-centric
git merge phase-1-raw-storage  # If you used a sub-branch
git push origin refactor/schema-file-centric

# Update progress tracker
# Mark Phase 1 tasks as complete in MIGRATION_PROGRESS.md
```

---

## Phase 2: Day 2 (6 hours)

**Goal**: Two-tier staleness detection

Follow `SCHEMA_REFACTOR_MIGRATION.md` Phase 2 steps.

---

## Phase 3: Day 3 (6 hours)

**Goal**: Event system

Follow `SCHEMA_REFACTOR_MIGRATION.md` Phase 3 steps.

---

## Phase 4-5: Day 4 (8 hours)

**Goals**:
- Incremental property resolution (4h)
- Type-driven validation (4h)

Follow `SCHEMA_REFACTOR_MIGRATION.md` Phases 4-5 steps.

---

## Phase 6: Day 5 (6 hours) ⚠️ BREAKING CHANGES

**Goal**: Flatten structure + remove wrappers

### Morning: Flatten adapter/ (3 hours)

```bash
# Step 1: Move files
cd lithos-core/src/schema
git mv adapter/stored.rs stored.rs
git mv adapter/query.rs db_query.rs
git mv adapter/command.rs db_command.rs
git mv adapter/ingestor.rs ingestor.rs
rmdir adapter

# Step 2: Fix imports in moved files
# Change: use super::super::* → use super::*
# (Do this for each moved file)

# Step 3: Update all imports across codebase
rg "schema::adapter::" --type rust  # Find all usages
# Update each file manually

# Verify
cargo check
cargo test --package lithos-core

# Commit
git add .
git commit -m "refactor(schema): flatten adapter/ folder to root"
```

---

### Afternoon: Remove wrappers + move loader (3 hours)

```bash
# Step 1: Extract table definitions (30 min)
# Create db_tables.rs with table definitions
# Update db_query.rs and db_command.rs imports

# Step 2: Delete wrappers (1 hour)
git rm lithos-core/src/schema/query.rs      # 810 lines
git rm lithos-core/src/schema/command.rs    # 394 lines

# Update schema/mod.rs
# Remove: pub mod query; pub mod command;
# Add: pub mod db_query; pub mod db_command;

# Update all usages (find with rg, fix manually)
rg "schema::Query|schema::Command" --type rust

# Verify
cargo check
cargo test

# Commit
git add .
git commit -m "refactor(schema): remove generic Query/Command wrappers (saves 1204 lines)"

# Step 3: Move loader (1.5 hours)
# Create schema/loader.rs (copy from application/schema.rs)
# Delete application/schema.rs
# Update imports across codebase

# Verify
cargo check
cargo test

# Commit
git add .
git commit -m "refactor(schema): move orchestration to schema/loader.rs"
```

---

## Phase 7-8: Day 6 (10 hours)

**Goals**:
- Remove aggregate layer (6h)
- Documentation (4h)

Follow `SCHEMA_REFACTOR_MIGRATION.md` Phases 7-8 steps.

---

## Daily Routine

### Start of Day
```bash
git checkout refactor/schema-file-centric
git pull origin refactor/schema-file-centric
mise run verify  # Ensure clean state
```

### End of Day
```bash
# Run full verification
mise run verify

# Push progress
git push origin refactor/schema-file-centric

# Update progress tracker
# Mark completed tasks in MIGRATION_PROGRESS.md
```

---

## If You Get Stuck

1. **Check migration guide**: `SCHEMA_REFACTOR_MIGRATION.md` (detailed steps)
2. **Check plan**: `SCHEMA_REFACTOR_PLAN.md` (context + rationale)
3. **Check decisions**: `SCHEMA_REFACTOR_DECISIONS.md` (why we chose this)
4. **Check research**: `SCHEMA_REFACTOR_RESEARCH.md` (security + GATs)
5. **Ask team**: Include phase + step number

---

## Emergency Rollback

```bash
# If Phase 1-5 (non-breaking)
git revert <bad-commit>
git push origin refactor/schema-file-centric

# If Phase 6-8 (breaking)
git checkout main
git checkout -b revert/schema-refactor
git revert <commit-range>
# ... manual fixes ...
cargo test
git push origin revert/schema-refactor
```

---

## Success Checklist

After each phase:
- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] Code formatted (`mise run fmt`)
- [ ] Git committed with clear message
- [ ] Progress tracker updated

Final success:
- [ ] All 8 phases complete
- [ ] 1204+ lines removed
- [ ] Zero-copy performance preserved (benchmarks)
- [ ] Documentation updated
- [ ] Ready for review

---

## Timeline

| Day | Phase | Hours | Goal |
|-----|-------|-------|------|
| 1 | Phase 1 | 8 | Raw file storage |
| 2 | Phase 2 | 6 | Staleness detection |
| 3 | Phase 3 | 6 | Event system |
| 4 | Phases 4-5 | 8 | Incremental + validation |
| 5 | Phase 6 | 6 | Flatten + remove wrappers |
| 6 | Phases 7-8 | 10 | Remove aggregate + docs |
| **Total** | | **44h** | Complete |

Add 2-4 hours buffer per day for testing, code review, and unexpected issues.

---

## Ready? Let's go! 🚀

```bash
# Start now:
cd /Users/jack/Documents/41_personal/lithos
git checkout -b refactor/schema-file-centric
cp SCHEMA_REFACTOR_MIGRATION.md MIGRATION_PROGRESS.md
echo "✅ Phase 1, Step 1: Add dependencies"
cd lithos-core
cargo add blake3
```
