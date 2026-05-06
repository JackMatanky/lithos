//! Persisted views for zero-copy schema discovery.
//!
//! Provides the [`RawSchemaView`] and [`RawPropertyBankView`] types which
//! manage the version history of schemas and property bank files. These
//! structures are designed for efficient staleness detection and incremental
//! updates by storing historical metadata (hashes, timestamps) in a way
//! that is compatible with zero-copy database access.

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    fs::{FileInfo, RelativePath},
    schema::{
        error::{SchemaIngestionError, SchemaStorageError},
        raw::{RawPropertyBank, RawSchema},
        views::{
            contracts::{RawView, RawViewRead, Version, VersionRead as _},
            hashes::HashRecord,
            snapshots::{
                ArchivedPropertyBankVersion, ArchivedSchemaVersion,
                PropertyBankVersion, SchemaVersion,
            },
        },
    },
    support::hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  RawSchemaView
// ─────────────────────────────────────────────────────────────────────────────

/// Manages the version history and identity metadata for a schema file.
///
/// This structure captures the stable mapping between a filesystem path and
/// a versioned history of its contents. It enables:
/// 1. **Staleness detection**: Fast timestamp checks followed by exact hash
///    checks.
/// 2. **Incremental updates**: Identifying which properties changed between
///    versions.
/// 3. **Stable Identity**: Maintaining the same ID for a schema even if its
///    content or filename changes (within configured boundaries).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// The vault-relative path to the schema file.
    path: RelativePath,
    /// Ring buffer of historical versions (first is current).
    versions: Vec<SchemaVersion>,
}

impl RawSchemaView {
    /// Creates a new schema view with an initial version.
    #[inline]
    #[must_use]
    pub fn new(path: RelativePath, initial_version: SchemaVersion) -> Self {
        let mut versions = Vec::with_capacity(Self::MAX_VERSIONS);
        versions.push(initial_version);

        Self {
            path,
            versions,
        }
    }

    /// Returns the relative path of the schema file.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// Returns the schema file's basename (filename without an extension)
    /// derived from path.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.path
            .as_path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
    }

    /// Creates a view from a raw schema with hashes.
    ///
    /// Use this when you have computed hashes and want to persist a new
    /// versioned view for staleness checks.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if metadata is missing or validation
    /// fails.
    #[inline]
    pub fn try_from_raw_with_hashes(
        raw: &RawSchema,
        path: RelativePath,
        hashes: HashRecord,
    ) -> Result<Self, SchemaIngestionError> {
        let info = *raw.info();
        let version = SchemaVersion::new(info, hashes, raw)?;

        Ok(Self::new(path, version))
    }
}

/// Implements [`RawView`] for [`RawSchemaView`].
impl RawView for RawSchemaView {
    type FilePath = RelativePath;
    type Version = SchemaVersion;

    #[inline]
    fn file_path(&self) -> &Self::FilePath {
        &self.path
    }

    #[inline]
    fn current(&self) -> Option<&Self::Version> {
        self.versions.first()
    }

    #[inline]
    fn current_mut(&mut self) -> Option<&mut Self::Version> {
        self.versions.first_mut()
    }

    #[inline]
    fn add_version(&mut self, version: Self::Version) {
        if self.versions.len() >= Self::MAX_VERSIONS {
            self.versions.pop();
        }
        self.versions.insert(0, version);
    }

    #[inline]
    fn update_file_info(&mut self, info: FileInfo) {
        if let Some(current) = self.current_mut() {
            current.set_file_info(info);
        }
    }

    #[inline]
    fn update_timestamps(
        &mut self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) {
        if let Some(current) = self.current_mut() {
            let size = Version::file_info(current).size();
            current.set_file_info(FileInfo::new(created_at, modified_at, size));
        }
    }

