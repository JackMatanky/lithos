# Story 3.1: Create Config Bounded Context

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer managing application configuration,
I want a Config domain model with validation,
So that configuration changes are validated and the domain enforces configuration integrity.

## Acceptance Criteria

**Given** I have researched configuration merging patterns
**When** I review the Config bounded context
**Then** Config supports merging VaultConfig and GlobalConfig with business rules

**Given** Config entity is defined
**When** I check validation integration
**Then** semantic validation ensures configuration integrity and type safety

**Given** configuration patterns are established
**When** I validate the design
**Then** Config supports encrypted sensitive fields and validation rules

**Given** the Config bounded context is defined
**When** I check domain events
**Then** ConfigUpdated event is emitted for configuration changes

**Given** hierarchical merging is needed
**When** I implement merging in domain
**Then** vault-level config overrides global-level (Vault > Global business rule)

**Given** CQRS separation is needed
**When** I define ports
**Then** ConfigCommand and ConfigQuery trait interfaces are provided for future implementation

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Config Domain Tests First (RED Phase - AC: All)
- [ ] **STRICT NAMING:** Mandate verb-first behavioral naming for config validation, merging, and structure tests
- [ ] Write failing unit tests for Config entity (hierarchical structure, validation, encryption)
- [ ] Write failing unit tests for ConfigValue enum (string, number, boolean, encrypted fields)
- [ ] Write failing unit tests for VaultConfig and GlobalConfig structures
- [ ] Write failing unit tests for semantic validation (type safety, required fields, constraints)
- [ ] Write failing property-based tests for merging logic and validation boundaries
- [ ] Write failing integration tests for encrypted field handling and validation
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)
- [ ] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 2: Implement Config Domain Entities (GREEN Phase - AC: 1-3)
- [ ] Create file `crates/domain/src/models/config.rs` and implement Config entities with merging logic
- [ ] **DOMAIN BUSINESS LOGIC:** Config defines structure, validation, AND merging precedence (Vault > Global)
- [ ] **SEPARATE LEVELS:** Define VaultConfig and GlobalConfig structs for each configuration level
- [ ] Define VaultConfig struct: `#[derive(Debug, Clone, PartialEq)] pub struct VaultConfig { pub filesystem: FileSystemConfig, pub frontmatter: FrontmatterConfig }`
- [ ] Define GlobalConfig struct: `#[derive(Debug, Clone, PartialEq)] pub struct GlobalConfig { pub filesystem: FileSystemConfig, pub frontmatter: FrontmatterConfig }`
- [ ] Define merged Config struct: `#[derive(Debug, Clone, PartialEq)] pub struct Config { pub filesystem: FileSystemConfig, pub frontmatter: FrontmatterConfig }`
- [ ] Define FileSystemConfig struct: `#[derive(Debug, Clone, PartialEq)] pub struct FileSystemConfig { pub vault_path: String, pub templates_dir: String, pub schemas_dir: String, pub property_bank_file: String, pub cache_dir: String, pub log_level: String }`
- [ ] Define FrontmatterConfig struct: `#[derive(Debug, Clone, PartialEq)] pub struct FrontmatterConfig { pub file_class_key: String, pub title_key: String, pub alias_key: String, pub date_created_key: String, pub date_modified_key: String }`
- [ ] Implement Config::merge() method: `pub fn merge(global: GlobalConfig, vault: VaultConfig) -> Result<Self, ConfigError>` with Vault > Global precedence
- [ ] Implement Config::new() constructor with validation, returns `Result<Self, ConfigError>`
- [ ] Implement Config::validate() method for business rule validation, returns `Result<(), ConfigError>`
- [ ] Set defaults: filesystem defaults (vault_path=".", templates_dir="templates/", etc.), frontmatter defaults (file_class_key="file_class", title_key="title", etc.)
- [ ] Define ConfigValue enum with `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub enum ConfigValue { String(String), Number(f64), Boolean(bool), Encrypted(Vec<u8>), Array(Vec<ConfigValue>), Object(HashMap<String, ConfigValue>) }`
- [ ] Implement From traits: `impl From<String> for ConfigValue`, `impl From<f64> for ConfigValue`, `impl From<bool> for ConfigValue`
- [ ] **DOMAIN MERGING:** Merging precedence is business logic, belongs in domain; adapters handle I/O only
- [ ] **TDD REQUIREMENT:** Make all Config tests pass (including merging logic, GREEN phase complete when all tests pass)

