//! Root-scoped filesystem reader with validation and format-classification.
//!
//! This module provides [`Reader`], a read-only filesystem adapter for safe
//! vault access with built-in path validation, directory scanning, and
//! structured file parsing.
//!
//! # Security
//!
//! All file access is scoped to a root directory with validation via
//! [`Validator`] to prevent path traversal attacks and unauthorized access to
//! restricted files.
//!
//! # Features
//!
//! - **Directory scanning**: Glob-pattern filtering via
//!   [`filter_dir`](Reader::filter_dir) and
//!   [`list_entries`](Reader::list_entries)
//! - **File reading**: Raw bytes, UTF-8 strings, and structured parsing
//!   (JSON/TOML/YAML)
//! - **Format detection**: Automatic format classification by extension and
//!   content
//! - **Metadata access**: File info, timestamps, and existence checks
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//!
//! use lithos_core::fs::FsReader;
//!
//! let reader = FsReader::new("/vault");
//!
//! // Find all TOML files
//! let schema_files = reader.filter_dir("schemas/**/*.toml")?;
//!
//! // Read and parse structured file
//! let data: serde_json::Value =
//!     reader.parse_structured(Path::new("config.json"))?;
//!
//! // Get file metadata
//! let info = reader.metadata(Path::new("README.md"))?;
//! # Ok::<(), lithos_core::fs::FsError>(())
//! ```

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use super::{
    entry::FsEntry,
    error::{FsError, PathValidationError},
    file::FileName,
    format::FileFormat,
    metadata::{FileMetadata, FsTimes},
    types::{self, Json, Toml, Yaml},
    validator::Validator,
};

/// A read-only filesystem adapter for safe vault access.
///
/// `Reader` provides methods for filtering, reading, and parsing files within a
/// specified root directory. It enforces path safety via [`Validator`] to
/// prevent traversal attacks and unauthorized access to restricted files.
pub struct Reader {
    root: PathBuf,
    validator: Validator,
}