    /// Adds a new version with updated content hash.
    ///
    /// Keeps property hashes from the current version.
    #[inline]
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        let current = self.current().ok_or(SchemaStorageError::NotFound {
            name: "current schema version".into(),
        })?;
        let info = *Version::file_info(current);
        let hashes = HashRecord::new(
            content_hash,
            current.hashes().properties().clone(),
        );
        let version = SchemaVersion::with_metadata(current, info, hashes);
        self.add_version(version);
        Ok(())
    }
}

/// Implements [`RawViewRead`] for [`RawSchemaView`].
impl RawViewRead for RawSchemaView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Implements [`RawViewRead`] for [`ArchivedRawSchemaView`] (zero-copy).
impl RawViewRead for ArchivedRawSchemaView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions.first().is_some_and(|v: &ArchivedSchemaVersion| {
            v.is_content_match(content_hash)
        })
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.versions.first().is_some_and(|v: &ArchivedSchemaVersion| {
            v.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  RawPropertyBankView
// ─────────────────────────────────────────────────────────────────────────────

/// Manages the version history and identity metadata for the property bank.
///
/// This structure captures the stable mapping between the property bank path
/// and its version history. It enables fast staleness detection for the global
/// property library.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// The vault-relative path to the property bank file.
    path: RelativePath,
    /// Ring buffer of historical versions (first is current).
    versions: Vec<PropertyBankVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with an initial version.
    #[inline]
    #[must_use]
    pub fn new(
        path: RelativePath,
        initial_version: PropertyBankVersion,
    ) -> Self {
        let mut versions = Vec::with_capacity(Self::MAX_VERSIONS);
        versions.push(initial_version);

        Self {
            path,
            versions,
        }
    }

    /// Returns the property bank path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// Creates a view from a raw property bank with hashes.
    ///
    /// Use this when you have computed hashes and want to persist a new
    /// versioned view for staleness checks.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if metadata is missing or validation
    /// fails.
    #[inline]
    pub fn try_from_raw_with_hashes(
        raw: &RawPropertyBank,
        path: RelativePath,
        raw_hash: HashRecord,
    ) -> Result<Self, SchemaIngestionError> {
        let info = *raw.info();

        let version = PropertyBankVersion::new(info, raw_hash, raw)?;

        Ok(Self::new(path, version))
    }
}

/// Implements [`RawView`] for [`RawPropertyBankView`].
impl RawView for RawPropertyBankView {
    type FilePath = RelativePath;
    type Version = PropertyBankVersion;

    #[inline]
    fn file_path(&self) -> &Self::FilePath {
        &self.path
    }

    #[inline]
    fn current(&self) -> Option<&Self::Version> {
        self.versions.first()
    }

    #[inline]
    fn current_mut(&mut self) -> Option<&mut Self::Version> {
        self.versions.first_mut()
    }

    #[inline]
    fn add_version(&mut self, version: Self::Version) {
        if self.versions.len() >= Self::MAX_VERSIONS {
            self.versions.pop();
        }
        self.versions.insert(0, version);
    }

    #[inline]
    fn update_file_info(&mut self, info: FileInfo) {
        if let Some(current) = self.current_mut() {
            current.set_file_info(info);
        }
    }

    /// Updates timestamps on the current version, preserving file size.
    #[inline]
    fn update_timestamps(
        &mut self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) {
        if let Some(current) = self.current_mut() {
            let size = Version::file_info(current).size();
            current.set_file_info(FileInfo::new(created_at, modified_at, size));
        }
    }

    /// Adds a new version with updated content hash.
    ///
    /// Preserves property hashes from the current version.
    /// Delegates to [`RawPropertyBankView::update_content_hash`].
    #[inline]
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        let current = self.current().ok_or(SchemaStorageError::NotFound {
            name: "current property bank version".into(),
        })?;
        let info = *Version::file_info(current);
        let hashes = HashRecord::new(
            content_hash,
            current.hashes().properties().clone(),
        );

        let version = PropertyBankVersion::with_metadata(current, info, hashes);
        self.add_version(version);
        Ok(())
    }
}

