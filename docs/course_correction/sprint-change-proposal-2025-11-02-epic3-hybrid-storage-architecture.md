# Sprint Change Proposal: Epic 3 Hybrid Storage Architecture

**Date**: November 2, 2024
**Reviewer**: Sarah (Product Owner)
**Status**: APPROVED
**Priority**: Critical - Epic 3 Completion

---

## Executive Summary

This proposal pivots Epic 3 (Vault Indexing Engine) from a JSON file-per-note caching approach to a hybrid BoltDB + SQLite architecture to ensure production-ready performance at realistic vault scales (500+ notes). The change enhances the final two Epic 3 stories while preserving all completed work through interface-based design.

---

## Change Trigger Analysis

### Triggering Issue
**Performance Scalability Concern**: The current JSON file-per-note caching approach (implemented in Story 3.2) will not scale to realistic vault sizes of 500+ notes, potentially making template queries too slow for production use.

### Issue Classification
- ✅ **Technical Limitation**: Performance constraint affecting core value proposition
- **Scale Reality**: Conservative estimate of 10 notes/day × 60 days = 600 notes
- **Performance Criticality**: Template queries must be fast (sub-100ms) - this is core value proposition

### Supporting Evidence
- **File I/O Scaling**: O(n) file operations for cache warming at scale
- **Template Query Requirements**: Sub-100ms response time for good UX
- **Obsidian Vault Reality**: Thousands of notes are common in real usage
- **Core Value Risk**: Slow template rendering defeats the purpose of structured templates

---

## Epic Impact Assessment

### Current Epic 3 Status
- ✅ **Stories 3.1-3.16**: COMPLETED - All core components implemented with extensive fixes
- 📋 **Story 3.17**: DRAFT - Dependency Injection and E2E Test
- 📋 **Story 3.18**: DRAFT - Documentation Update

### JSON Cache Implementation Analysis
- **Story 3.2**: ✅ IMPLEMENTED using `encoding/json` and `moby/sys/atomicwriter`
- **Current Approach**: File-per-note JSON caching in filesystem
- **Scale Limitation**: O(n) file operations for cache warming/querying at 500+ notes
- **Performance Risk**: Template queries (core value proposition) will be too slow

### Impact on Remaining Stories

**Story 3.17 (DI & E2E)**:
- ❌ **MAJOR IMPACT**: Tests would validate JSON approach that won't scale
- ❌ **Wasted Effort**: E2E tests built around JSON cache would need rewriting
- ❌ **False Validation**: Tests on small testdata wouldn't catch real-world performance issues

**Story 3.18 (Documentation)**:
- ❌ **MAJOR IMPACT**: Would document JSON approach needing immediate replacement
- ❌ **User Confusion**: Performance characteristics would be misleading
- ❌ **Technical Debt**: Documentation would need immediate updating

### Cross-Epic Dependencies
- **Epic 4 (Schema-Driven Lookups)**: Depends on high-performance queries from Epic 3
- **Template Functions**: `ByPath`, `ByFileClass`, `ByFrontmatter` all rely on fast indexing
- **Post-MVP Features**: Query performance affects all future capabilities

---

## Architecture Solution: Hybrid BoltDB + SQLite

### Strategic Architecture Decision

**Hybrid Approach Rationale**:
- **BoltDB**: Hot cache layer for frequent lookups (paths, basenames, titles, aliases, file_class)
- **SQLite**: Deep storage for complex queries and full content
- **Performance Optimization**: Route queries to optimal storage based on query type
- **NFR4 Alignment**: Sophisticated caching architecture for maximum performance

### Performance Architecture Benefits

**BoltDB Hot Cache Layer**:
- ✅ **Lightning Fast**: Path/basename/title/aliases lookups in microseconds
- ✅ **Directory Filtering**: Perfect for initial vault scanning and path-based queries
- ✅ **Memory Efficient**: Keep hot data in BoltDB, full content in SQLite
- ✅ **Concurrent Reads**: Excellent for template rendering performance

**SQLite Deep Storage**:
- ✅ **Complex Queries**: Frontmatter property searches, schema validation
- ✅ **Full Content**: Store complete note content and metadata
- ✅ **Query Optimization**: Leverage SQLite's sophisticated query optimizer
- ✅ **JSON Support**: Native JSON column support for flexible frontmatter

### Template Query Performance Strategy

**Fast Path (BoltDB)**:
```go
// Sub-millisecond lookups
notes := boltCache.ByPathPrefix("/projects/")
note := boltCache.ByBasename("meeting-notes")
aliases := boltCache.ByAlias("contact")
fileClass := boltCache.ByFileClass("contact-schema")
```

**Complex Queries (SQLite)**:
```go
// Still fast, optimized by SQLite
notes := sqliteStore.ByFrontmatter("status", "active")
notes := sqliteStore.ByFileClass("contact-schema")
combined := sqliteStore.ComplexQuery(criteria)
```

---

## Configuration Enhancement Required

### Missing File Class Key Configuration

**Current Problem**: Hard-coded assumption about file class key name in frontmatter
**Impact**: Users cannot choose between snake_case (`file_class`) vs camelCase (`fileClass`) preferences

**Solution**: Add configuration option to domain model

```go
// internal/domain/config.go
type Config struct {
    // ... existing fields ...
    FileClassKey string `yaml:"file_class_key" mapstructure:"file_class_key"`
}
```

**Default Configuration**:
```yaml
file_class_key: "file_class"  # snake_case preference
```

**Alternative Examples**:
```yaml
file_class_key: "fileClass"   # camelCase preference
file_class_key: "type"        # alternative naming
```

---

## Detailed Implementation Changes

### Story 3.2: JSON Cache Adapters → Multi-Storage Cache Adapters

**Current Title**: "JSON Cache Adapters"
**New Title**: "Multi-Storage Cache Adapters (JSON, BoltDB, SQLite)"

**Interface Preservation Strategy**:
- ✅ Keep all existing `CacheWriterPort`/`CacheReaderPort` interfaces
- ✅ Preserve JSON adapter as export/backup mechanism
- 🔄 Add BoltDB adapter for hot data
- 🔄 Add SQLite adapter for deep storage

**Enhanced Acceptance Criteria**:

```markdown
1. ✅ PRESERVE: `internal/adapters/spi/cache/json_writer.go` implements `CacheWriterPort` (for exports/debugging)

2. 🆕 ADD: `internal/adapters/spi/cache/boltdb_writer.go` implements `CacheWriterPort` for hot data storage:
   - Buckets: /paths/, /basenames/, /aliases/, /file_classes/, /directories/
   - Stores: {path, id, title, aliases, file_class} for fast lookups
   - Optimized for read-heavy workloads and concurrent access

3. 🆕 ADD: `internal/adapters/spi/cache/sqlite_writer.go` implements `CacheWriterPort` for deep storage:
   - Tables: notes with JSON frontmatter column
   - Indexes: path, file_class, modified_time
   - Full note content and complex query support

4. 🆕 ADD: Config model includes `file_class_key` setting for user preference
   - Default: "file_class" (snake_case)
   - Supports: "fileClass" (camelCase), "type", etc.
   - Used consistently across both BoltDB and SQLite storage

5. ✅ PRESERVE: All adapters honor cache directory configuration from `domain.Config`
6. ✅ PRESERVE: Structured logging and error wrapping per coding standards
7. ✅ PRESERVE: Unit test patterns for all storage adapters
```

**BoltDB Bucket Structure**:
```
/paths/         -> {path: {id, title, aliases, file_class, file_mod_time, index_time}}
/basenames/     -> {basename: [note_ids]}
/aliases/       -> {alias: note_id}
/file_classes/  -> {file_class: [note_ids]}
/directories/   -> {dir_path: [note_ids]}
/staleness/     -> {path: {file_mod_time, index_time}}  // Fast staleness checks
```

**SQLite Schema**:
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    path TEXT UNIQUE,
    content TEXT,
    frontmatter JSON,
    file_class TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.file_class')),
    file_mod_time INTEGER,      -- File ModTime from filesystem
    index_time INTEGER,         -- When this entry was indexed
    schema_id TEXT
);

CREATE INDEX idx_path ON notes(path);
CREATE INDEX idx_file_class ON notes(file_class);
CREATE INDEX idx_file_mod_time ON notes(file_mod_time);
CREATE INDEX idx_index_time ON notes(index_time);
CREATE INDEX idx_staleness ON notes(file_mod_time, index_time);  -- Staleness detection
CREATE INDEX idx_schema ON notes(schema_id);
```

### Story 3.6: QueryService → Hybrid Query Service

**Current Title**: "QueryService"
**New Title**: "Hybrid Query Service (Smart Routing)"

**Smart Query Routing Strategy**:
- **Hot Queries** → BoltDB: `ByPath`, `ByBasename`, `ByAlias`, directory filtering
- **Complex Queries** → SQLite: `ByFrontmatter`, `ByFileClass`, full-text search
- **Hybrid Queries** → Coordinate between both stores for optimal performance

**Enhanced Acceptance Criteria**:

```markdown
1. ✅ PRESERVE: QueryService implements same public interface (`ByID`, `ByPath`, `ByFileClass`, etc.)

