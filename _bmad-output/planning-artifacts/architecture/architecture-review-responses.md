# Architectural Review - File Path Corrections

## Issue Summary

After comprehensive review, we've identified 8 architectural concerns requiring resolution before Epic 3 implementation.

---

## 1️⃣ **storage_metadata.go - LOCATION ERROR**

### **What It Is:**
- Contains `BoltDBMetadata` and `SQLiteMetadata` structs
- These are **Transport DTOs** (not domain entities)
- Purpose: Storage-specific data structures for BoltDB (hot cache) and SQLite (deep storage)

### **Current (WRONG) Location:**
```
❌ internal/domain/storage_metadata.go
```

### **Why This Is Wrong:**
From data-models.md:
> **Transport DTOs (Adapter)** | Data transfer between layers, infrastructure concerns | VaultFile, BoltDBMetadata, SQLiteMetadata | `internal/adapters/spi/*/dto.go`

These are **infrastructure DTOs**, not domain models. Domain should have ZERO infrastructure dependencies.

### **Correct Location (DECISION REQUIRED):**

**Option A: Single cache DTO file**
```
✅ internal/adapters/spi/cache/dto.go
   - BoltDBMetadata
   - SQLiteMetadata
   - Conversion functions
```

**Option B: Split by storage solution (cleaner, recommended)**
```
✅ internal/adapters/spi/cache/boltdb/metadata.go
   - BoltDBMetadata
   - ToBoltDBMetadata()

✅ internal/adapters/spi/cache/sqlite/metadata.go
   - SQLiteMetadata
   - ToSQLiteMetadata()
```

**RECOMMENDATION: Option B** (aligns with Question 4 answer)

---

## 2️⃣ **Events - Should They Be in Subfolder?**

### **Current Architecture Decision:**
From data-models.md:
> **Domain Events** | Significant domain occurrences for pub/sub | NoteIndexed, VaultIndexingComplete, FrontmatterValidated | `internal/domain/events.go`

All events in **ONE file**: `internal/domain/events.go`

### **Analysis:**

**Arguments FOR Subfolder** (`internal/domain/events/`):
- ✅ Separates concerns (entities vs events)
- ✅ Easier to find event definitions
- ✅ Scales better if many events added
- ✅ Clear package boundary

**Arguments AGAINST Subfolder:**
- ❌ Only 5-6 event types for MVP
- ❌ Adds one more package to imports
- ❌ Architecture doc says single file

### **RECOMMENDATION:**

**Start with single file, refactor if grows:**
```
✅ internal/domain/events.go  (MVP - 5-6 events)
```

**Future refactor trigger:** If events.go exceeds 500 lines, split into:
```
internal/domain/events/
  ├── domain_event.go (interface)
  ├── indexing.go (NoteIndexed, VaultIndexingComplete)
  ├── validation.go (FrontmatterValidated)
  └── schema.go (SchemaLoaded, SchemasReloaded)
```

---

## 3️⃣ **markdown_parser.go - Vault-Specific or General?**

### **User Suggestion:**
```
internal/adapters/spi/vault/markdown.go
```

### **Current Story 3.18:**
```
internal/adapters/spi/markdown/markdown_parser.go
```

### **Analysis:**

**Is Markdown Parsing Vault-Specific?**
- ❌ **NO** - Markdown parsing is a general capability
- Frontmatter extraction could be used for:
  - Template files (also markdown with frontmatter)
  - Documentation parsing
  - Future markdown-based features

**Vault adapter uses markdown parsing, but parsing itself is not vault-specific.**

### **DECISION:**

Keep separate markdown adapter package:
```
✅ internal/adapters/spi/markdown/
   ├── markdown_parser.go  (MarkdownParserAdapter)
   └── markdown_parser_test.go
```

**Vault adapter depends on it:**
```go
// internal/adapters/spi/vault/reader.go
import "internal/adapters/spi/markdown"

func (r *VaultReaderAdapter) Read(ctx context.Context, path string) (domain.Note, error) {
    content, err := r.readFile(path)
    // ...
    note, err := r.markdownParser.ParseNote(ctx, path, content)
    return note, err
}
```

**Why?** - Separation of concerns: vault reads files, markdown parses content. Each adapter has single responsibility.

---

## 4️⃣ **Cache Folder Structure - Should Split into Subfolders?**

### **Current Structure (Flat):**
```
internal/adapters/spi/cache/
├── boltdb_reader.go
├── boltdb_writer.go
├── boltdb_reader_test.go
├── boltdb_writer_test.go
├── sqlite_reader.go
├── sqlite_writer.go
├── sqlite_reader_test.go
├── sqlite_writer_test.go
├── sqlite_views.go (to be added)
├── sqlite_views_test.go (to be added)
├── boltdb_common.go (to be added)
├── json_reader.go
├── json_writer.go
├── json_reader_test.go
├── json_writer_test.go
├── helper.go
├── helper_test.go
└── constants.go

Total: 17+ files (getting unwieldy)
```

