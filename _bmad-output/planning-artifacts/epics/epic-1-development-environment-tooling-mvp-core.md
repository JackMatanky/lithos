# Epic 1: Development Environment & Tooling **[MVP CORE]**

Developers have a fully configured development environment with quality gates, testing infrastructure, and task orchestration that enforces architectural standards.
**FRs covered:** Architecture requirements (tooling, quality gates)
**Implementation Notes:**
- Cargo workspace structure (4 crates: domain, app, adapters, cli)
- mise.toml with task orchestration (test, bench, coverage, watch, etc.)
- pre-commit-config.yaml with stringent quality gates
- clippy.toml with cognitive complexity < 15 (warn) / 25 (deny) and all anti-pattern denies
- rustfmt.toml with import sorting and formatting standards
- deny.toml for dependency security auditing
- ADR review process and validation of existing ADRs (0001-0007)
- README.md with project overview and setup instructions
- Foundation for all subsequent epics

## Story 1.1: Initialize Cargo Workspace Structure

As a developer setting up the project foundation,
I want to create the Cargo workspace with 4 hexagonal architecture crates,
So that the project has clear separation between domain, application, infrastructure, and CLI layers.

**Acceptance Criteria:**

**Given** a new Rust project directory
**When** I run the workspace initialization commands
**Then** a Cargo workspace is created with the following structure:
```
lithos/
├── Cargo.toml (workspace configuration)
├── crates/
│   ├── domain/ (pure business logic, no I/O)
│   ├── app/ (application services and orchestration)
│   ├── adapters/ (infrastructure implementations)
│   └── cli/ (binary entry point)
└── Cargo.lock
```

**Given** the Cargo workspace structure exists
**When** I check the crate dependencies
**Then** the dependencies follow hexagonal boundaries:
- domain crate has no external dependencies
- app crate depends only on domain
- adapters crate depends on domain + external crates
- cli crate depends on app + adapters

**Given** the workspace is initialized
**When** I run `cargo check`
**Then** all crates compile without errors

## Story 1.2: Set Up Stringent Pre-Commit Hooks

As a developer committing code,
I want automatic quality checks before every commit,
So that code quality standards are enforced and poor code is caught early.

**Acceptance Criteria:**

**Given** pre-commit framework is configured
**When** I check .pre-commit-config.yaml
**Then** the hooks include these stringent quality gates:
- `clippy` with all configured lints
- `rustfmt` with import sorting verification
- `cargo test` for unit tests
- `cargo deny check` for dependency security

**Given** I have researched pre-commit best practices for Rust projects
**When** I review the configuration
**Then** the hooks follow these best practices:
- Hooks run in parallel where possible for speed
- Hooks fail fast on critical issues
- Hooks include clear error messages for failures
- Hooks respect .gitignore patterns

**Given** pre-commit hooks are installed
**When** I attempt to commit code that violates quality standards
**Then** the commit is blocked with clear error messages

**Given** pre-commit hooks are installed
**When** I commit properly formatted, tested code
**Then** the commit succeeds without delays

## Story 1.3: Configure mise.toml for Task Orchestration

As a developer working on the project,
I want comprehensive mise tasks for development workflows,
So that I can efficiently run tests, benchmarks, formatting, and other development tasks.

**Acceptance Criteria:**

**Given** mise is installed in the project
**When** I run `mise run --list` or check mise.toml
**Then** the following tasks are available:
- `mise run test` - Run all tests
- `mise run test:unit` - Domain layer unit tests only
- `mise run test:integration` - Cross-crate integration tests
- `mise run test:coverage` - Generate coverage report with tarpaulin
- `mise run test:watch` - Watch mode for TDD development
- `mise run bench` - Run performance benchmarks
- `mise run fmt` - Format all code
- `mise run lint` - Run clippy linting
- `mise run verify` - Full quality gate (fmt + lint + test)

**Given** I have researched Rust project best practices for task orchestration
**When** I review the mise.toml configuration
**Then** tasks follow these best practices:
- Tool versions are pinned (Rust 1.92+, clippy, rustfmt versions)
- Tasks use proper shell escaping for cross-platform compatibility
- Tasks include helpful descriptions and usage examples
- Tasks integrate with pre-commit hooks where appropriate

**Given** mise tasks are configured
**When** I run `mise run verify`
**Then** the full quality pipeline executes successfully

## Story 1.4: Configure clippy.toml with Cognitive Complexity Limits

