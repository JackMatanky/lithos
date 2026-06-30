---
name: handoff-to-tdd
disable-model-invocation: true
description: Compact session into a tight opening prompt for TDD planning. User-invoked.
argument-hint: "<issue-file-path>"
---

# Handoff to TDD

Scratch folder = `dirname(<issue-file-path>)`.

## Workflow

### 0 — Freshen index

Check GitNexus index freshness (last indexed timestamp vs HEAD). Stale? Run `.gitnexus/run.cjs analyze`.
**Done:** index matches HEAD commit.

### 1 — Gather state

Record: current branch, uncommitted changes (`git status --porcelain`), index freshness (timestamp + SHA), relevant ADR paths.
**Done:** all four captured.

### 2 — Write tight handoff

Write the handoff doc directly. Save to OS temp dir (not workspace). Reference existing artifacts (PRDs, ADRs, plans, issues, commits, diffs) by path — don't duplicate. Redact secrets (API keys, passwords, PII).

Include in the doc:
- **Focus:** TDD planning for `<issue-file-path>` in context of `<scratch-folder-path>`.
- **Session state** from Step 1.
- **Suggested skills:** `rust-best-practices`, `tdd`, `gitnexus-exploring`, `gitnexus-impact-analysis`.
- **Next Agent Instructions** block below (paths substituted).

**Done:** handoff doc written to temp dir, path known.

### 3 — Deliver

Present: (1) handoff path, (2) opening prompt with path substituted.
**Done:** user acknowledged.

## Opening Prompt

```
Read handoff at <handoff-doc-path>.
Invoke `rust-best-practices`, `tdd`, `gitnexus-impact-analysis`, `gitnexus-exploring`.
Follow handoff instructions. No issue edits. No code. Present findings + plan.
```

## Next Agent Instructions

Copy into handoff doc (substitute paths).

**Objectives:**
1. Review `<issue-file-path>` in context of `<scratch-folder-path>`.
2. Find gaps, risks, side effects. Found? Present for review; no planning until resolved.
3. Produce TDD plan via GitNexus + `rust-best-practices` + `tdd`.

**Plan must:** specify all changes + impact, tests + coverage, follow `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

**Constraints:** no issue edits, no code, present findings + plan for approval.
