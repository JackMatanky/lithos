# Epic Impact Assessment - CORRECTED

## Epic Impact Assessment

This section analyzes how each of the 8 issue groups impacts Epic 3 (Vault Indexing Engine) by mapping architectural gaps to concrete story insertions, dependencies, and sequencing. All recommendations incorporate findings from Research Strategy (Go stdlib + Obsidian patterns), Gap Analysis (12 gaps), Actionable Insights (AI-1.1 through AI-3.1), and Entity Review (8 entities).

### Epic 3 Current State (Baseline for Impact Analysis)

**Completed Stories** (✅ DONE):
- 3.1-3.9: Original PRD stories (Cache ports, JSON adapters, Vault ports, Indexer, Query, Frontmatter, CLI)
- 3.10: Fix Note ID Collision and Path Handling

**In Progress** (🔄 Ready for Done):
- 3.11-3.16: Bug fixes and refinements (memory leak, cache management, query layer, performance, integration, QA)

**Awaiting Enhancement** (📋 Course Correction):
- Current 3.17: Hybrid Architecture DI and E2E Test (Ready for Enhancement - awaiting hybrid BoltDB+SQLite)
- Current 3.18: Documentation Update (Ready for Enhancement - awaiting architecture finalization)

**Strategic Context**: November 2, 2024 Sprint Change Proposal APPROVED pivot to hybrid BoltDB + SQLite architecture for production performance. This architectural review identifies 14 NEW stories (35+ points) that must be inserted BEFORE current 3.17-3.18 can proceed.

---

### Story Insertion Strategy

Since Stories 3.1-3.16 are mostly DONE, all new work inserts **AFTER 3.16** and **BEFORE current 3.17** in dependency-ordered groups:

```
3.1-3.16 (existing - mostly done)
  ↓
GROUP A: Foundation DTOs & Ports (3.17-3.18) - Sprint 1
GROUP B: Storage Implementation (3.19-3.22) - Sprint 2-3
GROUP C: Service Refactoring (3.23-3.26) - Sprint 3-4
GROUP D: Configuration & Docs (3.27-3.29) - Sprint 4
  ↓
3.30: Hybrid DI/E2E (was 3.17 - unblocked)
3.31: Documentation (was 3.18 - enhanced)
```

---

## GROUP A: Foundation DTOs & Ports (Sprint 1)

### Story 3.17: VaultFile DTO Redesign with Layered Architecture

**Incorporates**: AI-1.1 (Eliminate FileMetadata Duplication), AI-1.2 (Vault-Relative Paths), AI-2.2 (File vs Content Separation), Entity Review VaultFile Assessment

**Priority**: CRITICAL - Foundation for all storage work
**Effort**: 3 points
**Insert Location**: After 3.16, before current 3.17

**Description**: Redesign VaultFile DTO using layered architecture: base DTO with fs.FileInfo, content separation, and storage-specific DTOs.

**Acceptance Criteria**:

**Layer 1 - Base DTO** (AI-1.1 + AI-1.2):
1. ✅ Refactor `VaultFile` struct:
   ```go
   type VaultFile struct {
       Path    string      // Vault-relative: "notes/meeting.md"
       Info    fs.FileInfo // Delegate to stdlib (ModTime, Size, Mode, IsDir)
       Content []byte      // Loaded on-demand
   }
   ```
2. ✅ Add computed methods: `Basename()`, `Folder()`, `Ext()`, `ModTime()`, `Size()`
3. ✅ Remove duplicated fields: Basename, Folder, Ext, ModTime, Size, MimeType
4. ✅ Path normalization helper: `NormalizePath(absPath, vaultRoot string) (string, error)` using filepath.ToSlash
5. ✅ Helper: `AbsolutePath(vaultRoot string) string` for I/O operations

**Layer 2 - Content Separation** (AI-2.2):
6. ✅ Create `VaultFileMeta` struct (metadata only - NO Content):
   ```go
   type VaultFileMeta struct {
       Path string
       Info fs.FileInfo
   }
   ```
7. ✅ Create `VaultFileWithContent` struct (metadata + content):
   ```go
   type VaultFileWithContent struct {
       VaultFileMeta
       Content []byte
   }
   ```
8. ✅ Update VaultScannerPort methods:
   - `ScanAll(ctx) ([]VaultFileMeta, error)` - fast metadata scan
   - `ScanWithContent(ctx) ([]VaultFileWithContent, error)` - when content needed

**Layer 3 - Storage-Specific DTOs** (Entity Review Phase 3):
9. ✅ Create `BoltDBMetadata` (hot cache - minimal):
   ```go
   type BoltDBMetadata struct {
       Path      string
       Basename  string
       Aliases   []string
       FileClass string
       ModTime   time.Time
   }
   ```
