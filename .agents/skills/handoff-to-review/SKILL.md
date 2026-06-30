---
name: handoff-to-review
disable-model-invocation: true
description: Compact session into a tight opening prompt for adversarial review. User-invoked.
argument-hint: "<scratch-issue-path>"
---

# Handoff to Review

Scratch folder = `dirname(<scratch-issue-path>)`.

## Workflow

### 0 — Freshen index

Check GitNexus index freshness (last indexed timestamp vs HEAD). Stale? Run `.gitnexus/run.cjs analyze`.
**Done:** index matches HEAD commit.

### 1 — Gather state

Record: current branch, uncommitted changes (`git status --porcelain`), index freshness (timestamp + SHA), relevant ADR paths, the worktree path where implementation happened, key results and findings from implementation.

**Done:** all captured.

### 2 — Write tight handoff

Write the handoff doc directly. Save to OS temp dir (not workspace). Reference existing artifacts (PRDs, ADRs, plans, issues, commits, diffs) by path — don't duplicate. Redact secrets (API keys, passwords, PII).

Include in the doc:
- **Focus:** adversarial review of implementation for `<scratch-issue-path>`.
- **Worktree path** where implementation lives.
- **Session state** from Step 1.
- **Suggested skills:** `adversarial-review`, `rust-best-practices`, `gitnexus-exploring`, `gitnexus-impact-analysis`.
- **Next Agent Instructions** block below (paths substituted).

**Done:** handoff doc written to temp dir, path known.

### 3 — Deliver

Present: (1) handoff path, (2) opening prompt with path substituted.
**Done:** user acknowledged.

## Opening Prompt

```
Read handoff at <handoff-doc-path>.
Invoke `adversarial-review`, `rust-best-practices`, `gitnexus-impact-analysis`, `gitnexus-exploring`.
Follow handoff instructions. Work only in worktree.
```

## Next Agent Instructions

Copy into handoff doc (substitute paths).

**Objectives:**
1. Run `adversarial-review` against the implementation at the worktree path.
2. Surface all gaps, rust-best-practices violations, test gaps, and doc gaps.
3. Report organized findings.

**Process:**
1. Switch to the worktree path from this handoff doc.
2. Invoke `adversarial-review` — follow its workflow.
3. Present complete findings for approval.

**Constraints:** no code changes, no deviation from approved plan.
