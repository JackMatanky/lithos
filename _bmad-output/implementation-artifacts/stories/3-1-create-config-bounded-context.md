# Story 3.1: Create Config Bounded Context

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer managing application configuration,
I want a Config domain model with validation,
So that configuration changes are validated and the domain enforces configuration integrity.

## Acceptance Criteria

**Given** I have researched hierarchical configuration patterns
**When** I review the Config bounded context
**Then** Config entity supports hierarchical structure (Global → User → Project → Vault)

**Given** Config entity is defined
**When** I check validation integration
**Then** semantic validation ensures configuration integrity and type safety

**Given** configuration patterns are established
**When** I validate the design
**Then** Config supports encrypted sensitive fields and validation rules

**Given** the Config bounded context is defined
**When** I check domain events
**Then** ConfigUpdated event is emitted for configuration changes

**Given** CQRS separation is needed
**When** I define ports
**Then** ConfigCommand and ConfigQuery trait interfaces are provided for future implementation

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Config Domain Tests First (RED Phase - AC: All)
- [ ] **STRICT NAMING:** Mandate verb-first behavioral naming for complex merge-precedence and hierarchical tests
- [ ] Write failing unit tests for Config entity (hierarchical structure, validation, encryption)
- [ ] Write failing unit tests for ConfigValue enum (string, number, boolean, encrypted fields)
- [ ] Write failing unit tests for ConfigPath handling (Global/User/Project/Vault hierarchy)
- [ ] Write failing unit tests for semantic validation (type safety, required fields, constraints)
- [ ] Write failing property-based tests for hierarchical merging and override logic
- [ ] Write failing integration tests for encrypted field handling and validation
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)
- [ ] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 2: Implement Config Domain Entities (GREEN Phase - AC: 1-3)
- [ ] Create file `crates/domain/src/models/config.rs` and implement all Config entities in single file
- [ ] Define phantom type markers: `#[derive(Debug)] pub struct Global; #[derive(Debug)] pub struct User; #[derive(Debug)] pub struct Project; #[derive(Debug)] pub struct Vault;`
- [ ] Define ConfigValue enum with `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub enum ConfigValue { String(String), Number(f64), Boolean(bool), Encrypted(Vec<u8>), Array(Vec<ConfigValue>), Object(HashMap<String, ConfigValue>) }`
- [ ] Implement From traits: `impl From<String> for ConfigValue`, `impl From<f64> for ConfigValue`, `impl From<bool> for ConfigValue`
- [ ] Define ConfigPath enum: `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub enum ConfigPath<Level = Global> { Global(PhantomData<Global>), User(PhantomData<User>), Project(PhantomData<Project>), Vault(PhantomData<Vault>) }`
- [ ] Define type-safe aliases: `pub type GlobalPath = ConfigPath<Global>; pub type UserPath = ConfigPath<User>; pub type ProjectPath = ConfigPath<Project>; pub type VaultPath = ConfigPath<Vault>;`
- [ ] Implement ConfigPath methods: `precedence_order() -> Vec<ConfigPath>` returning [Global, User, Project, Vault], `is_higher_precedence(&self, other: &ConfigPath<Level>) -> bool`
- [ ] Define ValidationRule enum: `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub enum ValidationRule { Required, Enum(Vec<String>), Range { min: Option<f64>, max: Option<f64> }, Pattern(String), DependsOn(String) }`
- [ ] Define Config struct: `#[derive(Debug, Clone, PartialEq)] pub struct Config<Level = Global> { pub values: HashMap<ConfigPath<Level>, HashMap<String, ConfigValue>>, pub validation_rules: HashMap<String, ValidationRule>, pub encrypted_fields: HashSet<String>, _marker: PhantomData<Level> }`
- [ ] Define type-safe config aliases: `pub type GlobalConfig = Config<Global>; pub type UserConfig = Config<User>; pub type ProjectConfig = Config<Project>; pub type VaultConfig = Config<Vault>;`
- [ ] Implement Config::new() constructor that validates all inputs and returns `Result<Self, ConfigError>`
- [ ] Implement Config::merge_hierarchical() method that merges configs with Vault > Project > User > Global precedence, returns `Result<HashMap<String, ConfigValue>, ConfigError>`
- [ ] Implement Config::validate() method that applies all ValidationRule constraints to merged config, returns `Result<(), ConfigError>`
- [ ] Implement Config::get() method with automatic hierarchical fallback (Vault -> Project -> User -> Global), returns `Option<&ConfigValue>`
- [ ] Implement Config::decrypt_field() method that returns error if field not encrypted, actual decryption in adapter layer
- [ ] Define EncryptedField struct: `#[derive(Debug, Clone, PartialEq)] pub struct EncryptedField { pub encrypted_data: Vec<u8>, pub key_id: String }`
- [ ] **TDD REQUIREMENT:** Make all Config tests pass (GREEN phase complete when all tests pass)

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
- **ConfigPath Enum**: Hierarchical path levels (Global, User, Project, Vault)
- **Config Struct**: Main entity with hierarchical merging and validation
- **Immutability**: All config entities MUST be immutable following Rust ownership patterns
- **Validation**: Hierarchical merging with type safety and constraint validation
- **Error Handling**: Use `thiserror::Error` for typed configuration errors

