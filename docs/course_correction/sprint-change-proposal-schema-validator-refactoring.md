# Sprint Change Proposal: Schema Validator Architectural Refactoring

## Update: Schema System Optimizations Completed (2025-11-01)

**Status:** ✅ COMPLETED - All 7 schema system optimizations have been successfully implemented and tested.

**Completed Optimizations:**
1. ✅ **High Priority:** Fixed $ref resolution to preserve original PropertyBank key instead of regenerating hash ID
2. ✅ **Medium Priority:** Fixed context propagation in schemaDTO.toDomain to use caller's context
3. ✅ **Medium Priority:** Optimized inheritance performance from O(n²) to O(1) using index maps
4. ✅ **Medium Priority:** Optimized JSON decoding to unmarshal only once instead of twice
5. ✅ **Low Priority:** Removed dead code (SchemaValidator.validatePropertyRefs no-op method)
6. ✅ **Medium Priority:** Refactored dto.go structure (completed as part of other changes)
7. ✅ **Medium Priority:** Eliminated duplicate Property validation calls across layers

**Impact on This Proposal:**
- The schema system is now more efficient and correct
- Performance improvements in inheritance resolution and JSON processing
- Proper context propagation enables better error handling and cancellation
- Foundation is stronger for the architectural refactoring outlined below

**Next Steps:** Proceed with the architectural refactoring phases as outlined in this document.

---

## Analysis Summary

### Change Context

**Change Trigger:** Architectural inconsistency discovered in Story 2.7 implementation

**Nature of Change:** The current implementation of Story 2.7 "Implement Schema Validator Service" violates hexagonal architecture principles by keeping validation and business logic methods in domain models instead of extracting them to application services. This represents a significant architectural violation that must be addressed before proceeding with dependent stories.

**Scope of Impact:**
- Domain models (Schema, Property, PropertySpec implementations)
- SchemaValidator service
- New SchemaEngine service (to be created)
- All callers of domain model methods

**Business Value Impact:** The current approach violates the clean architecture pattern established in the PRD, making the codebase more difficult to maintain, test, and extend in the future. Correcting this now prevents architectural debt from accumulating early in the project lifecycle.

### Current State Assessment

**Current Implementation:**
- SchemaValidator service exists but only implements a subset of functionality
- Domain models (Schema, Property, PropertySpec) still contain validation methods
- Domain models also contain business logic methods (Get*/Has*)
- Violation of hexagonal architecture principles where:
  - Domain layer should contain only data structures
  - Application layer should contain all business logic and validation

**Specific Issues:**
1. `Property.Validate()` methods remain in domain models
2. `Schema.Validate()` methods remain in domain models
3. PropertySpec validation methods remain in implementations
4. `Schema.Get*()` and `Schema.Has*()` business methods remain in domain models
5. `Property.Get*()` and `Property.Has*()` business methods remain in domain models
6. No clear separation between domain data structures and application business logic

**Technical Debt:**
- Architectural confusion between domain and application responsibilities
- Inconsistent implementation of hexagonal architecture principles
- Potential maintenance challenges as project grows

### Epic/Story Impact Analysis

**Story 2.7:**
- Status must change from "Ready for Review" back to "In Progress"
- Acceptance criteria must be expanded to include SchemaEngine service
- Additional tasks needed for comprehensive method extraction

**Story 2.8:**
- Need to postpone Story 2.8 "Review Epic 2 Testing Alignment" to become Story 2.9
- Create a new intermediate Story 2.8 focused on errors package refactoring

**Dependencies:**
- Story 2.9+ (previously 2.8+) rely on proper architectural implementation
- Error handling package refactoring must precede full architecture refactoring
- Future domain services need clear precedent for hexagonal boundaries

**Required Adjustments:**
1. Add SchemaEngine to components.md documentation
2. Create new SchemaEngine service with clean API
3. Extract all validation methods to SchemaValidator
4. Extract all business logic methods to SchemaEngine
5. Make domain models pure data structures
6. Update all callers to use service methods instead of model methods
7. Refactor errors package to provide clean Result[T] pattern foundation
8. Postpone current Story 2.8 to become Story 2.9

## Path Forward

### Recommended Path: Complete Refactoring

**Path Description:** Implement a comprehensive refactoring that:
1. Creates proper SchemaValidator with clean public API
2. Creates new SchemaEngine service with business logic methods
3. Extracts ALL methods from domain models to appropriate services
4. Ensures domain models are pure data structures
5. Updates all callers to use service methods