### **RECOMMENDATION: Split into Subfolders**

```
internal/adapters/spi/cache/
├── boltdb/
│   ├── reader.go
│   ├── reader_test.go
│   ├── writer.go
│   ├── writer_test.go
│   ├── common.go
│   ├── common_test.go
│   └── metadata.go (BoltDBMetadata DTO - from Question 1)
├── sqlite/
│   ├── reader.go
│   ├── reader_test.go
│   ├── writer.go
│   ├── writer_test.go
│   ├── views.go
│   ├── views_test.go
│   ├── common.go
│   └── metadata.go (SQLiteMetadata DTO - from Question 1)
├── json/
│   ├── reader.go
│   ├── reader_test.go
│   ├── writer.go
│   └── writer_test.go
└── shared/
    ├── helper.go
    ├── helper_test.go
    └── constants.go
```

**Benefits:**
- ✅ Clear organization by storage technology
- ✅ Each subfolder is self-contained unit
- ✅ Easier to navigate and maintain
- ✅ Aligns with DDD bounded context pattern
- ✅ Future additions (Redis, etc.) fit naturally

---

## 5️⃣ **go_template.go - Naming & Location**

### **Current Story 4.1:**
```
internal/adapters/spi/template/go_template.go
```

### **Existing File:**
```
internal/adapters/spi/template/loader.go  (loads template FILES)
```

### **Clarification:**
- `loader.go` = Loads template **files** from filesystem
- `go_template.go` = Wraps `*template.Template` for **execution**
- These are **different responsibilities**

### **RECOMMENDATION: Keep Separate, Rename for Clarity**

```
internal/adapters/spi/template/
├── loader.go           (Existing - filesystem loading)
├── loader_test.go      (Existing)
├── executor.go         (NEW - GoTemplate wrapper for execution)
└── executor_test.go    (NEW)
```

**Rationale:**
- `loader.go` = **Discovery** (finds and reads template files)
- `executor.go` = **Execution** (wraps *template.Template, handles rendering)
- Clear separation of concerns
- More expressive than "go_template.go"

**Alternative Names:**
- `engine.go` (but might confuse with TemplateEngine domain service)
- `renderer.go` (clear but less precise)
- `wrapper.go` (too generic)
- ✅ `executor.go` (best - describes what it does)

---

## 6️⃣ **unit_of_work.go - Port or Service?**

### **Current Location:**
```
internal/app/vault/unit_of_work.go
```

### **Analysis:**

**Is it a Port (Interface)?**
- ❌ NO - It's a **concrete implementation**

**Is it a Service?**
- ✅ YES - It's an **application-layer pattern implementation**

**What is Unit of Work?**
- **Design Pattern**: Coordinates transactional writes across multiple storage systems
- **Layer**: Application (orchestrates domain + infrastructure)
- **Responsibility**: Write coordination, not business logic or infrastructure

### **CORRECT Classification:**

```
✅ Application Service (Pattern Implementation)
   Location: internal/app/vault/unit_of_work.go
```

**Why `internal/app/vault/`?**
- UoW coordinates **vault-specific** cache writes (BoltDB + SQLite)
- Used by VaultIndexer (same package)
- Not a reusable pattern across all domains (vault-specific)

**Not a Port because:**
- Ports are **interfaces** (contracts)
- UoW is a **concrete implementation** (pattern)
- UoW uses ports (CacheWriterPort), but isn't one itself

---

## 7️⃣ **Event Bus - Why Shared Package?**

### **Current Story 3.29:**
```
internal/shared/events/bus.go
```

### **Why Not in a Specific Layer?**

**Could it be in domain?**
- ❌ NO - EventBus is **infrastructure**, not business logic
- Domain defines events (DomainEvent interface)
- Domain doesn't know HOW events are delivered

**Could it be in adapters?**
- ❌ NO - EventBus serves **multiple layers**:
  - Application services publish events
  - Domain services could subscribe (if needed)
  - Adapters could subscribe for side effects

**Why shared?**
- ✅ **Cross-cutting concern** - used by multiple layers
- ✅ **Infrastructure service** - not business logic, not external interface
- ✅ **Internal implementation detail** - hidden behind usage

### **Analogy:**
```
Logger        → internal/shared/logger/      (infrastructure)
EventBus      → internal/shared/events/      (infrastructure)
Error Package → internal/shared/errors/      (infrastructure)
Registry      → internal/shared/registry/    (infrastructure)
```

All are **internal infrastructure** serving multiple layers.

### **DECISION: Correct as-is**
```
✅ internal/shared/events/bus.go
```

