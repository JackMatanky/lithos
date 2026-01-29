# Lithos Rust - AI Agent Reference

## Critical Files - READ FIRST

**MUST** review these files before starting any work:

- **Project Context**: [_bmad-output/project-context.md](_bmad-output/project-context.md) - Core rules and patterns
- **Workflow Status**: [_bmad-output/planning-artifacts/bmm-workflow-status.yaml](_bmad-output/planning-artifacts/bmm-workflow-status.yaml) - Current phase and next steps
- **Architecture**: [_bmad-output/planning-artifacts/architecture/](_bmad-output/planning-artifacts/architecture/) - System design decisions
- **PRD**: [_bmad-output/planning-artifacts/prd.md](_bmad-output/planning-artifacts/prd.md) - Product requirements

## BMAD Agent Activation

To activate specialized agents, use: `"As [agent-name], ..."` (e.g., `"As dev, implement the cache service"`)

**Available agents**: See [_bmad/_config/agent-manifest.csv](_bmad/_config/agent-manifest.csv) for full list
- **dev** - Implementation, debugging, refactoring
- **architect** - System design, ADRs, tech selection
- **tea** - Test strategy, quality gates
- **quick-flow-solo-dev** - Rapid prototyping
- **bmad-master** - General orchestration

**Available workflows**: See [_bmad/_config/workflow-manifest.csv](_bmad/_config/workflow-manifest.csv)

## Project-Specific Context

### Technology Stack
- **Language**: Rust (latest stable)
- **Architecture**: Hexagonal (Ports & Adapters) - domain isolated from infrastructure
- **Key Libraries**: redb (zero-copy DB), moka (concurrent cache), rkyv (serialization)
- **Testing**: nextest, criterion benchmarks, tarpaulin coverage
- **Build**: cargo workspace with mise task orchestration

