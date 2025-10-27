# Epic 3: Vault Indexing Engine

This epic delivers the vault indexing pipeline described in architecture v0.6.8. By implementing the cache and vault ports, filesystem adapters, frontmatter validation, indexing orchestration, and CLI control flow, Lithos will produce a consistent on-disk cache and populate in-memory query indices for downstream features. Stories progress sequentially from SPI definitions to service orchestration and CLI exposure.

---

## Story 3.1 Cache Port Interfaces

As a developer,
I want CacheWriterPort and CacheReaderPort defined per the architecture,
so that cache operations follow the CQRS pattern with clear contracts.

**Prerequisites:** Epic 2 stories (schema runtime) complete.

### Acceptance Criteria
1. `internal/ports/spi/cache.go` defines `CacheWriterPort` and `CacheReaderPort` signatures exactly as documented in `docs/architecture/components.md#cachewriterport` and `#cachereaderport`.
2. Interfaces include GoDoc referencing error wrapping requirements from `docs/architecture/error-handling-strategy.md` and FR9 in `docs/prd/requirements.md`.
3. Port definitions explicitly state thread-safety expectations and cache directory usage described in the architecture change log.
4. Linting (`golangci-lint run ./internal/ports/spi`) succeeds.

---

## Story 3.2 JSON Cache Adapters

As a developer,
I want filesystem adapters that satisfy the cache ports,
so that the indexer can persist notes and the query layer can read them.

**Prerequisites:** Story 3.1.

### Acceptance Criteria
1. `internal/adapters/spi/cache/json_writer.go` implements `CacheWriterPort` using `encoding/json` and `moby/sys/atomicwriter` exactly as described in `docs/architecture/components.md#jsoncachewriteadapter`.
2. `internal/adapters/spi/cache/json_reader.go` implements `CacheReaderPort`, wrapping IO errors with `CacheReadError` and preserving unknown JSON fields to satisfy FR6.
3. Adapters honour cache directory configuration from `domain.Config` and include structured logging required by `coding-standards.md`.
4. Unit tests cover persist/delete/read/list success paths, permission-denied failures, malformed JSON, and verify error wrapping semantics.
5. `golangci-lint run ./internal/adapters/spi/cache` and `go test ./internal/adapters/spi/cache` pass.

---

## Story 3.3 VaultReaderPort & Adapter

As a developer,
I want a vault reader port and adapter that expose the business-level scan operations,
so that indexing can read notes through an architecture-approved boundary.

**Prerequisites:** Stories 3.1–3.2.

### Acceptance Criteria
1. `internal/ports/spi/vault.go` declares `VaultReaderPort` exactly as defined in `docs/architecture/components.md#vaultreaderport`, with GoDoc outlining business-level semantics and references to FR9.
2. `internal/adapters/spi/vault/reader.go` scans the vault according to architecture guidance (ignoring cache directories, populating `VaultFile` metadata) and wraps filesystem failures with `FileSystemError` per `error-handling-strategy.md`.
3. Adapter honours Config vault path, supports incremental scanning via `ScanModified`, and logs progress per `coding-standards.md`.
4. Unit tests cover full scan, incremental scan, missing files, permission errors, and ensure returned `VaultFile` structures match `data-models.md#vaultfile`.
5. `golangci-lint run ./internal/adapters/spi/vault` and `go test ./internal/adapters/spi/vault` succeed.

---

## Story 3.4 VaultWriterPort & Adapter

As a developer,
I want a vault writer port and adapter,
so that CommandOrchestrator and the indexer can persist notes with atomic guarantees.

**Prerequisites:** Stories 3.1–3.3.

### Acceptance Criteria
1. `internal/ports/spi/vault.go` declares `VaultWriterPort` exactly as specified in `docs/architecture/components.md#vaultwriterport`, documenting idempotency expectations and FR6 preservation.
2. `internal/adapters/spi/vault/writer.go` persists notes using `moby/sys/atomicwriter`, ensures target directories exist, and wraps failures with `FileSystemError` including operation/path metadata.
3. Adapter logs write/delete operations per coding standards and supports overwrite semantics without mutating note content.
4. Unit tests cover new writes, overwrites, deletes, missing file deletes, permission failures, and confirm error wrapping behaviour.
5. `golangci-lint run ./internal/adapters/spi/vault` and `go test ./internal/adapters/spi/vault` succeed.

