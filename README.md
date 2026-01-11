# Lithos (Rust)

Lithos is a high-performance CLI tool for managing and processing Obsidian vaults, providing schema-driven lookups, template rendering, and interactive input capabilities. It is built in Rust with a hexagonal architecture for maximum maintainability and performance.

## Features

- **High-Performance Indexing**: Powered by `Redb` and `rkyv` for zero-copy metadata access.
- **Schema System**: Define structured metadata requirements for your notes.
- **Template Rendering**: Flexible templating with `MiniJinja`.
- **CLI Diagnostics**: Rich error reporting with `miette`.
- **AI-Ready**: Designed with strict quality gates for AI-assisted development.

## Installation

### Prerequisites

- **Rust 1.85+**
- **mise** (recommended for task orchestration)

### Build from Source

```bash
git clone https://github.com/jack/lithos-rust.git
cd lithos-rust
cargo build --release
```

## Development Workflow

We use `mise` for common tasks:

- `mise run verify`: Run all quality gates (fmt, lint, test).
- `mise run test`: Run the full test suite.
- `mise run lint`: Run stringent clippy checks.

## Code Quality & AI Safeguards

Lithos enforces strict quality standards to ensure that both human and AI-generated code remains maintainable and safe.

### Quality Gates

- **Cognitive Complexity**: Hard limit of 25 (deny) via Clippy.
- **No shortcuts**: Prohibits `unwrap()`, `expect()`, `todo!`, and `panic!` in production code.
- **Zero-Copy Performance**: Mandatory use of `rkyv` for persistence layers.
- **Hexagonal Integrity**: Strict boundary enforcement between domain, app, and adapters.

### AI Linting Policy

All code must pass the stringent clippy configuration defined in `clippy.toml` and `Cargo.toml`. Disabling lints is allowed only as a last resort and requires an audit trail:

```rust
// # LINT_DISABLE_REASON: [Short reason]
// | Options tried: [Attempts]
// | Justification: [Why necessary]
#[allow(clippy::lint_name)]
```

For more details, see [Clippy Standards](docs/standards/clippy.md).

## Project Structure

```
lithos/
├── crates/
│   ├── domain/     # Pure business logic & port definitions
│   ├── app/        # Application services & orchestration
│   ├── adapters/   # SPI implementations (Storage, FS, etc.)
│   └── cli/        # CLI entry point & UI
├── docs/           # Documentation & standards
└── _bmad-output/   # BMAD planning and implementation artifacts
```

## License

MIT OR Apache-2.0
