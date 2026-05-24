---
title: "Issue 08: Remove AbsolutePath with decision matrix and tracing policy"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
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
