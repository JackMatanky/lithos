//! File loader port definition.

use std::path::Path;

use async_trait::async_trait;

use crate::errors::FileLoaderError;

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
///     async fn read(&self, path: &Path) -> Result<String, FileLoaderError> {
///         // Adapter implementation
///         Ok(std::fs::read_to_string(path).map_err(|_| FileLoaderError::Io {
///             path: path.display().to_string().into(),
///             message: "read failed".into(),
///         })?)
///     }
/// }
/// ```
#[async_trait]
pub trait FileReader: Send + Sync {
    /// Read file content as UTF-8 text.
    ///
    /// # Errors
    /// Returns `FileLoaderError` when file loading or validation fails.
    async fn read(&self, path: &Path) -> Result<String, FileLoaderError>;
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
