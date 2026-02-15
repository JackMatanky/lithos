# Story 6.2: implement-hierarchical-configuration-loading-with-figment

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user configuring lithos,
I want hierarchical configuration that respects precedence rules,
So that I can override settings at different levels (global, user, project, vault).

## Acceptance Criteria

**Given** Epic 4 provides unified structured file loading (TOML, JSON, YAML) via parsers
**When** I implement hierarchical config using Figment per ADR 009
**Then** I create `crates/adapters/src/spi/config/loader.rs` implementing provider pattern

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
**When** I integrate parsers module
**Then** FileProvider uses `crates/adapters/src/spi/fs/parsers.rs` for TOML/JSON/YAML detection
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

## Tasks / Subtasks

- [ ] Task 1: Create Figment loader with provider architecture (AC: 1, 2, 3)
  - [ ] Subtask 1.1: Implement loader.rs with provider pattern base structure
  - [ ] Subtask 1.2: Create DefaultsProvider for compiled-in configuration
  - [ ] Subtask 1.3: Create FileProvider using Epic 4 FormatDispatcher
  - [ ] Subtask 1.4: Create EnvProvider for LITHOS_ environment variable mapping
  - [ ] Subtask 1.5: Create CliArgsProvider for test simulation
- [ ] Task 2: Configure Figment precedence and merging logic (AC: 4, 5)
  - [ ] Subtask 2.1: Implement precedence order configuration
  - [ ] Subtask 2.2: Implement deep merging for nested structures
  - [ ] Subtask 2.3: Test vault config overrides global config behavior
- [ ] Task 3: Integrate Epic 4 parsers for file loading (AC: 6)
  - [ ] Subtask 3.1: Use parsers::parse_format() in FileProvider
  - [ ] Subtask 3.2: Implement automatic format detection by file extension
  - [ ] Subtask 3.3: Handle file not found errors gracefully using validator.rs
- [ ] Task 4: Preserve and extract Figment metadata for error reporting (AC: 7, 8)
  - [ ] Subtask 4.1: Extract source location metadata from Figment
  - [ ] Subtask 4.2: Store metadata for validation error context
  - [ ] Subtask 4.3: Implement environment variable mapping for nested structures
- [ ] Task 5: Create comprehensive test suite with BDD and property testing (AC: 9)
  - [ ] Subtask 5.1: Write unit tests in loader.rs with #[cfg(test)]
  - [ ] Subtask 5.2: Create integration tests for precedence behavior
  - [ ] Subtask 5.3: Add property tests using proptest for edge cases
  - [ ] Subtask 5.4: Test CLI args provider readiness for clap integration

## Dev Notes

### 🚨 CRITICAL PATH CORRECTIONS

**Epic 4 Integration - File Structure Fixed:**
```rust
// CORRECT paths (validated against current codebase)
use crate::spi::fs::parsers;           // NOT format_dispatcher.rs
use crate::spi::fs::validator;         // NOT path_validator.rs
use crate::spi::fs::parsers::Format;    // Format enum for detection
```

**Domain Port Integration - Query Trait Only:**
- Loader implements **Query trait methods only** (read operations)
- Must NOT include write operations (belongs to Story 6.6 Command adapter)
- Follow strict CQRS pattern from recent git commits

**Story 6.1 Dependency - Test Fixtures Available:**
- Use `docs/defaults/global.toml` and `vault.toml` as test fixtures
- Reference `docs/schemas/config.schema.json` (validation in Story 6.3)
- Ensure loader produces Config that matches Story 6.1 schema

### Domain Model Integration

**Config Aggregate** (Epic 3): Domain models already exist in `crates/domain/src/config/`
- `aggregate.rs`: Config with GlobalConfig + VaultConfig merge logic
- `global.rs`: GlobalConfig with filesystem, frontmatter, logging, trusted_vaults
- `vault.rs`: VaultConfig with filesystem, cache_dir, optional frontmatter/logging

**Target Structure**: Final Config must match these domain models exactly
- Use serde deserialize directly into Config struct
- Validation happens in separate step (Story 6.3) - this story only loads

### Architecture Compliance

**Hexagonal Architecture**: Follow established patterns
- Create adapter in `crates/adapters/src/spi/config/`
- Domain models remain untouched in `crates/domain/src/config/`
- Port integration follows existing `crates/domain/src/ports/config.rs` pattern

**Provider Pattern**: Figment-specific architecture
- Each provider implements Figment's Provider trait
- Loader orchestrates providers in precedence order
- Clean separation between data sources

**Epic 4 Integration**: Critical dependency requirements
- `crates/adapters/src/spi/fs/parsers.rs::parse_format()` for multi-format support
- `crates/adapters/src/spi/fs/validator.rs::validate()` for security (FileProvider)
- Must handle Epic 4 parser and validator error types gracefully

### Epic 6.1 Previous Story Intelligence

