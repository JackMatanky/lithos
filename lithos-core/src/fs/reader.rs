//! Root-scoped filesystem reader with validation and format-classification.
//!
//! This module provides [`FileReader`], a read-only filesystem adapter for safe
//! vault access with built-in path validation and structured file parsing.
//!
//! # Security
//!
//! All file access is scoped to a root directory with validation via
//! [`Validator`] to prevent path traversal attacks and unauthorized access to
//! restricted files.
//!
//! # Features
//!
//! - **File reading**: Raw bytes, UTF-8 strings, and structured parsing
//!   (JSON/TOML/YAML)
//! - **Format detection**: Automatic format classification by extension and
//!   content
//! - **Path checks**: Root-relative existence checks via [`FileReader::exists`]
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//!
//! use lithos_core::fs::FileReader;
//!
//! let reader = FileReader::new("/vault");
//!
//! // Check whether a file exists
//! let _exists = reader.exists(Path::new("config.json"));
//!
//! // Read and parse structured file
//! let data: serde_json::Value =
//!     reader.parse_structured(Path::new("config.json"))?;
//!
//! # Ok::<(), lithos_core::fs::FsError>(())
//! ```

use std::path::{Path, PathBuf};

use super::{
    error::{FsError, PathValidationError},
    format::{FileFormat, parse_from_format, sniff_structured_format},
    name::FileName,
    validator::Validator,
};

/// A read-only filesystem adapter for safe vault access.
///
/// `FileReader` provides methods for reading and parsing files within a
/// specified root directory. It enforces path safety via [`Validator`] to
/// prevent traversal attacks and unauthorized access to restricted files.
pub struct FileReader {
    root: PathBuf,
    validator: Validator,
}

impl FileReader {
    /// Creates a new `FileReader` with flexible validation.
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

    /// Creates a new `FileReader` with strict validation.
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
    /// [`FileReader::new(vault_root)`](FileReader::new) instead to ensure
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
    /// use lithos_core::fs::reader::FileReader;
    ///
    /// let fs = FileReader::from_system_root();
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

    /// Extracts the filename (with extension) from a path.
    ///
    /// Returns the complete filename including its extension.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Path`] if the path has no filename or the filename
    /// is not valid UTF-8.
    #[inline]
    pub fn filename(&self, path: &Path) -> Result<FileName, FsError> {
        Ok(FileName::try_from(path)?)
    }

    /// Validates a path using the internal validator.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError`] if the path is invalid.
    #[inline]
    #[allow(
        dead_code,
        reason = "Currently unused after removing process_partial"
    )]
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
        Ok(parse_from_format(path, content, format)?)
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
            if let Some(format) = sniff_structured_format(trimmed) {
                return format;
            }
        }

        FileFormat::Unknown
    }
}

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
                FileReader::classify_path(Path::new("data"), Some(content)),
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
            assert_eq!(
                FileReader::classify_path(Path::new(path), None),
                expected
            );
        }

        #[test]
        fn favors_extension_over_content_sniffing() {
            assert_eq!(
                FileReader::classify_path(
                    Path::new("config.json"),
                    Some("name = \"toml\"")
                ),
                FileFormat::Json
            );
        }

        #[test]
        fn returns_unknown_without_content() {
            assert_eq!(
                FileReader::classify_path(Path::new("data"), None),
                FileFormat::Unknown
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn creates_flexible_reader_with_valid_root() {
            let dir = TempDir::new().expect("tempdir");
            let reader = FileReader::new(dir.path());
            assert_eq!(reader.root(), dir.path());
        }

        #[test]
        fn creates_strict_reader_with_absolute_root() {
            let dir = TempDir::new().expect("tempdir");
            let canonical = dir.path().canonicalize().expect("canonicalize");
            let reader = FileReader::new_strict(canonical.clone())
                .expect("strict reader");
            assert_eq!(reader.root(), canonical);
        }

        #[test]
        fn rejects_relative_root_in_strict_mode() {
            let result = FileReader::new_strict(PathBuf::from("relative/path"));
            assert!(matches!(
                result,
                Err(PathValidationError::RelativeRoot(_))
            ));
        }
    }

    mod exists {
        use super::*;

        #[test]
        fn returns_true_for_existing_file() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = FileReader::new(dir.path());
            assert!(reader.exists(Path::new("file.json")));
        }

        #[test]
        fn returns_false_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = FileReader::new(dir.path());
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
            let reader = FileReader::new(dir.path());
            assert!(!reader.exists(Path::new("broken")));
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
            let reader = FileReader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new(path));
            assert_eq!(result.is_ok(), should_succeed);
        }

        #[test]
        fn rejects_unknown_format() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "data.xml", b"<root/>");
            let reader = FileReader::new(dir.path());
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
            let reader = FileReader::new(dir.path());
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
            let reader = FileReader::new(dir.path());
            assert_eq!(
                reader.read_to_string(Path::new("file.txt")).expect("read"),
                "content"
            );
        }

        #[test]
        fn rejects_invalid_utf8() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "binary.bin", b"\xff\xfe");
            let reader = FileReader::new(dir.path());
            reader.read_to_string(Path::new("binary.bin")).unwrap_err();
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = FileReader::new(dir.path());
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
            let reader = FileReader::new(dir.path());
            assert_eq!(
                reader.read_bytes(Path::new("file.bin")).expect("read"),
                original
            );
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = FileReader::new(dir.path());
            reader.read_bytes(Path::new("nonexistent")).unwrap_err();
        }
    }

    mod read_with {
        use super::*;

        #[test]
        fn invokes_closure_with_content() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.txt", b"# Title");
            let reader = FileReader::new(dir.path());
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
            let reader = FileReader::new(dir.path());
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
        let reader = FileReader::new(dir.path());
        reader.validate_path(Path::new("safe.txt")).expect("valid path");
        reader
            .validate_path(Path::new("../unsafe.txt"))
            .expect_err("invalid path");
    }
}
