# Epic 3: Vault Indexing Engine

**Epic Goal:** Implement the vault indexing pipeline defined in architecture v0.6.8 so Lithos can scan the vault, validate frontmatter, persist a cache, and expose indexed data for downstream queries.

**Dependencies:** Epic 2 (Configuration & Schema Loading)

**Architecture References:**
- `docs/architecture/components.md` v0.6.8 — VaultIndexer, FrontmatterService, QueryService, CacheWriterPort, CacheReaderPort, VaultReaderPort, VaultWriterPort, CommandOrchestrator
- `docs/architecture/data-models.md` v0.6.8 — Note, NoteID, Frontmatter, VaultFile, FileMetadata
- `docs/architecture/error-handling-strategy.md` v0.5.9 — CacheReadError, CacheWriteError, FileSystemError, FrontmatterError
- `docs/architecture/coding-standards.md` v0.6.7 — `(T, error)` signatures, logging, testing

---

## Story 3.1: Define CacheWriterPort & CacheReaderPort

As a developer, I need cache ports that match the architecture so the indexing workflow can persist and read cached notes via CQRS contracts.

### Acceptance Criteria
1. In `internal/ports/spi/cache.go`, declare `CacheWriterPort` with `Persist(ctx context.Context, note Note) error` and `Delete(ctx context.Context, id NoteID) error` exactly as in `components.md#cachewriterport`.
2. In the same file, declare `CacheReaderPort` with `Read(ctx context.Context, id NoteID) (Note, error)` and `List(ctx context.Context) ([]Note, error)` per `components.md#cachereaderport`.
3. Add GoDoc referencing the architecture sections and error strategy expectations (CacheWriteError/CacheReadError wrapping).
4. Run `golangci-lint run ./internal/ports/spi`.

---

## Story 3.2: Implement JSON Cache Adapters

As a developer, I want filesystem adapters that implement the cache ports using the JSON strategy defined in the architecture.

### Acceptance Criteria
1. Implement `internal/adapters/spi/cache/json_writer.go` with constructor `NewJSONCacheWriter(root string, log zerolog.Logger)` that satisfies `CacheWriterPort`.
2. `Persist` writes `${root}/{noteID}.json` using `encoding/json` and `atomicwriter.WriteFile`; `Delete` removes the cached file and returns `CacheWriteError` on failure (except missing files).
3. Implement `internal/adapters/spi/cache/json_reader.go` with constructor `NewJSONCacheReader(root string, log zerolog.Logger)` that satisfies `CacheReaderPort` (`Read` + `List`).
4. Wrap IO errors with `CacheReadError`/`CacheWriteError`, never return raw `*os.PathError`.
5. Unit tests cover persist/delete/read/list happy paths and failure conditions (permission denied, malformed JSON).
6. Run `golangci-lint run ./internal/adapters/spi/cache` and `go test ./internal/adapters/spi/cache`.

---

## Story 3.3: Define VaultReaderPort & Filesystem Adapter

As a developer, I need a vault reader port and adapter that provide the scanning operations described in the architecture.

### Acceptance Criteria
1. In `internal/ports/spi/vault.go`, declare `VaultReaderPort` with methods `ScanAll(ctx context.Context) ([]VaultFile, error)`, `ScanModified(ctx context.Context, since time.Time) ([]VaultFile, error)`, and `Read(ctx context.Context, path string) (VaultFile, error)` per `components.md#vaultreaderport`.
2. Implement `internal/adapters/spi/vault/reader.go` using the OS filesystem to satisfy the port (ignore `.lithos/` and other non-note folders as noted in the architecture).
3. Populate `VaultFile` with metadata (Path, Basename, Folder, Ext, ModTime, Size, MimeType) plus file content.
4. Wrap filesystem errors with `FileSystemError` enriched with operation/path fields.
5. Unit tests cover full scan, incremental scan, single read, and error scenarios (permission denied, missing file).
6. Run `golangci-lint run ./internal/adapters/spi/vault` and `go test ./internal/adapters/spi/vault`.

---

## Story 3.4: Define VaultWriterPort & Filesystem Adapter

As a developer, I want a vault writer port and adapter so CommandOrchestrator and VaultIndexer can persist notes according to CQRS write patterns.

