# Epic 14: CLI Interface & Error Handling

## Overview

Users can execute lithos commands with intuitive CLI, comprehensive help, single-word shortcuts, and actionable error diagnostics.

**FRs covered:** FR41 (command structure), FR42 (help), FR43-47 (subcommands), FR48 (diagnostics), FR49-50 (interactive), FR30-31 (cross-platform), FR40 (audit)

## Implementation Notes

- **CLI Framework**: Clap v4 per ADR 005 (declarative derive macros, performance <50μs parsing)
- **Error Diagnostics**: Miette per ADR 005 (structured error reporting with source spans)
- **Terminal UI**: Dialoguer for interactive prompts, console for cross-platform detection
- **Integration Points**:
  - Epic 6: ConfigCommand/ConfigQuery for configuration management
  - Epic 7: SchemaCommand/SchemaQuery for schema operations
  - Epic 10: IndexerService for vault indexing
  - Epic 11: QueryService for vault searching
  - Epic 12: TemplateExecutor for template execution
  - Epic 8: EventBus integration for async operations and progress tracking
- **Dependency Injection**: All services injected via constructor pattern (no global singletons in CLI layer)
- **Cross-Platform**: Primary target macOS, tested on Linux, terminal capability detection via `console` crate
- **Performance Targets**:
  - CLI parsing: <50μs per ADR 005
  - Help display: <100ms for instant feedback (NFR4)
  - Command startup: <500ms for basic operations
  - Progress indicators for operations >2s (NFR2)
- **Audit Logging**: Tracing-based audit events to file (`tracing-appender` with rotation)
- **Location**: `crates/cli/src/` contains commands/, error.rs, main.rs, app.rs
- **Error Handling Strategy**:
  - Domain errors → Miette diagnostics with context
  - Validation errors → Highlighted fields with suggestions
  - File errors → Clickable paths in supported terminals
  - Network errors → Retry suggestions with exponential backoff guidance
- **Command Structure**:
  - `lithos template` - Template management (new, list, execute)
  - `lithos schema` - Schema operations (list, validate, create)
  - `lithos vault` - Vault operations (index, search, validate)
  - `lithos config` - Configuration management (show, set, reset)
  - Single-word shortcuts: `lithos new`, `lithos search`
- **May Create**: ADR for CLI error recovery patterns if complex coordination emerges

## Story 14.1: Implement Clap-based CLI Command Structure

As a user, I want a well-structured CLI with subcommands for all major lithos operations,
So that I can navigate and execute commands intuitively.

**Acceptance Criteria:**

**Given** Clap v4 provides derive macros per ADR 005
**When** I implement CLI structure in `crates/cli/src/app.rs`
**Then** `LithosApp` struct uses `#[derive(Parser)]` with subcommands:
- `Template(TemplateCommands)` - template operations
- `Schema(SchemaCommands)` - schema operations
- `Vault(VaultCommands)` - vault operations
- `Config(ConfigCommands)` - configuration operations

**Given** each subcommand has nested operations
**When** I define `TemplateCommands` enum
**Then** it includes variants:
- `New { template: String, output: Option<PathBuf> }` - execute template
- `List { format: Option<OutputFormat> }` - list available templates
- `Validate { template: PathBuf }` - validate template syntax

**Given** global options must be available to all subcommands
**When** I define `LithosApp` struct
**Then** global fields include:
- `#[arg(short, long)]` verbose: bool - enable verbose output
- `#[arg(long)]` vault_path: Option<PathBuf> - override vault location
- `#[arg(long)]` config: Option<PathBuf> - custom config file
- `#[arg(short, long)]` version: bool - display version info

**Given** Clap performance must meet ADR 005 baseline
**When** I benchmark CLI parsing
**Then** `LithosApp::parse()` completes in <50μs for simple commands
**And** complex nested commands parse in <200μs
**And** parsing benchmarks are tracked in CI/CD

