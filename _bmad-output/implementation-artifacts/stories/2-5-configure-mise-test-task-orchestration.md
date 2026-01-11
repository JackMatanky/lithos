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

- [ ] Analyze existing mise task infrastructure **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Review existing mise.toml configuration and task definitions
  - [ ] Examine .mise/tasks/ directory structure and existing scripts
  - [ ] Identify gaps between current setup and required test orchestration
  - [ ] Assess existing test scripts (unit.sh, integration.sh, coverage.sh, watch.sh)
- [ ] Configure test task orchestration in mise.toml **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Set up `test` meta-task with unit and integration dependencies
  - [ ] Configure `test:unit` to execute .mise/tasks/test/unit.sh
  - [ ] Configure `test:integration` to execute .mise/tasks/test/integration.sh
  - [ ] Implement `test:coverage` to run .mise/tasks/test/coverage.sh
  - [ ] Set up `test:watch` for TDD workflow using existing watch.sh script
- [ ] Optimize task dependencies and parallelization **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Configure task dependency graphs for optimal parallel execution
  - [ ] Ensure proper task ordering (unit before integration, etc.)
  - [ ] Implement task hiding for internal scripts
  - [ ] Add task aliases for developer convenience
- [ ] Enhance development workflow orchestration **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Improve existing `lint` and `fmt` tasks with better orchestration
  - [ ] Configure `verify` meta-task for comprehensive quality gates
  - [ ] Enhance `ci` task for better CI/CD pipeline simulation
  - [ ] Add performance monitoring to task execution
- [ ] Integrate with ADR 0010 test utilities **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Configure environment variables for ADR 0010 utility integration
  - [ ] Update tasks to leverage centralized test utilities and fixtures
  - [ ] Ensure proper test output directory management in orchestration
  - [ ] Validate task compatibility with ADR 0010 patterns
- [ ] Document and validate orchestration **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Update documentation for enhanced task orchestration
  - [ ] Validate task execution across different environments
  - [ ] Test parallel execution and dependency management
  - [ ] Establish procedures for future task orchestration updates

## Dev Notes

- **ADR 0011 Orchestration Focus**: Configure mise task orchestration to optimize existing scripts, not create new ones - leverage .mise/tasks/test/ infrastructure.

- **Architecture Compliance**: Tasks support hexagonal architecture testing with proper parallelization, dependency management, and environment consistency.

- **Implementation Priority**: Analyze existing setup (Priority 1), configure orchestration (Priority 2), optimize dependencies (Priority 3), enhance workflows (Priority 4), integrate utilities (Priority 5), document (Priority 6).

- **Source Tree Components**: mise.toml orchestration layer, .mise/tasks/ execution scripts, docs/development/tasks.md documentation.

- **Quality Assurance**: Orchestration includes validation, parallel execution testing, and CI/CD simulation for reliable development workflows.

### Project Structure Notes

- **Alignment with unified project structure**: mise.toml follows established configuration patterns and integrates with existing CI/CD workflows.

- **Detected conflicts or variances**: None - extends existing mise configuration with comprehensive test task orchestration.

### Technical Requirements

**Task Orchestration Configuration:**
- test: Meta-task combining unit and integration with parallel execution
- test:unit: Execute .mise/tasks/test/unit.sh for domain layer isolation
- test:integration: Execute .mise/tasks/test/integration.sh for cross-crate testing
- test:coverage: Run .mise/tasks/test/coverage.sh with HTML reports and CI/CD integration
- test:watch: Execute .mise/tasks/test/watch.sh for TDD with automatic re-running

**Enhanced Development Workflow:**
- verify: Meta-task combining fmt, lint, test, deny with optimized parallelization
- lint: Enhanced execution of .mise/tasks/lint.sh with comprehensive checks
- fmt: Improved execution of .mise/tasks/fmt.sh with consistent formatting
- ci: Complete CI/CD pipeline simulation using existing scripts

**Advanced Orchestration Features:**
- DAG-based dependency graph for optimal parallel execution
- Task hiding for internal scripts and aliases for developer convenience
- Environment variable configuration for ADR 0010 utility integration
- Performance monitoring and execution time tracking for optimization

### File Structure Requirements

- mise.toml in project root with comprehensive task definitions
- Task documentation in docs/development/tasks.md with examples and usage
- CI/CD configuration in .github/workflows/ with mise task integration
- Environment configuration files for different test scenarios
- Performance benchmark results in target/criterion/ for tracking

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

- [ADR 0011: Mise Task Orchestration](docs/adr/0011-mise-task-orchestration.md) - Comprehensive mise task orchestration framework and patterns
- [ADR 0010: Centralized Test Utilities](docs/adr/0010-centralized-test-utilities.md) - Test utilities framework integration
- [ADR 0009: CQRS Testing Patterns](docs/adr/0009-cqrs-testing-patterns.md) - CQRS testing patterns
- [Research: Mise Task Orchestration - https://mise.jdx.dev/tasks/]
- [Research: Parallel Task Execution - https://mise.jdx.dev/tasks/architecture.html]

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
- Execution Optimization: Follow ADR 0011 orchestration patterns for optimal parallelization and existing script integration
