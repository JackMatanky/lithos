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
- Epic 1 PRD: docs/prd/epic-1-foundational-cli-static-template-engine.md

**Technical Stack:**

- Go 1.23+ (generics, improved errors)
- cobra v1.9.1 (CLI framework)
- viper v1.21.0 (configuration)
- zerolog v1.34.0 (structured logging)
- text/template (stdlib, template engine)

[0.1.0]: https://github.com/JackMatanky/lithos/releases/tag/v0.1.0
