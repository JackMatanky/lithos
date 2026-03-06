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

use std::time::SystemTime;

use rkyv::with::{AsUnixTime, Map};

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
///
/// # Timestamps
///
/// Uses `SystemTime` with rkyv's `AsUnixTime` wrapper for safe serialization.
/// This stores timestamps as Unix epoch seconds internally while preserving
/// `SystemTime`'s type safety.
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
    #[rkyv(with = Map<AsUnixTime>)]
    pub created_at: Option<SystemTime>,

    /// Filesystem mtime (change detection - detects manual edits).
    ///
    /// Updated whenever the file content changes.
    #[rkyv(with = AsUnixTime)]
    pub modified_at: SystemTime,

    /// Wall-clock timestamp when this metadata was persisted to DB.
    ///
    /// Used for debugging and audit trails.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
}

impl ConfigMetadata {
    /// Create new metadata with current `recorded_at` timestamp.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::adapter::stored::ConfigMetadata;
    /// use std::time::SystemTime;
    ///
    /// let metadata = ConfigMetadata::new(
    ///     Some(SystemTime::now()),
    ///     SystemTime::now(),
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
    ) -> Self {
        let recorded_at = SystemTime::now();
        Self {
            created_at,
            modified_at,
            recorded_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_recorded_at_to_current_time() {
        use std::time::Duration;

        let before = SystemTime::now();
        let metadata = ConfigMetadata::new(
            None,
            SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
        );
        let after = SystemTime::now();

        assert!(metadata.recorded_at >= before);
        assert!(metadata.recorded_at <= after);
    }

    #[test]
    fn new_preserves_created_at_and_modified_at() {
        use std::time::Duration;

        let created = SystemTime::UNIX_EPOCH + Duration::from_secs(500);
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let metadata = ConfigMetadata::new(Some(created), modified);

        assert_eq!(metadata.created_at, Some(created));
        assert_eq!(metadata.modified_at, modified);
    }

    #[test]
    fn metadata_round_trips_through_rkyv() {
        use std::time::Duration;

        let original = ConfigMetadata::new(
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(500)),
            SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
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
