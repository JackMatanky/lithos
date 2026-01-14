# Test Coverage Target Analysis: 80%+ vs 90%+

## Issue
Epic 3 domain models require 90%+ test coverage, while the project standard (per @docs/testing/developer-guide.md) is 80%+. This creates pressure to write tests primarily to hit coverage numbers rather than focusing on testing business logic and edge cases.

## Current State
- **Testing Guide Target**: 80%+ coverage
- **Epic 3 Target**: 90%+ coverage for domain entities
- **Justification Document**: Missing (`_bmad-output/coverage-analysis/coverage-target-justification.md`)

## Analysis

### Arguments For 90%+ Target (Domain Criticality)
- Domain models contain critical business logic and validation rules
- Type safety and error handling are business-critical
- High coverage ensures robustness of core domain invariants
- Domain entities are foundational - bugs here affect entire system

### Arguments For 80%+ Target (Standard Alignment)
- Coverage metrics can drive poor testing practices
- Focus should be on testing business requirements, not lines of code
- 80%+ covers happy paths, error cases, and edge cases adequately
- Additional coverage often tests trivial code (getters/setters, simple constructors)

### Quality Over Quantity
The real issue isn't the percentage, but ensuring tests validate:
- Business requirements and invariants
- Error conditions and edge cases
- Integration between components
- Performance characteristics
- Maintainability and refactoring safety

## Recommendation

**Option A: Justify and Keep 90%+**
- Create the missing justification document
- Focus on testing business logic depth over line coverage
- Accept that some boilerplate code may remain untested

**Option B: Align with 80%+ Standard**
- Update Epic 3 stories to use 80%+ target
- Remove coverage-driven pressure
- Focus on comprehensive business logic testing

**Option C: Hybrid Approach**
- 80%+ overall coverage target
- Specific high-coverage requirements for critical business logic only
- Document coverage gaps and why they're acceptable

## Action Required
The missing justification document should be created to explain why Epic 3 domain models warrant a higher coverage target than the project standard. Without this rationale, the 90%+ requirement risks promoting coverage-driven testing over quality-driven testing.
