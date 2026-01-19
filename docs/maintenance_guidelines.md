# Test Maintenance Guidelines

To maintain the efficiency and quality of the Epic 3 test suite, follow these guidelines when evolving domain models or adding new features.

## 1. Maintenance Cost Monitoring
- Target: Test maintenance should consume <20% of feature development time.
- Tracking: Record time spent updating existing tests during refactors in the Dev Agent Record.
- Change Impact Analysis: Before refactoring a domain aggregate, identify all dependent tests (unit, integration, and doc-tests).

## 2. Test Quality Standards (Task 7 compliance)
- **BDD Documentation**: Every test body MUST use `// GIVEN:`, `// WHEN:`, `// THEN:` comments.
- **Strict Naming**: Use `unit_of_work` + `expected_behavior` + `state_under_test`.
- **Async Safety**: Use `#[tokio::test(flavor = "multi_thread")]` for all async tests.
- **Lint Discipline**: Use `#[expect(...)]` for intentional violations; never use `#[allow(...)]`.
- **KISS Principle**: Tests should be simpler than the code they test. Avoid complex loops or logic in test bodies.

## 3. Coverage Strategy
- Target: Maintain 80%+ line coverage for the `domain` crate.
- Focus: Prioritize business logic, validation rules, and error paths over 100% line coverage of boilerplate.
- Tools: Use `mise run test:coverage` to verify impact of changes.

## 4. Redundancy Elimination
- Use `lithos-test-utils` for shared fixtures and factories.
- Avoid duplicating test scenarios across unit and integration suites. Unit tests cover branches; integration tests cover entity interactions.

## 5. Performance Gates
- Full suite execution MUST remain <30 seconds.
- Individual unit tests should complete in <100ms.
- Use `nextest` for parallel execution.
