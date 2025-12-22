# System-Level Test Design: Lithos CLI

**Date:** 2025-12-22
**Author:** Jack (TEA Agent)
**Project:** lithos
**Phase:** Solutioning (Phase 2)
**Status:** Draft

---

## Executive Summary

**Scope:** System-level testability review for greenfield Go CLI project

**Mode:** System-Level Testability Review (Phase 3 - Pre-Implementation)

**Purpose:** Evaluate architecture for testability before implementation-readiness gate check

---

## Testability Assessment

### Controllability

**✅ PASS - Strong Controllability**

**Evidence:**
- ✅ Hexagonal Architecture with port interfaces enables mockable dependencies
- ✅ Constructor-based dependency injection provides test seams
- ✅ CQRS pattern allows controlled state manipulation
- ✅ Event-driven architecture enables controlled event triggering
- ✅ Comprehensive mock implementations available
- ✅ Singleton pattern includes test helpers (`SetInstanceForTesting()`, `ResetForTesting()`)

**Test Support:**
- `testing/fstest.MapFS` for in-memory filesystem testing
- Temporary directories via `t.TempDir()` for isolated test execution
- Mock adapters with configurable error injection
- Event-driven testing via mock EventBus

---

### Observability

**✅ PASS - Strong Observability**

**Evidence:**
- ✅ Structured logging with zerolog (JSON output + context-aware sub-loggers)
- ✅ Domain events provide state change visibility
- ✅ Idiomatic Go error handling with proper wrapping
- ✅ Test coverage tooling generates HTML reports
- ✅ Validation errors include field-level remediation hints
- ✅ Performance benchmarks track rendering performance

**Test Support:**
- Golden file comparisons for output validation
- Event subscribers for validation state tracking
- Structured error assertions via `errors.Is()` and `errors.As()`
- Coverage targets: ≥85% for `internal/app`, ≥80% overall

---

### Reliability

**✅ PASS with minor CONCERNS**

**Evidence:**
- ✅ Test isolation via `t.TempDir()` and `t.Cleanup()`
- ✅ Atomic file writes prevent partial writes
- ✅ CQRS separation (QueryService strictly read-only)
- ✅ Unit of Work Pattern coordinates transactional writes
- ✅ Test pyramid: Unit (70%), Integration (25%), E2E (5%)

**Minor Concerns:**
- ⚠️ Interactive library (`promptui`) unmaintained but architecture protects via InteractivePort abstraction
- ⚠️ Need production vault performance validation

**Mitigations:**
- Migration path to `charmbracelet/huh` documented for Post-MVP
- Production test data available (70MB+ vault, 500+ notes)

---

## Architecturally Significant Requirements (ASRs)

### NFR3: Performance - Template Rendering < 300ms

**Risk Score:** 2 × 2 = **4 (Medium)**

**Test Approach:**
- Benchmark tests track template rendering performance
- Performance targets: <300ms rendering, <1ms BoltDB, <50ms SQLite, <5s indexing
- Production vault available for realistic testing

**Mitigation:** Benchmark infrastructure in place, production data available

---

### NFR4: Hybrid Indexing (BoltDB + SQLite)

**Risk Score:** 2 × 3 = **6 (High - Requires Mitigation)**

**Test Approach:**
- Unit of Work Pattern coordinates two-phase commits
- Integration tests validate atomic behavior
- Need chaos testing for crash scenarios

**Mitigation:** ✅ CacheUnitOfWork implemented with rollback support

---

## Test Levels Strategy

**Unit Tests: 70%**
- Domain services, models, validation logic, events

**Integration Tests: 25%**
- CLI commands, template pipeline, cache coordination, event flows

**E2E Tests: 5%**
- Smoke tests for critical commands, golden file comparisons

---

## NFR Testing Approach

### Security (SEC)
- Schema validation, CLI flag validation, dependency scanning
- Tools: `gitleaks`, `golangci-lint`
- **Risk:** Low (local-only CLI)

### Performance (PERF)
- Benchmark tests with production data
- Targets: <300ms rendering, <1ms BoltDB, <50ms SQLite, <5s indexing
- **Risk:** Medium (requires validation)