**Given** help generation must be instant per NFR4
**When** user runs `lithos --help` or `lithos template --help`
**Then** help text displays in <100ms
**And** help includes command examples and common use cases
**And** Clap's built-in help formatting is customized for consistency

**Given** cross-platform support is required
**When** CLI runs on macOS and Linux
**Then** path arguments use `PathBuf` for platform-agnostic handling
**And** terminal capability detection adapts output formatting
**And** color output is disabled on non-TTY environments

**References:** FR41, FR42, NFR4, ADR 005

## Story 14.2: Implement Template Execution CLI Commands

As a user, I want CLI commands to execute templates with various output options,
So that I can create notes from the command line.

**Acceptance Criteria:**

**Given** Epic 12 provides TemplateExecutor service
**When** I implement `lithos template new <name>` command
**Then** it injects TemplateExecutor via dependency injection
**And** executes template interactively using Dialoguer prompts
**And** saves output to vault with schema validation

**Given** template execution is interactive (FR49, FR50)
**When** user runs `lithos template new contact`
**Then** CLI prompts for template variables using Dialoguer:
- Text input for string variables
- Select for enum variables (from schema or template-defined)
- Confirm for boolean variables
- Date input with validation

**Given** output location must be configurable
**When** user specifies `--output <path>` flag
**Then** template writes to specified path instead of default vault location
**And** path validation ensures file doesn't exist or `--force` flag is used
**And** parent directories are created if needed

**Given** output format must support multiple targets
**When** user specifies `--format <format>` flag
**Then** supported formats include:
- `markdown` (default) - Obsidian-compatible markdown
- `json` - structured JSON for programmatic use
- `yaml` - YAML frontmatter + content

**Given** template listing must be fast and informative
**When** user runs `lithos template list`
**Then** output includes columns: Name, Description, Schema, Last Modified
**And** list is sorted alphabetically by default
**And** `--format json` outputs machine-readable template metadata
**And** list command completes in <500ms for typical template directories

**Given** Epic 12 template validation is available
**When** user runs `lithos template validate <path>`
**Then** validation checks:
- MiniJinja syntax correctness
- Variable definitions match schema
- Required variables are defined
- File paths resolve correctly
**And** validation errors use Miette diagnostics with source spans

**Given** template execution may fail
**When** errors occur during execution
**Then** partial execution is rolled back (no incomplete notes written)
**And** error diagnostics include: failed variable, template location, validation context
**And** user can retry with `--resume` flag (if supported in future)

**References:** FR45, FR47, FR49, FR50

## Story 14.3: Implement Vault Management CLI Commands

As a user, I want CLI commands to manage vault operations like indexing and searching,
So that I can perform vault maintenance from the command line.

**Acceptance Criteria:**

**Given** Epic 10 provides IndexerService
**When** I implement `lithos vault index` command
**Then** it injects IndexerService and triggers full vault re-index
**And** displays progress indicator for vaults >100 files (NFR2: complete in <2s for 1000 notes)
**And** progress shows: "Indexed 573/1000 notes (57%)..."

**Given** indexing is long-running (potentially >2s for large vaults)
**When** indexing executes
**Then** Epic 8 EventBus provides progress events
**And** CLI subscribes to IndexProgress events on ControlPlane broadcast
**And** progress bar updates in real-time using `indicatif` crate
**And** final summary shows: "Indexed 1000 notes in 1.8s (556 notes/sec)"

**Given** indexing may encounter errors
**When** individual files fail validation/parsing
**Then** indexing continues with warnings: "Warning: Skipped invalid file at path/to/note.md"
**And** final summary reports: "Successfully indexed 995/1000 notes (5 errors)"
**And** `--strict` flag fails indexing on first error

**Given** Epic 11 provides QueryService
**When** I implement `lithos vault search <query>` command
**Then** it injects QueryService and executes search against Epic 9 storage indexes
**And** search supports query syntax:
- Simple text: `lithos vault search "project management"`
- Schema filter: `lithos vault search --schema contact "John"`
- Metadata filter: `lithos vault search --tag rust --status active`