**Schema and Defaults Available**: Story 6.1 completed configuration schema
- `docs/schemas/config.schema.json` authoritative schema (validation in Story 6.3)
- `docs/defaults/global.toml` and `vault.toml` with comprehensive defaults
- Domain model mapping verified - use these as test data

**File Format Support**: Epic 6.1 created TOML/JSON/YAML versions
- FormatDispatcher must handle all three formats
- Default files provide test fixtures for provider validation

### Figment Implementation Strategy

**Provider Design Pattern**: Use precedence order: Defaults → Global → Vault → Env → CLI Args
```rust
Figment::from(DefaultsProvider::default())
    .merge(FileProvider::global("global.toml"))
    .merge(FileProvider::vault("vault.toml"))
    .merge(EnvProvider::new())
    .merge(CliArgsProvider::new(args))
```

**Error Handling Integration**:
- Map Figment errors to existing `ConfigError` variants
- Preserve Figment metadata for rich miette diagnostics
- Handle missing files gracefully (optional configs)

**Performance Considerations**:
- Providers should be lazy where possible
- File reading only when files exist
- Environment variable access is cached in provider

### Git Intelligence from Recent Commits

**Strict CQRS Enforcement**: Recent commits show strict CQRS pattern enforcement
- Ensure loader is read-only (query side)
- Write operations belong in separate Command adapter (Story 6.6)
- Maintain clean separation between read/write concerns

**Test Suite Structure**: Recent refactor shows test organization patterns
- Place tests directly in source file with #[cfg(test)]
- Use BDD-style test names and comments
- Include property tests for edge case coverage

### File Structure Requirements

**Primary Implementation File**:
```
crates/adapters/src/spi/config/loader.rs
├── Provider implementations (Defaults, File, Env, CliArgs)
├── Loader struct with Figment orchestration
├── Error handling using Epic 4 parsers/validator
├── CQRS Query trait implementation (read-only operations)
└── Comprehensive test suite with #[cfg(test)]
```

**Module Integration**: Update `crates/adapters/src/spi/config/mod.rs`
- Re-export `Loader` as `ConfigLoader`
- Maintain existing module structure
- Follow established visibility patterns

### Environment Variable Mapping

**LITHOS_ Prefix Convention**:
```rust
// Environment → Config mapping examples
LITHOS_VAULT_PATH → vault.path
LITHOS_LOG_LEVEL → global.log_level
LITHOS_SCHEMAS_DIR → global.schemas_dir
LITHOS_TEMPLATES_DIR → global.templates_dir
LITHOS_CACHE_DIR → vault.cache_dir
```

**Nested Structure Mapping**:
- Use double underscore `__` for nested structures
- Example: `LITHOS_VAULT__FRONTMATTER__TITLE_KEY` → `vault.frontmatter.title_key`
- Follow snake_case to snake_case conversion

### Testing Strategy

