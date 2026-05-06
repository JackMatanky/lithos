# CI/CD Configuration and Maintenance Guide

This document details the CI/CD pipeline configuration for the Lithos project.

## Overview

The CI/CD pipeline is implemented using GitHub Actions and orchestrated via `mise` tasks. It follows a multi-stage architecture to ensure comprehensive quality assurance.

## Pipeline Architecture

The pipeline consists of the following stages:

1.  **Quality Gates**: Parallel execution of formatting checks and linting (clippy).
2.  **Test Matrix**: Concurrent testing across multiple operating systems (Ubuntu, macOS, Windows) and Rust versions (stable, beta, nightly).
3.  **Security Scan**: Automated dependency auditing and vulnerability scanning using `cargo-deny`.
4.  **Coverage Report**: Code coverage analysis using `tarpaulin`.
5.  **Performance Benchmarks**: Regression detection for performance-critical paths using `criterion`.
6.  **Deployment Readiness**: Final gate that confirms all previous stages succeeded.

## Workflow Configuration

The primary workflow is defined in `.github/workflows/ci.yml`.

### Key Components

- **mise-action**: Ensures tool version parity between local development and CI.
- **rust-cache**: Optimizes build times by caching Cargo dependencies and target directories.
- **nextest**: Used for high-performance concurrent test execution.
- **cargo-deny**: Handles security and license compliance.
- **criterion-compare**: Detects performance regressions in PRs.

## Maintenance Procedures

### Updating Tool Versions

Tool versions (Rust, etc.) are managed in `mise.toml`. Updating them there will automatically update the CI environment.

### Cache Invalidation

Caches are keyed by `Cargo.lock` and `mise.toml`. If you encounter build issues related to stale caches, pushing a change to these files or manually clearing caches in the GitHub UI will resolve it.

### Adding New Jobs

New jobs should be added to `ci.yml` and integrated into the `deployment-readiness` dependency tree to ensure they are treated as required quality gates.

## Troubleshooting

- **Matrix Failures**: Check if the failure is platform-specific (e.g., Windows-only path issues) or Rust-version specific (e.g., nightly-only features).
- **Security Alerts**: `cargo-deny` failures indicate a new vulnerability or a forbidden license. Review the SARIF report in the GitHub Security tab.
- **Performance Regressions**: Review the PR comment from the `criterion-compare` action to identify which benchmarks regressed.

## Branch Protection

The following status checks are required for merging:
- `Quality Gates`
- `Test`
- `Security Scan`
- `Coverage Report`
- `Performance Benchmarks` (for PRs)
- `Deployment Readiness`
