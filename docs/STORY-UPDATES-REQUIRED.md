# Epic 3 Story Updates Required

## Summary of Architectural Corrections

Based on architectural review, the following stories require updates for:
1. **Naming consistency**: `IndexedAt` → `IndexTime`, `ModTime` → `ModifiedAt`
2. **File location corrections**: VaultFile, markdown parser, EventBus
3. **Removed concepts**: BoltDBMetadata/SQLiteMetadata DTOs
4. **Cache staleness**: Add `IndexTime` tracking
5. **Test consolidation**: Extend existing tests instead of creating duplicates

---

## ✅ Story 3.17: VaultFile DTO Redesign - COMPLETED

**Changes Made:**
- ✅ Removed Layer 3 (Storage-Specific DTOs) - BoltDBMetadata/SQLiteMetadata
- ✅ Added Cache Staleness Detection (AC 11-12)
- ✅ Changed ModTime → ModifiedAt throughout
- ✅ Updated file locations:
  - `internal/shared/dto/file.go` → `internal/adapters/spi/dto/vault_file.go`
  - `internal/shared/dto/file_test.go` → `internal/adapters/spi/dto/vault_file_test.go`
- ✅ Updated test file references
- ✅ Added staleness test requirement

---

## Story 3.18: MarkdownParserPort - UPDATES NEEDED

### **File Path Changes:**

**FROM (current):**
```
internal/adapters/spi/markdown/markdown_parser.go
internal/adapters/spi/markdown/markdown_parser_test.go
```

**TO (corrected):**
```
internal/adapters/spi/vault/markdown.go
internal/adapters/spi/vault/markdown_test.go
```

