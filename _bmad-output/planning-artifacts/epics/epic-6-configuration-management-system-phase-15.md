# Epic 6: Configuration Management System **[PHASE 1.5]**

## Overview

Users can configure lithos through hierarchical TOML files with validation, supporting template packs and schema definitions.

**FRs covered:** FR26, FR27, FR28

## Implementation Notes

- **Schema-First Approach**: Defaults and Schema are defined first (Story 6.1) to guide adapter implementation
- **Figment-based hierarchical config** per ADR 0005 using Epic 4 loading foundation
- **Config adapters** (Command/Query with embedded Loader/Writer) created in this epic
- **Singleton Registry pattern** with `Arc<OnceLock<Config>>` for optimal CLI performance
- **Sample config files** based on JSON schema (lithos-specific)
- **Architecture**: Domain ports exist (Epic 3), domain aggregate exists (Epic 3), adapters integrate Epic 4 utilities + Epic 5 caching
- **Syntactic Validation**: Dedicated `validator.rs` for structural configuration validation before domain aggregate construction
- **Caching Strategy**: Uses Epic 5 `RedbReader`/`RedbWriter` to persist fully-merged `Config` aggregates for persistence and rollback
- **Adapter Structure**: `crates/adapters/src/spi/config/` contains query.rs, command.rs, loader.rs, writer.rs, registry.rs, validator.rs, cache.rs
- **No CLI Integration**: Epic 6 delivers tested adapters without CLI wiring (deferred to future epic)
- **ADR References**: ADR 0005 (Figment hierarchical config)

## Story 6.1: Create Default Configuration Files and Schema

As a user getting started with lithos,
I want default configuration files with proper schema validation,
So that I can understand configuration options and customize settings confidently.

**Acceptance Criteria:**

**Given** I need to define configuration structure
**When** I evaluate schema options
**Then** I create a NEW dedicated, authoritative schema file at `docs/schemas/config.schema.json`
**And** the schema enforces snake_case naming conventions to align with Rust and TOML standards

**Given** configuration schema is defined
**When** I create default config files in TOML format
**Then** `global.toml` and `vault.toml` are created in `docs/defaults/` with all fields set to default values
**And** these default files explicitly VALIDATE against the new `config.schema.json` definition to prove correctness

**Given** default config files use multiple formats
**When** I provide format examples
**Then** TOML (primary), JSON, and YAML versions are created showing the same default configuration

**Given** default files exist
**When** I validate against the schema
**Then** all default configs pass schema validation and demonstrate proper structure

**Given** default files use domain types
**When** I verify against Domain models
**Then** the structure maps correctly to the `Config` aggregate defined in Epic 3 (manual verification)

**Given** defaults include all configurable options
**When** I document each field
**Then** inline TOML comments explain purpose, valid values, and defaults for every option

## Story 6.2: Implement Hierarchical Configuration Loading with Figment

As a user configuring lithos,
I want hierarchical configuration that respects precedence rules,
So that I can override settings at different levels (global, user, project, vault).

**Acceptance Criteria:**

**Given** Epic 4 provides unified structured file loading (TOML, JSON, YAML) via FormatDispatcher
**When** I implement hierarchical config using Figment per ADR 0005
**Then** I create `crates/adapters/src/spi/config/figment_loader.rs` implementing the provider pattern

**Given** Figment requires provider pattern
**When** I implement providers
**Then** I create providers for:
- `DefaultsProvider` - compiled-in defaults (lowest priority)
- `FileProvider` - global.toml and vault.toml files
- `EnvProvider` - environment variables with `LITHOS_` prefix
- `CliArgsProvider` - simulated CLI arguments (for testing parity, ready for future CLI integration)

**Given** precedence must be enforced
**When** I configure Figment
**Then** precedence order is: CLI Args > Environment > Vault File > Global File > Defaults
**And** each layer can override any value from lower layers

**Given** vault-level config must override global-level
**When** I implement merging logic
**Then** vault config values take precedence over global config values
**And** merging is deep (nested structures merge field-by-field, not wholesale replacement)

**Given** configuration files are loaded using Epic 4 infrastructure
**When** I integrate FormatDispatcher
**Then** FileProvider uses `FormatDispatcher::parse()` for TOML/JSON/YAML detection
**And** format detection is automatic based on file extension

