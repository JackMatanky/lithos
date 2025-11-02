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

**Status**: APPROVED FOR IMPLEMENTATION
**Priority**: Critical - Epic 3 Completion
**Expected Outcome**: Production-ready vault indexing engine with excellent performance characteristics

---

**Document Control**
- **Version**: 1.0
- **Last Updated**: November 2, 2024
- **Next Review**: Upon implementation completion
- **Distribution**: Development Team, Product Owner, Technical Architecture Team
