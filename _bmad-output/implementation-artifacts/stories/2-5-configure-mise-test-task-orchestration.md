# Story 2.5: configure-mise-test-task-orchestration

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer running tests during development,
I want optimized mise task orchestration for existing test scripts,
So that I can efficiently run tests, check coverage, and maintain code quality during development.

## Acceptance Criteria

**Test Task Orchestration:**
- **Given** existing test scripts in .mise/tasks/test/ (unit.sh, integration.sh, coverage.sh, watch.sh)
- **When** configuring mise.toml orchestration
- **Then** tasks properly orchestrate existing scripts with dependencies, parallelization, and optimal execution

**Meta-Task Configuration:**
- **Given** mise test task orchestration is configured
- **When** running `mise run test`
- **Then** executes unit and integration tests in parallel with proper dependency management and consolidated reporting

**Coverage Task Integration:**
- **Given** existing coverage.sh script in .mise/tasks/test/
- **When** configuring mise task orchestration
- **Then** coverage task provides HTML reports, integrates with CI/CD, and supports local development workflows

**Watch Mode Orchestration:**
- **Given** existing watch.sh script for TDD workflows
- **When** configuring mise task orchestration
- **Then** watch task enables efficient TDD with automatic re-running and proper file change detection

## Tasks / Subtasks

- [ ] Review Mise Task Orchestration Guide **[Effort: 1-2 hours | Complexity: Low]**
  - [ ] Read the Mise Task Orchestration Guide for best practices
  - [ ] Understand DAG-based dependency management and parallel execution
  - [ ] Review existing .mise/tasks/ scripts and their purposes
  - [ ] Identify integration points with ADR 0010 test utilities
- [ ] Configure core test task orchestration **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Set up `test` meta-task combining unit and integration tests
  - [ ] Configure `test:unit` to execute existing .mise/tasks/test/unit.sh
  - [ ] Configure `test:integration` to execute existing .mise/tasks/test/integration.sh
  - [ ] Set up `test:coverage` to run existing .mise/tasks/test/coverage.sh
  - [ ] Implement `test:watch` for TDD using existing .mise/tasks/test/watch.sh
- [ ] Optimize quality gate orchestration **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Configure `verify` meta-task for comprehensive quality gates
  - [ ] Ensure parallel execution of fmt, lint, and test tasks
  - [ ] Set up proper dependency ordering for quality checks
  - [ ] Add performance monitoring to task execution
- [ ] Enhance development workflow tasks **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Improve `lint` and `fmt` tasks with better error handling
  - [ ] Configure task aliases for developer convenience
  - [ ] Set up task hiding for internal scripts
  - [ ] Enhance `ci` task for comprehensive CI/CD simulation
- [ ] Integrate with test utilities and validate **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Configure environment variables for ADR 0010 utility integration
  - [ ] Update tasks to leverage centralized test utilities
  - [ ] Test parallel execution and dependency management
  - [ ] Validate task orchestration across different environments

## Dev Notes

- **Guide-Based Implementation**: Follow the Mise Task Orchestration Guide for optimal configuration patterns, leveraging existing .mise/tasks/test/ infrastructure.

- **Architecture Compliance**: Orchestration supports hexagonal architecture with parallel testing, consistent environments, and proper dependency management.

- **Implementation Priority**: Review guide (Priority 1), configure core tasks (Priority 2), optimize quality gates (Priority 3), enhance workflows (Priority 4), integrate utilities (Priority 5).

- **Source Tree Components**: mise.toml configuration, .mise/tasks/ scripts, Mise Task Orchestration Guide documentation.

- **Quality Assurance**: Validate orchestration with parallel execution testing, dependency verification, and CI/CD simulation.

### Project Structure Notes

- **Alignment with unified project structure**: mise.toml follows established configuration patterns and integrates with existing CI/CD workflows.

- **Detected conflicts or variances**: None - extends existing mise configuration with comprehensive test task orchestration.

### Technical Requirements

**Core Test Orchestration (Following Guide Best Practices):**
- test: Meta-task with unit and integration dependencies for parallel execution
- test:unit: Execute existing .mise/tasks/test/unit.sh with proper isolation
- test:integration: Execute existing .mise/tasks/test/integration.sh for cross-crate testing
- test:coverage: Run existing .mise/tasks/test/coverage.sh with HTML reporting
- test:watch: Execute existing .mise/tasks/test/watch.sh for efficient TDD workflows

