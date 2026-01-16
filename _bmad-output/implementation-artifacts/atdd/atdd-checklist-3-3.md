# ATDD Checklist - Story 3.3: Create Schema Bounded Context

**Date:** 2026-01-14
**Author:** Jack (via TEA Agent)
**Primary Test Level:** Unit Tests (Domain Logic)

---

## Story Summary

As a developer defining metadata schemas, I want a complete schema domain with PropertyBank, Property, and PropertySpec variants, so that schemas can define reusable property definitions with rich validation constraints.

**As a** developer
**I want** a complete schema domain
**So that** I can define reusable property definitions and validate them

---

## Acceptance Criteria

1. Schema entity (Name, Extends, Excludes, Properties[], ResolvedProperties[])
2. PropertyBank entity (singleton registry of reusable Property definitions)
3. Property entity (ID, Name, Required, Array, Spec)
4. PropertySpec trait with variants: StringSpec, NumberSpec, BoolSpec, DateSpec, FileSpec
5. Schema supports inheritance (Extends) and property exclusion (Excludes)
6. Property ID is deterministically generated using Blake3 from Name + Spec content (R-002)
7. Circular Inheritance detection using a DFS-based algorithm (R-001)
8. Regex patterns in StringSpec are validated for safe compilation (R-005)
9. Domain events (SchemaCreated, PropertyBankUpdated) and CQRS ports (SchemaCommand, SchemaQuery)

---

## Failing Tests Created (RED Phase)

### Unit Tests (11 tests)

**File:** `crates/domain/src/models/schema.rs`

- ✅ **Test:** `detects_circular_inheritance`
  - **Status:** RED - unimplemented!
  - **Verifies:** R-001 DFS-based circular inheritance detection.
- ✅ **Test:** `resolves_inheritance_correctly`
  - **Status:** RED - unimplemented!
  - **Verifies:** Merging parent properties and applying excludes.
- ✅ **Test:** `validates_schema_name_format`
  - **Status:** RED - unimplemented!
  - **Verifies:** Name regex `^[a-z0-9]+(-[a-z0-9]+)*$`.
- ✅ **Test:** `id_is_deterministic_using_blake3`
  - **Status:** RED - unimplemented!
  - **Verifies:** R-002 deterministic hashing for Property IDs.
- ✅ **Test:** `rejects_invalid_property_names`
  - **Status:** RED - unimplemented!
  - **Verifies:** Property name regex `^[a-z0-9_-]+$`.
- ✅ **Test:** `validates_regex_patterns_safely`
  - **Status:** RED - unimplemented!
  - **Verifies:** R-005 safe regex compilation and ReDoS protection.
- ✅ **Test:** `deduplicates_properties_on_registration`
  - **Status:** RED - unimplemented!
  - **Verifies:** PropertyBank returns existing property for same ID.
- ✅ **Test:** `resolves_refs_correctly`
  - **Status:** RED - unimplemented!
  - **Verifies:** Resolving `#/properties/name` pointers.
- ✅ **Test:** `string_spec_validates_enums`
  - **Status:** RED - unimplemented!
  - **Verifies:** String validation against allowed enum values.
- ✅ **Test:** `number_spec_validates_steps`
  - **Status:** RED - unimplemented!
  - **Verifies:** Numeric validation against step constraints.
- ✅ **Test:** `file_spec_validates_file_classes`
  - **Status:** RED - unimplemented!
  - **Verifies:** File class restriction (image, pdf, note, etc.).

---

## Data Infrastructure Created

### Schema Fixtures

**File:** `crates/domain/src/models/schema.rs` (mod fixtures)

**Exports:**

- `TEST_SCHEMA_ID` - Deterministic UUID v7.
- `example_property()` - Pre-configured property for testing.

---

## Implementation Checklist

### Test: id_is_deterministic_using_blake3
**File:** `crates/domain/src/models/property.rs`
- [x] Implement `Property::compute_id` using `blake3`.
- [x] Ensure name and spec debug representation are hashed.
- [x] Run test: `cargo test models::property::tests::property::id_is_deterministic_using_blake3`

### Test: detects_circular_inheritance
**File:** `crates/domain/src/models/schema.rs`
- [x] Implement DFS-based cycle detection in `Schema::new`.
- [x] Use `visited` set to track inheritance chain (Note: unit test uses direct check, aggregate check in app layer).
- [x] Run test: `cargo test models::schema::tests::schema::detects_circular_inheritance`

### Test: validates_regex_patterns_safely
**File:** `crates/domain/src/models/property.rs`
- [x] Implement regex compilation check in `StringSpec::validate`.
- [x] Use `regex` crate for safe compilation.
- [x] Run test: `cargo test models::property::tests::property::validates_regex_patterns_safely`

---

## Running Tests

```bash
# Run all tests for the schema context
cargo test models::schema
cargo test models::property
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

- ✅ All tests written and failing (unimplemented!)
- ✅ DomainError variants added to errors.rs
- ✅ Ports defined in ports/schema.rs
- ✅ Implementation checklist created

### GREEN Phase (Complete) ✅

- ✅ PropertySpec variants implemented with validation logic.
- ✅ Property entity implemented with Blake3 deterministic IDs.
- ✅ PropertyBank singleton registry implemented with deduplication.
- ✅ Schema aggregate implemented with inheritance resolution.
- ✅ Circular inheritance detection implemented.
- ✅ All 11 tests passing.
- ✅ Clippy warnings resolved.

### REFACTOR Phase (Complete) ✅

- ✅ Split schema.rs into schema.rs and property.rs for maintainability.
- ✅ Refactored validation logic to use `is_some_and` for cleaner code.
- ✅ Simplified test structure and refined clippy expectations.

## Knowledge Base References Applied

- **test-quality.md** - Applied Given-When-Then structure and deterministic ID testing principles.
- **Story 3.1 Learnings** - Applied single-file model pattern for Schema bounded context.
- **R-001, R-002, R-005** - Integrated specific risk mitigations from Test Design.

---

**Generated by BMad TEA Agent** - 2026-01-14