As a developer writing code,
I want clippy to enforce cognitive complexity limits as a quality safeguard,
So that functions remain maintainable and complex logic is broken down appropriately.

**Acceptance Criteria:**

**Given** I have researched clippy best practices for cognitive complexity
**When** I review clippy.toml configuration
**Then** cognitive complexity limits are set to:
- `cognitive-complexity-threshold = 15` (warn level)
- `too-many-lines-threshold = 100` (function length limit)
- Deny level complexity threshold configured

**Given** I have researched anti-pattern prevention in Rust
**When** I check the clippy configuration
**Then** these anti-patterns are denied:
- `clippy::unwrap_used` - No unwrap in production code
- `clippy::expect_used` - No expect in production code
- `clippy::todo` - No TODO comments in production code
- `clippy::unimplemented` - No unimplemented!() in production code
- `clippy::dbg_macro` - No debug prints in production code

**Given** clippy.toml is configured with stringent rules
**When** I run `cargo clippy`
**Then** code exceeding complexity limits generates warnings/errors

**Given** code exceeds cognitive complexity limits
**When** I run clippy
**Then** specific line numbers and suggestions are provided for refactoring

## Story 1.5: Configure rustfmt.toml with Import Sorting

As a developer formatting code,
I want consistent import sorting and formatting standards,
So that code style is uniform and readable across the codebase.

**Acceptance Criteria:**

**Given** I have researched rustfmt best practices for large Rust projects
**When** I review rustfmt.toml configuration
**Then** import sorting is configured with:
- `imports_granularity = "Crate"` - Group imports by crate
- `group_imports = "StdExternalCrate"` - Standard library, external crates, then internal
- Consistent indentation and line width settings

**Given** rustfmt.toml is configured
**When** I run `cargo fmt`
**Then** all imports are sorted consistently across the codebase

**Given** code with unsorted imports
**When** I run `cargo fmt --check`
**Then** the command fails with specific file locations needing formatting

**Given** I have researched formatting standards for Rust ecosystems
**When** I check the configuration
**Then** settings align with Rust community standards for:
- Maximum line width (typically 100-120 characters)
- Brace style consistency
- Comment formatting
- Macro formatting

## Story 1.6: Set Up deny.toml for Dependency Security Auditing

As a developer managing dependencies,
I want automatic security and license auditing of dependencies,
So that vulnerabilities and incompatible licenses are caught before they become issues.

**Acceptance Criteria:**

**Given** I have researched cargo-deny best practices for Rust projects
**When** I review deny.toml configuration
**Then** the following checks are enabled:
- `advisories` - Security vulnerability scanning
- `licenses` - License compatibility checking
- `bans` - Forbidden dependency detection
- `sources` - Source verification

**Given** deny.toml is configured
**When** I run `cargo deny check`
**Then** all dependency checks pass without security issues

**Given** a dependency with security vulnerabilities exists
**When** I run `cargo deny check advisories`
**Then** specific CVEs and affected dependencies are reported

**Given** I have researched license compatibility for open source projects
**When** I check the license configuration
**Then** acceptable licenses include common permissive licenses:
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause
- GPL licenses excluded for compatibility
- Copyleft licenses flagged for review

## Story 1.7: Establish ADR Review Process and Validate Existing ADRs

As a developer making architectural decisions,
I want a clear process for documenting and reviewing ADRs,
So that architectural decisions are well-reasoned, documented, and validated.

**Acceptance Criteria:**

**Given** the ADR directory exists with documents 0001-0007
**When** I review the ADR review process
**Then** a clear process is documented for:
- When to create an ADR (architectural decisions affecting multiple epics)
- ADR template and required sections
- Review and approval process
- How ADRs relate to implementation

**Given** ADRs 0001-0007 exist
**When** I validate them against the established template
**Then** all ADRs follow the proper format:
- Status (Accepted/Rejected/Pending)
- Context and problem description
- Considered alternatives
- Decision with rationale
- Consequences and trade-offs

**Given** the ADR review process is established
**When** a new architectural decision is needed
**Then** the process guides creation of properly formatted ADRs

**Given** I have researched ADR best practices
**When** I review the process
**Then** it follows industry standards:
- MADR (Markdown Architectural Decision Records) format
- Clear decision drivers and constraints
- Stakeholder involvement in decisions
- Regular review and update process

## Story 1.8: Create Comprehensive README.md

