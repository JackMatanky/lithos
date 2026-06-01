# Task Plan: Merge issue-08/absolutepath-removal into main

## Goal
Merge the `issue-08/absolutepath-removal` worktree branch back into `main`, preserving all changes from both branches since their divergence.

## Branches
| Branch | Worktree | HEAD |
|--------|----------|------|
| `main` | `/Users/jack/Documents/41_personal/lithos` | `ea446e22` (+2 ahead of merge base) |
| `issue-08/absolutepath-removal` | `.worktrees/issue-08-absolutepath-removal` | `1beaec9a` (4 commits) |
| `issue/root-config-discovery-01-discovery-type-contracts` | `.worktrees/root-config-discovery-01-discovery-type-contracts` | `419fd613` (1 commit) |

## Merge Base
- Common ancestor: `0ad7aee67d832ffaccb0084bc218e0ac8f409c4e`

## Phases

### Phase 1: Analysis — Complete
- [x] Identify merge base: `0ad7aee`
- [x] List files changed on `main` since base (17 `.scratch/*.md` files)
- [x] List files changed on `issue-08` since base (6 files: 1 `.scratch/*.md` + 5 Rust source files)
- [x] Check overlapping edits with other worktree (root-config-discovery)
- [x] Run dry-run merge to verify no conflicts
- [x] Run GitNexus impact analysis
- [x] Re-review changes against Rust best practices

### Phase 2: Merge Strategy Design — Complete
- [x] Determine merge direction
- [x] Plan post-merge validation
- [x] Document rollback procedure

### Phase 3: Present for Approval — Waiting
- [ ] Show analysis, findings, and strategy
- [ ] Await user approval

### Phase 4: Execute Merge — Pending
- [ ] Commit planning artifacts
- [ ] Merge `issue-08/absolutepath-removal` into `main`
- [ ] Run `mise run fmt && mise run lint && mise run test:unit`
- [ ] Commit merge artifacts
- [ ] Push

## Decisions
| Decision | Choice | Rationale |
|----------|--------|-----------|
| Merge type | `--no-ff` merge commit | Preserves branch topology; main has advanced |
| Merge location | Main worktree | Target is `main`; merge feature branch in |
| Sequence | issue-08 first; root-config-discovery independent | No overlap between branches |
