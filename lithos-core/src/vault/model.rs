//! Vault domain types for files and folders.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    time::SystemTime,
};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::error::VaultPathError;
use crate::{
    fs::{
        DirMetadata, DirName, FileFormat, FileMetadata, FileName, PathValidator,
    },
    support::Blake3Hash,
    utils::UuidV7,
};

/// UUID-based file identifier for vault entries.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct FileId(UuidV7);

impl FileId {
    /// Creates a new random file identifier.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(UuidV7::new())
    }

    /// Returns the inner UUID wrapper.
    #[inline]
    #[must_use]
    pub const fn as_uuid_v7(&self) -> &UuidV7 {
        &self.0
    }

    /// Returns the underlying UUID bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for FileId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// UUID-based directory identifier for vault entries.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct DirId(UuidV7);

impl DirId {
    /// Creates a new random directory identifier.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(UuidV7::new())
    }

    /// Returns the inner UUID wrapper.
    #[inline]
    #[must_use]
    pub const fn as_uuid_v7(&self) -> &UuidV7 {
        &self.0
    }

    /// Returns the underlying UUID bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for DirId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Normalized, vault-relative path using forward slashes.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NormalizedPath(Box<str>);

impl NormalizedPath {
    /// Creates a new normalized vault-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] when path validation fails.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, VaultPathError> {
        let normalized = VaultPath::normalize(path);
        let normalized = normalized.as_ref().trim();
        PathValidator::validate_vault_path(normalized, None)
            .map_err(VaultPathError::from)?;
        Ok(Self(normalized.into()))
    }

    /// Returns the normalized path string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// File entry view with stable inode identity.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct FileView {
    id: FileId,
    parent_id: Option<DirId>,
    name: FileName,
    format: FileFormat,
    metadata: FileMetadata,
    content_hash: Blake3Hash,
}

impl FileView {
    /// Creates a new file view.
    #[expect(
        clippy::too_many_arguments,
        reason = "FileView requires explicit construction of domain fields"
    )]
    #[inline]
    #[must_use]
    pub fn new(
        id: FileId,
        parent_id: Option<DirId>,
        name: FileName,
        format: FileFormat,
        metadata: FileMetadata,
        content_hash: [u8; 32],
    ) -> Self {
        Self {
            id,
            parent_id,
            name,
            format,
            metadata,
            content_hash: Blake3Hash::from(content_hash),
        }
    }

    /// Returns file identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> FileId {
        self.id
    }

    /// Returns parent directory identifier, if available.
    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<DirId> {
        self.parent_id
    }

    /// Returns file name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &FileName {
        &self.name
    }

    /// Returns file format.
    #[inline]
    #[must_use]
    pub const fn format(&self) -> FileFormat {
        self.format
    }

    /// Returns file metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Returns file content hash bytes.
    #[inline]
    #[must_use]
    pub const fn content_hash(&self) -> &[u8; 32] {
        self.content_hash.as_bytes()
    }
}

/// Directory entry view with stable inode identity.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct DirView {
    id: DirId,
    parent_id: Option<DirId>,
    name: DirName,
    metadata: DirMetadata,
}

impl DirView {
    /// Creates a new directory view.
    #[inline]
    #[must_use]
    pub fn new(
        id: DirId,
        parent_id: Option<DirId>,
        name: DirName,
        metadata: DirMetadata,
    ) -> Self {
        Self {
            id,
            parent_id,
            name,
            metadata,
        }
    }

    /// Returns directory identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> DirId {
        self.id
    }

    /// Returns parent directory identifier, if available.
    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<DirId> {
        self.parent_id
    }

    /// Returns directory name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &DirName {
        &self.name
    }

    /// Returns directory metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &DirMetadata {
        &self.metadata
    }
}

/// Unified vault entry view for files and directories.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub enum FsEntryView {
    /// File entry view.
    File(FileView),
    /// Directory entry view.
    Dir(DirView),
}

