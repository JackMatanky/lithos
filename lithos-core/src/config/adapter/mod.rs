//! Config storage adapters.

pub mod command;
pub mod query;
pub(crate) mod stored;

use crate::config::{aggregate::Version, vault::VaultId};

/// Helper to generate the key for merged config versions.
///
/// Key format: `{vault_id}:{version}` where `vault_id` is a UUID (36 chars)
/// and version is u64 (max 20 chars).
///
/// TODO: Optimize with stack buffer to avoid format! allocation (57 bytes max).
#[inline]
pub(crate) fn merged_version_key(
    vault_id: VaultId,
    version: Version,
) -> String {
    format!("{}:{}", vault_id, version.value())
}