**Implementation Approach:**
A structured, four-phase approach will minimize risk while ensuring comprehensive architectural correction:

**Phase 0: Error Handling Refactoring (New Story 2.8)**
- Refactor `internal/shared/errors/` package to focus on Result[T] pattern
- Implement clean, lean error types based on `aidantwoods-go-result-digest.txt`
- Create extension points for domain-specific errors
- Update existing error uses to leverage the refactored system
- Document the approach in `internal/shared/errors/README.md`

**Phase 1: Foundation (Story 2.7)**
- Update components.md documentation (already completed by PO)
- Create SchemaValidator service with clean public API (limited to what callers need)
- Create SchemaEngine service with clean public API for business logic
- Define proper port interfaces in `internal/ports/api/schema.go`

**Phase 2: Method Extraction (Story 2.7)**
- Extract validation methods from domain models to SchemaValidator as private methods
- Extract business logic methods (Get*/Has*) from domain models to SchemaEngine
- Update all callers to use service methods instead of model methods
- Ensure domain models have no logic methods remaining

**Phase 3: Integration & Testing (Story 2.7)**
- Run full test suite to verify no regressions
- Execute story-dod-checklist
- Update story status to "Ready for Review"

**Key Technical Decisions:**
1. **Clean Public APIs:**
   - SchemaValidator: Only expose what callers need (e.g., `Validate()`)
   - SchemaEngine: Expose business logic methods with clear names
   - Internal implementation details stay private

2. **Comprehensive Method Extraction:**
   - ALL methods must be moved out of domain models
   - No exceptions for "convenience methods"
   - Domain models should only have getters/setters if absolutely necessary

3. **Service Responsibilities:**
   - SchemaValidator: All validation logic
   - SchemaEngine: All business logic operations (Get*/Has*)

4. **Error Handling Refactoring:**
   - Focus on Result[T] pattern as core foundation
   - Based on the `aidantwoods-go-result-digest.txt` reference
   - Create absolute minimum set of error types (2-3 at most)
   - Eliminate ALL bloat - each error type has only essential fields
   - Enforce correct domain terminology in error types
   - Audit ALL error usage throughout the codebase
   - No backward compatibility - enforce correct implementation
   - Design for consistent, minimal error handling

**Rollback Plan:**
If implementation difficulties arise:
1. Restore domain models to current state
2. Simplify SchemaValidator to match current implementation
3. Defer SchemaEngine to a separate story
4. Document architectural debt for future resolution

## Proposed Changes

### Documentation Changes

**`docs/architecture/components.md`:**
- Replace FrontmatterValidator with SchemaValidator:
  ```markdown
### SchemaValidator

**Responsibility:** Validate complete schema definitions and property banks to ensure data
integrity before use in frontmatter validation or schema registration.

**Key Interfaces:**

- `ValidateSchema(ctx context.Context, schema Schema) Result[ValidationResult]`
- `ValidatePropertyBank(ctx context.Context, bank PropertyBank) Result[ValidationResult]`

**Dependencies:** Logger, Error package, Result[T] pattern.

**Technology Stack:** Go stdlib validation, PropertySpec polymorphism, Result[T] pattern
for functional error handling. All specific validation logic (property validation, spec
validation, schema component validation) implemented as private methods behind the clean
public API.

### SchemaEngine

**Responsibility:** Coordinate schema loading, validation, and provide business logic
operations for schema and property access within the application layer.

**Key Interfaces:**

- `LoadSchema(ctx context.Context) Result[[]Schema]` - Load and validate schemas through
SchemaLoaderPort
- `LoadPropertyBank(ctx context.Context) Result[*PropertyBank]` - Load and validate
property bank through SchemaLoaderPort
- `GetSchema(ctx context.Context, name string) Result[Schema]` - Retrieve validated schema
by name
- `HasSchema(ctx context.Context, name string) Result[bool]` - Check if schema exists
- `GetProperty(ctx context.Context, name string) Result[Property]` - Retrieve property from
property bank
- `HasProperty(ctx context.Context, name string) Result[bool]` - Check if property exists
in bank

**Dependencies:** SchemaLoaderPort, SchemaValidator, Logger, Error package.

**Technology Stack:** Coordination layer using existing SchemaLoaderAdapter logic with
SchemaValidator injection for validation, business logic for schema/property access
operations, Result[T] pattern for error handling.
  ```