**Given** Figment preserves metadata
**When** I extract configuration
**Then** Figment metadata includes source location (file path, line, column) for each value
**And** metadata is preserved for error reporting in validation failures

**Given** environment variables need mapping
**When** I implement EnvProvider
**Then** it maps `LITHOS_VAULT_PATH` → `vault.path`, `LITHOS_LOG_LEVEL` → `global.log_level`, etc.
**And** mapping follows snake_case → nested structure conventions

**Given** testing requires CLI args simulation
**When** I implement CliArgsProvider
**Then** it accepts a HashMap<String, String> for test injection
**And** provider is ready for future clap integration without refactoring

## Story 6.3: Implement Structural Configuration Validation

As a user providing configuration,
I want structural validation of the document layout,
So that I am informed of missing sections before the application attempts to process logic.

**Acceptance Criteria:**

**Given** configuration source is parsed into generic data by Figment
**When** I implement `crates/adapters/src/spi/config/validator.rs`
**Then** it performs a structural check to ensure the document contains mandatory top-level sections (e.g., `[global]`, `[vault]`)
**And** provides `miette`-compatible diagnostics for missing structural components

**Given** TOML files have required sections
**When** I implement section validation
**Then** validator checks for `[global]` presence in global.toml (optional if defaults suffice)
**And** validator checks for `[vault]` presence in vault.toml

**Given** fields have required vs optional distinction
**When** I validate structure
**Then** required fields are verified present: `vault.path` (in vault config)
**And** optional fields are allowed to be missing without error

**Given** validation failures must be actionable
**When** I implement error reporting
**Then** validator provides `miette`-compatible diagnostics for missing structural components
**And** diagnostics include file path, line number, column number from Figment metadata

**Given** structural validation passes
**When** the flow continues
**Then** the validated data is ready for `Config::build()` domain validation
**And** structural validation does NOT perform business logic checks (that's domain's job)

**Given** Epic 4 provides ParseError
**When** I handle format parsing errors
**Then** validator converts Epic 4 ParseError to ConfigError::StructuralError with additional context
**And** error messages clearly distinguish syntax errors (invalid TOML) from structural errors (missing sections)

**Given** validation must be testable
**When** I write tests
**Then** tests verify detection of: missing sections, invalid types, malformed TOML
**And** tests verify error messages include exact file/line/column information

## Story 6.4: Implement ConfigCache using Epic 5 Persistence

As a developer requiring configuration persistence and rollback,
I want ConfigCache implemented using Epic 5 Redb adapter,
So that configurations survive process restarts and support rollback.

**Acceptance Criteria:**

**Given** Epic 5 provides `RedbBuilder`, `RedbReader`, `RedbWriter` for disk persistence
**When** I implement ConfigCache in `crates/adapters/src/spi/config/cache.rs`
**Then** it uses `RedbBuilder::new().path(db_path).table_name("config").build()` to create reader/writer pair
**And** constructor `new(db_path: PathBuf)` returns `(RedbReader<String, Config>, RedbWriter<String, Config>)` tuple
**And** ConfigCache wraps the reader/writer pair with domain-specific methods

**Given** ConfigCache needs to store snapshots
**When** I implement snapshot methods
**Then** it provides:
- `save_snapshot(config: &Config, metadata: SnapshotMetadata) -> Result<(), ConfigError>`
- `get_latest() -> Result<Option<(Config, SnapshotMetadata)>, ConfigError>`
- `get_by_timestamp(timestamp: u64) -> Result<Option<(Config, SnapshotMetadata)>, ConfigError>`
- `list_snapshots() -> Result<Vec<SnapshotMetadata>, ConfigError>`

**Given** rollback requires metadata
**When** I define SnapshotMetadata struct
**Then** it contains:
- `version: String` - lithos binary version that created snapshot
- `timestamp: u64` - Unix timestamp (seconds since epoch)
- `source_hash: String` - SHA-256 of concatenated global.toml + vault.toml content
- `source_files: Vec<PathBuf>` - list of config files that were loaded
**And** SnapshotMetadata derives `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`