2. 🔄 ENHANCE: Smart query routing based on query type:
   - `ByPath`, `ByBasename`, `ByAlias` → BoltDB for sub-millisecond performance
   - `ByFrontmatter`, complex filtering → SQLite for query optimization
   - Directory-based queries → BoltDB for fast filesystem-like operations

3. 🔄 ENHANCE: `RefreshFromCache` updates both BoltDB and SQLite stores atomically

4. 🆕 ADD: File class lookups use configurable `file_class_key` from config
   - Supports user preference for key naming convention
   - Consistent behavior across all query methods

5. ✅ PRESERVE: Thread-safety with `sync.RWMutex` for concurrent access
6. ✅ PRESERVE: Error handling consistent with error-handling-strategy
7. 🆕 ADD: Performance instrumentation for query routing decisions
8. 🆕 ADD: Query performance meets template rendering requirements (sub-100ms)
```

**Query Routing Logic**:
```go
func (q *QueryService) ByPath(path string) (*Note, error) {
    // Fast path: BoltDB lookup
    return q.boltStore.GetByPath(path)
}

func (q *QueryService) ByFrontmatter(key, value string) ([]*Note, error) {
    // Complex query: SQLite with optimization
    return q.sqliteStore.QueryByProperty(key, value)
}

func (q *QueryService) ByFileClass(fileClass string) ([]*Note, error) {
    // Hybrid: Use BoltDB index, fetch details as needed
    ids := q.boltStore.GetIDsByFileClass(fileClass)
    return q.sqliteStore.GetByIDs(ids)
}
```

### Story 3.17: DI & E2E → Production-Ready Architecture Testing

**Current Title**: "Dependency Injection and E2E Test for Vault Indexing Engine"
**New Title**: "Hybrid Architecture DI and Production-Scale E2E Testing"

**Enhanced Focus**: Validate production-ready performance with realistic vault sizes

**Additional Acceptance Criteria**:

```markdown
16. 🆕 ADD: E2E tests include performance validation with 500+ note test vault
    - Create testdata/vault-large/ with 500+ sample markdown files
    - Validate indexing performance meets requirements
    - Test query performance under realistic load

17. 🆕 ADD: Template query performance validation meets sub-100ms requirements
    - Benchmark path-based queries (BoltDB)
    - Benchmark frontmatter queries (SQLite)
    - Validate hybrid query coordination performance

18. 🆕 ADD: BoltDB and SQLite integration testing under realistic load
    - Test concurrent read/write operations
    - Validate data consistency between stores
    - Test cache refresh operations

19. 🆕 ADD: Configuration testing for file_class_key setting
    - Test with snake_case ("file_class")
    - Test with camelCase ("fileClass")
    - Test with alternative naming ("type")
    - Validate consistent behavior across all components

20. ✅ PRESERVE: All existing DI registration and e2e test patterns
```

**Production Test Data Available**:
```
docs/refs/obsidian/     # Jack's real Obsidian vault (70+ MB, gitignored)
├── 00_system/          # Templates, scripts, configuration
├── 44_work/            # Work projects and documentation
└── 70_pkm/             # Personal knowledge management notes

testdata/
├── vault/              # Small test vault (existing)
├── vault-large/        # 500+ notes extracted from docs/refs/obsidian/
│   ├── templates/      # Real Templater templates
│   ├── projects/       # Real project notes
│   ├── tools/          # Technical documentation
│   └── knowledge/      # PKM notes
└── config-variations/  # Different file_class_key configs
```

**Note**: `docs/refs/obsidian/` contains production-scale real-world data from Jack's Obsidian vault but is gitignored due to size (70+ MB). Use `ls docs/refs/obsidian/` to explore structure and extract subsets for testing. See `docs/refs/obsidian-vault-guide.md` for detailed usage instructions.

### Story 3.18: Documentation → Hybrid Architecture Documentation

**Current Title**: "Documentation Update for Vault Indexing Engine Release"
**New Title**: "Hybrid Architecture Documentation and Performance Guide"

**Enhanced Documentation Requirements**:

```markdown
6. 🆕 ADD: Document hybrid BoltDB+SQLite architecture design
   - Architecture decision rationale
   - Performance characteristics and benchmarks
   - Query routing strategy explanation
   - Storage layer responsibilities

7. 🆕 ADD: Document file_class_key configuration option
   - Configuration examples for different naming conventions
   - Migration guidance for existing frontmatter
   - Best practices for file class naming

8. 🆕 ADD: Document query routing strategy and performance optimization
   - When queries use BoltDB vs SQLite
   - Performance expectations for different query types
   - Optimization recommendations for template authors

9. 🆕 ADD: Document scalability characteristics and limitations
   - Tested vault sizes and performance benchmarks
   - Recommended maximum vault sizes
   - Performance tuning guidance

10. ✅ PRESERVE: All existing documentation update requirements
```

**New Documentation Sections**:

```markdown
## Storage Architecture

### Hybrid BoltDB + SQLite Design
The vault indexing engine uses a sophisticated dual-storage approach:

- **BoltDB Hot Cache**: Ultra-fast lookups for paths, basenames, aliases, file classes
- **SQLite Deep Storage**: Complex queries, full content, relational operations
- **Smart Routing**: Automatic query optimization based on operation type

### Performance Characteristics
- Path lookups: <1ms (BoltDB)
- Frontmatter queries: <50ms (SQLite)
- Full vault indexing: <5s for 1000 notes
- Template rendering: <100ms total

### Configuration
```yaml
file_class_key: "file_class"  # Choose your preference
```