10. ✅ Create `SQLiteMetadata` (deep storage - complete):
    ```go
    type SQLiteMetadata struct {
        Path        string
        Frontmatter map[string]any
        ModTime     time.Time
        Size        int64
    }
    ```
11. ✅ Conversion functions: `ToBeoltDBMetadata()`, `ToSQLiteMetadata()`

**Cross-Platform Paths** (Gap 7.1):
12. ✅ All path storage uses forward slashes (filepath.ToSlash)
13. ✅ I/O operations convert to OS-specific paths (filepath.FromSlash)

**Testing**:
14. ✅ Unit tests: fs.FileInfo delegation, computed methods, path normalization
15. ✅ Cross-platform tests: Windows/Linux/Mac path compatibility
16. ✅ Memory tests: VaultFileMeta vs VaultFileWithContent memory usage

**Dependencies**:
- Depends on: Stories 3.1-3.16 (foundation complete)
- Blocks: All GROUP B stories (storage implementations)
- Resolves: Gap 1.1, Gap 1.2, Gap 1.3, Gap 7.1, Issue D2

**Risks**:
- HIGH: VaultFile used across many adapters - comprehensive impact analysis required
- Mitigation: Update all usages in same story; comprehensive integration tests

**Architecture Doc Updates**:
- `docs/architecture/data-models.md`: Update VaultFile, add layered DTO explanation
- `docs/architecture/components.md`: Update VaultScannerPort interface

---

### Story 3.18: MarkdownParserPort - Dedicated Parsing Port & Adapter

**Incorporates**: AI-1.3 (Extract MarkdownParserPort), Gap 3.1 (Goldmark in Domain)

**Priority**: CRITICAL - Fixes hexagonal architecture violation
**Effort**: 3 points
**Insert Location**: After 3.17

**Description**: Create dedicated MarkdownParserPort and GoldmarkParserAdapter, moving goldmark parsing from domain layer (FrontmatterService) to adapter layer. This is a separate port/adapter, NOT just moving to VaultReaderAdapter.

**Acceptance Criteria**:

**New Port Definition**:
1. ✅ Create `/internal/ports/spi/markdown.go`:
   ```go
   type MarkdownParserPort interface {
       // ParseFrontmatter extracts YAML frontmatter from markdown content
       // Returns parsed fields as map, error if invalid YAML
       ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error)
   }
   ```

**New Adapter Implementation**:
2. ✅ Create `/internal/adapters/spi/markdown/goldmark_parser.go`:
   ```go
   type GoldmarkParserAdapter struct {
       markdown goldmark.Markdown
       log      zerolog.Logger
   }

   func (a *GoldmarkParserAdapter) ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error) {
       // Use goldmark + frontmatter extension
       // Syntactic validation (YAML structure)
       // Return parsed map or error
   }
   ```

**Domain Layer Cleanup**:
3. ✅ Remove goldmark imports from `internal/app/frontmatter/service.go`
4. ✅ Remove `FrontmatterService.Extract()` method entirely
5. ✅ FrontmatterService constructor accepts `MarkdownParserPort` (injected dependency)
6. ✅ FrontmatterService focuses on **semantic validation only** (schema compliance)

**Adapter Layer - Syntactic Validation**:
7. ✅ GoldmarkParserAdapter performs **syntactic validation**:
   - YAML structure validation
   - Parsing errors (malformed YAML)
   - Returns structured errors with line numbers

**Integration**:
8. ✅ Update VaultIndexer to inject MarkdownParserPort
9. ✅ VaultIndexer workflow: Read file → Parse frontmatter → Pass to FrontmatterService
10. ✅ VaultReaderAdapter.Read() still returns raw Content (parsing happens in indexer)

**Testing**:
11. ✅ Unit tests: FrontmatterService without goldmark (pure domain tests)
12. ✅ Unit tests: GoldmarkParserAdapter syntactic validation
13. ✅ Integration tests: End-to-end parsing workflow
14. ✅ Error handling tests: Malformed YAML, missing frontmatter

**Dependencies**:
- Depends on: Story 3.17 (VaultFile DTO ready)
- Blocks: Story 3.23 (FrontmatterService refactoring)
- Resolves: Gap 3.1, Issue B2, Hexagonal Principle violation

**Risks**:
- MEDIUM: FrontmatterService interface changes ripple to consumers
- Mitigation: Update all consumers in same story

**Future Expansion** (Post-Epic 3):
- Phase 2: Expand to parse Links, Headings, Tags (domain.NoteMetadata)
- Phase 3: Enable backlinks, graph queries