**Given** Epic 5 `redb::Entry<V>` wraps values with metadata (timestamp, metadata HashMap)
**When** I save snapshots
**Then** snapshot key is formatted as `"snapshot_{timestamp}"`
**And** `RedbWriter::put_with_metadata()` stores Config with metadata HashMap: version, source_hash, source_files (JSON serialized)
**And** Epic 5 `Entry<Config>` structure: `{ value: Config, timestamp: u64, metadata: HashMap<String, String> }`

**Given** snapshot history must be managed
**When** I implement retention logic
**Then** cache retains last 10 snapshots (configurable via constant)
**And** `save_snapshot()` automatically evicts oldest snapshot when limit exceeded
**And** eviction uses timestamp-based ordering (oldest first)

**Given** rollback must be fast
**When** I implement get_latest()
**Then** it retrieves most recent snapshot by finding highest timestamp key
**And** deserialization uses Epic 5 rkyv zero-copy per ADR 0002

**Given** cache operations may fail
**When** I handle errors
**Then** errors are mapped to ConfigError::CacheError(String) with descriptive context
**And** failures are logged via `tracing::error!` with full snapshot metadata

**Given** observability is required
**When** I instrument cache operations
**Then** all methods use `#[tracing::instrument(skip(self, config), level = "debug")]`
**And** events include attributes: operation, snapshot_count, timestamp

## Story 6.5: Implement Config Singleton Registry

As a developer implementing configuration management,
I want a singleton registry for Config access,
So that the entire application references a single merged Config instance with optimal performance.

**Acceptance Criteria:**

**Given** Config domain aggregate exists in `crates/domain/src/config/aggregate.rs`
**When** I implement Registry singleton in `crates/adapters/src/spi/config/registry.rs`
**Then** it uses `Arc<OnceLock<Config>>` pattern for thread-safe, zero-lock reads

**Given** CLI operations need consistent Config access
**When** I implement singleton instance method
**Then** `Registry::global()` returns the same `&'static Registry` instance across all calls
**And** initialization uses `std::sync::Once` to ensure single initialization

**Given** Registry must be initialized
**When** I implement initialization
**Then** `Registry::init(config: Config) -> Result<(), ConfigError>` populates the OnceLock
**And** subsequent calls to `init()` return `ConfigError::AlreadyInitialized`

**Given** Registry provides config access
**When** I implement getter
**Then** `Registry::get() -> Option<&Config>` returns reference to stored config
**And** returns None if not yet initialized (allows checking initialization state)

**Given** multiple concurrent CLI operations access config
**When** I benchmark access patterns
**Then** reads complete in <50ns for warm cache hits (no lock contention)
**And** benchmark confirms zero-lock reads using `cargo bench`

**Given** architecture must remain hexagonal
**When** I implement Registry service
**Then** Config domain contains no singleton logic while Registry manages all infrastructure concerns
**And** Registry is adapter-layer concern, not domain logic

**Given** configuration needs both read performance and mutability
**When** I design the singleton implementation
**Then** implement hybrid approach: `Arc<OnceLock<Config>>` for immutable merged config + `Arc<RwLock<RuntimeState>>` for mutable runtime state
**And** RuntimeState struct contains: `pending_events: Vec<ConfigEvents>` for event accumulation

**Given** RuntimeState needs mutation
**When** I implement event handling
**Then** Registry provides `add_event(event: ConfigEvents)` method using RwLock write lock
**And** Registry provides `drain_events() -> Vec<ConfigEvents>` method using RwLock write lock
**And** event operations are rare compared to config reads (write lock acceptable)

**Given** future hot reloading will be needed for LSP phase
**When** I design the singleton implementation
**Then** Registry supports atomic updates using AtomicPtr swap pattern for future extension
**And** Registry reserves private field: `hot_reload: Option<AtomicPtr<Config>>` (initially None)
**And** implementation includes method stub: `reload(new_config: Config) -> Result<(), ConfigError>` for future LSP integration

**Given** Registry stores merged config
**When** I validate singleton contents
**Then** it holds the final merged `Config` aggregate (not separate Global/Vault instances)
**And** Config is the result of Figment merging + domain validation

**Given** Registry is implemented
**When** I export it in `crates/adapters/src/spi/config/mod.rs`
**Then** internal struct `Registry` is re-exported as `ConfigRegistry`