/// Implements [`RawViewRead`] for [`RawPropertyBankView`].
impl RawViewRead for RawPropertyBankView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Implements [`RawViewRead`] for [`ArchivedRawPropertyBankView`] (zero-copy).
impl RawViewRead for ArchivedRawPropertyBankView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions.first().is_some_and(|v: &ArchivedPropertyBankVersion| {
            v.is_content_match(content_hash)
        })
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.versions.first().is_some_and(|v: &ArchivedPropertyBankVersion| {
            v.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod raw_schema_view {
        use super::*;
        use crate::schema::{
            raw::{RawPropertyMap, RawSchemaVersion},
            views::RawPropertyMapHash,
        };

        #[test]
        fn supports_zero_copy_staleness_checks() {
            let path = RelativePath::try_from("schemas/note.json").unwrap();
            let info = FileInfo::new(None, None, 100);
            let hashes = HashRecord::new(
                Blake3Hash::new([0; 32]),
                RawPropertyMapHash::default(),
            );
            let raw = RawSchema {
                version: RawSchemaVersion::default(),
                name: "Note".into(),
                extends: None,
                excludes: vec![],
                properties: RawPropertyMap::new(),
                info: FileInfo::new(None, None, 0),
            };

            let version =
                SchemaVersion::new(info, hashes.clone(), &raw).unwrap();
            let view = RawSchemaView::new(path, version);

            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view).unwrap();
            let archived = rkyv::access::<
                ArchivedRawSchemaView,
                rkyv::rancor::Error,
            >(&bytes)
            .unwrap();

            assert!(archived.is_content_match(hashes.content()));
            assert!(archived.is_timestamp_match(None, None));
            assert_eq!(archived.version_count(), 1);
        }

        #[test]
        fn update_file_info_replaces_full_metadata() {
            let path = RelativePath::try_from("schemas/note.json").unwrap();
            let info = FileInfo::new(None, None, 100);
            let hashes = HashRecord::new(
                Blake3Hash::new([0; 32]),
                RawPropertyMapHash::default(),
            );
            let raw = RawSchema {
                version: RawSchemaVersion::default(),
                name: "Note".into(),
                extends: None,
                excludes: vec![],
                properties: RawPropertyMap::new(),
                info: FileInfo::new(None, None, 0),
            };

            let version = SchemaVersion::new(info, hashes, &raw).unwrap();
            let mut view = RawSchemaView::new(path, version);

            let replacement = FileInfo::new(Some(SystemTime::now()), None, 500);
            view.update_file_info(replacement);

            let current = view.current().unwrap();
            assert_eq!(current.info(), &replacement);
        }
    }

    mod raw_property_bank_view {
        use super::*;
        use crate::schema::{
            raw::{RawPropertyMap, RawSchemaVersion},
            views::RawPropertyMapHash,
        };

        #[test]
        fn supports_zero_copy_staleness_checks() {
            let path =
                RelativePath::try_from("schemas/property_bank.json").unwrap();
            let info = FileInfo::new(None, None, 100);
            let hashes = HashRecord::new(
                Blake3Hash::new([0; 32]),
                RawPropertyMapHash::default(),
            );
            let raw = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: RawPropertyMap::new(),
                info: FileInfo::new(None, None, 0),
            };

            let version =
                PropertyBankVersion::new(info, hashes.clone(), &raw).unwrap();
            let view = RawPropertyBankView::new(path, version);

            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view).unwrap();
            let archived = rkyv::access::<
                ArchivedRawPropertyBankView,
                rkyv::rancor::Error,
            >(&bytes)
            .unwrap();

            assert!(archived.is_content_match(hashes.content()));
            assert!(archived.is_timestamp_match(None, None));
            assert_eq!(archived.version_count(), 1);
        }
    }
}