### Task 3: Implement Domain Error Types (GREEN Phase - AC: All)
- [ ] Implement comprehensive ConfigError enum with thiserror::Error derives
- [ ] Add error variants for validation failures (type mismatches, missing required fields)
- [ ] Add error variants for hierarchical issues (circular dependencies, invalid paths)
- [ ] Add error variants for encryption failures (decryption errors, invalid keys)
- [ ] Implement error conversion traits and proper error chaining
- [ ] Write unit tests for error message clarity and proper error handling
- [ ] **TDD REQUIREMENT:** All error-related tests must pass

### Task 4: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Optimize hierarchical merging performance (pre-allocated collections, efficient string handling)
- [ ] Implement memory-efficient config storage patterns (avoid cloning large structures)
- [ ] Add comprehensive documentation with invariants, examples, and error conditions
- [ ] Ensure hexagonal architecture compliance (no external dependencies in domain)
- [ ] Add performance optimizations for config validation and merging operations
- [ ] Verify proper ownership patterns and borrowing rules for hierarchical data
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 5: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for all Config domain entities and validation logic
- [ ] Create test fixtures module with hierarchical config examples and edge cases
- [ ] Implement property-based testing for hierarchical merging and validation boundaries
- [ ] Add integration tests for encrypted field handling and decryption workflows
- [ ] Add performance benchmarks for config loading and validation (<100μs target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 6: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update domain crate lib.rs with Config module public exports
- [ ] Add comprehensive doc comments with hierarchical examples and validation rules
- [ ] Ensure integration points with future Epic 5 (configuration loading) and Epic 6 (schema validation)
- [ ] Update Cargo.toml with required dependencies (serde for serialization, optional encryption crates)
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 8: Implement Domain Events (GREEN Phase - AC: All)
- [ ] Define ConfigUpdated domain event for configuration changes
- [ ] Add event emission points in Config entity methods
- [ ] Ensure events follow domain event naming conventions
- [ ] **TDD REQUIREMENT:** Make all domain event tests pass

### Task 9: Define CQRS Ports (GREEN Phase - AC: All)
- [ ] Define ConfigCommand trait interface (shell for future implementation)
- [ ] Define ConfigQuery trait interface (shell for future implementation)
- [ ] Ensure ports are placed in domain ports module
- [ ] **TDD REQUIREMENT:** Make all port interface tests pass

### Task 10: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch hierarchical merging edge cases
- [ ] **TDD VALIDATION:** Ensure performance benchmarks meet targets (<100μs config operations)
- [ ] **TDD VALIDATION:** Verify encryption/decryption works for sensitive config fields
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all domain entities pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (config domain purity)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement config bounded context with hierarchical validation, encryption, domain events, and CQRS ports`

## Technical Requirements

### Domain Model Foundation

**Core Entity Structure:**
- **ConfigValue Enum**: Unified representation for all configuration value types
- **VaultConfig Struct**: Configuration from vault-specific files
- **GlobalConfig Struct**: Configuration from global defaults
- **Config Struct**: Merged result with business rules (Vault overrides Global)
- **Immutability**: All config entities MUST be immutable following Rust ownership patterns
- **Validation**: Business rule validation with merging precedence
- **Error Handling**: Use `thiserror::Error` for typed configuration errors

**Configuration Merging - CRITICAL:**
- **Business Rule:** Vault configuration overrides Global configuration
- **Domain Responsibility:** Merging logic belongs in domain as business rules
- **Adapter Responsibility:** File loading and parsing belong in adapters

```rust
/// Configuration value types
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Encrypted(Vec<u8>),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

/// Vault-specific configuration
#[derive(Debug, Clone, PartialEq)]
pub struct VaultConfig {
    pub filesystem: FileSystemConfig,
    pub frontmatter: FrontmatterConfig,
}

/// Global default configuration
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalConfig {
    pub filesystem: FileSystemConfig,
    pub frontmatter: FrontmatterConfig,
}

/// Merged configuration result
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub filesystem: FileSystemConfig,
    pub frontmatter: FrontmatterConfig,
}