### Reliability (REL)
- Atomic writes, Unit of Work, transaction coordination
- **Risk:** High (hybrid storage) - Mitigated

### Maintainability (MAIN)
- Coverage: ≥85% `internal/app`, ≥80% overall
- Tools: `golangci-lint`, coverage reports

---

## Test Environment Requirements

### MVP (Local Development)
- Go 1.23+, macOS primary
- Test data: `testdata/` fixtures, `testdata/vault-large/` (500+ notes)
- Tooling: `mise` task runner, `golangci-lint`, `gitleaks`

### Post-MVP (CI/CD)
- GitHub Actions, cross-platform runners
- Artifact storage for coverage reports

---

## Testability Concerns

### ⚠️ Concern 1: Unmaintained Interactive Library

**Issue:** `promptui` last updated Oct 2021

**Mitigation:** ✅ InteractivePort abstraction enables migration to `charmbracelet/huh`

**Recommendation:** ACCEPT for MVP

---

### ⚠️ Concern 2: Production Performance Validation Needed

**Issue:** Performance targets require validation with production data

**Mitigation:** ✅ Production vault available, benchmark infrastructure ready

**Recommendation:** RUN Sprint 0 performance validation

---

## Recommendations for Sprint 0

### 1. Performance Validation (High Priority)

**Action:** Run benchmark tests with production vault data

**Command:** `mise run perf-validate`

**Success Criteria:**
- Template rendering: <300ms average
- Full vault indexing: <5s for 1000+ notes
- BoltDB lookups: <1ms average
- SQLite queries: <50ms average

---

### 2. Chaos Testing (Medium Priority)

**Action:** Add integration tests for crash scenarios
- Disk-full during cache write
- Process termination mid-transaction
- Corrupt cache file recovery

---

### 3. CI/CD Pipeline Setup (Medium Priority)

**Pipeline Stages:**
1. Format check
2. Lint
3. Unit tests
4. Integration tests
5. Coverage report

**Quality Gates:**
- All tests pass (100%)
- Coverage ≥85% (`internal/app`), ≥80% (overall)
- No critical lint violations

---

### 4. Test Data Management (Low Priority)

**Action:** Document test data generation from production vault

---

## Assessment Summary

| Dimension | Status | Rating | Notes |
|-----------|--------|--------|-------|
| **Controllability** | ✅ PASS | Strong | Hexagonal architecture, DI, mocks, event-driven |
| **Observability** | ✅ PASS | Strong | Structured logging, events, error wrapping, coverage |
| **Reliability** | ✅ PASS | Strong | Atomic writes, Unit of Work, test isolation, CQRS |

**Overall Testability:** ✅ **PASS**

**Concerns:** 2 minor concerns (unmaintained library, performance validation) - both mitigated

**Gate Recommendation:** ✅ **PROCEED** to implementation-readiness after Sprint 0

---

## Sprint 0 Execution Plan

### Tasks

1. ✅ **System-Level Test Design** - This document
2. ⏸️ **Performance Validation** - Run `mise run perf-validate`
3. ⏸️ **Coverage Check** - Run `mise run test-coverage`
4. ⏸️ **Quality Gates** - Run `mise run verify`

### Execution Command

```bash
mise run sprint0
```

This will:
1. Create large test vault (500+ notes from production data)
2. Run performance benchmarks
3. Validate coverage targets
4. Run quality gates (format, lint, test)
5. Generate performance results report

---

## Next Steps

**After Sprint 0 Completion:**

1. ✅ Review performance results in `tests/artifacts/performance/performance-results.txt`
2. ✅ Verify coverage meets targets (≥85% `internal/app`, ≥80% overall)
3. ✅ Proceed to: `/bmad:bmm:workflows:implementation-readiness`

**Implementation Phase:**

1. Begin sprint planning with confidence in testable architecture
2. Use test-driven development for all new features
3. Maintain coverage targets throughout development

---

**Generated by:** BMad TEA Agent - Test Architect Module
**Workflow:** `_bmad/bmm/workflows/testarch/test-design`
**Version:** 4.0 (BMad v6)
**Date:** 2025-12-22
