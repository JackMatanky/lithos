//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
/// Configuration aggregate root.
pub mod aggregate;
/// Configuration command implementations (CQRS write operations).
pub mod command;
/// Configuration error types.
pub mod error;
/// Configuration domain events.
pub mod events;
/// Global configuration types and validation.
pub mod global;
/// Configuration ports for CQRS.
pub mod ports;
/// Configuration query implementations (CQRS read operations).
pub mod query;
/// Shared configuration value types.
pub mod types;
/// Vault-scoped configuration types.
pub mod vault;

// --- Public API & Submodules ---
// Types are accessed via config::<submodule>::<Type>
