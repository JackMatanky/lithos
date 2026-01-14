# Test Design: Epic 3 - Core Domain Models (Config, Note, Schema, Template)

**Date:** Wed Jan 14 2026
**Author:** Jack (via Tea Agent)
**Status:** Draft

---

## Executive Summary

**Scope:** Epic-Level test design for the Core Domain Models (Config, Note, Schema, Template) in the Lithos project. This covers stories 3.1, 3.2, 3.3, and 3.4.

**Risk Summary:**

- Total risks identified: 8
- High-priority risks (≥6): 4
- Critical categories: TECH, DATA, PERF, SEC

**Coverage Summary:**

- P0 scenarios: 16 (32 hours)
- P1 scenarios: 22 (22 hours)
- P2/P3 scenarios: 18 (9 hours)
- **Total effort**: 63 hours (~8 days)

---

## Risk Assessment

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ---------- | ----- | -------- |
| R-001 | TECH | Circular inheritance/composition in Schemas or Templates causing infinite loops/stack overflow. | 3 (Likely) | 3 (Critical) | 9 | Implement Depth-First Search (DFS) with visited set and max-depth limits in resolution logic. | DEV | Sprint 3 |
| R-002 | DATA | Deterministic Property ID generation failure leading to duplicate properties or schema corruption. | 2 (Possible) | 3 (Critical) | 6 | Use `blake3` with strict input normalization; validate with property-based testing (proptest). | DEV | Sprint 3 |
| R-003 | PERF | Domain entity construction exceeding latency budgets (Note > 100μs, Schema > 10μs), blocking indexing NFRs. | 2 (Possible) | 3 (Critical) | 6 | Enforce zero-allocation patterns for hot paths; strict Criterion benchmarking in CI. | DEV | Sprint 3 |
| R-007 | SEC | Sensitive configuration data (API keys) exposed in logs or dumps due to improper encryption handling. | 2 (Possible) | 3 (Critical) | 6 | Ensure `ConfigValue::Encrypted` stores opaque bytes only; strictly control `Debug` impls to mask secrets; verify decryption flows. | DEV | Sprint 3 |

### Medium-Priority Risks (Score 3-4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ---------- | ----- |
| R-004 | SEC | Path traversal vulnerability via invalid Note/Embed paths (e.g., `../../secret`). | 2 (Possible) | 2 (Degraded) | 4 | Strict regex validation for all paths; allow only relative paths without parent directory components. | DEV |
| R-005 | BUS | Regex patterns in Schemas rejecting valid user input, causing user frustration. | 2 (Possible) | 2 (Degraded) | 4 | Provide pre-tested regex patterns (email, URL) in `patterns` module; allow escape hatches in schema config. | PRODUCT |
| R-006 | TECH | MiniJinja template syntax incompatibility in Adapter layer affecting Domain validation assumptions. | 2 (Possible) | 2 (Degraded) | 4 | Integration tests verifying Domain variable definitions match Adapter rendering capabilities. | QA |
| R-008 | BUS | Hierarchical config merging (Vault > Global) fails for complex nested structures, causing unexpected behavior. | 2 (Possible) | 2 (Degraded) | 4 | Comprehensive unit tests for deep merging logic; property-based testing for merge commutativity/idempotence where applicable. | DEV |

### Low-Priority Risks (Score 1-2)

None identified at this stage.

### Risk Category Legend

- **TECH**: Technical/Architecture
- **SEC**: Security
- **PERF**: Performance
- **DATA**: Data Integrity
- **BUS**: Business Impact
- **OPS**: Operations

---

## Test Coverage Plan

### P0 (Critical) - Run on every commit

**Criteria**: Blocks core journey + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Schema Circular Inheritance | Unit | R-001 | 3 | DEV | Detect direct (A->A) and indirect (A->B->A) cycles. |
| Template Composition Cycles | Unit | R-001 | 3 | DEV | Detect cycles in `includes` and `extends`. |
| Property ID Determinism | Unit (Property) | R-002 | 2 | DEV | Verify same inputs produce identical hash IDs. |
| Note Identity Stability | Unit (Property) | R-002 | 2 | DEV | UUID v7 monotonicity and sorting. |
| Entity Construction Perf | Benchmark | R-003 | 2 | DEV | Criterion bench for Note/Schema creation. |
| Config Encryption Safety | Unit | R-007 | 2 | DEV | Verify `Encrypted` variant handles bytes opaque; `Debug` masks content. |
| Config Merge Logic | Unit | R-008 | 2 | DEV | Verify Vault config strictly overrides Global config. |

**Total P0**: 16 tests, 32 hours

### P1 (High) - Run on PR to main

**Criteria**: Important features + Medium risk (3-4) + Common workflows

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Path Validation | Unit | R-004 | 4 | DEV | Verify rejection of `../`, absolute paths, empty paths. |
| Schema Regex Validation | Unit | R-005 | 4 | DEV | Test standard patterns (email, slug) against edge cases. |
| Schema Inheritance | Integration | - | 3 | DEV | Verify property merging and overrides (parent vs child). |
| Template Var Resolution | Integration | R-006 | 3 | DEV | Verify variable constraints (type, default) apply correctly. |
| Frontmatter Type Parsing | Unit | - | 4 | DEV | Test "Best Effort" parsing logic (Date, Number, Bool). |
| Config Validation | Unit | - | 4 | DEV | Test required fields, type checks, and constraints. |