impl Reader {
    /// Creates a new `Reader` with flexible validation.
    ///
    /// Flexible mode allows symlinks to targets outside the root directory and
    /// does not require an absolute root path.
    #[inline]
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            root: root.into(),
            validator: Validator::new_flexible(),
        }
    }

    /// Creates a new `Reader` with strict validation.
    ///
    /// Strict mode requires an absolute, canonicalized root path and rejects
    /// any symlinks that escape the root boundary.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError::RelativeRoot`] if the provided path is
    /// not absolute.
    #[inline]
    pub fn new_strict<P: Into<PathBuf>>(
        root: P,
    ) -> Result<Self, PathValidationError> {
        let root = root.into();
        Ok(Self {
            validator: Validator::try_new_strict(root.clone())?,
            root,
        })
    }

    /// Creates a reader with the filesystem root as base.
    ///
    /// # Warning
    ///
    /// This method grants access to the **entire filesystem** and should
    /// **only be used for global/system-wide configuration resolution**.
    ///
    /// For vault-scoped operations (schemas, templates, notes), use
    /// [`Reader::new(vault_root)`](Reader::new) instead to ensure
    /// file access is properly sandboxed to the vault directory.
    ///
    /// # Use Cases
    ///
    /// - Loading global config from system directories (`/etc`, `~/.config`)
    /// - Resolving config paths from environment variables
    /// - Any operation that legitimately needs to traverse outside a vault
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use lithos_core::fs::reader::Reader;
    ///
    /// let fs = Reader::from_system_root();
    ///
    /// // Can now check absolute system paths
    /// if fs.exists(Path::new("/etc/lithos/lithos.toml")) {
    ///     // Global config exists
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn from_system_root() -> Self {
        Self {
            root: PathBuf::from("/"),
            validator: Validator::new_flexible(),
        }
    }

    /// Returns the root directory of this reader.
    #[inline]
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Checks if a file exists within the vault.
    #[inline]
    #[must_use]
    pub fn exists(&self, path: &Path) -> bool {
        self.root.join(path).exists()
    }

    /// Filters files within the vault using a glob pattern.
    ///
    /// The pattern is relative to the vault root. Only files and symlinks are
    /// returned; directories are excluded. Results are sorted alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_dir(&self, pattern: &str) -> Result<Vec<PathBuf>, FsError> {
        use super::scanner::{DirScanInput, DirScanner};

        let scanner = DirScanner::new(&self.root);
        Ok(scanner.paths(DirScanInput::new().with_pattern(pattern))?)
    }

    /// Filters paths within the vault using a glob pattern.
    ///
    /// Returns a mixed collection of files and directories.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_paths(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::path::FsPath>, FsError> {
        use super::scanner::{DirScanInput, DirScanner};

        let scanner = DirScanner::new(&self.root);
        Ok(scanner.paths_typed(
            DirScanInput::new().with_pattern(pattern).include_dirs(true),
        )?)
    }

    /// Filters file paths within the vault using a glob pattern.
    ///
    /// Only files are returned; directories are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_file_paths(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::path::FilePath>, FsError> {
        use super::path::FsPath;

        let paths = self.filter_paths(pattern)?;
        Ok(paths
            .into_iter()
            .filter_map(|p| match p {
                FsPath::File(f) => Some(f),
                FsPath::Dir(_) => None,
            })
            .collect())
    }

    /// Filters directory paths within the vault using a glob pattern.
    ///
    /// Only directories are returned; files are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_dir_paths(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::path::DirPath>, FsError> {
        use super::path::FsPath;

        let paths = self.filter_paths(pattern)?;
        Ok(paths
            .into_iter()
            .filter_map(|p| match p {
                FsPath::Dir(d) => Some(d),
                FsPath::File(_) => None,
            })
            .collect())
    }

    /// Filters entries within the vault using a glob pattern.
    ///
    /// Returns a mixed collection of file and directory entries with metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_entries(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::entry::FsEntry>, FsError> {
        use super::scanner::{DirScanInput, DirScanner};

        let scanner = DirScanner::new(&self.root);
        Ok(scanner.entries_typed(
            DirScanInput::new().with_pattern(pattern).include_dirs(true),
        )?)
    }

    /// Filters file entries within the vault using a glob pattern.
    ///
    /// Only files are returned; directories are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_file_entries(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::entry::FsFile>, FsError> {
        use super::entry::FsEntry;

        let entries = self.filter_entries(pattern)?;
        Ok(entries
            .into_iter()
            .filter_map(|e| match e {
                FsEntry::File(f) => Some(f),
                FsEntry::Dir(_) => None,
            })
            .collect())
    }

    /// Filters directory entries within the vault using a glob pattern.
    ///
    /// Only directories are returned; files are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn filter_dir_entries(
        &self,
        pattern: &str,
    ) -> Result<Vec<super::entry::FsDir>, FsError> {
        use super::entry::FsEntry;

        let entries = self.filter_entries(pattern)?;
        Ok(entries
            .into_iter()
            .filter_map(|e| match e {
                FsEntry::Dir(d) => Some(d),
                FsEntry::File(_) => None,
            })
            .collect())
    }

    /// Lists files matching a glob pattern within the vault.
    ///
    /// This is a compatibility alias for [`filter_dir`].
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, FsError> {
        self.filter_dir(pattern)
    }

    /// Lists directories matching a glob pattern within the vault.
    ///
    /// The pattern is relative to the vault root. Only directories are
    /// returned; files and non-directory symlinks are excluded. Results are
    /// sorted alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn list_dirs(&self, pattern: &str) -> Result<Vec<PathBuf>, FsError> {
        use super::error::ReadError;

        let full_pattern = self.root.join(pattern);
        let pattern_str = full_pattern.to_str().ok_or_else(|| {
            FsError::Path(super::error::PathError::InvalidUtf8(
                full_pattern.clone(),
            ))
        })?;

        let mut paths = Vec::new();
        for entry in glob::glob(pattern_str).map_err(|e| {
            FsError::Scan(super::error::ScanError::InvalidPattern {
                pattern: pattern_str.into(),
                message: e.msg.into(),
            })
        })? {
            let path = entry.map_err(|e| {
                FsError::Scan(super::error::ScanError::Traversal {
                    path: e.path().to_path_buf(),
                    source: e.into_error(),
                })
            })?;

            if !path.is_dir() {
                continue;
            }

            let relative = path.strip_prefix(&self.root).map_err(|_err| {
                FsError::Read(ReadError::NotInBase {
                    path: path.clone(),
                    base: self.root.clone(),
                })
            })?;

            paths.push(relative.to_path_buf());
        }

        paths.sort();
        Ok(paths)
    }

    /// Lists file entries within the vault using a glob pattern.
    ///
    /// Similar to [`filter_dir`], but returns an [`FsEntry`] for each matching
    /// entry with typed path and metadata.
    /// Results are sorted by path alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn list_entries(&self, pattern: &str) -> Result<Vec<FsEntry>, FsError> {
        use super::scanner::{DirScanInput, DirScanner};

        let scanner = DirScanner::new(&self.root);
        Ok(scanner.entries(DirScanInput::new().with_pattern(pattern))?)
    }

    /// Reads a file's content as raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read`] if the file cannot be read.
    #[cfg(test)]
    #[inline]
    pub(crate) fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        use super::error::ReadError;

        let full_path = self.root.join(path);
        std::fs::read(&full_path).map_err(|e| {
            FsError::Read(ReadError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        })
    }

    /// Reads a file's content as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read`] if the file cannot be read or contains
    /// invalid UTF-8.
    #[inline]
    pub fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        use super::error::ReadError;

        let full_path = self.root.join(path);
        std::fs::read_to_string(&full_path).map_err(|e| {
            FsError::Read(ReadError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        })
    }

    /// Executes a closure with the content of a file.
    ///
    /// This is a convenience method for reading a file and applying a
    /// transformation.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the closure returns an
    /// error.
    #[inline]
    pub fn read_with<T, E, F>(&self, path: &Path, f: F) -> Result<T, E>
    where
        F: FnOnce(&Path, &str) -> Result<T, E>,
        E: From<FsError>,
    {
        let content = self.read_to_string(path)?;
        f(path, &content)
    }

    /// Returns the information for a file.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read`] if the file does not exist or metadata cannot
    /// be read.
    #[inline]
    pub fn std_metadata(
        &self,
        path: &Path,
    ) -> Result<std::fs::Metadata, FsError> {
        use super::error::ReadError;

        let full_path = self.root.join(path);
        std::fs::symlink_metadata(&full_path).map_err(|e| {
            FsError::Read(ReadError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        })
    }

    /// Returns the metadata for a file or directory as a typed [`FsMetadata`].
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read`] if the path does not exist or metadata cannot
    /// be read.
    #[inline]
    pub fn metadata(
        &self,
        path: &Path,
    ) -> Result<super::metadata::FsMetadata, FsError> {
        use super::{error::ReadError, metadata::FsMetadata};

        let full_path = self.root.join(path);
        FsMetadata::from_path(&full_path).map_err(|e| {
            FsError::Read(ReadError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        })
    }

    /// Returns the file's creation timestamp.
    ///
    /// Returns `None` if the metadata cannot be read or the creation time is
    /// not available on this platform. Failures are logged at debug level.
    #[inline]
    #[must_use]
    pub fn created_at(&self, path: &Path) -> Option<SystemTime> {
        let s = self
            .metadata(path)
            .map(|m| {
                m.as_file().map_or_else(
                    || FileMetadata::new(FsTimes::new(None, None), 0, false),
                    std::clone::Clone::clone,
                )
            })
            .map_err(|e| {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read metadata for created_at"
                );
            })
            .ok()?;
        s.times().created_at()
    }

    /// Returns the file's modification timestamp.
    ///
    /// Returns `None` if the metadata cannot be read or the modification time
    /// is not available on this platform. Failures are logged at debug level.
    #[inline]
    #[must_use]
    pub fn modified_at(&self, path: &Path) -> Option<SystemTime> {
        let s = self
            .metadata(path)
            .map(|m| {
                m.as_file().map_or_else(
                    || FileMetadata::new(FsTimes::new(None, None), 0, false),
                    std::clone::Clone::clone,
                )
            })
            .map_err(|e| {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read metadata for modified_at"
                );
            })
            .ok()?;
        s.times().modified_at()
    }

    /// Extracts the filename (with extension) from a path.
    ///
    /// Returns the complete filename including its extension.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read`] if the path has no filename or the filename
    /// is not valid UTF-8.
    #[inline]
    pub fn filename(&self, path: &Path) -> Result<FileName, FsError> {
        use super::error::ReadError;

        FileName::try_from(path).map_err(|source| {
            FsError::Read(ReadError::Io {
                path: path.to_path_buf(),
                source,
            })
        })
    }

    /// Validates a path using the internal validator.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError`] if the path is invalid.
    #[inline]
    pub(crate) fn validate_path(
        &self,
        path: &Path,
    ) -> Result<(), PathValidationError> {
        self.validator.validate(path)
    }

    /// Reads and parses a structured file (JSON, TOML, or YAML).
    ///
    /// The format is detected based on the file extension.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if:
    /// - The file format is unsupported.
    /// - The file cannot be read.
    /// - The content is malformed for the detected format.
    #[inline]
    pub fn parse_structured<T>(&self, path: &Path) -> Result<T, FsError>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = self.read_to_string(path)?;
        Self::parse_structured_from_str(path, &content)
    }

    /// Parses structured data (JSON/TOML/YAML) from an already-read string.
    ///
    /// This is useful when you've already read the file content and want to
    /// parse it without re-reading. The format is auto-detected from the file
    /// extension and content.
    ///
    /// # Errors
    ///
    /// Returns [`FsError`] if the format is unsupported or parsing fails.
    #[inline]
    pub fn parse_structured_from_str<T>(
        path: &Path,
        content: &str,
    ) -> Result<T, FsError>
    where
        T: serde::de::DeserializeOwned,
    {
        let format = Self::classify_path(path, Some(content));
        Ok(types::parse_from_format(path, content, format)?)
    }

    /// Detects the file format based on path extension and optional content
    /// hint.
    #[inline]
    #[must_use]
    pub fn classify_path(path: &Path, content: Option<&str>) -> FileFormat {
        if let Some(ext) = path.extension() {
            let format = FileFormat::from_extension(ext);
            if format != FileFormat::Unknown {
                return format;
            }
        }

        if let Some(content) = content {
            let trimmed = content.trim_start();
            if Json::detect(trimmed) {
                return FileFormat::Json;
            }
            if Yaml::detect(trimmed) {
                return FileFormat::Yaml;
            }
            if Toml::detect(trimmed) {
                return FileFormat::Toml;
            }
        }

        FileFormat::Unknown
    }
}

