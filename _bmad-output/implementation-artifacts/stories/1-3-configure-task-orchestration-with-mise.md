# Story 1.3: Configure Task Orchestration with Mise

Status: backlog

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer working on the project,
I want comprehensive mise tasks for development workflows implemented as pure scripts,
so that I can efficiently run tests, benchmarks, formatting, and other development tasks with full IDE support and automated verification.

## Acceptance Criteria

### 1. Task Availability
- **Given** mise is installed in the project
- **When** I run `mise run --list`
- **Then** the following tasks are available as pure scripts in `.mise/tasks/`:
  - `test`, `test:unit`, `test:integration`
  - `test:coverage` (tarpaulin)
  - `test:watch` (cargo-watch)
  - `bench` (criterion)
  - `fmt` (rustfmt)
  - `lint` (clippy)
  - `doc` (generate and open crate docs)
  - `verify` (Full quality gate orchestration)

### 2. Best Practices Implementation
- **Given** I am writing task scripts
- **When** I review the `.mise/tasks/` implementation
- **Then** they follow these mandatory standards:
  - **Mise Native:** All tasks are standalone executable scripts in `.mise/tasks/`.
  - **Google Style:** All shell scripts follow the **Google Shell Style Guide** (strict error handling, `main()` function, and mandatory function comments for multi-function scripts).
  - **Task Metadata:** Scripts use `#MISE` headers for caching/descriptions and `#USAGE` for advanced argument parsing.
  - **Efficiency:** Scripts implement `sources` and `outputs` caching where applicable.

### 3. Pipeline Verification
- **Given** mise tasks are configured
- **When** I run `mise run verify`
- **Then** the full quality pipeline executes successfully, verified by the pre-commit hooks established in Story 1.2.

## Tasks / Subtasks

- [ ] Clear legacy Go-based `mise.toml` configuration
- [ ] Configure `[tools]` section in `mise.toml` with Rust 1.92+ and essential dev tools
  - [ ] Use `profile = "default"` to include `clippy` and `rustfmt`
  - [ ] Pin `cargo-tarpaulin`, `cargo-watch`, and `cargo-deny`
- [ ] Implement core development tasks as **Pure Script-Based Tasks** in `.mise/tasks/`
  - [ ] Implement `test` hierarchy (unit, integration, coverage) using directory grouping
  - [ ] Implement quality tasks (`fmt`, `lint`, `doc`)
  - [ ] Implement orchestration task (`verify`) using `#MISE depends`
- [ ] Ensure all scripts pass `shellcheck` and `shfmt` (via hooks from 1.2)
- [ ] Validate full pipeline with `mise run verify`
- [ ] Stage and commit changes
  - [ ] Use conventional commit message: `feat(env): implement high-performance task orchestration with mise`
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
- **Modern Argument Parsing**: **MANDATORY**: Use the `usage` spec (e.g., `#USAGE flag "-v --verbose"`) for all task arguments.
- **Task Grouping**: Leverage directory-based grouping (e.g., `.mise/tasks/test/unit`) which automatically generates colon-separated names (`test:unit`).
- **Caching & Efficiency**: Use `sources` (e.g., `src/**/*.rs`, `Cargo.toml`) and `outputs` (or `outputs = { auto = true }`) to prevent redundant test/build executions.

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

### Previous Story Intelligence (Story 1.1 & 1.2)

- **Learnings**: Story 1.1 established the workspace structure. Story 1.2 established the shell quality hooks.
- **Integrity**: Task scripts created here MUST pass the `shellcheck` and `shfmt` hooks established in 1.2.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.3]
- [Source: _bmad-output/implementation-artifacts/stories/1-1-initialize-cargo-workspace-structure.md]
- [Source: _bmad-output/implementation-artifacts/stories/1-2-set-up-base-pre-commit-and-shell-quality.md]
- [Source: mise Documentation (https://mise.jdx.dev/)]
- [Source: Google Shell Style Guide]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
