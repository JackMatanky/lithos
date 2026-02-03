# Test Design: Story 3.3 - Schema Bounded Context

**Date:** Wed Jan 14 2026
**Author:** Jack (via Tea Agent)
**Status:** Draft
**Story Link:** [_bmad-output/implementation-artifacts/stories/3-3-create-schema-bounded-context.md](_bmad-output/implementation-artifacts/stories/3-3-create-schema-bounded-context.md)

---

## Executive Summary

**Scope:** Targeted test design for the Schema Bounded Context, focusing on the Schema aggregate, PropertyBank, Property, and PropertySpec variants.

**Risk Summary:**
- Total story-specific risks identified: 6
- High-priority risks (≥6): 3
- Critical categories: TECH, DATA, BUS

**Coverage Summary:**
- P0 scenarios: 10
- P1 scenarios: 12
- P2/P3 scenarios: 8
- **Total effort**: ~35 hours

---

## Risk Assessment

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation |
| ------- | -------- | ----------- | ----------- | ------ | ----- | ---------- |
| **R-001** | **TECH** | **Circular Inheritance**: A schema extending itself or forming a cycle (A->B->A) causes stack overflow. | 3 | 3 | **9** | Implement DFS with `visited` set during inheritance resolution. |
| **R-002** | **DATA** | **ID Determinism**: Blake3 hash collisions or non-deterministic inputs lead to duplicate/corrupt Property IDs. | 2 | 3 | **6** | Canonical serialization of Spec content before hashing. Proptest validation. |
| **R-005** | **BUS** | **Regex Vulnerability**: Malicious or poorly written regex (ReDoS) in StringSpec causes performance degradation. | 2 | 3 | **6** | Use `regex` crate with safe compilation; pre-validated pattern module. |
| **R-009** | **TECH** | **Inheritance Resolution**: Properties from parent not correctly merged, overridden, or excluded in child schema. | 2 | 2 | 4 | Deep testing of the `resolve_properties` algorithm. |
| **R-010** | **DATA** | **Type Constraint Leakage**: Constraints from one spec variant applying to another (e.g., Number constraints on String). | 1 | 3 | 3 | Strict exhaustive matching in validation logic; type-specific unit tests. |
| **R-011** | **TECH** | **Ref Resolution**: `PropertyBank.resolve_ref` fails to resolve pointers like `#/properties/name` correctly. | 2 | 2 | 4 | Integration tests for $ref resolution path. |

---

## Test Coverage Plan

### P0 (Critical) - Run on every commit
*Focus: Security, Stability, and Core Identity.*

| Requirement | Test Level | Risk Link | Scenario |
| ----------- | ---------- | --------- | -------- |
| Circular Detection | Unit | R-001 | Schema extending itself (direct cycle). |
| Circular Detection | Unit | R-001 | Schema extending parent that extends child (indirect cycle). |
| ID Determinism | Unit | R-002 | Same Name + Same Spec produces identical ID. |
| ID Uniqueness | Proptest | R-002 | Different Specs (even subtle) produce different IDs. |
| Regex Safety | Unit | R-005 | Invalid regex strings fail compilation gracefully. |
| PropertyBank Deduplication | Unit | - | Registering same property twice returns existing instance. |
| StringSpec Validation | Unit | - | Enum validation (value in/out of set). |
| NumberSpec Validation | Unit | - | Step validation (e.g., 0.5 step for 1.0, 1.1, 1.5). |
| Inheritance Logic | Unit | R-009 | Child schema correctly excludes properties from parent. |
| Inheritance Logic | Unit | R-009 | Child schema correctly overrides parent property with same name. |

### P1 (High) - Run on PR to main
*Focus: Feature completeness and boundary conditions.*

| Requirement | Test Level | Risk Link | Scenario |
| ----------- | ---------- | --------- | -------- |
| Property Name Regex | Unit | - | Reject names with uppercase, spaces, or special chars (except `_-`). |
| Schema Name Regex | Unit | - | Reject names with underscores or uppercase (hyphens only). |
| NumberSpec Bounds | Unit | - | Min/Max boundary checks (inclusive). |
| StringSpec Length | Unit | - | Min/Max length constraints. |
| DateSpec Format | Unit | - | Format string validation (valid ISO patterns). |
| FileSpec Class | Unit | - | Reject invalid file classes (must be image/pdf/note/audio/video). |
| FileSpec Dir | Unit | - | Reject absolute paths in directory restriction. |
| PropertyBank Lookup | Unit | - | Lookup by definition (name + spec) returns correct property. |
| PropertyBank Ref | Unit | R-011 | Parse and resolve `#/properties/name` correctly. |
| Domain Events | Unit | - | `SchemaCreated` event contains correct ID and metadata. |
| Domain Events | Unit | - | `PropertyBankUpdated` event fired on new registration. |
| CQRS Ports | Unit | - | `SchemaCommand` and `SchemaQuery` traits have correct signatures. |

### P2/P3 (Medium/Low) - Nightly/On-demand
*Focus: Edge cases and Performance.*

| Requirement | Test Level | Risk Link | Scenario |
| ----------- | ---------- | --------- | -------- |
| ID Gen Perf | Benchmark | - | Blake3 hashing < 1μs for standard properties. |
| Resolution Perf | Benchmark | - | Inheritance resolution < 10μs for 5-level deep chains. |
| Large Property Bank | Unit | - | Performance/Memory with 1000+ properties. |
| String Patterns | Unit | R-005 | Verify `patterns::EMAIL` correctly rejects/accepts edge cases. |
| String Patterns | Unit | R-005 | Verify `patterns::WIKILINK` handles aliases correctly. |
| Serialization | Unit | - | Property/Schema round-trip via Serde (JSON/TOML). |

---

## Quality Gate Criteria

- **AC Validation**: All 8 Story ACs must have at least one P0/P1 test scenario.
- **Circular Check**: 100% of inheritance paths must pass cycle detection.
- **ID Stability**: Property IDs must remain stable across different execution environments.
- **Coverage**: 90%+ code coverage for `schema/` (domain logic).
- **Complexity**: All functions in `schema/` must have cognitive complexity < 25.

---

## Acceptance Criteria Review

The current Story ACs in `3-3-create-schema-bounded-context.md` are **Sufficient** but could be strengthened with:
1. **Explicit mention of ReDoS prevention** or safe regex handling (addressed in this test design).
2. **Deterministic ID normalization rules**: Mentioning that the spec must be canonicalized before hashing to avoid whitespace/order variations in the debug string.

---

## Next Steps

1. Implement `DFS` cycle detection first as it is the highest risk (Score 9).
2. Set up `proptest` for `Property::compute_id` to verify collision resistance.
3. Establish the `patterns` module with the required regexes (Email, URL, etc.) to mitigate R-005.
4. Use `criterion` to verify performance targets for ID generation and inheritance.