**Architecture Doc Updates**:
- `docs/architecture/components.md`: Add MarkdownParserPort, GoldmarkParserAdapter
- `docs/architecture/coding-standards.md`: Document validation layer separation

---

## GROUP B: Storage Implementation (Sprint 2-3)

### Story 3.19: Implement BoltDB Hot Cache Adapter

**Priority**: CRITICAL - Hybrid storage hot layer
**Effort**: 5 points
**Insert Location**: After 3.18

**Description**: Implement BoltDB cache adapter using BoltDBMetadata (Story 3.17) for sub-millisecond hot path queries (PathQuery, BasenameQuery, AliasQuery).

**Acceptance Criteria**:
1. ✅ `/internal/adapters/spi/cache/boltdb_writer.go` implements CacheWriterPort
2. ✅ `/internal/adapters/spi/cache/boltdb_reader.go` implements CacheReaderPort
3. ✅ Bucket structure:
   - `/notes/` - primary bucket: NoteID → BoltDBMetadata
   - `/indices/byPath/` - secondary index: Path → NoteID
   - `/indices/byBasename/` - secondary index: Basename → []NoteID
   - `/indices/byAlias/` - secondary index: Alias → []NoteID
   - `/indices/byFileClass/` - secondary index: FileClass → []NoteID
4. ✅ Store BoltDBMetadata only (not full Note) - minimal hot data
5. ✅ Atomic writes using `bolt.Tx` transactions
6. ✅ Secondary index maintenance on write (update all indices transactionally)
7. ✅ Performance target: Path/basename/alias lookups < 1ms
8. ✅ Error wrapping per FR9 requirements
9. ✅ Structured logging (zerolog) for all operations
10. ✅ Unit tests: persist/delete/read/list, index queries, error paths
11. ✅ Integration tests: concurrent reads/writes, index consistency
12. ✅ `golangci-lint run` and `go test` succeed

**Dependencies**:
- Depends on: Story 3.17 (BoltDBMetadata DTO defined)
- Blocks: Story 3.21 (write coordination needs BoltDB ready)

**Risks**:
- MEDIUM: BoltDB transaction rollback complexity
- Mitigation: Comprehensive error handling; transaction testing

---

### Story 3.20: Implement SQLite Deep Storage with Schema-Driven Views

**Incorporates**: AI-1.4 (MetadataQueryPort for O(1) indexed queries), Gap 4.1 resolution

**Priority**: CRITICAL - Hybrid storage deep layer + indexed queries
**Effort**: 5 points
**Insert Location**: After 3.19

**Description**: Implement SQLite adapter using SQLiteMetadata (Story 3.17) with schema-driven view generation for O(1) indexed frontmatter queries.

**Acceptance Criteria**:

**Basic SQLite Adapter**:
1. ✅ `/internal/adapters/spi/cache/sqlite_writer.go` implements CacheWriterPort
2. ✅ `/internal/adapters/spi/cache/sqlite_reader.go` implements CacheReaderPort
3. ✅ Table schema:
   ```sql
   CREATE TABLE notes (
       id          TEXT PRIMARY KEY,
       path        TEXT UNIQUE NOT NULL,
       frontmatter TEXT,  -- JSON
       mod_time    INTEGER,
       size        INTEGER
   );
   CREATE INDEX idx_notes_path ON notes(path);
   CREATE INDEX idx_notes_mod_time ON notes(mod_time);
   ```

**Schema-Driven View Generation**:
4. ✅ Function: `GenerateSchemaView(schema domain.Schema) (string, error)`
5. ✅ View naming: `v_{schema_name}_notes` (e.g., `v_contact_notes`, `v_project_notes`)
6. ✅ Typed column extraction from JSON:
   ```sql
   CREATE VIEW v_contact_notes AS
   SELECT
       id,
       path,
       json_extract(frontmatter, '$.name') AS name,
       json_extract(frontmatter, '$.email') AS email,
       json_extract(frontmatter, '$.phone') AS phone,
       json_extract(frontmatter, '$.status') AS status,
       mod_time
   FROM notes
   WHERE json_extract(frontmatter, '$.fileClass') = 'contact';
   ```
7. ✅ Index creation on view columns:
   ```sql
   CREATE INDEX idx_contact_status ON v_contact_notes(status);
   CREATE INDEX idx_contact_name ON v_contact_notes(name);
   ```
8. ✅ PropertySpec type → SQL type mapping (string, integer, real, text)
9. ✅ View generation during SQLite adapter initialization
10. ✅ Schema changes trigger view recreation (migration strategy documented)

