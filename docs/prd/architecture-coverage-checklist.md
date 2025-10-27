# Architecture Coverage Checklist

**Purpose:** Systematic verification that ALL components from architecture v0.6.8 are mapped to epic and story assignments. Prevents forgotten functionality during implementation.

**Created:** 2025-10-27
**Architecture Version:** v0.6.8 (2025-10-26)
**Status:** 🔴 INCOMPLETE - Awaiting epic alignment

---

## How to Use This Checklist

1. **During Epic Alignment (Phase 2.1-2.6):** For each component below, verify it has epic assignment and add story reference
2. **During Story Generation (Phase 4):** Verify each story number exists and implements the component
3. **During Implementation (Phase 5):** Mark components ✅ as implemented

**Coverage Status Key:**
- ❌ = No epic/story assignment
- 🟡 = Has epic but no story yet
- ✅ = Has epic + story + implementation

---

## 1. Domain Models (11 models)

### 1.1 Domain Core Models

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **NoteID** | v0.5.2 | ? | ? | ❌ | `internal/domain/note_id.go` |
| **Frontmatter** | v0.1.0 | ? | ? | ❌ | `internal/domain/frontmatter.go` |
| **Note** | v0.5.2 | ? | ? | ❌ | `internal/domain/note.go` |
| **TemplateID** | v0.5.6 | ? | ? | ❌ | `internal/domain/template_id.go` |
| **Template** | v0.5.6 | ? | ? | ❌ | `internal/domain/template.go` |
| **Schema** | v0.1.0 (rich model v0.6.0) | ? | ? | ❌ | `internal/domain/schema.go` |
| **Property** | v0.1.0 (rich model v0.6.0) | ? | ? | ❌ | `internal/domain/property.go` |
| **PropertySpec** (interface + 5 variants) | v0.1.0 (rich model v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **PropertyBank** | v0.5.5 | ? | ? | ❌ | `internal/domain/property_bank.go` |
| **Config** | v0.5.7 | ? | ? | ❌ | `internal/domain/config.go` |

### 1.2 PropertySpec Variants

| Variant | Version Added | Epic | Story | Status | File Path |
|---------|---------------|------|-------|--------|-----------|
| **StringSpec** | v0.1.0 (rich v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **NumberSpec** | v0.1.0 (rich v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **DateSpec** | v0.1.0 (rich v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **FileSpec** | v0.1.0 (rich v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **BoolSpec** | v0.1.0 (rich v0.6.0) | ? | ? | ❌ | `internal/domain/property_spec.go` |

### 1.3 SPI Adapter Models

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **FileMetadata** | v0.5.1 | ? | ? | ❌ | `internal/adapters/spi/file_metadata.go` |
| **VaultFile** | v0.6.8 | ? | ? | ❌ | `internal/adapters/spi/vault_file.go` |

---

## 2. Domain Services (7 services)

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **TemplateEngine** | v0.1.0 (updated v0.6.2) | ? | ? | ❌ | `internal/domain/template_engine.go` |
| **FrontmatterService** | v0.1.0 (updated v0.6.2) | ? | ? | ❌ | `internal/domain/frontmatter_service.go` |
| **SchemaEngine** | v0.5.11 (generics) | ? | ? | ❌ | `internal/domain/schema_engine.go` |
| **SchemaValidator** | v0.6.1 | ? | ? | ❌ | `internal/domain/schema_validator.go` |
| **SchemaResolver** | v0.6.1 | ? | ? | ❌ | `internal/domain/schema_resolver.go` |
| **VaultIndexer** | v0.5.11 | ? | ? | ❌ | `internal/domain/vault_indexer.go` |
| **QueryService** | v0.5.11 | ? | ? | ❌ | `internal/domain/query_service.go` |
| **CommandOrchestrator** | v0.6.4 | ? | ? | ❌ | `internal/domain/command_orchestrator.go` |

---

## 3. SPI Ports (9 ports)

### 3.1 CQRS Cache Ports

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **CacheWriterPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/cache_writer.go` |
| **CacheReaderPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/cache_reader.go` |

### 3.2 CQRS Vault Ports

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **VaultReaderPort** | v0.6.8 | ? | ? | ❌ | `internal/ports/spi/vault_reader.go` |
| **VaultWriterPort** | v0.6.8 | ? | ? | ❌ | `internal/ports/spi/vault_writer.go` |

### 3.3 Schema & Template Ports

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **SchemaPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/schema.go` |
| **SchemaRegistryPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/schema_registry.go` |
| **TemplatePort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/template.go` |

### 3.4 Interactive & Config Ports

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **PromptPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/prompt.go` |
| **FinderPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/finder.go` |
| **ConfigPort** | v0.5.11 | ? | ? | ❌ | `internal/ports/spi/config.go` |

---

## 4. SPI Adapters (11 adapters)

### 4.1 CQRS Cache Adapters

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **JSONCacheWriteAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/cache/json_cache_writer.go` |
| **JSONCacheReadAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/cache/json_cache_reader.go` |

### 4.2 CQRS Vault Adapters

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **VaultReaderAdapter** | v0.6.8 | ? | ? | ❌ | `internal/adapters/spi/vault/vault_reader.go` |
| **VaultWriterAdapter** | v0.6.8 | ? | ? | ❌ | `internal/adapters/spi/vault/vault_writer.go` |

### 4.3 Schema & Template Adapters

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **SchemaLoaderAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/schema/schema_loader.go` |
| **SchemaRegistryAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/schema/schema_registry.go` |
| **TemplateLoaderAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/template/template_loader.go` |

### 4.4 Interactive & Config Adapters

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **PromptUIAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/interactive/promptui.go` |
| **FuzzyfindAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/interactive/fuzzyfind.go` |
| **ViperAdapter** | v0.5.11 | ? | ? | ❌ | `internal/adapters/spi/config/viper.go` |

---

## 5. API Ports (1 port)

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **CLICommandPort** | v0.6.4 | ? | ? | ❌ | `internal/ports/api/cli_command.go` |

**Related Interface:**
| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **CommandHandler** (callback interface) | v0.6.4 | ? | ? | ❌ | `internal/ports/api/command_handler.go` |

---

## 6. API Adapters (1 adapter)

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **CobraCLIAdapter** | v0.5.11 (updated v0.6.4) | ? | ? | ❌ | `internal/adapters/api/cobra_cli.go` |

---

## 7. Shared Internal Packages (3 packages)

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **Logger** (shared/logger) | v0.1.0 | ? | ? | ❌ | `internal/shared/logger/logger.go` |
| **Error Package** (shared/errors) | v0.1.0 (updated v0.5.9) | ? | ? | ❌ | `internal/shared/errors/` |
| **Registry Package** (shared/registry) | v0.1.0 | ? | ? | ❌ | `internal/shared/registry/` |

### 7.1 Error Package Components

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **BaseError** | v0.1.0 | ? | ? | ❌ | `internal/shared/errors/types.go` |
| **ValidationError** | v0.1.0 | ? | ? | ❌ | `internal/shared/errors/types.go` |
| **ResourceError** | v0.1.0 | ? | ? | ❌ | `internal/shared/errors/types.go` |
| **FrontmatterError** | v0.5.9 | ? | ? | ❌ | `internal/shared/errors/frontmatter.go` |
| **SchemaError** | v0.1.0 | ? | ? | ❌ | `internal/shared/errors/schema.go` |
| **TemplateError** | v0.1.0 | ? | ? | ❌ | `internal/shared/errors/template.go` |
| **CacheReadError** | v0.5.9 | ? | ? | ❌ | `internal/shared/errors/cache.go` |
| **CacheWriteError** | v0.5.9 | ? | ? | ❌ | `internal/shared/errors/cache.go` |
| **FileSystemError** | v0.5.9 | ? | ? | ❌ | `internal/shared/errors/filesystem.go` |

---

## 8. Template Engine Functions (11 functions)

### 8.1 User Interaction Functions

| Function | Version Added | Epic | Story | Status | Implementation |
|----------|---------------|------|-------|--------|----------------|
| **prompt(name, label, default)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |
| **suggester(name, label, options)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |
| **now(format)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |

### 8.2 Vault Query Functions

| Function | Version Added | Epic | Story | Status | Implementation |
|----------|---------------|------|-------|--------|----------------|
| **lookup(basename)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |
| **query(filter)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |
| **fileClass(noteID)** | v0.1.0 | ? | ? | ❌ | TemplateEngine function map |

### 8.3 File Path Control Functions

| Function | Version Added | Epic | Story | Status | Implementation |
|----------|---------------|------|-------|--------|----------------|
| **path()** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |
| **folder(path)** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |
| **basename(path)** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |
| **extension(path)** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |
| **join(parts...)** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |
| **vaultPath()** | v0.6.3 | ? | ? | ❌ | TemplateEngine function map |

---

## 9. Dependency Injection & Initialization

| Component | Version Added | Epic | Story | Status | File Path |
|-----------|---------------|------|-------|--------|-----------|
| **main.go** (DI wiring) | v0.6.5 | ? | ? | ❌ | `cmd/lithos/main.go` |
| **Initialization Order** (documented pattern) | v0.6.5 | ? | ? | ❌ | N/A (architecture doc) |

---

## 10. Validation Architecture

### 10.1 Schema Validation (Structural)

| Component | Version Added | Epic | Story | Status | Implementation |
|-----------|---------------|------|-------|--------|----------------|
| **Schema.Validate()** (rich model method) | v0.6.0 | ? | ? | ❌ | `internal/domain/schema.go` |
| **Property.Validate()** (rich model method) | v0.6.0 | ? | ? | ❌ | `internal/domain/property.go` |
| **PropertySpec.Validate()** (polymorphic) | v0.6.0 | ? | ? | ❌ | `internal/domain/property_spec.go` |
| **SchemaValidator.ValidateAll()** (orchestrator) | v0.6.1 | ? | ? | ❌ | `internal/domain/schema_validator.go` |

### 10.2 Frontmatter Validation (Business Rules)

| Component | Version Added | Epic | Story | Status | Implementation |
|-----------|---------------|------|-------|--------|----------------|
| **FrontmatterService.Extract()** | v0.1.0 | ? | ? | ❌ | `internal/domain/frontmatter_service.go` |
| **FrontmatterService.Validate()** | v0.1.0 (updated v0.6.2) | ? | ? | ❌ | `internal/domain/frontmatter_service.go` |

---

## Coverage Summary

**Total Components:** ~70+ (including variants, functions, error types)

| Category | Total | Assigned | Unassigned | Coverage % |
|----------|-------|----------|------------|------------|
| **Domain Models** | 13 | 0 | 13 | 0% |
| **Domain Services** | 8 | 0 | 8 | 0% |
| **SPI Ports** | 11 | 0 | 11 | 0% |
| **SPI Adapters** | 11 | 0 | 11 | 0% |
| **API Ports** | 2 | 0 | 2 | 0% |
| **API Adapters** | 1 | 0 | 1 | 0% |
| **Shared Packages** | 3 | 0 | 3 | 0% |
| **Error Types** | 9 | 0 | 9 | 0% |
| **Template Functions** | 12 | 0 | 12 | 0% |
| **Validation Components** | 6 | 0 | 6 | 0% |
| **TOTAL** | **76** | **0** | **76** | **0%** |

---

## Next Steps

### Phase 2.1-2.6: Epic Alignment

For each epic file in `docs/prd/`, systematically:

1. **Read epic file** to understand current story assignments
2. **Check checklist** for components that should be in this epic
3. **Add missing stories** for uncovered components
4. **Update checklist** with epic and story numbers for each component
5. **Verify 100% coverage** before proceeding to next epic

### Verification Criteria

✅ **Epic Coverage Complete** when:
- Every component has epic assignment
- Every component has story number (or "N/A" with justification)
- Coverage summary shows 100% assigned
- No ❌ or 🟡 status remaining

✅ **Story Generation Ready** when:
- All epics aligned with architecture v0.6.8
- All components have story assignments
- Story count estimates verified (~54-60 stories total)

---

## Epic Assignment Guide

**Suggested epic mapping** (to be validated during Phase 2):

- **Epic 1 (Foundational CLI):**
  - Domain models: Note, NoteID, Frontmatter, Template, TemplateID, Config
  - Shared packages: Logger, Error Package (base types), Registry
  - File path template functions: path(), folder(), basename(), extension(), join(), vaultPath()
  - Main.go DI wiring (basic)

- **Epic 2 (Config & Schema Loading):**
  - Domain models: Schema, Property, PropertySpec (all variants), PropertyBank
  - Domain services: SchemaEngine, SchemaValidator, SchemaResolver
  - SPI ports: SchemaPort, SchemaRegistryPort, ConfigPort
  - SPI adapters: SchemaLoaderAdapter, SchemaRegistryAdapter, ViperAdapter
  - SPI models: none (pure domain)
  - Error types: SchemaError variants
  - Validation: Schema.Validate(), Property.Validate(), PropertySpec.Validate()

- **Epic 3 (Vault Indexing):**
  - Domain services: VaultIndexer, QueryService, FrontmatterService
  - SPI ports: VaultReaderPort, VaultWriterPort, CacheWriterPort, CacheReaderPort
  - SPI adapters: VaultReaderAdapter, VaultWriterAdapter, JSONCacheWriteAdapter, JSONCacheReadAdapter
  - SPI models: FileMetadata, VaultFile
  - Error types: FrontmatterError, CacheReadError, CacheWriteError, FileSystemError
  - Validation: FrontmatterService.Extract(), FrontmatterService.Validate()

- **Epic 4 (Interactive Input):**
  - SPI ports: PromptPort, FinderPort
  - SPI adapters: PromptUIAdapter, FuzzyfindAdapter
  - Template functions: prompt(), suggester(), now()

- **Epic 5 (Schema-Driven Lookups):**
  - Domain services: CommandOrchestrator (complete)
  - API ports: CLICommandPort, CommandHandler
  - API adapters: CobraCLIAdapter
  - Template functions: lookup(), query(), fileClass()
  - Main.go DI wiring (complete)

---

## Related Documents

- **Architecture:** `docs/architecture/` (v0.6.8)
- **Epics:** `docs/prd/epic-*.md`
- **Sprint Proposal:** `docs/course_correction/sprint-change-proposal-2025-10-27-complete-architecture-alignment.md`
- **Lessons Learned:** `docs/course_correction/lessons-learned-phase-1-archive.md`
