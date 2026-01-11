# Story 1.2: Set Up Base Pre-Commit and Shell Quality

Status: ready-for-dev

## Story

As a developer committing code,
I want automatic quality checks for my shell scripts and project hygiene,
so that the development environment is consistent and verified from the first commit.

## Acceptance Criteria

### 1. Framework Configuration
- **Given** a Git repository
- **When** I check `.pre-commit-config.yaml`
- **Then** the following foundational hooks are configured with **Latest Stable Versions**:
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
- **Given** a Rust workspace
- **When** I commit code
- **Then** it must pass these stringent quality gates:
  - **fmt**: `cargo fmt --all -- --check` (Verified via `doublerebel/pre-commit-rust`).
  - **clippy**: `cargo clippy --all-targets --all-features -- -D warnings` (Verified via `doublerebel/pre-commit-rust`).
  - **test**: `cargo test --workspace` (Verified via `doublerebel/pre-commit-rust`).

### 3. Shell Quality Gates (MANDATORY)
- **Given** I am writing shell scripts for task orchestration
- **When** I commit shell code
- **Then** it must pass these specific quality gates:
  - **shfmt**: Formatted according to **Google Shell Style** (`-i 2 -ci`).
  - **shellcheck**: No warnings or errors (Verified via `shellcheck-py/shellcheck-py`).

### 4. Installation & Verification
- **Given** `.pre-commit-config.yaml` is updated
- **When** I run `pre-commit install` and `pre-commit run --all-files`
- **Then** the hooks are successfully installed and the current codebase passes all checks.

### 5. Quality Discipline
- **Given** the hooks are installed and verified
- **When** I stage and commit the configuration files
- **Then** all hooks must pass automatically.
- **MANDATORY**: The `--no-verify` flag must NEVER be used.
- **MANDATORY**: Commits follow **Conventional Commits** style (e.g., `feat(env): ...`).

## Tasks / Subtasks

- [ ] Clear legacy Go-based `.pre-commit-config.yaml` content
- [ ] Add `pre-commit/pre-commit-hooks` (v6.0.0) with comprehensive hygiene list
- [ ] Add `gitleaks/gitleaks` (v8.30.0) for secret scanning
- [ ] Add `shellcheck-py/shellcheck-py` for automated shell linting
- [ ] Add `https://github.com/mvdan/sh` for `shfmt` (Google style: `-i 2 -ci`)
- [ ] Add `https://github.com/doublerebel/pre-commit-rust` for workspace-wide Rust checks:
  - [ ] `fmt`
  - [ ] `clippy`
  - [ ] `cargo-test`
- [ ] Run `pre-commit autoupdate` to lock in latest versions
- [ ] Run `pre-commit install` to activate the hooks
- [ ] **Verification**: Run `pre-commit run --all-files` to ensure existing files comply
- [ ] **Validation**: Create a dummy shell script that violates style to verify hooks block the commit
- [ ] **Finalize**: Stage all environment configuration files (`.pre-commit-config.yaml`, `.gitleaks.toml`, etc.)
- [ ] **Commit**: Create final commit: `feat(env): establish high-integrity pre-commit quality gates`
  - [ ] **MANDATORY**: Hook execution must succeed; do NOT use `--no-verify`.

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

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