**MetadataQueryPort Implementation**:
11. ✅ Create `/internal/ports/spi/metadata_query.go`:
    ```go
    type MetadataQueryPort interface {
        TagQuery(ctx context.Context, tag string) ([]domain.Note, error)
        FileClassQuery(ctx context.Context, fileClass string) ([]domain.Note, error)
        FrontmatterQuery(ctx context.Context, field, value string) ([]domain.Note, error)
    }
    ```
12. ✅ SQLiteReader implements MetadataQueryPort
13. ✅ FileClassQuery uses schema-specific view (not base table):
    ```go
    SELECT * FROM v_contact_notes WHERE status = ?
    ```

**Performance**:
14. ✅ Performance test: Query `v_contact_notes WHERE status = 'active'` < 50ms for 1000 notes
15. ✅ Benchmark: O(1) indexed queries vs O(n) JSON scanning (show improvement)

**Testing**:
16. ✅ Unit tests: View generation, type mapping, index creation
17. ✅ Integration tests: Schema changes, view migration
18. ✅ Performance tests: Query speed, scaling to 1000+ notes
19. ✅ `golangci-lint run` and `go test` succeed

**Dependencies**:
- Depends on: Story 3.17 (SQLiteMetadata DTO defined)
- Blocks: Story 3.21 (write coordination needs SQLite ready)
- Resolves: Gap 4.1, Issue A5

**Risks**:
- HIGH: View generation complexity (PropertySpec → SQL type mapping)
- Mitigation: Start with simple types (string, integer); incremental expansion
- MEDIUM: Schema changes require view migration
- Mitigation: Document regeneration procedure; version views

---

### Story 3.21: Implement Storage Write Coordination (Unit of Work)

**Incorporates**: Issue A6 (Storage Write Coordination)

**Priority**: CRITICAL - Prevents data inconsistency
**Effort**: 5 points
**Insert Location**: After 3.20

**Description**: Implement Unit of Work pattern for coordinated BoltDB + SQLite dual-write operations with transactional guarantees and rollback on partial failure.

**Acceptance Criteria**:

**Unit of Work Pattern**:
1. ✅ Create `/internal/app/cache/unit_of_work.go`:
   ```go
   type CacheUnitOfWork struct {
       boltWriter   spi.CacheWriterPort
       sqliteWriter spi.CacheWriterPort
       operations   []operation
       mu           sync.Mutex
   }

   func (uow *CacheUnitOfWork) Begin() error
   func (uow *CacheUnitOfWork) AddWrite(note domain.Note) error
   func (uow *CacheUnitOfWork) AddDelete(id domain.NoteID) error
   func (uow *CacheUnitOfWork) Commit(ctx context.Context) error
   func (uow *CacheUnitOfWork) Rollback(ctx context.Context) error
   ```

**Transaction Semantics**:
2. ✅ Batch operations collected during transaction (not immediate writes)
3. ✅ Commit sequence: BoltDB first, then SQLite (hot cache priority)
4. ✅ BoltDB write failure → entire transaction rollback (no SQLite write attempted)
5. ✅ SQLite write failure → rollback BoltDB changes (compensating transaction)
6. ✅ Transaction isolation - concurrent transactions don't interfere (mutex)

**Rollback Strategy**:
7. ✅ BoltDB rollback: Use `bolt.Tx.Rollback()`
8. ✅ SQLite rollback: Use SQL `ROLLBACK` statement
9. ✅ Compensating writes: If SQLite fails, delete from BoltDB what was written

**Integration**:
10. ✅ VaultIndexer uses CacheUnitOfWork for all cache writes
11. ✅ CLI commands wrap operations in UoW transactions
12. ✅ Error handling: Proper context, structured logging

**Testing**:
13. ✅ Unit tests: Begin/Commit/Rollback lifecycle
14. ✅ Integration test: Simulate SQLite write failure, verify BoltDB rollback
15. ✅ Integration test: Simulate BoltDB write failure, verify no SQLite write
16. ✅ Concurrency test: Parallel transactions don't interfere
17. ✅ Performance test: Transactional write overhead < 10% vs direct writes

**Dependencies**:
- Depends on: Stories 3.19 (BoltDB), 3.20 (SQLite)
- Blocks: Story 3.30 (Hybrid DI/E2E needs coordinated writes)
- Resolves: Issue A6, Gap 4.2 (partial)

**Risks**:
- HIGH: Transaction rollback complexity (two storage systems)
- Mitigation: Comprehensive error handling; test all failure modes
- MEDIUM: Performance impact of transactions
- Mitigation: Benchmark; acceptable for consistency guarantees

**Architecture Doc Updates**:
- `docs/architecture/patterns.md`: Add Unit of Work pattern explanation
- `docs/architecture/components.md`: Document CacheUnitOfWork