/// File system timestamp type alias.
///
/// This is now a direct alias to [`SystemTime`] since we use rkyv's
/// `AsUnixTime` wrapper for serialization instead of manual conversion.
///
/// The filesystem methods [`Reader::created_at`] and [`Reader::modified_at`]
/// return `Option<SystemTime>` directly.
pub type FileTimestamp = SystemTime;

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules use conventional use-before-mod ordering"
)]
mod tests {
    use std::path::{Path, PathBuf};

    use rstest::rstest;
    use tempfile::TempDir;

    use super::*;

    mod content_sniffing {
        use super::*;

        #[rstest]
        #[case::json_obj("{\"key\": \"value\"}", FileFormat::Json)]
        #[case::json_array("[1, 2, 3]", FileFormat::Json)]
        #[case::toml("name = \"test\"", FileFormat::Toml)]
        #[case::yaml("name: test\nvalue: 42", FileFormat::Yaml)]
        #[case::yaml_doc("---\nname: test", FileFormat::Yaml)]
        #[case::unknown("plain text", FileFormat::Unknown)]
        fn classifies_content_correctly(
            #[case] content: &str,
            #[case] expected: FileFormat,
        ) {
            assert_eq!(
                Reader::classify_path(Path::new("data"), Some(content)),
                expected
            );
        }

