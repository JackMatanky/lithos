# Epic 6: Technical Debt Refactoring - Event-Driven Decomposition

## Epic Overview

**Epic Goal:** Refactor god-objects into focused, event-driven components following SOLID principles, improving maintainability, testability, and extensibility.

**Business Value:** Reduce technical debt, improve code quality, enable faster feature development, reduce bugs, improve onboarding for new developers.

**Target Completion:** TBD (Based on team capacity)

---

## Problem Statement

The lithos codebase has accumulated technical debt in the form of god-objects - large, monolithic classes that violate the Single Responsibility Principle:

| Service | Lines | Methods | Issues |
|---------|-------|---------|--------|
| VaultIndexer | 938 | 22 | Does everything: scanning, parsing, validation, caching, events |
| FrontmatterService | 806 | ~15 | Mixed schema validation, FileSpec validation, events, hints |
| TemplateEngine | 584 | ~10 | Template rendering mixed with query functions |
| QueryService | 567 | ~12 | Hybrid routing, events, observability all mixed |
| CommandOrchestrator | 408 | ~8 | CLI orchestration mixed with business logic |

**Consequences:**
- ❌ Hard to test (require complex mocking)
- ❌ Hard to understand (too many responsibilities)
- ❌ Hard to extend (tight coupling)
- ❌ Hard to maintain (changes affect multiple concerns)
- ❌ Hard to onboard (steep learning curve)

---

## Solution Approach

**Event-Driven Decomposition Pattern:**

Instead of monolithic services, decompose into:

1. **Core Coordinator** (100-150 lines)
   - Lightweight orchestrator
   - Dependency injection only
   - No business logic

2. **Focused Components** (50-200 lines each)
   - Single responsibility
   - Independently testable
   - Event-driven communication

3. **Event Publishers** (50 lines)
   - Consolidate event logic
   - No business logic

4. **Helper Services** (50-100 lines)
   - Support single concerns
   - Reusable utilities

**Benefits:**
- ✅ Easy to test (mock single component)
- ✅ Easy to understand (clear responsibilities)
- ✅ Easy to extend (add new components)
- ✅ Easy to maintain (changes isolated)
- ✅ Easy to onboard (small focused files)

---

## Epic Stories

### Story 6.1: VaultIndexer Decomposition (CRITICAL PRIORITY)

**Lines:** 938 → ~500 (5 files)
**Goal:** Break VaultIndexer into FileScannerService, NoteProcessingPipeline, IndexingEventPublisher, IndexStatsCollector

**Impact:** Highest - VaultIndexer is core to entire system

**Acceptance Criteria:** 52
**Estimated Effort:** 5-8 days

**Architecture:**
```
VaultIndexer (100 lines) - Coordinator
├── FileScannerService (150 lines)
├── NoteProcessingPipeline (200 lines)
├── IndexingEventPublisher (50 lines)
└── IndexStatsCollector (50 lines)
```

**Key Events:**
- FileDiscoveredEvent
- NoteIndexedEvent
- NoteIndexingFailedEvent

---

### Story 6.2: FrontmatterService Decomposition (HIGH PRIORITY)

**Lines:** 806 → ~500 (6 files)
**Goal:** Break FrontmatterService into SchemaValidator, FileSpecValidator, WikilinkResolver, ValidationEventPublisher, RemediationEngine

**Impact:** High - Validation is used throughout system

**Acceptance Criteria:** 52
**Estimated Effort:** 4-6 days

**Architecture:**
```
FrontmatterService (150 lines) - Orchestrator
├── SchemaValidator (150 lines)
├── FileSpecValidator (100 lines)
│   └── WikilinkResolver (50 lines)
├── ValidationEventPublisher (50 lines)
└── RemediationEngine (50 lines)
```

**Key Separation:**
- Schema validation logic → SchemaValidator
- FileSpec validation → FileSpecValidator
- Event publishing → ValidationEventPublisher

---

### Story 6.3: TemplateEngine Function Provider Pattern (MEDIUM PRIORITY)

**Lines:** 584 → ~440 (6 files)
**Goal:** Extract template functions into provider pattern

**Impact:** Medium - Improves extensibility for new template functions

**Acceptance Criteria:** 40
**Estimated Effort:** 3-4 days

**Architecture:**
```
TemplateEngine (200 lines) - Core
├── TemplateFunctionRegistry (100 lines)
├── LookupFunctionProvider (80 lines)
├── QueryFunctionProvider (80 lines)
├── FileClassFunctionProvider (80 lines)
└── TemplateCache (50 lines)
```

**Benefit:** New template functions become pluggable extensions

---

### Story 6.4: QueryService Observability Extraction (MEDIUM PRIORITY)

**Lines:** 567 → ~350 (4 files)
**Goal:** Separate query logic from observability concerns

**Impact:** Medium - Improves query service maintainability

**Acceptance Criteria:** 35
**Estimated Effort:** 2-3 days

