# Epic 3: Vault Indexing Engine

This epic focuses on building the core data layer of Lithos. It will scan the user's vault, parse frontmatter, and build a persistent cache that will power all future dynamic and lookup-based features. This epic implements the CQRS-based cache architecture with VaultIndexer and QueryService following hexagonal design principles.

**Dependencies:** Epic 2 (Configuration & Schema Loading)

## Story 3.1: Implement CQRS Cache Port Interfaces

As a developer, I want to define separate command and query interfaces for the cache, so that I follow CQRS principles and have clear separation of concerns.

### Acceptance Criteria

- 3.1.1: `internal/ports/spi/` contains CacheCommandPort interface with Store and Remove methods per `docs/architecture/components.md#spi-port-interfaces`.
- 3.1.2: CacheQueryPort interface with Fetch and List methods is defined for read-side access per `docs/architecture/components.md#spi-port-interfaces`.
- 3.1.3: Interfaces use the Note domain model (File + Frontmatter composition) from `docs/architecture/data-models.md#note`.
- 3.1.4: Port contracts include context support and structured error handling.

## Story 3.2: Implement JSON File Cache Adapter

As a developer, I want to implement the JSONFileCacheAdapter that provides both command and query cache access, so that I have a persistent cache following the architecture.

### Acceptance Criteria

- 3.2.1: `internal/adapters/spi/cache/` contains JSONFileCacheAdapter implementing both CacheCommandPort and CacheQueryPort.
- 3.2.2: Store method serializes Note struct to JSON file in `.lithos/cache/` directory using atomic write pattern.
- 3.2.3: Fetch method deserializes JSON file back into Note struct with proper error handling.
- 3.2.4: Adapter uses LocalFileSystemAdapter (composition) and follows tech stack specifications for JSON serialization.

## Story 3.3: Enhance FileSystem Port for Vault Scanning

As a developer, I want to enhance the FileSystemPort to support vault scanning operations, so that vault indexing can discover markdown files through the proper architectural interface.

### Acceptance Criteria

- 3.3.1: FileSystemPort interface includes Walk method with WalkFunc signature per `docs/architecture/components.md#filesystemport`.
- 3.3.2: LocalFileSystemAdapter implements Walk using Go stdlib `filepath.Walk` functionality.
- 3.3.3: Vault scanning logic filters for `.md` files and ignores `.lithos` cache directory.
- 3.3.4: Scanner returns slice of absolute paths and handles filesystem errors gracefully.

## Story 3.4: Implement Custom Frontmatter Extraction

As a developer, I want to implement custom frontmatter extraction using the specified technology stack, so that I have control over parsing behavior and minimize dependencies.

### Acceptance Criteria

- 3.4.1: `internal/app/indexing/` contains custom frontmatter extraction logic using delimiter detection per `docs/architecture/tech-stack.md#yaml-parsing`.
- 3.4.2: Implementation uses `github.com/goccy/go-yaml` for YAML parsing after delimiter extraction.
- 3.4.3: Parser returns frontmatter as `map[string]interface{}` and creates Frontmatter domain model per `docs/architecture/data-models.md#frontmatter`.
- 3.4.4: Graceful handling of files with no frontmatter, invalid YAML, or unsupported delimiter variants (only `---` supported).

## Story 3.5: Implement VaultIndexer Domain Service

As a developer, I want to implement the VaultIndexer domain service, so that vault indexing follows the architectural patterns.

### Acceptance Criteria

- 3.5.1: `internal/app/indexing/` contains VaultIndexer implementing the interface from `docs/architecture/components.md#vaultindexer`.
- 3.5.2: Service orchestrates vault scanning, frontmatter parsing, and cache persistence through port interfaces.

## Story 3.6: Implement Indexing Orchestration

As a developer, I want to implement the indexing orchestration logic, so that the rebuild method coordinates all components.

### Acceptance Criteria

- 3.6.1: Rebuild method coordinates FileSystemPort, CacheCommandPort, and SchemaValidator interactions.
- 3.6.2: Service logs indexing progress using structured logging and returns IndexStats result object.

## Story 3.7: Implement Query Service

As a developer, I want to implement the QueryService domain service, so that read-side access to indexed data follows the architectural patterns.

### Acceptance Criteria

- 3.7.1: QueryService includes Filter struct with Key, Operator, and Value fields for flexible searching.
- 3.7.2: Query method accepts filter parameters and returns filtered Note slices through CacheQueryPort.
- 3.7.3: Initial implementation supports basic iteration with preparation for advanced filtering.
- 3.7.4: Service maintains thread-safe access patterns using `sync.RWMutex` as specified in components documentation.

## Story 3.8: Add Index Command to CLI

As a user, I want to run `lithos index` through the CLI adapter, so that I can trigger vault indexing through the proper architectural layers.

### Acceptance Criteria

- 3.8.1: CobraCLIAdapter includes `index` subcommand that calls CLICommandPort.Index method.
- 3.8.2: CommandOrchestrator implements Index method that delegates to VaultIndexer.
- 3.8.3: Command returns structured IndexStats and logs progress through proper service layers.
- 3.8.4: CLI handles errors gracefully and provides user-friendly feedback.

## Story 3.9: Implement Cache Versioning and Migration

As a developer, I want the cache to include versioning and migration capabilities, so that schema changes don't break existing installations.

### Acceptance Criteria

- 3.9.1: Cache metadata includes semantic version field for cache schema per original requirements.
- 3.9.2: VaultIndexer detects version mismatches and handles migration or clean rebuild.
- 3.9.3: Migration logic is documented and includes rollback guidance.
- 3.9.4: Cache version changes trigger appropriate user notifications and recovery procedures.
