//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

pub(crate) mod aggregate;
pub(crate) mod events;
pub(crate) mod global;
pub(crate) mod types;
pub(crate) mod vault;

// --- Public API & Re-exports ---

// Aggregate
pub use aggregate::Config;
// Events
pub use events::{ConfigEvents, ConfigUpdated};
// Global Configuration Types (Renamed for clarity)
pub use global::{
    Filesystem as GlobalFilesystemConfig, Global as GlobalConfig,
    TrustedVaults as TrustedVaultsConfig,
};
// Shared Configuration Types
pub use types::{
    Frontmatter as FrontmatterConfig, Logging as LoggingConfig,
    Schema as SchemaConfig, SettingValue as ConfigValue,
    Template as TemplateConfig,
};
// Vault Configuration Types (Renamed for clarity)
pub use vault::{
    Filesystem as VaultFilesystemConfig, Metadata as VaultMetadataConfig,
    Vault as VaultConfig,
};
