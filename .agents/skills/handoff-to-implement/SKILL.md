---
name: handoff-to-implement
description: Orchestrates structured handoff from TDD planning to implementation. Invokes the handoff skill to compact the session, then produces a ready-to-use opening prompt for the next agent containing worktree creation, validation, and implementation instructions using GitNexus, rust-best-practices, tdd, and subagent-driven-development skills. Use when the TDD plan is approved and the issue is ready for implementation. Triggered by "handoff to implement", "create handoff for implementation", "ready for next agent", "handoff to implement", or after completing TDD planning.
argument-hint: "<scratch-issue-path>"
---

# Handoff to Implementation

## Argument

One positional argument:

| # | Argument | Example |
|---|----------|---------|
| 1 | scratch-issue-path | `.scratch/feature/ISSUE-42.md` |

## Workflow

### Step 0 — Pre-flight: ensure GitNexus index is fresh

Before creating the handoff, verify the GitNexus index is current:

1. Read `gitnexus://repo/lithos/context` to check index freshness (last indexed date vs HEAD).
2. If stale, run `.gitnexus/run.cjs analyze` or `npx gitnexus analyze` from the project root.
3. Confirm the index is up to date before proceeding.

### Step 1 — Gather session state

Collect factual state to seed the handoff:

- Current branch name (`git branch --show-current`).
- Whether there are uncommitted changes (`git status --porcelain`).
- GitNexus index freshness confirmation (timestamp and HEAD SHA).
- Any relevant ADRs discovered during this session (paths only).
- Key findings from TDD planning that the next agent should know.

### Step 2 — Create handoff document

Invoke the `handoff` skill with argument:

> Handoff for implementing the plan in `<scratch-issue-path>`

Include the following in the handoff document:
- **Session state** — branch, uncommitted changes, index freshness, ADR paths.
- A "Suggested skills" section listing: `using-git-worktrees`, `subagent-driven-development`, `rust-best-practices`, `tdd`, and relevant `gitnexus-*` skills.
- The **Next Agent Instructions** block below (substituted with actual paths).
- Any additional context, findings, recommendations, or guidance relevant to the task.

### Step 3 — Present deliverables

Report to the user:
1. **Handoff document path** — path returned by the handoff skill.
2. **Opening prompt** — the template below with `<handoff-doc-path>` substituted.

## Opening Prompt Template

Substitute `<handoff-doc-path>` and present to the user as a ready-to-copy block.

---

Read the handoff document at `<handoff-doc-path>`.

Use the Skill tool to invoke: `using-git-worktrees`, `subagent-driven-development`, `rust-best-practices`, `tdd`, `gitnexus-impact-analysis`, `gitnexus-exploring`.

Follow the instructions in the handoff document. Do not deviate from the approved implementation plan without approval.

---

## Next Agent Instructions

Copy into the handoff document with `<scratch-issue-path>` replaced. These are the minimum required instructions — include any additional context, findings, recommendations, or guidance relevant to the task.

---

Your deliverables:
1. Implement the approved plan in a dedicated worktree.
2. Fulfill all acceptance criteria.

Process:
1. `use_skill "using-git-worktrees"` — create a dedicated worktree for `<scratch-issue-path>`.
2. Review the issue, acceptance criteria, approved implementation plan, and relevant code.
3. Use GitNexus (query, context, impact), `rust-best-practices`, and `tdd` skills to validate the implementation approach against the existing codebase.
4. If no blockers are found, implement using `use_skill "subagent-driven-development"` together with the above skills.

Review requirements:
- Verify the issue, acceptance criteria, implementation plan, and codebase remain consistent.
- Identify any ambiguities, inconsistencies, dependencies, implementation risks, or unaddressed side effects.
- If any blockers are identified, present them for review and do not proceed until resolved.

Implementation requirements:
- Treat the approved implementation plan as the source of truth.
- Fulfill all acceptance criteria.
- All agent and subagent work occurs only in the dedicated worktree.

Constraints:
- Do not deviate from the approved implementation plan without approval.
