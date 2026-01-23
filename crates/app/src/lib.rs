//! # Lithos Application Crate
//!
//! This crate implements the application layer of the hexagonal architecture,
//! providing orchestration logic for domain use cases. It translates domain
//! commands and queries into concrete operations using adapters, while
//! maintaining architectural boundaries.
//!
//! ## Architectural Invariants
//!
//! - **No I/O Dependencies**: Application services are pure orchestration logic
//!   with no direct file system, network, or external I/O operations.
//! - **CQRS Pattern**: Commands (write operations) and queries (read
//!   operations) are handled separately through dedicated services.
//! - **Unit of Work**: All commands are executed within a transactional context
//!   to ensure atomicity and deferred event dispatching.
//! - **Dependency Injection**: Ports are injected as `Arc<dyn Trait>` for
//!   thread-safe sharing across async tasks.
//! - **Event-Driven**: Domain events are staged within commands and dispatched
//!   post-commit to prevent phantom events from failed transactions.
//!
//! ## Key Components
//!
//! - **Command Handlers**: Execute domain commands with transactional
//!   guarantees.
//! - **Query Handlers**: Retrieve data through optimized read models.
//! - **Event Bus Integration**: Publishes domain events to notify subscribers.
//! - **Configuration Services**: Manage global and vault-specific settings.
//!
//! ## Example
//!
//! ```ignore
//! // Example of using a command handler (adapters not yet implemented)
//! use lithos_app::commands::CreateNote;
//!
//! let command = CreateNote { /* ... */ };
//! let result = command_handler.handle(command).await?;
//! ```
//!
//! This crate depends only on the domain crate and external utility libraries.
