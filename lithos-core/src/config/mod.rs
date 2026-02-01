//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

#![allow(clippy::pub_use, reason = "Re-exports provide clean public API")]
#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

pub(crate) mod aggregate;
pub mod error;
pub(crate) mod events;
pub(crate) mod global;
pub mod ports;
pub(crate) mod types;
pub(crate) mod vault;

// --- Public API & Re-exports ---
// Types are accessed via config:: prefix, removing Config suffix
pub use aggregate::Config;
pub use error::ConfigError;
pub use events::{ConfigUpdated, Events};
pub use global::{Global, TrustedVaults};
pub use ports::{Command, Query};
pub use types::{
    Frontmatter, Logging, Schema, SettingValue as Value, Template,
};
pub use vault::{Metadata, Vault};
