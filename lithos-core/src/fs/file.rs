//! Type-safe wrappers for files and their metadata.
//!
//! Provides the [`FileName`] and [`FileEntry`] types for
//! capturing and processing file information in a way that is compatible with
//! zero-copy storage.
//!
//! ## Usage
//!
//! These types are primarily used by:
//! - [`crate::fs::scanner::DirScanner`]: Returns `Vec<FileEntry>` from
//!   directory scans
//! - [`crate::fs::reader::Reader`]: Uses `FileEntry` in `list_entries()` method
//! - Domain contexts: Store and query file metadata with zero-copy access via
//!   rkyv

#![expect(
    clippy::module_name_repetitions,
    reason = "File* names are intentional and clear in this file-specific \
              module"
)]

pub use super::name::FileName;

/// A general-purpose filesystem entry.
///
/// Captures path, metadata, and filename in a single structure, suitable for
/// unified filesystem processing.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FileEntry {
    /// The path to the entry.
    pub path: std::path::PathBuf,
    /// The entry's filename.
    pub filename: FileName,
    /// The entry's metadata.
    pub metadata: super::metadata::FileMetadata,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn basename_extracts_filename_without_extension() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.basename_str(), "note");
    }

    #[test]
    fn basename_handles_hyphens() {
        let filename = FileName::new("base-note.toml".into());
        assert_eq!(filename.basename_str(), "base-note");
    }

    #[test]
    fn extension_returns_file_extension() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.extension(), Some("toml"));
    }

    #[test]
    fn extension_returns_none_for_no_extension() {
        let filename = FileName::new("note".into());
        assert_eq!(filename.extension(), None);
    }

    #[test]
    fn as_str_returns_full_filename() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.as_str(), "note.toml");
    }

    #[test]
    fn try_from_path_extracts_filename() {
        let path = Path::new("schemas/user.json");
        let filename = FileName::try_from(path).unwrap();
        assert_eq!(filename.as_str(), "user.json");
        assert_eq!(filename.basename_str(), "user");
    }

    #[test]
    fn try_from_path_rejects_empty_filename() {
        let path = Path::new("schemas/..");
        let result: Result<FileName, _> = FileName::try_from(path);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
}
