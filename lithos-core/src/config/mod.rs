//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
#![allow(missing_docs, reason = "Transitional state of documentation")]

pub mod aggregate;
pub mod error;
pub mod events;
pub mod global;
pub mod ports;
pub mod types;
pub mod vault;

// --- Public API & Submodules ---
// Types are accessed via config::<submodule>::<Type>