**Given** search results must be user-friendly
**When** search returns results
**Then** output format includes:
- Path (clickable in supported terminals)
- Title (extracted from frontmatter or first heading)
- Excerpt (surrounding matched text with highlighting)
- Score (relevance ranking)
**And** `--format json` outputs machine-readable results
**And** results are paginated for large result sets (default 20 per page)

**Given** vault validation checks schema compliance
**When** user runs `lithos vault validate`
**Then** validation checks all notes against their declared schemas (Epic 7)
**And** reports validation errors grouped by schema
**And** output includes:
- Total notes validated
- Validation errors by severity (error, warning)
- Notes without schemas (info-level)

**Given** vault statistics are useful for maintenance
**When** user runs `lithos vault stats`
**Then** output includes:
- Total note count
- Notes by schema (breakdown)
- Index size on disk
- Last indexed timestamp
- Vault size on disk

**References:** FR44, NFR2

### Story 14.4: [Adapters/API] Schema Management CLI Commands

As a user, I want CLI commands to manage schemas and validate notes, so that I can maintain schema definitions from the command line.
**Acceptance Criteria:**

- **Given** the `lithos schema` subcommand
- **When** I run `lithos schema list`
- **Then** it shows all available schema definitions.
- **And** `lithos schema validate <file>` validates a specific note against its schema.
- **And** `lithos schema create <name>` launches the schema creation workflow.
  **References:** FR43

### Story 14.5: [Adapters/API] Configuration CLI Commands

As a user, I want CLI commands to manage application configuration, so that I can set preferences and view current settings.
**Acceptance Criteria:**

- **Given** the `lithos config` subcommand
- **When** I run `lithos config show`
- **Then** it displays the current configuration hierarchy (global → user → project → vault).
- **And** `lithos config set <key> <value>` allows setting configuration values.
- **And** `lithos config reset` restores default configuration.
  **References:** FR46

## Story 14.6: Implement Miette-based Error Diagnostics

As a user, I want clear, actionable error messages when operations fail,
So that I can understand and resolve issues quickly.

**Acceptance Criteria:**

**Given** Miette provides structured diagnostics per ADR 005
**When** I implement error handling in `crates/cli/src/error.rs`
**Then** `CliError` enum wraps all domain/adapter errors
**And** `CliError` implements `miette::Diagnostic` trait for rich formatting
**And** error variants include: `TemplateError`, `SchemaError`, `VaultError`, `ConfigError`, `IoError`

**Given** domain errors must be user-friendly
**When** I convert domain errors to CLI errors
**Then** error context includes:
- Source file path and line number (when applicable)
- User-actionable suggestions (e.g., "Run `lithos vault index` to rebuild")
- Related documentation links (e.g., "See: https://lithos.dev/docs/schemas")
- Severity level (error, warning, info)

**Given** file errors must be clickable in terminals
**When** I format file path errors
**Then** Miette source spans point to specific file locations
**And** paths are absolute for terminal hyperlinks (macOS Terminal, iTerm2 support)
**And** error output includes: `error: template not found at /path/to/template.md:5:10`

**Given** validation errors need field-level detail
**When** schema or template validation fails
**Then** error highlights specific YAML/TOML fields with source spans:
```
error: invalid property type
  ┌─ schemas/contact.yaml:10:5
  │
10│     type: strin
  │           ^^^^^ expected "string", found "strin"
  │
  = help: Valid types are: string, number, boolean, date, file
```

**Given** Epic 6 ConfigError, Epic 7 SchemaError need CLI formatting
**When** I wrap adapter errors
**Then** `From<ConfigError> for CliError` preserves error context
**And** `From<SchemaError> for CliError` adds schema-specific help text
**And** all conversions use `miette::wrap_err()` to maintain error chain