**Rationale:** Markdown parsing is ONLY used by vault operations (YAGNI - don't create separate package until needed elsewhere).

### **Test File Changes:**

**FROM:**
```
tests/integration/frontmatter_parsing_test.go (NEW)
```

**TO:**
```
tests/integration/frontmatter_workflow_test.go (EXTEND EXISTING)
```

**Rationale:** Prevent test duplication - extend existing frontmatter workflow test.

### **Search & Replace:**

1. Find: `/internal/adapters/spi/markdown/markdown_parser.go`
   Replace: `/internal/adapters/spi/vault/markdown.go`

2. Find: `/internal/adapters/spi/markdown/markdown_parser_test.go`
   Replace: `/internal/adapters/spi/vault/markdown_test.go`

3. Find: `Create /internal/adapters/spi/markdown/ directory`
   Replace: `Create implementation in /internal/adapters/spi/vault/markdown.go`

4. Find: `tests/integration/frontmatter_parsing_test.go`
   Replace: `tests/integration/frontmatter_workflow_test.go (extend existing)`

---

## Story 3.19: BoltDB Hot Cache - UPDATES NEEDED

### **Add IndexTime Support:**

**New Acceptance Criteria (add after AC 4):**
```
5. ✅ IndexTime tracking for cache staleness:
   - Each cached note includes IndexTime (when note was indexed)
   - Cached data structure: { Note data, ModifiedAt (file's time), IndexTime (cache time) }
6. ✅ IsStale(ctx, path string) (bool, error) method:
   - Returns true if file.ModifiedAt() > cached.IndexTime
   - Enables incremental reindexing of changed files
```

**Update Bucket Structure (AC 3):**
```go
// Primary bucket stores:
type CachedNote struct {
    // ... note fields ...
    ModifiedAt time.Time  // File's modification time (from filesystem)
    IndexTime  time.Time  // When we indexed this note
}
```

### **New Subtasks:**

Add to Feature 2 (Writer Implementation):
```
- [ ] Subtask X.X: Add IndexTime parameter to Write() method
- [ ] Subtask X.X: Store both ModifiedAt and IndexTime in cached data
```

Add new Feature:
```
### Feature X: Staleness Detection

- [ ] Task X: Implement IsStale() Method (TDD Cycle)
  - [ ] Subtask X.1: Write tests for IsStale() with various scenarios
  - [ ] Subtask X.2: Implement IsStale() comparing file ModifiedAt vs cached IndexTime
  - [ ] Subtask X.3: Handle edge cases (missing file, missing cache entry)
```

### **Search & Replace:**

1. Find all: `ModTime`
   Replace: `ModifiedAt`

2. Find: `Store BoltDBMetadata only (not full Note) - minimal hot data`
   Replace: `Store Note with ModifiedAt and IndexTime for staleness detection`

---

## Story 3.20: SQLite Deep Storage - UPDATES NEEDED

### **Schema Changes:**

**FROM (current AC 3):**
```sql
CREATE TABLE notes (
    path        TEXT PRIMARY KEY,
    frontmatter TEXT,  -- JSON
    mod_time    INTEGER,
    size        INTEGER
);
CREATE INDEX idx_notes_mod_time ON notes(mod_time);
```

**TO (corrected):**
```sql
CREATE TABLE notes (
    path          TEXT PRIMARY KEY,
    frontmatter   TEXT,     -- JSON
    modified_at   INTEGER,  -- File's modification time (UNIX timestamp)
    indexed_time  INTEGER,  -- When we cached this (UNIX timestamp)
    size          INTEGER
);
CREATE INDEX idx_notes_modified_at ON notes(modified_at);
CREATE INDEX idx_notes_indexed_time ON notes(indexed_time);

-- Query for stale notes
-- SELECT path FROM notes WHERE modified_at > indexed_time;
```

### **New Acceptance Criteria:**

**Add after current ACs:**
```
XX. ✅ Staleness detection support:
    - indexed_time column tracks when note was cached
    - modified_at column tracks file's modification time
    - Query method: GetStaleNotes() returns paths where modified_at > indexed_time
```

### **View Updates:**

Update all view examples to use `modified_at` instead of `mod_time`:

**FROM:**
```sql
CREATE VIEW v_contact_notes AS
SELECT
    path,
    ...,
    mod_time,
    size
FROM notes
WHERE json_extract(frontmatter, '$.fileClass') = 'contact';
```

**TO:**
```sql
CREATE VIEW v_contact_notes AS
SELECT
    path,
    ...,
    modified_at,
    indexed_time,
    size
FROM notes
WHERE json_extract(frontmatter, '$.fileClass') = 'contact';
```

### **New Subtasks:**

Add feature for staleness queries:
```
### Feature X: Staleness Detection Queries

- [ ] Task X: Implement Staleness Query Methods (TDD Cycle)
  - [ ] Subtask X.1: Write tests for GetStaleNotes() query
  - [ ] Subtask X.2: Implement GetStaleNotes() returning paths where modified_at > indexed_time
  - [ ] Subtask X.3: Add IsStale(path) method for single-note checks
```

---

## Story 3.21: Unit of Work - UPDATES NEEDED

### **Method Signature Changes:**

**Update all UoW write methods to include IndexTime:**

**FROM:**
```go
func (uow *CacheUnitOfWork) AddWrite(note domain.Note) error
```

**TO:**
```go
func (uow *CacheUnitOfWork) AddWrite(note domain.Note, indexTime time.Time) error
```

### **New Subtasks:**

Add to implementation tasks:
```
- [ ] Subtask X.X: Update AddWrite() to accept indexTime parameter
- [ ] Subtask X.X: Pass indexTime to both BoltDB and SQLite writers
- [ ] Subtask X.X: Ensure indexTime is consistent across all storage systems (same timestamp for both)
```

### **Usage Example Update:**

**FROM:**
```go
uow.AddWrite(note)
```

**TO:**
```go
indexTime := time.Now()
uow.AddWrite(note, indexTime)
```

---

## Story 3.22: QueryService Hybrid Enhancement - UPDATES NEEDED

### **New Acceptance Criteria:**

**Add staleness detection:**
```
XX. ✅ Staleness detection on query:
    - Check cache staleness before returning results
    - Optionally trigger background reindex for stale entries
    - Log warnings for stale cache entries
```

### **New Subtasks:**

```
### Feature X: Cache Staleness Integration

- [ ] Task X: Integrate Staleness Checks in Query Flow (TDD Cycle)
  - [ ] Subtask X.1: Write tests for staleness detection in query path
  - [ ] Subtask X.2: Add cache staleness check before returning query results
  - [ ] Subtask X.3: Add logging for stale cache detections
  - [ ] Subtask X.4: (Optional) Trigger background reindex for stale entries
```

---

## Story 3.29: Event-Driven Architecture - UPDATES NEEDED

### **File Location Change:**

**FROM (current):**
```
internal/shared/events/bus.go
internal/shared/events/bus_test.go
```

**TO (corrected):**
```
internal/app/events/bus.go
internal/app/events/bus_test.go
```

**Rationale:** EventBus only handles domain/application events (not infrastructure events), belongs in application layer.

### **Search & Replace:**

1. Find all: `internal/shared/events/bus.go`
   Replace: `internal/app/events/bus.go`

2. Find all: `internal/shared/events/bus_test.go`
   Replace: `internal/app/events/bus_test.go`

3. Find: `Create /internal/shared/events/ directory`
   Replace: `Create /internal/app/events/ directory`

---

## Additional Global Changes

### **All Stories - Naming Consistency:**

**Search & Replace across ALL Epic 3 stories:**

1. `ModTime` → `ModifiedAt` (file modification time)
2. `IndexedAt` → `IndexTime` (when note was cached)
3. `mod_time` → `modified_at` (SQL column names)
4. `indexed_at` → `indexed_time` (SQL column names)

### **Test File Consolidation:**

**DO NOT CREATE these new test files:**
- ❌ `tests/integration/vault_scanning_test.go` → Extend existing `vault_indexing_test.go`
- ❌ `tests/integration/frontmatter_parsing_test.go` → Extend existing `frontmatter_workflow_test.go`
- ❌ `tests/integration/frontmatter_test.go` → Extend existing `frontmatter_workflow_test.go`
- ❌ `tests/integration/frontmatter_enrichment_test.go` → Extend existing `frontmatter_workflow_test.go`
- ❌ `tests/integration/note_enrichment_test.go` → Extend existing `vault_indexing_test.go`
- ❌ `tests/e2e/vault_indexing_test.go` → Extend existing `tests/e2e/lithos_index_test.go`

---

## Implementation Checklist

### **Story 3.17:** ✅ COMPLETE
- [x] Remove storage-specific DTOs
- [x] Add IndexTime concept
- [x] Update file paths
- [x] Update test locations

### **Story 3.18:** ⏳ PENDING
- [ ] Move markdown parser to vault package
- [ ] Update all file references
- [ ] Change test file to extend existing

### **Story 3.19:** ⏳ PENDING
- [ ] Add IndexTime to cached data structure
- [ ] Implement IsStale() method
- [ ] Update all ModTime references to ModifiedAt

### **Story 3.20:** ⏳ PENDING
- [ ] Add indexed_time column to schema
- [ ] Update all column names (mod_time → modified_at)
- [ ] Add staleness query methods
- [ ] Update all view definitions

### **Story 3.21:** ⏳ PENDING
- [ ] Add indexTime parameter to AddWrite()
- [ ] Update all usage examples
- [ ] Ensure consistent timestamp across storages

### **Story 3.22:** ⏳ PENDING
- [ ] Add staleness detection to query flow
- [ ] Add logging for stale cache entries

### **Story 3.29:** ⏳ PENDING
- [ ] Move EventBus from shared/ to app/
- [ ] Update all file path references

---

## Files Requiring Manual Updates

Due to complexity, the following story files need manual editing:

1. `docs/stories/3.18.markdown-parser-port.md` - Update file paths, test locations
2. `docs/stories/3.19.boltdb-hot-cache.md` - Add IndexTime, IsStale(), rename ModTime
3. `docs/stories/3.20.sqlite-deep-storage.md` - Update schema, add indexed_time column
4. `docs/stories/3.21.storage-write-coordination.md` - Add indexTime parameter
5. `docs/stories/3.22.queryservice-hybrid-enhancement.md` - Add staleness detection
6. `docs/stories/3.29.event-driven-architecture.md` - Update file paths

---

## Validation Steps

After updates:
1. Search all stories for `ModTime` - should only appear in context of "file's ModTime()"
2. Search all stories for `IndexedAt` - should be zero occurrences (renamed to IndexTime)
3. Search for `internal/shared/dto/file.go` - should be zero (moved to spi/dto/)
4. Search for `internal/shared/events/` - should be zero (moved to app/events/)
5. Search for `internal/adapters/spi/markdown/` - should be zero (moved to vault/)
6. Verify no new integration test files in duplication list

---

## Status: READY FOR MANUAL IMPLEMENTATION

**Next Step:** Manually edit stories 3.18, 3.19, 3.20, 3.21, 3.22, 3.29 following the specifications above.
