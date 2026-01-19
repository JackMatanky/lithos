//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and events.

pub mod core;
pub mod events;

// Re-export main types for convenience
pub use core::{Config, FileSystem, Frontmatter, Global, SettingValue, Vault};
