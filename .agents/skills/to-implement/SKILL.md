---
name: to-implement
disable-model-invocation: true
description: Compact session into a tight opening prompt for implementation. User-invoked.
argument-hint: "<scratch-issue-path>"
---

# Handoff to Implementation

## Workflow

### 0 — Freshen index

Check GitNexus index freshness (last indexed timestamp vs HEAD). Stale? Run `.gitnexus/run.cjs analyze`.
**Done:** index matches HEAD commit.

### 1 — Gather state

Record: current branch, uncommitted changes (`git status --porcelain`), index freshness (timestamp + SHA), relevant ADR paths, key findings from TDD planning.
**Done:** all five captured.

### 2 — Append TDD plan to issue

Locate `<scratch-issue-path>` and append the TDD plan. Then `git add` only that file and invoke `caveman-commit` to stage and commit it with an auto-generated message.

**Done:** issue file committed with TDD plan — only that file.

### 3 — Write tight handoff

Write the handoff doc directly. Save to OS temp dir (not workspace). Reference existing artifacts (PRDs, ADRs, plans, issues, commits, diffs) by path — don't duplicate. Redact secrets (API keys, passwords, PII).

Include in the doc:
- **Focus:** implementing the plan in `<scratch-issue-path>`.
- **Session state** from Step 1.
- **Suggested skills:** `using-git-worktrees`, `subagent-driven-development`, `rust-best-practices`, `rust-skills`, `tdd`, `gitnexus-exploring`, `gitnexus-impact-analysis`, `gitnexus-refactoring`, `gitnexus-debugging`.
- **Next Agent Instructions** block below (paths substituted).

**Done:** handoff doc written to temp dir, path known.

### 4 — Deliver

Present: (1) handoff path, (2) opening prompt with path substituted.
**Done:** user acknowledged.

## Opening Prompt

```
Read handoff at <handoff-doc-path>.
Invoke `using-git-worktrees`, `subagent-driven-development`, `rust-skills`, `rust-best-practices`, `tdd`, `gitnexus-impact-analysis`, `gitnexus-exploring`, `gitnexus-refactoring`, `gitnexus-debugging`.
Follow handoff instructions. No deviation from approved plan. Work only in worktree.
```

## Next Agent Instructions

Copy into handoff doc (substitute paths).

**Objectives:**
1. Implement the approved plan for `<scratch-issue-path>` in a dedicated worktree.
2. Fulfill all acceptance criteria.
3. Validate approach against the codebase via GitNexus + `rust-skills` + `rust-best-practices` + `tdd`.

**Process:**
1. Use `using-git-worktrees` to create a dedicated worktree.
2. Review issue, ACs, plan, and relevant code.
3. Found blockers? Present for review; do not implement until resolved.
4. Implement via `subagent-driven-development` + above skills.

**Constraints:** no deviation from approved plan without approval.
