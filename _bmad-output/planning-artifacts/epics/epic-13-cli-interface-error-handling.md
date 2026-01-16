# Epic 13: CLI Interface & Error Handling
Users can execute lithos commands with intuitive CLI, comprehensive help, single-word shortcuts, and actionable error diagnostics.
**FRs covered:** FR41, FR42, FR43, FR44, FR45, FR46, FR47, FR48, FR49, FR50, FR30, FR31
**Implementation Notes:**
- Clap for CLI, miette for diagnostics per ADR 0006
- CommandPort, AuditPort created if needed
- Dependency injection wiring for all ports
- Cross-platform support (macOS primary, Linux)
- Observability/audit logging for FR40 (basic version)
- User CLI documentation

### Story 13.1: [Adapters/API] Clap-based CLI Command Structure
As a user, I want a well-structured CLI with subcommands for all major lithos operations, so that I can navigate and execute commands intuitively.
**Acceptance Criteria:**
- **Given** the clap crate for CLI parsing
- **When** I define the CLI structure
- **Then** it includes subcommands for `template`, `schema`, `vault`, and `config`.
- **And** each subcommand has appropriate sub-subcommands (e.g., `template new`, `template list`).
- **And** global options like `--help`, `--version`, and `--verbose` are supported.
**References:** FR41, FR42

### Story 13.2: [Adapters/API] Template Execution CLI Commands
As a user, I want CLI commands to execute templates with various output options, so that I can create notes from the command line.
**Acceptance Criteria:**
- **Given** the `lithos template` subcommand
- **When** I run `lithos template new <template-name>`
- **Then** it launches the interactive template execution for the specified template.
- **And** `lithos template list` shows available templates.
- **And** output options like `--output <file>` and `--format <markdown|json>` are supported.
**References:** FR45, FR47

### Story 13.3: [Adapters/API] Vault Management CLI Commands
As a user, I want CLI commands to manage vault operations like indexing and searching, so that I can perform vault maintenance from the command line.
**Acceptance Criteria:**
- **Given** the `lithos vault` subcommand
- **When** I run `lithos vault index`
- **Then** it indexes the current vault and updates the search index.
- **And** `lithos vault search <query>` performs searches across indexed notes.
- **And** `lithos vault validate` checks schema compliance across the vault.
**References:** FR44

### Story 13.4: [Adapters/API] Schema Management CLI Commands
As a user, I want CLI commands to manage schemas and validate notes, so that I can maintain schema definitions from the command line.
**Acceptance Criteria:**
- **Given** the `lithos schema` subcommand
- **When** I run `lithos schema list`
- **Then** it shows all available schema definitions.
- **And** `lithos schema validate <file>` validates a specific note against its schema.
- **And** `lithos schema create <name>` launches the schema creation workflow.
**References:** FR43

### Story 13.5: [Adapters/API] Configuration CLI Commands
As a user, I want CLI commands to manage application configuration, so that I can set preferences and view current settings.
**Acceptance Criteria:**
- **Given** the `lithos config` subcommand
- **When** I run `lithos config show`
- **Then** it displays the current configuration hierarchy (global → user → project → vault).
- **And** `lithos config set <key> <value>` allows setting configuration values.
- **And** `lithos config reset` restores default configuration.
**References:** FR46

### Story 13.6: [Adapters/API] Miette-based Error Diagnostics
As a user, I want clear, actionable error messages when operations fail, so that I can understand and resolve issues quickly.
**Acceptance Criteria:**
- **Given** the miette crate for diagnostics
- **When** a command fails
- **Then** it displays structured error messages with context and suggestions.
- **And** file path errors include clickable links in supported terminals.
- **And** validation errors highlight specific fields and provide correction hints.
**References:** FR48

### Story 13.7: [Adapters/API] Comprehensive Help System
As a user, I want comprehensive help and documentation accessible from the CLI, so that I can learn how to use lithos without leaving the terminal.
**Acceptance Criteria:**
- **Given** any lithos command
- **When** I add `--help` or `-h`
- **Then** it shows detailed usage information with examples.
- **And** `lithos help <topic>` provides in-depth documentation for specific features.
- **And** help text includes command-line examples and common use cases.
**References:** FR42

