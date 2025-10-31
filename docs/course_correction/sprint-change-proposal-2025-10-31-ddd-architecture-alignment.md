# Sprint Change Proposal: DDD Architecture Alignment & Infrastructure Layer Separation

**Date:** October 31, 2025
**Prepared By:** Sarah (Product Owner)
**Status:** APPROVED
**Type:** Architectural Refactoring

## Executive Summary

The schema system currently violates Domain-Driven Design (DDD) principles by mixing infrastructure concerns (JSON schema processing, $ref dereferencing) with domain logic (business rules, validation). This architectural misalignment is causing development friction, particularly the need for IProperty interface due to PropertyRef/Property mutual exclusion, and making the frontmatter service implementation difficult due to bloated schema system design.

**Recommended Solution:** Restructure the schema system to follow proper DDD boundaries with infrastructure layer separation, transforming domain models into proper entities/aggregates/value objects while moving infrastructure concerns to the adapter layer.

**Impact:** 3 completed stories (2.2, 2.6, 2.7) require refactoring. Epic 3+ development protected by stable SchemaEngine interface.

**Timeline:** 2-3 story cycles with Test-Driven Development approach.

## Change Context & Trigger Analysis

### Triggering Issue

**Source:** Architectural analysis during Epic 2 (Schema system) development, specifically affecting Story 3.7 (FrontmatterService) implementation.

**Primary Problem:** Schema system architectural misalignment causing:
- IProperty interface needed due to Property/PropertyRef mutual exclusion
- Schema system bloat hindering frontmatter service development
- Infrastructure concerns mixed with domain logic
- Violation of DDD principles

**Root Cause Analysis:**
1. **Incorrect Assumption:** Treated $ref from JSON as equivalent to Property objects
2. **Misaligned Heuristic:** Applied "validation is always domain concern" without distinguishing data structure validation (infrastructure) from business rule validation (domain)
3. **Layer Confusion:** PropertyRef model exists in domain when it should be infrastructure-only

### Evidence

**Development Pain Points:**
- Had to create IProperty interface because Property and PropertyRef are mutually exclusive
- Schema system becoming bloated, making frontmatter service implementation difficult
- Circular dependency issues mentioned in Story 3.7 related to this architectural problem

**Proactive Recognition:** Issue identified before becoming a blocking technical debt, allowing for planned refactoring rather than emergency fixes.

## Epic Impact Assessment

### Current Epic Analysis

**Epic 2 (Configuration Schema Loading):**
- ✅ **Stories Affected:** 2.2 (Done), 2.6 (Done), 2.7 (Done) - all require refactoring
- ✅ **Can be completed:** Yes, with DDD refactoring approach
- ✅ **Modification needed:** Update story status to In Progress, add new acceptance criteria

**Epic 3 (Vault Indexing Engine):**
- ✅ **Protection:** Story 3.7 (FrontmatterService) protected by stable SchemaEngine interface
- ✅ **No pause needed:** Development can continue through SchemaEngine abstraction layer
- ✅ **Dependencies:** Only uses SchemaEngine, which remains stable

### Future Epic Analysis

**Epic 4 & 5 (Schema-driven lookups, Interactive input):**
- ✅ **Benefits:** Will gain cleaner domain boundaries and improved architecture
- ✅ **No breaking changes:** SchemaEngine interface stability preserves compatibility
- ✅ **Enhanced maintainability:** Proper DDD structure improves future development velocity

**Epic Sequence:** No reordering needed - refactoring fits within current Epic 2 completion.

## Artifact Conflict & Impact Analysis

### Architecture Document Updates Required

**docs/architecture/data-models.md:**
- ✅ **Property Model:** Transform to DDD entity with hash-based ID
- ✅ **Schema Model:** Enhance as DDD aggregate
- ✅ **PropertyBank Model:** Confirm as DDD aggregate
- ✅ **PropertyRef Model:** Remove from domain, document as infrastructure-only

**docs/architecture/components.md:**
- ✅ **Component Reassignment:** Move SchemaValidator, SchemaResolver to adapter layer
- ✅ **Service Layer:** Clean SchemaEngine definition as orchestrator service
- ✅ **Adapter Layer:** Document PropertyDereferencer, SchemaExtender, SchemaValidator

