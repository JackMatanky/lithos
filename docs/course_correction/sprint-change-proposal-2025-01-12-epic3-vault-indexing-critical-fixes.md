# Sprint Change Proposal: Epic 3 Vault Indexing Critical Fixes

**Date:** 2025-01-12
**Status:** Approved
**Agent:** Sarah (Product Owner)
**Epic:** Epic 3 (Vault Indexing Engine)
**Priority:** Critical

---

## Executive Summary

Critical architectural flaws have been identified in the vault indexing implementation before completing Epic 3 stories 3.10 (DI and E2E test) and 3.11 (Documentation update). These issues fundamentally break core functionality including data integrity, system reliability, performance, and query capabilities. This proposal outlines a fix-forward approach with dedicated remediation stories to resolve all critical issues before Epic 3 completion.

**Impact:** Epic 3 completion delayed by 2-3 weeks to ensure robust foundation for all future vault-dependent features.

---

## Section 1: Change Context Analysis

### Triggering Story
- **Epic:** Epic 3 (Vault Indexing Engine)
- **Incomplete Stories:** 3.10 (Dependency Injection and E2E Test), 3.11 (Documentation Update)
- **Discovery Point:** Pre-completion code review revealed fundamental implementation flaws

### Issue Classification
**Type:** Technical limitation/architectural flaw
**Severity:** Critical - renders vault indexing engine unusable
**Root Cause:** Implementation deviates significantly from architecture specifications

### Core Problems Identified

#### 1. **Vault Index - Data Integrity Issues**

**Problem:** Note ID collision causing complete data loss
- `deriveNoteIDFromPath` strips directory segments and extension
- Two notes `projects/foo.md` and `ideas/foo.md` both create cache file `foo.json`
- Later cache write overwrites earlier one
- Query layer only sees one record
- Incremental refresh cannot reconcile deletions

**Impact:** Complete data loss for notes with same basename in different directories

#### 2. **Vault Index - Performance & Memory Issues**

**Problem:** Inefficient file scanning loads unnecessary content
- `ScanAll` uses `filepath.Walk` to read ALL file contents into memory
- Only after loading does it filter out non-Markdown files
- Large vaults with PDFs, images, binaries cause excessive RAM usage
- Unsafe filtering: `strings.Contains(path, ".lithos")` removes legitimate notes

**Impact:** Poor performance, memory exhaustion, incorrect file exclusion

#### 3. **Vault Index - Cache Management Failures**

**Problem:** Incomplete cache management and stale data
- Incremental refresh never removes deleted files from cache
- No `CacheWriterPort.Delete` calls for missing source files
- Schema loading missing in incremental refresh path
- Cache entries persist forever after note deletion

**Impact:** Stale data in query results, schema validation inconsistencies

#### 4. **Cache - Performance & Consistency Issues**

**Problem:** Inefficient serialization and code duplication
- `json.MarshalIndent` with 2-space indent doubles payload size
- Two `ensureCacheDir` implementations creating maintenance risk
- Missing directory handling causes `ENOENT` failures on fresh install

**Impact:** Slower cache operations, maintenance complexity, boot failures

#### 5. **Query - Complete Functionality Breakdown**

**Problem:** Query layer fundamentally broken
- `RefreshFromCache` only fills `byID`, `byFileClass`, `byFrontmatter` indices
- `ByPath` and `ByBasename` methods always return "not found"
- `domain.Note` doesn't store original path information
- Frontmatter index panics on non-comparable types (arrays, maps)
- Type mismatches cause lookup failures (int 2 vs float 2.0)

**Impact:** Path-based queries completely broken, system crashes, type-dependent failures

---

## Section 2: Epic Impact Assessment

### Current Epic Status (Epic 3)
- **Status:** BLOCKED - Cannot complete with fundamental flaws
- **Stories 3.10-3.11:** Cannot proceed until core issues resolved
- **Risk:** E2E tests would fail, documentation would be incorrect

### Future Epic Dependencies
- **Epic 4 (Schema-driven lookups):** BLOCKED - depends on working query layer
- **Epic 5 (Interactive input):** PARTIALLY AFFECTED - may depend on file lookups
- **Epic 6+ (Future features):** AFFECTED - All vault-dependent features impacted

### Technical Debt Assessment
**Severity:** Critical
**Type:** Architectural debt requiring immediate resolution
**Scope:** Core vault indexing pipeline, cache management, query layer

---

## Section 3: Artifact Impact Analysis

### PRD Conflicts
- **File:** `docs/prd/epic-3-vault-indexing-engine.md`
- **Issue:** Current implementation fails to meet basic vault indexing requirements
- **Required Updates:** Must acknowledge architectural fixes needed