impl Config {
    /// Merge Vault and Global configs with business rules
    /// Vault overrides Global (business requirement)
    pub fn merge(global: GlobalConfig, vault: VaultConfig) -> Result<Self, ConfigError> {
        // Business logic: vault takes precedence
        let filesystem = merge_filesystem(global.filesystem, vault.filesystem);
        let frontmatter = merge_frontmatter(global.frontmatter, vault.frontmatter);
        Ok(Config { filesystem, frontmatter })
    }
}
```

**Separation Benefits:**
- Business rules (precedence) in domain
- Infrastructure (file I/O) in adapters
- Clear hexagonal boundaries maintained

### Encryption Support

**Encrypted Fields - CRITICAL:**
- **AES-256-GCM**: Industry-standard encryption for sensitive configuration
- **Key Derivation**: PBKDF2 or Argon2 for key derivation from passwords
- **Key Storage**: Keys stored separately from encrypted data (adapter layer)
- **Domain Layer**: Domain stores encrypted blobs, encryption/decryption in adapter
- **Error Handling**: Clear error messages for decryption failures

**Encryption Flow:**
```
Raw Sensitive Value → Encrypt (Adapter) → Encrypted Blob → Store in Config
Stored Config → Decrypt (Adapter) → Raw Value → Domain Logic
```

### Validation Rules

**Semantic Validation - CRITICAL:**
- **Type Safety**: Configuration values must match expected types
- **Required Fields**: Critical configuration fields must be present
- **Value Constraints**: Numeric ranges, string patterns, enum values
- **Cross-Field Validation**: Dependencies between configuration fields
- **Hierarchical Consistency**: Overrides must maintain type compatibility

**Validation Rules:**
- Business rule validation for config structure and consistency
- Type safety checks for ConfigValue variants
- Required field validation for critical settings
- Custom validation rules as needed for specific config fields

### Architecture Compliance - MANDATORY READING

**Hexagonal Boundary Enforcement:**
- Domain crate contains Config entities, validation, and merging business logic
- Adapter crate handles file I/O, parsing, and calls domain merging methods
- Application layer orchestrates configuration loading and usage
- NO file I/O in domain (merging logic is business rules, not infrastructure)

**Standard Traits - REQUIRED:**
```rust
// ALWAYS derive these for domain entities:
#[derive(Debug, Clone, PartialEq)]
// Add validation methods for business rules
// Keep domain focused on business logic
```

**Exhaustive Matching:**
```rust
// Use #[non_exhaustive] on domain enums
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Encrypted(Vec<u8>),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

