# Starter Template Evaluation

## Primary Technology Domain

CLI Tool (Rust) - Complex vault templating system with 50 functional requirements requiring hexagonal architecture, CQRS patterns, and async operations.

## Technical Preferences Confirmed

Based on project requirements analysis: Rust 1.70+, async runtime, hexagonal ports/adapters, CQRS for vault operations, embedded storage. Research of Rust ecosystem patterns confirms these as optimal for complex CLI applications with performance requirements and concurrent operations.

## Starter Options Evaluated

**Generic CLI Templates**: Keats/rust-cli-template and similar provide basic clap setup but lack the sophisticated hexagonal organization, CQRS separation, async infrastructure, and domain modeling patterns required for complex vault operations.

**Custom Single-Crate Setup**: Traditional approach but doesn't scale for 50-FR requirements or enable the semi-microservices development pattern you established in Go.

**Resources Reviewed**: Rust-Trends/example_project_structure provides basic layout. Djamware guide offers organizational principles but lacks the architectural depth of your implementation. Your Go source tree demonstrates the gold standard for hexagonal organization.

## Selected Starter: Workspace-Based Hexagonal Architecture

**Rationale for Selection:**
Cargo workspaces provide the Rust-native foundation for hexagonal architecture in complex applications. This approach enables compile-time enforcement of architectural boundaries, supports parallel development through independent crate compilation, and provides natural evolution toward microservices while maintaining clean separation between domain, application, and infrastructure concerns.

**Workspace Organization Benefits:**

- **Hexagonal Enforcement**: Crate boundaries enforce ports/adapters patterns at compile time
- **Parallel Development**: Independent crate compilation matches your Go development velocity
- **CQRS Support**: Natural separation of commands/queries following your Go patterns
- **Async Native**: Tokio leverages Rust's strengths for concurrent vault operations
- **Testability**: Domain purity enables comprehensive testing like your Go implementation
- **Scalability**: Semi-microservices structure for team growth and ecosystem expansion
- **Architecture Preservation**: Maintains your established hexagonal patterns in Rust

**Initialization Commands:**

```bash
# Create workspace root
mkdir lithos && cd lithos
cargo new crates/domain --lib
cargo new crates/app --lib
cargo new crates/adapters --lib
cargo new crates/cli --bin
```

**Workspace Cargo.toml:**

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.49", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
redb = "3.1"
rkyv = "0.8"
anyhow = "1.0"
thiserror = "2.0"
miette = { version = "7.6", features = ["fancy"] }
tracing = "0.1"
minijinja = "2.14"
pulldown-cmark = "0.13"
figment = { version = "0.10", features = ["toml", "env"] }
uuid = { version = "1.19", features = ["v7", "serde"] }
```

**Crate Structure (Following Rust Hexagonal Best Practices):**

```
lithos/
├── crates/
│   ├── domain/           # Core business models & logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── config/    # Config bounded context
│   │       ├── note/      # Note bounded context
│   │       ├── schema/    # Schema bounded context
│   │       ├── template/  # Template bounded context
│   │       ├── ports/     # Traits/interfaces for external dependencies
│   │       ├── errors.rs  # Domain errors
│   │       └── validation.rs # Shared validation utilities
│   ├── app/              # Application services & orchestrators
│   │   ├── Cargo.toml    # Depends on domain
│   │   └── src/
│   │       ├── commands/ # CQRS command handlers
│   │       ├── queries/  # CQRS query handlers
│   │       ├── vault/    # VaultIndexer orchestrator
│   │       ├── schema/   # SchemaEngine orchestrator
│   │       └── template/ # TemplateEngine orchestrator
│   ├── adapters/         # Infrastructure implementations
│   │   ├── Cargo.toml    # Depends on domain + external crates
│   │   └── src/
│   │       ├── api/      # Driver adapters (CLI, future LSP)
│   │       ├── spi/      # Driven adapters (storage, filesystem, config)
│   │       └── dto/      # Data transfer objects
│   └── cli/              # Binary entry point
│       ├── Cargo.toml    # Depends on app + adapters
│       └── src/main.rs
└── Cargo.lock
```

**Architectural Decisions (Following Rust Ecosystem Patterns):**

- **Workspace Enforcement**: Crate boundaries enforce hexagonal dependency inversion using Rust's module system
- **Domain Purity**: Domain crate contains only business logic, no external dependencies (standard Rust practice)
- **CQRS Implementation**: App crate separates commands (writes) from queries (reads) using async patterns
- **Adapter Pattern**: Adapters crate implements domain traits with external systems using Rust's trait system
- **Semi-Microservices**: Workspace enables parallel development and future service extraction
- **Async Native**: Tokio integration across crates for concurrent vault operations leveraging Rust's async strengths
- **Testing Architecture**: Domain tests require no setup, integration tests span crates using Rust's testing framework
- **Development Velocity**: Independent crate compilation optimizes Rust's incremental compilation

**Development Benefits:**

- **Clean Boundaries**: Compile-time enforcement of hexagonal architectural rules using Rust's ownership system
- **Parallel Iteration**: Domain, application, and infrastructure developed independently with cargo's workspace features
- **Testability**: Pure domain logic tested without infrastructure complexity using Rust's unit testing
- **Scalability**: Natural evolution path for team growth and microservices using Rust's ecosystem patterns
- **Performance**: Leverages Rust's zero-cost abstractions and async runtime for high-performance CLI operations

**Note:** Project initialization using this workspace structure should be the first implementation story, establishing the hexagonal foundation following Rust ecosystem best practices for complex applications.
