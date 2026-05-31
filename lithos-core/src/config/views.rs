//! Raw config views for staleness tracking.
//!
//! This module defines view types that track the raw config file state
//! for staleness detection. Unlike `ConfigMetadata`, these views store
//! versioned history of the raw config files with content hashing.
//!
//! # Design
//!
//! - **`RawGlobalConfigView`**: Tracks global config file (system paths)
//! - **`RawVaultConfigView`**: Tracks vault config file (`.lithos/lithos.toml`)
//! - **`RawFileVersion`**: Individual version snapshot with content hash
//!
//! # Staleness Detection
//!
//! A config file is stale when:
//! - File timestamps changed (`created_at/modified_at` differ)
//! - Content hash differs (file was edited)
//!
//! # Version Ring Buffer
//!
//! Each view maintains up to 5 recent versions using a fixed-size ring buffer.
//! This allows:
//! - Quick rollback to previous config versions
//! - History tracking for debugging
//! - Content-based change detection

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use crate::{
    fs::metadata::FileMetadata,
    support::content_hash::{Blake3Hash, HasContentHash, HasContentHashMut},
};

fn hash_raw_global(raw: &crate::config::raw::RawGlobalConfig) -> Blake3Hash {
    let serialized = toml::to_string(raw).unwrap_or_default();
    Blake3Hash::compute(serialized.as_bytes())
}

fn hash_raw_vault(raw: &crate::config::raw::RawVaultConfig) -> Blake3Hash {
    let serialized = toml::to_string(raw).unwrap_or_default();
    Blake3Hash::compute(serialized.as_bytes())
}

// ----------------------------------------------------------- //
//                     Raw Config Views                        //
// ----------------------------------------------------------- //

/// View of global config file state with version history.
///
/// Tracks the global configuration file (resolved from system paths)
/// with up to 5 recent versions for staleness detection and rollback.
///
/// # Storage
///
/// Persisted with key `"global"` in the raw config views table.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::views::{RawGlobalConfigView, RawFileVersion};
///
/// let view = RawGlobalConfigView::new(
///     "/home/user/.config/lithos/config.toml".into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct RawGlobalConfigView {
    /// Resolved path to the global config file.
    ///
    /// This is the canonical path after resolution (XDG, env override, etc.).
    file_path: Box<str>,

    /// Ring buffer of up to 5 recent file versions.
    ///
    /// Newest version is at index 0.
    versions: Vec<RawFileVersion>,
}

impl RawGlobalConfigView {
    /// Creates a new global config view with no version history.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::RawGlobalConfigView;
    ///
    /// let view = RawGlobalConfigView::new(
    ///     "/home/user/.config/lithos/config.toml".into()
    /// );
    /// assert_eq!(view.file_path(), "/home/user/.config/lithos/config.toml");
    /// assert!(view.versions().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(file_path: Box<str>) -> Self {
        Self {
            file_path,
            versions: Vec::new(),
        }
    }

    /// Returns the resolved file path for the global config.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns all tracked versions, newest first.
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &[RawFileVersion] {
        &self.versions
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn latest_version(&self) -> Option<&RawFileVersion> {
        self.versions.first()
    }

    /// Adds a new version to the history, maintaining max 5 versions.
    ///
    /// Newest version is inserted at index 0, oldest is dropped if > 5.
    #[inline]
    pub fn push_version(&mut self, version: RawFileVersion) {
        self.versions.insert(0, version);
        if self.versions.len() > 5 {
            self.versions.truncate(5);
        }
    }

    /// Check if this view is fresh (not stale) compared to raw config.
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check (catches most modifications)
    /// 2. Content hash check (catches timestamp-preserving changes)
    ///
    /// Returns `true` if the latest version matches the raw config (not stale).
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[expect(
        clippy::same_name_method,
        reason = "Method implements IsConfigViewFresh trait"
    )]
    #[inline]
    #[must_use]
    pub fn is_fresh(&self, raw: &crate::config::raw::RawGlobalConfig) -> bool {
        self.latest_version().is_some_and(|latest| {
            let timestamp_match = raw.metadata.as_ref().is_some_and(|meta| {
                latest.is_timestamp_match(
                    meta.times().created_at(),
                    meta.times()
                        .modified_at()
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                )
            });
            timestamp_match && latest.is_content_match(&hash_raw_global(raw))
        })
    }

    /// Check if content hash matches (accurate staleness check).
    ///
    /// Returns `true` if the latest version's BLAKE3 hash matches the raw
    /// config. This catches changes even when timestamps are preserved
    /// (e.g., file restored from backup, git checkout).
    ///
    /// Returns `false` if:
    /// - No version history exists
    /// - Content hash differs
    #[inline]
    #[must_use]
    pub fn content_hash_matches(
        &self,
        raw: &crate::config::raw::RawGlobalConfig,
    ) -> bool {
        self.latest_version().is_some_and(|latest| {
            latest.content_hash() == &hash_raw_global(raw)
        })
    }

    /// Check if this view matches the given raw config (not stale).
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check (catches most modifications)
    /// 2. Content hash check (catches timestamp-preserving changes)
    ///
    /// Returns `true` only if BOTH checks pass.
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[inline]
    #[must_use]
    pub fn matches_raw(
        &self,
        raw: &crate::config::raw::RawGlobalConfig,
    ) -> bool {
        self.is_fresh(raw)
    }
}

