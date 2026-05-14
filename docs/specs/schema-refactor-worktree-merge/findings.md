# Findings & Decisions

## Requirements
- Plan how to merge a heavily diverged `schema-refactor` worktree back into `main`.
- Avoid harming work in either worktree.
- Produce a safe, practical procedure that can be executed with low rollback risk.

## Research Findings
- Two active worktrees exist:
  - `/Users/jack/Documents/41_personal/lithos` on `main` at `2a00f493...`
  - `/Users/jack/Documents/41_personal/lithos-schema-refactor` on `schema-refactor` at `623b2052...`
- Current branch in this workspace is `schema-refactor`; working tree is clean.
- Divergence from merge base is substantial:
  - `main...schema-refactor` unique commits count = `183` (main) / `337` (schema-refactor).
- `schema-refactor` includes extensive schema pipeline and architecture changes.
- `main` includes extensive note/parser/scanner evolution and docs/tooling changes.
- Local `main` is ahead of `origin/main` in this repo context, so remote parity should be verified before final promotion.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Use third, temporary reconciliation worktree | Isolates risky merge operations from both active worktrees |
| Add checkpoint tags before merge rehearsal | Provides deterministic anchors for recovery and comparison |
| Generate a git bundle backup before integration | Adds durable backup beyond reflog convenience |
| Rehearse merge and validate before touching `main` | Reduces chance of polluting mainline with unresolved integration risk |
| Use non-destructive rollback (`git revert -m 1`) if needed | Preserves shared history integrity and avoids force pushes |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| Session-catchup script returned no text output | Proceeded by directly collecting git topology and divergence data |

## Resources
- `task_plan.md`
- `progress.md`
- `docs/specs/unified-schema-discovery-engine/task_plan.md`
- Git commands used:
  - `git worktree list --porcelain`
  - `git status --short --branch`
  - `git branch --all --verbose --no-abbrev`
  - `git merge-base main schema-refactor`
  - `git rev-list --left-right --count main...schema-refactor`
  - `git log --oneline --decorate --graph --left-right main...schema-refactor`

## Visual/Browser Findings
- None.

---
*Updated with repository-specific branch topology and divergence metrics.*
