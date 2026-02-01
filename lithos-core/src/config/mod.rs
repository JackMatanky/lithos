//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
#![allow(missing_docs, reason = "Transitional state of documentation")]

pub(crate) mod aggregate;
pub mod error;
pub(crate) mod events;
pub(crate) mod global;
pub mod ports;
pub(crate) mod types;
pub(crate) mod vault;

// --- Public API & Re-exports ---
// Types are accessed via config:: prefix, removing Config suffix
/// Config aggregate root.
pub type Config = aggregate::Config;
/// Configuration error.
pub type ConfigError = error::ConfigError;
/// Configuration updated event.
pub type ConfigUpdated = events::ConfigUpdated;
/// Configuration events.
pub type Events = events::Events;
/// Global configuration.
pub type Global = global::Global;
/// Trusted vaults configuration.
pub type TrustedVaults = global::TrustedVaults;

/// Command port for configuration.
pub trait Command: ports::Command {}
impl<T> Command for T where T: ports::Command + ?Sized {}

/// Query port for configuration.
pub trait Query: ports::Query {}
impl<T> Query for T where T: ports::Query + ?Sized {}

/// Frontmatter configuration.
pub type Frontmatter = types::Frontmatter;
/// Logging configuration.
pub type Logging = types::Logging;
/// Schema configuration.
pub type Schema = types::Schema;
/// Configuration value.
pub type Value = types::SettingValue;
/// Template configuration.
pub type Template = types::Template;
/// Vault metadata.
pub type Metadata = vault::Metadata;
/// Vault configuration.
pub type Vault = vault::Vault;
