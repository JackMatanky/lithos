# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - TBD

### Added - Epic 1: Foundational CLI with Static Template Engine

**Features:**

- CLI with `version` and `new` commands
- Static template rendering using Go text/template
- Basic template functions: now, toLower, toUpper
- File path control functions: path, folder, basename, extension, join, vaultPath
- Configuration loading from lithos.json, environment variables, and defaults
- Structured logging with zerolog (JSON and pretty-print modes)
- Template loading from filesystem
- Note creation workflow: template → rendered content → file

**Architecture:**

- Clean hexagonal architecture following v0.6.8 specifications
- Domain models: NoteID, Frontmatter, Note, TemplateID, Template, Config
- Domain services: TemplateEngine, CommandOrchestrator
- API Ports: CLIPort, CommandPort
- SPI Ports: ConfigPort, TemplatePort
- Adapters: CobraCLIAdapter, ViperAdapter, TemplateLoaderAdapter
- Shared packages: Logger, Error handling (idiomatic Go), Registry (generic CQRS)

**Testing:**

- Unit tests for all domain models and services
- Integration tests for template loading and rendering
- End-to-end test for complete CLI workflow
- Test coverage: >70% for domain/app layers

**Documentation:**

- Complete architecture documentation in docs/architecture/
- README with installation, quick start, and reference
- Template function reference with examples
- Configuration reference with all options
- Epic 1 PRD: docs/prd/epic-1-foundational-cli-static_template-engine.md

**Technical Stack:**

- Go 1.23+ (generics, improved errors)
- cobra v1.9.1 (CLI framework)
- viper v1.21.0 (configuration)
- zerolog v1.34.0 (structured logging)
- text/template (stdlib, template engine)

## [0.2.0] - TBD

### Added - Epic 2: Configuration & Schema Loading

**Schema System:**

- Schema-based note structure definition and validation
- Property bank for reusable property definitions (single source of truth)
- Schema inheritance with `extends` (multi-level support) and `excludes` (property removal)
- Property reference resolution with `$ref` for DRY schemas
- Structural and cross-schema validation (missing parents, invalid $refs, cycles)
- Cycle detection in inheritance chains with actionable error messages
- Thread-safe in-memory schema registry with concurrent read support
- Schema loading with stage duration logging for NFR3 observability
- PropertySpec variants: String (regex), Number (min/max/step), Boolean, Date, File (directory/fileClass)
- Constraint validation: regex, enum, min/max, step, directory, fileClass

**Configuration:**

- Configuration system updates: SchemasDir, PropertyBankFile fields
- Environment variable overrides: LITHOS_SCHEMAS_DIR, LITHOS_PROPERTY_BANK_FILE
- PropertyBankPath() helper method for path construction

**Error Handling:**

- Actionable error messages with remediation hints
- File path context in schema loading errors
- Schema and property identifiers in validation errors
- Error message examples in documentation

**Architecture:**

- Domain models: Schema, Property, PropertySpec (5 variants), PropertyBank, Config updates
- Domain services: SchemaValidator (structural + cross-schema), SchemaResolver (inheritance + $ref), SchemaEngine (orchestration)
- Ports: SchemaPort (loading), SchemaRegistryPort (registration/lookup)
- Adapters: SchemaLoaderAdapter (filesystem), SchemaRegistryAdapter (in-memory)
- Updated CommandOrchestrator with SchemaEngine dependency
- Updated main.go initialization order with schema system

**Testing:**

- Unit tests for all schema domain models and services
- Integration tests for schema loading, validation, and resolution
- End-to-end tests for complete schema workflow
- Golden files for error output validation
- Thread safety tests for concurrent registry access
- Test coverage: >70% for domain/app layers

**Documentation:**

- Comprehensive schema system documentation in docs/schemas/
- Schema quick start guide with complete examples
- Updated README with schema system introduction
- Updated configuration reference with schema fields
- Error handling examples with remediation hints
- Architecture documentation updates: components.md, data-models.md
- Epic 2 PRD: docs/prd/epic-2-configuration-schema-loading.md

**Technical Stack:**

- Continued: Go 1.23+, cobra v1.9.1, viper v1.21.0, zerolog v1.34.0
- New capabilities: Schema validation, inheritance resolution, property bank

[0.2.0]: https://github.com/JackMatanky/lithos/releases/tag/v0.2.0

[0.1.0]: https://github.com/JackMatanky/lithos/releases/tag/v0.1.0