As a developer or user discovering the project,
I want a clear overview of the project with setup instructions,
So that I can quickly understand what lithos is and how to get started.

**Acceptance Criteria:**

**Given** README.md is created
**When** I review the content structure
**Then** it includes these essential sections:
- Project description and what makes it special
- Key features and capabilities
- Quick start installation instructions
- Basic usage examples
- Architecture overview (hexagonal, CQRS, etc.)
- Development setup for contributors
- Links to detailed documentation

**Given** I have researched README best practices for open source Rust projects
**When** I check the README format
**Then** it follows best practices:
- Clear badges for CI status, coverage, version
- Table of contents for long documents
- Code examples that are tested and runnable
- Links to CONTRIBUTING.md and CODE_OF_CONDUCT.md
- Performance benchmarks and compatibility matrix

**Given** README.md exists
**When** a new user visits the repository
**Then** they can understand the project purpose within 2 minutes

**Given** README.md exists
**When** a developer wants to contribute
**Then** setup instructions are clear and comprehensive

## Story 1.9: Establish Project Roadmap in Milestones

As a project manager and stakeholder,
I want a comprehensive project roadmap with milestones,
So that project progress, timelines, and dependencies are clearly communicated.

**Acceptance Criteria:**

**Given** I have analyzed the complete project scope from all epics
**When** I review the roadmap structure
**Then** the roadmap includes:
- Epic-level milestones with completion criteria
- Story dependencies and critical path identification
- Timeline estimates with realistic delivery dates
- Risk assessment for major milestones
- Success metrics for measuring progress

**Given** the roadmap is established
**When** stakeholders review project progress
**Then** they can clearly see:
- What has been completed (baseline from Epics 1-8)
- What is currently in progress
- What remains to be done (Epics 9-15)
- Critical dependencies between workstreams
- Go/no-go decision points

**Given** milestones are defined
**When** I check milestone criteria
**Then** each milestone has:
- SMART objectives (Specific, Measurable, Achievable, Relevant, Time-bound)
- Clear deliverables and acceptance criteria
- Resource requirements identified
- Risk mitigation strategies
- Success measurement criteria

**Given** the roadmap is maintained
**When** project changes occur
**Then** the roadmap includes:
- Change control process for scope adjustments
- Regular review cycles (monthly) for timeline updates
- Communication protocols for stakeholder updates
- Contingency planning for identified risks

## Story 1.10: Configure CI/CD for Comprehensive Quality Assurance

As a developer contributing to the project,
I want CI/CD pipelines that provide comprehensive quality assurance through automated testing, security scanning, and performance validation,
So that code quality is guaranteed and regressions are caught early in the development cycle.

**Acceptance Criteria:**

**Multi-Stage Pipeline Architecture:**
- **Given** CI/CD best practices research for Rust projects
- **When** reviewing .github/workflows/ci.yml
- **Then** pipeline includes separated stages: quality gates, testing, security, performance, deployment readiness

**Comprehensive Quality Assurance:**
- **Given** pipeline executes comprehensive quality checks
- **When** PRs are submitted or pushes occur
- **Then** all quality gates pass: formatting, linting, testing, security scanning, ADR validation, performance benchmarks

**Optimization and Performance:**
- **Given** CI optimization techniques for Rust projects
- **When** measuring pipeline performance
- **Then** builds complete within target times with effective caching, parallel execution, and incremental builds

**Matrix Testing and Compatibility:**
- **Given** Rust ecosystem compatibility requirements
- **When** testing across environments
- **Then** matrix builds cover multiple Rust versions, operating systems, and feature combinations

**Artifact Management and Reporting:**
- **Given** CI/CD artifact and reporting best practices
- **When** builds complete
- **Then** comprehensive artifacts uploaded: test results, coverage reports, security scans, performance metrics, build artifacts

**Security and Compliance:**
- **Given** security scanning integration requirements
- **When** CI pipeline executes
- **Then** automated security checks include: dependency vulnerabilities, secrets detection, license compliance, code quality metrics

**Branch Protection and Automation:**
- **Given** GitHub branch protection best practices
- **When** configuring repository settings
- **Then** branch protection requires CI checks, status checks configured, auto-merge policies established

**Monitoring and Alerting:**
- **Given** CI/CD monitoring requirements
- **When** pipeline issues occur
- **Then** notifications configured for failures, performance regressions, security vulnerabilities
