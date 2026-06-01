## Objective

Plan a safe merge of `refactor/fsreader-purge-methods` into `main` from the exact divergence point, preserving all changes on both sides and defining conflict handling, validation, and rollback.

## Scope

- Analyze commit/file divergence between current worktree branch and `main`.
- Identify overlap/conflict risk areas.
- Define merge sequence with manual interventions and migration steps.
- Define validation and rollback procedure.
- Do not perform merge in this phase.

## Phases

1. **Divergence Analysis** (complete)
   - Compute merge-base and side-specific commit ranges.
   - Enumerate files changed on each side.
2. **Overlap + Conflict Analysis** (complete)
   - Detect overlapping paths and semantic hotspots.
   - Assess likely conflict classes (textual, API, docs, test).
3. **GitNexus + Rust Best Practices Review** (complete)
   - Map affected modules/processes around `FileReader` changes.
   - Verify expected integration risks.
4. **Merge Strategy Authoring** (complete)
   - Author sequence, interventions, migrations, validation, rollback.
5. **Approval Gate** (in_progress)
   - Present findings/strategy before any merge action.

## Constraints

- All actions must run in `.worktrees/06-rename-fsreader-purge-methods`.
- Planning artifacts stored only under `.scratch/fs-reader-scanner-split/06-worktree-merge`.
- No merge execution until explicit approval.

## Deliverables

- `task_plan.md` (this file)
- `findings.md` (analysis + recommended strategy)
- `progress.md` (chronological execution log)

## Risks Tracked

- Main branch may advance after this analysis, invalidating no-overlap assumption.
- Large documentation rename footprint can hide semantic regressions.
- Re-export/API symbol rename can break doctests if examples are stale.
