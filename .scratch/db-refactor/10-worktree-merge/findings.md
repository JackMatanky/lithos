# Findings & Decisions: Worktree Merge Analysis

## Requirements
- Preserve ALL changes from both `db-cleanup-task-10` worktree AND `main` since divergence point `5642c4870cea4e8792e7eeec366e473b39bc20c0`
- Identify overlapping edits and merge conflicts
- Define recommended merge sequence
- Document required migrations or manual interventions
- Include validation and rollback procedures

## Research Findings

### Branch Topology

```
main       ... A ← B ← C ← ... ← Z (29 commits since base)
               ↑
               base (5642c487)
               ↓
worktree   ... 1 ← 2 ← 3  (3 commits since base)
```

### Worktree Changes (3 commits)

1. **`39a7f93f`** — Remove reader.rs, writer.rs (~2800 lines); Remove `Database` struct from core.rs; Normalize trait names across all 5 contexts
2. **`a49d0089`** — Fix access modifiers: `RedbRepository` → `pub`, `InMemoryRepository` → `pub(crate)`
3. **`b0f29cd7`** — Update issue tracking docs

**Files changed (worktree only):**
- `.scratch/db-refactor/10-cross-context-verification-and-legacy-cleanup.md` (updated)
- `lithos-core/src/db/reader.rs` (deleted)
- `lithos-core/src/db/writer.rs` (deleted)
- `lithos-core/src/db/core.rs` (Database removed)
- `lithos-core/src/db/mod.rs` (exports cleaned)
- `lithos-core/src/vault/processor.rs` (type alias removal, trait-based signatures)
- `lithos-core/src/schema/storage/mod.rs` (visibility fixes)
- All 5 context storage/testing files (trait normalization)
- All 3 benchmark files (Store/Table API migration)
- Various docs/scratch files (ADRs, proposals, test doc)

### Main Changes (29 commits)

Major changesets (from newest to oldest):

**A. ComparisonBranch inline + deepening analysis**
- `schema/builder.rs`: inlined ComparisonBranch
- Various docs

**B. Bash compat fix**
- `.mise/tasks/`: bash 3.2 compat (remove namerefs)

**C. Impl Tightening**
- `schema/property_bank_processor.rs`: stage init, clone elim, dead code

**D. Property Bank Remove Discovery Stage**
- `schema/property_bank_processor.rs`: removed Discovery stage
- `schema/builder.rs`: adapted to new pipeline

**E. PathKey redb traits + PathUuidTable/UuidPathTable (LARGEST CHANGESET)**
- `lithos-core/src/db/path.rs` — NEW FILE: PathKey redb Value/Key trait impls
- `lithos-core/src/db/table.rs` — PathUuidTable, UuidPathTable types added
- `lithos-core/src/db/mod.rs` — mod path; exports PathUuidTable, UuidPathTable
- `lithos-core/src/vault/storage/`:
  - Tables: `PathTable` → `PathUuidTable`, `UuidTable<FileId, String>` → `UuidPathTable<FileId>`
  - Read: direct PathKey table ops
  - Write: `&str` → `&PathKey` params, `String` → `PathKey` in delete contexts
- `lithos-core/src/note/storage/`:
  - Tables: `PathTable<&[u8]>` → `PathUuidTable<NoteId>`
  - Read: PathKey conversion for table lookups
  - Write: PathKey for all insert/remove
- `lithos-core/src/config/storage/`:
  - Tables: `PathTable` → `Table<&str>` (regressed to string keys)
  - Read/Write: `.as_key()` → `.as_key().as_str()`
- `lithos-core/src/vault/processor.rs`: typed FilePath/DirPath conversion, removed `normalized_path_from_relative`, added path conversion tests
- `lithos-core/src/schema/storage/mod.rs`: removed `path_key_to_string()` helper

**F. Post-merge compat fix**
- `lithos-core/src/config/storage/`: PathTable compatibility fix

### Overlapping Files (3 files changed on BOTH branches)

#### 1. `AGENTS.md` — Trivial
- **Main**: GitNexus stats (19642 symbols, 25835 relationships, 265 execution flows)
- **Worktree**: GitNexus stats (19489 symbols, 25680 relationships, 300 execution flows)
- **Conflict**: Same line different values → take main's (newer index)
- **Resolution**: Accept main's version (auto-merge)

#### 2. `lithos-core/src/db/mod.rs` — Structural conflict
- **Main's changes**:
  - Added `mod path;`
  - Added exports: `PathUuidTable`, `UuidPathTable`
  - KEPT: `Database`, `BatchReader`, `BatchWriter`, `reader`, `writer`
- **Worktree's changes**:
  - Removed `mod reader;`
  - Removed `mod writer;`
  - Changed `pub use core::{Database, Store}` → `pub use core::Store`
  - Removed `pub use reader::BatchReader`
  - Removed `pub use writer::BatchWriter`
  - Updated doc comment (removed Database reference)
- **Resolution**: Merge manually. Keep worktree's deletions + main's additions.

