//! Lithos Domain Crate.
//!
//! This crate contains the pure business logic, domain entities, and port definitions
//! for the Lithos system. It has no dependencies on external I/O or frameworks.
//!
//! # Architecture
//! - **Models**: Domain entities and value objects (pure business logic)
//! - **Ports**: Trait interfaces for adapters (hexagonal architecture)
//! - **Errors**: Domain-specific error types
//! - **Events**: Domain events for event-driven architecture
//!
//! # Hexagonal Architecture Compliance
//! - NO external I/O dependencies
//! - NO framework dependencies
//! - Pure business logic only
//! - Adapters implement ports

#![allow(clippy::pub_use, reason = "Simplified public API for external crates")]

pub mod errors;
pub mod events;
pub mod models;
pub mod ports;

// Re-export commonly used types for convenience.
// This provides a simplified public API for external crates without requiring
// deep module path knowledge (e.g., `lithos_domain::Config` vs `lithos_domain::models::config::Config`).
pub use errors::{ConfigError, DomainError};
pub use events::ConfigUpdated;
pub use models::config::{
    Config, ConfigValue, FileSystemConfig, FrontmatterConfig, GlobalConfig,
    VaultConfig,
};
pub use ports::config::{Command as ConfigCommand, Query as ConfigQuery};