**Hierarchical Structure - CRITICAL:**
- **RULE 82 COMPLIANCE:** All paths MUST be managed via Figment/project-root. **PROHIBITED:** `std::env::current_dir` usage.
```rust
/// Configuration value types supporting hierarchical merging
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigValue {
    /// String configuration value
    String(String),
    /// Numeric configuration value
    Number(f64),
    /// Boolean configuration value
    Boolean(bool),
    /// Encrypted sensitive value (stored as encrypted blob)
    Encrypted(Vec<u8>),
    /// Array of configuration values
    Array(Vec<ConfigValue>),
    /// Object/map of configuration key-value pairs
    Object(HashMap<String, ConfigValue>),
}

/// Hierarchical configuration levels with override precedence
/// Enhanced with phantom types for compile-time context safety
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigPath<Level = Global> {
    /// System-wide global configuration
    Global(PhantomData<Global>),
    /// User-specific configuration
    User(PhantomData<User>),
    /// Project-specific configuration
    Project(PhantomData<Project>),
    /// Vault-specific configuration (highest precedence)
    Vault(PhantomData<Vault>),
}

// Phantom type markers for context safety
pub struct Global;
pub struct User;
pub struct Project;
pub struct Vault;

// Type-safe config path aliases
pub type GlobalPath = ConfigPath<Global>;
pub type UserPath = ConfigPath<User>;
pub type ProjectPath = ConfigPath<Project>;
pub type VaultPath = ConfigPath<Vault>;

/// Main configuration entity with hierarchical support
/// Uses phantom types to prevent mixing incompatible config contexts
#[derive(Debug, Clone, PartialEq)]
pub struct Config<Level = Global> {
    /// Configuration values by hierarchical level (type-safe)
    values: HashMap<ConfigPath<Level>, HashMap<String, ConfigValue>>,
    /// Validation rules and constraints
    validation_rules: HashMap<String, ValidationRule>,
    /// Encrypted field metadata
    encrypted_fields: HashSet<String>,
    _marker: PhantomData<Level>,
}

// Type-safe config aliases
pub type GlobalConfig = Config<Global>;
pub type UserConfig = Config<User>;
pub type ProjectConfig = Config<Project>;
pub type VaultConfig = Config<Vault>;
```

**Hierarchical Merging Algorithm:**
1. Start with Global level configuration
2. Override with User level (same keys) - compile-time guaranteed type compatibility
3. Override with Project level (same keys) - phantom types prevent mixing
4. Override with Vault level (highest precedence) - type-safe precedence
5. Apply validation rules to merged configuration
6. Decrypt encrypted fields on access