---

### Story 3.22: QueryService Hybrid Storage Enhancement

**Incorporates**: Issue B1 (QueryService Command/Query Mixing), CQRS compliance

**Priority**: HIGH - Fixes shipped CQRS violation, enables hybrid queries
**Effort**: 3 points
**Insert Location**: After 3.21

**Description**: Refactor QueryService to support hybrid BoltDB+SQLite query routing and fix CQRS violation (RefreshFromCache is write operation in query service).

**Acceptance Criteria**:

**Hybrid Storage Support**:
1. ✅ QueryService constructor accepts both readers:
   ```go
   func NewQueryService(
       boltReader   spi.CacheReaderPort,
       sqliteReader spi.CacheReaderPort,  // Also implements MetadataQueryPort
       config       domain.Config,
       log          zerolog.Logger,
   ) *QueryService
   ```

**Query Routing Strategy**:
2. ✅ Hot path (BoltDB): `PathQuery()`, `BasenameQuery()`, `AliasQuery()` → sub-millisecond
3. ✅ Deep path (SQLite): `FrontmatterFieldQuery()`, complex queries → use MetadataQueryPort
4. ✅ Merger logic: Combine results from both stores with consistency validation
5. ✅ Consistency check: Verify BoltDB and SQLite ModTime match before returning

**CQRS Compliance Fix**:
6. ✅ Remove `RefreshFromCache()` from QueryService (write operation in query service)
7. ✅ Move index rebuilding to new `IndexMaintenanceService` (command side)
8. ✅ QueryService is now **read-only** (true CQRS query side)

**Incremental Refresh Fix**:
9. ✅ Fix "ModTime filtering broken" (lines 464-467 in service.go)
10. ✅ Implement staleness detection using SQLite: `SELECT * FROM notes WHERE mod_time > ?`
11. ✅ IndexMaintenanceService.RefreshIncremental(since time.Time) loads only modified notes

**Testing**:
12. ✅ Unit tests: Query routing (hot vs deep), merger logic
13. ✅ Integration test: Index 100 notes, modify 5, RefreshIncremental loads only 5
14. ✅ Performance test: Hot path < 1ms, deep path < 50ms
15. ✅ Consistency test: BoltDB/SQLite mismatch detection

**Dependencies**:
- Depends on: Story 3.21 (hybrid storage operational)
- Blocks: Story 3.30 (Hybrid DI/E2E needs correct query routing)
- Resolves: Issue B1, Gap 2.1 (partial)

**Risks**:
- MEDIUM: Query routing logic complexity
- Mitigation: Clear routing rules; comprehensive testing
- LOW: Consistency validation overhead
- Mitigation: Only validate on mismatch suspicion (heuristic)

**Architecture Doc Updates**:
- `docs/architecture/components.md`: Update QueryService, add IndexMaintenanceService
- `docs/architecture/patterns.md`: Document CQRS read/write separation

---

## GROUP C: Service Refactoring (Sprint 3-4)

### Story 3.23: FrontmatterService Refactoring - Use MarkdownParserPort

**Priority**: HIGH - Completes domain purity refactoring
**Effort**: 2 points
**Insert Location**: After 3.22

**Description**: Refactor FrontmatterService to use MarkdownParserPort (Story 3.18) instead of direct goldmark parsing, focusing on semantic validation only.

**Acceptance Criteria**:
1. ✅ FrontmatterService constructor accepts MarkdownParserPort (injected)
2. ✅ Remove all goldmark-related code from FrontmatterService
3. ✅ FrontmatterService.Validate() renamed to `IsSchemaCompliant()` (semantic validation)
4. ✅ Semantic validation focus: schema compliance, required fields, type checking
5. ✅ Uses Frontmatter entity methods (from Story 3.24) for field access
6. ✅ Unit tests run without goldmark (pure domain tests)
7. ✅ Integration tests: End-to-end with MarkdownParserPort

**Dependencies**:
- Depends on: Story 3.18 (MarkdownParserPort exists), Story 3.24 (Frontmatter enriched)
- Resolves: Hexagonal architecture compliance

---

### Story 3.24: Enrich Frontmatter Entity with Validation & Factory

**Priority**: HIGH - Enables proper domain modeling
**Effort**: 2 points
**Insert Location**: After 3.23

**Description**: Transform Frontmatter from anemic data bag into rich domain entity with validation, factory methods, and type-safe field access.