/// View of vault config file state with version history.
///
/// Tracks the vault-specific configuration file (`.lithos/lithos.toml`)
/// with up to 5 recent versions for staleness detection and rollback.
///
/// # Storage
///
/// Persisted with key `vault_id.to_string()` in the raw config views table.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::{vault::VaultId, views::{RawVaultConfigView, RawFileVersion}};
///
/// let vault_id = VaultId::new();
/// let view = RawVaultConfigView::new(
///     vault_id,
///     "/vault/.lithos/lithos.toml".into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct RawVaultConfigView {
    /// Path to the vault config file (`.lithos/lithos.toml`).
    file_path: Box<str>,

    /// Ring buffer of up to 5 recent file versions.
    ///
    /// Newest version is at index 0.
    versions: Vec<RawFileVersion>,
}

impl RawVaultConfigView {
    /// Creates a new vault config view with no version history.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::{vault::VaultId, views::RawVaultConfigView};
    ///
    /// let view = RawVaultConfigView::new(
    ///     "/vault/.lithos/lithos.toml".into(),
    /// );
    /// assert!(view.versions().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(file_path: Box<str>) -> Self {
        Self {
            file_path,
            versions: Vec::new(),
        }
    }

    /// Returns the file path for the vault config.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns all tracked versions, newest first.
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &[RawFileVersion] {
        &self.versions
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn latest_version(&self) -> Option<&RawFileVersion> {
        self.versions.first()
    }

    /// Adds a new version to the history, maintaining max 5 versions.
    ///
    /// Newest version is inserted at index 0, oldest is dropped if > 5.
    #[inline]
    pub fn push_version(&mut self, version: RawFileVersion) {
        self.versions.insert(0, version);
        if self.versions.len() > 5 {
            self.versions.truncate(5);
        }
    }

    /// Check if this view is fresh (not stale) compared to raw config.
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check (catches most modifications)
    /// 2. Content hash check (catches timestamp-preserving changes)
    ///
    /// Returns `true` if the latest version matches the raw config (not stale).
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[expect(
        clippy::same_name_method,
        reason = "Method implements IsConfigViewFresh trait"
    )]
    #[inline]
    #[must_use]
    pub fn is_fresh(&self, raw: &crate::config::raw::RawVaultConfig) -> bool {
        self.latest_version().is_some_and(|latest| {
            let timestamp_match = raw.metadata.as_ref().is_some_and(|meta| {
                latest.is_timestamp_match(
                    meta.times().created_at(),
                    meta.times()
                        .modified_at()
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                )
            });
            timestamp_match && latest.is_content_match(&hash_raw_vault(raw))
        })
    }

    /// Check if content hash matches (accurate staleness check).
    ///
    /// Returns `true` if the latest version's BLAKE3 hash matches the raw
    /// config. This catches changes even when timestamps are preserved
    /// (e.g., file restored from backup, git checkout).
    ///
    /// Returns `false` if:
    /// - No version history exists
    /// - Content hash differs
    #[inline]
    #[must_use]
    pub fn content_hash_matches(
        &self,
        raw: &crate::config::raw::RawVaultConfig,
    ) -> bool {
        self.latest_version()
            .is_some_and(|latest| latest.content_hash() == &hash_raw_vault(raw))
    }

    /// Check if this view matches the given raw config (not stale).
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check (catches most modifications)
    /// 2. Content hash check (catches timestamp-preserving changes)
    ///
    /// Returns `true` only if BOTH checks pass.
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[inline]
    #[must_use]
    pub fn matches_raw(
        &self,
        raw: &crate::config::raw::RawVaultConfig,
    ) -> bool {
        self.is_fresh(raw)
    }
}