### Critical Coding Standards
- **Zero-copy patterns** for performance-critical paths (see [Technical Reference](#technical-reference-documentation))
- **Hexagonal architecture**: Domain crate has no external dependencies
- **Test-first development**: Red-green-refactor cycle required
- **ADR documentation**: All architectural decisions documented in [docs/adr/](docs/adr/)

### Project Structure
- `crates/domain/` - Pure business logic (no I/O, no external deps)
- `crates/app/` - Application services and use cases
- `crates/adapters/` - Infrastructure (DB, file system, cache)
- `crates/cli/` - Command-line interface
- `tests/` - Integration and E2E tests
- `benches/` - Performance benchmarks

For complete rules, see [_bmad-output/project-context.md](_bmad-output/project-context.md)

## Common Commands (mise tasks)

| Command                      | Action                                                                            |
| :--------------------------- | :-------------------------------------------------------------------------------- |
| `mise run verify`            | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |
| `mise run quality`           | Run all quality gates (fmt, lint, adr:validate) (alias: `q`).                     |
| `mise run lint`              | Run linting checks using clippy.                                                  |
| `mise run fmt`               | Format code using rustfmt.                                                        |
| `mise run deny`              | Check dependencies for security and license issues.                               |
| `mise run clean`             | Clean build artifacts and temporary files.                                        |
| `mise run clean:cargo`       | Clean only cargo build artifacts.                                                 |
| `mise run clean:test`        | Clean only test output artifacts.                                                 |
| `mise run clean:reports`     | Clean only coverage and JUnit reports.                                            |
| `mise run build`             | Build the project binaries.                                                       |
| `mise run doc`               | Generate and open project documentation.                                          |
| `mise run dev-setup`         | Set up development environment and dependencies.                                  |
| `mise run adr:validate`      | Validate ADR files for compliance.                                                |
| `mise run adr:metrics`       | Generate metrics for ADR management.                                              |
| `mise run ci`                | Simulate CI/CD pipeline.                                                          |
| `mise run timing`            | Run verify with detailed timing information.                                      |
| `mise run test`              | Run all unit and integration tests (alias: `t`).                                  |
| `mise run test:unit`         | Run all unit tests across the workspace using `nextest`.                          |
| `mise run test:unit:<crate>` | Run unit tests for a specific crate (e.g., `test:unit:app`).                      |
| `mise run test:unit:domain`  | Run domain crate unit tests (alias: `tud`).                                       |
| `mise run test:unit:app`     | Run app crate unit tests (alias: `tuap`).                                         |
| `mise run test:unit:adapters`| Run adapters crate unit tests (alias: `tuad`).                                    |
| `mise run test:unit:cli`     | Run CLI crate unit tests (alias: `tuc`).                                          |
| `mise run test:bench`        | Run all performance benchmarks using `criterion`.                                 |
| `mise run test:bench:domain` | Run domain crate benchmarks (alias: `tbd`).                                       |
| `mise run test:bench:app`    | Run app crate benchmarks (alias: `tbap`).                                         |
| `mise run test:bench:adapters`| Run adapters crate benchmarks (alias: `tbad`).                                   |
| `mise run test:bench:cli`    | Run CLI crate benchmarks (alias: `tbc`).                                          |
| `mise run test:integration`  | Run all integration tests across the workspace.                                   |
| `mise run test:e2e`          | Run end-to-end tests using `cli_smoke` binary.                                    |
| `mise run test:arch`         | Run architectural enforcement tests using `purity` binary.                        |
| `mise run test:coverage`     | Generate code coverage reports using `tarpaulin`.                                 |
| `mise run test:watch`        | Watch mode: automatically run tests on file changes.                              |

## Glossary of Terms

- **BMAD Method**: A structured approach to software development using specialized AI agents and workflows for efficient task execution.
- **Hexagonal Architecture**: A design pattern separating core business logic from external interfaces, ensuring testability and flexibility.
- **Session ID**: A unique identifier used to maintain state across multiple agent invocations.
- **Trimodal Workflows**: Workflows that handle creation, editing, and validation in a single framework.
- **Task Orchestration**: The process of coordinating multiple agents and tools to complete complex tasks.
- **Agent Persona**: The defined role, communication style, and capabilities of a BMAD agent.

## Troubleshooting and FAQ

### Common Issues
- **Agent Activation Fails**: Ensure the agent name matches exactly (case-sensitive). Check for typos in "As [agent-name]".
- **Workflow Times Out**: Provide more specific prompts or break tasks into smaller steps. Use session_ids for stateful workflows.
- **Mise Command Errors**: Verify mise.toml for correct tool versions. Run `mise run dev-setup` to install dependencies.
- **Session Continuity Lost**: Always pass session_id in chained calls. Agents are stateless by default.

### Frequently Asked Questions
- **Q: Which agent should I use for code review?** A: Use `tea` for test architecture or `dev` for general coding tasks.
- **Q: How do I add a new workflow?** A: Use `workflow-builder` agent to create and validate it.
- **Q: What if no agent matches my task?** A: Start with `bmad-master` for orchestration guidance.
- **Q: How to handle agent errors?** A: Check prompts for clarity; retry with more context or switch agents.

## Technical Reference Documentation

Performance-critical library references for zero-copy and high-performance systems:

- [redb Reference](./docs/refs/redb-reference.md) - Zero-copy embedded database with MVCC and ACID transactions
- [moka Reference](./docs/refs/moka-reference.md) - High-performance concurrent cache with TinyLFU eviction
- [rkyv Reference](./docs/refs/rkyv-reference.md) - Zero-copy serialization framework with validation
- [Lithos Integration Guide](./docs/refs/lithos-integration-guide.md) - Integration patterns combining all three libraries

**Quick Reference:**
- Zero-copy persistent storage → redb
- In-memory concurrent caching → moka
- Zero-copy serialization format → rkyv
- Combined architecture patterns → Integration Guide

## MCP Servers

When you need to search docs, use `context7` tools.
Key Architectural Constraints

⚠️ **NON-NEGOTIABLE RULES**:
1. **Domain purity**: `crates/domain/` MUST have zero external dependencies
2. **Zero-copy patterns**: Use rkyv for serialization, avoid cloning in hot paths
3. **Test-first**: Red-green-refactor cycle required - tests before implementation
4. **ADRs required**: Document all architectural decisions in [docs/adr/](docs/adr/)
5. **Hexagonal architecture**: Domain → App → Adapters (dependencies flow inward only)