**Architecture:**
```
QueryService (150 lines) - Core
├── HybridStorageRouter (already extracted ✅)
├── QueryEventSubscriber (50 lines)
└── BackendHealthMonitor (100 lines)
    ├── BackendFailureTracker (already extracted ✅)
    └── StalenessObserver (already extracted ✅)
```

**Note:** Already partially refactored - finish extraction

---

### Story 6.5: CommandOrchestrator Workflow Pattern (LOW PRIORITY)

**Lines:** 408 → ~300 (4 files)
**Goal:** Extract use cases into explicit workflow objects

**Impact:** Low - Improves CLI command extensibility

**Acceptance Criteria:** 30
**Estimated Effort:** 2-3 days

**Architecture:**
```
CLIComander (100 lines) - Coordinator
├── NoteCreationWorkflow (100 lines)
│   └── NoteEventPublisher (50 lines)
└── VaultIndexingWorkflow (50 lines)
```

**Benefit:** New CLI commands become explicit workflows

---

### Story 6.6: Epic 6 Retrospective (REQUIRED)

**Goal:** Capture lessons learned from refactoring approach

**Acceptance Criteria:**
- Document refactoring patterns used
- Identify reusable patterns for future epics
- Measure improvement metrics (test coverage, complexity, etc.)
- Team reflection on event-driven decomposition

---

## Success Metrics

### Before Refactoring

| Metric | Current |
|--------|---------|
| Largest file | 938 lines |
| Avg file size (top 5) | 621 lines |
| Test coverage (vault) | ~75% |
| Cyclomatic complexity (max) | 25+ |
| Time to onboard new dev | 2+ weeks |

### After Refactoring (Target)

| Metric | Target |
|--------|--------|
| Largest file | <250 lines |
| Avg file size | <150 lines |
| Test coverage (all) | >85% |
| Cyclomatic complexity (max) | <15 |
| Time to onboard new dev | <1 week |

### Code Quality Improvements

- **Maintainability Index:** Increase from 60 → 80+
- **Technical Debt Ratio:** Decrease from 15% → <5%
- **Bug Rate:** Decrease by 30% (fewer unintended side effects)
- **Feature Velocity:** Increase by 20% (easier to add features)

---

## Implementation Order

**Phase 1: Critical Path (Stories 6.1-6.2)**
1. Story 6.1: VaultIndexer Decomposition
2. Story 6.2: FrontmatterService Decomposition

**Phase 2: Medium Priority (Stories 6.3-6.4)**
3. Story 6.3: TemplateEngine Function Providers
4. Story 6.4: QueryService Observability Extraction

**Phase 3: Nice to Have (Story 6.5)**
5. Story 6.5: CommandOrchestrator Workflow Pattern

**Phase 4: Reflection (Story 6.6)**
6. Story 6.6: Epic 6 Retrospective

---

## Risks & Mitigation

### Risk 1: Breaking Existing Functionality

**Probability:** Medium
**Impact:** High
**Mitigation:**
- Comprehensive integration tests before refactoring
- Incremental migration (keep old code until new works)
- All existing tests must pass before deletion

### Risk 2: Increased File Count

**Probability:** High
**Impact:** Low
**Mitigation:**
- Clear directory structure (group related files)
- Strong naming conventions
- Comprehensive package documentation

### Risk 3: Over-Engineering

**Probability:** Medium
**Impact:** Medium
**Mitigation:**
- Follow YAGNI (You Aren't Gonna Need It)
- Extract only when complexity warrants it
- Measure actual improvements (not just theoretical)

---

## Dependencies

**Prerequisites:**
- Epic 3 (Vault Indexing Engine) - DONE ✅
- Epic 4 (Event-Driven Architecture) - DONE ✅

**Enables:**
- Faster feature development in future epics
- Easier debugging and troubleshooting
- Better test coverage
- Improved code reviews

---

## Definition of Done

Epic 6 is complete when:

- ✅ All 6 stories completed (6.1-6.6)
- ✅ All existing integration tests pass
- ✅ Test coverage >85% across refactored components
- ✅ No file exceeds 250 lines
- ✅ Cyclomatic complexity <15 for all functions
- ✅ All linter warnings resolved
- ✅ Documentation updated (architecture diagrams, component docs)
- ✅ Retrospective completed with lessons learned

---

## Related Documents

- [Story 6.1: VaultIndexer Decomposition](../stories/6.1.vault-indexer-decomposition.md)
- [Story 6.2: FrontmatterService Decomposition](../stories/6.2.frontmatter-service-decomposition.md)
- [Architecture Components](../architecture/components.md)
- [Event-Driven Architecture](../architecture/event-driven-architecture.md)

---

## Change Log

| Date       | Version | Description                    | Author    |
| ---------- | ------- | ------------------------------ | --------- |
| 2025-12-28 | 1.0     | Epic created for refactoring   | Dev Agent |