**Given** multi-error scenarios (e.g., multiple validation failures)
**When** operations produce multiple errors
**Then** Miette's `related` feature aggregates errors in single report
**And** errors are grouped by category (file errors, validation errors, etc.)
**And** total error count displayed: `error: found 5 validation errors`

**Given** error recovery suggestions must be contextual
**When** I implement diagnostic suggestions
**Then** file not found → "Create file or check path"
**And** permission denied → "Run with elevated permissions or check file ownership"
**And** invalid schema → "Validate schema with `lithos schema validate <file>`"
**And** vault not indexed → "Run `lithos vault index` to build search index"

**Given** error output must respect terminal capabilities
**When** CLI runs in different environments
**Then** color output disabled on non-TTY (piped output, CI)
**And** Unicode box-drawing characters fallback to ASCII on unsupported terminals
**And** `console` crate detects terminal features (color depth, Unicode support)

**References:** FR48, ADR 005

### Story 14.7: [Adapters/API] Comprehensive Help System

As a user, I want comprehensive help and documentation accessible from the CLI, so that I can learn how to use lithos without leaving the terminal.
**Acceptance Criteria:**

- **Given** any lithos command
- **When** I add `--help` or `-h`
- **Then** it shows detailed usage information with examples.
- **And** `lithos help <topic>` provides in-depth documentation for specific features.
- **And** help text includes command-line examples and common use cases.
  **References:** FR42

### Story 14.8: [Adapters/API] Cross-Platform Terminal Support

As a user, I want lithos to work consistently across operating systems, so that I can use it on macOS, Linux, and potentially Windows.
**Acceptance Criteria:**

- **Given** the terminal environment
- **When** lithos runs on different platforms
- **Then** it detects and adapts to terminal capabilities (colors, Unicode, etc.).
- **And** file paths are handled correctly for each platform's conventions.
- **And** CLI behavior is consistent across supported platforms (macOS primary, Linux).
  **References:** FR30, FR31

### Story 14.9: [Adapters/SPI] Basic Audit Logging Infrastructure

As an administrator, I want basic audit logging for template execution and critical operations, so that I can track system usage and troubleshoot issues.
**Acceptance Criteria:**

- **Given** template execution or vault operations
- **When** they complete
- **Then** key events are logged with timestamps and user context.
- **And** logs are written to a configurable location with rotation.
- **And** log levels can be configured (error, warn, info, debug).
  **References:** FR40

### Story 14.10: [Adapters/API] Single-Word Command Shortcuts

As a power user, I want single-word shortcuts for common operations, so that I can execute frequent commands quickly.
**Acceptance Criteria:**

- **Given** common lithos operations
- **When** I use shortcuts like `lithos new`
- **Then** it launches the template picker for creating new notes.
- **And** `lithos search <query>` performs a vault search.
- **And** shortcuts are documented in the help system.
  **References:** FR47

## Story 14.11: Implement System-Wide Error Recovery Coordination

As a user experiencing system-wide issues, I want coordinated error recovery across all components,
So that complex operations can be safely rolled back and system state remains consistent.

**Acceptance Criteria:**

**Given** complex operations span multiple epics (template execution → indexing → storage)
**When** operation fails mid-execution
**Then** CLI coordinates rollback via Epic 9 UnitOfWork pattern
**And** rollback order is reverse of execution order (LIFO)
**And** each epic's rollback is atomic (all-or-nothing per component)

**Given** template execution creates note → triggers indexing → updates storage
**When** storage write fails after indexing succeeds
**Then** CLI rollback sequence:
1. Abort storage transaction (Epic 9 UnitOfWork rollback)
2. Remove from search index (Epic 11 cache invalidation)
3. Delete partially-written note file (Epic 10 file cleanup)
**And** user sees: "Rolling back: storage → index → file (3/3 complete)"

**Given** rollback operations may partially fail
**When** individual rollback steps fail
**Then** CLI continues best-effort rollback for remaining steps
**And** logs all rollback failures: "Warning: Failed to remove index entry (cache error)"
**And** final summary reports: "Rollback completed with 1 warning (system may be inconsistent)"

