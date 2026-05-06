---
title: "Lithos CI/CD Guide"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
source_of_truth:
  - ".github/workflows/ci.yml"
  - "mise.toml"
scope: "Canonical CI/CD policy and execution guide"
supersedes:
  - "docs/ci-cd.md"
  - "docs/ci/README.md"
  - "docs/ci/ANALYSIS.md"
  - "docs/ci/IMPROVEMENTS.md"
  - "docs/ci/MONITORING.md"
  - "docs/ci/secrets-checklist.md"
---

# Lithos CI/CD Guide

This is the canonical CI/CD document for Lithos.

## Purpose

- Define the active CI/CD pipeline behavior and quality gates.
- Keep local and CI workflows aligned through `mise` tasks.
- Provide a single place to debug failures and verify merge readiness.

## Pipeline Contract

The pipeline runs in GitHub Actions and is task-driven via `mise`.

Core stages:

1. Detect changes (optimize what runs)
2. Quality gates (`fmt`, `lint`, ADR validation)
3. Tests (changed-only on PRs; full suite where required)
4. Coverage reporting
5. Security checks (`cargo deny`, secret scanning)
6. Optional burn-in / benchmark gates (workflow-dependent)
7. Deployment-readiness aggregate gate

## Pre-commit Policy and CI Relationship

Pre-commit is a mandatory local quality gate and the first enforcement layer before CI.

- Local expectation: contributors run (and pass) pre-commit checks before push.
- CI expectation: workflow gates replicate and enforce the same policy categories so branch protection does not rely on local discipline alone.

Pre-commit and CI are complementary:

- **Pre-commit** optimizes fast local feedback and blocks low-quality commits.
- **CI** provides authoritative remote enforcement and required status checks for merge.

If pre-commit and CI disagree, treat this as a configuration drift bug and align them immediately.

Typical pre-commit-enforced categories in this repository include:

- hygiene checks (line endings, merge conflicts, trailing whitespace, etc.)
- secret scanning
- formatting/lint gates
- commit policy checks (for example conventional commit validation where configured)

## Required Quality Gates

All merges must satisfy the configured required checks in GitHub branch protection.

At minimum, CI must prove:

- Formatting and linting pass
- Test suite policy passes for the event type (PR/full)
- Security checks pass
- Final aggregate gate passes

## Local-to-CI Parity (Mise First)

Run the same tasks locally before pushing:

```bash
# full local preflight
mise run verify

# quality only
mise run quality

# changed tests (PR-aligned)
mise run test:changed

# full tests
mise run test

# coverage
mise run test:coverage

# flaky detection
mise run test:burn-in
```

## Toolchain and Components

- CI and local development are expected to use the toolchain configured in repo-level toolchain/task files.
- If CI fails on formatting component availability, ensure required Rust components (for example `rustfmt`, `clippy`) are installed in CI job setup.
- Treat toolchain mismatches as configuration bugs, not test failures.

## Secrets Policy

- `GITHUB_TOKEN` is the only required default CI token.
- Additional secrets are optional and must be justified by a specific workflow integration.
- Apply least privilege and regular rotation for any added secret.

## Failure Triage

When a run fails:

1. Identify first failing stage (not downstream cascades).
2. Confirm local reproduction using matching `mise` task.
3. Classify failure type:
   - toolchain/config
   - test regression
   - security/policy
   - infra/transient
4. Fix root cause, then rerun relevant stage(s).

## Governance

- This file is authoritative for CI/CD process.
- Historical run analyses and one-off incident notes belong in `docs/history/`, not in active engineering docs.
- Do not create parallel CI guides; update this file instead.
