# Project Roadmap - lithos

This roadmap outlines the phased development of Lithos, from its core Rust foundation to a comprehensive developer tooling platform. It is based on the Product Requirements Document (PRD) and Architectural Decision Records (ADRs).

## 🚀 Phase 1: MVP Core (Milestones 1-6)
**Goal:** Deliver a stable, high-performance CLI tool for interactive, schema-validated templating in large vaults.

### Milestone 1: Foundation & Domain Modeling
**Goal:** Core development environment and fundamental entities.
**Status:** In-Progress (Stories 1.1-1.8 complete)
- [x] Workspace structure and quality gates (Clippy, Rustfmt, Pre-commit).
- [x] ADR process established (ADRs 0001-0007).
- [ ] Implement fundamental domain models (Note, Frontmatter, Schema).
**Success Metrics:** Hexagonal boundaries enforced at compile-time; 80%+ test coverage.

### Milestone 2: Persistence & Schema Engine
**Goal:** ACID storage and schema validation.
- [ ] Multi-strategy file loading with error recovery (Epic 4).
- [ ] Redb + rkyv persistence with zero-copy deserialization (Epic 8).
- [ ] Schema validation engine with inheritance (FR8, FR10, FR14).
**Success Metrics:** 98%+ automatic validation compliance; Zero-copy boundaries verified.

### Milestone 3: Vault Indexing & Intelligence
**Goal:** High-performance indexing and metadata query service.
- [ ] Actor-based indexing engine for incremental updates (Epic 9).
- [ ] Metadata query service with wiki-link/alias resolution (FR22, FR23).
- [ ] Hybrid event bus (MPSC/Broadcast/Watch) (Epic 7).
**Success Metrics:** Indexing 1000+ files in < 2 seconds; < 50ms query response.

### Milestone 4: Interactive Template System
**Goal:** Core value - interactive, scriptable templating.
- [ ] MiniJinja-based template engine implementation (Epic 11).
- [ ] Interactive CLI prompts, suggesters, and multi-suggesters (FR15-FR17).
- [ ] Modular template composition (FR1, FR3).
**Success Metrics:** Template creation time < 30 minutes; Execution < 500ms.

### Milestone 5: CLI Excellence & MVP Quality
**Goal:** Production-ready CLI and error handling.
- [ ] Clap-based CLI with subcommands and single-word shortcuts (FR41, FR47).
- [ ] Miette diagnostics for high-fidelity terminal errors (FR48).
- [ ] Performance regression suite and cross-epic integration tests (Epic 14).
**Success Metrics:** 0% crash rate in large vaults; 99.9% uptime for CLI operations.

### Milestone 6: User Onboarding & Launch
**Goal:** Documentation and community starter kits.
- [ ] Installation guides and "Quick Start" tutorials (Epic 15).
- [ ] Starter template packs and schema library.
- [ ] Public API documentation.
**Success Metrics:** Successful setup for non-technical users in < 5 minutes.

---

## 🛠 Phase 2: Intelligence & Ecosystem (Milestones 7-8)
**Goal:** Expand into a comprehensive developer tool with LSP and advanced linting.

### Milestone 7: LSP & Intelligence Foundation (Phase 2b)
- [ ] Tree-sitter grammar for Markdown and template syntax.
- [ ] LSP implementation (Go-to-definition, backlinks, rename refactoring).
- [ ] Advanced Templater parity (app, config, system modules).

### Milestone 8: Linter & Formatter Integration (Phase 2c)
- [ ] Built-in Markdown linter using LSP foundation.
- [ ] Automatic formatting on template output.
- [ ] Custom validation rules for schemas (FR29).

---

## 🔌 Phase 3: Editor Integration (Milestones 9)
**Goal:** Native editor support for seamless workflow.

### Milestone 9: Editor Plugins (Phase 3a/3b)
- [ ] Neovim plugin leveraging the Lithos LSP.
- [ ] VS Code extension and Zed editor support.
- [ ] Git-based template pack distribution (FR34).

---

## 🏢 Phase 4: Enterprise & Scale (Milestone 10)
**Goal:** Support for professional teams and massive data sets.

### Milestone 10: Enterprise Features (Phase 4)
- [ ] Multi-vault support and cross-vault linking.
- [ ] Advanced audit logging and access control (FR38, FR40).
- [ ] Encrypted configuration and secret management (FR39).

---

## Timeline Visualization

```mermaid
gantt
    title lithos Project Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 1: MVP
    M1: Foundation         :done,    m1, 2026-01-01, 30d
    M2: Persistence        :active,  m2, 2026-02-01, 28d
    M3: Indexing           :         m3, 2026-03-01, 31d
    M4: Templating         :         m4, 2026-04-01, 30d
    M5: CLI & Quality      :         m5, 2026-05-01, 31d
    M6: Onboarding         :         m6, 2026-06-01, 30d
    section Phase 2: Intelligence
    M7: LSP Foundation     :         m7, 2026-07-01, 60d
    M8: Linter/Formatter   :         m8, 2026-09-01, 30d
    section Phase 3: Plugins
    M9: Editor Plugins     :         m9, 2026-10-01, 60d
    section Phase 4: Scale
    M10: Enterprise        :         m10, 2026-12-01, 30d
```

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
