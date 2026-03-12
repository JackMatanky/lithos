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

use crate::config::vault::VaultId;

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
/// use lithos_core::config::views::raw::{RawGlobalConfigView, RawFileVersion};
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
    /// use lithos_core::config::views::raw::RawGlobalConfigView;
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

    /// Check if this view matches the given raw config (not stale).
    ///
    /// Returns `true` if the latest version in this view matches the
    /// raw config's metadata (timestamps AND content hash), indicating no
    /// staleness.
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[must_use]
    pub fn matches_raw(
        &self,
        raw: &crate::config::raw::RawVaultConfig,
    ) -> bool {
        self.latest_version().map_or(false, |latest| {
            let meta = &raw.metadata;
            // Compare created_at (latest has Option, meta has Option)
            latest.created_at() == meta.created_at
                // Compare modified_at (latest has SystemTime, meta has Option)
                && meta.modified_at.map_or(false, |mt| latest.modified_at() == mt)
                // Compare content hash
                && latest.content_hash()
                    == &meta.content_hash.unwrap_or([0; 32])
        })
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
/// use lithos_core::config::{vault::VaultId, views::raw::{RawVaultConfigView, RawFileVersion}};
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
    /// Vault identifier this config belongs to.
    vault_id: VaultId,

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
    /// use lithos_core::config::{vault::VaultId, views::raw::RawVaultConfigView};
    ///
    /// let vault_id = VaultId::new();
    /// let view = RawVaultConfigView::new(
    ///     vault_id,
    ///     "/vault/.lithos/lithos.toml".into(),
    /// );
    /// assert!(view.versions().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(vault_id: VaultId, file_path: Box<str>) -> Self {
        Self {
            vault_id,
            file_path,
            versions: Vec::new(),
        }
    }

    /// Returns the vault ID for this config.
    #[inline]
    #[must_use]
    pub fn vault_id(&self) -> VaultId {
        self.vault_id
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

    /// Check if this view matches the given raw config (not stale).
    ///
    /// Returns `true` if the latest version in this view matches the
    /// raw config's metadata (timestamps AND content hash), indicating no
    /// staleness.
    ///
    /// Returns `false` if:
    /// - No version history exists (never ingested)
    /// - Timestamps differ (file was modified)
    /// - Content hash differs (file content changed)
    #[must_use]
    pub fn matches_raw(
        &self,
        raw: &crate::config::raw::RawGlobalConfig,
    ) -> bool {
        self.latest_version().map_or(false, |latest| {
            let meta = &raw.metadata;
            // Compare created_at (latest has Option, meta has Option)
            latest.created_at() == meta.created_at
                // Compare modified_at (latest has SystemTime, meta has Option)
                && meta.modified_at.map_or(false, |mt| latest.modified_at() == mt)
                // Compare content hash
                && latest.content_hash()
                    == &meta.content_hash.unwrap_or([0; 32])
        })
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
/// use lithos_core::config::views::raw::RawFileVersion;
/// use std::time::SystemTime;
///
/// let content = b"vault_path = \"/vault\"\nname = \"My Vault\"";
/// let version = RawFileVersion::new(content)?;
/// assert_eq!(version.content_hash().len(), 32);
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct RawFileVersion {
    /// Compressed raw TOML content (zstd).
    ///
    /// Stored compressed to reduce database size. Typical config files
    /// compress well (70-80% reduction).
    compressed_content: Vec<u8>,

    /// BLAKE3 hash of the uncompressed content.
    ///
    /// Used for fast content-based change detection without decompressing.
    content_hash: [u8; 32],

    /// Filesystem birthtime (file creation timestamp).
    ///
    /// None if the filesystem doesn't support birthtime.
    #[rkyv(with = rkyv::with::Map<AsUnixTime>)]
    created_at: Option<SystemTime>,

    /// Filesystem mtime (last modification timestamp).
    #[rkyv(with = AsUnixTime)]
    modified_at: SystemTime,

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
    /// use lithos_core::config::views::raw::RawFileVersion;
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let version = RawFileVersion::new(
    ///     content,
    ///     None,
    ///     SystemTime::now(),
    /// )?;
    /// ```
    #[inline]
    pub fn new(
        content: &[u8],
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
    ) -> Result<Self, std::io::Error> {
        let content_hash = blake3::hash(content).into();
        let compressed_content = zstd::encode_all(content, 3)?;
        let recorded_at = SystemTime::now();

        Ok(Self {
            compressed_content,
            content_hash,
            created_at,
            modified_at,
            recorded_at,
        })
    }

    /// Returns the BLAKE3 content hash.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &[u8; 32] {
        &self.content_hash
    }

    /// Returns the file creation timestamp, if available.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the file modification timestamp.
    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> SystemTime {
        self.modified_at
    }

    /// Returns the recording timestamp.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Decompresses and returns the raw TOML content.
    ///
    /// # Errors
    ///
    /// Returns an error if decompression fails (corruption or invalid data).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::raw::RawFileVersion;
    ///
    /// let version = RawFileVersion::new(b"vault_path = \"/vault\"", None, std::time::SystemTime::now())?;
    /// let content = version.decompress()?;
    /// assert_eq!(content, b"vault_path = \"/vault\"");
    /// ```
    #[inline]
    pub fn decompress(&self) -> Result<Vec<u8>, std::io::Error> {
        zstd::decode_all(self.compressed_content.as_slice())
    }

    /// Checks if this version matches the given file state.
    ///
    /// Returns `true` if timestamps and content hash match.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::views::raw::RawFileVersion;
    /// use std::time::SystemTime;
    ///
    /// let content = b"vault_path = \"/vault\"";
    /// let mtime = SystemTime::now();
    /// let version = RawFileVersion::new(content, None, mtime)?;
    ///
    /// assert!(version.matches(None, mtime, &blake3::hash(content).into()));
    /// ```
    #[inline]
    #[must_use]
    pub fn matches(
        &self,
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
        content_hash: &[u8; 32],
    ) -> bool {
        self.created_at == created_at
            && self.modified_at == modified_at
            && &self.content_hash == content_hash
    }
}

// ----------------------------------------------------------- //
//                          Tests                              //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

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
        let version1 =
            RawFileVersion::new(b"content1", None, SystemTime::now())
                .expect("version creation should succeed");

        view.push_version(version1.clone());
        assert_eq!(view.versions().len(), 1);
        assert_eq!(view.latest_version(), Some(&version1));
    }

    #[test]
    fn raw_global_config_view_maintains_max_5_versions() {
        let mut view = RawGlobalConfigView::new("/path/to/global.toml".into());

        // Push 7 versions
        for i in 0i32..7i32 {
            let content = format!("content{i}");
            let version = RawFileVersion::new(
                content.as_bytes(),
                None,
                SystemTime::now(),
            )
            .expect("version creation should succeed");
            view.push_version(version);
        }

        assert_eq!(view.versions().len(), 5);
    }

    #[test]
    fn raw_vault_config_view_new() {
        let vault_id = VaultId::new();
        let view = RawVaultConfigView::new(
            vault_id,
            "/vault/.lithos/lithos.toml".into(),
        );
        assert_eq!(view.vault_id(), vault_id);
        assert_eq!(view.file_path(), "/vault/.lithos/lithos.toml");
        assert!(view.versions().is_empty());
    }

    #[test]
    fn raw_file_version_new() {
        let content = b"vault_path = \"/vault\"";
        let mtime = SystemTime::now();
        let version = RawFileVersion::new(content, None, mtime)
            .expect("version creation should succeed");

        let expected_hash: [u8; 32] = blake3::hash(content).into();
        assert_eq!(version.content_hash(), &expected_hash);
        assert_eq!(version.created_at(), None);
        assert_eq!(version.modified_at(), mtime);
    }

    #[test]
    fn raw_file_version_decompress() {
        let content = b"vault_path = \"/vault\"";
        let version = RawFileVersion::new(content, None, SystemTime::now())
            .expect("version creation should succeed");

        let decompressed =
            version.decompress().expect("decompression should succeed");
        assert_eq!(decompressed, content);
    }

    #[test]
    fn raw_file_version_matches() {
        let content = b"vault_path = \"/vault\"";
        let mtime = SystemTime::now();
        let hash = blake3::hash(content).into();
        let version = RawFileVersion::new(content, None, mtime)
            .expect("version creation should succeed");

        assert!(version.matches(None, mtime, &hash));

        // Different mtime should not match
        let different_mtime = SystemTime::UNIX_EPOCH;
        assert!(!version.matches(None, different_mtime, &hash));

        // Different hash should not match
        let different_hash = blake3::hash(b"different").into();
        assert!(!version.matches(None, mtime, &different_hash));
    }

    #[test]
    fn raw_file_version_round_trips_through_rkyv() {
        let content = b"vault_path = \"/vault\"";
        let original = RawFileVersion::new(content, None, SystemTime::now())
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
        assert_eq!(deserialized.created_at, original.created_at);
        assert_eq!(deserialized.modified_at, original.modified_at);
        assert_eq!(
            deserialized.compressed_content,
            original.compressed_content
        );
    }
}
