# Epic 5: Configuration Management System **[PHASE 1.5]**

Users can configure lithos through hierarchical TOML files with validation, supporting template packs and schema definitions.
**FRs covered:** FR26, FR27, FR28
**Implementation Notes:**

- Figment-based hierarchical config per ADR 0005 using Epic 4 loading foundation
- Config adapters (Command/Query with embedded Loader/Writer) created in this epic
- Singleton Registry pattern with `Arc<OnceLock<Config>>` for optimal CLI performance
- Sample config files based on JSON schema (lithos-specific)
- User documentation for configuration
- **Architecture**: Domain ports exist (Epic 3), domain aggregate exists (Epic 3), adapters integrate Epic 4 utilities
- **Adapter Structure**: `crates/adapters/src/spi/config/` contains query.rs, command.rs, loader.rs, writer.rs, registry.rs
- **No CLI Integration**: Epic 5 delivers tested adapters without CLI wiring (deferred to future epic)

## Story 5.1: Implement Config Adapters with Embedded Utilities

As a developer completing the configuration bounded context,
I want ConfigCommand and ConfigQuery adapters that embed specialized loader/writer utilities,
So that port implementations are clean and easily extensible for future operations.

**Acceptance Criteria:**

**Given** existing ports in `crates/domain/src/ports/config.rs`
**When** I update the trait definitions
**Then** `load`, `load_global`, and `load_vault` methods are moved from `Query` to `Command` trait
**And** `Query` trait retains only side-effect-free methods (if any remain)

**Given** Epic 3 defined Command and Query trait interfaces in `crates/domain/src/ports/config.rs`
**When** I implement Loader utility in `crates/adapters/src/spi/config/loader.rs`
**Then** it provides read operations (load, load_global, load_vault) using Epic 4 FormatDispatcher + PathValidator + Figment

**Given** Loader is implemented
**When** I implement Query adapter in `crates/adapters/src/spi/config/query.rs`
**Then** it implements Query trait by delegating to embedded Loader instance

**Given** I need write operations
**When** I implement Writer utility in `crates/adapters/src/spi/config/writer.rs`
**Then** it provides write operations (save_global, save_vault) using Epic 4 PathValidator + tokio::fs

**Given** Writer is implemented
**When** I implement Command adapter in `crates/adapters/src/spi/config/command.rs`
**Then** it implements Command trait by delegating to embedded Writer instance

**Given** both adapters are implemented
**When** I test integration
**Then** Command and Query work together for complete configuration management

**Given** future extensions are needed (e.g., update_value, lookup_value)
**When** I add methods to Command/Query traits
**Then** adapters can implement new methods by coordinating between loaders/writers without refactoring I/O layer

**Given** Epic 4 provides FS utilities
**When** I implement Loader and Writer
**Then** both use FormatDispatcher::parse() for format detection and PathValidator::validate() for path security

**Given** adapters are implemented
**When** I export them in `crates/adapters/src/spi/config/mod.rs`
**Then** internal structs (Query, Command, Loader, Writer) are re-exported with Config prefix (ConfigQuery, ConfigCommand, ConfigLoader, ConfigWriter)

## Story 5.2: Implement Config Singleton Registry

As a developer implementing configuration management,
I want a singleton registry for Config access,
So that the entire application references a single merged Config instance with optimal performance.

**Acceptance Criteria:**

**Given** Config domain aggregate exists in `crates/domain/src/config/aggregate.rs`
**When** I implement Registry singleton in `crates/adapters/src/spi/config/registry.rs`
**Then** it uses `Arc<OnceLock<Config>>` pattern for thread-safe, zero-lock reads

**Given** CLI operations need consistent Config access
**When** I implement singleton instance method
**Then** `Registry::global()` returns the same instance across all calls

**Given** multiple concurrent CLI operations access config
**When** I benchmark access patterns
**Then** reads complete in <10ns for warm cache hits (no lock contention)

