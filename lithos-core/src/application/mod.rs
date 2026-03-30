//! Application services layer.
//!
//! This module contains application services that orchestrate domain operations
//! and infrastructure concerns. Services in this layer are responsible for:
//!
//! - **Workflow Coordination**: Orchestrating multi-step processes (file → raw
//!   → domain → database)
//! - **Cross-Cutting Concerns**: Transaction management, logging, error
//!   handling
//! - **Dependency Injection**: Accepting ports/interfaces rather than concrete
//!   implementations
//!
//! ## Architecture
//!
//! Application services follow the **Service Layer** pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │     Application Services Layer          │
//! │  (Workflow Orchestration & Use Cases)   │
//! └──────────┬──────────────────────────────┘
//!            │
//!            ├─→ Domain Layer (Business Logic)
//!            ├─→ Infrastructure Layer (I/O)
//!            └─→ Storage Ports (Persistence)
//! ```
//!
//! ## Design Principles
//!
//! - **Port-based dependencies**: Services use explicit ports (`Query`,
//!   `Command`) and infrastructure types (`FsReader`)
//! - **Thin services**: Business logic lives in domain aggregates, services
//!   only orchestrate
//! - **Error translation**: Convert infrastructure/domain errors into
//!   application errors
//! - **Observability**: Services use tracing for workflow visibility

// NOTE: Ingestion services temporarily archived in _archive/ for future
// re-implementation.
// NOTE: Schema loader moved to schema module (schema::loader) as part of Phase
// 6.
// NOTE: Config service removed - use config::loader::Loader directly.
