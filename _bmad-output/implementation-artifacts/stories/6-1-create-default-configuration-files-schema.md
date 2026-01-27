# Story 6.1: create-default-configuration-files-schema

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user getting started with lithos,
I want default configuration files with proper schema validation,
so that I can understand configuration options and customize settings confidently.

## Acceptance Criteria

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

## Tasks / Subtasks

- [ ] Task 1: Create authoritative JSON schema for configuration (AC: 1, 2, 3)
  - [ ] Subtask 1.1: Analyze existing domain models and extract all configuration fields
  - [ ] Subtask 1.2: Generate JSON schema with snake_case enforcement and proper types
  - [ ] Subtask 1.3: Validate schema against existing domain structures
- [ ] Task 2: Create default configuration files with validation (AC: 4, 5, 6)
  - [ ] Subtask 2.1: Create global.toml with comprehensive defaults and inline documentation
  - [ ] Subtask 2.2: Create vault.toml with vault-specific defaults and inline documentation
  - [ ] Subtask 2.3: Generate JSON and YAML equivalents for cross-format compatibility
  - [ ] Subtask 2.4: Implement schema validation to prove correctness of default files
- [ ] Task 3: Verify domain model alignment (AC: 7)
  - [ ] Subtask 3.1: Manual verification that config structure maps to Config aggregate from Epic 3
  - [ ] Subtask 3.2: Document any mapping discrepancies or required adjustments
- [ ] Task 4: Create comprehensive documentation (AC: 8)
  - [ ] Subtask 4.1: Add inline TOML comments explaining every field's purpose and valid values
  - [ ] Subtask 4.2: Create configuration guide documentation in docs/configuration.md

## Dev Notes

### Domain Model Analysis

**Config Aggregate Structure** (from Epic 3 domain models):
- **GlobalConfig**: filesystem (SchemaConfig, TemplateConfig), frontmatter (FrontmatterConfig), logging (LoggingConfig), trusted_vaults
- **VaultConfig**: filesystem (SchemaConfig, TemplateConfig, cache_dir), frontmatter (Option<FrontmatterConfig>), logging (Option<LoggingConfig>)
- **Config**: Merged result with vault_metadata, logging, global_filesystem, vault_filesystem, frontmatter, pending_events

**Key Configuration Fields**:
- `schemas_dir`: Directory for schema definitions (default: "schemas")
- `templates_dir`: Directory for template definitions (default: "templates")
- `property_bank_filename`: JSON file for property definitions (default: "property_bank.json")
- `cache_dir`: Cache directory for vault operations (default: ".cache")
- `log_level`: Logging level (default: "info")
- Frontmatter keys: `alias_key`, `date_created_key`, `date_modified_key`, `file_class_key`, `title_key`
- Vault metadata: `schema_version`, `name`, `vault_path`

### Architecture Compliance

**Hexagonal Architecture**: Follow established patterns - domain models exist in `crates/domain/src/config/`, adapters will be created in `crates/adapters/src/spi/config/`
**CQRS Pattern**: Separate Command/Query ports already defined in `crates/domain/src/ports/config.rs`
**Figment Integration**: Per ADR 0005 for hierarchical configuration (not yet created, but referenced in epic)

### Naming Conventions

**snake_case Enforcement**: All JSON schema properties must use snake_case to align with Rust serde defaults and TOML standards
**File Naming**: `global.toml`, `vault.toml` in `docs/defaults/`, `config.schema.json` in `docs/schemas/`

### Error Handling Strategy

**Domain Validation**: Use existing `ConfigError` variants from domain layer
**Schema Validation**: JSON schema validation with clear error messages for structural violations
**Miette Integration**: Use `miette` for rich error diagnostics as per project standards

### Testing Requirements

**Unit Tests**: Test schema validation, default file generation, and domain model mapping
**Integration Tests**: Test Figment loading and merging behavior (when implemented in future stories)
**Property Tests**: Use `proptest` for configuration edge cases and validation boundaries
**Coverage**: Target 80%+ coverage as per project standards
**Test Location**: `crates/domain/src/config/tests/` for unit tests, `crates/app/tests/` for integration tests