## Story 6.6: Implement Config Adapters with Command and Query

As a developer completing the configuration bounded context,
I want Command and Query adapters that orchestrate specialized utilities,
So that port implementations are clean and easily extensible for future operations.

**Acceptance Criteria:**

**Given** existing ports in `crates/domain/src/ports/config.rs`
**When** I update the trait definitions
**Then** `load()`, `load_global()`, and `load_vault()` methods are in `ConfigCommand` trait
**And** `rollback()` method is added to `ConfigCommand` trait to restore state from persistent cache
**And** `ConfigQuery` trait retains only side-effect-free methods (e.g., `get() -> Result<&Config, ConfigError>`)

**Given** loading requires file I/O
**When** I implement `Loader` utility in `crates/adapters/src/spi/config/loader.rs`
**Then** it provides read operations using Story 6.2 Figment integration + Epic 4 FormatDispatcher + Epic 4 PathValidator
**And** Loader struct holds references to: figment_loader, path_validator

**Given** Loader integrates Epic 4 security
**When** I implement file loading
**Then** Loader uses `PathValidator::validate()` to check path security before reading files
**And** Loader uses `FormatDispatcher::parse()` for format detection and deserialization

**Given** writing requires file I/O
**When** I implement `Writer` utility in `crates/adapters/src/spi/config/writer.rs`
**Then** it provides write operations (save_global, save_vault) using Epic 4 PathValidator + tokio::fs
**And** Writer uses format-specific serialization (e.g., `toml::to_string`) to ensure output matches source format

**Given** ConfigCommand trait must be implemented
**When** I implement Command adapter in `crates/adapters/src/spi/config/command.rs`
**Then** Command orchestrates the full flow:
1. `Loader` reads and parses files (using Figment from Story 6.2)
2. `Validator` performs structural check (Story 6.3)
3. `Config::build()` performs domain validation (Epic 3)
4. `ConfigCache::save_snapshot()` persists result (Story 6.4)
5. `Registry::init()` updates singleton (Story 6.5)

**Given** Command orchestrates cache integration
**When** I implement load() method
**Then** it uses `ConfigCache` from Story 6.4 to persist snapshots
**And** snapshot metadata includes version, timestamp, source_hash
**And** if cache write fails, log error but don't fail the load operation (cache is non-critical)

**Given** Command must support rollback
**When** I implement rollback() method
**Then** it calls `ConfigCache::get_latest()` to retrieve most recent valid snapshot
**And** it calls `Registry::init()` with restored Config (or updates if already initialized)
**And** it logs the restoration event via `tracing::info!` with snapshot timestamp

**Given** ConfigQuery trait must be implemented
**When** I implement Query adapter in `crates/adapters/src/spi/config/query.rs`
**Then** it implements the `ConfigQuery` trait by delegating to `Registry::get()` for high-performance, side-effect-free reads
**And** Query returns ConfigError::NotInitialized if Registry is not yet initialized

**Given** both adapters are implemented
**When** I test integration
**Then** Command.load() + Query.get() work together for complete configuration management
**And** tests verify: load → cache → registry → query flow

**Given** future extensions are needed (e.g., update_value, lookup_value)
**When** I add methods to Command/Query traits
**Then** adapters can implement new methods by coordinating between Loader/Writer without refactoring I/O layer

**Given** adapters are implemented
**When** I export them in `crates/adapters/src/spi/config/mod.rs`
**Then** internal structs are re-exported with Config prefix:
- `ConfigQuery`, `ConfigCommand`, `ConfigLoader`, `ConfigWriter`, `ConfigRegistry`, `ConfigCache`

**Given** Story 6.1 provides default configuration files
**When** I test the Loader against `docs/defaults/global.toml`
**Then** it successfully loads and deserializes into the Domain Config structure

**Given** error handling must be comprehensive
**When** I implement Command/Query
**Then** errors are mapped appropriately:
- Epic 4 ParseError → ConfigError::ParseError
- Epic 4 PathValidationError → ConfigError::SecurityError
- Validation errors → ConfigError::ValidationError
- Cache errors → ConfigError::CacheError

