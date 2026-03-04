//! Storage representation for config metadata.
//!
//! This module defines metadata types for config staleness checking,
//! following the same pattern as the schema module's `adapter::stored`.

// Suppress field name lint for timestamp fields - semantically meaningful
// naming
#![allow(
    clippy::struct_field_names,
    reason = "Timestamp fields (_at suffix) are semantically meaningful and \
              match schema module pattern"
)]

use crate::config::aggregate::Timestamp;

/// Metadata for config staleness checking.
///
/// Stores file timestamps to detect when config files have changed.
/// Used by the query adapter to determine if cached configs are still valid.
///
/// # Storage
///
/// Persisted in the `config_metadata` table with keys:
/// - `"global"` for global config metadata
/// - `vault_id.to_string()` for vault config metadata
///
/// # Staleness Detection
///
/// A config is considered stale when:
/// - `created_at` differs (file replaced)
/// - `modified_at` is newer (file edited)
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct ConfigMetadata {
    /// Filesystem birthtime (identity check - detects file replacement).
    ///
    /// When a config file is deleted and recreated, the `created_at`
    /// timestamp will differ, indicating a new file.
    pub created_at: Option<Timestamp>,

    /// Filesystem mtime (change detection - detects manual edits).
    ///
    /// Updated whenever the file content changes.
    pub modified_at: Timestamp,

    /// Wall-clock timestamp when this metadata was persisted to DB.
    ///
    /// Used for debugging and audit trails.
    pub recorded_at: Timestamp,
}

impl ConfigMetadata {
    /// Create new metadata with current `recorded_at` timestamp.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::adapter::stored::ConfigMetadata;
    /// use lithos_core::config::aggregate::Timestamp;
    ///
    /// let metadata = ConfigMetadata::new(
    ///     Some(Timestamp::from_secs(1000)),
    ///     Timestamp::from_secs(2000),
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub fn new(created_at: Option<Timestamp>, modified_at: Timestamp) -> Self {
        Self {
            created_at,
            modified_at,
            recorded_at: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_recorded_at_to_current_time() {
        let before = Timestamp::now();
        let metadata = ConfigMetadata::new(None, Timestamp::from_secs(1000));
        let after = Timestamp::now();

        assert!(metadata.recorded_at.as_secs() >= before.as_secs());
        assert!(metadata.recorded_at.as_secs() <= after.as_secs());
    }

    #[test]
    fn new_preserves_created_at_and_modified_at() {
        let created = Timestamp::from_secs(500);
        let modified = Timestamp::from_secs(1000);
        let metadata = ConfigMetadata::new(Some(created), modified);

        assert_eq!(metadata.created_at, Some(created));
        assert_eq!(metadata.modified_at, modified);
    }

    #[test]
    fn metadata_round_trips_through_rkyv() {
        let original = ConfigMetadata::new(
            Some(Timestamp::from_secs(500)),
            Timestamp::from_secs(1000),
        );

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
            .expect("serialization should succeed");
        let archived = rkyv::access::<
            rkyv::Archived<ConfigMetadata>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access should succeed");
        let deserialized: ConfigMetadata =
            rkyv::deserialize::<ConfigMetadata, rkyv::rancor::Error>(archived)
                .expect("deserialization should succeed");

        assert_eq!(deserialized.created_at, original.created_at);
        assert_eq!(deserialized.modified_at, original.modified_at);
        assert_eq!(deserialized.recorded_at, original.recorded_at);
    }
}