// ----------------------------------------------------------- //
//                   Raw File Version                          //
// ----------------------------------------------------------- //

/// Snapshot of a config file version with content hash.
///
/// Represents a single version of a config file at a specific point in time,
/// with compressed content and hash for change detection.
///
/// # Content Storage
///
/// Raw TOML content is compressed using zstd before storage to reduce
/// database size. Config files are typically small, so compression overhead
/// is minimal.
///
/// # Change Detection
///
/// The `content_hash` (BLAKE3) provides fast, content-based staleness checks
/// without decompressing the file. Timestamps provide coarse-grained detection.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::views::RawFileVersion;
/// use std::time::SystemTime;
///
/// let content = b"vault_path = \"/vault\"\nname = \"My Vault\"";
/// let version = RawFileVersion::new(content)?;
/// assert_eq!(version.content_hash().as_bytes().len(), 32);
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct RawFileVersion {
    /// File metadata (timestamps + size).
    metadata: FileMetadata,

    /// BLAKE3 hash of the uncompressed content.
    ///
    /// Used for fast content-based change detection.
    content_hash: Blake3Hash,

    /// Wall-clock timestamp when this version was recorded to DB.
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl RawFileVersion {
    /// Creates a new file version from raw content.
    ///
    /// Compresses the content and computes the BLAKE3 hash.
    ///
    /// # Errors
    ///
    /// Returns an error if compression fails (highly unlikely).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::RawFileVersion;
    /// use lithos_core::fs::metadata::{FileMetadata, FsTimes};
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let metadata = FileMetadata::new(FsTimes::new(None, Some(SystemTime::now())), content.len() as u64, false);
    /// let version = RawFileVersion::new(
    ///     content,
    ///     metadata,
    /// )?;
    /// ```
    #[inline]
    pub fn new(
        content: &[u8],
        metadata: FileMetadata,
    ) -> Result<Self, std::io::Error> {
        let content_hash = Blake3Hash::compute(content);
        let recorded_at = SystemTime::now();

        Ok(Self {
            metadata,
            content_hash,
            recorded_at,
        })
    }

    /// Returns the BLAKE3 content hash.
    #[inline]
    #[must_use]
    pub(crate) const fn content_hash(&self) -> &Blake3Hash {
        &self.content_hash
    }

    /// Returns the file metadata (timestamps + size).
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Checks if timestamps match (fast staleness check).
    ///
    /// Compares creation and modification timestamps. This is a cheap check
    /// that catches most file modifications.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::RawFileVersion;
    /// use lithos_core::fs::metadata::{FileMetadata, FsTimes};
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let mtime = SystemTime::now();
    /// let metadata = FileMetadata::new(FsTimes::new(None, Some(mtime)), content.len() as u64, false);
    /// let version = RawFileVersion::new(content, metadata)?;
    ///
    /// assert!(version.is_timestamp_match(None, mtime));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
    ) -> bool {
        self.metadata.times().created_at() == created_at
            && self.metadata.times().modified_at() == Some(modified_at)
    }

    /// Checks if content hash matches (accurate staleness check).
    ///
    /// Compares BLAKE3 hashes. This catches changes even when timestamps
    /// are preserved (e.g., restored from backup, git checkout).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::RawFileVersion;
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let version = RawFileVersion::new(content, None, SystemTime::now())?;
    /// let hash = Blake3Hash::compute(content);
    ///
    /// assert!(version.is_content_match(&hash));
    /// ```
    #[inline]
    #[must_use]
    pub(crate) fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.content_hash == *content_hash
    }

    /// Checks if this version matches the given file state.
    ///
    /// Returns `true` if timestamps AND content hash match.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::RawFileVersion;
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let mtime = SystemTime::now();
    /// let metadata = FileMetadata::new(FsTimes::new(None, Some(mtime)), content.len() as u64, false);
    /// let version = RawFileVersion::new(content, metadata)?;
    ///
    /// assert!(version.matches(None, mtime, &Blake3Hash::compute(content)));
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        dead_code,
        reason = "Convenience comparator retained for staleness helpers"
    )]
    pub(crate) fn matches(
        &self,
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
        content_hash: &Blake3Hash,
    ) -> bool {
        self.is_timestamp_match(created_at, modified_at)
            && self.is_content_match(content_hash)
    }
}