impl FsEntryView {
    /// Returns the entry ID as UUID bytes.
    #[inline]
    #[must_use]
    pub const fn id_bytes(&self) -> &[u8; 16] {
        match self {
            Self::File(view) => view.id.as_bytes(),
            Self::Dir(view) => view.id.as_bytes(),
        }
    }

    /// Returns the parent directory ID, if available.
    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<&DirId> {
        match self {
            Self::File(view) => view.parent_id.as_ref(),
            Self::Dir(view) => view.parent_id.as_ref(),
        }
    }

    /// Returns the entry name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::File(view) => view.name.as_str(),
            Self::Dir(view) => view.name.as_str(),
        }
    }

    /// Returns `true` when the entry is a file.
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Returns `true` when the entry is a directory.
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }
}

/// Validated vault-relative path.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultPath(Box<str>);

impl VaultPath {
    /// Creates a new validated vault path.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if the path is invalid.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, VaultPathError> {
        let normalized = Self::normalize(path);
        let normalized = normalized.as_ref().trim();
        PathValidator::validate_vault_path(normalized, None)
            .map_err(VaultPathError::from)?;
        Ok(Self(normalized.into()))
    }

    /// Creates a validated vault path from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if the path is invalid or not UTF-8.
    #[inline]
    pub fn try_from_path(path: &Path) -> Result<Self, VaultPathError> {
        let path_str = path.to_str().ok_or_else(|| {
            VaultPathError::InvalidPathEncoding {
                path: path.to_path_buf(),
            }
        })?;
        Self::try_new(path_str)
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the path as a `Path` reference.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.0.as_ref())
    }

    #[inline]
    fn normalize(path: &str) -> Cow<'_, str> {
        if path.contains('\\') {
            let mut owned = String::with_capacity(path.len());
            for ch in path.chars() {
                if ch == '\\' {
                    owned.push('/');
                } else {
                    owned.push(ch);
                }
            }
            Cow::Owned(owned)
        } else {
            Cow::Borrowed(path)
        }
    }
}

/// File metadata tracked for a vault entry.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultFile {
    path: VaultPath,
    basename: Box<str>,
    filename: Box<str>,
    parent: Box<str>,
    extension: Option<Box<str>>,
    size: u64,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl VaultFile {
    /// Builds a vault file entry from a path and filesystem metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if path components are invalid or UTF-8
    /// conversion fails.
    #[inline]
    pub fn try_new(
        path: VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<Self, VaultPathError> {
        let parts = PathParts::try_new(path.as_path())?;
        Ok(Self {
            path,
            basename: parts.basename,
            filename: parts.filename,
            parent: parts.parent,
            extension: parts.extension,
            size: metadata.len(),
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().ok(),
        })
    }

    /// Returns the vault-relative path for this file.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    /// Returns the filename without extension.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        &self.basename
    }

    /// Returns the filename with extension.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the parent folder path (empty for root).
    #[inline]
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Returns the file extension, if any.
    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Returns the file size in bytes.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the file creation timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the file modification timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }
}

/// Folder metadata tracked for a vault entry.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultFolder {
    path: VaultPath,
    basename: Box<str>,
    parent: Box<str>,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl VaultFolder {
    /// Builds a vault folder entry from a path and filesystem metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if path components are invalid or UTF-8
    /// conversion fails.
    #[inline]
    pub fn try_new(
        path: VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<Self, VaultPathError> {
        let parts = FolderParts::try_new(path.as_path())?;
        Ok(Self {
            path,
            basename: parts.basename,
            parent: parts.parent,
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().ok(),
        })
    }

    /// Returns the vault-relative path for this folder.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    /// Returns the folder name.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        &self.basename
    }

    /// Returns the parent folder path (empty for root).
    #[inline]
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Returns the folder creation timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the folder modification timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }
}

struct PathParts {
    basename: Box<str>,
    filename: Box<str>,
    parent: Box<str>,
    extension: Option<Box<str>>,
}