### Acceptance Criteria
1. Extend `internal/ports/spi/vault.go` with `VaultWriterPort` exposing `Persist(ctx context.Context, note Note, path string) error` and `Delete(ctx context.Context, path string) error` as in `components.md#vaultwriterport`.
2. Implement `internal/adapters/spi/vault/writer.go` using `atomicwriter.WriteFile` for atomic writes and `os.Remove` for deletes.
3. Ensure directories are created with `os.MkdirAll` and errors are wrapped with `FileSystemError`.
4. Unit tests cover writing new files, overwriting existing files, delete success, and failure cases.
5. Run `golangci-lint run ./internal/adapters/spi/vault` and `go test ./internal/adapters/spi/vault`.

---

## Story 3.5: Implement FrontmatterService

As a developer, I need FrontmatterService to extract and validate frontmatter exactly as documented so indexing produces canonical Note objects.

### Acceptance Criteria
1. Create `internal/app/frontmatter/service.go` with methods `Extract(content []byte) (Frontmatter, error)` and `Validate(ctx context.Context, fm Frontmatter) error` per `components.md#frontmatterservice`.
2. `Extract` must parse YAML frontmatter using `goccy/go-yaml`, returning `Frontmatter` domain model; missing or malformed frontmatter results in `FrontmatterError`.
3. `Validate` must look up schemas through SchemaRegistryPort, enforce required/array/type checks, and use QueryService for FileSpec existence (as described in the architecture workflow).
4. Unit tests cover extraction edge cases (no frontmatter, malformed YAML) and validation scenarios (missing required field, array vs scalar mismatch, file reference validation).
5. Run `golangci-lint run ./internal/app/frontmatter` and `go test ./internal/app/frontmatter`.

---

## Story 3.6: Implement VaultIndexer Service

As a developer, I want VaultIndexer to orchestrate the indexing workflow exactly as defined in the architecture.

### Acceptance Criteria
1. Implement `internal/app/vault/indexer.go` with `type Indexer struct` and method `Build(ctx context.Context) (IndexStats, error)` following the sequence: `VaultReaderPort.ScanAll` → `FrontmatterService.Extract/Validate` → construct Note → `CacheWriterPort.Persist` → update QueryService indices.
2. `IndexStats` should capture totals for scanned files, indexed notes, validation failures, and duration (per architecture guidance).
3. Log progress at info/debug levels using zerolog.
4. Unit tests use fakes for ports/services to verify call order, error propagation, and stats aggregation.
5. Run `golangci-lint run ./internal/app/vault` and `go test ./internal/app/vault`.

---

## Story 3.7: Implement QueryService

As a developer, I need QueryService to expose the lookup methods outlined in the architecture for downstream queries.

### Acceptance Criteria
1. Create `internal/app/query/service.go` with methods `ByID`, `ByPath`, `ByFileClass`, and `ByFrontmatter` exactly as described in `components.md#queryservice`.
2. Maintain in-memory indices (maps protected by `sync.RWMutex`) populated by VaultIndexer after cache persistence.
3. Implement `RefreshFromCache(ctx context.Context, reader CacheReaderPort) error` to rebuild indices on demand using `CacheReaderPort.List`.
4. Unit tests cover index population, each query method, and concurrent reads (use `t.Parallel()` and the race detector in CI).
5. Run `golangci-lint run ./internal/app/query` and `go test ./internal/app/query`.

---

## Story 3.8: Wire Index Command through CommandOrchestrator

As a developer, I want the CLI index command to call VaultIndexer.Build and surface results.

### Acceptance Criteria
1. Extend `internal/app/command/orchestrator.go` to implement `IndexVault(ctx context.Context) (IndexStats, error)` that delegates to VaultIndexer.Build and returns stats.
2. Update the CLI adapter (`internal/adapters/api/cli`) to register an `index` command that calls `CommandPort.IndexVault`, prints summary statistics, and handles errors per coding standards.
3. Add integration test(s) demonstrating a CLI run against sample vault data (using fixtures) and verifying cache files/console output.
4. Run `golangci-lint run ./internal/app/command ./internal/adapters/api/cli` and relevant `go test` packages.
