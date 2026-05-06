# Documentation Index

## Root Documents

### [Elicitation Summary](./project-genesis/elicitation_summary.md)

This document provides a comprehensive summary of the key findings, discussions, and decisions made during the advanced elicitation sessions for the Lithos project.

### [Project Brief](./project-genesis/project_brief.md)

This document outlines the project vision, problem statement, proposed solution, target users, goals, and scope for the Lithos CLI tool.

### [Product Requirements (PRD)](../_bmad-output/planning-artifacts/prd.md)

The definitive product requirements document for the Lithos project.

### [Architecture Document](../_bmad-output/planning-artifacts/architecture.md)

The core architectural definition and decision records for the system.

## ADRs (Architectural Decision Records)

| ADR | Title |
|---|---|
| [ADR 001](../docs/adr/001-adr-process.md) | ADR Process |
| [ADR 006](../docs/adr/006-persistence-cache-infrastructure.md) | Persistence & Cache Infrastructure |
| [ADR 007](../docs/adr/007-template-engine.md) | Template Engine - MiniJinja |
| [ADR 008](../docs/adr/008-markdown-parsing.md) | Markdown Parsing - pulldown-cmark |
| [ADR 009](../docs/adr/009-configuration-management.md) | Configuration Management |
| [ADR 005](../docs/adr/005-error-handling.md) | Error Handling & Diagnostics |
| [ADR 004](../docs/adr/004-event-orchestration.md) | Event Orchestration (Minimal Foundation) |
| [ADR 012](../docs/adr/012-benchmarking-infrastructure.md) | Benchmarking Infrastructure |
| [ADR 003](../docs/adr/003-domain-serialization.md) | Domain Serialization (Feature-Gated) |
| [ADR 011](../docs/adr/011-rename-detection.md) | Rename Detection Strategy |
| [ADR 010](../docs/adr/010-file-loading-port-boundary.md) | File Loading Port Boundary |
| [ADR 013](../docs/adr/013-caching-strategy.md) | Caching Strategy (Superseded) |

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