**Acceptance Criteria**:
1. ✅ `Frontmatter.Validate() error` - semantic validation
2. ✅ `NewFrontmatter(fields map[string]any) (Frontmatter, error)` - factory with validation
3. ✅ Type-safe accessors: `GetString()`, `GetStringSlice()`, `GetInt()`, `GetBool()`
4. ✅ `HasField(key string) bool` - field existence check
5. ✅ All Frontmatter creation through factory (enforced validation)
6. ✅ FrontmatterService uses entity methods (not direct field access)
7. ✅ Unit tests: validation edge cases, type-safe accessors

**Dependencies**:
- Depends on: Story 3.23 (FrontmatterService ready)
- Blocks: Story 3.25 (Note entity needs Frontmatter methods)
- Resolves: Issue D1 (partial - Frontmatter anemic model)

---

### Story 3.25: Enrich Note Entity with Behavior Methods

**Priority**: MEDIUM - Completes domain model enrichment
**Effort**: 2 points
**Insert Location**: After 3.24

**Description**: Transform Note from anemic data bag into rich domain entity with behavior methods.

**Acceptance Criteria**:
1. ✅ `Note.Validate() error` - semantic validation
2. ✅ Convenience delegation methods: `HasFrontmatterField()`, `GetFrontmatterString()`
3. ✅ `NewNote(id NoteID, frontmatter Frontmatter) (Note, error)` - factory with validation
4. ✅ Services use Note methods instead of direct field access
5. ✅ Unit tests: validation, delegation

**Dependencies**:
- Depends on: Story 3.24 (Frontmatter enriched)
- Resolves: Issue D1 (complete - Note anemic model)

---

### Story 3.26: Document Validation Layer Separation

**Priority**: HIGH - Clarifies hexagonal architecture pattern
**Effort**: 1 point
**Insert Location**: After 3.25

**Description**: Document validation layer separation with clear naming conventions.

**Acceptance Criteria**:
1. ✅ Architecture doc: validation layer table (syntactic vs semantic)
2. ✅ Naming convention:
   - `ValidateSyntax()` / `IsValidSyntax()` - adapter layer
   - `Validate()` / `IsSchemaCompliant()` - domain layer
3. ✅ All validation call sites updated to use new naming
4. ✅ Code comments clarify validation types

**Dependencies**:
- Depends on: Stories 3.23-3.25 (validation refactoring complete)
- Resolves: Hexagonal Principle violation (complete)

---

## GROUP D: Configuration & Documentation (Sprint 4)

### Story 3.27: Implement Singleton Pattern for Config & PropertyBank

**Incorporates**: Issue A2 (Singleton Pattern Implementation)

**Priority**: HIGH - Required for DI
**Effort**: 2 points
**Insert Location**: After 3.26

**Description**: Implement singleton pattern using sync.Once for Config and PropertyBank.

**Acceptance Criteria**:
1. ✅ `config.Instance() *Config` - singleton accessor using sync.Once
2. ✅ `propertybank.Instance() *PropertyBank` - singleton accessor using sync.Once
3. ✅ Thread-safe initialization (sync.Once guarantees single initialization)
4. ✅ Test harness support: `SetInstanceForTesting()` for test isolation
5. ✅ Update all Config/PropertyBank usage to use Instance()
6. ✅ Unit tests: singleton behavior, concurrency safety
7. ✅ Integration tests: DI container uses singletons

**Dependencies**:
- Depends on: Stories 3.1-3.26 (foundation complete)
- Blocks: Story 3.30 (DI needs singletons)
- Resolves: Issue A2

**Architecture Doc Updates**:
- `docs/architecture/patterns.md`: Add Singleton pattern explanation

---

### Story 3.28: Add FileClassKey Configuration Support

**Incorporates**: Issue A3 (FileClassKey Configuration Impact)

**Priority**: MEDIUM - Enables schema flexibility
**Effort**: 1 point
**Insert Location**: After 3.27

**Description**: Add Config.FileClassKey field with default "fileClass" to support custom schema selection keys.

**Acceptance Criteria**:
1. ✅ `Config.FileClassKey string` field added (default: "fileClass")
2. ✅ Schema resolution uses Config.FileClassKey instead of hardcoded "fileClass"
3. ✅ Config loading validates FileClassKey is non-empty
4. ✅ Architecture doc: fileClass key customization guidance
5. ✅ Unit tests: custom key resolution
6. ✅ Integration test: vault using custom schema key (e.g., "type")

**Dependencies**:
- Depends on: Story 3.27 (singleton Config)
- Resolves: Issue A3

---

### Story 3.29: Document Orchestration Pattern Decision

**Incorporates**: Issue A1 (Component Orchestration Architecture)

**Priority**: MEDIUM - Clarifies architecture pattern
**Effort**: 1 point
**Insert Location**: After 3.28