**Given** architecture must remain hexagonal
**When** I implement Registry service
**Then** Config domain contains no singleton logic while Registry manages all infrastructure concerns

**Given** configuration needs both read performance and mutability
**When** I design the singleton implementation
**Then** implement hybrid approach: `Arc<OnceLock<Config>>` for immutable merged config + `Arc<RwLock<T>>` for mutable runtime state

**Given** future hot reloading will be needed for LSP phase
**When** I design the singleton implementation
**Then** Registry supports atomic updates using AtomicPtr swap pattern for future extension

**Given** Story 5.1 provides Loader adapter
**When** I integrate with Registry
**Then** Loader populates the singleton on application startup via `Config::build()`

**Given** Story 5.3 provides Figment hierarchical loading
**When** I implement initialization
**Then** Registry loads config via Loader with proper precedence (CLI > Env > Files > Defaults)

**Given** Registry stores merged config
**When** I validate singleton contents
**Then** it holds the final merged `Config` aggregate (not separate Global/Vault instances)

**Given** Registry is implemented
**When** I export it in `crates/adapters/src/spi/config/mod.rs`
**Then** internal struct Registry is re-exported as ConfigRegistry

## Story 5.3: Implement Hierarchical Configuration Loading

As a user configuring lithos,
I want hierarchical configuration that respects precedence rules,
So that I can override settings at different levels (global, user, project, vault).

**Acceptance Criteria:**

**Given** Epic 4 provides unified structured file loading (TOML, JSON, YAML) via FormatDispatcher
**When** I implement hierarchical config using Figment per ADR 0005 in Loader
**Then** configuration loads with proper precedence: CLI > Environment > Config files > Defaults

**Given** hierarchical loading is implemented
**When** I test precedence
**Then** vault-level config overrides global-level as defined in `Config::build()`

**Given** configuration files are loaded using Epic 4 infrastructure
**When** I validate TOML parsing in Loader
**Then** complex nested structures are properly deserialized through Epic 4's FormatDispatcher

**Given** Figment provides hierarchical merging
**When** I implement Loader.load_global() and Loader.load_vault()
**Then** Figment merges CLI args, environment variables, and file sources with correct precedence

**Given** Loader uses Epic 4 utilities
**When** I implement file loading
**Then** PathValidator::validate() checks path security before FormatDispatcher::parse() handles format detection

## Story 5.4: Add Configuration Validation and Error Handling

As a user providing configuration,
I want clear validation and helpful error messages,
So that I can identify and fix configuration issues quickly.

**Acceptance Criteria:**

**Given** configuration is loaded via Loader
**When** I validate config structure
**Then** Epic 4 PathValidator checks path security and FormatDispatcher handles format parsing

**Given** domain aggregate is built
**When** I call `Config::build()`
**Then** domain validation occurs for required fields (vault_path) and enum constraints (log_level)

**Given** validation fails
**When** I check error messages
**Then** errors are actionable with specific field locations via ConfigError variants

**Given** configuration validation is implemented
**When** I test error handling
**Then** partial invalid configs provide clear guidance via miette-formatted errors

**Given** Epic 4 provides ParseError and PathValidationError
**When** I implement Loader error handling
**Then** adapter converts Epic 4 errors to ConfigError with additional context

## Story 5.5: Implement Configuration Versioning and Migration

As a developer maintaining lithos,
I want configuration versioning and migration support,
So that configuration files can evolve safely across versions without breaking user setups.

**Acceptance Criteria:**

**Given** configuration evolves over time
**When** I implement versioning in Global and Vault domain models
**Then** config files include version field for compatibility checking

**Given** version mismatches are detected
**When** I implement migration logic in Loader
**Then** automatic migration transforms old config to new format before calling `Config::build()`

**Given** breaking changes occur
**When** users upgrade
**Then** clear error messages guide them through manual migration steps via ConfigError

