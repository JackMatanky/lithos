# Mise Task Orchestration Guide

This guide provides best practices for configuring mise task orchestration in Lithos, building on the existing `.mise/tasks/` infrastructure established by previous stories.

## Overview

Mise provides sophisticated task orchestration with dependency graph resolution, parallel execution, and watch capabilities. This guide focuses on configuring `mise.toml` to optimize existing task scripts for maximum development efficiency.

## Key Orchestration Concepts

### Task Dependency Graph
Mise uses a directed acyclic graph (DAG) to manage task execution order and parallelism:
- Dependencies run before dependents
- Independent tasks execute simultaneously
- No circular dependencies allowed
- Critical path optimization for fastest execution

### Parallel Execution
- Tasks without dependencies run in parallel by default
- Mix scripts with task references in complex workflows
- Watch mode for automatic re-execution on file changes
- Environment consistency across all task executions

## Test Task Orchestration

### Meta-Task Configuration
Configure high-level test orchestration that leverages existing scripts:

```toml
[tasks.test]
description = "Run all tests (unit and integration)"
depends = ["test:unit", "test:integration"]
```

### Individual Test Tasks
Map existing scripts to mise task definitions:

```toml
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
```

## Quality Gate Orchestration

### Comprehensive Verification
Combine all quality checks with optimized parallelization:

```toml
[tasks.verify]
description = "Full quality gate orchestration"
depends = ["fmt", "lint", "test", "deny"]
```

### Parallel Quality Checks
Run formatting and linting in parallel with tests:

```toml
[tasks.fmt]
description = "Format all code"
run = ".mise/tasks/fmt.sh"

[tasks.lint]
description = "Run clippy linting"
run = ".mise/tasks/lint.sh"

[tasks.deny]
description = "Check dependencies for security"
run = "cargo deny check"
```

## Development Workflow Enhancement

### Watch Mode Integration
Enable efficient TDD workflows:

```toml
[tasks.watch:test]
description = "Watch mode for test development"
run = ".mise/tasks/test/watch.sh"
```

### Alias Configuration
Provide convenient shortcuts:

```toml
[tasks.t]
description = "Quick test run"
alias = "t"
run = "mise run test"
```

### Task Organization
Hide internal tasks and organize by purpose:

```toml
[tasks._setup]
description = "Internal setup task"
hide = true
run = ".mise/tasks/setup.sh"
```

## CI/CD Simulation

### Local Pipeline Testing
Simulate complete CI/CD workflows locally:

```toml
[tasks.ci]
description = "Simulate CI/CD pipeline"
depends = ["verify", "test:integration", "build"]
```

### Environment Consistency
Ensure local development matches CI/CD:

```toml
[tasks.ci:check]
description = "Validate CI environment"
run = "mise exec -- echo 'CI environment ready'"
```

## Performance Optimization

### Execution Monitoring
Track task performance for optimization:

```toml
[tasks.bench:tasks]
description = "Benchmark task execution"
run = "mise run --timing test"
```

### Resource Management
Monitor resource usage in development:

```toml
[tasks.usage]
description = "Show current mise resource usage"
run = "mise usage"
```

## Best Practices

### Task Design Principles
- Keep tasks focused on single responsibilities
- Use descriptive names and descriptions
- Leverage dependencies for complex workflows
- Prefer parallel execution when possible

### Script Integration
- Leverage existing `.mise/tasks/` scripts
- Maintain script portability across environments
- Use mise environment variables in scripts
- Document script dependencies and requirements

### Error Handling
- Configure proper error propagation
- Use task dependencies to prevent invalid states
- Implement graceful failure handling
- Provide clear error messages for debugging

### Maintenance
- Regularly review and refactor task dependencies
- Update scripts to leverage new mise features
- Document complex orchestration patterns
- Monitor task execution performance

## Troubleshooting

### Common Issues
- **Circular Dependencies**: Check for cycles in task graphs
- **Environment Differences**: Ensure consistent mise versions
- **Resource Conflicts**: Monitor parallel execution conflicts
- **Watch Mode Issues**: Verify file change detection paths

### Debugging Commands
```bash
# Show task dependency graph
mise run --dry-run verify

# Time task execution
mise run --timing test

# Debug environment variables
mise run --env test
```

## Integration with Lithos Architecture

### Hexagonal Architecture Support
- Tasks support port/adapter testing patterns
- Environment consistency across architectural layers
- Parallel testing of independent components

### CQRS Workflow Integration
- Separate test tasks for command and query paths
- Eventual consistency testing orchestration
- Cross-aggregate workflow validation

### Async-First Compatibility
- Tokio-aware task execution
- Concurrent testing with proper isolation
- Performance monitoring for async operations

This guide provides the foundation for efficient mise task orchestration in Lithos, maximizing development productivity while maintaining code quality and environment consistency.