---

## Story 3.5 FrontmatterService

As a developer,
I want FrontmatterService to extract and validate frontmatter exactly as described,
so that the indexer produces canonical Note objects.

**Prerequisites:** Stories 2.4–2.8 (schema runtime) and Stories 3.1–3.4.

### Acceptance Criteria
1. `internal/app/frontmatter/service.go` implements `Extract` and `Validate` exactly as described in `docs/architecture/components.md#frontmatterservice`, using `goccy/go-yaml` and SchemaRegistryPort/QueryService dependencies.
2. Validation enforces FR6/FR7 requirements, including strict type checks, array vs scalar handling, and FileSpec lookups via QueryService.
3. Service returns structured `FrontmatterError` instances per `error-handling-strategy.md`, preserving unknown fields as required.
4. Unit tests cover extraction edge cases, missing required fields, type mismatches, file reference resolution, and error message content.
5. `golangci-lint run ./internal/app/frontmatter` and `go test ./internal/app/frontmatter` succeed.

---

## Story 3.6 VaultIndexer Service

As a developer,
I want VaultIndexer to orchestrate the indexing workflow,
so that the cache and in-memory indices stay consistent with the vault.

**Prerequisites:** Stories 3.1–3.5.

### Acceptance Criteria
1. `internal/app/vault/indexer.go` implements `Indexer.Build` following the steps in `docs/architecture/components.md#vaultindexer` (vault scan → frontmatter extract/validate → note creation → cache persist → query index update) and respects FR9.
2. `IndexStats` records counts for scanned notes, indexed notes, validation failures, cache failures, and total duration; logging uses zerolog per coding standards and feeds NFR3 metrics.
3. Indexer updates QueryService indices via the package-private hooks defined in the architecture and handles cache write failures by logging warnings without aborting the build.
4. Unit tests with fakes verify call order, error handling for validation and cache operations, and stats accuracy.
5. `golangci-lint run ./internal/app/vault` and `go test ./internal/app/vault` succeed.

---

## Story 3.7 QueryService

As a developer,
I want QueryService to expose the lookup methods described in the architecture,
so that templates and validators can retrieve indexed notes efficiently.

**Prerequisites:** Stories 3.1–3.6.

### Acceptance Criteria
1. `internal/app/query/service.go` implements `ByID`, `ByPath`, `ByFileClass`, `ByFrontmatter`, and `RefreshFromCache` exactly as described in `docs/architecture/components.md#queryservice`, using in-memory indices with `sync.RWMutex`.
2. Query methods satisfy FR9 by supporting lookups by path, basename, and schema-defined keys; helpers return errors consistent with `error-handling-strategy.md`.
3. Service exposes instrumentation hooks or logging recommended in the architecture appendix for query debugging.
4. Unit tests cover index population, each query method, cache refresh, concurrent reads, and error paths when entries are missing.
5. `golangci-lint run ./internal/app/query` and `go test ./internal/app/query` succeed.

---

## Story 3.8 CLI Index Command

As a developer,
I want the CLI to trigger vault indexing via CommandOrchestrator,
so that users can rebuild the cache and indices on demand.

**Prerequisites:** Stories 3.1–3.7.

### Acceptance Criteria
1. `internal/app/command/orchestrator.go` implements `IndexVault(ctx context.Context) (IndexStats, error)` delegating to `VaultIndexer.Build`, logging summary statistics per `components.md#commandorchestrator`, and wrapping errors per the error strategy.
2. The CLI adapter registers an `index` command mirroring the architecture workflow: parse flags, call CommandPort, print stats, return non-zero exit code on failure.
3. Integration or end-to-end tests execute `lithos index` against fixtures, verify cache files, CLI output, and satisfaction of FR9.
4. Documentation or help output references the new command consistent with `docs/prd/requirements.md#functional` entries.