- Update component diagram to include SchemaEngine

### Story Changes

**`docs/stories/2.7.implement-schema-validator-service.md`:**
- Change status to "In Progress"
- Expand acceptance criteria to include SchemaEngine and comprehensive method extraction
- Add new tasks for SchemaEngine implementation and business logic extraction
- Update Definition of Done to include all architectural requirements

**Create new Story 2.8:**
- Create a new story focused on refactoring the errors package
- Include requirements for lean Result[T] pattern implementation based on the reference document
- Define error type hierarchies for extension by domain-specific errors
- Update existing error usages to work with the refactored system

**Renumber existing Story 2.8 to 2.9:**
- Update the existing "Review Epic 2 Testing Alignment" story to become Story 2.9
- Adjust references in other documents as needed

### Code Changes

**New Files to Create:**
- `internal/app/schema/validator.go`: SchemaValidator implementation
- `internal/app/schema/validator_test.go`: Comprehensive tests
- `internal/app/schema/engine.go`: SchemaEngine implementation
- `internal/app/schema/engine_test.go`: Comprehensive tests

**Errors Package Refactoring:**
Instead of creating new error types, we will perform a comprehensive refactoring of the `internal/shared/errors` package to:

- Focus on the Result[T] pattern as the core foundation (based on `docs/refs/aidantwoods-go-result-digest.txt`)
- Create an absolute minimum set of error types (exactly 3 base types):
  1. BaseError (minimal with just message and cause)
  2. ValidationError (simplified for domain validation errors)
  3. ResourceError (for resource operations)
- Eliminate ALL bloat - current BaseError has 5 attributes which is excessive
- Ensure each error type has only essential fields (no bloat allowed)
- Enforce correct domain terminology in all error types (e.g., schema errors must use "property" not "field")
- Audit and update ALL uses of the error system throughout the codebase
- NO backward compatibility - enforce correct usage everywhere

**Files to Modify:**
- `internal/shared/errors/result.go`: Refactor to a cleaner Result[T] pattern implementation
- `internal/shared/errors/types.go`: Streamline core error types
- `internal/shared/errors/schema.go`: Update to use the refactored Result pattern
- `internal/shared/errors/README.md`: Document the error handling approach
- `internal/domain/schema.go`: Remove all methods
- `internal/domain/property.go`: Remove all methods
- `internal/domain/property_bank.go`: Remove all methods
- `internal/domain/property_specs.go`: Remove all validation methods from implementations

**Story Sequencing Changes:**
- Postpone Story 2.8 "Review Epic 2 Testing Alignment" to become Story 2.9
- Insert this architectural refactoring as the new Story 2.8

## Action Items

1. PO updates components.md and story 2.7 (COMPLETED)
2. PO creates new story for errors package refactoring and renumbers Story 2.8 to 2.9
3. Dev implements errors package refactoring in new Story 2.8
4. Dev implements Phase 1: Foundation for Story 2.7
5. Dev implements Phase 2: Method Extraction for Story 2.7
6. Dev implements Phase 3: Integration & Testing for Story 2.7
7. QA reviews updated implementation

## Approvals

| Role | Name | Approval Date |
|------|------|---------------|
| Product Owner | Sarah | 2025-10-22 |
| Developer | James | TBD |
| Architect | Winston | TBD |

## Conclusion

This Sprint Change Proposal addresses a critical architectural issue early in the project lifecycle. By implementing a comprehensive refactoring that properly separates domain models from application services, we ensure the codebase follows hexagonal architecture principles consistently. This will improve maintainability, testability, and extensibility as the project grows.

The approach also addresses an important foundational concern in our error handling system. By refactoring the errors package to focus on a clean, lean Result[T] pattern implementation, we establish a solid foundation for error handling throughout the application. This is a prerequisite for proper architectural boundaries and clean APIs in our services.

The four-phase approach (starting with errors package refactoring) minimizes risk while ensuring thorough architectural correction. Upon completion, we will have:

1. A clean, consistent error handling foundation based on the Result[T] pattern
2. Clear separation between domain data models and application layer services
3. Properly implemented hexagonal architecture principles
4. A solid foundation for subsequent stories and clear precedents for future services

This comprehensive refactoring sets the stage for a more maintainable, testable codebase that better adheres to our architectural principles.
