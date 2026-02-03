# Test Design: Story 3.4 - Create Template Bounded Context

**Date:** Wed Jan 14 2026
**Author:** Murat (via Tea Agent)
**Status:** Approved

---

## Executive Summary

**Scope:** targeted test design for Story 3.4 (Template Bounded Context)

**Risk Summary:**

- Total risks identified: 4
- High-priority risks (≥6): 1
- Critical categories: TECH, DATA

**Coverage Summary:**

- P0 scenarios: 6 (12 hours)
- P1 scenarios: 4 (4 hours)
- P2/P3 scenarios: 4 (2 hours)
- **Total effort**: 18 hours (~2.5 days)

---

## Risk Assessment

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ---------- | ----- | -------- |
| R-001 | TECH | Circular inheritance/composition in Templates causing infinite loops. | 3 | 3 | 9 | Implement DFS with depth limit (5) and visited set in `TemplateComposition::detect_cycles`. | DEV | Sprint 3 |

### Medium-Priority Risks (Score 3-4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ---------- | ----- |
| R-006 | TECH | MiniJinja template syntax incompatibility in Adapter layer affecting Domain validation assumptions. | 2 | 2 | 4 | Integration tests verifying Domain variable definitions match Adapter rendering capabilities. | QA |
| R-009 | DATA | Variable type mismatch between definition and override in composition. | 2 | 2 | 4 | `TemplateComposition::validate` must check override types against base template. | DEV |

### Low-Priority Risks (Score 1-2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ------ |
| R-010 | PERF | Deeply nested composition or massive variable counts causing validation lag. | 1 | 2 | 2 | Enforce MAX 50 variables and MAX 5 depth; monitor with benchmarks. | Monitor |

### Risk Category Legend

- **TECH**: Technical/Architecture
- **DATA**: Data Integrity

---

## Test Coverage Plan

### P0 (Critical) - Run on every commit

**Criteria**: Blocks core journey + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Template Composition Cycles | Unit | R-001 | 3 | DEV | Detect direct (A->A) and indirect (A->B->A) cycles. |
| Composition Depth Limit | Unit | R-001 | 1 | DEV | Verify rejection of depth > 5. |
| Template Name Validation | Unit | - | 4 | DEV | Regex `^[a-zA-Z0-9_-]+$`, max 64 chars. |
| Variable Name Validation | Unit | R-006 | 4 | DEV | Regex `^[a-zA-Z_][a-zA-Z0-9_]*$`, no reserved words. |
| Variable Type Validation | Unit | R-009 | 5 | DEV | String, Number, Boolean, Date, File constraints. |
| Domain Event Emission | Unit | - | 1 | DEV | `TemplateCreated` event on successful creation. |

**Total P0**: 18 tests, 12 hours

### P1 (High) - Run on PR to main

**Criteria**: Important features + Medium risk (3-4) + Common workflows

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Var usage vs definition | Integration | R-006 | 3 | QA | Domain validates all `{{var}}` in content are in `variables` map. |
| Override Type Consistency | Unit | R-009 | 3 | DEV | Composition override value type must match base variable type. |
| Section Insertion | Unit | - | 4 | DEV | Beginning, End, BeforeVariable, AfterVariable logic. |
| Content Size Limits | Unit | R-010 | 2 | DEV | Verify rejection of content > 1MB. |

**Total P1**: 12 tests, 4 hours

### P2 (Medium) - Run nightly/weekly

**Criteria**: Secondary features + Low risk (1-2) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Metadata timestamps | Unit | - | 2 | DEV | `created_at` / `updated_at` defaults and updates. |
| Variable default values | Unit | - | 3 | DEV | Verification of default value resolution. |
| MiniJinja Syntax Check | Integration | R-006 | 4 | QA | Adapter layer syntax validation for complex blocks. |

**Total P2**: 9 tests, 2 hours

---

## Execution Order

### Smoke Tests (<5 min)

**Purpose**: Fast feedback, catch build-breaking issues

- [ ] Create simple valid Template (10ms)
- [ ] Create simple valid TemplateComposition (10ms)
- [ ] Validate valid Template (10ms)

**Total**: 3 scenarios

### P0 Tests (<10 min)

**Purpose**: Critical path validation

- [ ] Cycle Detection (Direct/Indirect)
- [ ] Depth Limit Enforcement
- [ ] Name/Variable Regex Validation
- [ ] Type constraint validation (Range/Pattern)

**Total**: 6 scenarios

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Hours/Test | Total Hours | Notes |
| -------- | ----- | ---------- | ----------- | ----- |
| P0 | 18 | 0.6 | 10.8 | Algorithmic (Cycles) |
| P1 | 12 | 0.4 | 4.8 | Business Logic |
| P2 | 9 | 0.2 | 1.8 | Metadata/Defaults |
| P3 | 0 | 0.25 | 0.0 | Exploratory |
| **Total** | **39** | **-** | **17.4** | **~2.5 days** |

### Prerequisites

**Test Data:**

- `fixtures::circular_composition_graph`
- `fixtures::max_depth_composition`
- `fixtures::reserved_word_variables`

**Tooling:**

- `criterion` for validation latency benchmarking (R-010)
- `proptest` for template name/variable name fuzzing

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions)
- **P1 pass rate**: ≥95% (waivers required for failures)
- **High-risk mitigations**: 100% complete (DFS logic verified)

### Coverage Targets

- **Validation Logic**: 100%
- **Composition Logic**: 100%
- **Variable Constraints**: ≥90%

---

## Mitigation Plans

### R-001: Circular Reference (Score: 9)

**Mitigation Strategy:** Implement Depth-First Search (DFS) with a `visited` stack during resolution in `TemplateComposition::detect_cycles`. Use a max depth of 5.
**Owner:** DEV
**Timeline:** Sprint 3
**Verification:** Unit tests with cycles A->B->A and A->A must return `TemplateError::CircularReference`.

---

## Acceptance Criteria Validation

### Current AC Assessment:
The ACs in Story 3.4 are high-quality but could be improved for clarity on error states.

### Suggested Improvements:
- **ADD AC**: "Given a composition with depth > 5, when validated, then it returns `TemplateError::MaxDepthExceeded`."
- **ADD AC**: "Given a composition with a circular reference, when validated, then it returns `TemplateError::CircularReference` with the list of participating templates."
- **CLARIFY AC**: "template business rules and composition logic are validated internally" -> Specify that this includes variable name regex and type consistency.

---

## Approval

**Test Design Approved By:**

- [ ] Product Manager: ____________________ Date: ________
- [ ] Tech Lead: ____________________ Date: ________
- [ ] QA Lead: ____________________ Date: ________

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/bmm/testarch/test-design`