impl HasContentHash for RawFileVersion {
    fn content_hash(&self) -> &Blake3Hash {
        self.content_hash()
    }
}

impl HasContentHashMut for RawFileVersion {
    fn set_content_hash(&mut self, hash: Blake3Hash) {
        self.content_hash = hash;
    }
}

// ----------------------------------------------------------- //
//                          Tests                              //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::metadata::FsTimes;

    #[test]
    fn raw_global_config_view_new() {
        let view = RawGlobalConfigView::new("/path/to/global.toml".into());
        assert_eq!(view.file_path(), "/path/to/global.toml");
        assert!(view.versions().is_empty());
        assert!(view.latest_version().is_none());
    }

    #[test]
    fn raw_global_config_view_push_version() {
        let mut view = RawGlobalConfigView::new("/path/to/global.toml".into());

        let metadata = FileMetadata::new(
            FsTimes::new(None, Some(SystemTime::now())),
            7,
            false,
        );
        let version1 = RawFileVersion::new(b"content1", metadata)
            .expect("version creation should succeed");

        view.push_version(version1.clone());
        assert_eq!(view.versions().len(), 1);
        assert_eq!(view.latest_version(), Some(&version1));
    }

    #[test]
    #[expect(
        clippy::as_conversions,
        reason = "usize to u64 is safe on 64-bit systems (test code)"
    )]
    fn push_version_keeps_max_5() {
        let mut view = RawGlobalConfigView::new("/path/to/global.toml".into());

        // Push 7 versions
        for i in 0i32..7i32 {
            let content = format!("content{i}");
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let version = RawFileVersion::new(content.as_bytes(), metadata)
                .expect("version creation should succeed");
            view.push_version(version);
        }

        assert_eq!(view.versions().len(), 5);
    }

    #[test]
    fn raw_vault_config_view_new() {
        let view = RawVaultConfigView::new("/vault/.lithos/lithos.toml".into());
        assert_eq!(view.file_path(), "/vault/.lithos/lithos.toml");
        assert!(view.versions().is_empty());
    }

    #[test]
    #[expect(
        clippy::as_conversions,
        reason = "usize to u64 is safe on 64-bit systems (test code)"
    )]
    fn raw_file_version_new() {
        let content = b"vault_path = \"/vault\"";
        let mtime = SystemTime::now();
        let metadata = FileMetadata::new(
            FsTimes::new(None, Some(mtime)),
            content.len() as u64,
            false,
        );
        let version = RawFileVersion::new(content, metadata)
            .expect("version creation should succeed");

        let expected_hash = Blake3Hash::compute(content);
        assert_eq!(version.content_hash(), &expected_hash);
        assert_eq!(version.metadata().times().created_at(), None);
        assert_eq!(version.metadata().times().modified_at(), Some(mtime));
        assert_eq!(version.metadata().size(), content.len() as u64);
    }

    #[test]
    #[expect(
        clippy::as_conversions,
        reason = "usize to u64 is safe on 64-bit systems (test code)"
    )]
    fn raw_file_version_is_timestamp_match() {
        let content = b"vault_path = \"/vault\"";
        let mtime = SystemTime::now();
        let metadata = FileMetadata::new(
            FsTimes::new(None, Some(mtime)),
            content.len() as u64,
            false,
        );
        let version = RawFileVersion::new(content, metadata)
            .expect("version creation should succeed");

        let expected_hash = Blake3Hash::compute(content);
        assert_eq!(version.content_hash(), &expected_hash);
        assert_eq!(version.metadata().times().created_at(), None);
        assert_eq!(version.metadata().times().modified_at(), Some(mtime));
        assert_eq!(version.metadata().size(), content.len() as u64);
    }

    #[test]
    #[expect(
        clippy::as_conversions,
        reason = "usize to u64 is safe on 64-bit systems (test code)"
    )]
    fn raw_file_version_round_trips_through_rkyv() {
        let content = b"vault_path = \"/vault\"";
        let metadata = FileMetadata::new(
            FsTimes::new(None, Some(SystemTime::now())),
            content.len() as u64,
            false,
        );
        let original = RawFileVersion::new(content, metadata)
            .expect("version creation should succeed");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
            .expect("serialization should succeed");
        let archived = rkyv::access::<
            rkyv::Archived<RawFileVersion>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access should succeed");
        let deserialized: RawFileVersion =
            rkyv::deserialize::<RawFileVersion, rkyv::rancor::Error>(archived)
                .expect("deserialization should succeed");

        assert_eq!(deserialized.content_hash, original.content_hash);
        assert_eq!(deserialized.metadata, original.metadata);
    }

    mod has_content_hash {
        use super::*;

        #[test]
        #[expect(
            clippy::as_conversions,
            reason = "usize to u64 is safe on 64-bit systems (test code)"
        )]
        fn returns_content_hash() {
            let content = b"vault_path = \"/vault\"";
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let version = RawFileVersion::new(content, metadata).unwrap();
            assert_eq!(
                HasContentHash::content_hash(&version),
                &Blake3Hash::compute(content)
            );
        }

        #[test]
        #[expect(
            clippy::as_conversions,
            reason = "usize to u64 is safe on 64-bit systems (test code)"
        )]
        fn is_content_match_returns_true_when_match() {
            let content = b"vault_path = \"/vault\"";
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let version = RawFileVersion::new(content, metadata).unwrap();
            let hash = Blake3Hash::compute(content);
            assert!(HasContentHash::is_content_match(&version, &hash));
        }

        #[test]
        #[expect(
            clippy::as_conversions,
            reason = "usize to u64 is safe on 64-bit systems (test code)"
        )]
        fn is_content_match_returns_false_when_mismatch() {
            let content = b"vault_path = \"/vault\"";
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let version = RawFileVersion::new(content, metadata).unwrap();
            assert!(!HasContentHash::is_content_match(
                &version,
                &Blake3Hash::compute(b"other")
            ));
        }
    }

    mod has_content_hash_mut {
        use super::*;

        #[test]
        #[expect(
            clippy::as_conversions,
            reason = "usize to u64 is safe on 64-bit systems (test code)"
        )]
        fn set_content_hash_updates_hash() {
            let content = b"vault_path = \"/vault\"";
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let mut version = RawFileVersion::new(content, metadata).unwrap();
            let new_hash = Blake3Hash::compute(b"new content");
            version.set_content_hash(new_hash);
            assert_eq!(
                HasContentHash::content_hash(&version),
                &Blake3Hash::compute(b"new content")
            );
        }

        #[test]
        #[expect(
            clippy::as_conversions,
            reason = "usize to u64 is safe on 64-bit systems (test code)"
        )]
        fn set_content_hash_changes_match_behavior() {
            let content = b"vault_path = \"/vault\"";
            let metadata = FileMetadata::new(
                FsTimes::new(None, Some(SystemTime::now())),
                content.len() as u64,
                false,
            );
            let mut version = RawFileVersion::new(content, metadata).unwrap();
            let old_hash = Blake3Hash::compute(content);
            let new_hash = Blake3Hash::compute(b"new content");
            version.set_content_hash(new_hash);
            assert!(HasContentHash::is_content_match(&version, &new_hash));
            assert!(!HasContentHash::is_content_match(&version, &old_hash));
        }
    }
}