**Quality Gate Orchestration:**
- verify: Meta-task combining fmt, lint, test, deny with optimized parallelization
- fmt: Execute existing .mise/tasks/fmt.sh with consistent formatting
- lint: Execute existing .mise/tasks/lint.sh with comprehensive code quality checks
- deny: Run cargo deny check for dependency security and license compliance

**Advanced Orchestration Features:**
- DAG-based dependency graphs ensuring correct execution order and maximum parallelism
- Task aliases and hiding for improved developer experience
- Environment variable configuration for ADR 0010 test utility integration
- Performance monitoring with execution time tracking and resource usage analysis

### File Structure Requirements

- mise.toml in project root with comprehensive task definitions
- Task documentation in docs/mise-task-orchestration.md with examples and usage
- CI/CD configuration in .github/workflows/ with mise task integration
- Environment configuration files for different test scenarios
- Performance benchmark results in target/criterion/ for tracking

### Mise.toml Configuration Examples

**Final mise.toml structure after implementation:**

```toml
# Test task orchestration (added to existing mise.toml)
[tasks.test]
description = "Run all tests (unit and integration)"
depends = ["test:unit", "test:integration"]

[tasks.test:unit]
description = "Run domain layer unit tests"
run = ".mise/tasks/test/unit.sh"

[tasks.test:integration]
description = "Run cross-crate integration tests"
run = ".mise/tasks/test/integration.sh"

[tasks.test:coverage]
description = "Generate coverage reports"
run = ".mise/tasks/test/coverage.sh"

[tasks.test:watch]
description = "TDD workflow with auto-restart"
run = ".mise/tasks/test/watch.sh"

# Enhanced quality gates
[tasks.verify]
description = "Full quality gate orchestration"
depends = ["fmt", "lint", "test", "deny"]

# CI/CD simulation
[tasks.ci]
description = "Simulate CI/CD pipeline"
depends = ["verify", "test:integration"]
```

### Testing Requirements

- Unit tests for custom mise task logic and configurations
- Integration tests validating task execution and dependencies
- Cross-platform testing for mise task compatibility
- Performance tests ensuring task execution speed and resource usage
- Documentation validation tests for task usage examples

### Previous Story Intelligence

- Story 2.4 established test utilities - orchestrate mise tasks to leverage ADR 0010 framework for optimal test execution
- Story 2.3 established CQRS testing patterns - configure tasks that support ADR 0009 command/query separation testing
- Story 2.2 established event testing infrastructure - include ADR 0008 event flow testing in orchestration
- Story 2.1 established async testing infrastructure - configure tasks with tokio compatibility and ADR 0009 patterns
- Existing .mise/tasks/ infrastructure - build orchestration layer on top of established scripts

### Git Intelligence Summary

- Recent commits show mise configuration evolution - build upon existing task patterns
- Test infrastructure development follows established conventions - maintain consistency

### References

- [Mise Task Orchestration Guide](docs/mise-task-orchestration.md) - Best practices for mise task orchestration in Lithos
- [ADR 0010: Centralized Test Utilities](docs/adr/0010-centralized-test-utilities.md) - Test utilities framework integration
- [ADR 0009: CQRS Testing Patterns](docs/adr/0009-cqrs-testing-patterns.md) - CQRS testing patterns
- [Mise Documentation - https://mise.jdx.dev/tasks/](https://mise.jdx.dev/tasks/) - Official mise task documentation

### Latest Tech Information

- Mise v2025.9.16: DAG-based task dependency resolution with automatic parallelization
- Task orchestration: Mix scripts with task references, dependency chains, and watch capabilities
- Existing scripts: Leverage .mise/tasks/test/ infrastructure (unit.sh, integration.sh, coverage.sh, watch.sh)
- Orchestration optimization: Parallel execution of independent tasks with proper ordering

### Project Context Reference

- Lithos uses mise for task orchestration with existing scripts in .mise/tasks/ and mise.toml configuration
- Previous stories established test infrastructure - this story optimizes orchestration layer
- Comprehensive test tasks enable efficient development workflows and quality maintenance
- Integration with ADR 0010/0009 test utilities provides enhanced testing capabilities
- Async-first and CQRS patterns require sophisticated task dependency management

### Story Completion Status

- Status: ready-for-dev
- All acceptance criteria defined with comprehensive mise test task orchestration requirements
- Technical requirements complete with specific cargo commands and mise configuration patterns
- Integration points identified with ADR 0010 test utilities and existing test infrastructure
- Risk assessment: Low risk, follows established mise patterns with comprehensive validation
- Execution Optimization: Follow Mise Task Orchestration Guide for optimal parallelization, dependency management, and existing script integration
