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
//! - **Port-based dependencies**: Services depend on traits (`FileSource`,
//!   Query, Command) not concrete types
//! - **Thin services**: Business logic lives in domain aggregates, services
//!   only orchestrate
//! - **Error translation**: Convert infrastructure/domain errors into
//!   application errors
//! - **Observability**: Services use tracing for workflow visibility
//!
//! ## Modules
//!
//! - **services**: Ingestion services for each bounded context (schema,
//!   template, note)
//! - **error**: Unified error types for application-layer failures

/// Application-layer error types.
pub mod error;
/// Ingestion services for bounded contexts.
pub mod services;

/// Ingestion error type alias.
pub type IngestionError = error::IngestionError;
