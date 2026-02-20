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
    PathValidationError,
    error::ParseError,
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
    /// Path validator for security checks.
    ///
    /// Stored for future use when strict mode path validation is needed
    /// (e.g., validating paths before file operations in vault mode).
    #[expect(
        dead_code,
        reason = "Stored for strict mode path validation; will be used when \
                  Reader exposes path validation to callers."
    )]
    validator: Validator,
}

impl Reader {
    /// Creates a new filesystem reader with flexible path validation.
    ///
    /// Flexible mode allows external symlinks (e.g., dotfiles) while still
    /// checking input traversal and hidden-file access.
    #[inline]
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            validator: Validator::new_flexible(),
        }
    }

    /// Creates a new filesystem reader with strict path validation.
    ///
    /// Strict mode enforces a root boundary and rejects symlinks that escape
    /// it.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError::RelativeRoot`] if `root` is not an
    /// absolute path.
    #[inline]
    pub fn new_strict(root: PathBuf) -> Result<Self, PathValidationError> {
        let validator = Validator::try_new_strict(root.clone())?;
        Ok(Self {
            root,
            validator,
        })
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
    /// Returns an error if the pattern is invalid or if any per-entry operation
    /// fails (e.g., permission denied, broken symlink).
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
            .map(|entry| -> io::Result<Option<PathBuf>> {
                let path = entry.map_err(io::Error::other)?;
                if !path.is_file() && !path.is_symlink() {
                    return Ok(None);
                }
                Ok(path.strip_prefix(&self.root).ok().map(Path::to_path_buf))
            })
            .filter_map(io::Result::transpose)
            .collect::<io::Result<_>>()?;

        paths.sort();
        Ok(paths)
    }

    /// Read metadata for a path without following symlinks.
    ///
    /// Uses `symlink_metadata` to avoid following symlinks, ensuring the caller
    /// sees the symlink itself rather than its target.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata cannot be read.
    #[inline]
    pub fn metadata(
        &self,
        path: &Path,
    ) -> Result<std::fs::Metadata, io::Error> {
        std::fs::symlink_metadata(self.resolve_path(path))
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
    /// The closure can return any error type that implements
    /// `From<ParseError>`, allowing callers to produce domain-specific
    /// errors directly.
    ///
    /// # Errors
    ///
    /// Returns the closure's error type for I/O or closure failures.
    #[inline]
    pub fn read_with<T, E, F>(&self, path: &Path, f: F) -> Result<T, E>
    where
        F: FnOnce(&Path, &str) -> Result<T, E>,
        E: From<ParseError>,
    {
        let content = self.read_to_string(path).map_err(|error| {
            E::from(ParseError::Io {
                path: path.to_path_buf(),
                source: error,
            })
        })?;

        f(path, &content)
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
        ext.eq_ignore_ascii_case("png")
            || ext.eq_ignore_ascii_case("jpg")
            || ext.eq_ignore_ascii_case("jpeg")
            || ext.eq_ignore_ascii_case("gif")
            || ext.eq_ignore_ascii_case("pdf")
            || ext.eq_ignore_ascii_case("mp3")
            || ext.eq_ignore_ascii_case("mp4")
            || ext.eq_ignore_ascii_case("zip")
            || ext.eq_ignore_ascii_case("wasm")
    })
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
                classify_path(Path::new("data"), Some(content)),
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
            assert_eq!(classify_path(Path::new(path), None), expected);
        }

        #[test]
        fn extension_takes_precedence_over_content() {
            assert_eq!(
                classify_path(
                    Path::new("config.json"),
                    Some("name = \"toml\"")
                ),
                FormatKind::Json
            );
        }

        #[test]
        fn returns_unknown_without_content() {
            assert_eq!(
                classify_path(Path::new("data"), None),
                FormatKind::Unknown
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn creates_flexible_reader() {
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
        fn excludes_directories() {
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
        fn includes_symlinks() {
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
        fn parses_formats(
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
        fn returns_io_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let result: Result<serde_json::Value, _> =
                reader.parse_structured(Path::new("nonexistent.json"));
            assert!(matches!(result, Err(ParseError::Io { .. })));
        }
    }

    mod read_operations {
        use super::*;

        #[test]
        fn read_to_string_returns_content() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.txt", b"content");
            let reader = Reader::new(dir.path());
            assert_eq!(
                reader.read_to_string(Path::new("file.txt")).expect("read"),
                "content"
            );
        }

        #[test]
        fn read_to_string_rejects_invalid_utf8() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "binary.bin", b"\xff\xfe");
            let reader = Reader::new(dir.path());
            reader.read_to_string(Path::new("binary.bin")).unwrap_err();
        }

        #[test]
        fn read_bytes_preserves_content() {
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
        fn metadata_returns_file_size() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.json", b"{}");
            let reader = Reader::new(dir.path());
            let meta = reader.metadata(Path::new("file.json")).expect("meta");
            assert_eq!(meta.len(), 2);
        }

        #[test]
        fn operations_return_error_for_nonexistent_file() {
            let dir = TempDir::new().expect("tempdir");
            let reader = Reader::new(dir.path());
            let path = Path::new("nonexistent");
            reader.read_to_string(path).unwrap_err();
            reader.read_bytes(path).unwrap_err();
            reader.metadata(path).unwrap_err();
        }

        #[cfg(unix)]
        #[test]
        fn metadata_detects_symlink() {
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
}