// PROHIBIT catch-all patterns in domain logic:
match value {
    ConfigValue::String(s) => { /* validate string */ },
    ConfigValue::Number(n) => { /* validate number */ },
    ConfigValue::Boolean(b) => { /* validate bool */ },
    // NO: _ => {} catch-alls!
}
```

**Error Standards:**
- Use `thiserror` for domain error types
- Every error variant must have descriptive message
- Use `#[from]` attribute for error conversions
- NO `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code

**Required Error Variants:**
```rust
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("Configuration validation failed: {field} - {message}")]
    ValidationFailed { field: String, message: String },

    #[error("Required configuration field missing: {field}")]
    MissingRequiredField { field: String },

    #[error("Invalid configuration value type for {field}: expected {expected}, got {actual}")]
    InvalidType { field: String, expected: String, actual: String },

    #[error("Configuration value out of range for {field}: {value} not in {min:?}..{max:?}")]
    OutOfRange { field: String, value: f64, min: Option<f64>, max: Option<f64> },

    #[error("Invalid enum value for {field}: {value} not in {allowed:?}")]
    InvalidEnumValue { field: String, value: String, allowed: Vec<String> },

    #[error("Configuration dependency violation: {field} requires {depends_on}")]
    DependencyViolation { field: String, depends_on: String },

    #[error("Encryption error for field {field}: {message}")]
    EncryptionError { field: String, message: String },

    #[error("Configuration merge conflict: {field} has incompatible types at {path1} and {path2}")]
    MergeConflict { field: String, path1: String, path2: String },
}
```

### Testing Requirements

**Hexagonal Testing Hierarchy:**

**Domain Tests (Pure Unit Tests):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_merge_vault_overrides_global() {
        // Test business rule: Vault overrides Global
        let global = GlobalConfig { /* global defaults */ };
        let vault = VaultConfig { /* vault overrides */ };

        let merged = Config::merge(global, vault).unwrap();

        // Assert vault values take precedence
        assert_eq!(merged.some_field, vault_expected_value);
    }

    #[test]
    fn test_config_validation_rules() {
        let config = Config::new(/* ... */);
        let result = config.validate();

        // Test business validation rules
        assert!(result.is_ok());
    }
}
```

**Test Coverage Target:**
- **90%+ coverage** for Config domain entities and validation logic (per Epic 3 AC)
- Test both success and error cases for all validation rules
- Property-based testing for hierarchical merging and validation edge cases
- Deterministic testing with fixed test data

**Test Fixtures Strategy:**
```rust
#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub fn sample_global_config() -> GlobalConfig {
        GlobalConfig {
            filesystem: FileSystemConfig {
                vault_path: ".".to_string(),
                templates_dir: "templates".to_string(),
                // ... other defaults
            },
            frontmatter: FrontmatterConfig {
                file_class_key: "file_class".to_string(),
                title_key: "title".to_string(),
                // ... other defaults
            },
        }
    }

    pub fn sample_vault_config() -> VaultConfig {
        VaultConfig {
            filesystem: FileSystemConfig {
                vault_path: "/vault".to_string(),
                templates_dir: "custom_templates".to_string(),
                // ... vault overrides
            },
            frontmatter: FrontmatterConfig {
                file_class_key: "type".to_string(), // vault override
                title_key: "title".to_string(),
                // ... other settings
            },
        }
    }
}
```

