# Merge Rehearsal Findings (schema-refactor -> main)

Date: 2026-05-06
Branch: `reconcile/schema-refactor-into-main`

## Outcome
- Rehearsal merge executed: `git merge --no-ff schema-refactor`
- Merge stopped with conflicts (expected for this divergence level).

## Conflict Summary
- **True content conflicts (UU/AA/DU):**
  - `.opencode/package-lock.json` (AA)
  - `.opencode/plans/system-reminder.md` (AA)
  - `skills-lock.json` (AA)
  - `lithos-core/src/config/aggregate.rs` (UU)
  - `lithos-core/src/fs/reader.rs` (UU)
  - `lithos-core/src/application/vault.rs` (DU)
- **Path relocation conflicts (UA due directory renames on main):**
  - `docs/history/bmad/planning/schema-error-plan.md`
  - `docs/history/bmad/research/ARCHITECTURE_DECISIONS.md`
  - `docs/history/bmad/research/IMPLEMENTATION_PLAN.md`
  - `docs/history/bmad/research/schema-pipeline-review.md`
  - `docs/history/bmad/research/schema-pipeline-typestate-redesign.md`
  - `docs/legacy/research/note/redb_tree_storage_research.md`
  - `docs/legacy/research/note/rust-graph-best-practices.md`
  - `docs/legacy/research/note/state-machine-patterns-rust.md`

## Interpretation
- The merge is operationally feasible in an isolated branch.
- Most changes merged cleanly; conflict set is concentrated in:
  - tooling metadata lock files,
  - docs tree moves/renames,
  - two core Rust files with independent edits,
  - one delete-vs-modify decision (`application/vault.rs`).

## Recommended Next Resolution Order
1. Resolve **UA path-relocation** entries first (accept relocated targets, verify no duplicates).
2. Resolve **AA lock/metadata files** (`skills-lock.json`, package lock, system reminder).
3. Resolve **Rust code conflicts** (`config/aggregate.rs`, `fs/reader.rs`).
4. Decide `vault.rs` policy (keep deleted from `main` or restore/adapt from `schema-refactor`).
5. Run incremental verification (`mise run fmt`, `mise run lint`, targeted tests), then `mise run verify`.