**Description**: Evaluate and document orchestration pattern decision (Orchestrator vs Event-Driven).

**Acceptance Criteria**:
1. ✅ Architecture doc: orchestration pattern decision
2. ✅ Trade-offs analysis: Orchestrator vs Event-Driven vs Mediator vs Saga
3. ✅ Decision rationale: why chosen pattern fits Lithos needs
4. ✅ If Event-Driven chosen: Document migration path (defer to post-Epic 3)
5. ✅ If Orchestrator retained: Document god-object mitigation strategies
6. ✅ Write coordination pattern (Story 3.21 UoW) aligned with decision

**Dependencies**:
- Depends on: Story 3.21 (UoW pattern for write coordination)
- Informs: Story 3.30 (DI wiring follows chosen pattern)
- Resolves: Issue A1 (decision documented)

**Recommendation**: Retain Orchestrator pattern (CLICommander) for Epic 3 MVP; evaluate Event-Driven for Epic 6+ if complexity grows.

---

## COMPLETION LAYER: Final Integration

### Story 3.30: Hybrid Architecture DI and Production-Scale E2E Testing

**Original Story**: 3.17 (Ready for Enhancement - Course Correction Applied)
**New Number**: 3.30 (after 14 story insertions)
**Status**: ✅ UNBLOCKED - All foundation work complete
**Enhanced Effort**: 7 points (was 5 points)

**Description**: Wire all hybrid architecture components through dependency injection and validate production-ready performance with 500+ note test vault.

**Additional Acceptance Criteria** (beyond original story):
1. ✅ DI container includes MarkdownParserPort → GoldmarkParserAdapter
2. ✅ DI container includes singleton Config, PropertyBank (Story 3.27)
3. ✅ DI container includes CacheUnitOfWork for coordinated writes (Story 3.21)
4. ✅ DI container includes BoltDB + SQLite dual cache readers/writers
5. ✅ QueryService wired with hybrid BoltDB+SQLite readers (Story 3.22)
6. ✅ IndexMaintenanceService wired (CQRS command side)
7. ✅ E2E tests use layered VaultFile DTOs (Story 3.17)
8. ✅ E2E tests validate schema-driven view queries (Story 3.20)
9. ✅ Performance test: Template query < 100ms at 500+ note scale
10. ✅ Performance test: BoltDB hot path < 1ms, SQLite deep path < 50ms

**Dependencies**:
- Depends on: ALL stories 3.17-3.29 (complete foundation)
- Unblocks: Story 3.31 (documentation can finalize)

---

### Story 3.31: Documentation Update with Architecture Patterns

**Original Story**: 3.18 (Ready for Enhancement - Course Correction Applied)
**New Number**: 3.31 (after 14 story insertions)
**Enhanced Effort**: 3 points (was 1 point)

**Description**: Update all architecture documentation including new patterns catalog.

**Additional Acceptance Criteria**:
1. ✅ Create `docs/architecture/patterns.md` with pattern catalog:
   - Singleton Pattern (Config, PropertyBank)
   - Factory Pattern (NewNote, NewFrontmatter)
   - Repository Pattern (VaultReaderPort, CacheReaderPort)
   - Unit of Work Pattern (CacheUnitOfWork)
   - CQRS Pattern (read/write separation)
   - DTO Pattern (layered architecture)
   - Hexagonal Architecture (Ports & Adapters)
2. ✅ Each pattern: intent, when to use, implementation example, trade-offs
3. ✅ Update `docs/architecture/components.md`: all new components
4. ✅ Update `docs/architecture/data-models.md`: layered DTO architecture
5. ✅ Cross-references between docs

**Dependencies**:
- Depends on: Story 3.30 (all implementation complete)

---

## Technical Debt Items (Deferred Post-Epic 3)

### Template System Refactoring (AI-2.3)

**Defer to**: Epic 5 (Interactive Input Engine)
**Effort**: 2 points
**Description**: Refactor template caching to use text/template composition instead of custom caching.

**Rationale**: Template system is functional; this is optimization, not blocker.

---

### MetadataCacheService (AI-2.1)

**Defer to**: Post-Epic 3 optimization
**Effort**: 3 points
**Description**: Add in-memory parsed metadata cache with checksum validation for performance.

**Rationale**: QueryService works without it; nice-to-have optimization.

---

### Schema System Minor Issues

**Defer to**: Post-Epic 3 cleanup
**Effort**: 2 points
**Issues**: Property.validated flag mutation, Schema.ResolvedProperties leak

**Rationale**: Minor code quality issues, not blockers.

---

## Epic Impact Assessment Summary

### Total New Stories: 14