## Query Performance Guide
[Detailed performance expectations and optimization strategies]
```

---

## Technical Debt Assessment

### Interface Preservation Benefits
- ✅ **Zero Breaking Changes**: All existing interfaces remain unchanged
- ✅ **Gradual Migration**: Can implement adapters incrementally
- ✅ **Fallback Options**: JSON adapter preserved for exports/debugging
- ✅ **Testing Continuity**: Existing test patterns remain valid

### JSON Adapter Preservation Strategy
**Debt Level**: LOW ✅
- **Rationale**: Useful for debugging, exports, and backward compatibility
- **Implementation**: Keep as `JsonCacheAdapter` implementing same interfaces
- **Cost**: Minimal - already implemented and tested
- **Benefit**: Flexibility and debugging capabilities

### Configuration Enhancement
**Debt Level**: LOW ✅
- **Addition**: Single config field for `file_class_key`
- **Impact**: Enables user preference without breaking existing setups
- **Default**: Maintains backward compatibility
- **Validation**: Standard config validation patterns

---

## Risk Assessment and Mitigation

### Implementation Risks

**Risk 1: Increased Complexity**
- **Mitigation**: Interface-based design isolates complexity
- **Monitoring**: Performance benchmarks validate benefits outweigh costs

**Risk 2: Data Consistency Between Stores**
- **Mitigation**: Atomic operations and proper error handling
- **Testing**: E2E tests validate consistency under various scenarios

**Risk 3: Dependency Management**
- **Mitigation**: BoltDB and SQLite are mature, well-supported libraries
- **Fallback**: JSON adapter provides alternative if issues arise

### Performance Validation Plan

**Benchmark Suite**:
1. **Path Lookups**: Validate <1ms BoltDB performance
2. **Complex Queries**: Validate <50ms SQLite performance
3. **Full Indexing**: Validate <5s for 1000 notes
4. **Template Rendering**: Validate <100ms end-to-end

**Load Testing**:
- Test with 500, 1000, 2000+ note vaults
- Concurrent query operations
- Memory usage profiling
- Storage size efficiency

---

## Implementation Timeline

### Phase 1: Enhanced Story 3.2 (Multi-Storage Cache Adapters)
- **Duration**: 2-3 development sessions
- **Deliverables**:
  - BoltDB cache adapter implementation
  - SQLite cache adapter implementation
  - file_class_key configuration integration
  - Preserved JSON adapter functionality

### Phase 2: Enhanced Story 3.6 (Hybrid Query Service)
- **Duration**: 2-3 development sessions
- **Deliverables**:
  - Smart query routing implementation
  - Performance optimization for hot queries
  - Configuration integration for file class key

### Phase 3: Enhanced Story 3.17 (Production-Scale Testing)
- **Duration**: 2-3 development sessions
- **Deliverables**:
  - Large test vault creation (500+ notes)
  - Performance validation test suite
  - Configuration variation testing
  - Integration testing under load

### Phase 4: Enhanced Story 3.18 (Comprehensive Documentation)
- **Duration**: 1-2 development sessions
- **Deliverables**:
  - Hybrid architecture documentation
  - Performance guide and benchmarks
  - Configuration documentation
  - User migration guidance

---

## Success Criteria

### Functional Success Metrics
- ✅ All Epic 3 acceptance criteria met with enhanced performance
- ✅ Template query performance <100ms consistently
- ✅ Vault indexing supports 1000+ notes efficiently
- ✅ file_class_key configuration works with all naming conventions
- ✅ All existing functionality preserved and enhanced

### Technical Success Metrics
- ✅ BoltDB path lookups <1ms average
- ✅ SQLite complex queries <50ms average
- ✅ Full vault indexing <5s for 1000 notes
- ✅ Memory usage efficient and bounded
- ✅ No performance regressions in existing functionality

### Process Success Metrics
- ✅ Zero breaking changes to existing APIs
- ✅ All tests pass including new performance validations
- ✅ Documentation accurately reflects production capabilities
- ✅ Epic 3 delivers truly production-ready indexing engine

---

## Dependency and Integration Impact

### New Dependencies
- **BoltDB**: `go.etcd.io/bbolt` - Mature, embedded key-value store
- **SQLite**: `modernc.org/sqlite` - Pure Go SQLite implementation
- **Impact**: Both are lightweight, embedded, no external service dependencies

### Epic 4 Integration Benefits
- ✅ **Enhanced Performance**: Template functions will perform excellently
- ✅ **Solid Foundation**: Complex lookups and validations well-supported
- ✅ **Scalability Confidence**: No concerns about query performance

### Future Feature Enablement
- ✅ **Full-Text Search**: SQLite FTS capabilities available
- ✅ **Advanced Queries**: SQL query capabilities for complex operations
- ✅ **Analytics**: Query performance metrics and usage patterns
- ✅ **Caching Strategies**: Multiple storage layers for optimization

---

## Rollback Plan

### Rollback Triggers
- Performance benchmarks not met after implementation
- Critical bugs in BoltDB or SQLite integration
- Unacceptable increase in complexity

### Rollback Strategy
1. **Revert to JSON Adapter**: Already preserved and functional
2. **Interface Compatibility**: No breaking changes to revert
3. **Configuration Rollback**: Remove file_class_key config, use defaults
4. **Test Coverage**: Existing tests continue to validate JSON approach

### Rollback Effort
- **Estimated Time**: <1 development session
- **Risk Level**: LOW - JSON adapter preserved throughout
- **Data Loss Risk**: NONE - all data remains in vault files

---

## Approval and Next Steps

### Change Approval Status
- ✅ **Technical Review**: Architecture validated for performance and scalability
- ✅ **Impact Assessment**: All implications evaluated and documented
- ✅ **Risk Mitigation**: Comprehensive risk assessment and mitigation strategies
- ✅ **User Approval**: Product Owner approves hybrid architecture pivot

### Immediate Next Steps

1. **Story 3.2 Enhancement**: Begin multi-storage cache adapter implementation
   - Start with BoltDB adapter for hot data
   - Add SQLite adapter for deep storage
   - Integrate file_class_key configuration

2. **Story 3.6 Enhancement**: Implement smart query routing
   - Route queries to optimal storage layer
   - Validate performance requirements met
   - Test configuration integration

3. **Story 3.17 Enhancement**: Create production-scale testing
   - Generate large test vault (500+ notes)
   - Implement performance validation suite
   - Test configuration variations

4. **Story 3.18 Enhancement**: Document hybrid architecture
   - Create comprehensive architecture documentation
   - Document configuration options and performance characteristics
   - Provide user migration guidance

### Epic 3 Completion Target
- **Enhanced Scope**: Hybrid architecture implementation
- **Quality Target**: Production-ready performance at scale
- **Documentation**: Comprehensive user and developer guides
- **Validation**: Performance benchmarks and scalability testing

---

## Conclusion

This course correction transforms Epic 3 from a functional but limited JSON-based implementation to a production-ready, high-performance hybrid storage architecture. By preserving all existing work through interface-based design while adding sophisticated storage capabilities, we deliver:

- **Production Performance**: Sub-100ms template queries at realistic scale
- **User Flexibility**: Configurable file class naming conventions
- **Future-Proof Architecture**: Solid foundation for advanced features
- **Zero Technical Debt**: All work preserved and enhanced

The hybrid BoltDB + SQLite approach represents a sophisticated, performance-optimized solution that exceeds the original Epic 3 goals while maintaining full compatibility with existing functionality.

---

## COURSE CORRECTION: Critical Architectural Questions

**Date**: November 4, 2025
**Status**: IMPLEMENTATION HALTED - ARCHITECTURAL DESIGN REQUIRED
**Trigger**: Post-implementation review revealed critical design gaps requiring resolution

### Implementation Status Update
- ✅ **Story 3.2**: COMPLETED - BoltDB and SQLite cache adapters implemented
- 🔄 **Story 3.6**: UPDATED - QueryService modifications in progress
- ❌ **Critical Gap**: Integration architecture and design validation incomplete

### Architectural Questions Requiring Resolution

#### **Question 1: Component Orchestration Architecture**
**Issue**: How should component orchestration be structured to avoid god objects while maintaining clean architecture?

**Critical Concerns**:
- How should domain services be orchestrated without creating tight coupling?
- What is the proper boundary between application and domain layer orchestration?
- How do we maintain testability while coordinating complex workflows?
- What are the implications for future adapter types (CLI, TUI, LSP)?

**Recommendation Options**:
- [ ] **Option A**: CLIComander handles all orchestration (creates new god-object)
- [ ] **Option B**: Domain services self-orchestrate through DI (makes services god-objects)
- [ ] **Option C**: Hybrid approach with clear layer boundaries (unclear boundaries)
- [ ] **Option D**: Use case services with thin CLIComander router (still has extra layer)
- [ ] **Option E**: Remove CLIComander, DI orchestrates everything (loses hexagonal callback)
- [x] **Option F**: CLIComander as proper orchestrator with focused domain services

**Decision**: **FINALIZED: Option F - CLIComander as Proper Orchestrator (Workflow Coordinator)**

**Rationale**:

CLIComander (renamed from CommandOrchestrator to reduce pattern-name confusion) is the **workflow coordinator** that sequences focused domain services. A deprecated constructor alias `NewCommandOrchestrator` remains for backward compatibility. This solves the god-object problem by:

1. **CLIComander Responsibilities** (The Conductor):
   - Orchestrate workflows by calling focused domain services in sequence
   - Handle cross-cutting concerns (logging, error handling, metrics)
   - Coordinate transactions across multiple services
   - Implement hexagonal callback pattern (domain starts app)
   - Provide unified API surface for all adapter types (CLI, TUI, LSP)

2. **Domain Services Become Focused** (The Specialists):
   - **VaultIndexer**: ONLY orchestrates vault scanning → indexing workflow
     - Dependencies: `vaultScanner`, `cacheWriter`, `config`, `log`
     - Removed: FrontmatterService, SchemaEngine (CommandOrchestrator calls these separately)

   - **FrontmatterService**: ONLY extracts and validates frontmatter
     - Dependencies: `schemaRegistry`, `config`, `log`
     - Removed: Direct SchemaEngine injection (use port instead)

   - **SchemaEngine**: ONLY handles schema operations
     - Dependencies: `schemaLoader`, `schemaRegistry`, `config`, `log`

   - **TemplateEngine**: ONLY handles template rendering
     - Dependencies: `templateLoader`, `config`, `log`

3. **CLIComander Workflow Example**:
   ```go
   // NewNote workflow orchestrated by CLIComander
   func (o *CLIComander) NewNote(ctx context.Context, templateName string) error {
       // Step 1: Load schemas (if needed)
       if err := o.schemaEngine.LoadSchemas(ctx); err != nil {
           return fmt.Errorf("loading schemas: %w", err)
       }

       // Step 2: Render template
       content, err := o.templateEngine.Render(ctx, templateName)
       if err != nil {
           return fmt.Errorf("rendering template: %w", err)
       }

       // Step 3: Extract and validate frontmatter
       fm, err := o.frontmatterService.Extract(ctx, content)
       if err != nil {
           return fmt.Errorf("extracting frontmatter: %w", err)
       }

       // Step 4: Create note and write to vault
       note := domain.NewNote(content, fm)
       if err := o.vaultWriter.Write(ctx, note); err != nil {
           return fmt.Errorf("writing note: %w", err)
       }

       return nil
   }

   // IndexVault workflow orchestrated by CLIComander
   func (o *CLIComander) IndexVault(ctx context.Context) error {
       // Step 1: Load schemas first
       if err := o.schemaEngine.LoadSchemas(ctx); err != nil {
           return fmt.Errorf("loading schemas: %w", err)
       }

       // Step 2: Build vault index (VaultIndexer now focused on just scan→cache)
       stats, err := o.vaultIndexer.Build(ctx)
       if err != nil {
           return fmt.Errorf("building index: %w", err)
       }

       // Step 3: Extract and validate frontmatter for all notes
       for _, notePath := range stats.IndexedPaths {
           content, _ := o.vaultReader.Read(ctx, notePath)
           fm, err := o.frontmatterService.Extract(ctx, content)
           if err != nil {
               o.log.Warn("frontmatter extraction failed", "path", notePath, "error", err)
               continue
           }
           // Update cache with validated frontmatter
           o.cacheWriter.UpdateFrontmatter(ctx, notePath, fm)
       }

       return nil
   }
   ```

**Key Benefits**:
- ✅ **Solves God-Object Problem**: Each domain service has 3-4 dependencies max
- ✅ **Clear Responsibilities**: CommandOrchestrator = workflow coordination, Services = focused operations
- ✅ **Maintains Testability**: Services remain focused and easily testable
- ✅ **Preserves Hexagonal Pattern**: Domain starts app, unified API surface
- ✅ **Standard Go Pattern**: Orchestrator pattern is well-understood in Go
- ✅ **Future-Proof**: Easy to add TUI/LSP adapters using same orchestrator

**Implementation Changes Required** (in progress):
1. **Refactor VaultIndexer**: Remove FrontmatterService and SchemaEngine dependencies
2. **Refactor FrontmatterService**: Change SchemaEngine to SchemaRegistryPort, add Config
3. **Enhance CLIComander**: Move orchestration logic from VaultIndexer into orchestrator workflows
4. **Update main.go**: Simplified DI with focused services
5. **FileClassKey Adoption**: Ensure note/frontmatter constructors utilize Config.FileClassKey (pending)
6. **Documentation Alignment**: Complete replacement of residual "CommandOrchestrator" references (this document patched; audit others)

**Trade-offs Accepted**:
- CLIComander aggregates more dependencies (its core purpose)
- Slightly larger workflow methods vs previously distributed logic
- Backward-compatible alias temporarily increases surface area
- Services become simpler and more testable (target ≤4 deps each)

**Status**: QUESTION 1 COMPLETE. Proceed to Question 2 only after VaultIndexer & FrontmatterService refactors + FileClassKey integration are merged and tests pass.

---

#### **Question 2: Singleton Pattern Implementation**
**Issue**: How should Config and PropertyBank be managed to balance simplicity with testability?

**Critical Concerns**:
- What are the trade-offs between singleton patterns and dependency injection?
- How should configuration lifecycle and reloading be handled?
- What testing implications exist for different approaches?
- How do we handle concurrent access and initialization safely?

**Recommendation Options**:
- [ ] **Option A**: Implement singleton patterns for global access
- [ ] **Option B**: Maintain pure dependency injection approach
- [ ] **Option C**: Hybrid approach with selective singletons
- [ ] **Option D**: [To be determined based on analysis]

**Current State Analysis**:
Config and PropertyBank are currently passed via DI, but there is only ever one instance per vault lifecycle. This matches the classic singleton use case: a single, authoritative configuration and property registry for the running vault. The Go ecosystem supports safe, idiomatic singletons using sync.Once and package-level variables, as shown in https://refactoring.guru/design-patterns/singleton/go/example#example-0.

**Requirements Analysis**:
- There must only ever be one Config and one PropertyBank per vault instance.
- Both must be globally accessible, but initialized exactly once.
- Thread safety is required for concurrent access and possible reloads.
- Testability must be preserved (ability to reset/replace for tests).

**Option Evaluation**:
1. Option A: Proper Singleton (Recommended)
   - Use package-level variables and sync.Once for initialization.
   - Provide `GetConfig()` and `GetPropertyBank()` accessors.
   - Allow explicit `SetConfigForTest()` and `SetPropertyBankForTest()` for test harnesses.
   - Pros: Simple, idiomatic, zero DI boilerplate, always one instance.
   - Cons: Requires discipline to avoid hidden mutation; test reset logic must be clear.

2. Option B: Pure DI
   - Pass config/propertyBank everywhere; no global state.
   - Pros: Maximum explicitness, easy to swap for tests.
   - Cons: Verbose, unnecessary for single-instance objects, friction for helpers.

3. Option C: Hybrid
   - Mix DI for core services, singleton for helpers/utilities.
   - Pros: Flexible, but can lead to confusion and inconsistent access patterns.

**Decision**: FINALIZED: Option A – Proper Singleton for Config and PropertyBank

**Rationale**:
Config and PropertyBank are true singletons by vault design: only one valid instance per vault lifecycle, and all components must share the same authoritative state. The Go singleton pattern (see refactoring.guru example) is safe, idiomatic, and testable if accessor and reset methods are provided. This avoids DI boilerplate, ensures global consistency, and matches the real-world usage pattern. For tests, explicit reset/set methods allow swapping instances as needed. Thread safety is guaranteed via sync.Once and atomic.Value for PropertyBank. This approach is simple, robust, and aligns with Go best practices for single-instance objects.

**Implementation Impact**:
1. Implement package-level variables for Config and PropertyBank in their respective packages.
2. Use sync.Once for initialization:
   ```go
   var (
       configInstance *Config
       configOnce sync.Once
   )

   func GetConfig() *Config {
       configOnce.Do(func() {
           configInstance = loadConfig() // or set externally
       })
       return configInstance
   }
   ```
   Similar for PropertyBank, using atomic.Value for snapshotting if needed.
3. Provide SetConfigForTest(cfg *Config) and SetPropertyBankForTest(bank *PropertyBank) for test harnesses.
4. Remove deep DI wiring for config/propertyBank; use singleton accessors in all components.
5. Document singleton lifecycle and test reset patterns in coding standards and architecture docs.

**Trade-offs Accepted**:
- Global state requires careful test isolation (reset methods).
- Slightly less explicit than DI, but matches real-world constraints and Go idioms.
- All components must use accessor methods, not direct package-level vars.

**Risk Assessment & Mitigation**:
- Risk: Hidden mutation or accidental re-init. Mitigation: sync.Once, no public setters except for test-only methods.
- Risk: Test flakiness due to global state. Mitigation: Provide explicit reset/set methods for test harnesses.

**Validation Approach**:
1. Prototype singleton accessors; verify only one instance is ever created.
2. Run concurrent access tests; confirm thread safety.
3. Run test suite with SetConfigForTest/SetPropertyBankForTest; confirm isolation.
4. Static analysis: grep for direct package-level var access; enforce accessor usage.

**Success Criteria**:
- Only one Config and one PropertyBank instance per vault lifecycle.
- All components use accessor methods; no direct var access.
- Test suite passes with explicit instance swapping.
- No data races or re-init bugs under -race detector.

**Next Steps Gate**:
Proceed to Question 3 only after: (a) Singleton accessors implemented and used everywhere; (b) Test harnesses updated for instance swapping; (c) Documentation updated for singleton lifecycle and test patterns.

**Documentation Updates Required**:
- Add "Singleton Lifecycle" section to `docs/architecture/components.md`.
- Update `testing-strategy.md` for singleton test isolation.
- Note singleton accessor usage in `coding-standards.md`.


#### **Question 3: FileClassKey Configuration Impact Analysis**
**Issue**: Adding FileClassKey to `internal/domain/config.go` affects multiple components, but its primary purpose is to enable correct schema selection in frontmatter validation. The key is only necessary for updating the `SchemaName` method on the `Frontmatter` struct, which is then used by the `Validate()` method to select and validate against the correct schema. **Additionally, the configuration adapter (ViperAdapter in `internal/adapters/spi/config/viper.go`) must be updated to support FileClassKey: loading from config file, environment variable, and logging.**

**Critical Concerns**:
- Which specific files require updates for FileClassKey support?
- How should configuration validation and migration work?
- What's the backward compatibility strategy?
- How do we ensure consistent behavior across all components?
- How do we ensure schema selection for frontmatter validation is config-driven and reliable?

**Current State Analysis**:
 - **Config**: FileClassKey field added with default "file_class"
 - **ViperAdapter (viper.go)**: ❌ Does not yet load or override FileClassKey from config file or environment variable
 - **QueryService**: ✅ Already uses `config.FileClassKey` correctly
 - **Cache Adapters (BoltDB/SQLite)**: ✅ Already accept `fileClassKey` parameter and use it
 - **Domain Note**: ❌ `extractFileClass()` hard-codes "fileClass" key lookup
 - **FrontmatterService**: ❌ Created without config, calls `domain.NewFrontmatter()` which hard-codes key
 - **Frontmatter Struct**: ❌ No built-in validation method; validation logic is scattered
 - **Schema Selection**: ❌ `SchemaName()` does not use config-driven key, causing inconsistent schema validation

**Requirements Analysis**:
- Need configurable file class key for user preference (snake_case vs camelCase)
- Must maintain backward compatibility with existing "fileClass" usage
- Configuration should be validated and have sensible defaults
- All components must use the same key consistently for schema selection
- Validation logic should be centralized for maintainability and reliability
- Schema selection for frontmatter validation must use the config-driven key via `SchemaName(config.FileClassKey)`

**Analysis Results**:

**Files Requiring Updates:**


1. **internal/domain/frontmatter.go**
   - Update `SchemaName(key string) string` to use the configured key for extracting the schema name from frontmatter.
   - Add `Validate(config *Config) error` method to `Frontmatter` struct, which calls `SchemaName(config.FileClassKey)` and validates against the correct schema.
   - Refactor domain and adapter code to use this method for validation.
   - Backward compatibility: default to "fileClass" if no key provided.

2. **internal/app/frontmatter/service.go**
   - After extraction, call `fm.Validate(config)` before caching or further processing.
   - Update all creation sites (main.go, tests) to ensure config is available for validation.

3. **internal/adapters/spi/config/viper.go**
   - Update `loadConfigFile` to read "file_class_key" from the config file and set `cfg.FileClassKey`.
   - Update `loadEnvironmentVars` to support the `LITHOS_FILE_CLASS_KEY` environment variable.
   - Ensure the field is included in logging for visibility.

4. **Test Files**
   - Add unit tests for `Frontmatter.SchemaName()` and `Validate()` with different FileClassKey configs.
   - Update domain/note tests for configurable key and validation.

**Configuration Validation:**
- Default: "file_class" (snake_case preference)
- Allowed: Any string, validated for non-empty
- Migration: Existing code continues working with "fileClass" default

**Backward Compatibility Strategy:**
- Domain functions accept optional key parameter
- If no key provided, use "fileClass" (current behavior)
- Config default is "file_class" but can be set to "fileClass" for compatibility
- `SchemaName()` and `Validate()` methods should support both legacy and new config

**Testing Approach:**
- Unit tests for each configuration variant
- Integration tests with different FileClassKey settings
- Migration tests ensuring existing behavior preserved
- Tests for `Frontmatter.SchemaName()` and `Validate()` logic and error handling

**Decision**: Implement FileClassKey support across affected components and add a `Validate()` method to the `Frontmatter` struct. Refactor domain, adapter, and service layers to use this method for consistent, config-driven validation before caching or further use.
**Decision**: Implement FileClassKey support specifically for schema selection in frontmatter validation. Update the `SchemaName()` method on the `Frontmatter` struct to use the config-driven key, and ensure the `Validate(config *Config) error` method uses this for correct schema selection. Refactor domain, adapter, and service layers to use this pattern for consistent, config-driven validation before caching or further use.

**Rationale**: The configuration field exists but domain logic doesn't use it for schema selection, causing inconsistent validation. QueryService and cache adapters correctly use the config, but the core domain extraction hard-codes the key. By updating `SchemaName()` to use the config-driven key and ensuring `Validate()` uses this for schema selection, we centralize validation logic, improve maintainability, and ensure all frontmatter is validated against the correct schema before caching or further use. This supports flexible configuration, simplifies service and adapter code, and improves reliability. Impact is manageable—primarily updating domain functions and validation flow. Backward compatibility can be maintained through optional parameters and sensible defaults.

**Implementation Impact:**
 - Update `internal/domain/frontmatter.go`:
    - Update `SchemaName(key string) string` to use the configured key.
    - Add `Validate(config *Config) error` to use `SchemaName(config.FileClassKey)` for schema selection and validation.
 - Refactor domain, adapter, and service code to use this pattern.
 - Update `internal/adapters/spi/config/viper.go`:
    - In `loadConfigFile`, read "file_class_key" and set `cfg.FileClassKey`.
    - In `loadEnvironmentVars`, support `LITHOS_FILE_CLASS_KEY` env var.
    - Add `FileClassKey` to config logging for visibility.
 - Update tests for schema selection and validation logic with different config variants.
**Rationale**: The configuration field exists but domain logic doesn't use it for schema selection, causing inconsistent validation. QueryService and cache adapters correctly use the config, but the core domain extraction hard-codes the key. **ViperAdapter (viper.go) must also be updated to load and override FileClassKey from both config file and environment variable, ensuring the config is fully respected and visible in logs.** By updating `SchemaName()` to use the config-driven key and ensuring `Validate()` uses this for schema selection, we centralize validation logic, improve maintainability, and ensure all frontmatter is validated against the correct schema before caching or further use. This supports flexible configuration, simplifies service and adapter code, and improves reliability. Impact is manageable—primarily updating domain functions, config adapter, and validation flow. Backward compatibility can be maintained through optional parameters and sensible defaults.

**Documentation Updates Required:**
- Update architecture docs to describe the config-driven schema selection and validation flow.
- Note that adapters and services now rely on `Frontmatter.Validate()` and `SchemaName()` for correctness.

**Success Criteria:**
- All components use config-driven file class key for schema selection.
- All frontmatter is validated via the new method before caching or further use.
- Backward compatibility with legacy key usage is maintained.
- Test suite passes for all config variants and validation logic.

**Next Steps Gate:**
Proceed to Question 4 only after: (a) FileClassKey support for schema selection and `Validate()` method are implemented and used everywhere; (b) Test harnesses updated for schema selection and validation logic; (c) Documentation updated for config-driven schema selection and validation flow.

---

#### **Question 4: Data Transfer Object Architecture**
**Issue**: How should DTOs be structured to balance reusability with storage-specific optimizations?

**Critical Concerns**:
- What is the right level of DTO sharing across storage adapters?
- How should data transformation between layers be handled?
- What are the performance implications of different DTO approaches?
- How do we maintain clean separation between filesystem and storage concerns?
- **CRITICAL**: What data does each storage system actually need to store?
- **CRITICAL**: Should storage adapters store full note content (wasteful for metadata-only queries)?

**Current State Analysis**:

**What Each Storage System Currently Stores:**

1. **BoltDBNoteMetadata** (Hot Cache - Fast Lookups):
   ```go
   type BoltDBNoteMetadata struct {
       Path        string    // Derived from NoteID
       ID          string    // NoteID
       Title       string    // From frontmatter["title"]
       Aliases     []string  // From frontmatter["aliases"]
       FileClass   string    // From frontmatter[fileClassKey]
       FileModTime time.Time // From frontmatter["file_mod_time"]
       IndexTime   time.Time // Now()
   }
   ```
   - **Purpose**: Fast path/basename/title/alias/file_class lookups
   - **Does NOT store**: Full frontmatter, note content

2. **SQLite** (Deep Storage - Complex Queries):
   ```sql
   CREATE TABLE notes (
       id TEXT PRIMARY KEY,
       path TEXT NOT NULL,
       title TEXT,
       file_class TEXT,
       frontmatter TEXT,  -- JSON blob with ALL frontmatter
       file_mod_time DATETIME,
       index_time DATETIME
   )
   ```
   - **Currently stores**: Complete frontmatter as JSON blob
   - **Does NOT store**: Note content (markdown body)
   - **Purpose**: Complex frontmatter queries, schema validation

3. **JSON** (Export/Debug - Full Serialization):
   - **Currently stores**: Complete `domain.Note` as JSON (ID + Frontmatter)
   - **Does NOT store**: Note content (not yet on domain.Note)
   - **Purpose**: Debugging, exports, backward compatibility

4. **domain.Note** (Domain Model):
   ```go
   type Note struct {
       ID          NoteID
       Frontmatter Frontmatter
       // Content []byte -- NOT YET IMPLEMENTED
   }
   ```
   - **Currently**: Only ID + Frontmatter
   - **Future**: Will include Content []byte for note body

**Critical Problem Identified:**

**User's Point 2**: "SQLite transforms domain.Note directly. For example, a Note struct will have a content attribute in the future, but sqlite and json must NOT save all the note content, but rather only data or queryable information"

**Analysis**:
- ✅ **BoltDB**: Already extracts only metadata (correct)
- ✅ **SQLite**: Already extracts only queryable metadata + frontmatter (correct, does NOT store content)
- ✅ **JSON**: Stores full Note for exports/debugging (acceptable for its purpose)
- ⚠️ **Future Risk**: When `Note.Content` is added, SQLite/JSON will serialize it unless extraction logic is updated

**Overlapping Concerns Identified:**

**Common Metadata Extraction** (All Storage Systems Need):
1. **ID/Path** - Identifier (string)
2. **Title** - From frontmatter["title"] (string)
3. **FileClass** - From frontmatter[fileClassKey] (string)
4. **FileModTime** - From frontmatter["file_mod_time"] (time.Time)
5. **IndexTime** - Now() (time.Time)
6. **Aliases** - From frontmatter["aliases"] ([]string) - BoltDB only currently

**Storage-Specific Data:**
- **BoltDB**: Needs Aliases for fast alias lookups (not in SQLite)
- **SQLite**: Needs full frontmatter JSON for complex queries (not in BoltDB)
- **JSON**: Needs full Note for debugging/export (not for queries)

**Recommendation Options**:
- [ ] **Option A**: Create shared base DTO with common fields, extend per storage
- [ ] **Option B**: Keep current pattern but extract common transformation functions
- [ ] **Option C**: Create NoteMetadataDTO shared across BoltDB and SQLite
- [x] **Option D**: Extract common metadata, pass only what each storage needs

**Decision**: **FINALIZED - Shared Metadata DTO with Storage-Specific Extensions**

**Rationale**:
The three storage systems (BoltDB, SQLite, JSON) share common metadata extraction needs but differ in what additional data they persist. A layered DTO approach eliminates duplication, prevents accidental `Content` persistence, and provides clear transformation boundaries.

**Finalized Architecture**:

**Step 1: Shared Metadata DTO (Core)**
```go
// NoteMetadataDTO contains queryable metadata common to all storage systems
type NoteMetadataDTO struct {
    ID          string    // NoteID as string
    Path        string    // Full path to note file
    Title       string    // From frontmatter["title"]
    FileClass   string    // From frontmatter[fileClassKey] - config-driven
    FileModTime time.Time // File modification time (for staleness)
    IndexTime   time.Time // When indexed (for staleness)
}
```

**Step 2: Storage-Specific Extensions**
```go
// BoltDBMetadata extends NoteMetadataDTO with hot-cache specific fields
type BoltDBMetadata struct {
    NoteMetadataDTO        // Embedded common fields
    Aliases     []string   // BoltDB needs for fast alias index
}

