# Merge Strategy: db-cleanup-task-10 ← main

## Approach
Merge `main` INTO `db-cleanup-task-10` with manual conflict resolution.

## Why this direction?
- Worktree has **irreversible deletions** (reader.rs, writer.rs, Database struct, BatchReader/BatchWriter exports)
- Merging the other way would resurrect deleted files and require re-cleaning
- Worktree is 3 commits ahead of base; main is 29 commits ahead; the merge base is `5642c487`

## Merge Sequence

### Step 1: Initiate merge
```bash
git merge main
```
Expected: merge fails with conflicts in 3 files (AGENTS.md, db/mod.rs, vault/processor.rs)

### Step 2: Resolve `AGENTS.md`
- **Problem**: Both branches changed GitNexus stats on the same line
- **Resolution**: Accept main's version (newer index: 19642 symbols, 25835 relationships, 265 execution flows)
- **Command**: `git checkout --theirs AGENTS.md`
- **Risk**: None (cosmetic)

### Step 3: Resolve `lithos-core/src/db/mod.rs`
- **Problem**: Both branches restructured exports
- **Main adds**: `mod path;`, `PathUuidTable`/`UuidPathTable` exports, keeps reader/writer/Database
- **Worktree removes**: `mod reader;`, `mod writer;`, `Database`, `BatchReader`, `BatchWriter`
- **Resolution**: Manually construct the merged version containing:
  - `mod path;` — from main
  - NO `mod reader;` or `mod writer;` — from worktree
  - `pub use core::Store;` — from worktree (no Database)
  - All table exports from main including `PathUuidTable`/`UuidPathTable`
  - NO `BatchReader` or `BatchWriter` exports — from worktree
- **Risk**: LOW — clearly additive/deletive changes, no logic changes

### Step 4: Resolve `lithos-core/src/vault/processor.rs`
- **Problem**: Both branches restructured the discovery functions and pipeline
- **Main changes**: PathFile/FilePath typed path conversion, removed `normalized_path_from_relative`, added path tests
- **Worktree changes**: Removed type aliases, made signatures generic (`impl vault_repository::Repository`)
- **Resolution**: Accept BOTH changes. The path conversion (main) replaces `normalized_path_from_relative`. The generic signatures (worktree) replace concrete type aliases. Tests from main need to be kept.
- **Key integration points**:
  - `discover()`: uses `vault_storage::RedbRepository::new()` (worktree) + `entry.path().as_key(&root)` (main) — both compatible
  - `compare()`: `&impl vault_repository::ReadRepository` (worktree) — no conflict with main's path changes
  - `route()`: `&impl note_repository::Repository` (worktree) — no conflict
  - `prune()`: `&impl vault_repository::Repository` (worktree) — no conflict
  - Test imports: main's path tests use `DirPath`/`FilePath` from `crate::fs::path` — these are independent
- **Risk**: MEDIUM — need to ensure the type alias removal (worktree) and path conversion (main) are both properly integrated without double-import or missing import issues

### Step 5: Verify auto-merged files
All remaining files should auto-merge cleanly. Specifically verify:
- `lithos-core/src/db/path.rs` (new file from main) — present
- `lithos-core/src/db/core.rs` (Database removed by worktree) — Database still gone
- `lithos-core/src/db/table.rs` (PathUuidTable/UuidPathTable from main) — present
- All context storage files (changes from main on vault/note/config/schema) — applied
- `.mise/tasks/` (bash compat from main) — applied
- All schema changes (property bank, builder) from main — applied

### Step 6: Build and test
```bash
cargo check
mise run fmt
mise run lint
mise run test
```

## Validation Procedures

### Compile check
```bash
cargo check 2>&1
```
Expected: zero errors

### Format check
```bash
mise run fmt  # or: cargo fmt --check
```

### Lint check
```bash
mise run lint  # or: cargo clippy --all-targets -- -D warnings
```

### Test
```bash
mise run test  # or: cargo nextest run
```

### GitNexus change detection
```bash
npx gitnexus detect-changes 2>/dev/null || true
```

## Manual Interventions Required

### Required: db/mod.rs
Must be manually edited to combine main's additions with worktree's deletions.

### Required: vault/processor.rs
Must be manually edited to combine both changesets.

### Optional: AGENTS.md
Can auto-resolve by accepting main's version with `--theirs`.

## Rollback Procedure

### If merge fails at Step 1 (auto-merge conflicts)
```bash
git merge --abort
```
Restores clean pre-merge state. Fix the conflicting file according to the resolution guide above, then retry.

### If validation fails at Step 6
```bash
git merge --abort
```
Investigate the specific error. The merge strategy assumes all non-overlapping files merge cleanly; an unexpected conflict indicates an incorrect assumption about file exclusivity.

## Migration Notes

### No data migration required
- All changes are code-level refactors
- No schema changes to the redb database files
- Backward compatible at the storage format level

### Import compatibility
- `BatchReader`, `BatchWriter`, `Database` were public exports — any external consumers (none identified) would break
- `PathUuidTable`/`UuidPathTable` are new exports — no existing consumers to break
- All trait renames (`SchemaReadRepository` → `ReadRepository`, etc.) are internal