**Given** Loader detects version field
**When** I validate version compatibility
**Then** migration occurs transparently before domain validation

## Story 5.6: Configuration Error Recovery and Rollback

As a user who has made configuration mistakes,
I want the system to provide clear error messages and recovery options,
So that I can fix configuration issues without losing my work.

**Acceptance Criteria:**

**Given** configuration validation fails
**When** I attempt to load invalid configuration via Loader
**Then** clear error messages identify the specific problems via ConfigError and suggest fixes

**Given** validation fails
**When** I check error handling
**Then** the system falls back to default values for invalid settings per `Config::build()` logic

**Given** previous valid configuration exists
**When** I load invalid configuration
**Then** previous valid configuration is preserved (Registry retains last known good config)

**Given** configuration changes cause system instability
**When** I need to rollback
**Then** the system can restore previous known-good configuration via Registry cache

**Given** rollback is needed
**When** I implement recovery logic
**Then** configuration history is maintained for recovery (optional future enhancement)

## Story 5.7: Review Epic 5 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 5 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** _bmad-output/test-design-system.md and _bmad-output/test-developer-guide.md provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, and utilities

**Given** all Epic 5 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 5 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 5 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 5 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

## Story 5.8: Create Default Configuration Files and Schema

As a user getting started with lithos,
I want default configuration files with proper schema validation,
So that I can understand configuration options and customize settings confidently.

**Acceptance Criteria:**

**Given** I need to define configuration structure
**When** I evaluate schema options
**Then** I decide whether to extend `docs/schemas/lithos.schema.json` with config definitions or create a separate `docs/schemas/config.schema.json`

**Given** configuration schema is defined
**When** I create default config files in TOML format
**Then** `global.toml` and `vault.toml` are created in `docs/defaults/` with all fields set to default values

**Given** default config files use multiple formats
**When** I provide format examples
**Then** TOML (primary), JSON, and YAML versions are created showing the same default configuration

**Given** default files exist
**When** I validate against the schema
**Then** all default configs pass schema validation and demonstrate proper structure

**Given** users have default configs
**When** they start lithos using Epic 4's file loading
**Then** configurations load successfully via Loader using default values

**Given** default files use domain types
**When** I test defaults with Loader
**Then** they deserialize correctly into Global and Vault domain models

## Story 5.9: Document Configuration System

As a user configuring lithos,
I want comprehensive documentation for configuration options,
So that I can understand and customize lithos behavior effectively.

**Acceptance Criteria:**

**Given** all Epic 5 code is implemented
**When** I review all doc comments
**Then** they are accurate, precise, and follow project standards from project-context.md

**Given** all Epic 5 public components are documented
**When** I verify doc comments
**Then** all public structs, enums, traits, and functions have proper `///` documentation

**Given** all Epic 5 public APIs are documented
**When** I verify doc tests
**Then** all public components include `# Examples` sections with runnable code snippets

**Given** configuration system is implemented
**When** I create user documentation
**Then** it includes all configuration options with examples and defaults

**Given** user documentation exists
**When** I check completeness
**Then** it covers hierarchical loading, validation rules, and troubleshooting

**Given** Epic 5 adapters are implemented
**When** I create developer documentation
**Then** `docs/config-adapters.md` is created following the pattern from `docs/domain-models.md` (Epic 3) for comprehensive epic-level documentation

**Given** developer documentation exists
**When** I validate completeness
**Then** it covers adapter architecture, composition pattern, Epic 4 integration, singleton Registry, and extension guidelines

**Given** Epic 5 module structure is finalized
**When** I create module-level documentation
**Then** `crates/adapters/src/spi/config/README.md` is created with module structure, file purposes, and quick start examples

**Given** module README exists
**When** I validate content
**Then** it provides quick reference for developers navigating the config adapter module and links to comprehensive docs

**Given** users and developers read the documentation
**When** they work with configuration
**Then** they can successfully customize behavior and extend adapters without assistance