// SQLiteMetadata extends NoteMetadataDTO with deep-storage specific fields
type SQLiteMetadata struct {
    NoteMetadataDTO           // Embedded common fields
    FrontmatterJSON string    // Full frontmatter as JSON (for schema-driven views)
}

// JSON adapter continues to serialize full domain.Note (ID + Frontmatter)
// for debugging/export - intentionally includes all data
```

**Step 3: Extraction Functions**
```go
// extractNoteMetadata creates shared metadata from domain.Note
// This is the single source of truth for common metadata extraction
func extractNoteMetadata(note domain.Note, fileClassKey string) (NoteMetadataDTO, error) {
    title, _ := note.Frontmatter.Fields["title"].(string)
    fileClass, _ := note.Frontmatter.Fields[fileClassKey].(string)
    fileModTime, _ := note.Frontmatter.Fields["file_mod_time"].(time.Time)
    path, _ := note.Frontmatter.Fields["path"].(string)

    return NoteMetadataDTO{
        ID:          note.ID.String(),
        Path:        path,
        Title:       title,
        FileClass:   fileClass,
        FileModTime: fileModTime,
        IndexTime:   time.Now(),
    }, nil
}

// extractBoltDBMetadata creates BoltDB-specific metadata
func extractBoltDBMetadata(note domain.Note, fileClassKey string) (BoltDBMetadata, error) {
    base, err := extractNoteMetadata(note, fileClassKey)
    if err != nil {
        return BoltDBMetadata{}, err
    }

    aliases, _ := note.Frontmatter.Fields["aliases"].([]string)
    return BoltDBMetadata{
        NoteMetadataDTO: base,
        Aliases:         aliases,
    }, nil
}