**docs/architecture/source-tree.md:**
- ✅ **New Structure:** Document adapter layer organization
- ✅ **Layer Boundaries:** Clarify domain vs infrastructure concerns

### Story Artifacts Requiring Updates

**Story 2.2 (Property & PropertySpec Models):**
- ✅ **Status Change:** Done → In Progress
- ✅ **New Acceptance Criteria:** DDD entity transformation with ID generation
- ✅ **Tasks:** TDD approach for Property entity with hash-based ID

**Story 2.6 (SchemaValidator Service):**
- ✅ **Status Change:** Done → In Progress
- ✅ **New Acceptance Criteria:** Move to adapter layer at `internal/adapter/spi/schema/validator.go`
- ✅ **Tasks:** File relocation with import updates

**Story 2.7 (SchemaResolver Service):**
- ✅ **Status Change:** Done → In Progress
- ✅ **New Acceptance Criteria:** Move to adapter layer and split into dereferencer + extender
- ✅ **Tasks:** Component decomposition with TDD validation

### PRD Impact Assessment

**✅ No PRD conflicts:** This is internal architecture improvement that preserves all functional requirements and user-facing behavior.

## Path Forward Evaluation

### Option 1: Direct Adjustment / Integration ✅ RECOMMENDED

**Scope:**
- Refactor Stories 2.2, 2.6, 2.7 to follow proper DDD principles
- Move infrastructure concerns to adapter layer
- Restructure domain models as proper DDD entities/aggregates/value objects
- Create JSON schema reference deliverable
- Leverage existing propertyRefDTO in schema adapter

**Effort Assessment:**
- **Timeline:** Medium effort (~2-3 story cycles)
- **Risk Level:** Low - SchemaEngine interface preserved
- **Work Preserved:** All algorithms and comprehensive test suites maintained
- **Infrastructure Advantage:** Existing propertyRefDTO already available

**Benefits:**
- ✅ **Protects Epic 3:** SchemaEngine interface stability ensures no Epic 3 disruption
- ✅ **Preserves Quality:** Excellent implementations in 2.6 and 2.7 maintained
- ✅ **Improves Maintainability:** Proper DDD boundaries enable sustainable development
- ✅ **Leverages Existing Work:** propertyRefDTO infrastructure already in place

### Option 2: Potential Rollback ❌ NOT RECOMMENDED

**Assessment:** Would lose significant high-quality work
- Stories 2.6 and 2.7 have exceptional implementation quality
- Comprehensive test suites (96%+ coverage) would be lost
- Sophisticated algorithms (inheritance resolution, cycle detection) would be discarded

**Rejection Rationale:** Refactoring preserves valuable work while fixing architectural issues.

### Option 3: PRD MVP Review & Re-scoping ❌ NOT NEEDED

**Assessment:** Not applicable for internal architecture improvement
- MVP scope unchanged - all functional requirements preserved
- User-facing behavior identical
- Internal improvement only

### Selected Path: Option 1 - Direct Adjustment / Integration

**Rationale:**
- Preserves SchemaEngine interface (protects Epic 3)
- Builds on existing high-quality implementations
- Leverages existing propertyRefDTO infrastructure
- Improves long-term maintainability without scope change
- Addresses architectural debt proactively

## Detailed Implementation Plan

### Target Architecture

**Domain Layer (Pure DDD):**
- **Property:** Entity with hash-based ID for uniqueness and PropertyBank membership checking
- **PropertySpec:** Value objects (unchanged)
- **Schema:** Aggregate (minimal changes)
- **PropertyBank:** Aggregate (unchanged)

**Service Layer (Orchestration):**
- **SchemaEngine:** Clean orchestrator service accessing adapter layer

**Adapter Layer (Infrastructure):**
- **PropertyDereferencer:** Handle $ref replacement with PropertyBank lookups
- **SchemaExtender:** Handle extends/excludes inheritance logic
- **SchemaValidator:** JSON file structure validation
- **SchemaRegistry:** Schema storage and lookup (moved from service layer)

