# Documentation Index

## Root Documents

### [Elicitation Summary](../_bmad-output/planning-artifacts/discovery/elicitation_summary.md)

This document provides a comprehensive summary of the key findings, discussions, and decisions made during the advanced elicitation sessions for the Lithos project.

### [Project Brief](../_bmad-output/planning-artifacts/discovery/project_brief.md)

This document outlines the project vision, problem statement, proposed solution, target users, goals, and scope for the Lithos CLI tool.

### [Product Requirements (PRD)](../_bmad-output/planning-artifacts/prd.md)

The definitive product requirements document for the Lithos project.

### [Architecture Document](../_bmad-output/planning-artifacts/architecture.md)

The core architectural definition and decision records for the system.

## ADRs (Architectural Decision Records)

| ADR | Title |
|---|---|
| [ADR 0001](../docs/adr/0001-adr-process.md) | ADR Process |
| [ADR 0002](../docs/adr/0002-persistence-cache-infrastructure.md) | Persistence & Cache Infrastructure |
| [ADR 0003](../docs/adr/0003-template-engine.md) | Template Engine - MiniJinja |
| [ADR 0004](../docs/adr/0004-markdown-parsing.md) | Markdown Parsing - pulldown-cmark |
| [ADR 0005](../docs/adr/0005-configuration-management.md) | Configuration Management |
| [ADR 0006](../docs/adr/0006-error-handling-diagnostics.md) | Error Handling & Diagnostics |
| [ADR 0007](../docs/adr/0007-event-orchestration.md) | Event Orchestration (Minimal Foundation) |
| [ADR 0008](../docs/adr/0008-benchmarking-infrastructure.md) | Benchmarking Infrastructure |
| [ADR 0009](../docs/adr/0009-domain-serialization-strategy.md) | Domain Serialization (Feature-Gated) |
| [ADR 0010](../docs/adr/0010-rename-detection-strategy.md) | Rename Detection Strategy |
| [ADR 0011](../docs/adr/0011-file-loading-port-boundary.md) | File Loading Port Boundary |
| [ADR 0012](../docs/adr/0012-caching-strategy.md) | Caching Strategy (Superseded) |

## Testing

Developer-focused guides and patterns for testing in Lithos.

### [Lithos Test Guide](./test_guide.md)

Comprehensive reference for testing standards, patterns, and tools.

### [Async Testing Guidelines](./testing/async.md)

Patterns and best practices for testing asynchronous code with Tokio.

### [Event-Driven Testing Patterns](./testing/event.md)

Testing patterns for the hybrid event bus and CQRS.

### [CQRS Testing Patterns](./testing/cqrs.md)

Detailed patterns for testing command and query handlers.

## Epics

| Epic | Title |
|---|---|
| [Epic 1](../_bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md) | Development Environment & Tooling |
| [Epic 2](../_bmad-output/planning-artifacts/epics/epic-2-test-architecture-patterns-utilities-mvp-core.md) | Test Architecture & Utilities |
| [Epic 3](../_bmad-output/planning-artifacts/epics/epic-3-core-domain-models-value-objects-phase-15.md) | Core Domain Models |
| [Epic 4](../_bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md) | File Loading Strategy |

## Implementation Stories

See the [Sprint Status](../_bmad-output/implementation-artifacts/sprint-status.yaml) for current progress and individual story files in `_bmad-output/implementation-artifacts/stories/`.