| Group | Stories | Effort | Priority | Sprint |
|-------|---------|--------|----------|--------|
| **GROUP A** | Foundation (3.17-3.18) | 6 pts | CRITICAL | 1 |
| **GROUP B** | Storage (3.19-3.22) | 18 pts | CRITICAL | 2-3 |
| **GROUP C** | Services (3.23-3.26) | 7 pts | HIGH | 3-4 |
| **GROUP D** | Config/Docs (3.27-3.29) | 4 pts | MEDIUM | 4 |
| **COMPLETION** | Integration (3.30-3.31) | 10 pts | CRITICAL | 5 |
| **TOTAL** | **14 stories** | **45 pts** | - | **4-5 sprints** |

### Final Epic 3 Structure

**Original Epic 3**: 18 stories (3.1-3.18)
**After Course Correction**: 31 stories (3.1-3.31)

```
3.1-3.16   Existing stories (mostly complete)
3.17-3.18  GROUP A: Foundation DTOs & Ports
3.19-3.22  GROUP B: Storage Implementation
3.23-3.26  GROUP C: Service Refactoring
3.27-3.29  GROUP D: Configuration & Documentation
3.30       Hybrid DI/E2E (was 3.17 - unblocked)
3.31       Documentation (was 3.18 - enhanced)
```

### Critical Path Dependencies

```
[FOUNDATION LAYER - Sprint 1]
3.17 VaultFile DTO Redesign
  ↓
3.18 MarkdownParserPort & Adapter
  ↓
[STORAGE LAYER - Sprint 2-3]
3.19 BoltDB Hot Cache
  ↓
3.20 SQLite Deep Storage + Views
  ↓
3.21 Write Coordination (UoW)
  ↓
3.22 QueryService Hybrid Enhancement
  ↓
[SERVICE LAYER - Sprint 3-4]
3.23 FrontmatterService Refactoring
  ↓
3.24 Enrich Frontmatter Entity
  ↓
3.25 Enrich Note Entity
  ↓
3.26 Document Validation Layers
  ↓
[CONFIGURATION LAYER - Sprint 4]
3.27 Singleton Pattern
  ↓
3.28 FileClassKey Configuration
  ↓
3.29 Orchestration Pattern Decision
  ↓
[COMPLETION LAYER - Sprint 5]
3.30 Hybrid DI/E2E Testing
  ↓
3.31 Documentation Finalization
```

### Gaps Resolved

| Gap ID | Description | Resolved By |
|--------|-------------|-------------|
| 1.1 | FileMetadata Duplication | Story 3.17 |
| 1.2 | Absolute Paths | Story 3.17 |
| 1.3 | File + Content Conflation | Story 3.17 |
| 2.1 | No Metadata Cache | Story 3.22 (partial), Tech Debt (full) |
| 3.1 | Goldmark in Domain | Story 3.18 |
| 4.1 | No Indexed Query Layer | Story 3.20 |
| 4.2 | No Hot/Deep Separation | Stories 3.19-3.21 |
| 5.1 | Custom Template Cache | Tech Debt (Epic 5) |
| 7.1 | Platform-Specific Paths | Story 3.17 |

### Issues Resolved

| Issue | Description | Resolved By |
|-------|-------------|-------------|
| D1 | Anemic Domain Model | Stories 3.24-3.25 |
| D2 | DTO Architecture Mismatch | Story 3.17 |
| B2 | IO in Domain Layer | Story 3.18 |
| B1 | QueryService Command/Query Mixing | Story 3.22 |
| A1 | Component Orchestration | Story 3.29 |
| A2 | Singleton Pattern | Story 3.27 |
| A3 | FileClassKey Config | Story 3.28 |
| A4 | DTO Architecture | Story 3.17 |
| A5 | SQLite Schema Optimization | Story 3.20 |
| A6 | Storage Write Coordination | Story 3.21 |

### Performance Targets

- BoltDB hot path queries: < 1ms
- SQLite deep path queries: < 50ms
- Template rendering at 500+ notes: < 100ms
- Incremental indexing: Only modified notes loaded

### Timeline Estimate

**Total Duration**: 4-5 sprints (assuming 10-13 points per sprint)

- Sprint 1: Foundation (6 pts)
- Sprint 2: Storage Part 1 (10 pts)
- Sprint 3: Storage Part 2 + Services Start (8 pts)
- Sprint 4: Services Complete + Config (11 pts)
- Sprint 5: Integration & Documentation (10 pts)

**Epic 3 Completion**: ~10-12 weeks from start of course correction

---

_Epic Impact Assessment Complete. Ready for Synthesis Phase: Cross-group dependency map and comprehensive story plan finalization._