### Property ID Strategy: Hash-Based Recommendation

**Selected Approach:** Hash-based IDs for Property entities

**Rationale:**
- ✅ **Reproducible:** Same property definition = same ID
- ✅ **Deterministic:** Enables reliable Property.InPropertyBank() checking
- ✅ **Content-based:** Hash of (Name + Spec content) ensures uniqueness
- ✅ **Simple:** No external ID generation service needed
- ✅ **Testable:** Predictable IDs for unit testing

**Implementation:** `hash := sha256.Sum256([]byte(name + specContent))`

### Domain Layer Changes (Minimal)

**File Modifications:** ONLY `internal/domain/property.go` and `internal/domain/schema.go`

**internal/domain/property.go Changes:**
- ✅ **Add Property.ID field** with hash-based generation
- ✅ **Add Property.InPropertyBank() method** for membership checking
- ✅ **Remove PropertyRef model** from domain (becomes infrastructure-only)
- ✅ **Remove IProperty interface** (no longer needed with proper layer separation)
- ✅ **Update constructors** to generate IDs automatically

**internal/domain/schema.go Changes:**
- ✅ **Minimal modifications** to support Property entity pattern
- ✅ **Preserve existing validation logic**
- ✅ **Maintain aggregate pattern** with defensive copying

### Infrastructure Layer Reorganization

**New File Structure:**
```
internal/
  domain/                          # Pure DDD domain
    property.go                    # Property entity with ID (MODIFIED)
    schema.go                      # Schema aggregate (MODIFIED)
    property_spec.go               # PropertySpec value objects (UNCHANGED)
    property_bank.go               # PropertyBank aggregate (UNCHANGED)
  app/schema/                      # Service layer
    engine.go                      # SchemaEngine service only
  adapter/spi/schema/              # Infrastructure adapters
    dereferencer.go                # PropertyDereferencer (NEW - from SchemaResolver split)
    extender.go                    # SchemaExtender (NEW - from SchemaResolver split)
    validator.go                   # SchemaValidator (MOVED from app/schema/)
    registry.go                    # SchemaRegistry (MOVED from app/schema/)
    dto/                           # Existing propertyRefDTO (UNCHANGED)
```

**Component Responsibilities:**

**PropertyDereferencer** (`internal/adapter/spi/schema/dereferencer.go`):
- Handle $ref replacement with PropertyBank property lookups
- Pure infrastructure concern - JSON pointer resolution
- Error on missing $ref targets
- One-to-one mapping validation with PropertyBank

**SchemaExtender** (`internal/adapter/spi/schema/extender.go`):
- Handle extends/excludes inheritance attribute processing
- Topological sorting for inheritance chains
- Cycle detection with informative error paths
- Property merge semantics (complete override by name)

**SchemaValidator** (`internal/adapter/spi/schema/validator.go`):
- JSON file structure validation
- Cross-schema reference validation
- Duplicate name detection
- Infrastructure-level constraint checking

### Story Refactoring Sequence

**Phase 1: Story 2.2 Refactoring - DDD Domain Models**

**Status Change:** Done → In Progress

**New Acceptance Criteria:**
- 2.2.29: Transform Property to DDD entity with hash-based ID generation
- 2.2.30: Add Property.InPropertyBank() method for membership checking
- 2.2.31: Remove PropertyRef model from domain layer
- 2.2.32: Remove IProperty interface (no longer needed)
- 2.2.33: Update Property constructor to auto-generate IDs
- 2.2.34: Maintain all existing PropertySpec value objects unchanged
- 2.2.35: Update unit tests for entity semantics with ID-based equality

**Tasks:**
- [ ] RED: Write failing tests for Property entity with ID
- [ ] GREEN: Implement hash-based ID generation
- [ ] RED: Write failing tests for InPropertyBank() method
- [ ] GREEN: Implement PropertyBank membership checking
- [ ] REFACTOR: Remove PropertyRef and IProperty interface
- [ ] REFACTOR: Update all tests for entity pattern

