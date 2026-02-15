//! Config storage adapters.

pub mod command;
pub mod query;

use crate::config::{aggregate::Version, vault::VaultId};

/// Helper to generate the key for merged config versions.
#[inline]
pub(crate) fn merged_version_key(
    vault_id: VaultId,
    version: Version,
) -> String {
    format!("{}:{}", vault_id, version.value())
}