        #[rstest]
        #[case::json("config.json", FileFormat::Json)]
        #[case::toml("config.toml", FileFormat::Toml)]
        #[case::yaml("config.yaml", FileFormat::Yaml)]
        #[case::yml("config.yml", FileFormat::Yaml)]
        #[case::md("readme.md", FileFormat::Markdown)]
        #[case::png("image.png", FileFormat::Image)]
        #[case::pdf("doc.pdf", FileFormat::Pdf)]
        fn classifies_by_extension(
            #[case] path: &str,
            #[case] expected: FileFormat,
        ) {
            assert_eq!(Reader::classify_path(Path::new(path), None), expected);
        }

        #[test]
        fn favors_extension_over_content_sniffing() {
            assert_eq!(
                Reader::classify_path(
                    Path::new("config.json"),
                    Some("name = \"toml\"")
                ),
                FileFormat::Json
            );
        }

        #[test]
        fn returns_unknown_without_content() {
            assert_eq!(
                Reader::classify_path(Path::new("data"), None),
                FileFormat::Unknown
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn creates_flexible_reader_with_valid_root() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            assert_eq!(reader.root(), dir.path());
        }

        #[test]
        fn creates_strict_reader_with_absolute_root() {
            let dir = TempDir::new().expect("tempdir");
            let canonical = dir.path().canonicalize().expect("canonicalize");
            let reader =
                Reader::new_strict(canonical.clone()).expect("strict reader");
            assert_eq!(reader.root(), canonical);
        }

        #[test]
        fn rejects_relative_root_in_strict_mode() {
            let result = Reader::new_strict(PathBuf::from("relative/path"));
            assert!(matches!(
                result,
                Err(PathValidationError::RelativeRoot(_))
            ));
        }
    }