**Phase 2: Story 2.6 Refactoring - Move SchemaValidator to Adapter**

**Status Change:** Done → In Progress

**New Acceptance Criteria:**
- 2.6.25: Move SchemaValidator from `internal/app/schema/` to `internal/adapter/spi/schema/validator.go`
- 2.6.26: Update all imports across codebase for new location
- 2.6.27: Verify SchemaValidator remains pure infrastructure logic
- 2.6.28: Maintain all existing comprehensive test coverage
- 2.6.29: Update architecture documentation for layer assignment

**Tasks:**
- [ ] RED: Write failing tests expecting SchemaValidator in adapter layer
- [ ] GREEN: Move validator.go to adapter/spi/schema/ location
- [ ] GREEN: Update all import statements across codebase
- [ ] REFACTOR: Verify no domain concerns leaked into validator
- [ ] REFACTOR: Update documentation references

**Phase 3: Story 2.7 Refactoring - Split SchemaResolver into Infrastructure Components**

**Status Change:** Done → In Progress

**New Acceptance Criteria:**
- 2.7.29: Move SchemaResolver from service layer to adapter layer
- 2.7.30: Split SchemaResolver into PropertyDereferencer component at `internal/adapter/spi/schema/dereferencer.go`
- 2.7.31: Split SchemaResolver into SchemaExtender component at `internal/adapter/spi/schema/extender.go`
- 2.7.32: PropertyDereferencer handles $ref replacement with PropertyBank lookups
- 2.7.33: SchemaExtender handles extends/excludes inheritance processing
- 2.7.34: Maintain all existing inheritance resolution algorithms
- 2.7.35: Preserve all comprehensive test coverage (96%+ coverage maintained)
- 2.7.36: Update SchemaEngine to use separated components

**Tasks:**
- [ ] RED: Write failing tests for PropertyDereferencer component
- [ ] GREEN: Extract $ref substitution logic to PropertyDereferencer
- [ ] RED: Write failing tests for SchemaExtender component
- [ ] GREEN: Extract inheritance resolution logic to SchemaExtender
- [ ] GREEN: Update SchemaEngine orchestration for separated components
- [ ] REFACTOR: Remove original SchemaResolver from service layer
- [ ] REFACTOR: Verify test coverage maintained

**Phase 4: New Story - JSON Schema Reference Deliverable**

**Story:** Create JSON Schema Reference Documentation

**Acceptance Criteria:**
- Create formal JSON schema file for property and schema definitions
- Document schema structure for development reference
- Enable future user-facing schema documentation
- Provide validation reference for schema files
- Integrate with development tooling

**Tasks:**
- [ ] Design JSON schema structure for Property and PropertySpec models
- [ ] Create schema file with formal validation rules
- [ ] Document schema usage in architecture documentation
- [ ] Add schema validation to development workflow
- [ ] Update user documentation with schema reference

### PropertyBank Loading Priority Specification

**Requirement:** PropertyBank file must always load first with complete one-to-one mapping

**Implementation Details:**
- SchemaEngine ensures PropertyBank.Load() completes before schema processing
- Every $ref must have exact match in PropertyBank.Properties[name]
- Object key used for Name attribute provides the mapping reference
- Validation fails if any $ref lacks PropertyBank match
- Loading order enforced in SchemaEngine orchestration

### Risk Mitigation & Dependencies

**Epic 3 Protection:**
- ✅ **SchemaEngine Interface Stability:** All Epic 3 dependencies preserved
- ✅ **FrontmatterService Isolation:** Only uses SchemaEngine, protected from internal changes
- ✅ **No Development Pause:** Epic 3 can continue concurrently

**Quality Preservation:**
- ✅ **Test Coverage:** All existing comprehensive test suites maintained
- ✅ **Algorithm Preservation:** Inheritance resolution and cycle detection algorithms unchanged
- ✅ **Performance:** No performance degradation from architectural improvements

**Implementation Risks:**
- **Low Risk:** Infrastructure concerns clearly separated from domain logic
- **Mitigation:** TDD approach ensures no functionality regression
- **Rollback:** Existing implementation preserved until refactoring validated

