//! File system abstraction for testable file I/O.
//!
//! This module provides the [`Reader`] concrete type for scoped filesystem
//! access. The reader keeps path resolution anchored to a root directory so
//! adapters can perform deterministic file access without leaking filesystem
//! details into domain logic.

use std::{
    io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use super::{
    error::{ParseError, PathValidationError},
    types::{Json, Markdown, Toml, Yaml},
    validator::Validator,
};

/// Internal file classification for read pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatKind {
    /// JSON structured data.
    Json,
    /// TOML structured data.
    Toml,
    /// YAML structured data.
    Yaml,
    /// Markdown text.
    Markdown,
    /// Likely binary data.
    Binary,
    /// Unknown or unsupported format.
    Unknown,
}

/// Lightweight file metadata used by ingestion pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileMetadata {
    /// Last modification time, if available.
    pub modified: Option<std::time::SystemTime>,
    /// File size in bytes.
    pub size: u64,
    /// True when the path is a symlink.
    pub is_symlink: bool,
}

/// Production file reader using `std::fs` for real filesystem access.
///
/// The reader enforces root-scoped access and is intended for adapter layers
/// that ingest files into domain models.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use lithos_core::fs::reader::Reader;
/// # let unique = format!("lithos_fs_reader_example_{}", std::process::id());
/// # let root = std::env::temp_dir().join(unique);
/// # std::fs::create_dir_all(&root)?;
/// # std::fs::write(root.join("config.json"), "{}")?;
/// let reader = Reader::new(root.as_path());
/// let content = reader.read_to_string(Path::new("config.json"))?;
/// assert_eq!(content, "{}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct Reader {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl Reader {
    /// Creates a new filesystem reader scoped to the given root directory.
    #[inline]
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Returns the root directory for this reader.
    #[inline]
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a relative path against the root directory.
    #[inline]
    fn resolve_path(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    /// Classify a path based on extension with optional content-sniffing
    /// fallback.
    ///
    /// When content is provided and the extension is unknown or absent, the
    /// method uses format detection heuristics to identify JSON, YAML, or TOML.
    #[inline]
    #[must_use]
    pub(crate) fn classify(path: &Path, content: Option<&str>) -> FormatKind {
        classify_path(path, content)
    }

    /// Check whether a file exists at the given path.
    #[inline]
    #[must_use]
    pub fn exists(&self, path: &Path) -> bool {
        self.resolve_path(path).exists()
    }

    /// List files matching a glob pattern, relative to the root.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or traversal fails.
    #[inline]
    pub fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, io::Error> {
        let full_pattern = self.root.join(pattern);
        let pattern_str = full_pattern.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Pattern contains invalid UTF-8",
            )
        })?;

        let mut paths: Vec<PathBuf> = glob::glob(pattern_str)
            .map_err(|error| {
                io::Error::new(io::ErrorKind::InvalidInput, error)
            })?
            .filter_map(|entry| {
                let path = entry.ok()?;
                if !path.is_file() && !path.is_symlink() {
                    return None;
                }
                path.strip_prefix(&self.root).ok().map(Path::to_path_buf)
            })
            .collect();

        paths.sort();
        Ok(paths)
    }

    /// Read metadata for a path without following symlinks.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata cannot be read.
    #[inline]
    pub fn metadata(&self, path: &Path) -> Result<FileMetadata, io::Error> {
        let metadata = std::fs::symlink_metadata(self.resolve_path(path))?;
        Ok(FileMetadata {
            modified: metadata.modified().ok(),
            size: metadata.len(),
            is_symlink: metadata.file_type().is_symlink(),
        })
    }

    /// Parse a structured file into type `T` based on its extension.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for I/O, parse, or unsupported format errors.
    #[inline]
    pub fn parse_structured<T: DeserializeOwned>(
        &self,
        path: &Path,
    ) -> Result<T, ParseError> {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error,
            })?;

        match Self::classify(path, Some(&content)) {
            FormatKind::Json => Json::parse(path, &content),
            FormatKind::Toml => Toml::parse(path, &content),
            FormatKind::Yaml => Yaml::parse(path, &content),
            FormatKind::Markdown | FormatKind::Binary | FormatKind::Unknown => {
                Err(ParseError::UnsupportedFormat {
                    path: path.to_path_buf(),
                    supported: &["json", "toml", "yaml", "yml"],
                })
            }
        }
    }

    /// Read the entire file contents as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    #[inline]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "pub(crate) API for future callers; tested but not yet \
                      used outside the fs module."
        )
    )]
    pub(crate) fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
        std::fs::read(self.resolve_path(path))
    }

    /// Read the entire file contents as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or is not valid UTF-8.
    #[inline]
    pub fn read_to_string(&self, path: &Path) -> Result<String, io::Error> {
        std::fs::read_to_string(self.resolve_path(path))
    }

    /// Read a file and parse it with a caller-provided closure.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for I/O or closure failures.
    #[inline]
    pub fn read_with<T, F>(&self, path: &Path, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&Path, &str) -> Result<T, ParseError>,
    {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error,
            })?;

        f(path, &content)
    }

    /// Validate a path against the configured policy.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError`] if the path is invalid.
    #[inline]
    pub fn validate_path(
        &self,
        path: &Path,
    ) -> Result<(), PathValidationError> {
        Validator::new_flexible().validate(path)
    }
}

