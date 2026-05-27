//! Table definitions for configuration storage.

use crate::{
    config::vault::VaultId,
    db::{Table, UuidTable},
    impl_redb_uuid,
};

impl_redb_uuid!(VaultId);

/// Versioned global configuration.
///
/// Keys: `"{version}"` → `Global`.
/// Example: `"1"` → `Global { ... }`.
pub const GLOBAL_CONFIG: Table<&str, &[u8]> = Table::new("global_config");

/// Versioned vault-specific configuration.
///
/// Keys: `"{vault_id}:{version}"` → `Vault`.
/// Example: `"abc123:1"` → `Vault { ... }`.
pub const VAULT_CONFIG: Table<&str, &[u8]> = Table::new("vault_config");

/// Versioned final configuration (result of merging global + vault).
///
/// Keys: `"{vault_id}:{version}"` → `Config`.
/// Example: `"abc123:1"` → `Config { ... }`.
pub const CONFIG_VERSIONS: Table<&str, &[u8]> = Table::new("config_versions");

/// Vault path-to-ID mapping.
///
/// Keys: `vault_root.as_key()` → `VaultId`.
pub const VAULT_ID_BY_PATH: Table<&str, &[u8]> = Table::new("vault_id_by_path");

/// Vault ID-to-path reverse mapping.
///
/// Keys: `VaultId` → `VaultRoot`.
pub const VAULT_PATH_BY_ID: UuidTable<VaultId, &[u8]> =
    UuidTable::new("vault_path_by_id");

/// Raw global config view with version history.
///
/// Keys: `"global"` → `RawGlobalConfigView`.
pub const RAW_GLOBAL_CONFIG_VIEW: Table<&str, &[u8]> =
    Table::new("raw_global_config_view");

/// Raw vault config views with version history.
///
/// Keys: `VaultId` → `RawVaultConfigView`.
pub const RAW_VAULT_CONFIG_VIEW: UuidTable<VaultId, &[u8]> =
    UuidTable::new("raw_vault_config_view");
