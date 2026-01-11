# Story 1.2: Configure mise.toml for Task Orchestration

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer working on the project,
I want comprehensive mise tasks for development workflows,
so that I can efficiently run tests, benchmarks, formatting, and other development tasks.

## Acceptance Criteria

### 1. Task Availability
- **Given** mise is installed in the project
- **When** I run `mise run --list` or check `mise.toml`
- **Then** the following tasks are available:
  - `mise run test` - Run all tests
  - `mise run test:unit` - Domain layer unit tests only
  - `mise run test:integration` - Cross-crate integration tests
  - `mise run test:coverage` - Generate coverage report with tarpaulin
  - `mise run test:watch` - Watch mode for TDD development
  - `mise run bench` - Run performance benchmarks
  - `mise run fmt` - Format all code
  - `mise run lint` - Run clippy linting
  - `mise run verify` - Full quality gate (fmt + lint + test)

### 2. Best Practices Implementation
- **Given** I have researched Rust project best practices for task orchestration
- **When** I review the `.mise/tasks/` configuration
- **Then** tasks follow these mandatory standards:
  - **Mise Native:** All tasks are implemented as standalone executable scripts in `.mise/tasks/`.
  - **Google Style:** All shell scripts follow the **Google Shell Style Guide** (error handling, function structure, naming, and mandatory function comments for multi-function scripts).
  - **Task Metadata:** Each script uses `#MISE` comments for configuration and `#USAGE` for advanced argument parsing.
  - **Versioning:** Tool versions are pinned (Rust 1.92+, clippy, rustfmt versions).
  - **Safety:** Proper shell escaping and cross-platform compatibility are ensured.

### 3. Pipeline Verification
- **Given** mise tasks are configured
- **When** I run `mise run verify`
- **Then** the full quality pipeline executes successfully

## Tasks / Subtasks

- [ ] Clear legacy Go-based `mise.toml` configuration (AC: 2)
- [ ] Configure `[tools]` section with Rust 1.92+ and essential dev tools (AC: 2)
  - [ ] Use `profile = "default"` to include `clippy` and `rustfmt`
  - [ ] Pin `cargo-tarpaulin`, `cargo-watch`, and `cargo-deny`
- [ ] Implement core development tasks as **Pure Script-Based Tasks** (AC: 1, 2)
  - [ ] Move ALL task logic to standalone executable files in `.mise/tasks/`
  - [ ] Ensure every script follows the **Google Shell Style Guide**
  - [ ] Use `usage` spec for advanced argument parsing in every script
  - [ ] Implement `sources` and `outputs` for build/test caching in script headers
- [ ] Establish directory-based task grouping (AC: 1)
  - [ ] Organize `.mise/tasks/test/` with `unit`, `integration`, and `coverage`
  - [ ] Ensure `mise.toml` is kept lean, containing only global tools and env vars
- [ ] Verify task descriptions and usage examples (AC: 2)
- [ ] Update `.pre-commit-config.yaml` with `shfmt` and `shellcheck` (AC: 2)
  - [ ] Add `https://github.com/mvdan/sh` for `shfmt`
  - [ ] Add `https://github.com/koalaman/shellcheck-precommit` for `shellcheck`
- [ ] Run `pre-commit install` and verify new hooks (AC: 2)
- [ ] Validate full pipeline with `mise run verify` (AC: 3)
- [ ] Stage and commit changes (AC: 2)
  - [ ] Use conventional commit message: `feat(env): configure mise task orchestration and shell quality hooks`
  - [ ] **MANDATORY**: Ensure all pre-commit hooks pass; NEVER use `--no-verify`.

## Dev Notes

- **Architecture Compliance**: This story ensures that all architectural quality gates defined in `architecture.md` (cognitive complexity, import sorting) are easily executable via a single tool (`mise`).
- **Pure Script-Based Tasks**: **MANDATORY**: All tasks must be executable files in `.mise/tasks/`. The `mise.toml` file must NOT contain a `[tasks]` section.
- **Google Shell Style Guide**: **MANDATORY**:
  - Use `#!/usr/bin/env bash`.
  - Prefer functions over raw script logic; include a `main()` function.
  - **Function Comments**: If a script contains more than one function, each must have a Google-style comment block (Description, Globals, Arguments, Outputs).
  - Implement strict error handling: `set -e`, `set -u`, `set -o pipefail`.
  - Use `local` for variables inside functions.
- **Shell Quality**: **MANDATORY**: Scripts must pass `shellcheck` and be formatted with `shfmt`.
- **Modern Argument Parsing**: **MANDATORY**: Use the `usage` spec (e.g., `#USAGE flag "-v --verbose"`) for all task arguments. Do NOT use Tera templates (`{{arg()}}`) as they are deprecated and scheduled for removal in mise 2026.11.0.

...

### Elite mise Implementation Patterns

- **Tool Configuration (mise.toml)**:
  ```toml
  [tools]
  rust = { version = "1.92", profile = "default" }
  "cargo:cargo-tarpaulin" = "latest"
  "cargo:cargo-watch" = "latest"
  "cargo:cargo-deny" = "latest"
  ```
- **Standard Script Header (`.mise/tasks/test/unit`)**:
  ```bash
  #!/usr/bin/env bash
  #MISE description="Run unit tests for domain layer"
  #MISE sources=["crates/domain/src/**/*.rs"]
  #MISE outputs={auto=true}
  #USAGE flag "-v --verbose" help="Verbose output"

  set -euo pipefail

  #######################################
  # Execute unit tests for the domain crate.
  # Globals:
  #   usage_verbose
  # Arguments:
  #   None
  # Outputs:
  #   Writes test results to stdout
  #######################################
  run_domain_tests() {
    local verbose_flag=""
    if [[ "${usage_verbose:-false}" == "true" ]]; then
      verbose_flag="--verbose"
    fi
    cargo test -p lithos-domain ${verbose_flag}
  }

  main() {
    run_domain_tests
  }

  main "$@"
  ```

### Previous Story Intelligence (Story 1.1)

- **Learnings**: Story 1.1 established the 4-crate workspace structure. `mise` tasks must now target these crates correctly.
- **Patterns**: Use `[workspace.dependencies]` pattern from 1.1 to ensure consistency, but for `mise`, focus on the `[tools]` and `[tasks]` sections to orchestrate these workspace-aware commands.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.2]
- [Source: _bmad-output/implementation-artifacts/stories/1-1-initialize-cargo-workspace-structure.md]
- [Source: mise Documentation (https://mise.jdx.dev/)]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