**Performance Testing:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_config_merge(c: &mut Criterion) {
    let global = fixtures::sample_global_config();
    let vault = fixtures::sample_vault_config();

    c.bench_function("config_merge_business_logic", |b| {
        b.iter(|| {
            black_box(Config::merge(black_box(&global), black_box(&vault)));
        });
    });
    // Target: <100μs for typical config merges
}
```

### File Structure Requirements

**File Structure (Single File per Context - Split at 1000+ Lines):**
```
crates/domain/src/
├── lib.rs                    # Public API surface, re-exports
├── models/
│   ├── mod.rs               # Module declarations
│   └── config.rs            # Config entities: VaultConfig, GlobalConfig, Config (merged),
│                           # FileSystemConfig, FrontmatterConfig, ConfigValue, merging logic
├── ports/
│   ├── mod.rs               # Port trait declarations
│   └── config.rs            # ConfigCommand/ConfigQuery traits (shells)
└── errors.rs                # Domain errors (EXTENDED with config errors)
```

**Splitting Guideline:** Start with single file. Split when >1000 lines into config_levels.rs, config_core.rs, config_merging.rs.

**Implementation Decision:**
Use **subfolder organization** for Config bounded context due to complexity of hierarchical merging, validation rules, and encryption support.

**Naming Conventions - STRICT:**
- Files: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Traits: `PascalCase` with `Port` suffix for ports

### Code Quality Standards

**Clippy Complexity Limits - ENFORCED:**
- Cognitive complexity: **max 25 (deny)**
- Function length: **max 100 lines (deny)**
- Keep hierarchical merging logic composable

**MANDATORY Quality Gates (Task 7):**
- **NO EXCEPTIONS:** All clippy warnings MUST be fixed (no bypassing)
- **NO EXCEPTIONS:** All pre-commit hooks MUST pass (no bypassing)
- **MANDATORY:** Run `mise run verify` for comprehensive quality assurance
- **MANDATORY:** Commit blocked until all quality gates pass
- **MANDATORY:** Conventional commit message required for final commit

**Formatting:**
- Run `mise run verify` before committing
- Pre-commit hooks enforce formatting
- Import grouping: `StdExternalCrate`

## Dev Notes

### Project Context Integration

**Current Codebase State:**
- Workspace structure exists at `crates/domain/`, `crates/app/`, `crates/adapters/`, `crates/cli/`
- Domain crate has basic error types and ports structure
- Story 3.1 (Note) and 3.2 (Schema) completed - Config is the third domain model
- Epic 5 will implement configuration loading using this domain model

**Technology Stack (from project-context.md):**
- **Rust 1.92+**: Memory safety, zero-cost abstractions
- **serde 1.0**: Configuration serialization/deserialization
- **thiserror 2.0**: Structured domain error definitions
- **AES-256-GCM**: For sensitive field encryption (adapter layer)
- **Argon2/PBKDF2**: For key derivation (adapter layer)

**Critical Anti-Patterns to AVOID:**
- ❌ Using `unwrap()`, `expect()`, `todo()`, `panic!()` in production code
- ❌ Using `as` casting (use `.try_into().expect("...")` or proper error handling)
- ❌ Leaking encryption logic into domain (only encrypted blob storage)
- ❌ Creating ad-hoc conversion methods instead of From/TryFrom traits
- ❌ Using catch-all `_ => {}` patterns in exhaustive domain logic matches

### Architecture Intelligence

**Configuration Requirements:**
- **Global Level**: System-wide defaults (read-only for users)
- **Vault Level**: Vault-specific configuration (highest precedence)
- **Business Rule**: Vault configurations override Global configurations

**Performance Targets (from Architecture):**
- Config loading: <500ms for typical configurations
- Hierarchical merging: <100μs for standard config sets
- Validation: <50μs for typical configurations
- Memory usage: Bounded under 50MB for large configurations

**Security Requirements:**
- Encrypted fields for sensitive data (API keys, passwords, tokens)
- Key separation from encrypted data
- Secure key derivation from user passwords
- Clear error messages without exposing encryption internals

### Implementation Strategy

**TDD Business Rule Merging:**
```rust
impl Config {
    /// Merge configurations with business rules (Vault overrides Global)
    pub fn merge(global: GlobalConfig, vault: VaultConfig) -> Result<Self, ConfigError> {
        // Business logic: vault takes precedence over global
        let filesystem = merge_filesystem(global.filesystem, vault.filesystem)?;
        let frontmatter = merge_frontmatter(global.frontmatter, vault.frontmatter)?;

        Ok(Config { filesystem, frontmatter })
    }
}

fn merge_filesystem(global: FileSystemConfig, vault: FileSystemConfig) -> Result<FileSystemConfig, ConfigError> {
    // Business rules for filesystem config merging
    Ok(FileSystemConfig {
        vault_path: vault.vault_path, // vault-specific
        templates_dir: vault.templates_dir.or(global.templates_dir), // vault overrides
        // ... other fields with precedence rules
    })
}
```

**Encryption Domain Boundary:**
```rust
// Domain stores encrypted blobs
pub struct EncryptedField {
    pub encrypted_data: Vec<u8>,
    pub key_id: String,  // Reference to encryption key (adapter manages)
}

// Adapter handles encryption/decryption
impl ConfigAdapter {
    pub async fn encrypt_field(&self, plaintext: &str) -> Result<EncryptedField, ConfigError> {
        // Use AES-256-GCM with derived key
        // Return encrypted blob for domain storage
        unimplemented!("Adapter implementation")
    }