### Success Criteria

**Technical Success Criteria:**
- ✅ Property entity with hash-based ID functional
- ✅ Property.InPropertyBank() method working correctly
- ✅ PropertyRef removed from domain layer
- ✅ Infrastructure components properly separated
- ✅ All existing tests passing with maintained coverage
- ✅ SchemaEngine interface unchanged
- ✅ Epic 3 development unaffected

**Architectural Success Criteria:**
- ✅ Clean DDD boundaries between domain/service/adapter layers
- ✅ Infrastructure concerns isolated in adapter layer
- ✅ Domain models follow proper entity/aggregate/value object patterns
- ✅ JSON schema reference deliverable completed
- ✅ Documentation updated for new architecture

**Quality Success Criteria:**
- ✅ No regression in functionality or performance
- ✅ Test coverage maintained at existing high levels (96%+)
- ✅ Code quality standards preserved
- ✅ All linting and quality gates passing

## Agent Handoff Plan

### Immediate Next Steps

**1. Dev Agent Handoff:**
- **Task:** Begin Story 2.2 refactoring with DDD domain model transformation
- **Approach:** Follow TDD methodology with RED-GREEN-REFACTOR cycles
- **Focus:** Property entity with hash-based ID and PropertyRef removal
- **Constraint:** ONLY modify `internal/domain/property.go` and `internal/domain/schema.go`

**2. PO Validation:**
- **Task:** Validate DDD compliance during development
- **Checkpoint:** Review Property entity implementation against DDD principles
- **Quality Gate:** Ensure domain layer changes remain minimal and focused

**3. QA Review:**
- **Task:** Verify architectural alignment and test coverage preservation
- **Focus:** Confirm infrastructure layer separation properly implemented
- **Validation:** All existing test coverage maintained

### Long-term Coordination

**Architecture Documentation:**
- Update docs/architecture/data-models.md for DDD model changes
- Update docs/architecture/components.md for layer reassignments
- Update docs/architecture/source-tree.md for new file structure

**Epic 3 Monitoring:**
- Monitor Epic 3 development for any SchemaEngine interface issues
- Ensure FrontmatterService implementation proceeds without obstruction
- Validate that architectural changes enable rather than hinder Epic 3

## Final Review & Approval

### Change Checklist Completion

- [x] **Trigger Analysis:** Architectural misalignment clearly identified and documented
- [x] **Epic Impact:** All affected stories and future epics analyzed
- [x] **Artifact Impact:** Architecture documentation and story updates specified
- [x] **Path Evaluation:** Direct adjustment selected with clear rationale
- [x] **Implementation Plan:** Detailed refactoring sequence with acceptance criteria
- [x] **Risk Mitigation:** Epic 3 protection and quality preservation addressed
- [x] **Success Criteria:** Clear technical and architectural goals defined
- [x] **Handoff Plan:** Agent roles and responsibilities specified

### Approval Status

**✅ APPROVED** by Product Owner on October 31, 2025

**Approval Rationale:**
- Addresses architectural technical debt proactively
- Preserves all high-quality existing implementations
- Protects Epic 3 development through interface stability
- Follows proper DDD principles for sustainable architecture
- Clear implementation plan with defined success criteria

### Change Implementation Authorization

**Authorized Actions:**
- ✅ Update Stories 2.2, 2.6, 2.7 status from Done to In Progress
- ✅ Add new acceptance criteria and tasks to affected stories
- ✅ Begin domain layer refactoring with Property entity transformation
- ✅ Move infrastructure components to adapter layer
- ✅ Create JSON schema reference deliverable
- ✅ Update architecture documentation for new layer structure

**Implementation Start:** Immediate - Dev Agent authorized to begin Story 2.2 refactoring

**Next Review:** After Story 2.2 completion to validate DDD implementation before proceeding to Stories 2.6 and 2.7

---

**Document Status:** FINAL
**Implementation Status:** APPROVED - Ready for Dev Agent execution
**Epic Impact:** Epic 2 refactoring, Epic 3+ protected
**Timeline:** 2-3 story cycles with TDD approach