### Architecture Document Conflicts

#### `docs/architecture/data-models.md`
- **NoteID model:** Doesn't support unique identification across vault
- **FileMetadata model:** Path-to-ID mapping unclear
- **VaultFile model:** Content loading strategy needs revision
- **Note model:** Missing path information for queries

#### `docs/architecture/components.md`
- **VaultIndexer:** Implementation doesn't match specification
- **QueryService:** Index population requirements missing
- **Cache components:** Management procedures incomplete

### Story Conflicts
- **Story 3.10:** E2E tests would fail with current implementation
- **Story 3.11:** Documentation would be inaccurate

---

## Section 4: Recommended Path Forward

### Option Analysis

#### Option 1: Direct Adjustment ❌
**Assessment:** Not viable - issues too fundamental
**Risk:** High - architectural changes during late implementation
**Effort:** Extremely high - requires rewriting core logic

#### Option 2: Rollback ❌
**Assessment:** Not practical - would lose significant Epic 3 progress
**Cost:** Too high - entire vault indexing infrastructure
**Timeline:** Worse than fix-forward approach

#### Option 3: Fix-Forward ✅ **RECOMMENDED**
**Assessment:** Most practical approach
**Strategy:** Dedicated remediation stories before Epic 3 completion
**Benefits:** Solid foundation for all future epics
**Timeline:** 2-3 week delay but ensures quality

---

## Section 5: Specific Proposed Changes

### A. Epic 3 Status Update

**Current Status:** "Near completion (stories 3.10-3.11 remaining)"
**New Status:** "Critical fixes in progress - completion delayed for quality assurance"

### B. New Remediation Stories (Sequential Implementation)

#### **Story 3.12: Fix Note ID Collision and Path Handling**
**Priority:** P0 - Data Integrity
**Scope:** Core identification and caching

**Changes Required:**
- Modify `deriveNoteIDFromPath` in `internal/app/vault/indexer.go:335`
- Change from basename-only to vault-relative path preservation
- Update cache key generation in `internal/adapters/spi/cache/json_writer.go:148`
- Ensure unique cache filenames (e.g., `projects-foo.json`, `ideas-foo.json`)
- Update domain.Note to include path information for queries

**Acceptance Criteria:**
1. Notes with same basename in different directories create unique cache entries
2. All notes remain accessible via query layer
3. Incremental refresh correctly maps cache entries to source files
4. No data loss during vault indexing

#### **Story 3.13: Optimize File Scanning and Memory Usage**
**Priority:** P0 - Performance & Reliability
**Scope:** Vault scanning and content loading

**Changes Required:**
- Modify `internal/adapters/spi/vault/reader.go:66-117` to filter before content loading
- Implement FileMetadata-first approach: scan → filter → load content only for .md files
- Replace `strings.Contains(path, ".lithos")` with proper path segment comparison
- Add mime-type detection for better file classification

**Acceptance Criteria:**
1. Only markdown files have content loaded into memory
2. Cache directory exclusion works correctly without false positives
3. Large vaults with media files scan efficiently
4. Memory usage remains predictable regardless of vault size

#### **Story 3.14: Implement Complete Cache Management**
**Priority:** P0 - Data Consistency
**Scope:** Cache lifecycle and incremental operations

**Changes Required:**
- Add deletion reconciliation to `internal/app/vault/indexer.go:172-204`
- Implement `CacheWriterPort.Delete` calls for missing source files
- Add schema loading to incremental refresh path
- Consolidate `ensureCacheDir` implementations

**Acceptance Criteria:**
1. Incremental refresh removes cache entries for deleted notes
2. Schema loading occurs in both Build and Refresh operations
3. Single, consistent `ensureCacheDir` implementation
4. Cache state accurately reflects vault state

#### **Story 3.15: Fix Query Layer Functionality**
**Priority:** P0 - Core Functionality
**Scope:** Query service and index management

**Changes Required:**
- Update `internal/app/query/service.go:138-164` to populate all indices
- Add path information to Note domain model
- Implement comparable key normalization for frontmatter queries
- Handle missing cache directory gracefully in `List` operations

**Acceptance Criteria:**
1. `ByPath` and `ByBasename` queries return correct results
2. Frontmatter queries handle all YAML value types without panicking
3. Fresh installation works without manual cache directory creation
4. Type-agnostic frontmatter lookups (int 2 matches float 2.0)

#### **Story 3.16: Cache Performance and Reliability**
**Priority:** P1 - Performance Optimization
**Scope:** Cache adapter efficiency

**Changes Required:**
- Replace `json.MarshalIndent` with `json.Marshal` in cache writer
- Remove duplicate `ensureCacheDir` helper function
- Add graceful handling for missing cache directory

