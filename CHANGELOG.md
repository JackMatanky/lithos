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
- Established Architectural Decision Record (ADR) process (ADRs 0001-0011).
- Defined comprehensive project roadmap with 10 milestones and 4 phases in `ROADMAP.md`.
- Created technical foundation for async testing patterns and event-driven architecture.
- Finalized Product Requirements Document (PRD) alignment.

**Documentation:**
- Created comprehensive `README.md` with installation, architecture overview, and roadmap.
- Established sprint tracking system in `sprint-status.yaml`.
- Documented system data flow and architectural integrity standards.

---
*Note: This project was converted from a legacy Go implementation to Rust in Jan 2026. Previous Go-based history is archived in Git history.*