## Story 6.7: Implement Configuration Versioning and Migration

As a developer maintaining lithos,
I want configuration versioning and migration support,
So that configuration files can evolve safely across versions without breaking user setups.

**Acceptance Criteria:**

**Given** configuration evolves over time
**When** I implement versioning in Global and Vault domain models
**Then** config files include optional `version` field for compatibility checking
**And** version field defaults to current lithos binary version if omitted (backward compatibility)

**Given** version field is present
**When** I implement version detection in Loader
**Then** Loader extracts version from parsed config before calling `Config::build()`
**And** version is compared against current binary version

**Given** version mismatches are detected
**When** I implement migration logic in Loader
**Then** automatic migration transforms old config format to new format before calling `Config::build()`
**And** migration is only attempted for compatible versions (e.g., v0.1 → v0.2, not v0.1 → v2.0)

**Given** breaking changes occur
**When** users upgrade to incompatible version
**Then** clear error messages guide them through manual migration steps via ConfigError::IncompatibleVersion
**And** error includes: old version, new version, link to migration guide

**Given** migration succeeds
**When** I validate migrated config
**Then** migration occurs transparently before domain validation
**And** migrated config is logged via `tracing::warn!` with old and new versions

**Given** migration framework must be extensible
**When** I implement migration infrastructure
**Then** migrations are defined as functions: `fn migrate_v0_1_to_v0_2(old: Value) -> Result<Value, ConfigError>`
**And** migrations are registered in a migration registry keyed by (from_version, to_version)

## Story 6.8: Configuration Error Recovery and Rollback

As a user who has made configuration mistakes,
I want the system to provide clear error messages and recovery options,
So that I can fix configuration issues without losing my work.

**Acceptance Criteria:**

**Given** configuration validation fails
**When** I attempt to load invalid configuration via Command.load()
**Then** clear error messages identify the specific problems via ConfigError and suggest fixes
**And** errors use miette formatting with source file, line, column highlighting

**Given** structural validation fails
**When** I check error handling
**Then** ConfigError::StructuralError includes:
- Missing section name (e.g., "[vault]")
- File path where error occurred
- Suggested fix (e.g., "Add [vault] section to vault.toml")

**Given** validation fails
**When** I check error handling
**Then** the system falls back to default values for invalid settings per `Config::build()` logic
**And** fallback is logged via `tracing::warn!` with field name and default value used

**Given** domain validation fails
**When** Config::build() returns errors
**Then** ConfigError::ValidationError includes:
- Field name that failed validation
- Invalid value provided
- Constraint that was violated (e.g., "vault_path is required")

**Given** previous valid configuration exists
**When** I load invalid configuration
**Then** previous valid configuration is preserved (Registry retains last known good config)
**And** Command.load() returns error without updating Registry

**Given** configuration changes cause system instability
**When** I need to rollback
**Then** Command.rollback() can restore the previous known-good configuration from ConfigCache even after a process restart

**Given** rollback is implemented
**When** I call Command.rollback()
**Then** it retrieves the most recent valid snapshot from ConfigCache using `get_latest()`
**And** it updates Registry with restored Config
**And** it logs the restoration event with snapshot metadata (timestamp, version, source_hash)

**Given** rollback uses ConfigCache snapshots
**When** I specify snapshot selection
**Then** rollback defaults to most recent snapshot
**And** optional API `rollback_to_timestamp(timestamp: u64)` allows selecting specific snapshot

**Given** multiple snapshots are retained
**When** I manage snapshot history
**Then** ConfigCache retains last 10 snapshots as configured in Story 6.4
**And** Command.load() creates new snapshot on every successful load

**Given** fallback to defaults is needed
**When** all snapshots are corrupted or missing
**Then** Command can reinitialize with compiled-in defaults from Story 6.1
**And** operation is logged as emergency fallback

## Story 6.9: Review Epic 6 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 6 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guides during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 6 public components are implemented (Loader, Writer, Validator, Cache, Registry, Command, Query)
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** each ConfigError variant has a test case ensuring proper error propagation

**Given** all Epic 6 public APIs are documented
**When** I verify doc test coverage
**Then** all public components (traits, structs, enums, methods) have runnable doc tests in `# Examples` sections demonstrating usage
**And** doc tests cover both success cases and error handling
**And** doc tests compile and pass when run via `cargo test --doc`