**Acceptance Criteria:**
1. Cache writes are faster and more compact
2. Single, maintained cache directory management function
3. Query service handles missing cache directory on first boot

### C. Updated Story Sequence

**New Epic 3 Completion Order:**
1. Stories 3.1-3.9 ✅ (Completed)
2. **Story 3.10: Fix Note ID Collision and Path Handling** (New - Critical)
3. **Story 3.11: Optimize File Scanning and Memory Usage** (New - Critical)
4. **Story 3.12: Implement Complete Cache Management** (New - Critical)
5. **Story 3.13: Fix Query Layer Functionality** (New - Critical)
6. **Story 3.14: Optimize Cache Performance and Reliability** (New - Performance)
7. **Story 3.15: Comprehensive Integration Testing for Fixed Vault Indexing** (New - Validation)
8. **Story 3.16: Epic 3 Completion Quality Assurance** (New - QA)
9. Story 3.17: Dependency Injection and E2E Test (Delayed from 3.10)
10. Story 3.18: Documentation Update (Delayed from 3.11)

### D. Architecture Documentation Updates

#### `docs/architecture/data-models.md`

**Section: NoteID**
```markdown
- `value` (string) - Vault-relative path (e.g., "projects/foo.md").
  Preserves directory structure for unique identification across vault.
  Adapters translate between NoteID and filesystem absolute paths.
```

**Section: FileMetadata**
```markdown
- `CacheKey` (string, computed) - Deterministic cache filename derived from
  vault-relative path. Replaces directory separators with safe characters
  (e.g., "projects/foo.md" → "projects-foo.json").
```

**Section: Note**
```markdown
- `Path` (string) - Vault-relative path for query layer path-based lookups.
  Enables ByPath and ByBasename query functionality.
```

#### `docs/architecture/components.md`

**Section: VaultIndexer**
```markdown
Build Process:
1. ScanAll returns FileMetadata only (no content loading)
2. Filter for .md files based on extension
3. Load content only for markdown files
4. Extract frontmatter and validate
5. Create Note with vault-relative path as ID
6. Persist to cache with path-based cache key

Refresh Process:
1. Load current schemas (same as Build)
2. ScanModified for changed files
3. Process changed files as in Build
4. Identify and delete cache entries for missing source files
```

**Section: QueryService**
```markdown
Index Population:
- byID: NoteID (vault-relative path) → Note
- byPath: Full vault-relative path → Note
- byBasename: Filename without extension → Note
- byFileClass: Schema name → []Note
- byFrontmatter: Normalized field values → []Note

RefreshFromCache must populate ALL indices from cache data.
```

---

## Section 6: Implementation Plan

### Phase 1: Critical Fixes (Stories 3.12-3.15)
**Timeline:** 2-3 weeks
**Goal:** Resolve all critical functionality issues
**Success Criteria:** Vault indexing works as architecturally specified

### Phase 2: Performance Optimization (Story 3.16)
**Timeline:** 1 week
**Goal:** Optimize cache performance
**Success Criteria:** Efficient cache operations

### Phase 3: Integration & Documentation (Stories 3.10-3.11)
**Timeline:** 1 week
**Goal:** Complete Epic 3 as originally planned
**Success Criteria:** E2E tests pass, documentation accurate

### Testing Strategy
- **Unit Tests:** Each fix story includes comprehensive unit test updates
- **Integration Tests:** Validate component interactions after each fix
- **E2E Tests:** Comprehensive end-to-end validation in Story 3.10
- **Performance Tests:** Memory usage and scanning efficiency validation

---

## Section 7: Risk Assessment & Mitigation

### Implementation Risks

#### Risk: Further architectural issues discovered during fixes
**Probability:** Medium
**Impact:** High
**Mitigation:** Incremental approach allows early detection and course correction

#### Risk: Timeline pressure to skip proper fixes
**Probability:** Low
**Impact:** Very High
**Mitigation:** Explicit approval for timeline extension, quality-first approach

#### Risk: Regression in existing functionality
**Probability:** Low
**Impact:** Medium
**Mitigation:** Comprehensive test coverage, incremental implementation

### Quality Assurance
- **Code Review:** All fix stories require thorough code review
- **Architecture Review:** Ensure fixes align with overall system design
- **User Acceptance:** Validate fixes meet Epic 3 original goals

---

## Section 8: Communication Plan

### Stakeholder Updates
- **Development Team:** Daily standup updates on fix progress
- **Project Timeline:** Adjust Epic 4+ start dates accordingly
- **Quality Gates:** Each fix story must pass QA review before proceeding

