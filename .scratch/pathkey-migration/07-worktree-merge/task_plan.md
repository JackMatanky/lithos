# Task Plan: Merge Worktree `07/pathkey-note-template` into `main`

## Goal
Integrate issue 07 implementation from dedicated worktree back into `main`.

## Phases

### Phase 1: Analysis [COMPLETE]
- [x] Gather divergence details (commits, files, overlap)
- [x] Run GitNexus impact analysis on worktree changes
- [x] Compare main's state vs worktree state for overlapping files
- [x] Identify any merge conflicts or required manual interventions

**Key finding**: Zero overlapping files. Main added 21 docs/skills/config files since divergence; worktree added 5 files (4 source + 1 issue). No source-level overlap.

### Phase 2: Strategy Definition [COMPLETE]
- [x] Define merge sequence (order of operations)
- [x] Identify migrations / manual interventions needed
- [x] Define validation plan
- [x] Define rollback procedure

**Strategy**: Commit worktree → merge main into worktree → validate → fast-forward main.
**Migrations required**: None.
**Manual interventions**: None.

### Phase 3: Approval [PENDING]
- [ ] Present findings and strategy to user
- [ ] Await sign-off before execution

### Phase 4: Execution [PENDING]
- [ ] Stage and commit worktree changes
- [ ] Merge main into worktree
- [ ] Validate merged state (tests, clippy, fmt)
- [ ] Fast-forward main to worktree tip

### Phase 5: Cleanup [PENDING]
- [ ] Stage and commit planning artifacts to main
- [ ] Run `npx gitnexus analyze`
- [ ] Final status report

## Artifacts
- `findings.md` — Full divergence, overlap, and risk analysis
- `merge-strategy.md` — Merge sequence with validation and rollback
- `progress.md` — Session log

## Approval Question
Merge `07/pathkey-note-template` into `main`? No overlapping files. No migrations. No manual interventions. Fast-forward merge with validation gate.
