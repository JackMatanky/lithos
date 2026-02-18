//! File system abstraction for testable file I/O.
//!
//! This module provides the [`FsReader`] trait and its production
//! implementation for scoped filesystem access.

use std::{
    io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use super::{
    Json, Toml, Yaml,
    error::{ParseError, PathValidationError},
    validator::Validator,
};

/// Internal file classification for read pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormatKind {
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

/// Abstraction over file system operations for dependency injection.
///
/// Implementations must be `Send + Sync` to support concurrent access in
/// ingestion services.
pub trait FsReader: Send + Sync {
    /// Error type for file operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Classifies the format based on extension and heuristics.
    #[inline]
    #[must_use]
    fn classify(&self, path: &Path) -> FormatKind {
        classify_path(path)
    }

    /// Checks if a file exists at the given path.
    ///
    /// Returns `true` if the file exists, `false` otherwise.
    /// Does not distinguish between "file not found" and other errors.
    #[must_use]
    fn exists(&self, path: &Path) -> bool;

    /// Lists all files matching a glob pattern.
    ///
    /// The pattern syntax follows standard glob conventions:
    /// - `*.json` - all JSON files in the root directory
    /// - `**/*.json` - all JSON files recursively
    /// - `schemas/*.{json,toml,yaml}` - schema files with multiple extensions
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The glob pattern is invalid
    /// - Directory traversal fails (permissions, I/O error)
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error>;

    /// Returns metadata for a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata cannot be read.
    fn metadata(&self, path: &Path) -> Result<FileMetadata, Self::Error>;

    /// Parse a structured file into type `T` using extension-based routing.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for I/O, parse, or unsupported format errors.
    #[inline]
    fn parse_structured<T: DeserializeOwned>(
        &self,
        path: &Path,
    ) -> Result<T, ParseError>
    where
        Self::Error: Into<io::Error>,
    {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error.into(),
            })?;

        match self.classify(path) {
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

    /// Reads the entire contents of a file as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;

    /// Reads the entire contents of a file as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be read (permissions, I/O error)
    /// - The file contents are not valid UTF-8
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;

    /// Read a file and parse it with a custom closure.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] for I/O or closure failures.
    #[inline]
    fn read_with<T, F>(&self, path: &Path, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&Path, &str) -> Result<T, ParseError>,
        Self::Error: Into<io::Error>,
    {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error.into(),
            })?;

        f(path, &content)
    }

    /// Validates the path against the configured policy.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError`] if the path is invalid.
    #[inline]
    fn validate_path(&self, path: &Path) -> Result<(), PathValidationError> {
        Validator::new_flexible().validate(path)
    }
}

/// Production file reader using `std::fs` for real filesystem access.
#[derive(Debug, Clone)]
pub struct OsFsReader {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl OsFsReader {
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
}

impl FsReader for OsFsReader {
    type Error = io::Error;

    #[inline]
    fn classify(&self, path: &Path) -> FormatKind {
        classify_path(path)
    }

    #[inline]
    fn exists(&self, path: &Path) -> bool {
        self.resolve_path(path).exists()
    }

    #[inline]
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
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

    #[inline]
    fn metadata(&self, path: &Path) -> Result<FileMetadata, Self::Error> {
        let metadata = std::fs::symlink_metadata(self.resolve_path(path))?;
        Ok(FileMetadata {
            modified: metadata.modified().ok(),
            size: metadata.len(),
            is_symlink: metadata.file_type().is_symlink(),
        })
    }

    #[inline]
    fn parse_structured<T: DeserializeOwned>(
        &self,
        path: &Path,
    ) -> Result<T, ParseError>
    where
        Self::Error: Into<io::Error>,
    {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error,
            })?;

        match self.classify(path) {
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

    #[inline]
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        std::fs::read(self.resolve_path(path))
    }

    #[inline]
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        std::fs::read_to_string(self.resolve_path(path))
    }

    #[inline]
    fn read_with<T, F>(&self, path: &Path, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&Path, &str) -> Result<T, ParseError>,
        Self::Error: Into<io::Error>,
    {
        let content =
            self.read_to_string(path).map_err(|error| ParseError::Io {
                path: path.to_path_buf(),
                source: error,
            })?;

        f(path, &content)
    }

    #[inline]
    fn validate_path(&self, path: &Path) -> Result<(), PathValidationError> {
        Validator::new_flexible().validate(path)
    }
}

#[inline]
#[must_use]
fn classify_path(path: &Path) -> FormatKind {
    if Json::is_supported(path) {
        return FormatKind::Json;
    }
    if Toml::is_supported(path) {
        return FormatKind::Toml;
    }
    if Yaml::is_supported(path) {
        return FormatKind::Yaml;
    }
    if is_markdown(path) {
        return FormatKind::Markdown;
    }
    if is_likely_binary(path) {
        return FormatKind::Binary;
    }
    FormatKind::Unknown
}

#[inline]
#[must_use]
fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
    })
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