**Given** system corruption detected (Epic 9 storage, Epic 11 cache, or Epic 7 schema)
**When** user runs `lithos vault repair`
**Then** repair sequence:
1. Validate Epic 9 storage integrity (rkyv checksums)
2. Rebuild Epic 11 cache from Epic 9 storage (clean slate protocol)
3. Re-index all notes (Epic 10 full scan)
4. Validate schema consistency (Epic 7)
**And** each step shows progress: "Validating storage: 573/1000 entries (57%)..."

**Given** repair operations are long-running
**When** repair executes
**Then** Epic 8 EventBus provides progress events across all components
**And** CLI aggregates progress from multiple event sources
**And** estimated completion time shown: "Estimated 45s remaining..."
**And** repair can be cancelled with Ctrl+C (graceful shutdown)

**Given** system integrity validation is needed
**When** repair completes
**Then** validation checks:
- Epic 9 storage: all tables have valid rkyv checksums
- Epic 11 cache: indexes match storage contents
- Epic 10: all files referenced in storage exist on disk
- Epic 7: all schemas are valid and loaded
**And** validation report shows: "✓ Storage integrity OK, ✓ Cache consistency OK, ✗ 3 orphaned files found"

**Given** recovery options must be flexible
**When** user encounters errors
**Then** CLI suggests context-specific recovery:
- Single note validation error → "Fix schema or skip with --force"
- Index corruption → "Run `lithos vault repair --index-only`"
- Storage corruption → "Run `lithos vault repair --full` (WARNING: rebuilds from files)"
- Config errors → "Run `lithos config reset` to restore defaults"

**Given** detailed logging is required for troubleshooting
**When** recovery operations run
**Then** audit log records:
- Timestamp of failure and recovery attempt
- Component-by-component rollback steps
- Errors encountered during rollback
- Final system state (consistent/inconsistent)
**And** logs written to `.lithos/logs/recovery-{timestamp}.log`

**Given** Epic 6 ConfigCache supports rollback
**When** config update fails
**Then** `lithos config rollback` restores previous valid snapshot
**And** rollback uses Epic 6 ConfigCache snapshot history (last 10 versions)
**And** user confirms rollback: "Restore config from 2025-01-27 14:32:15? [y/N]"

**References:** FR48, NFR2

### Story 14.12: [Test] CLI Performance Benchmarking (NFR4 Validation)

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

### Story 14.13: Clap CLI Performance Regression Testing

As a performance engineer, I want automated regression tests for Clap CLI operations, so that the architectural choice of Clap remains optimal and CLI parsing stays under 50μs baseline.
**Acceptance Criteria:**
**Given** Clap CLI implementation
**When** performance regression tests run
**Then** command parsing benchmarks are compared against 50μs baseline
**And** help generation performance is validated under 100ms
**And** complex command structures maintain fast parsing
**And** CLI benchmarks run in CI/CD for every CLI-related change

### Story 14.14: Review Epic 14 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 14 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 14 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 14 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 14 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 14 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**Given** the implementation of Epic 14
**When** I run the test suite
**Then** it achieves 90%+ coverage for CLI parsing, error formatting, and platform detection
**And** integration tests verify end-to-end CLI workflows
**And** the suite validates that all commands produce consistent help output

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

**References:** NFR16

### Story 14.15: Epic 14 CLI Documentation

As a user, I want comprehensive CLI documentation with examples and tutorials, so that I can master the command-line interface.
**Acceptance Criteria:**

- **Given** a completed Epic 14
- **When** I review the CLI documentation
- **Then** it includes a command reference for all subcommands and options.
- **And** it provides usage examples for common workflows.
- **And** it documents error message interpretation and troubleshooting.
- **And** it includes error recovery procedures and system coordination guidelines.
- **And** it documents Clap architectural choice rationale and performance baselines.
  **References:** NFR13