**Unit Tests** (in loader.rs with #[cfg(test)]):
- Test each provider independently
- Test precedence order with mock data
- Test error conditions and edge cases

**Integration Tests**:
- Test complete configuration loading flow
- Test file format detection with real files
- Test environment variable override behavior

**Property Tests**:
- Use proptest for environment variable generation
- Test deep merging behavior with random configs
- Test precedence invariants never violated

**Performance Tests**:
- Benchmark configuration loading time (<100ms target)
- Test memory usage with large configurations
- Verify lazy loading behavior

### Error Handling Requirements

**Error Mapping**: Convert Figment errors to ConfigError variants
- `FigmentError::Missing` → `ConfigError::MissingField`
- `FigmentError::Type` → `ConfigError::InvalidType`
- `FigmentError::Parse` → `ConfigError::ParseError`
- `FigmentError::Path` → `ConfigError::InvalidPath`

**Metadata Preservation**: Store source location for debugging
- File path, line, column information
- Original value and attempted conversion
- Chain errors with proper context

### Performance Constraints

**Loading Time**: Complete configuration loading in <50ms (aligned with Epic 6 NFR4)
- File I/O minimized through lazy loading
- Environment variable access optimized
- Figment merging overhead minimized

**Memory Usage**: Configuration structures should be lightweight
- Avoid unnecessary clones during merging
- Use string references where possible
- Clean up temporary data structures

### Security Considerations

**File Access Security**: Use Epic 4 PathValidator
- Validate all file paths before reading
- Prevent path traversal attacks
- Handle symbolic links safely

**Environment Variable Security**:
- Only process LITHOS_ prefixed variables
- Sanitize environment variable values
- Handle sensitive data appropriately

### Project Structure Notes

**Target Files**:
- `crates/adapters/src/spi/config/loader.rs` - Main implementation
- `crates/adapters/src/spi/config/mod.rs` - Module exports

**Dependencies**:
- `figment` - Hierarchical configuration
- `serde` - Deserialization into Config domain models
- Epic 4 parsers and validator (corrected paths)
- Existing domain models and error types

**No New Crate Dependencies**: Use existing dependencies only
- Figment should already be available for config system
- Avoid adding heavy dependencies for simple loading

### Figment Configuration Patterns

**Provider Precedence**: Order matters for override behavior
```rust
Figment::from(DefaultsProvider::default())
    .merge(FileProvider::global(...))        // Global config overrides defaults
    .merge(FileProvider::vault(...))          // Vault config overrides global
    .merge(EnvProvider::new())               // Environment overrides files
    .merge(CliArgsProvider::new(args))      // CLI args override everything
```

**Deep Merging Strategy**:
- Figment automatically handles nested structure merging
- Arrays are replaced wholesale (not merged element-wise)
- Objects merge field-by-field recursively

### Integration Dependencies

**Critical Epic Dependencies**:
- **Epic 3**: Config domain models (must already exist)
- **Epic 4**: parsers and validator modules (must be completed)
- **Epic 6.1**: Schema and defaults (completed - provides test fixtures)

**Future Story Dependencies**:
- Story 6.3 (validation) depends on this loader
- Story 6.6 (Command adapter) will use this loader for reading
- Epic 5 cache integration will cache loaded configurations

### Validation and Error Reporting

**Error Context**: Preserve Figment metadata for miette integration
- Source file location for field errors
- Environment variable names for override issues
- Line/column information for parse errors

**Structural Validation**: This story only loads, does not validate
- Validation belongs in Story 6.3
- Preserve all data for later validation step
- Handle missing optional fields gracefully

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-6-configuration-management-system-phase-15.md#Story-6.2] - Complete story requirements and acceptance criteria
- [Source: _bmad-output/planning-artifacts/prd.md#Configuration-Management] - FR26, FR27, FR28 functional requirements
- [Source: _bmad-output/project-context.md#Technology-Stack] - Rust 1.92+, Figment configuration, error handling standards
- [Source: _bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md#ADR-0005] - Figment hierarchical configuration decision
- [Source: crates/domain/src/config/aggregate.rs] - Config aggregate target structure
- [Source: crates/domain/src/config/global.rs] - GlobalConfig domain model
- [Source: crates/domain/src/config/vault.rs] - VaultConfig domain model
- [Source: crates/domain/src/ports/config.rs] - Port definitions for integration
- [Source: _bmad-output/implementation-artifacts/stories/6-1-create-default-configuration-files-schema.md] - Previous story with schema and defaults
- [Source: crates/adapters/src/spi/fs/parsers.rs] - Epic 4 parsers for multi-format file loading
- [Source: crates/adapters/src/spi/fs/validator.rs] - Epic 4 validator for security validation

## Dev Agent Record

### Agent Model Used

big-pickle (opencode/big-pickle)

### Debug Log References

- Story creation: 2026-01-28T00:00:00Z
- Epic analysis: 2026-01-28T00:00:00Z
- Previous story review: 2026-01-28T00:00:00Z
- Architecture review: 2026-01-28T00:00:00Z
- Git history analysis: 2026-01-28T00:00:00Z

### Completion Notes List

- Story 6.2 requirements fully extracted with specific Figment provider pattern
- Previous story 6.1 intelligence applied for schema and defaults integration
- File naming corrected: loader.rs (not figment_loader.rs) based on user feedback
- Test structure corrected: tests must be in same file with #[cfg(test)]
- Git intelligence applied: strict CQRS pattern enforcement from recent commits
- Epic 4 integration requirements identified as critical dependency
- Environment variable mapping strategy designed for snake_case convention
- Error handling strategy mapped to existing ConfigError domain types
- Performance constraints identified (<100ms loading, lightweight memory usage)

### File List

### Target Files

```
crates/adapters/src/spi/config/
├── loader.rs                           # Main implementation with providers and tests
└── mod.rs                             # Module exports (update to re-export ConfigLoader)
```

**Test Data References from Story 6.1**:
- `docs/defaults/global.toml` - Global configuration defaults
- `docs/defaults/vault.toml` - Vault configuration defaults
- `docs/defaults/global.json` - JSON format test fixture
- `docs/defaults/vault.json` - JSON format test fixture
- `docs/defaults/global.yaml` - YAML format test fixture
- `docs/defaults/vault.yaml` - YAML format test fixture
- `docs/schemas/config.schema.json` - Schema reference (validation in Story 6.3)

**Domain Model References**:
- `crates/domain/src/config/aggregate.rs` - Target Config structure
- `crates/domain/src/config/global.rs` - GlobalConfig fields and validation
- `crates/domain/src/config/vault.rs` - VaultConfig fields and validation
- `crates/domain/src/ports/config.rs` - Port interface definitions

**Epic 4 Integration References**:
- `crates/adapters/src/spi/file/format_dispatcher.rs` - Multi-format file parsing
- `crates/adapters/src/spi/file/path_validator.rs` - Security validation
