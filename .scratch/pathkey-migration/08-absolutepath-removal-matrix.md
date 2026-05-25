---
title: "Issue 08: Remove AbsolutePath with decision matrix and tracing policy"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
date_completed: null
---

# Issue 08: Remove AbsolutePath with decision matrix and tracing policy

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Remove `AbsolutePath` from production flows, replacing with `DirPath`/`FilePath`, while explicitly classifying each replacement as hard error, warning+continue, or trace-only.

## Agent Brief

**Category:** enhancement
**Summary:** Remove `AbsolutePath` usage with explicit severity-policy mapping and structured tracing.

**Current behavior:**
`AbsolutePath` exists alongside `DirPath`/`FilePath`, creating overlapping semantics and unclear handling policies when resolution fails.

**Desired behavior:**
`AbsolutePath` is completely excised. All instances are replaced by `DirPath` or `FilePath`. For operations that previously panicked or ambiguously failed, an explicit matrix dictates fallback severity:
- **Boundary/security checks**: Hard error.
- **Optional/discovery features**: Downgrade to warning + continue.
- **Low-value/noise**: Downgrade to trace-only.

**Key interfaces:**
- Internal `fs` boundary signatures previously taking `AbsolutePath`.
- Structured logging points invoking `tracing::warn!` or `tracing::trace!`.
- `TrustedVaultPath` (migrate from `AbsolutePath` wrapper to thin newtype over `DirPath`).

**Acceptance criteria:**
- [ ] `AbsolutePath` is completely deleted from the codebase.
- [ ] `TrustedVaultPath` wraps `DirPath`.
- [ ] Every replacement is documented via comments or commit logs highlighting the severity choice (Error vs Warn vs Trace).
- [ ] Downgraded checks utilize structured `tracing` fields: `context`, `root`, `path`, `decision`.
- [ ] No panic regressions introduced during replacement.

**Out of scope:**
- Deletion of `RelativePath`.

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- Explicitly map operational downgrades using structured tracing instead of panics or silent failures.

**Behaviors to Test (Prioritized):**
1. Hard boundary checks map invalid paths to formal `Result::Err`.
2. Optional feature failures emit structured downgrade traces instead of failing.

### 2. Tracer Bullet: Hard Error Replacement
**Behavior:** Hard boundary checks map invalid paths to formal `Result::Err`.
- **RED:** Call a strictly required boundary check with an invalid path. Assert it returns a mapped error.
- **GREEN:** Replace `AbsolutePath` with `DirPath`/`FilePath`. Return `Result` instead of panic.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Tracing/Downgrade
**Behavior:** Optional feature failures emit structured downgrade traces instead of failing.
- **RED:** Using `tracing-test`, assert an optional resolution failure emits a `tracing::warn!` or `tracing::trace!` event with the appropriate `decision` context.
- **GREEN:** Implement structured `tracing` fields for downgraded `AbsolutePath` checks.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 4. Refactor
- [ ] Document all downgraded checks with `//` comments explaining *why* the severity was chosen (Rust Best Practice: Comments explain why).
