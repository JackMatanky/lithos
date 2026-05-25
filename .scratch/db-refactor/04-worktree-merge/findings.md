# Findings: Merge Planning for `db-refactor-segregated-traits` -> `main`

## Repository Topology
- Worktrees:
  - `/Users/jack/Documents/41_personal/lithos` -> `main` (`eb068e84`)
  - `/Users/jack/Documents/41_personal/lithos/.worktrees/db-refactor-segregated-traits` -> `db-refactor-segregated-traits` (`71cc37c3`)
  - `/Users/jack/Documents/41_personal/lithos-schema-refactor` -> `schema-refactor`

## Divergence
- `main...db-refactor-segregated-traits` commit count: `34` on main side, `37` on db-refactor side.
- This is a true two-way divergence; fast-forward is not possible.

## Concrete Overlap Analysis (not assumption)
- Files changed since merge-base:
  - `main`: 49 files
  - `db-refactor-segregated-traits`: 44 files
  - overlap: 9 files
- Overlap file list:
  - `.gitignore`
  - `lithos-core/src/config/error.rs`
  - `lithos-core/src/schema/error.rs`
  - `lithos-core/src/schema/mod.rs`
  - `lithos-core/src/schema/repository.rs`
  - `lithos-core/src/schema/schema_processor.rs`
  - `lithos-core/src/schema/storage_v2/core.rs`
  - `lithos-core/src/schema/storage_v2/mod.rs`
  - `lithos-core/src/schema/storage_v2/tables.rs`

## Merge-Tree Conflict Preview (actual conflict candidates)
- `changed in both`:
  - `lithos-core/src/config/error.rs`
  - `lithos-core/src/schema/error.rs`
  - `lithos-core/src/schema/mod.rs`
  - `lithos-core/src/schema/schema_processor.rs`
- `removed in local` (main deleted, db branch modified):
  - `lithos-core/src/schema/repository.rs`
- `removed in remote` (db branch deleted, main kept):
  - `docs/refs/rust/patterns/typestate-branching-enums.md`
  - `docs/refs/rust/state-machine-pattern.md`
  - `docs/refs/rust/typestate-pattern-research.md`
  - `lithos-core/src/schema/storage.rs`
  - `lithos-core/src/schema/testing.rs`

## Interpretation
- Your concern is correct: there is non-trivial overlap in pervasive infrastructure files.
- Highest-risk integration points are schema seam files that changed in both branches, especially:
  - `lithos-core/src/schema/mod.rs`
  - `lithos-core/src/schema/repository.rs` (delete/modify conflict)
  - `lithos-core/src/schema/schema_processor.rs`
- Test harness files also show likely semantic reconciliation needs after merge preview:
  - `lithos-core/tests/common/mod.rs`
  - `lithos-core/tests/property_bank_processor.rs`
  - `lithos-core/tests/schema_loader.rs`
  - `lithos-core/tests/schema_storage.rs`

## User Clarifications Captured
- `lithos-core/src/schema/repository.rs` should be taken fully from db-refactor.
- `lithos-core/tests/**` should mostly follow db-refactor unless there are necessary post-divergence main edits.
- Typestate docs were intentionally moved; merged tree must keep them under `docs/refs/rust/state_machine_typestate/` and not revert to old paths.
- `lithos-core/src/schema/testing.rs` was intentionally moved/replaced by `lithos-core/src/schema/storage/testing.rs`.
- Main-side `storage_v2` files are known mistaken work and should be excluded in final merge result:
  - `lithos-core/src/schema/storage_v2/core.rs`
  - `lithos-core/src/schema/storage_v2/mod.rs`
  - `lithos-core/src/schema/storage_v2/tables.rs`

## Workstream Separation (Observed)
- Main-only commits heavily include `fs` and issue updates tied to fs-inode architecture.
- DB-refactor-only commits heavily include schema repository/storage migration and db-refactor issue closeout docs.

## Risk Assessment
- Primary risk is conflict in shared files (docs/harness/agent metadata), not direct fs-vs-schema source overlap.
- Merge commit is safer than rebase for preserving both timelines and simplifying auditability.

## Recommended Merge Mechanics
1. Merge into `main` from main worktree using `git merge --no-ff db-refactor-segregated-traits`.
2. Resolve conflicts by domain ownership first (schema from db-refactor, fs from main).
3. Manually reconcile shared files line-by-line.
4. Run full verification gates before cleanup.

## Why This Approach
- Preserves existing commit identities from both lines of work.
- Minimizes risk of dropping fs work during history rewriting.
- Produces explicit integration commit that can be reviewed independently.
