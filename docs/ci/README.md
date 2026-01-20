# CI/CD Pipeline Guide

## Overview
The Lithos CI pipeline is managed via GitHub Actions and orchestrated locally using **mise**. It is designed for fast feedback, security, and flakiness detection.

## Pipeline Stages
1. **Detect Changes**: Optimizes the run by identifying which crates changed.
2. **Quality Gates**: Runs `mise run quality` (fmt, lint, ADR validation).
3. **Secrets Detection**: Scans for leaked credentials using Gitleaks.
4. **Test**:
   - **Pull Requests**: Runs `mise run test:changed` to test only affected code.
   - **Main/Ref**: Runs the full `mise run test` suite across OS matrix (Ubuntu, macOS, Windows).
5. **Burn-In**: Runs 10 iterations of the test suite on PRs and weekly schedules to detect non-deterministic (flaky) failures.
6. **Coverage**: Generates and uploads code coverage reports.
7. **Security**: Dependency auditing via `cargo deny`.
8. **Cross-Compile Check**: Ensures compilation works for `wasm32` and `linux-gnu`.

## Local Execution
You can simulate the CI environment locally using these mise tasks:

- **Run affected tests only**: `mise run test:changed`
- **Run burn-in loop**: `mise run test:burn-in 5` (runs default test suite 5 times)
- **Full verification**: `mise run verify` (lint + full test suite)

## Debugging Failures
- **Artifacts**: Check the "Summary" page of a failed GitHub Action run to download `test-results` (nextest XML) and `coverage-report`.
- **Traces**: Nextest outputs logs for failed tests; review the "Run Tests" step output in GitHub.