impl PathParts {
    fn try_new(path: &Path) -> Result<Self, VaultPathError> {
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let basename = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let extension =
            path.extension().and_then(|value| value.to_str()).map(Into::into);

        let parent =
            path.parent().and_then(|value| value.to_str()).unwrap_or("");

        Ok(Self {
            basename: basename.into(),
            filename: filename.into(),
            parent: parent.into(),
            extension,
        })
    }
}

struct FolderParts {
    basename: Box<str>,
    parent: Box<str>,
}

impl FolderParts {
    fn try_new(path: &Path) -> Result<Self, VaultPathError> {
        let basename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let parent =
            path.parent().and_then(|value| value.to_str()).unwrap_or("");

        Ok(Self {
            basename: basename.into(),
            parent: parent.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FsTimes;

    #[test]
    fn file_and_dir_ids_generate_v7_uuid_bytes() {
        let file_id = FileId::new();
        let dir_id = DirId::new();

        assert_eq!(file_id.as_bytes().len(), 16);
        assert_eq!(dir_id.as_bytes().len(), 16);
    }

    #[test]
    fn normalized_path_normalizes_separators_and_validates() {
        let normalized =
            NormalizedPath::try_new("notes\\daily\\today.md").expect("ok");
        assert_eq!(normalized.as_str(), "notes/daily/today.md");

        assert!(NormalizedPath::try_new("../outside.md").is_err());
    }

    #[test]
    fn file_and_dir_view_construction_preserves_fields() {
        let parent_id = DirId::new();
        let file_id = FileId::new();
        let file_meta = FileMetadata::new(FsTimes::new(None, None), 42, false);
        let file_name = FileName::new("note.md".into());
        let file = FileView::new(
            file_id,
            Some(parent_id),
            file_name,
            FileFormat::Markdown,
            file_meta,
            [7u8; 32],
        );

        assert_eq!(file.id(), file_id);
        assert_eq!(file.parent_id(), Some(parent_id));
        assert_eq!(file.name().as_str(), "note.md");
        assert_eq!(file.format(), FileFormat::Markdown);
        assert_eq!(*file.content_hash(), [7u8; 32]);

        let dir_meta = DirMetadata::new(FsTimes::new(None, None), false);
        let dir_name = DirName::new("notes".into());
        let dir = DirView::new(parent_id, None, dir_name, dir_meta);

        assert_eq!(dir.id(), parent_id);
        assert_eq!(dir.parent_id(), None);
        assert_eq!(dir.name().as_str(), "notes");
    }

    #[test]
    fn fs_entry_view_helpers_dispatch_by_variant() {
        let parent_id = DirId::new();
        let file = FsEntryView::File(FileView::new(
            FileId::new(),
            Some(parent_id),
            FileName::new("note.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 1, false),
            [1u8; 32],
        ));
        let dir = FsEntryView::Dir(DirView::new(
            DirId::new(),
            None,
            DirName::new("root".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        ));

        assert!(file.is_file());
        assert!(!file.is_dir());
        assert_eq!(file.parent_id(), Some(&parent_id));
        assert_eq!(file.name(), "note.md");
        assert_eq!(file.id_bytes().len(), 16);

        assert!(dir.is_dir());
        assert!(!dir.is_file());
        assert_eq!(dir.parent_id(), None);
        assert_eq!(dir.name(), "root");
        assert_eq!(dir.id_bytes().len(), 16);
    }

    #[test]
    fn fs_entry_view_supports_rkyv_roundtrip() {
        let view = FsEntryView::Dir(DirView::new(
            DirId::new(),
            None,
            DirName::new("root".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        ));

        let bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(&view).expect("serialize");
        let archived =
            rkyv::access::<ArchivedFsEntryView, rkyv::rancor::Error>(&bytes)
                .expect("archive access");

        assert!(matches!(archived, ArchivedFsEntryView::Dir(_)));
    }
}
