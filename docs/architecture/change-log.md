# Change Log

| Date       | Version | Description                                          | Author  |
| ---------- | ------- | ---------------------------------------------------- | ------- |
| 2025-10-08 | 0.1.0   | Initial architecture document creation               | Winston |
| 2025-10-08 | 0.2.0   | Added High Level Architecture section                | Winston |
| 2025-10-08 | 0.3.0   | Added Tech Stack section with verified versions      | Winston |
| 2025-10-08 | 0.4.0   | Added Data Models section                            | Winston |
| 2025-10-08 | 0.4.1   | Refined Data Models with architecture layers         | Winston |
| 2025-10-11 | 0.4.2   | Removed non-existent component references            | Winston |
| 2025-10-11 | 0.4.3   | Unified data models and added Additional Information | Winston |
| 2025-10-24 | 0.5.0   | Updated High Level Architecture with clean hexagonal design principles | Winston |
| 2025-10-24 | 0.5.1   | Renamed File to FileMetadata and moved to SPI Adapter layer, added CQRS adapter references | Winston |
| 2025-10-24 | 0.5.2   | Added NoteID domain model, updated Note to use NoteID + Frontmatter composition, clarified FrontmatterService | Winston |
| 2025-10-24 | 0.5.3   | Simplified Schema and Property models to lean domain structures, removed ResolvedProperties from domain | Winston |
| 2025-10-24 | 0.5.4   | Restructured PropertySpec section with DDD value object classification, separated spec variants into subsections | Winston |
| 2025-10-24 | 0.5.5   | Updated PropertyBank with singleton pattern, single JSON file, and $ref resolution details | Winston |
| 2025-10-24 | 0.5.6   | Added TemplateID domain model, updated Template to use TemplateID (basename as domain concept), removed TemplateMetadata (reuse FileMetadata) | Winston |
| 2025-10-24 | 0.5.7   | Reclassified Config as Domain Value Object, changed to JSON format for MVP, added PropertyBankFile attribute | Winston |
| 2025-10-24 | 0.5.8   | Updated data model relationships diagram with hierarchical composition structure and clean architecture principles | Winston |
| 2025-10-24 | 0.5.9   | Updated error handling strategy: removed Result[T] references, renamed ValidationError to FrontmatterError, split StorageError into CacheReadError/CacheWriteError/FileSystemError | Winston |
| 2025-10-24 | 0.5.10  | Added moby/sys/atomicwriter to tech stack for atomic file writes | Winston |
| 2025-10-24 | 0.5.11  | Updated components.md: removed CommandOrchestrator, updated domain services (TemplateEngine, FrontmatterService, SchemaEngine with generics, VaultIndexer, QueryService), updated API ports (CLICommandPort), updated SPI ports (CacheWriter/Reader, SchemaPort, TemplatePort, PromptPort/FinderPort split, ConfigPort, SchemaRegistryPort), removed filesystem ports per YAGNI, updated adapters (split JSONCache adapters, SchemaLoaderAdapter, TemplateLoaderAdapter, PromptUIAdapter, FuzzyfindAdapter, ViperAdapter, SchemaRegistryAdapter, CobraCLIAdapter) | Winston |
| 2025-10-26 | 0.6.0   | Updated data-models.md: Made Schema, Property, and PropertySpec rich domain models with Validate() methods for structural integrity checks | Winston |
| 2025-10-26 | 0.6.1   | Updated components.md: Added SchemaValidator service (orchestrates schema validation), SchemaResolver service (inheritance and $ref resolution), clarified SchemaEngine internally instantiates both | Winston |
| 2025-10-26 | 0.6.2   | Updated components.md: Expanded FrontmatterService documentation with detailed validation workflow, type coercion rules, and strict validation philosophy (no semantic coercion) | Winston |
| 2025-10-26 | 0.6.3   | Updated components.md: Added file path template functions to TemplateEngine (path, folder, basename, extension, join, vaultPath) for controlling note save locations | Winston |
| 2025-10-26 | 0.6.4   | Updated components.md: Reinstated CommandOrchestrator as proper use case orchestrator, redesigned CLICommandPort with CommandHandler callback interface for hexagonal architecture, updated CobraCLIAdapter with SRP decomposition pattern | Winston |
| 2025-10-26 | 0.6.5   | Updated components.md: Added Dependency Injection Pattern section documenting initialization order and internal vs injected dependencies | Winston |
| 2025-10-26 | 0.6.6   | Updated components.md: Added Validation Architecture Overview section separating schema validation (structural, startup) from frontmatter validation (business rules, runtime) | Winston |
| 2025-10-26 | 0.6.7   | Updated coding-standards.md: Removed Result[T] pattern references, replaced with idiomatic Go (T, error) signatures throughout, updated error handling guidelines with fmt.Errorf wrapping and domain error types | Winston |
| 2025-10-26 | 0.6.8   | Added VaultReaderPort and VaultWriterPort with CQRS pattern for vault operations, added VaultReaderAdapter and VaultWriterAdapter, renamed CacheWriterPort.Write to Persist for consistency, updated VaultIndexer and CommandOrchestrator dependencies, updated component diagrams | Winston |