#### 3. `lithos-core/src/vault/processor.rs` — Semantic conflict
- **Main's changes**:
  - Replaced `normalized_path_from_relative()` with typed `entry.path().as_key(&root)` calls
  - Removed `normalized_path_from_relative` function
  - Added path conversion tests
  - KEPT: `NoteRepository`/`VaultRepository` type aliases
  - KEPT: `&VaultRepository`/`&NoteRepository` parameter types
- **Worktree's changes**:
  - Replaced `NoteRepository`/`VaultRepository` type aliases with direct `vault_storage::RedbRepository`/`note_storage::RedbRepository`
  - Changed signatures: `&VaultRepository` → `&impl vault_repository::Repository`
  - Changed signatures: `&NoteRepository` → `&impl note_repository::Repository`
  - KEPT: `normalized_path_from_relative` function
- **Compatibility**: Both changesets are COMPATIBLE (different parts of the file mostly). The worktree uses trait-based generics, main uses typed path conversion. However, the test imports may need adjustment (main adds tests that reference `DirPath`/`FilePath`, worktree removes type aliases).
- **Resolution**: Merge both. Accept main's path conversion code + tests. Keep worktree's trait-based signatures.

### Files Changed Exclusively on One Branch

**Main only (42 files + 3 deleted):**
- `lithos-core/src/db/path.rs` (NEW)
- `lithos-core/src/db/table.rs` (modified — PathUuidTable/UuidPathTable)
- `lithos-core/src/vault/storage/read.rs` (modified — PathKey direct)
- `lithos-core/src/vault/storage/tables.rs` (modified — PathUuidTable/UuidPathTable)
- `lithos-core/src/vault/storage/write.rs` (modified — PathKey types)
- `lithos-core/src/note/storage/read.rs` (modified — PathKey lookup)
- `lithos-core/src/note/storage/tables.rs` (modified — PathUuidTable)
- `lithos-core/src/note/storage/write.rs` (modified — PathKey insert/remove)
- `lithos-core/src/config/storage/read.rs` (modified — string table keys)
- `lithos-core/src/config/storage/tables.rs` (modified — Table<&str>)
- `lithos-core/src/config/storage/write.rs` (modified — string table keys)
- `lithos-core/src/schema/storage/mod.rs` (modified — removed path_key helper)
- `lithos-core/src/schema/storage/read.rs` (modified — path_key removal)
- `lithos-core/src/schema/storage/write.rs` (modified — path_key removal)
- `lithos-core/src/schema/property_bank_processor.rs` (modified — Discovery removal)
- `lithos-core/src/schema/builder.rs` (modified — pipeline changes)
- `lithos-core/src/vault/processor.rs` (modified — path conversion) [OVERLAP]
- `lithos-core/src/schema/discovery.rs` (modified)
- `.mise/tasks/build` (modified — bash compat)
- `.mise/tasks/lint` (modified — bash compat)
- Various `.scratch/` docs, `docs/adr/`, `docs/history/`, `AGENTS.md`

**Worktree only (33 files + 2 deleted):**
- `lithos-core/src/db/reader.rs` (DELETED)
- `lithos-core/src/db/writer.rs` (DELETED)
- `lithos-core/src/db/core.rs` (modified — Database removed)
- `lithos-core/src/db/mod.rs` (modified — exports cleaned) [OVERLAP]
- All 5 context `storage/mod.rs` + `storage/testing.rs` (visibility/naming)
- All 5 context `storage/read.rs` + `storage/write.rs` (trait normalization)
- All 3 benchmark files
- Various `docs/` and `.scratch/` files
- `lithos-core/tests/note_reader.rs`

### Non-Overlapping Files — Auto-Merge Safe

All files changed exclusively on ONE branch will auto-merge cleanly:
- ✅ `lithos-core/src/db/path.rs` — new file from main, added automatically
- ✅ `lithos-core/src/db/reader.rs` — deleted by worktree, not changed on main
- ✅ `lithos-core/src/db/writer.rs` — deleted by worktree, not changed on main
- ✅ `lithos-core/src/db/core.rs` — Database removed by worktree, not changed on main
- ✅ All context storage files changed only on one branch
- ✅ All benchmark files
- ✅ All schema files
- ✅ All .mise/tasks files

## Technical Decisions

| Decision | Rationale |
|----------|-----------|
| Merge main INTO worktree branch | Worktree has irreversible deletions (reader/writer); merging the other way would leave dead files |
| Accept main's vault/note/config PathUuidTable changes | They are the correct evolutionary step on main |
| Accept worktree's trait normalization + deletion | They are the correct cleanup |
| Custom resolution for db/mod.rs | Need worktree's deletions AND main's additions |
| Custom resolution for vault/processor.rs | Need typed paths from main AND trait-based signatures from worktree |
| Take main's AGENTS.md stats | Reflects a more recent GitNexus index |
| Rollback via `git merge --abort` | Standard git safety net |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| Previous merge attempt failed (resurrected reader/writer/Database) | Aborted with `git merge --abort` — this plan avoids that by manual resolution of db/mod.rs |

## Resources
- Worktree directory: `/Users/jack/Documents/41_personal/lithos/.worktrees/db-cleanup-task-10`
- Merge base: `5642c4870cea4e8792e7eeec366e473b39bc20c0`
- Previous issue: `.scratch/db-refactor/10-cross-context-verification-and-legacy-cleanup.md`
