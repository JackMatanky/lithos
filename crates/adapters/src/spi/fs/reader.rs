//! Filesystem adapter for reading configuration files.
//!
//! This module provides secure file reading with format detection.
//! Parsing logic belongs in Story 4.2 (`LoadingStrategy` pattern).

use std::path::Path;

use async_trait::async_trait;
use lithos_domain::{FileContent, FileFormat, FileLoaderError, FileReaderPort};
use tokio::task::spawn_blocking;

const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Filesystem-backed file reader.
///
/// # Story 4.1 Scope
/// - Secure file reading (path validation, size limits)
/// - UTF-8 validation (binary rejection)
/// - Format detection (extension + content analysis)
///
/// # Deferred to Story 4.2
/// - TOML/JSON/YAML parsing (`LoadingStrategy` pattern)
/// - Schema validation (Story 4.3)
#[derive(Debug, Clone)]
pub struct Reader {
    max_file_size_bytes: u64,
}

impl Default for Reader {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Reader {
    fn detect_format(path: &Path, content: &str) -> Option<FileFormat> {
        // Prefer extension-based (fast, explicit)
        Self::format_from_extension(path)
            .or_else(|| Self::format_from_content(content))
    }

    fn format_from_content(content: &str) -> Option<FileFormat> {
        let trimmed = content.trim_start();

        if trimmed.starts_with('{') {
            Some(FileFormat::Json)
        } else if trimmed.starts_with('[') {
            Some(FileFormat::Toml)
        } else if trimmed.starts_with("---") {
            Some(FileFormat::Yaml)
        } else {
            None
        }
    }

    fn format_from_extension(path: &Path) -> Option<FileFormat> {
        let ext = path.extension().and_then(|ext| ext.to_str())?;
        match ext.to_ascii_lowercase().as_str() {
            "json" => Some(FileFormat::Json),
            "toml" => Some(FileFormat::Toml),
            "yaml" | "yml" => Some(FileFormat::Yaml),
            _ => None,
        }
    }

    /// Creates a new file reader with default limits.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
        }
    }

    async fn read_file_with_limits(
        path: &Path,
        max_size: u64,
    ) -> Result<Vec<u8>, FileLoaderError> {
        let path_buf = path.to_path_buf();
        let path_display = path.display().to_string();

        spawn_blocking(move || {
            let metadata = std::fs::metadata(&path_buf).map_err(|e| {
                FileLoaderError::Io {
                    path: path_display.clone().into(),
                    message: e.to_string().into(),
                }
            })?;

            if metadata.len() > max_size {
                return Err(FileLoaderError::SizeLimitExceeded {
                    path: path_display.into(),
                    max_bytes: max_size,
                    actual_bytes: metadata.len(),
                    message: "file exceeds size limit".into(),
                });
            }

            std::fs::read(&path_buf).map_err(|e| FileLoaderError::Io {
                path: path_display.into(),
                message: e.to_string().into(),
            })
        })
        .await
        .map_err(|e| FileLoaderError::Io {
            path: path.display().to_string().into(),
            message: format!("spawn_blocking failed: {e}").into(),
        })?
    }

    fn validate_path(path: &Path) -> Result<(), FileLoaderError> {
        if path.is_absolute() {
            return Err(FileLoaderError::InvalidPath {
                path: path.display().to_string().into(),
                message: "absolute paths are not allowed".into(),
            });
        }

        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(FileLoaderError::InvalidPath {
                path: path.display().to_string().into(),
                message: "parent directory traversal is not allowed".into(),
            });
        }

        Ok(())
    }

    /// Creates a file reader with custom size limit.
    #[must_use]
    #[inline]
    pub const fn with_max_size(max_file_size_bytes: u64) -> Self {
        Self {
            max_file_size_bytes,
        }
    }
}

#[async_trait]
impl FileReaderPort for Reader {
    #[inline]
    async fn read(&self, path: &Path) -> Result<FileContent, FileLoaderError> {
        Self::validate_path(path)?;

        let bytes =
            Self::read_file_with_limits(path, self.max_file_size_bytes).await?;

        if bytes.is_empty() {
            return Err(FileLoaderError::EmptyFile {
                path: path.display().to_string().into(),
            });
        }

        let content = String::from_utf8(bytes).map_err(|_e| {
            FileLoaderError::InvalidContent {
                path: path.display().to_string().into(),
                message: "file is not valid UTF-8 (binary content rejected)"
                    .into(),
            }
        })?;

        if content.trim().is_empty() {
            return Err(FileLoaderError::EmptyFile {
                path: path.display().to_string().into(),
            });
        }

        let format = Self::detect_format(path, &content);

        Ok(FileContent::new(content, format))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn format_from_extension_works() {
        assert_eq!(
            Reader::format_from_extension(&PathBuf::from("a.toml")),
            Some(FileFormat::Toml)
        );
        assert_eq!(
            Reader::format_from_extension(&PathBuf::from("a.json")),
            Some(FileFormat::Json)
        );
        assert_eq!(
            Reader::format_from_extension(&PathBuf::from("a.yaml")),
            Some(FileFormat::Yaml)
        );
        assert_eq!(
            Reader::format_from_extension(&PathBuf::from("a.yml")),
            Some(FileFormat::Yaml)
        );
        assert_eq!(
            Reader::format_from_extension(&PathBuf::from("a.ini")),
            None
        );
    }

    #[test]
    fn format_from_content_works() {
        assert_eq!(
            Reader::format_from_content("{\"key\": 1}"),
            Some(FileFormat::Json)
        );
        assert_eq!(
            Reader::format_from_content("[section]"),
            Some(FileFormat::Toml)
        );
        assert_eq!(
            Reader::format_from_content("---\nkey: value"),
            Some(FileFormat::Yaml)
        );
        assert_eq!(Reader::format_from_content("unknown"), None);
    }

    #[test]
    fn detect_format_prefers_extension() {
        let path = PathBuf::from("config.json");
        // Content looks like TOML, but extension wins
        assert_eq!(
            Reader::detect_format(&path, "[section]"),
            Some(FileFormat::Json)
        );
    }

    #[test]
    fn validates_absolute_paths() {
        assert!(Reader::validate_path(Path::new("/etc/passwd")).is_err());
        let result = Reader::validate_path(Path::new("relative/path.toml"));
        assert!(result.is_ok(), "Expected Ok, got: {result:?}");
    }

    #[test]
    fn validates_parent_traversal() {
        assert!(Reader::validate_path(Path::new("../etc/passwd")).is_err());
        assert!(
            Reader::validate_path(Path::new("config/../secrets.toml")).is_err()
        );
        let result = Reader::validate_path(Path::new("config/app.toml"));
        assert!(result.is_ok(), "Expected Ok, got: {result:?}");
    }

    #[test]
    fn reader_default_uses_standard_limits() {
        let reader = Reader::default();
        assert_eq!(reader.max_file_size_bytes, DEFAULT_MAX_FILE_SIZE_BYTES);
    }
}