### Success Metrics
- **Data Integrity:** Zero note collisions or data loss
- **Performance:** Vault scanning completes within acceptable time limits
- **Functionality:** All query methods return correct results
- **Reliability:** System handles fresh installs and edge cases gracefully

---

## Section 9: Lessons Learned & Prevention

### Root Cause Analysis
1. **Implementation Divergence:** Code developed without strict adherence to architecture
2. **Testing Gaps:** Edge cases (same basename, large vaults) not covered in unit tests
3. **Review Process:** Late discovery indicates need for architecture compliance checks

### Prevention Measures
1. **Architecture Compliance:** Regular architecture-to-implementation validation
2. **Comprehensive Testing:** Include edge cases in story acceptance criteria
3. **Early Integration:** Don't wait until E2E tests to validate core functionality
4. **Progressive Quality Gates:** QA review after each component, not just at epic end

---

## Section 10: Approval & Next Steps

### Sprint Change Proposal Status
**Status:** ✅ **APPROVED**
**Approved By:** User
**Approval Date:** 2025-01-12

### Immediate Next Steps
1. **Create Detailed Stories:** Generate stories 3.12-3.16 with full technical specifications
2. **Update Epic 3 Timeline:** Revise completion date and communicate impacts
3. **Begin Story 3.12:** Start with most critical data integrity fixes
4. **Update Architecture Docs:** Revise documentation to reflect correct implementation

### Agent Handoffs
1. **Dev Agent:** Implement stories 3.12-3.16 in sequence
2. **QA Agent:** Review each fix story for architecture compliance
3. **Architect Agent:** Validate architecture document updates
4. **PM Agent:** Communicate timeline impacts and adjust project schedule

---

## Conclusion

This Sprint Change Proposal ensures Epic 3 delivers a robust, working vault indexing engine rather than proceeding with known critical issues. The fix-forward approach addresses all identified problems systematically while preserving the significant progress already made. The 2-3 week delay is justified by the critical nature of these issues and their impact on all future vault-dependent features.

The comprehensive fix plan provides a solid foundation for Epic 4 and beyond, preventing technical debt accumulation and ensuring long-term project success.

---

---

## Summary of Completed Sprint Story Creation

### ✅ **Stories Created for Epic 3 Critical Fixes Sprint**

| Story | Title | Priority | Status |
|-------|-------|----------|---------|
| 3.10 | Fix Note ID Collision and Path Handling | P0 - Critical | ✅ Created |
| 3.11 | Optimize File Scanning and Memory Usage | P0 - Critical | ✅ Created |
| 3.12 | Implement Complete Cache Management | P0 - Critical | ✅ Created |
| 3.13 | Fix Query Layer Functionality | P0 - Critical | ✅ Created |
| 3.14 | Optimize Cache Performance and Reliability | P1 - Performance | ✅ Created |
| 3.15 | Comprehensive Integration Testing | P1 - Validation | ✅ Created |
| 3.16 | Epic 3 Completion Quality Assurance | P1 - QA | ✅ Created |
| 3.17 | Dependency Injection and E2E Test | P2 - Integration | ✅ Renumbered |
| 3.18 | Documentation Update | P2 - Documentation | ✅ Renumbered |

### **Sprint Scope Summary**

**Total Stories:** 9 (7 new critical fixes + 2 renumbered completion stories)
**Estimated Timeline:** 3-4 weeks for stories 3.10-3.16, then 1 week for 3.17-3.18
**Primary Focus:** Resolve all critical vault indexing issues before Epic 3 completion

### **Next Steps**

1. **Dev Agent:** Begin implementation with Story 3.10 (Note ID collision fixes)
2. **QA Agent:** Review each story for architecture compliance as completed
3. **PM Agent:** Update Epic 3 timeline and communicate to stakeholders
4. **Architect Agent:** Review architecture document updates as needed

### **Success Criteria**

Epic 3 will be considered successfully completed when:
- ✅ All critical vault indexing issues resolved (Stories 3.10-3.14)
- ✅ Comprehensive testing validates all fixes (Stories 3.15-3.16)
- ✅ Integration and documentation completed (Stories 3.17-3.18)
- ✅ Vault indexing engine is robust and ready for Epic 4 dependencies

---

**Document Version:** 1.1
**Last Updated:** 2025-01-12
**Sprint Stories Created:** 2025-01-12
**Related Documents:**
- `docs/prd/epic-3-vault-indexing-engine.md`
- `docs/architecture/data-models.md`
- `docs/architecture/components.md`
- `docs/stories/3.17.dependency-injection-and-e2e-test.md` (renumbered)
- `docs/stories/3.18.documentation-update.md` (renumbered)
- **New Stories:** `docs/stories/3.10.*.md` through `docs/stories/3.16.*.md`
