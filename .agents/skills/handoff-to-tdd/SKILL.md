---
name: handoff-to-tdd
description: Orchestrates structured handoff from issue triage to TDD implementation planning. Invokes the handoff skill to compact the session, then produces a ready-to-use opening prompt for the next agent containing review, gap analysis, and TDD plan generation instructions using GitNexus, rust-best-practices, and tdd skills. Use when user provides an issue file path and a scratch folder path for TDD planning, or says "handoff to tdd", "prepare for implementation", "create handoff for planning", "triage to tdd".
argument-hint: "<issue-file-path> <scratch-folder-path>"
---

# Handoff to TDD

## Arguments

Two positional arguments:

| # | Argument | Example |
|---|----------|---------|
| 1 | issue-file-path | `.scratch/feature/ISSUE-42.md` |
| 2 | scratch-folder-path | `.scratch/feature/` |

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

### Step 2 — Create handoff document

Invoke the `handoff` skill with argument:

> Handoff for TDD planning of `<issue-file-path>` in context of `<scratch-folder-path>`

Include the following in the handoff document:
- **Session state** — branch, uncommitted changes, index freshness, ADR paths.
- A "Suggested skills" section listing: `rust-best-practices`, `tdd`, and relevant `gitnexus-*` skills.
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

Use the Skill tool to invoke: `rust-best-practices`, `tdd`, `gitnexus-impact-analysis`, `gitnexus-exploring`.

Follow the instructions in the handoff document. Do not modify the issue file. Do not implement code. Present findings + plan for approval.

---

## Next Agent Instructions

Copy into the handoff document with `<issue-file-path>` and `<scratch-folder-path>` replaced. These are the minimum required instructions — include any additional context, findings, recommendations, or guidance relevant to the task.

---

**Objectives:**
1. Review `<issue-file-path>` in the context of the overall plan for `<scratch-folder-path>`.
2. Identify gaps, inconsistencies, implementation risks, dependencies, or unaddressed side effects.
   - If any found, present for review; do not proceed to planning until resolved.
3. Using GitNexus skills + Skill tool invocations of `rust-best-practices` and `tdd`, produce a comprehensive TDD plan.

**Plan requirements:**
- Cover all work required to satisfy acceptance criteria.
- Identify required codebase changes and their impacts.
- Define behaviors to verify, tests required, and required test coverage.
- Adhere to `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

**Constraints:**
- Do not modify the issue file.
- Do not implement code.
- Present findings and the complete plan for approval.