// extractSQLiteMetadata creates SQLite-specific metadata
func extractSQLiteMetadata(note domain.Note, fileClassKey string) (SQLiteMetadata, error) {
    base, err := extractNoteMetadata(note, fileClassKey)
    if err != nil {
        return SQLiteMetadata{}, err
    }

    // Serialize full frontmatter to JSON (Question 5: keep as JSON, use views for typed access)
    frontmatterJSON, err := json.Marshal(note.Frontmatter.Fields)
    if err != nil {
        return SQLiteMetadata{}, fmt.Errorf("failed to marshal frontmatter: %w", err)
    }

    return SQLiteMetadata{
        NoteMetadataDTO: base,
        FrontmatterJSON: string(frontmatterJSON),
    }, nil
}
```

**Benefits Validated**:
- ✅ **Eliminates Duplication**: Common extraction logic (`extractNoteMetadata`) shared
- ✅ **Type Safety**: Explicit about what each storage needs
- ✅ **Future-Proof**: When `Note.Content` is added, it's explicitly NOT in `NoteMetadataDTO`
- ✅ **Clear Boundaries**: Metadata vs Content separation is explicit and enforced
- ✅ **Testability**: Shared extraction logic can be unit tested once
- ✅ **Schema-Driven**: FileClass extraction uses config-driven key (Question 3 integration)
- ✅ **View-Ready**: SQLite stores full JSON for schema-driven view generation (Question 5 decision)

**Implementation Impact**:
1. **Create DTOs**: Add `NoteMetadataDTO`, `BoltDBMetadata`, `SQLiteMetadata` to appropriate packages
2. **Refactor BoltDB Writer**: Use `extractBoltDBMetadata` instead of inline extraction
3. **Refactor SQLite Writer**: Use `extractSQLiteMetadata` instead of inline extraction
4. **JSON Writer**: Keep as-is (full `domain.Note` serialization for debug/export)
5. **Add Tests**: Unit tests for extraction functions with various frontmatter configurations
6. **Documentation**: Update architecture docs to describe DTO layering pattern

**Trade-offs Accepted**:
- Slightly more types (3 DTOs) vs inline extraction, but gains clarity and reusability
- Extraction functions add one level of indirection, but centralize transformation logic
- Future `Content` field will NOT accidentally leak into metadata storage (design goal achieved)

**Status**: QUESTION 4 FINALIZED. Proceed to Question 5 with DTO architecture established.

---

#### **Question 5: SQLite Schema Optimization**
**Issue**: We currently persist the entire frontmatter map as a JSON blob in SQLite (`frontmatter TEXT`). This simplifies writes but introduces extra JSON (de)serialization, makes per-field indexing less explicit, and risks wasted storage / query overhead once `Note.Content` is added (which we DON'T want to store in deep storage if not needed). You raised the concern: "why keep frontmatter as JSON if we have already extracted the fields?" – we need to evaluate a more efficient, query-focused schema.

**Current Reality (SQLite Adapter)**:
```sql
CREATE TABLE notes (
     id TEXT PRIMARY KEY,
     path TEXT NOT NULL,
     title TEXT,
     file_class TEXT,
     frontmatter TEXT,    -- full JSON blob today (all fields)
     file_mod_time DATETIME,
     index_time DATETIME,
     UNIQUE(path)
);
```
We do NOT currently store markdown content in SQLite (good). Only metadata + full frontmatter JSON.

**Drivers for Change**:
1. Avoid unnecessary JSON round‑trip when querying common fields (title, status, priority, etc.).
2. Support efficient indexing of frequently queried properties without JSON extraction functions.
3. Prevent accidental inclusion of full future `Note.Content` in deep storage serialization path.
4. Enable schema evolution without brittle migrations for arbitrary/rare fields.

**Frontmatter Field Characteristics**:
| Field Type              | Examples              | Query Frequency     | Stability | Notes                           |
|-------------------------|-----------------------|---------------------|-----------|---------------------------------|
| Core identity           | title, file_class     | High                | Stable    | Already columns or derived      |
| Workflow/status         | status, state, stage  | High                | Medium    | Often filtered in templates     |
| Classification          | tags, aliases, area   | Medium              | Medium    | tags may be list type           |
| Temporal                | created, updated      | Medium              | Medium    | might be normalized to DATETIME |
| Optional / user defined | arbitrary custom keys | Low / unpredictable | Low       | Should not drive schema         |

**Option Evaluation**:

1. **Option A: Fully Columnar (Separate Column Per Known Field)**
    - Create explicit columns for every observed frontmatter key (e.g. `status`, `priority`, `tags`, `created`, `updated`).
    - PROS: Fast queries; no JSON parsing; clear typing per column; easy to index.
    - CONS: Schema churn as new user fields appear; migrations required; dynamic/rare fields bloat table; arrays need join tables or text encoding.
    - Implementation Impact: Needs migration tooling + discovery pass to enumerate stable set. Introduces risk if user adds new unpredictable fields.

2. **Option B: Hybrid (Core Columns + Residual JSON)** *(Common industry pattern)*
    - Keep columns for stable, high‑value fields: `title`, `file_class`, `status`, `file_mod_time`, `index_time`, maybe `created`, `updated`.
    - Store remaining frontmatter in a trimmed JSON blob `frontmatter_extra` or reuse existing `frontmatter` for only non‑core fields.
    - PROS: Performance for common queries; flexibility for long tail; simpler migrations (core set evolves slowly). Can add generated columns referencing JSON if a field graduates to "core".
    - CONS: Two representations of metadata; must ensure no duplication conflicts; requires extraction logic split.
    - Implementation Impact: Moderate—introduce extraction that partitions fields; update adapter and DTOs; add small migration routine.

3. **Option C: Generated Virtual Columns over JSON**
    - Keep single JSON column; add generated columns (SQLite `GENERATED ALWAYS AS (json_extract(...))`) for frequently queried properties (`file_class`, `status`). Index those generated columns.
    - PROS: No duplication; easy to add or remove generated columns; keeps write path simple; avoids storing duplicates.
    - CONS: Still stores whole JSON; read cost for non‑indexed JSON fields; some overhead for each insertion computing generated columns; arrays remain clunky.
    - Implementation Impact: Low—add DDL for generated columns + indices.

4. **Option D: Per-FileClass (Schema-per-Class / Table-per-Type)**
    - Create one table per file_class with tailored columns.
    - PROS: Strong typing; highly efficient queries per class; natural evolution per schema.
    - CONS: High complexity; dynamic DDL; cross-class queries harder; fragmentation; migrations heavy.
    - Implementation Impact: High—requires schema registry + migrator + multi-table query router.

5. **Option E: EAV (Entity-Attribute-Value) Side Table**
    - Keep core `notes` table minimal; add `frontmatter_properties(note_id, key, value TEXT)` with composite indexes.
    - PROS: Infinite flexibility; indexing per key; no JSON parsing.
    - CONS: JOIN overhead; storage inflation; complex queries for multi-key filters; typing lost (all TEXT unless variant tables).
    - Implementation Impact: Moderate/High—introduces additional query builder complexity.

**Performance & Maintenance Trade-offs**:
| Option | Write Simplicity | Query Speed (Core)  | Query Flex (Rare) | Schema Churn | Complexity |
|--------|------------------|---------------------|-------------------|--------------|------------|
| A      | Medium           | High                | Low               | High         | Medium     |
| B      | Medium           | High                | Medium            | Low/Medium   | Medium     |
| C      | High             | High (indexed gens) | Medium            | Low          | Low/Med    |
| D      | Low              | High (per type)     | Low               | High         | High       |
| E      | Medium           | Medium              | High              | Low          | High       |

**Recommended Direction (Preliminary – NEED USER CONFIRMATION)**:
Adopt a **Hybrid with Generated Columns** combining Option B + C:
1. Keep a single `frontmatter` JSON storing ONLY non-core fields (strip out promoted keys during persistence).
2. Promote stable, frequently queried keys to dedicated columns: `title`, `file_class`, `status`, `priority` (if used), `file_mod_time`, `created`, `updated`.
3. Add generated columns for emergent keys before fully promoting them (e.g., `severity` if templates start filtering on it) to avoid immediate migrations.
4. Provide a migration script that: (a) scans existing JSON blobs, (b) populates new columns, (c) rewrites JSON minus promoted keys.
5. Add a metadata registry (simple slice/const list) enumerating "core" frontmatter keys to extract.

**Proposed Revised Schema**:
```sql
CREATE TABLE notes (
     id TEXT PRIMARY KEY,
     path TEXT NOT NULL,
     title TEXT,            -- core
     file_class TEXT,       -- core
     status TEXT,           -- core (optional)
     priority TEXT,         -- core (optional)
     created DATETIME,      -- core (if present)
     updated DATETIME,      -- core (if present)
     file_mod_time DATETIME,
     index_time DATETIME,
     frontmatter TEXT,      -- JSON of ONLY non-core fields
     UNIQUE(path)
);

