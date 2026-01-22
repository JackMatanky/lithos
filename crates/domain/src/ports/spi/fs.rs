//! File loader port definition.

use std::path::Path;

use async_trait::async_trait;

use crate::errors::FileLoaderError;

/// Supported configuration file formats.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// JSON format (.json).
    Json,
    /// TOML format (.toml).
    Toml,
    /// YAML format (.yaml, .yml).
    Yaml,
}

/// File content with optional format hint.
///
/// Note: This struct is intentionally exhaustive (not marked
/// `#[non_exhaustive]`) because it represents a stable, simple data container
/// at the SPI boundary. The two fields (content + format) are unlikely to
/// change, and struct literal construction provides ergonomic usage. A
/// constructor is provided for those who prefer explicit construction.
#[expect(
    clippy::exhaustive_structs,
    reason = "Stable SPI boundary struct with two simple fields unlikely to \
              change; struct literal construction is ergonomic and preferred"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContent {
    /// The UTF-8 validated file content.
    pub content: String,
    /// Optional format detected from extension or content analysis.
    pub format: Option<FileFormat>,
}

impl FileContent {
    /// Creates new file content with optional format.
    #[must_use]
    #[inline]
    pub const fn new(content: String, format: Option<FileFormat>) -> Self {
        Self {
            content,
            format,
        }
    }
}

/// Port for reading configuration file text.
///
/// # Invariants
/// - All methods must be async.
/// - Implementations must enforce security constraints (path validation, size
///   limits, binary rejection).
/// - Returned content is valid UTF-8 text.
///
/// # Examples
/// ```ignore
/// struct FsReader;
///
/// #[async_trait]
/// impl FileReader for FsReader {
///     async fn read(&self, path: &Path) -> Result<FileContent, FileLoaderError> {
///         // Adapter implementation
///         Ok(FileContent::new(
///             std::fs::read_to_string(path)?,
///             detect_format(path),
///         ))
///     }
/// }
/// ```
#[async_trait]
pub trait FileReader: Send + Sync {
    /// Read file content as UTF-8 text with optional format detection.
    ///
    /// # Errors
    /// Returns `FileLoaderError` when file loading or validation fails.
    async fn read(&self, path: &Path) -> Result<FileContent, FileLoaderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_reader_port_is_object_safe() {
        fn _assert_object_safe(_: &dyn FileReader) {}
    }

    #[test]
    fn file_reader_port_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Box<dyn FileReader>>();
    }
}