    pub async fn decrypt_field(&self, field: &EncryptedField) -> Result<String, ConfigError> {
        // Decrypt using appropriate key
        // Return plaintext for domain use
        unimplemented!("Adapter implementation")
    }
}
```

### Cross-Story Dependencies

**Prerequisites:**
- ✅ Epic 1 completed (workspace, tooling, quality gates)
- ✅ Story 3.1 completed (Note bounded context established patterns)
- ✅ Story 3.2 ready (Schema bounded context for validation)

**Enables Future Stories:**
- **Epic 5**: Configuration loading and management (uses this domain model)
- **Epic 6**: Schema validation (configuration drives schema loading)
- **Epic 9**: Vault operations (vault-specific configuration)
- **Epic 10**: Query operations (configuration affects query behavior)

**Integration Points:**
- **Configuration Loading (Epic 5)**: Adapters load TOML files into Config domain model
- **Schema System (Epic 6)**: Configuration specifies schema file locations and validation rules
- **Template System (Epic 11)**: Configuration provides template pack locations and settings
- **CLI (Epic 13)**: Configuration drives CLI behavior, help text, and command options

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for configuration entities, mock file loaders, and performance benchmarking utilities

### References

**Architecture Documents:**
- [Source: _bmad-output/planning-artifacts/architecture.md#Configuration Management]
  - Hierarchical TOML configuration (global → user → project) with validation
  - Security requirements for encrypted sensitive TOML sections
  - CQRS architecture patterns for configuration management
- [Source: _bmad-output/planning-artifacts/architecture.md#Hexagonal Architecture]
  - Domain ports and adapter implementation patterns
  - CQRS separation and async trait requirements
  - Error handling and validation layer separation

**Epic Context:**
- [Source: _bmad-output/planning-artifacts/epics/epic-3-core-domain-models-value-objects-phase-15.md#Story 3.3]
  - Complete acceptance criteria for Config bounded context
  - Hierarchical structure requirements (Global → User → Project → Vault)
  - Semantic validation for configuration integrity and type safety
  - Encrypted sensitive fields and validation rules support

**PRD Requirements:**
- [Source: _bmad-output/planning-artifacts/prd.md#Configuration Management FR26-FR29]
  - TOML-based configuration file support with hierarchical loading
  - Schema-driven validation and enterprise audit features
  - Encrypted sensitive configuration fields

**Previous Story Learnings:**
- [Source: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md]
  - Domain entity patterns with validation and error handling
  - TDD framework implementation with red-green-refactor phases
  - File structure conventions and naming standards
  - Quality assurance task structure and requirements

**Project Context:**
- [Source: _bmad-output/project-context.md#Critical Implementation Rules]
  - Hexagonal architecture boundary enforcement
  - Async trait patterns and tokio integration
  - Error handling with thiserror and proper error chaining
  - Quality gates (clippy cognitive complexity <25, no unwrap/expect)

## Dev Agent Record

### Agent Model Used

<!-- Dev agent will fill this in during implementation -->

### Debug Log References

<!-- Dev agent will add references to logs if debugging is needed -->

### Completion Notes List

<!-- Dev agent will document completion status and any deviations -->

### File List

<!-- Dev agent will list all files created/modified during implementation -->
```
Expected files to be created (7 TDD tasks for 3-3):
- crates/domain/src/errors.rs (EXTENDED with config error variants)
- crates/domain/src/models/mod.rs (UPDATED with config module declaration)
- crates/domain/src/models/config/mod.rs (re-exports Config, ConfigValue, ConfigPath)
- crates/domain/src/models/config/config.rs (Config entity with hierarchical merging)
- crates/domain/src/models/config/value.rs (ConfigValue enum and conversions)
- crates/domain/src/models/config/path.rs (ConfigPath enum and hierarchy logic)
- crates/domain/src/ports/config.rs (ConfigPort trait - future adapter integration)
- crates/domain/src/lib.rs (UPDATED with public config re-exports)
- crates/domain/Cargo.toml (UPDATED with serde dependency)
- benches/config_benchmarks.rs (performance benchmarks - optional)

Comprehensive tests in each file with #[cfg(test)] modules (90%+ coverage target)
```
