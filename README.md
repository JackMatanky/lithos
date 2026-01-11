# Lithos

[![CI](https://github.com/jack/lithos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/jack/lithos-rust/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/lithos.svg)](https://crates.io/crates/lithos)
[![Docs.rs](https://docs.rs/lithos/badge.svg)](https://docs.rs/lithos)

> Powerful, scriptable template generation for Obsidian vaults.

Lithos is a command-line powerhouse for Obsidian vaults, bridging the gap between terminal efficiency and structured knowledge management. It provides a robust engine for executing modular templates, enforcing metadata schemas, and performing vault-wide queries without ever leaving your editor or terminal.

Whether you're an Alex Chen power user needing scriptable note creation or a Sarah Martinez researcher managing thousands of interconnected files, Lithos ensures your vault stays consistent and performant. Built in Rust with a quality-first mindset, it leverages zero-copy persistence and a hexagonal architecture to deliver sub-500ms operations even in massive vaults.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)
- [Changelog](#changelog)
- [Acknowledgments](#acknowledgments)

---

## Installation

### Prerequisites

- **Rust 1.92+**: [Install Rust](https://www.rust-lang.org/tools/install)
- **mise**: (Recommended) For tool versioning and task orchestration.

### From Source

```bash
git clone https://github.com/jack/lithos-rust.git
cd lithos-rust
cargo build --release
```

### Via Cargo (Coming Soon)

```bash
cargo install lithos
```

---

## Quick Start

Initialize a new note interactively from a template:

```bash
# Run the interactive template picker
lithos new --interactive

# Or specify a template directly
lithos new project-decision --vault ~/my-obsidian-vault
```

---

## Architecture

Lithos follows a **Hexagonal Architecture** (Ports and Adapters) combined with **CQRS** (Command Query Responsibility Segregation) to ensure the core domain logic remains isolated and testable.

### Bounded Contexts

- **Note**: The primary unit of knowledge, containing frontmatter, links, and content.
- **Schema**: Machine-readable metadata definitions enforcing vault consistency.
- **Template**: Modular, reusable rendering blocks powered by `MiniJinja`.
- **Config**: Hierarchical settings management (Global -> User -> Project -> Vault).

### Component Overview

- **Crates**:
  - `crates/domain`: Pure business logic, entities, and Port traits. No external I/O.
  - `crates/app`: Use case orchestrators (Commands/Queries) and event handling.
  - `crates/adapters`: Infrastructure implementations (Redb storage, MiniJinja rendering, miette diagnostics).
  - `crates/lithos`: Binary CLI entry point.

For more details, see the [Architecture Documentation](_bmad-output/planning-artifacts/architecture.md) and the [System Data Flow Diagram](_bmad-output/planning-artifacts/architecture.md#architectural-integrity).

---

## Roadmap

Lithos development is organized into four major phases, starting with a high-performance CLI core and evolving into a full editor ecosystem.

**Current Status: [Milestone 1 (Foundation)](ROADMAP.md#milestone-1-foundation--domain-modeling)**
- [x] Environment & Tooling (Epic 1)
- [x] Test Architecture (Epic 2)
- [ ] Core Domain Models (In-Progress)

### Upcoming Milestones:
1. **Persistence & Schema**: Redb storage and validation engine.
2. **Vault Intelligence**: Incremental indexing and metadata queries.
3. **Interactive Templates**: The core scriptable templating experience.

For the comprehensive roadmap, including future phases (LSP, Neovim, Enterprise), see [ROADMAP.md](ROADMAP.md).

---

## API Documentation

Detailed API documentation for each crate is available via `docs.rs`:

- [lithos-domain](https://docs.rs/lithos-domain): Core traits and models.
- [lithos-app](https://docs.rs/lithos-app): Service orchestration.
- [lithos-adapters](https://docs.rs/lithos-adapters): Implementation details.

---

## Development

We use `mise` as our primary task runner to ensure development parity.

### Setup

```bash
# Install required tools via mise
mise install

# Run the dev setup task
mise run dev-setup
```

### Quality Tools

Lithos enforces strict quality standards through `pre-commit` hooks and Clippy limits:

- **Formatting**: `mise run fmt` (Import sorting enabled)
- **Linting**: `mise run lint` (Cognitive complexity < 25)
- **Security**: `mise run deny` (Dependency auditing)
- **Verification**: `mise run verify` (Runs all quality checks)

---

## Testing

We prioritize a high-fidelity testing suite with **80%+ coverage**.

```bash
# Run all tests using nextest
mise run test

# Run tests with coverage report
mise run test:coverage
```

- **Unit Tests**: Located inline in `src/` modules.
- **Integration Tests**: Located in `tests/integration/`.
- **E2E Tests**: Located in `tests/e2e/` (validating CLI behavior).

---

## Contributing

We welcome contributions! Please follow our established patterns:

1. **Architecture First**: Ensure changes respect hexagonal boundaries.
2. **Quality Gates**: All PRs must pass `mise run verify`.
3. **ADR Process**: Significant changes require an Architectural Decision Record in `docs/adr/`.
4. **Commits**: Follow conventional commits for clean history.

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

---

## Community & Support

- **GitHub Discussions**: For questions and brainstorming.
- **Issues**: For bug reports and feature requests.

## Maintainers

- **Jack** (@jack) - Project Lead

---

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a history of changes.

---

## Acknowledgments

- **Richard Littauer** for the [Standard README](https://github.com/RichardLitt/standard-readme) specification.
- **Standard-Readme** community for best practices.
- The **Rust Ecosystem** for provide the tools (Tokio, Serde, Redb, etc.) that make Lithos possible.
- **Obsidian** for inspiring a new wave of personal knowledge management.
