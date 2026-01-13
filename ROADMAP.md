# Project Roadmap - lithos

This roadmap outlines the phased development of Lithos, from its core Rust foundation to a comprehensive developer tooling platform. It is based on the [Product Requirements Document (PRD)](_bmad-output/planning-artifacts/prd.md) and [Architectural Decision Records (ADRs)](docs/adr/README.md).

## 🚀 Phase 1: MVP Core (Milestones 1-4)
**Goal:** Deliver a stable, high-performance CLI tool for interactive, schema-validated templating in large vaults.

### Milestone 1: Foundation & Domain Modeling
**Goal:** Core development environment and fundamental entities.
**Status:** Completed
- [x] Workspace structure and quality gates (Clippy, Rustfmt, Pre-commit).
- [x] ADR process established (ADRs 0001-0012).
- [x] Project Roadmap and Milestone definition (Story 1.9).
- [x] Establish async testing patterns and infrastructure (Epic 2).
- [x] Create comprehensive Developer Testing Guide.
- [ ] Implement fundamental domain models (Note, Frontmatter, Schema) - Epic 3.
**Success Metrics:** Hexagonal boundaries enforced at compile-time; 80%+ test coverage.

### Milestone 2: Persistence & Configuration
**Goal:** ACID storage and hierarchical configuration.
- [ ] Multi-strategy file loading with error recovery (Epic 4).
- [ ] Redb + rkyv persistence with zero-copy deserialization (Epic 8).
- [ ] Hierarchical configuration management - Epic 5.
**Success Metrics:** Zero-copy boundaries verified via architecture tests; < 100ms config load.

### Milestone 3: Vault Indexing & Intelligence
**Goal:** High-performance indexing and metadata query service.
- [ ] Actor-based indexing engine for incremental updates (Epic 9).
- [ ] Metadata query service with wiki-link/alias resolution (FR22, FR23) - Epic 10.
- [ ] Hybrid event bus (MPSC/Broadcast/Watch) (Epic 7).
- [ ] Schema validation engine with inheritance (FR8, FR10, FR14) - Epic 6.
**Success Metrics:** Indexing 1000+ files in < 2 seconds; < 50ms query response.

### Milestone 4: Interactive Template System
**Goal:** Core value - interactive, scriptable templating.
- [ ] MiniJinja-based template engine implementation (Epic 11).
- [ ] Interactive CLI prompts, suggesters, and multi-suggesters (FR15-FR17).
- [ ] Modular template composition (FR1, FR3).
- [ ] Clap-based CLI with subcommands and error diagnostics - Epic 13.
**Success Metrics:** Template creation time < 30 minutes; Execution < 500ms.

---

## 🛠 Phase 1.5: Core Templater Parity & Basic UX (Milestone 5)
**Goal:** Reach feature parity with essential Obsidian Templater functions while improving onboarding.

### Milestone 5: Parity & Onboarding
- [ ] Essential Templater functions (file, frontmatter, date functions) - Epic 12.
- [ ] Dynamic commands and whitespace control.
- [ ] Beginner mode with guided template creation.
- [ ] Installation guides and "Quick Start" tutorials (Epic 15).
**Success Metrics:** 90% parity with core Templater functions; Setup for new users in < 5 minutes.

---

## 🧠 Phase 2: Intelligence & Ecosystem (Milestones 6-8)
**Goal:** Expand into a comprehensive developer tool with LSP and advanced linting.

### Milestone 6: Phase 2a - Advanced Templater
- [ ] Full module system (app, config, system, web).
- [ ] Advanced user functions and complex hooks.
- [ ] Metadata-Menu integration (core field types).

### Milestone 7: Phase 2b - LSP Foundation
- [ ] Tree-sitter grammar for Markdown and template syntax.
- [ ] LSP implementation (Go-to-definition, backlinks, rename refactoring).
- [ ] Intelligent linking and unresolved link support.

### Milestone 8: Phase 2c - Linter & Formatter
- [ ] Built-in Markdown linter using LSP foundation.
- [ ] Automatic formatting on template output.
- [ ] Custom validation rules for schemas (FR29).

---

## 🔌 Phase 3: Editor Integration (Milestone 9)
**Goal:** Native editor support for seamless workflow.

### Milestone 9: Editor Plugins
- [ ] Neovim plugin leveraging the Lithos LSP.
- [ ] VS Code extension and Zed editor support.
- [ ] Git-based template pack distribution (FR34).

---

## 🏢 Phase 4: Enterprise & Scale (Milestone 10)
**Goal:** Support for professional teams and massive data sets.

### Milestone 10: Enterprise Features
- [ ] Multi-vault support and cross-vault linking.
- [ ] Advanced audit logging and access control (FR38, FR40).
- [ ] Encrypted configuration and secret management (FR39).

---

## Timeline Visualization

```mermaid
gantt
    title Lithos Project Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 1: MVP
    M1: Foundation         :done,    m1, 2026-01-01, 30d
    M2: Persistence        :active,  m2, 2026-02-01, 28d
    M3: Indexing           :         m3, 2026-03-01, 31d
    M4: Templating         :         m4, 2026-04-01, 30d
    section Phase 1.5 & 2
    M5: Parity & UX        :         m5, 2026-05-01, 60d
    M6: Adv. Templates     :         m6, 2026-07-01, 30d
    M7: LSP Foundation     :         m7, 2026-08-01, 60d
    M8: Linter/Formatter   :         m8, 2026-10-01, 30d
    section Phase 3 & 4
    M9: Editor Plugins     :         m9, 2026-11-01, 60d
    M10: Enterprise        :         m10, 2027-01-01, 30d
```

---

## Critical Path & Dependencies

The project's critical path is driven by the **Vault Indexing Engine (Epic 9)**, which depends on the **Storage Foundation (Epic 8)** and **Schema Resolver (Epic 6)**.

1. **Domain Foundation**: Epics 1-3 establish the types and testing patterns.
2. **Infrastructure Blockers**: Epics 4, 5, 8 provide the data plane.
3. **Core Value Prop**: Epics 9 & 11 (Indexing & Templates) are the primary differentiators.
4. **Ecosystem**: LSP and Editor plugins follow the core CLI stability.

---

## Risk Assessment & Mitigation

| Risk | Severity | Impact Area | Mitigation Strategy |
| --- | --- | --- | --- |
| **Redb/rkyv complexity** | High | Persistence | Early technical spikes in Epic 8; use integration tests to verify zero-copy boundaries. |
| **Large vault performance** | High | Indexing | Continuous benchmarking (NFR1-4) in Milestone 3; use Criterion for regression tracking. |
| **LSP Implementation** | Medium | Intelligence | Leverage Tree-sitter for robust parsing; follow LSP specification strictly. |
| **Async/Concurrency bugs** | Medium | Event Bus | Implement ADR 0007 (tiered planes) early; use `tokio-test` and tracing. |

---

## Success Criteria Summary (from PRD)
1. **Efficiency**: 75% reduction in template creation time.
2. **Reliability**: Zero crashes during large vault indexing.
3. **Accuracy**: 98%+ automated schema compliance.
4. **Onboarding**: < 5 minute setup for non-technical users.