**Given** all Epic 6 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate:
- False positives (tests that pass but don't validate behavior)
- Redundant tests (duplicate coverage)
- Inadequate edge case coverage (error paths, boundary conditions)

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests actually validate business requirements vs implementation details
**And** tests verify contract adherence (port semantics) not internal state

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 6 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify:
- Tests use proper fixtures (test data builders, sample config files)
- Tests avoid flaky behavior (no timing dependencies, no hard-coded sleep)
- Test intent is clear (descriptive names, Given/When/Then structure in comments)

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code
**And** complex test scenarios have inline documentation explaining setup

**Given** Figment integration must be tested
**When** I review hierarchical loading tests
**Then** tests verify correct precedence: CLI > Env > Vault File > Global File > Defaults
**And** tests verify deep merging (nested structures merge field-by-field)

**Given** ConfigCache persistence must be validated
**When** I test cache behavior
**Then** integration tests verify:
- Snapshot survives process restart (save, drop cache, recreate, load)
- Metadata is preserved across reads/writes
- Rollback retrieves correct snapshot by timestamp

**Given** documentation quality is critical
**When** I review all doc comments
**Then** every public component has:
- Precise `///` doc comments explaining purpose and behavior
- Well-written doc tests in `# Examples` sections
- Error cases documented with `# Errors` sections where applicable
- Panic conditions documented with `# Panics` sections where applicable
**And** doc tests demonstrate realistic usage patterns
**And** doc comments follow project standards from `project-context.md`

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

## Story 6.10: Document Configuration System

As a user configuring lithos,
I want comprehensive documentation for configuration options and comprehensive doc comments,
So that I can understand and customize lithos behavior effectively.

**Acceptance Criteria:**

**Given** all Epic 6 code is implemented
**When** I review all doc comments
**Then** they are accurate, precise, and follow project standards from `project-context.md`
**And** every public component uses proper `///` documentation format

**Given** all Epic 6 public components are documented
**When** I verify doc comments
**Then** all public traits, structs, enums, functions, and methods have:
- Clear `///` doc comments explaining their purpose
- `# Examples` sections with runnable, well-written doc tests
- `# Errors` sections documenting error conditions where applicable
- `# Panics` sections documenting panic conditions where applicable
**And** doc tests demonstrate realistic usage patterns
**And** doc tests compile and pass via `cargo test --doc`

**Given** configuration system is implemented
**When** I create user documentation
**Then** it includes all configuration options with examples and defaults
**And** user docs are created at `docs/configuration.md`

**Given** user documentation exists
**When** I check completeness
**Then** it covers:
- Hierarchical loading (precedence rules)
- All configuration fields (global and vault)
- Validation rules and constraints
- Troubleshooting common errors
- Rollback and recovery procedures

**Given** Epic 6 adapters are implemented
**When** I create developer documentation
**Then** `docs/adapters/config-adapters.md` is created following the pattern from `docs/domain-models.md` (Epic 3)

**Given** developer documentation exists
**When** I validate completeness
**Then** it covers:
- Adapter architecture (Command/Query pattern)
- Composition pattern (Loader/Writer/Validator/Cache/Registry)
- Epic 4 integration (FormatDispatcher, PathValidator)
- Epic 5 integration (RedbCache for ConfigCache)
- Singleton Registry design and zero-lock reads
- Figment hierarchical loading (provider pattern)
- Extension guidelines (adding new config fields, migration functions)

**Given** Epic 6 module structure is finalized
**When** I create module-level documentation
**Then** `crates/adapters/src/spi/config/README.md` is created with:
- Module structure overview (query.rs, command.rs, loader.rs, writer.rs, registry.rs, validator.rs, cache.rs, figment_loader.rs)
- File purposes and responsibilities
- Quick start examples (basic load, rollback, querying)

**Given** module README exists
**When** I validate content
**Then** it provides quick reference for developers navigating the config adapter module
**And** links to comprehensive docs (`docs/adapters/config-adapters.md`)

**Given** users and developers read the documentation
**When** they work with configuration
**Then** they can successfully customize behavior and extend adapters without assistance