### Performance Considerations

**Schema Validation**: Should complete in <100ms for typical configuration files
**File Generation**: Default file creation should be I/O bound, not CPU bound
**Memory Usage**: Configuration objects should be lightweight and memory-efficient

### Project Structure Notes

**Target Files**:
- `docs/schemas/config.schema.json` - Authoritative JSON schema
- `docs/defaults/global.toml` - Global configuration defaults with documentation
- `docs/defaults/vault.toml` - Vault-specific defaults with documentation
- `docs/defaults/global.json` - JSON equivalent for cross-platform compatibility
- `docs/defaults/vault.json` - JSON equivalent
- `docs/defaults/global.yaml` - YAML equivalent for human readability
- `docs/defaults/vault.yaml` - YAML equivalent

**Directory Structure**: Follows established `docs/` conventions for user-facing documentation
**No Code Changes**: This story creates documentation and schema files only - no Rust code modifications required

### Validation Strategy

**Schema Self-Validation**: Use external JSON schema validator or serde_json to validate schema correctness
**Domain Mapping Verification**: Manual verification that all domain fields are represented in schema
**Default File Validation**: Implement validation script to prove defaults conform to schema
**Cross-Format Consistency**: Ensure TOML, JSON, YAML versions produce identical configuration structures

### Dependencies and Integration Points

**Domain Models**: Depends on Epic 3 Config aggregate being complete and stable
**Future Stories**: This story enables subsequent Epic 6 stories that depend on having schema and defaults
**No External Dependencies**: Pure documentation and schema generation work
**Tooling**: May use existing Rust JSON schema generation libraries if beneficial

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-6-configuration-management-system-phase-15.md#Story-6.1] - Complete epic requirements and acceptance criteria
- [Source: _bmad-output/planning-artifacts/prd.md#Configuration-Management] - FR26, FR27, FR28 functional requirements
- [Source: _bmad-output/project-context.md#Technology-Stack] - Rust 1.92+, TOML support, error handling standards
- [Source: crates/domain/src/config/aggregate.rs] - Config aggregate structure and business rules
- [Source: crates/domain/src/config/global.rs] - GlobalConfig domain model
- [Source: crates/domain/src/config/vault.rs] - VaultConfig domain model
- [Source: crates/domain/src/ports/config.rs] - Command/Query port definitions for future adapter implementation
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Progressive-Complexity] - User experience requirements for configuration onboarding

## Dev Agent Record

### Agent Model Used

big-pickle (opencode/big-pickle)

### Debug Log References

- Story creation: 2026-01-28T00:00:00Z
- Epic analysis: 2026-01-28T00:00:00Z
- Domain model review: 2026-01-28T00:00:00Z
- Sprint status update: 2026-01-28T00:00:00Z

### Completion Notes List

- Epic 6.1 requirements fully extracted and translated into actionable tasks
- Domain model alignment verified against existing Config aggregate structure
- File structure planned to follow established docs/ conventions
- Schema validation strategy designed for snake_case enforcement
- Cross-format compatibility planned (TOML primary, JSON/YAML secondary)
- No code dependencies identified - pure documentation/story deliverable

### File List

**Files to Create:**
1. `docs/schemas/config.schema.json` - Authoritative JSON schema with snake_case enforcement
2. `docs/defaults/global.toml` - Global defaults with comprehensive inline documentation
3. `docs/defaults/vault.toml` - Vault defaults with vault-specific overrides
4. `docs/defaults/global.json` - JSON format equivalent
5. `docs/defaults/vault.json` - JSON format equivalent
6. `docs/defaults/global.yaml` - YAML format equivalent
7. `docs/defaults/vault.yaml` - YAML format equivalent
8. `docs/configuration.md` - User guide for configuration options (optional, for Story 6.10)

**Files to Reference:**
- `crates/domain/src/config/aggregate.rs` - For domain model mapping verification
- `crates/domain/src/config/global.rs` - For GlobalConfig field extraction
- `crates/domain/src/config/vault.rs` - For VaultConfig field extraction
- `crates/domain/src/ports/config.rs` - For Command/Query port understanding
