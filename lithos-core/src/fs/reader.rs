use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use super::{
    error::PathValidationError,
    filename::Filename,
    stats::FileStats,
    types::{Binary, Json, Markdown, Toml, Yaml},
    validator::Validator,
};
use crate::fs::error::ParseError;

/// Supported file formats for structured parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormatKind {
    /// JSON format.
    Json,
    /// TOML format.
    Toml,
    /// YAML format.
    Yaml,
    /// Markdown format.
    Markdown,
    /// Binary format.
    Binary,
    /// Unknown or unsupported format.
    Unknown,
}

/// A read-only filesystem adapter for safe vault access.
///
/// `Reader` provides methods for listing, reading, and parsing files within a
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

    /// Lists files matching a glob pattern within the vault.
    ///
    /// The pattern is relative to the vault root. Only files and symlinks are
    /// returned; directories are excluded. Results are sorted alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or if I/O operations fail.
    #[inline]
    pub fn list_files(
        &self,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, ParseError> {
        let full_pattern = self.root.join(pattern);
        let pattern_str =
            full_pattern.to_str().ok_or_else(|| ParseError::Io {
                path: full_pattern.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid UTF-8 in pattern",
                ),
            })?;

        let mut paths = Vec::new();
        for entry in glob::glob(pattern_str).map_err(|e| ParseError::Io {
            path: full_pattern.clone(),
            source: std::io::Error::other(e),
        })? {
            let path = entry.map_err(|e| ParseError::Io {
                path: e.path().to_path_buf(),
                source: e.into_error(),
            })?;

            if !path.is_file() && !path.is_symlink() {
                continue;
            }

            let relative = path.strip_prefix(&self.root).map_err(|_err| {
                ParseError::Io {
                    path: path.clone(),
                    source: std::io::Error::other("Path outside root"),
                }
            })?;

            paths.push(relative.to_path_buf());
        }

        paths.sort();
        Ok(paths)
    }

    /// Reads a file's content as raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the file cannot be read.
    #[inline]
    pub(crate) fn read_bytes(
        &self,
        path: &Path,
    ) -> Result<Vec<u8>, ParseError> {
        let full_path = self.root.join(path);
        std::fs::read(&full_path).map_err(|e| ParseError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Reads a file's content as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the file cannot be read or contains
    /// invalid UTF-8.
    #[inline]
    pub fn read_to_string(&self, path: &Path) -> Result<String, ParseError> {
        let full_path = self.root.join(path);
        std::fs::read_to_string(&full_path).map_err(|e| ParseError::Io {
            path: path.to_path_buf(),
            source: e,
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
        E: From<ParseError>,
    {
        let content = self.read_to_string(path)?;
        f(path, &content)
    }

    /// Returns the statistics for a file.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the file does not exist or metadata cannot
    /// be read.
    #[inline]
    pub fn stats(&self, path: &Path) -> Result<FileStats, ParseError> {
        self.metadata(path).map(FileStats::from)
    }

    /// Returns the metadata for a file.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the file does not exist or metadata cannot
    /// be read.
    #[inline]
    pub fn metadata(
        &self,
        path: &Path,
    ) -> Result<std::fs::Metadata, ParseError> {
        let full_path = self.root.join(path);
        std::fs::symlink_metadata(&full_path).map_err(|e| ParseError::Io {
            path: path.to_path_buf(),
            source: e,
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
            .stats(path)
            .map_err(|e| {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read metadata for created_at"
                );
            })
            .ok()?;
        s.created_at
    }

    /// Returns the file's modification timestamp.
    ///
    /// Returns `None` if the metadata cannot be read or the modification time
    /// is not available on this platform. Failures are logged at debug level.
    #[inline]
    #[must_use]
    pub fn modified_at(&self, path: &Path) -> Option<SystemTime> {
        let s = self
            .stats(path)
            .map_err(|e| {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read metadata for modified_at"
                );
            })
            .ok()?;
        s.modified_at
    }

    /// Extracts the basename (filename without extension) from a path.
    ///
    /// Returns the filename without its extension as a string reference.
    /// This is useful for deriving names from file paths (e.g., schema names).
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the path has no filename or the filename
    /// is not valid UTF-8.
    #[inline]
    pub fn basename<'path>(
        &self,
        path: &'path Path,
    ) -> Result<&'path str, ParseError> {
        path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            ParseError::Io {
                path: path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path has no valid UTF-8 filename",
                ),
            }
        })
    }

    /// Extracts the filename (with extension) from a path.
    ///
    /// Returns the complete filename including its extension.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::Io`] if the path has no filename or the filename
    /// is not valid UTF-8.
    #[inline]
    pub fn filename(&self, path: &Path) -> Result<Filename, ParseError> {
        Filename::try_from(path).map_err(|source| ParseError::Io {
            path: path.to_path_buf(),
            source,
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
    /// Returns [`ParseError`] if:
    /// - The file format is unsupported.
    /// - The file cannot be read.
    /// - The content is malformed for the detected format.
    #[inline]
    pub fn parse_structured<T>(&self, path: &Path) -> Result<T, ParseError>
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
    /// Returns [`ParseError`] if the format is unsupported or parsing fails.
    #[inline]
    pub fn parse_structured_from_str<T>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError>
    where
        T: serde::de::DeserializeOwned,
    {
        match Self::classify_path(path, Some(content)) {
            FormatKind::Json => Json::parse(path, content),
            FormatKind::Toml => Toml::parse(path, content),
            FormatKind::Yaml => Yaml::parse(path, content),
            FormatKind::Markdown | FormatKind::Binary | FormatKind::Unknown => {
                Err(ParseError::UnsupportedFormat {
                    path: path.to_path_buf(),
                    supported: &["json", "toml", "yaml", "yml"],
                })
            }
        }
    }

    /// Detects the file format based on path extension and optional content
    /// hint.
    #[must_use]
    fn classify_path(path: &Path, content: Option<&str>) -> FormatKind {
        if Json::is_supported(path) {
            return FormatKind::Json;
        }
        if Toml::is_supported(path) {
            return FormatKind::Toml;
        }
        if Yaml::is_supported(path) {
            return FormatKind::Yaml;
        }
        if Markdown::is_supported(path) {
            return FormatKind::Markdown;
        }
        if Binary::is_supported(path) {
            return FormatKind::Binary;
        }

        if let Some(content) = content {
            let trimmed = content.trim_start();
            if Json::detect(trimmed) {
                return FormatKind::Json;
            }
            if Yaml::detect(trimmed) {
                return FormatKind::Yaml;
            }
            if Toml::detect(trimmed) {
                return FormatKind::Toml;
            }
        }

        FormatKind::Unknown
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
        #[case::json_obj("{\"key\": \"value\"}", FormatKind::Json)]
        #[case::json_array("[1, 2, 3]", FormatKind::Json)]
        #[case::toml("name = \"test\"", FormatKind::Toml)]
        #[case::yaml("name: test\nvalue: 42", FormatKind::Yaml)]
        #[case::yaml_doc("---\nname: test", FormatKind::Yaml)]
        #[case::unknown("plain text", FormatKind::Unknown)]
        fn classifies_content_correctly(
            #[case] content: &str,
            #[case] expected: FormatKind,
        ) {
            assert_eq!(
                Reader::classify_path(Path::new("data"), Some(content)),
                expected
            );
        }

        #[rstest]
        #[case::json("config.json", FormatKind::Json)]
        #[case::toml("config.toml", FormatKind::Toml)]
        #[case::yaml("config.yaml", FormatKind::Yaml)]
        #[case::yml("config.yml", FormatKind::Yaml)]
        #[case::md("readme.md", FormatKind::Markdown)]
        #[case::png("image.png", FormatKind::Binary)]
        #[case::pdf("doc.pdf", FormatKind::Binary)]
        fn classifies_by_extension(
            #[case] path: &str,
            #[case] expected: FormatKind,
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
                FormatKind::Json
            );
        }

        #[test]
        fn returns_unknown_without_content() {
            assert_eq!(
                Reader::classify_path(Path::new("data"), None),
                FormatKind::Unknown
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

    mod list_files {
        use super::*;

        #[test]
        fn returns_sorted_matches() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "schemas/b.json", b"{}");
            write_file(dir.path(), "schemas/a.json", b"{}");
            let reader = Reader::new(dir.path());
            let files = reader.list_files("schemas/**/*.json").expect("list");
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
            let files = reader.list_files("*.json").expect("list");
            assert_eq!(files.len(), 1);
        }

        #[test]
        fn rejects_invalid_pattern() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            reader.list_files("[invalid").unwrap_err();
        }

        #[test]
        fn returns_empty_when_no_matches() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let files = reader.list_files("*.json").expect("list");
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
            let files = reader.list_files("*.json").expect("list");
            assert_eq!(files.len(), 2);
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
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new("nonexistent.json"));
            assert!(matches!(result, Err(ParseError::Io { .. })));
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
        fn returns_file_size() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = Reader::new(dir.path());
            let meta = reader.metadata(Path::new("file.json")).expect("meta");
            assert_eq!(meta.len(), 2);
        }

        #[test]
        fn returns_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            reader.metadata(Path::new("nonexistent")).unwrap_err();
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
            let meta = reader.metadata(Path::new("link.json")).expect("meta");
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
                .read_with::<_, ParseError, _>(Path::new("file.txt"), |_, s| {
                    Ok(s.trim_start().starts_with('#'))
                })
                .expect("read_with");
            assert!(has_heading);
        }

        #[test]
        fn propagates_io_error() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result: Result<String, ParseError> = reader
                .read_with(Path::new("nonexistent"), |_, _| Ok("x".into()));
            assert!(matches!(result, Err(ParseError::Io { .. })));
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
