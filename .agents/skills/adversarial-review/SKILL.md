---
name: adversarial-review
disable-model-invocation: true
description: Adversarial review of an approved implementation. User-invoked.
argument-hint: "<handoff-doc-path>"
---

# Adversarial Review

Adversarial stance: assume gaps exist, hunt them.

## Workflow

### 1 — Enter worktree from handoff

Read `<handoff-doc-path>` to find the worktree and review context. Switch to the worktree. Verify working directory is inside it before proceeding.

**Done:** in the correct worktree.

### 2 — Surface implementation gaps

Review the implementation against the issue, acceptance criteria, and approved plan. Surface missing behavior, incomplete edge cases, unhandled errors.

**Done:** gaps documented.

### 3 — Audit rust-best-practices

Invoke `rust-best-practices`. Review every component against the skill's rules. Surface every shortcoming.

**Done:** each component scored, shortcomings listed.

### 4 — Audit tests

Read `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`. Review tests for:
- Coverage of desired behavior
- Adherence to codebase standards (naming, placement, patterns, assertions)
- Adherence to `rust-best-practices`

**Done:** test compliance gaps documented.

### 5 — Audit doc comments

Review module-level and public-component doc comments against:
- `rust-best-practices` conventions
- https://doc.rust-lang.org/rustdoc/ conventions
- Accuracy and completeness

**Done:** doc comment gaps documented.

### 6 — Report

Present organized findings: implementation gaps, rust-best-practices violations, test gaps, doc gaps.

**Done:** findings presented to user.