### Story 13.8: [Adapters/API] Cross-Platform Terminal Support
As a user, I want lithos to work consistently across operating systems, so that I can use it on macOS, Linux, and potentially Windows.
**Acceptance Criteria:**
- **Given** the terminal environment
- **When** lithos runs on different platforms
- **Then** it detects and adapts to terminal capabilities (colors, Unicode, etc.).
- **And** file paths are handled correctly for each platform's conventions.
- **And** CLI behavior is consistent across supported platforms (macOS primary, Linux).
**References:** FR30, FR31

### Story 13.9: [Adapters/SPI] Basic Audit Logging Infrastructure
As an administrator, I want basic audit logging for template execution and critical operations, so that I can track system usage and troubleshoot issues.
**Acceptance Criteria:**
- **Given** template execution or vault operations
- **When** they complete
- **Then** key events are logged with timestamps and user context.
- **And** logs are written to a configurable location with rotation.
- **And** log levels can be configured (error, warn, info, debug).
**References:** FR40

### Story 13.10: [Adapters/API] Single-Word Command Shortcuts
As a power user, I want single-word shortcuts for common operations, so that I can execute frequent commands quickly.
**Acceptance Criteria:**
- **Given** common lithos operations
- **When** I use shortcuts like `lithos new`
- **Then** it launches the template picker for creating new notes.
- **And** `lithos search <query>` performs a vault search.
- **And** shortcuts are documented in the help system.
**References:** FR47

### Story 13.11: [Recovery] System-Wide Error Recovery Coordination
As a user experiencing system-wide issues, I want coordinated error recovery across all components, so that complex operations can be safely rolled back and system state remains consistent.
**Acceptance Criteria:**
**Given** multi-epic operations fail
**When** the CLI detects cascading failures
**Then** it coordinates rollback across storage, indexing, and templates
**And** it provides clear status on what operations were reverted
**And** it offers recovery options (retry, partial recovery, full reset)

**Given** system corruption is detected
**When** recovery operations run
**Then** the CLI provides progress indicators and estimated completion times
**And** it validates system integrity after recovery
**And** it logs detailed recovery actions for troubleshooting

### Story 13.12: [Test] CLI Performance Benchmarking (NFR4 Validation)
As a performance engineer, I want CLI command performance benchmarks, so that NFR4 (instant feedback and help) is validated and maintained.
**Acceptance Criteria:**
**Given** CLI commands are implemented
**When** I benchmark CLI performance
**Then** help commands (--help, -h) display instantly (<100ms)
**And** basic commands complete with feedback in <500ms
**And** complex operations provide progress indicators

**Given** CLI performance is monitored
**When** I detect regressions
**Then** performance benchmarks are part of CI/CD pipeline
**And** startup time remains fast across all supported platforms

### Story 13.13: Clap CLI Performance Regression Testing
As a performance engineer, I want automated regression tests for Clap CLI operations, so that the architectural choice of Clap remains optimal and CLI parsing stays under 50μs baseline.
**Acceptance Criteria:**
**Given** Clap CLI implementation
**When** performance regression tests run
**Then** command parsing benchmarks are compared against 50μs baseline
**And** help generation performance is validated under 100ms
**And** complex command structures maintain fast parsing
**And** CLI benchmarks run in CI/CD for every CLI-related change

### Story 13.14: Review Epic 13 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 13 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** docs/testing/developer-guide.md provides testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, and utilities

**Given** the implementation of Epic 13
**When** I run the test suite
**Then** it achieves 90%+ coverage for CLI parsing, error formatting, and platform detection
**And** integration tests verify end-to-end CLI workflows
**And** the suite validates that all commands produce consistent help output

**Given** all Epic 13 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests actually validate business requirements vs implementation details

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 13 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**References:** NFR16

### Story 13.15: Epic 13 CLI Documentation
As a user, I want comprehensive CLI documentation with examples and tutorials, so that I can master the command-line interface.
**Acceptance Criteria:**
- **Given** a completed Epic 13
- **When** I review the CLI documentation
- **Then** it includes a command reference for all subcommands and options.
- **And** it provides usage examples for common workflows.
- **And** it documents error message interpretation and troubleshooting.
- **And** it includes error recovery procedures and system coordination guidelines.
- **And** it documents Clap architectural choice rationale and performance baselines.
**References:** NFR13