-- Optional generated column example (if we keep full JSON initially):
-- ALTER TABLE notes ADD COLUMN status TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.status')) STORED;
-- CREATE INDEX idx_notes_status ON notes(status);
```

**Extraction / DTO Adjustment Plan**:
```go
type NoteMetadataDTO struct { /* shared core fields */ }
type SQLiteMetadata struct {
     NoteMetadataDTO
     ExtraJSON string // remaining non-core fields
}

// Partition logic during persistence:
core := map[string]interface{}{"title":..., "file_class":..., "status":..., /* etc */}
extra := filterNonCore(note.Frontmatter.Fields, coreKeys)
```

**Why NOT Full Columnar (Option A) Immediately?**
- Field set may evolve; premature promotion increases migration churn.
- Some frontmatter keys are user-specific or experimental.
- Hybrid lets us observe usage patterns (telemetry / query stats) before locking schema.

**Why NOT Keep Pure JSON?**
- Added overhead for every common query (json_extract or client-side filtering).
- Harder to enforce type constraints (dates, enums) at DB layer.
- Slower indexing for frequently accessed fields.

**Risks & Mitigations**:
| Risk                                     | Mitigation                                                         |
|------------------------------------------|--------------------------------------------------------------------|
| Core key drift (too many promoted)       | Add acceptance threshold (query frequency heuristic)               |
| JSON / columns divergence                | Strip promoted keys consistently; unit test extraction             |
| Migration complexity                     | Provide idempotent migration tool; version stamp table             |
| Future Content field accidentally stored | Explicit DTO excluding Content; adapter never touches Note.Content |

**Validation Strategy**:
1. Prototype hybrid schema on sample vault (500 notes).
2. Benchmark queries: `ByFileClass`, `ByStatus`, combined filters.
3. Compare JSON-only vs Hybrid query latency (expect 20–40% improvement on filtered scans).
4. Confirm storage size delta (expect modest increase from added columns, reduction from trimmed JSON).
5. Write migration test: legacy rows with full JSON upgraded seamlessly.

**Decision**: **FINALIZED - Schema-Driven Views over JSON Storage**

**User Input Received**:
1. Core keys should be **schema-driven** (not manually enumerated)
2. Multi-field filtering is **common** in templates
3. Array fields (tags, aliases) remain in JSON initially (not relational)
4. Migration acceptable; prefer flexible base schema

**Rationale**:
Keep frontmatter as a single JSON column for flexibility and simplicity, but create **schema-driven views** that expose typed columns for each schema's defined properties. This elegantly balances:
- **Write simplicity**: Single JSON write, no field partitioning logic
- **Read performance**: Views provide typed columns with indexes where needed
- **Schema alignment**: Views automatically match schema definitions
- **Migration avoidance**: Base table never changes; views adapt to schema evolution
- **Query flexibility**: Can query via typed view columns OR raw JSON as needed

**Finalized Schema Architecture**:

**Base Table (Simple & Stable)**:
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    file_mod_time INTEGER NOT NULL,  -- Unix timestamp for staleness detection
    index_time INTEGER NOT NULL,     -- Unix timestamp for staleness detection
    frontmatter_json TEXT NOT NULL,  -- Complete frontmatter as JSON

    -- Indexes for common operations
    CREATE INDEX idx_notes_path ON notes(path);
    CREATE INDEX idx_notes_staleness ON notes(file_mod_time, index_time);
);
```