**Total P1**: 22 tests, 22 hours

### P2 (Medium) - Run nightly/weekly

**Criteria**: Secondary features + Low risk (1-2) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| ----------- | ---------- | --------- | ---------- | ----- | ----- |
| Tag Hierarchical Parsing | Unit | - | 5 | DEV | Edge cases: empty segments, multi-level nesting. |
| Task Status Parsing | Unit | - | 3 | DEV | Standard and custom status characters. |
| Link/Embed Parsing | Unit | - | 4 | DEV | Wiki-link aliases, standard markdown links. |
| PropertyBank Lookup | Unit | - | 3 | DEV | Lookup by ID vs Definition. |
| ConfigValue Variants | Unit | - | 3 | DEV | Test String, Number, Boolean, Array, Object variants. |

**Total P2**: 18 tests, 9 hours

### P3 (Low) - Run on-demand

None.

---

## Execution Order

### Smoke Tests (<5 min)

**Purpose**: Fast feedback, catch build-breaking issues

- [ ] Create valid Note with all subentities (10ms)
- [ ] Create valid Schema with 1 property (10ms)
- [ ] Create valid Template with 1 variable (10ms)
- [ ] Create valid Config (Merged Global + Vault) (10ms)

**Total**: 4 scenarios

### P0 Tests (<10 min)

**Purpose**: Critical path validation

- [ ] Schema Cycle Detection (Unit)
- [ ] Template Cycle Detection (Unit)
- [ ] Property ID Hash Collision Check (Proptest)
- [ ] UUID v7 Sorting Check (Proptest)
- [ ] Config Encryption Safety (Unit)
- [ ] Config Merge Logic (Unit)
- [ ] Path Traversal Rejection (Unit)

**Total**: 7 groups

### P1 Tests (<30 min)

**Purpose**: Important feature coverage

- [ ] Full Schema Inheritance Resolution (Integration)
- [ ] Template Composition & Override (Integration)
- [ ] Frontmatter "Best Effort" Type Parsing (Unit)
- [ ] Config Validation Rules (Unit)

**Total**: 4 groups

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Hours/Test | Total Hours | Notes |
| -------- | ----- | ---------- | ----------- | ----- |
| P0 | 16 | 2.0 | 32.0 | Complex algorithmic validation (cycles, crypto, merge) |
| P1 | 22 | 1.0 | 22.0 | Standard business logic |
| P2 | 18 | 0.5 | 9.0 | Simple parsing logic |
| **Total** | **56** | **-** | **63.0** | **~8 days** |

### Prerequisites

**Test Data:**

- `fixtures::complex_schema_graph` (for cycle detection)
- `fixtures::template_dependency_tree` (for composition tests)
- `fixtures::hierarchical_config` (for merging tests)
- `proptest` generators for random strings/paths

**Tooling:**

- `criterion` for benchmarks (R-003)
- `proptest` for fuzzing (R-002)
- `mockall` for port mocking

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions)
- **P1 pass rate**: ≥95% (waivers required for failures)
- **High-risk mitigations**: 100% complete (DFS checks, Merge logic verified)

### Coverage Targets

- **Domain Entities**: ≥80%
- **Validation Logic**: 100% (cycles, regex, paths, merging)

---

## Mitigation Plans

### R-001: Circular Inheritance (Score: 9)

**Mitigation Strategy:** Implement Depth-First Search (DFS) with a `visited` stack during resolution. If current node is in `visited`, return `DomainError::CircularReference`.
**Owner:** DEV
**Timeline:** Sprint 3
**Verification:** Unit test with A->B->A schema graph MUST fail with specific error.

### R-002: Deterministic ID Collisions (Score: 6)

**Mitigation Strategy:** Use `blake3` hashing on normalized canonical representation of Property Spec (sorted keys, consistent serialization).
**Owner:** DEV
**Timeline:** Sprint 3
**Verification:** Proptest generating 10,000 properties asserting `hash(a) == hash(a)` and `hash(a) != hash(b)`.

### R-007: Encryption Exposure (Score: 6)

**Mitigation Strategy:** `ConfigValue::Encrypted` variant must store `Vec<u8>`. `Debug` impl for `ConfigValue` must print `***` for encrypted variant.
**Owner:** DEV
**Timeline:** Sprint 3
**Verification:** Unit test `format!("{:?}", config_value)` asserts output does not contain raw secret bytes.

### R-008: Config Merge Conflicts (Score: 4)

**Mitigation Strategy:** Implement robust deep merge logic for nested structures where appropriate, or clear replacement rules (Vault completely replaces Global section).
**Owner:** DEV
**Timeline:** Sprint 3
**Verification:** Unit tests covering all merge combinations (Global only, Vault only, Both present, Nested overrides).

---

## Assumptions and Dependencies

### Assumptions

1. `uuid::v7` implementation in Rust crate is strictly monotonic (or handles clock rollback safely).
2. `blake3` provides sufficient collision resistance for 16-char hex truncation.
3. `chrono` parsing is sufficient for "Best Effort" date detection.
4. `thiserror` and `serde` are available and stable.

### Dependencies

1. `proptest` crate availability.
2. `criterion` crate availability.

---

## Approval

**Test Design Approved By:**

- [ ] Product Manager: ____________________ Date: ________
- [ ] Tech Lead: ____________________ Date: ________
- [ ] QA Lead: ____________________ Date: ________

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/bmm/testarch/test-design`
