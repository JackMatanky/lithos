# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Epic 1: Development Environment & Foundation (Rust Conversion)

**Workspace & Infrastructure:**
- Initialized Cargo workspace with hexagonal architecture (domain, app, adapters, cli crates).
- Configured `mise.toml` for task orchestration (test, lint, fmt, verify).
- Established stringent quality gates via `.pre-commit-config.yaml`.
- Configured `clippy.toml` with cognitive complexity limits (max 25) and anti-pattern denies.
- Set up `rustfmt.toml` with `imports_granularity = "Crate"` and sorted group imports.
- Implemented `cargo-deny` configuration for dependency security, license, and advisory auditing.

**Architecture & Planning:**
- Established Architectural Decision Record (ADR) process (ADRs 0001-0012).
- Defined comprehensive project roadmap with 10 milestones and 4 phases in `ROADMAP.md`.
- Created technical foundation for async testing patterns and event-driven architecture.
- Finalized Product Requirements Document (PRD) alignment.

### Added - Epic 2: Test Architecture Patterns & Infrastructure (Milestone 1)

**Testing Infrastructure:**
- Established centralized test utilities in `lithos-test-utils` (temp directories, fixtures, assertions).
- Implemented async testing patterns for Tokio with multi-threaded runtime configuration.
- Created event-driven testing framework for hybrid event bus (MPSC, Broadcast, Watch).
- Established CQRS testing patterns for command/query separation and eventual consistency.
- Configured integration testing structure in `tests/` with parallel execution support via `nextest`.
- Established performance benchmarking infrastructure using `criterion`.

**Documentation:**
- Created comprehensive `docs/testing/developer-guide.md` as a single source of truth for testing standards.
- Authored detailed `docs/testing/async.md` and `docs/testing/event.md` guidelines.
- Published ADRs 0008-0012 covering testing, CQRS, and benchmarking infrastructure.
- Produced gap analysis report for continuous documentation improvement.

**Documentation:**
- Created comprehensive `README.md` with installation, architecture overview, and roadmap.
- Established sprint tracking system in `sprint-status.yaml`.
- Documented system data flow and architectural integrity standards.

---
*Note: This project was converted from a legacy Go implementation to Rust in Jan 2026. Previous Go-based history is archived in Git history.*