**Schema-Driven View Generation**:
For each loaded schema (e.g., `contact-schema`, `project-schema`, `meeting-schema`), automatically generate a view:

```sql
-- Example: contact-schema defines properties: name, email, phone, company, status
CREATE VIEW IF NOT EXISTS v_contact_notes AS
SELECT
    id,
    path,
    json_extract(frontmatter_json, '$.name') AS name,
    json_extract(frontmatter_json, '$.email') AS email,
    json_extract(frontmatter_json, '$.phone') AS phone,
    json_extract(frontmatter_json, '$.company') AS company,
    json_extract(frontmatter_json, '$.status') AS status,
    json_extract(frontmatter_json, '$.created') AS created,
    json_extract(frontmatter_json, '$.updated') AS updated,
    file_mod_time,
    index_time,
    frontmatter_json  -- Keep full JSON accessible
FROM notes
WHERE json_extract(frontmatter_json, '$.file_class') = 'contact-schema';

-- Optional: Create indexes on frequently queried view columns
CREATE INDEX IF NOT EXISTS idx_contact_notes_status
    ON notes(json_extract(frontmatter_json, '$.status'))
    WHERE json_extract(frontmatter_json, '$.file_class') = 'contact-schema';

-- Example: project-schema defines: title, status, priority, area, stage, owner
CREATE VIEW IF NOT EXISTS v_project_notes AS
SELECT
    id,
    path,
    json_extract(frontmatter_json, '$.title') AS title,
    json_extract(frontmatter_json, '$.status') AS status,
    json_extract(frontmatter_json, '$.priority') AS priority,
    json_extract(frontmatter_json, '$.area') AS area,
    json_extract(frontmatter_json, '$.stage') AS stage,
    json_extract(frontmatter_json, '$.owner') AS owner,
    json_extract(frontmatter_json, '$.created') AS created,
    json_extract(frontmatter_json, '$.updated') AS updated,
    file_mod_time,
    index_time,
    frontmatter_json
FROM notes
WHERE json_extract(frontmatter_json, '$.file_class') = 'project-schema';
```

**View Generation Algorithm**:
```go
// Automatically generate views from loaded schemas
func generateSchemaViews(db *sql.DB, schemaRegistry SchemaRegistry) error {
    for _, schema := range schemaRegistry.AllSchemas() {
        viewName := fmt.Sprintf("v_%s_notes", schema.ID)

        // Extract property names from schema definition
        columns := []string{"id", "path"}
        for propName, propDef := range schema.Properties {
            // Only extract scalar/simple types for view columns
            if propDef.Type == "string" || propDef.Type == "integer" ||
               propDef.Type == "number" || propDef.Type == "boolean" {
                columns = append(columns, fmt.Sprintf(
                    "json_extract(frontmatter_json, '$.%s') AS %s",
                    propName, propName,
                ))
            }
        }
        columns = append(columns, "file_mod_time", "index_time", "frontmatter_json")

        createViewSQL := fmt.Sprintf(`
            CREATE VIEW IF NOT EXISTS %s AS
            SELECT %s
            FROM notes
            WHERE json_extract(frontmatter_json, '$.%s') = '%s'
        `, viewName, strings.Join(columns, ", "), config.FileClassKey, schema.ID)

        if _, err := db.Exec(createViewSQL); err != nil {
            return fmt.Errorf("failed to create view %s: %w", viewName, err)
        }

        // Optionally create indexes for commonly filtered fields
        if schema.QueryIndexes != nil {
            for _, indexField := range schema.QueryIndexes {
                createIndexSQL := fmt.Sprintf(`
                    CREATE INDEX IF NOT EXISTS idx_%s_%s
                    ON notes(json_extract(frontmatter_json, '$.%s'))
                    WHERE json_extract(frontmatter_json, '$.%s') = '%s'
                `, schema.ID, indexField, indexField, config.FileClassKey, schema.ID)

                db.Exec(createIndexSQL) // Best effort
            }
        }
    }
    return nil
}
```

**Query Patterns Enabled**:

```go
// Query via typed view (gets columnar access)
rows, err := db.Query(`
    SELECT name, email, status
    FROM v_contact_notes
    WHERE status = 'active' AND company = 'Acme Corp'
`)

// Multi-field filtering (what user confirmed as common)
rows, err := db.Query(`
    SELECT title, status, priority, area
    FROM v_project_notes
    WHERE status = 'active'
      AND priority IN ('high', 'critical')
      AND area = 'infrastructure'
    ORDER BY updated DESC
`)

// Query raw JSON (for ad-hoc or unstructured queries)
rows, err := db.Query(`
    SELECT frontmatter_json
    FROM notes
    WHERE json_extract(frontmatter_json, '$.custom_field') = 'value'
`)
```

