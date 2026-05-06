# Story 1.2: Set Up Base Pre-Commit and Shell Quality

Status: done

## Story

As a developer committing code,
I want automatic quality checks for my shell scripts and project hygiene,
so that the development environment is consistent and verified from the first commit.

## Acceptance Criteria

### 1. Framework Configuration
- [x] **Given** a Git repository
- [x] **When** I check `.pre-commit-config.yaml`
- [x] **Then** the following foundational hooks are configured with **Latest Stable Versions**:
  - `check-added-large-files` (Prevent blobs)
  - `check-case-conflict` (Cross-platform safety)
  - `check-executables-have-shebangs` (Script integrity)
  - `check-merge-conflict` (Hygiene)
  - `check-symlinks` (Integrity)
  - `end-of-file-fixer` (Hygiene)
  - `trailing-whitespace` (Hygiene)
  - `mixed-line-ending` (Hygiene)
  - `gitleaks` (Secret scanning using project's `.gitleaks.toml`)

### 2. Rust Quality Gates (MANDATORY)
- [x] **Given** a Rust workspace
- [x] **When** I commit code
- [x] **Then** it must pass these stringent quality gates:
  - **fmt**: `cargo fmt --all -- --check` (Verified via `doublerebel/pre-commit-rust`).
  - **clippy**: `cargo clippy --all-targets --all-features -- -D warnings` (Verified via `doublerebel/pre-commit-rust`).
  - **test**: `cargo test --workspace` (Verified via `doublerebel/pre-commit-rust`).

### 3. Shell Quality Gates (MANDATORY)
- [x] **Given** I am writing shell scripts for task orchestration
- [x] **When** I commit shell code
- [x] **Then** it must pass these specific quality gates:
  - **shfmt**: Formatted according to **Google Shell Style** (`-i 2 -ci`).
  - **shellcheck**: No warnings or errors (Verified via `shellcheck-py/shellcheck-py`).

### 4. Installation & Verification
- [x] **Given** `.pre-commit-config.yaml` is updated
- [x] **When** I run `pre-commit install` and `pre-commit run --all-files`
- [x] **Then** the hooks are successfully installed and the current codebase passes all checks.

### 5. Quality Discipline
- [x] **Given** the hooks are installed and verified
- [x] **When** I stage and commit the configuration files
- [x] **Then** all hooks must pass automatically.
- [x] **MANDATORY**: The `--no-verify` flag must NEVER be used.
- [x] **MANDATORY**: Commits follow **Conventional Commits** style (e.g., `feat(env): ...`).

## Tasks / Subtasks

- [x] Clear legacy Go-based `.pre-commit-config.yaml` content
- [x] Add `pre-commit/pre-commit-hooks` (v6.0.0) with comprehensive hygiene list
- [x] Add `gitleaks/gitleaks` (v8.30.0) for secret scanning
- [x] Add `shellcheck-py/shellcheck-py` for automated shell linting
- [x] Add `https://github.com/mvdan/sh` for `shfmt` (Google style: `-i 2 -ci`)
- [x] Add `https://github.com/doublerebel/pre-commit-rust` for workspace-wide Rust checks:
  - [x] `fmt`
  - [x] `clippy`
  - [x] `cargo-test`
- [x] Run `pre-commit autoupdate` to lock in latest versions
- [x] Run `pre-commit install` to activate the hooks
- [x] **Verification**: Run `pre-commit run --all-files` to ensure existing files comply
- [x] **Validation**: Create a dummy shell script that violates style to verify hooks block the commit
- [x] **Finalize**: Stage all environment configuration files (`.pre-commit-config.yaml`, `.gitleaks.toml`, etc.)
- [x] **Commit**: Create final commit: `feat(env): establish high-integrity pre-commit quality gates`
  - [x] **MANDATORY**: Hook execution must succeed; do NOT use `--no-verify`.

## Dev Notes

- **Google Style Integration**: This story ensures that all subsequent stories involving scripts (like Story 1.3 Mise orchestration) are automatically held to the highest quality standards.
- **Rust Workspace Awareness**: Ensure `cargo` hooks are configured to run against the full workspace (`--all` / `--workspace`).
- **Gitleaks Precision**: The hook must leverage the existing `.gitleaks.toml` which contains project-specific exclusions for documentation and test data.
- **No-Verify Policy**: We are establishing a "Broken Window" policy early—quality gates are not optional.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.2]
- [Source: pre-commit Documentation (https://pre-commit.com/)]
- [Source: Google Shell Style Guide]

### Dev Agent Record

### Agent Model Used

Gemini 2.0 Flash Thinking 01-21 (Implementation) / Adversarial Reviewer (Refinement)

### Debug Log References

### Completion Notes List

- Established comprehensive pre-commit quality gates for Rust, Shell, and Project Hygiene.
- Configured hooks: `check-added-large-files`, `check-case-conflict`, `check-executables-have-shebangs`, `check-merge-conflict`, `check-symlinks`, `end-of-file-fixer`, `trailing-whitespace`, `mixed-line-ending`, `gitleaks`, `shellcheck`, `shfmt`.
- Implemented Rust quality gates via local hooks: `cargo fmt`, `cargo clippy`, `cargo test` (workspace-wide).
- **Refinement (Adversarial Review)**:
    - Set `pedantic`, `nursery`, and `missing_docs` lints to `deny` in `Cargo.toml` for true "stringent" quality enforcement.
    - Added `conventional-pre-commit` hook to enforce Conventional Commits mandatory requirement (AC 5).
    - Switched to `scop/pre-commit-shfmt` for managed shell formatting (Task 1.2).
    - Purged legacy Go-specific allowlist rules and "FlowForge" branding from `.gitleaks.toml`.
    - Resolved `clippy::lint_groups_priority` errors in `Cargo.toml`.
    - Verified all hooks pass at `deny` level across the workspace.

### File List

- `.pre-commit-config.yaml`
- `Cargo.toml` (workspace root)
- `.gitleaks.toml`
- `_bmad-output/implementation-artifacts/stories/1-2-set-up-base-pre-commit-and-shell-quality.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