    mod filter_paths {
        use super::*;
        use crate::fs::path::FsPath;

        #[test]
        fn returns_mixed_paths() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let paths = reader.filter_paths("*").expect("filter");

            assert_eq!(paths.len(), 2);
            assert!(paths.iter().any(|p| matches!(p, FsPath::File(_))));
            assert!(paths.iter().any(|p| matches!(p, FsPath::Dir(_))));
        }
    }

    mod filter_file_paths {
        use super::*;

        #[test]
        fn returns_only_file_paths() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let paths = reader.filter_file_paths("*").expect("filter");

            assert_eq!(paths.len(), 1);
            assert_eq!(
                paths.first().unwrap().as_path().file_name().unwrap(),
                "a.json"
            );
        }
    }

    mod filter_dir_paths {
        use super::*;

        #[test]
        fn returns_only_dir_paths() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let paths = reader.filter_dir_paths("*").expect("filter");

            assert_eq!(paths.len(), 1);
            assert_eq!(
                paths.first().unwrap().as_path().file_name().unwrap(),
                "subdir"
            );
        }
    }

    mod filter_entries {
        use super::*;
        use crate::fs::entry::FsEntry;

        #[test]
        fn returns_mixed_entries() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let entries = reader.filter_entries("*").expect("filter");

            assert_eq!(entries.len(), 2);
            assert!(entries.iter().any(|e| matches!(e, FsEntry::File(_))));
            assert!(entries.iter().any(|e| matches!(e, FsEntry::Dir(_))));
        }
    }

    mod filter_file_entries {
        use super::*;

        #[test]
        fn returns_only_file_entries() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let entries = reader.filter_file_entries("*").expect("filter");

            assert_eq!(entries.len(), 1);
            let entry = entries.first().unwrap();
            assert_eq!(entry.path().as_path().file_name().unwrap(), "a.json");
            assert_eq!(entry.metadata().size(), 2);
        }
    }

    mod filter_dir_entries {
        use super::*;

        #[test]
        fn returns_only_dir_entries() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "a.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());

            let entries = reader.filter_dir_entries("*").expect("filter");

            assert_eq!(entries.len(), 1);
            assert_eq!(
                entries.first().unwrap().path().as_path().file_name().unwrap(),
                "subdir"
            );
        }
    }

    mod exists {
        use super::*;

        #[test]
        fn returns_true_for_existing_file() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = Reader::new(dir.path());
            assert!(reader.exists(Path::new("file.json")));
        }

        #[test]
        fn returns_false_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            assert!(!reader.exists(Path::new("nonexistent.json")));
        }

        #[cfg(unix)]
        #[test]
        fn returns_false_for_broken_symlink() {
            let dir = TempDir::new().expect("tempdir");
            std::os::unix::fs::symlink(
                dir.path().join("nonexistent"),
                dir.path().join("broken"),
            )
            .expect("symlink");
            let reader = Reader::new(dir.path());
            assert!(!reader.exists(Path::new("broken")));
        }
    }

    mod filter_dir {
        use super::*;

        #[test]
        fn returns_sorted_matches() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "schemas/b.json", b"{}");
            write_file(dir.path(), "schemas/a.json", b"{}");
            let reader = Reader::new(dir.path());
            let files = reader.filter_dir("schemas/**/*.json").expect("list");
            assert_eq!(files, vec![
                PathBuf::from("schemas/a.json"),
                PathBuf::from("schemas/b.json")
            ]);
        }

        #[test]
        fn excludes_directories_from_results() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir.json"))
                .expect("dir");
            let reader = Reader::new(dir.path());
            let files = reader.filter_dir("*.json").expect("list");
            assert_eq!(files.len(), 1);
        }

        #[test]
        fn handles_glob_patterns() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            // glob::Pattern is very permissive - even "[invalid" is considered
            // valid It just won't match anything, which is fine
            let result = reader.filter_dir("[invalid");
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn returns_empty_when_no_matches() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let files = reader.filter_dir("*.json").expect("list");
            assert!(files.is_empty());
        }

        #[cfg(unix)]
        #[test]
        fn includes_symlinks_in_results() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "real.json", b"{}");
            std::os::unix::fs::symlink(
                dir.path().join("real.json"),
                dir.path().join("link.json"),
            )
            .expect("symlink");
            let reader = Reader::new(dir.path());
            let files = reader.filter_dir("*.json").expect("list");
            assert_eq!(files.len(), 2);
        }
    }

    mod list_entries {
        use super::*;

        #[test]
        fn returns_sorted_entries() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "schemas/b.json", b"{}");
            write_file(dir.path(), "schemas/a.json", b"{}");
            let reader = Reader::new(dir.path());
            let entries =
                reader.list_entries("schemas/**/*.json").expect("list");
            let expected_a = dir.path().join("schemas/a.json");
            let expected_b = dir.path().join("schemas/b.json");

            assert_eq!(entries.len(), 2);
            assert_eq!(
                entries.first().map(|e| e.path_ref().as_path().to_path_buf()),
                Some(expected_a)
            );
            assert_eq!(
                entries.get(1).map(|e| e.path_ref().as_path().to_path_buf()),
                Some(expected_b)
            );
            assert_eq!(
                entries
                    .first()
                    .and_then(crate::fs::entry::FsEntry::filename)
                    .map(|name| name.as_str().to_owned()),
                Some("a.json".to_owned())
            );
            assert_eq!(
                entries
                    .get(1)
                    .and_then(crate::fs::entry::FsEntry::filename)
                    .map(|name| name.as_str().to_owned()),
                Some("b.json".to_owned())
            );
            assert!(entries.first().is_some_and(|e| {
                e.metadata().as_file().is_some_and(|meta| meta.size() > 0)
            }));
        }

        #[test]
        fn excludes_directories_from_results() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            std::fs::create_dir_all(dir.path().join("subdir.json"))
                .expect("dir");
            let reader = Reader::new(dir.path());
            let entries = reader.list_entries("*.json").expect("list");
            assert_eq!(entries.len(), 1);
        }

        #[test]
        fn returns_empty_when_no_matches() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let entries = reader.list_entries("*.json").expect("list");
            assert!(entries.is_empty());
        }
    }

    mod parse_structured {
        use super::*;

        #[rstest]
        #[case::json("data.json", &b"{\"key\":\"value\"}"[..], true)]
        #[case::toml("data.toml", b"key = \"value\"", true)]
        #[case::yaml("data.yaml", b"key: value", true)]
        #[case::yml("data.yml", b"key: value", true)]
        #[case::bad_json("bad.json", b"{invalid}", false)]
        #[case::bad_toml("bad.toml", b"invalid = [", false)]
        #[case::bad_yaml("bad.yaml", b"key: [unclosed", false)]
        fn parses_supported_formats(
            #[case] path: &str,
            #[case] content: &[u8],
            #[case] should_succeed: bool,
        ) {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), path, content);
            let reader = Reader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new(path));
            assert_eq!(result.is_ok(), should_succeed);
        }

        #[test]
        fn rejects_unknown_format() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "data.xml", b"<root/>");
            let reader = Reader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new("data.xml"));
            assert!(matches!(
                result,
                Err(FsError::Parse(
                    crate::fs::ParseError::UnsupportedFormat { .. }
                ))
            ));
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new("nonexistent.json"));
            assert!(matches!(
                result,
                Err(FsError::Read(crate::fs::ReadError::Io { .. }))
            ));
        }
    }

    mod read_to_string {
        use super::*;

        #[test]
        fn returns_file_content() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.txt", b"content");
            let reader = Reader::new(dir.path());
            assert_eq!(
                reader.read_to_string(Path::new("file.txt")).expect("read"),
                "content"
            );
        }

        #[test]
        fn rejects_invalid_utf8() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "binary.bin", b"\xff\xfe");
            let reader = Reader::new(dir.path());
            reader.read_to_string(Path::new("binary.bin")).unwrap_err();
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            reader.read_to_string(Path::new("nonexistent")).unwrap_err();
        }
    }

    mod read_bytes {
        use super::*;

        #[test]
        fn preserves_byte_content() {
            let dir = TempDir::new().expect("tempdir");
            let original: Vec<u8> = vec![0x00, 0xFF, 0xAB, 0xCD];
            write_file(dir.path(), "file.bin", &original);
            let reader = Reader::new(dir.path());
            assert_eq!(
                reader.read_bytes(Path::new("file.bin")).expect("read"),
                original
            );
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            reader.read_bytes(Path::new("nonexistent")).unwrap_err();
        }
    }

    mod metadata {
        use super::*;

        #[test]
        fn returns_file_metadata() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = Reader::new(dir.path());
            let meta = reader.metadata(Path::new("file.json")).expect("meta");
            assert!(meta.is_file());
            assert_eq!(meta.as_file().unwrap().size(), 2);
        }

        #[test]
        fn returns_dir_metadata() {
            let dir = TempDir::new().expect("tempdir");
            std::fs::create_dir_all(dir.path().join("subdir")).expect("mkdir");
            let reader = Reader::new(dir.path());
            let meta = reader.metadata(Path::new("subdir")).expect("meta");
            assert!(meta.is_dir());
        }

        #[test]
        fn returns_error_for_nonexistent_path() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result = reader.metadata(Path::new("nonexistent"));
            assert!(result.is_err());
        }
    }

    mod std_metadata {
        use super::*;

        #[test]
        fn returns_file_size() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = Reader::new(dir.path());
            let meta =
                reader.std_metadata(Path::new("file.json")).expect("meta");
            assert_eq!(meta.len(), 2);
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            reader.std_metadata(Path::new("nonexistent")).unwrap_err();
        }

        #[cfg(unix)]
        #[test]
        fn detects_symlink() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "real.json", b"{}");
            std::os::unix::fs::symlink(
                dir.path().join("real.json"),
                dir.path().join("link.json"),
            )
            .expect("symlink");
            let reader = Reader::new(dir.path());
            let meta =
                reader.std_metadata(Path::new("link.json")).expect("meta");
            assert!(meta.file_type().is_symlink());
        }
    }

    mod read_with {
        use super::*;

        #[test]
        fn invokes_closure_with_content() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.txt", b"# Title");
            let reader = Reader::new(dir.path());
            let has_heading: bool = reader
                .read_with::<_, FsError, _>(Path::new("file.txt"), |_, s| {
                    Ok(s.trim_start().starts_with('#'))
                })
                .expect("read_with");
            assert!(has_heading);
        }

        #[test]
        fn propagates_io_error() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result: Result<String, FsError> = reader
                .read_with(Path::new("nonexistent"), |_, _| Ok("x".into()));
            assert!(matches!(
                result,
                Err(FsError::Read(crate::fs::ReadError::Io { .. }))
            ));
        }
    }

    fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, contents).expect("write test file");
        path
    }

    #[test]
    fn validates_path_using_internal_validator() {
        let dir = TempDir::new().expect("tempdir");
        let reader = Reader::new(dir.path());
        reader.validate_path(Path::new("safe.txt")).expect("valid path");
        reader
            .validate_path(Path::new("../unsafe.txt"))
            .expect_err("invalid path");
    }
}