#[inline]
#[must_use]
fn classify_path(path: &Path, content: Option<&str>) -> FormatKind {
    // 1. Extension-first (fast, zero allocation)
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
    if is_likely_binary(path) {
        return FormatKind::Binary;
    }

    // 2. Content-sniffing fallback (extension-less or unknown files)
    // Detection order: JSON → YAML → TOML
    // JSON is unambiguous ({/[, YAML before TOML because --- is unambiguous,
    // TOML's heuristic (= without :) is most likely to produce false positives.
    if let Some(content) = content {
        if Json::detect(content) {
            return FormatKind::Json;
        }
        if Yaml::detect(content) {
            return FormatKind::Yaml;
        }
        if Toml::detect(content) {
            return FormatKind::Toml;
        }
    }

    FormatKind::Unknown
}

#[inline]
#[must_use]
fn is_likely_binary(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "pdf"
                | "mp3"
                | "mp4"
                | "zip"
                | "wasm"
        )
    })
}

#[cfg(test)]
mod tests {
    mod content_sniffing {
        use super::*;

        #[test]
        fn detects_json_from_content() {
            assert_eq!(
                classify_path(Path::new("data"), Some("{\"key\": \"value\"}")),
                FormatKind::Json
            );
            assert_eq!(
                classify_path(Path::new("data"), Some("[1, 2, 3]")),
                FormatKind::Json
            );
        }

        #[test]
        fn detects_toml_from_content() {
            // Note: content starting with '[' is ambiguous (could be JSON array
            // or TOML table header). Use key-value format for unambiguous TOML.
            assert_eq!(
                classify_path(
                    Path::new("data"),
                    Some("name = \"test\"\nversion = \"1.0\"")
                ),
                FormatKind::Toml
            );
            assert_eq!(
                classify_path(Path::new("data"), Some("name = \"test\"")),
                FormatKind::Toml
            );
        }

        #[test]
        fn returns_unknown_for_unrecognized_content() {
            assert_eq!(
                classify_path(
                    Path::new("data"),
                    Some("plain text without structure")
                ),
                FormatKind::Unknown
            );
        }

        #[test]
        fn returns_unknown_without_content() {
            assert_eq!(
                classify_path(Path::new("data"), None),
                FormatKind::Unknown
            );
        }

        #[test]
        fn extension_takes_precedence_over_content() {
            // A .json file with TOML content should still be classified as JSON
            assert_eq!(
                classify_path(
                    Path::new("config.json"),
                    Some("name = \"toml content\"")
                ),
                FormatKind::Json
            );
        }
    }

    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::*;

    fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, contents).expect("write test file");
        path
    }

    #[test]
    fn list_files_returns_sorted_matches() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/b.json", b"{}");
        write_file(dir.path(), "schemas/a.json", b"{}");
        write_file(dir.path(), "schemas/c.toml", b"key = 1");

        let reader = Reader::new(dir.path());
        let files = reader.list_files("schemas/**/*.json").expect("list files");

        assert_eq!(files, vec![
            PathBuf::from("schemas/a.json"),
            PathBuf::from("schemas/b.json"),
        ]);
    }

    #[test]
    fn list_files_supports_root_patterns() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "note.md", b"# Title");
        write_file(dir.path(), "note.txt", b"plain");

        let reader = Reader::new(dir.path());
        let files = reader.list_files("*.md").expect("list files");

        assert_eq!(files, vec![PathBuf::from("note.md")]);
    }

    #[test]
    fn list_files_rejects_invalid_pattern() {
        let dir = TempDir::new().expect("tempdir");
        let reader = Reader::new(dir.path());
        let result = reader.list_files("[invalid");

        result.unwrap_err();
    }

    #[test]
    fn read_to_string_reads_content() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/note.json", b"{}");

        let reader = Reader::new(dir.path());
        let content = reader
            .read_to_string(Path::new("schemas/note.json"))
            .expect("read to string");

        assert_eq!(content, "{}");
    }

    #[test]
    fn read_bytes_reads_binary() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "bin/blob.bin", b"\x00\x01\x02");

        let reader = Reader::new(dir.path());
        let bytes =
            reader.read_bytes(Path::new("bin/blob.bin")).expect("read bytes");

        assert_eq!(bytes, b"\x00\x01\x02");
    }

    #[test]
    fn metadata_returns_size() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/note.json", b"{}");

        let reader = Reader::new(dir.path());
        let metadata =
            reader.metadata(Path::new("schemas/note.json")).expect("metadata");

        assert_eq!(metadata.size, 2);
        assert!(!metadata.is_symlink);
    }

    #[cfg(unix)]
    #[test]
    fn metadata_marks_symlink() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/real.json", b"{}");

        let link = dir.path().join("schemas/link.json");
        std::os::unix::fs::symlink(dir.path().join("schemas/real.json"), &link)
            .expect("symlink");

        let reader = Reader::new(dir.path());
        let metadata =
            reader.metadata(Path::new("schemas/link.json")).expect("metadata");

        assert!(metadata.is_symlink);
    }

    #[cfg(unix)]
    #[test]
    fn list_files_includes_symlinks() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/real.json", b"{}");

        let link = dir.path().join("schemas/link.json");
        std::os::unix::fs::symlink(dir.path().join("schemas/real.json"), &link)
            .expect("symlink");

        let reader = Reader::new(dir.path());
        let files = reader.list_files("schemas/**/*.json").expect("list files");

        assert!(files.contains(&PathBuf::from("schemas/real.json")));
        assert!(files.contains(&PathBuf::from("schemas/link.json")));
    }

    #[test]
    fn parse_structured_reads_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/note.json", b"{\"name\":\"note\"}");

        let reader = Reader::new(dir.path());
        let value: serde_json::Value = reader
            .parse_structured(Path::new("schemas/note.json"))
            .expect("parse structured");

        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("note"));
    }

    #[test]
    fn parse_structured_rejects_unknown_format() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/note.xml", b"<note></note>");

        let reader = Reader::new(dir.path());
        let result: Result<serde_json::Value, _> =
            reader.parse_structured(Path::new("schemas/note.xml"));

        assert!(matches!(result, Err(ParseError::UnsupportedFormat { .. })));
    }

    #[test]
    fn read_with_invokes_closure() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "notes/readme.md", b"# Title");

        let reader = Reader::new(dir.path());
        let result = reader
            .read_with(Path::new("notes/readme.md"), |_, text| {
                Ok(text.trim_start().starts_with('#'))
            })
            .expect("read with");

        assert!(result);
    }
}