**Phantom Type Benefits:**
- Compile-time prevention of mixing config contexts (e.g., can't merge Global with User directly)
- Type-safe APIs: functions can accept specific config types
- Zero-cost abstraction: phantom types have no runtime overhead

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

**Validation Rule Types:**
```rust
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ValidationRule {
    /// Field must be present in final merged configuration
    Required,
    /// Field value must match one of these string values
    Enum(Vec<String>),
    /// Numeric field must be within this range
    Range { min: Option<f64>, max: Option<f64> },
    /// String field must match this regex pattern
    Pattern(String),
    /// Field depends on another field being present/set
    DependsOn(String),
}
```

### Architecture Compliance - MANDATORY READING

**Hexagonal Boundary Enforcement:**
- Domain crate in `crates/domain/src/` with Config entities and validation
- Adapter crate in `crates/adapters/src/` with encryption and file I/O
- Application layer orchestrates hierarchical loading and merging
- NO encryption logic in domain (only encrypted blob storage)
- NO file I/O in domain layer (only configuration value types)

**Standard Traits - REQUIRED:**
```rust
// ALWAYS derive these for domain entities:
#[derive(Debug, Clone, PartialEq)]
// Add Serialize/Deserialize for config persistence
// Use custom implementations for complex validation

// Advanced Rust Patterns:
// - Use phantom types for compile-time context safety
// - Leverage associated types in port traits for type-safe operations
```

**Conversion Traits - MANDATORY:**
- Use `From/Into` for converting between config levels and merged config
- Use `TryFrom/TryInto` for validation during config construction
- NEVER create ad-hoc `to_x()` methods

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
    fn test_config_hierarchical_merge() {
        // Test Global → User → Project → Vault override precedence
        let global = ConfigValue::String("global_value".to_string());
        let user = ConfigValue::String("user_value".to_string());
        let vault = ConfigValue::String("vault_value".to_string());

        let merged = Config::merge_hierarchical(vec![
            (ConfigPath::Global, HashMap::from([("key".to_string(), global)])),
            (ConfigPath::User, HashMap::from([("key".to_string(), user)])),
            (ConfigPath::Vault, HashMap::from([("key".to_string(), vault)])),
        ]);

        assert_eq!(merged.get("key"), Some(&ConfigValue::String("vault_value".to_string())));
    }

    #[test]
    fn test_config_validation_rules() {
        let config = Config::new(/* ... */);
        let result = config.validate();

        // Test validation rules are applied correctly
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

    pub fn sample_global_config() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("app.name".to_string(), ConfigValue::String("lithos".to_string())),
            ("app.version".to_string(), ConfigValue::String("0.1.0".to_string())),
        ])
    }

    pub fn sample_user_config() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("ui.theme".to_string(), ConfigValue::String("dark".to_string())),
        ])
    }

    pub fn sample_encrypted_field() -> ConfigValue {
        // Note: In tests, we use mock encryption
        ConfigValue::Encrypted(b"mock_encrypted_data".to_vec())
    }
}
```

**Performance Testing:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_config_merge(c: &mut Criterion) {
    let configs = vec![
        fixtures::sample_global_config(),
        fixtures::sample_user_config(),
        // Add project and vault configs
    ];

    c.bench_function("config_hierarchical_merge", |b| {
        b.iter(|| {
            black_box(Config::merge_hierarchical(black_box(&configs)));
        });
    });
    // Target: <100μs for typical hierarchical merges
}
```

### File Structure Requirements

**File Structure (Single File per Context - Split at 1000+ Lines):**
```
crates/domain/src/
├── lib.rs                    # Public API surface, re-exports
├── models/
│   ├── mod.rs               # Module declarations
│   └── config.rs            # All Config entities, validation, and logic
├── ports/
│   ├── mod.rs               # Port trait declarations
│   └── config.rs            # ConfigCommand/ConfigQuery traits (shells)
└── errors.rs                # Domain errors (EXTENDED with config errors)
```

**Splitting Guideline:** Start with single file. Split when >1000 lines into logical modules (e.g., config_types.rs, config_validation.rs, config_hierarchy.rs).

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

**Hierarchical Configuration Requirements:**
- **Global Level**: System-wide defaults (read-only for users)
- **User Level**: User-specific overrides (~/.config/lithos/)
- **Project Level**: Project-specific settings (.lithos/ in project root)
- **Vault Level**: Vault-specific configuration (.obsidian/ or .lithos/ in vault)

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

**TDD Hierarchical Merging:**
```rust
impl Config {
    /// Merge configurations with proper precedence (Vault > Project > User > Global)
    pub fn merge_hierarchical(
        configs: Vec<(ConfigPath, HashMap<String, ConfigValue>)>
    ) -> HashMap<String, ConfigValue> {
        let mut merged = HashMap::new();

        // Process in precedence order (lowest to highest)
        let precedence_order = [
            ConfigPath::Global,
            ConfigPath::User,
            ConfigPath::Project,
            ConfigPath::Vault,
        ];

        for level in precedence_order {
            if let Some(level_config) = configs.iter().find(|(path, _)| path == &level) {
                // Override or add values at this level
                for (key, value) in &level_config.1 {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }

        merged
    }
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