**Alternative considered:**
- `internal/infrastructure/events/bus.go` (more explicit but less consistent with existing patterns)

---

## 8️⃣ **Integration Tests - Avoiding Duplication**

### **Existing Integration Tests:**
```
tests/integration/
├── schema_lookup_test.go
├── template_loader_test.go
├── index_command_test.go
├── config_loading_test.go
├── template_engine_test.go
├── duplicate_basename_test.go
├── vault_indexing_test.go
├── frontmatter_workflow_test.go
└── schema_system_test.go
```

### **Epic 3 Stories Want to Create:**
```
Story 3.17: tests/integration/vault_scanning_test.go
Story 3.18: tests/integration/frontmatter_parsing_test.go
Story 3.19: tests/integration/boltdb_cache_test.go
Story 3.20: tests/integration/sqlite_cache_test.go
Story 3.21: tests/integration/unit_of_work_test.go
Story 3.22: tests/integration/hybrid_query_test.go
Story 3.23: tests/integration/frontmatter_test.go
Story 3.24: tests/integration/frontmatter_enrichment_test.go
Story 3.25: tests/integration/note_enrichment_test.go
Story 3.29: tests/integration/event_flow_test.go
Story 3.30: tests/e2e/vault_indexing_test.go (E2E, not integration)
```

### **Duplication Analysis:**

#### **CONFLICT 1: Vault Indexing Tests**
**Existing:** `vault_indexing_test.go`
**Story 3.17 wants:** `vault_scanning_test.go`

**DECISION:**
- ✅ **EXTEND existing** `vault_indexing_test.go` with new scanning test cases
- ❌ **DO NOT CREATE** new `vault_scanning_test.go`

#### **CONFLICT 2: Frontmatter Tests**
**Existing:** `frontmatter_workflow_test.go`
**Stories 3.18, 3.23, 3.24 want:** `frontmatter_parsing_test.go`, `frontmatter_test.go`, `frontmatter_enrichment_test.go`

**DECISION:**
- ✅ **EXTEND existing** `frontmatter_workflow_test.go`
- ❌ **DO NOT CREATE** 3 separate frontmatter test files

#### **NEW TESTS (No Conflicts):**
- ✅ `boltdb_cache_test.go` (NEW - no conflict)
- ✅ `sqlite_cache_test.go` (NEW - no conflict)
- ✅ `unit_of_work_test.go` (NEW - no conflict)
- ✅ `hybrid_query_test.go` (NEW - no conflict)
- ✅ `event_flow_test.go` (NEW - no conflict)

#### **E2E vs Integration:**
**Story 3.30:** `tests/e2e/vault_indexing_test.go`
**Existing:** `tests/e2e/lithos_index_test.go`

**DECISION:**
- ✅ **EXTEND existing** `lithos_index_test.go` with hybrid storage test cases
- ❌ **DO NOT CREATE** new E2E test file

### **UPDATED INTEGRATION TEST PLAN:**

```
tests/integration/
├── vault_indexing_test.go         (EXTEND - add scanning tests)
├── frontmatter_workflow_test.go   (EXTEND - add parsing, enrichment tests)
├── boltdb_cache_test.go           (NEW)
├── sqlite_cache_test.go           (NEW)
├── unit_of_work_test.go           (NEW)
├── hybrid_query_test.go           (NEW)
├── event_flow_test.go             (NEW)
└── (keep all existing tests)

tests/e2e/
├── lithos_index_test.go           (EXTEND - add hybrid storage E2E)
└── (keep all existing tests)
```

---

## Summary of Required Changes

### **Stories Requiring Updates:**

1. **Story 3.17** - Move storage_metadata from domain to cache adapters
2. **Story 3.18** - Keep markdown adapter separate (not in vault/)
3. **Story 3.19** - Restructure to `cache/boltdb/` subfolder
4. **Story 3.20** - Restructure to `cache/sqlite/` subfolder
5. **Story 3.21** - Clarify as application service (correct as-is)
6. **Story 3.23** - Extend existing frontmatter_workflow_test.go
7. **Story 3.24** - Extend existing frontmatter_workflow_test.go
8. **Story 3.29** - Event bus location correct (shared infrastructure)
9. **Story 4.1** - Rename go_template.go to executor.go

### **Test File Consolidation:**
- Reduce 11 planned test files to 5 new + 2 extended
- Prevents test suite bloat
- Improves maintainability

---

## Architectural Principles Validated

✅ **Hexagonal Architecture** - Clear layer boundaries maintained
✅ **Single Responsibility** - Each package has one clear purpose
✅ **Separation of Concerns** - Infrastructure separated from business logic
✅ **DDD Bounded Contexts** - Storage solutions are separate contexts
✅ **Test Organization** - Tests co-located with code, integration tests consolidated
