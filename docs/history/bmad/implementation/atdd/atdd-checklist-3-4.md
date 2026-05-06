# ATDD Checklist - Epic 3, Story 3.4: Create Template Bounded Context

**Date:** 2026-01-14
**Author:** Jack (via TEA Agent)
**Primary Test Level:** Unit (Domain)

---

## Story Summary

Implement the Template domain model with comprehensive validation and composition logic. This bounded context ensures that template structures, variable definitions, and composition rules (like circular dependency detection and depth limits) are strictly enforced at the domain layer.

**As a** developer working with template definitions
**I want** a Template domain model with validation
**So that** template structure and business rules are properly validated at the domain level.

---

## Acceptance Criteria

1. Template entity includes structure validation and business rules.
2. Circular Composition is detected in `includes` and `extends` using DFS (R-001).
3. Composition depth is limited to Max Depth 5 to prevent stack overflow (R-001).
4. Variable definitions are verified for compatibility with MiniJinja syntax (R-006).
5. Template supports modular composition and variable definitions.
6. TemplateCreated event is emitted for template lifecycle.
7. TemplateCommand and TemplateQuery trait interfaces are provided for future implementation.

---

## Failing Tests Created (RED Phase)

### Unit Tests (12 tests)

**File:** `crates/domain/src/template.rs` (557 lines)

- ✅ **Test:** `creates_valid_template_successfully`
  - **Status:** RED - Returns `ValidationFailed("Not implemented")`
  - **Verifies:** Happy path creation of a Template with valid variables.
- ✅ **Test:** `rejects_invalid_template_names`
  - **Status:** GREEN (Negative test) - Correctly catches implementation's current error return.
  - **Verifies:** Validation of name regex `^[a-zA-Z0-9_-]+$`.
- ✅ **Test:** `rejects_large_content`
  - **Status:** RED - Expected `TemplateContentTooLarge` but got `ValidationFailed`.
  - **Verifies:** 1MB content size limit.
- ✅ **Test:** `rejects_too_many_variables`
  - **Status:** RED - Expected `MaxVariablesExceeded` but got `ValidationFailed`.
  - **Verifies:** 50 variable limit per template.
- ✅ **Test:** `validates_template_name_format` (proptest)
  - **Status:** RED - Fails for all generated valid names.
  - **Verifies:** Format compliance via fuzzing.
- ✅ **Test:** `rejects_invalid_variable_names`
  - **Status:** GREEN (Negative test) - Correctly catches error.
  - **Verifies:** Variable name regex `^[a-zA-Z_][a-zA-Z0-9_]*$`.
- ✅ **Test:** `validates_string_constraints`
  - **Status:** RED - `validate_value` not implemented.
  - **Verifies:** String length and pattern constraints.
- ✅ **Test:** `validates_number_constraints`
  - **Status:** RED - `validate_value` not implemented.
  - **Verifies:** Numeric range constraints.
- ✅ **Test:** `detects_direct_circular_composition`
  - **Status:** RED - DFS not implemented.
  - **Verifies:** Prevention of A -> A inclusions.
- ✅ **Test:** `detects_indirect_circular_composition`
  - **Status:** RED - Placeholder for complex cycle test.
  - **Verifies:** Prevention of A -> B -> A cycles.
- ✅ **Test:** `enforces_max_depth_limit`
  - **Status:** RED - Depth check not implemented.
  - **Verifies:** Rejection of depth > 5.
- ✅ **Test:** `validates_override_type_consistency`
  - **Status:** RED - Type check not implemented.
  - **Verifies:** Overrides match defined variable types.

---

## Data Factories Created

N/A - Using internal `fixtures` module in `template.rs` for domain tests.

---

## Fixtures Created

### Template Fixtures

**File:** `crates/domain/src/template.rs`

**Fixtures:**

- `example_template` - Provides a valid template with a "title" variable.
  - **Setup:** Manual construction.
  - **Provides:** `Template` struct.
  - **Cleanup:** N/A (Memory only).

---

## Mock Requirements

N/A - Domain layer tests are pure and do not require mocks.

---

## Required data-testid Attributes

N/A - Backend domain logic only.

---

## Implementation Checklist

### Test: Creates Valid Template

**File:** `crates/domain/src/template.rs`

**Tasks to make this test pass:**

- [ ] Implement `Template::new` constructor.
- [ ] Add basic validation for name and content size.
- [ ] Initialize `metadata` with defaults.
- [ ] Run test: `mise run test:unit:core -- creates_valid_template_successfully`
- [ ] ✅ Test passes (green phase)

**Estimated Effort:** 1 hour

### Test: Cycle Detection & Depth Limits

**File:** `crates/domain/src/template.rs`

**Tasks to make this test pass:**

- [ ] Implement DFS algorithm in `TemplateComposition::detect_cycles`.
- [ ] Add tracking of visited nodes to prevent infinite loops.
- [ ] Check depth counter against limit (5).
- [ ] Run test: `mise run test:unit:core -- composition`
- [ ] ✅ Test passes (green phase)

**Estimated Effort:** 4 hours

---

## Running Tests

```bash
# Run all failing tests for this story
mise run test:unit:core -- models::template::tests

# Run specific test group
mise run test:unit:core -- models::template::tests::composition

# Run tests with coverage
mise run test:coverage
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

**TEA Agent Responsibilities:**

- ✅ All tests written and failing
- ✅ Fixtures and factories created with auto-cleanup (in-memory)
- ✅ Mock requirements documented (None)
- ✅ Implementation checklist created

**Verification:**

- Verified 10 tests fail as expected, 2 pass correctly as negative tests.
- Failure messages: `Validation failed: Not implemented`.

---

## Next Steps

1. **Share this checklist and failing tests** with the dev workflow (manual handoff).
2. **Begin implementation** in `crates/domain/src/template.rs`.
3. **Implement CQRS logic** in adapters following the defined ports.
4. **Emit TemplateCreated event** upon successful creation.

---

## Knowledge Base References Applied

- **test-quality.md** - Applied Given-When-Then and atomic assertions.
- **test-levels-framework.md** - Selected Unit level for pure domain logic.

---

## Test Execution Evidence

### Initial Test Run (RED Phase Verification)

**Command:** `mise run test:unit:core -- models::template::tests`

**Summary:**

- Total tests: 12
- Passing: 2 (Negative validation tests)
- Failing: 10 (Expected due to missing implementation)
- Status: ✅ RED phase verified

---

**Generated by BMad TEA Agent** - 2026-01-14