**Key Benefits**:
- ✅ **Schema-Driven**: Views auto-generate from schema definitions (answers user point #1)
- ✅ **Multi-Field Performance**: Indexed columns support complex filters (answers user point #2)
- ✅ **Write Simplicity**: Single JSON column write, no partitioning logic
- ✅ **Query Flexibility**: Can use typed views OR raw JSON as needed
- ✅ **Zero Migration**: Base table stable; views adapt to schema changes
- ✅ **Future-Proof**: Content field will NOT be in frontmatter JSON (separate concern)
- ✅ **Type Safety**: Views provide typed column access; SQLite enforces at query time
- ✅ **Array Support**: Keep arrays (tags, aliases) in JSON; query with json_each() when needed

**Implementation Impact**:
1. **Simplify SQLite Writer**: Single JSON column write (no field extraction beyond metadata)
2. **Add View Generator**: Implement `generateSchemaViews()` function in SQLite adapter
3. **Schema Loading Hook**: Regenerate views when schemas are loaded/reloaded
4. **Query Helpers**: Add utility functions to query via views vs raw JSON
5. **Index Strategy**: Allow schemas to declare which fields need indexes (`queryIndexes` metadata)
6. **Documentation**: Explain view-based query pattern in architecture docs

**Schema Metadata Extension (Optional)**:
```yaml
# In schema YAML files, optionally declare frequently filtered fields
id: contact-schema
properties:
  name: {type: string}
  email: {type: string}
  status: {type: string}
  company: {type: string}
queryIndexes:  # NEW: Hint which fields need indexes for performance
  - status
  - company
```

**Trade-offs Accepted**:
- Views add slight query overhead vs direct columns (negligible in practice)
- Index creation on JSON paths less efficient than native columns (but SQLite handles this well)
- View regeneration needed when schemas change (but automatic via schema load hook)
- Array queries require json_each() (acceptable; not common in templates)

**Why This Approach Wins**:
1. **Eliminates manual core key enumeration**: Schema properties = view columns automatically
2. **Supports multi-field filtering efficiently**: Views give typed columns with indexes
3. **No migration churn**: Base table never changes; only views adapt
4. **Perfect schema alignment**: Views are mechanically derived from schemas
5. **Simple write path**: Single JSON persist (aligns with Question 4 DTO decision)
6. **Query power**: Full SQL capabilities on typed view columns

**Status**: QUESTION 5 FINALIZED. Proceed to Question 6 (write coordination) with schema-driven view architecture established.

---

#### **Question 6: Storage Write Coordination Design**
**Issue**: How should BoltDB and SQLite writes be coordinated when QueryService currently accepts one storage adapter at a time?

**Critical Concerns**:
- What happens when one write succeeds and the other fails?
- Should writes be atomic across both stores or eventual consistency?
- How do we handle rollback scenarios for partial failures?
- What are the performance implications of different coordination approaches?

**Recommendation Options**:
- [ ] **Option A**: Synchronous atomic writes with transaction rollback
- [ ] **Option B**: Eventual consistency with conflict resolution
- [ ] **Option C**: Primary store + async replication pattern
- [ ] **Option D**: [To be determined based on analysis]

**Decision**: [PENDING]
**Rationale**: [TO BE DOCUMENTED]

### Resolution Process

#### **Phase 1: Sequential Question Analysis & Decision Making**
Work through questions in order (1→6), documenting decisions in this document:

**Questions 1-3: Foundation Architecture**
- Analyze each question through discussion and small prototypes
- Document decision and rationale in this change proposal
- Focus on establishing fundamental system structure

**Questions 4-6: Storage & Data Architecture**
- Resolve sequentially as each decision informs the next
- Document decisions and rationale in this change proposal
- Validate decisions through code analysis of existing implementation

#### **Phase 2: Story Creation Based on Decisions**
After all questions resolved, create implementation stories for Epic 3:

- **Story Creation**: Based on architectural decisions, create specific implementation stories
- **Story Scope**: Each story implements one aspect of the decided architecture
- **Integration Focus**: Stories ensure proper integration of existing BoltDB/SQLite work

#### **Phase 3: Implementation & Validation** (Enhanced Stories 3.17-3.18)
- **Story 3.17**: Test validated architectural approach with realistic data
- **Story 3.18**: Document final architecture with decision rationale

### Success Criteria for Course Correction

- [ ] All 6 architectural questions resolved with documented decisions
- [ ] Implementation approach validated through prototyping/analysis
- [ ] Integration patterns defined and tested
- [ ] Performance characteristics validated against realistic requirements
- [ ] Epic 3 completion path clearly defined with proper architecture

### Next Steps

1. **HALT current Epic 3 implementation** until architectural questions resolved
2. **Create analysis stories** (3.19-3.24) to address each architectural question
3. **Document decisions** in this change proposal as they are made
4. **Validate integration approach** before completing Epic 3
5. **Update Epic 3 stories** (3.17-3.18) based on architectural decisions

---

## ARCHITECTURAL ANALYSIS FRAMEWORK

### Analysis Preparation

**Current Implementation Context**:
- ✅ **Story 3.2 COMPLETED**: BoltDB and SQLite adapters implemented with working readers/writers
- 🔄 **Story 3.6 IN PROGRESS**: QueryService modifications started
- 📋 **Available Code**: Existing implementation can be analyzed for patterns and constraints

**Analysis Tools Available**:
- **Code Review**: Examine existing BoltDB/SQLite implementations
- **Interface Analysis**: Review CacheWriterPort/CacheReaderPort contracts
- **Dependency Analysis**: Map current DI patterns and component relationships
- **Performance Testing**: Benchmark existing implementations against requirements

### Question Analysis Template

For each architectural question, use this framework:

#### **1. Current State Analysis**
- What exists in the current implementation?
- What patterns are already established?
- What constraints are imposed by existing code?

#### **2. Requirements Analysis**
- What are the functional requirements?
- What are the non-functional requirements (performance, maintainability, etc.)?
- What are the architectural principles to uphold?

#### **3. Option Evaluation**
- **Option A**: [Description, Pros, Cons, Implementation Impact]
- **Option B**: [Description, Pros, Cons, Implementation Impact]
- **Option C**: [Description, Pros, Cons, Implementation Impact]
- **Option D**: [To be determined through analysis]

#### **4. Recommendation & Rationale**
- **Recommended Option**: [Selected approach]
- **Rationale**: [Why this option best meets requirements]
- **Implementation Impact**: [What changes are needed]
- **Risk Assessment**: [Potential issues and mitigation]

#### **5. Validation Approach**
- How will we validate the decision?
- What prototypes or tests are needed?
- What success criteria apply?

### Analysis Execution Plan

#### **Questions 1-3: Foundation Architecture** (Start Here)

**Question 1: Component Orchestration Architecture**
- **Analysis Focus**: Review current DI patterns, cmd/lithos/ structure, component relationships
- **Key Evidence**: Examine existing FrontmatterService → VaultIndexer flow
- **Decision Impact**: Affects all subsequent architectural decisions

**Question 2: Singleton Pattern Implementation**
- **Analysis Focus**: Review Config and PropertyBank usage patterns
- **Key Evidence**: Examine current initialization and access patterns
- **Decision Impact**: Affects configuration management throughout system

**Question 3: FileClassKey Configuration Impact Analysis**
- **Analysis Focus**: Map all components using file classification
- **Key Evidence**: Code search for file_class usage, frontmatter processing
- **Decision Impact**: Defines configuration integration scope

#### **Questions 4-6: Storage & Data Architecture** (After Questions 1-3 Complete)

**Question 4: Data Transfer Object Architecture**
- **Analysis Focus**: Review FileMetadata/VaultFile usage patterns
- **Key Evidence**: Examine existing BoltDB/SQLite implementations
- **Decision Impact**: Affects storage adapter design

**Question 5: SQLite Schema Optimization**
- **Analysis Focus**: Evaluate current JSON schema vs column-based approach
- **Key Evidence**: Performance testing, query patterns analysis
- **Decision Impact**: Affects data storage efficiency

**Question 6: Storage Write Coordination Design**
- **Analysis Focus**: Design coordination between BoltDB/SQLite writes
- **Key Evidence**: Review existing QueryService interface constraints
- **Decision Impact**: Defines data consistency approach

### Decision Documentation Process

As each question is analyzed:

1. **Update Question Section**: Fill in Decision and Rationale fields
2. **Update Dependencies**: Note how decision affects subsequent questions
3. **Validate Integration**: Ensure decision aligns with existing implementation
4. **Document Trade-offs**: Record what was gained/lost with the decision

### Success Criteria for Analysis Phase

- [ ] All 6 questions have documented decisions with rationale
- [ ] Decisions are validated against existing implementation
- [ ] Integration approach is clear and feasible
- [ ] Performance and maintainability requirements are met
- [ ] Path to Epic 3 completion is defined

### Transition to Implementation

**After All Questions Resolved**:
1. **Create Implementation Stories**: Based on architectural decisions
2. **Update Epic 3**: Modify Stories 3.17-3.18 to reflect validated approach
3. **Resume Development**: Proceed with confidence in architectural foundation
4. **Monitor Integration**: Validate decisions work in practice

---

**Status**: IMPLEMENTATION HALTED - ARCHITECTURAL DESIGN REQUIRED
**Priority**: Critical - Epic 3 Architecture Foundation
**Expected Outcome**: Properly designed hybrid storage architecture with validated integration patterns

**Next Action**: RESTART Question 1 analysis - Need user clarification on architecture roles and responsibilities

---

**Document Control**
- **Version**: 2.0 - Course Correction Added
- **Last Updated**: November 4, 2025
- **Next Review**: Upon architectural questions resolution
- **Distribution**: Development Team, Product Owner, Technical Architecture Team
