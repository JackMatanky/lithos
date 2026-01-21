//! Filesystem adapter for loading configuration files.

use std::path::Path;

use async_trait::async_trait;
use lithos_domain::{FileLoaderError, FileReaderPort};
use tokio::task::spawn_blocking;

const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Filesystem-backed implementation of `FileReaderPort`.
///
/// # Invariants
/// - All I/O uses `spawn_blocking` to protect async runtime.
/// - Security checks run before format parsing.
/// - Returned content is valid UTF-8.
#[derive(Debug, Clone)]
pub struct FileReader {
    max_file_size_bytes: u64,
}

impl Default for FileReader {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl FileReader {
    fn decode_utf8(
        bytes: Vec<u8>,
        path: &Path,
    ) -> Result<String, FileLoaderError> {
        String::from_utf8(bytes)
            .map_err(|err| Self::invalid_content_error(path, err.to_string()))
    }

    fn detect_format(path: &Path, content: &str) -> Option<FileFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(FileFormat::from_extension)
            .or_else(|| FileFormat::from_content(content))
    }

    fn detect_format_or_error(
        path: &Path,
        content: &str,
    ) -> Result<FileFormat, FileLoaderError> {
        let extension = path.extension().and_then(|ext| ext.to_str());
        Self::detect_format(path, content)
            .ok_or_else(|| Self::unsupported_format_error(path, extension))
    }

    fn empty_file_error(path: &Path) -> FileLoaderError {
        FileLoaderError::EmptyFile {
            path: Self::path_string(path).into(),
        }
    }

    fn ensure_not_binary(
        bytes: &[u8],
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        Self::reject_null_bytes(bytes, path)
    }

    fn ensure_safe_path(path: &Path) -> Result<(), FileLoaderError> {
        Self::reject_absolute_path(path)?;
        Self::reject_parent_traversal(path)?;
        Ok(())
    }

    fn ensure_size_limit(
        metadata: &std::fs::Metadata,
        max_file_size_bytes: u64,
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        Self::reject_size_over_limit(metadata, max_file_size_bytes, path)
    }

    fn invalid_content_error(
        path: &Path,
        message: impl Into<String>,
    ) -> FileLoaderError {
        FileLoaderError::InvalidContent {
            path: Self::path_string(path).into(),
            message: message.into().into(),
        }
    }

    fn invalid_path_error(
        path: &Path,
        message: impl Into<String>,
    ) -> FileLoaderError {
        FileLoaderError::InvalidPath {
            path: Self::path_string(path).into(),
            message: message.into().into(),
        }
    }

    fn io_error(path: &Path, message: impl Into<String>) -> FileLoaderError {
        FileLoaderError::Io {
            path: Self::path_string(path).into(),
            message: message.into().into(),
        }
    }

    /// Creates a new file loader adapter with default limits.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
        }
    }

    fn parse_json(content: &str, path: &str) -> Result<(), FileLoaderError> {
        serde_json::from_str::<serde_json::Value>(content)
            .map(|_| ())
            .map_err(|err| format_parse_error(path, "JSON", &err.to_string()))
    }

    fn parse_toml(content: &str, path: &str) -> Result<(), FileLoaderError> {
        toml::from_str::<toml::Value>(content)
            .map(|_| ())
            .map_err(|err| format_parse_error(path, "TOML", &err.to_string()))
    }

    fn parse_yaml(content: &str, path: &str) -> Result<(), FileLoaderError> {
        serde_yaml::from_str::<serde_yaml::Value>(content)
            .map(|_| ())
            .map_err(|err| format_parse_error(path, "YAML", &err.to_string()))
    }

    fn path_string(path: &Path) -> String {
        path.display().to_string()
    }

    async fn read_bytes_and_metadata(
        path: &Path,
    ) -> Result<(Vec<u8>, std::fs::Metadata), FileLoaderError> {
        let path_buf = path.to_path_buf();
        let path_for_err = path.to_path_buf();
        spawn_blocking(move || {
            let metadata = std::fs::metadata(&path_buf)?;
            let bytes = std::fs::read(&path_buf)?;
            Ok::<_, std::io::Error>((bytes, metadata))
        })
        .await
        .map_err(|err| Self::io_error(&path_for_err, err.to_string()))?
        .map_err(|err| Self::io_error(&path_for_err, err.to_string()))
    }

    fn reject_absolute_path(path: &Path) -> Result<(), FileLoaderError> {
        if path.is_absolute() {
            return Err(Self::invalid_path_error(
                path,
                "absolute paths are not allowed",
            ));
        }

        Ok(())
    }

    fn reject_empty_bytes(
        bytes: &[u8],
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        if bytes.is_empty() {
            return Err(Self::empty_file_error(path));
        }

        Ok(())
    }

    fn reject_empty_content(
        content: &str,
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        if content.trim().is_empty() {
            return Err(Self::empty_file_error(path));
        }

        Ok(())
    }

    fn reject_null_bytes(
        bytes: &[u8],
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        if bytes.contains(&0) {
            return Err(Self::invalid_content_error(
                path,
                "binary content is not allowed",
            ));
        }

        Ok(())
    }

    fn reject_parent_traversal(path: &Path) -> Result<(), FileLoaderError> {
        if path.components().any(|component| {
            matches!(component, std::path::Component::ParentDir)
        }) {
            return Err(Self::invalid_path_error(
                path,
                "parent directory traversal is not allowed",
            ));
        }

        Ok(())
    }

    fn reject_size_over_limit(
        metadata: &std::fs::Metadata,
        max_file_size_bytes: u64,
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        if metadata.len() > max_file_size_bytes {
            return Err(Self::size_limit_error(
                path,
                max_file_size_bytes,
                metadata.len(),
            ));
        }

        Ok(())
    }

    fn size_limit_error(
        path: &Path,
        max_bytes: u64,
        actual_bytes: u64,
    ) -> FileLoaderError {
        FileLoaderError::SizeLimitExceeded {
            path: Self::path_string(path).into(),
            max_bytes,
            actual_bytes,
            message: "file exceeds size limit".to_owned().into(),
        }
    }

    fn unsupported_format_error(
        path: &Path,
        extension: Option<&str>,
    ) -> FileLoaderError {
        FileLoaderError::UnsupportedFormat {
            path: Self::path_string(path).into(),
            extension: extension.unwrap_or_default().to_owned().into(),
        }
    }

    async fn validate_format(
        content: &str,
        format: FileFormat,
        path: &Path,
    ) -> Result<(), FileLoaderError> {
        let path_string = path.display().to_string();
        let content = content.to_owned();
        let path_for_thread = path_string.clone();
        spawn_blocking(move || format.parse(&content, &path_for_thread))
            .await
            .map_err(|err| {
            Self::io_error(Path::new(&path_string), err.to_string())
        })?
    }

    /// Creates a new file loader adapter with a custom size limit.
    #[must_use]
    #[inline]
    pub fn with_max_size(max_file_size_bytes: u64) -> Self {
        Self {
            max_file_size_bytes,
        }
    }
}

#[async_trait]
impl FileReaderPort for FileReader {
    #[inline]
    async fn read(&self, path: &Path) -> Result<String, FileLoaderError> {
        Self::ensure_safe_path(path)?;

        let (bytes, metadata) = Self::read_bytes_and_metadata(path).await?;

        Self::ensure_size_limit(&metadata, self.max_file_size_bytes, path)?;
        Self::reject_empty_bytes(&bytes, path)?;
        Self::ensure_not_binary(&bytes, path)?;

        let content = Self::decode_utf8(bytes, path)?;

        Self::reject_empty_content(&content, path)?;

        let format = Self::detect_format_or_error(path, &content)?;
        Self::validate_format(&content, format, path).await?;

        Ok(content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileFormat {
    Json,
    Toml,
    Yaml,
}

impl FileFormat {
    fn from_content(content: &str) -> Option<Self> {
        let trimmed = content.trim_start();
        Self::from_json_marker(trimmed)
            .or_else(|| Self::from_toml_marker(trimmed))
            .or_else(|| Self::from_yaml_marker(trimmed))
    }

    fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    fn from_json_marker(trimmed: &str) -> Option<Self> {
        if trimmed.starts_with('{') {
            return Some(Self::Json);
        }

        None
    }

    fn from_toml_marker(trimmed: &str) -> Option<Self> {
        if trimmed.starts_with('[') {
            return Some(Self::Toml);
        }

        None
    }

    fn from_yaml_marker(trimmed: &str) -> Option<Self> {
        if trimmed.starts_with("---") {
            return Some(Self::Yaml);
        }

        None
    }

    fn parse(self, content: &str, path: &str) -> Result<(), FileLoaderError> {
        match self {
            Self::Json => FileReader::parse_json(content, path),
            Self::Toml => FileReader::parse_toml(content, path),
            Self::Yaml => FileReader::parse_yaml(content, path),
        }
    }
}

fn format_parse_error(
    path: &str,
    format: &str,
    message: &str,
) -> FileLoaderError {
    FileLoaderError::InvalidContent {
        path: path.to_owned().into(),
        message: format!("{format} parse error: {message}").into(),
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::disallowed_methods,
    reason = "Test utilities use simplified error handling patterns"
)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use lithos_test_utils::async_test;
    use proptest::prelude::*;

    use super::*;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> std::io::Result<Self> {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = PathBuf::from(format!(".tmp-file-loader-{nanos}"));
            std::fs::create_dir_all(&path)?;
            Ok(Self {
                path,
            })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _result = std::fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(path: &Path, contents: &str) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }

    #[test]
    fn detect_format_by_content_falls_back_to_markers() {
        assert_eq!(
            FileFormat::from_content("{ \"key\": true }"),
            Some(FileFormat::Json)
        );
        assert_eq!(
            FileFormat::from_content("---\nkey: value"),
            Some(FileFormat::Yaml)
        );
        assert_eq!(
            FileFormat::from_content("[section]\nkey = 1"),
            Some(FileFormat::Toml)
        );
    }

    #[test]
    fn detect_format_by_extension_prefers_known_values() {
        assert_eq!(FileFormat::from_extension("toml"), Some(FileFormat::Toml));
        assert_eq!(FileFormat::from_extension("json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("yaml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_extension("yml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_extension("ini"), None);
    }

    #[test]
    fn detect_format_prefers_extension_over_content() {
        let path = PathBuf::from("config.json");
        let content = "[section]\nkey = 1";

        assert_eq!(
            FileReader::detect_format(&path, content),
            Some(FileFormat::Json)
        );
    }

    #[test]
    fn detect_format_returns_none_for_unknown_content() {
        assert_eq!(FileFormat::from_content(""), None);
        assert_eq!(FileFormat::from_content("not sure"), None);
    }

    #[test]
    fn file_reader_default_uses_standard_limits() {
        let reader = FileReader::default();
        assert_eq!(reader.max_file_size_bytes, DEFAULT_MAX_FILE_SIZE_BYTES);
    }

    #[test]
    fn format_parse_error_includes_context() {
        let err = format_parse_error("test.toml", "TOML", "missing key");
        assert!(err.to_string().contains("test.toml"));
        assert!(err.to_string().contains("TOML"));
        assert!(err.to_string().contains("missing key"));
    }

    proptest! {
        #[test]
        fn detect_format_by_extension_is_case_insensitive(ext in "(TOML|toml|Json|JSON|yaml|YAML|yml|YML)") {
            let format = FileFormat::from_extension(&ext).expect("Valid extension");
            if ext.eq_ignore_ascii_case("toml") {
                prop_assert_eq!(format, FileFormat::Toml);
            } else if ext.eq_ignore_ascii_case("json") {
                prop_assert_eq!(format, FileFormat::Json);
            } else {
                prop_assert_eq!(format, FileFormat::Yaml);
            }
        }

        #[test]
        fn reject_absolute_path_always_fails(path in "/[a-z0-9/]+") {
            let p = PathBuf::from(path);
            prop_assert!(FileReader::reject_absolute_path(&p).is_err());
        }

        #[test]
        fn reject_parent_traversal_always_fails(path in "[a-z0-9/]*/\\.\\./[a-z0-9/]*") {
            let p = PathBuf::from(path);
            prop_assert!(FileReader::reject_parent_traversal(&p).is_err());
        }
    }

    async_test!(
        async fn load_file_reject_binary_content() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("binary.json");
            let contents = String::from("{\0}");
            if let Err(err) = write_file(&file_path, &contents) {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_empty_content() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("empty.toml");
            if let Err(err) = write_file(&file_path, "") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_invalid_json() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.json");
            if let Err(err) = write_file(&file_path, "{ invalid }") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_invalid_toml() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.toml");
            if let Err(err) = write_file(&file_path, "key = ") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_invalid_utf8() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("invalid.json");
            let contents = vec![0x7b, 0x22, 0xff, 0x22, 0x3a, 0x20, 0x31, 0x7d];
            if let Err(err) = std::fs::write(&file_path, &contents) {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_invalid_yaml() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.yaml");
            if let Err(err) = write_file(&file_path, "key: : value") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_oversized_content() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("big.toml");
            let contents = "a".repeat(64);
            if let Err(err) = write_file(&file_path, &contents) {
                panic!("write error: {err}");
            }

            let adapter = FileReader::with_max_size(4);
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_reject_unsupported_format() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.ini");
            if let Err(err) = write_file(&file_path, "key=value") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let result = adapter.read(&file_path).await;

            result.unwrap_err();
        }
    );

    async_test!(
        async fn load_file_succeeds_for_valid_toml() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.toml");
            if let Err(err) = write_file(&file_path, "[section]\nkey = 1") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let content =
                adapter.read(&file_path).await.unwrap_or_else(|err| {
                    panic!("expected file load to succeed: {err}");
                });

            assert!(content.contains("section"));
        }
    );

    async_test!(
        async fn load_file_works_for_valid_json() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.json");
            if let Err(err) = write_file(&file_path, "{\"key\": 1}") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let content =
                adapter.read(&file_path).await.unwrap_or_else(|err| {
                    panic!("expected file load to succeed: {err}");
                });

            assert!(content.contains("key"));
        }
    );

    async_test!(
        async fn load_file_works_for_valid_yaml() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config.yaml");
            if let Err(err) = write_file(&file_path, "key: value\n") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let content =
                adapter.read(&file_path).await.unwrap_or_else(|err| {
                    panic!("expected file load to succeed: {err}");
                });

            assert!(content.contains("key"));
        }
    );

    async_test!(
        async fn load_file_works_without_extension_by_content() {
            let temp_dir = TestDir::new().unwrap_or_else(|err| {
                panic!("temp dir error: {err}");
            });
            let file_path = temp_dir.path().join("config_no_ext");
            if let Err(err) = write_file(&file_path, "{\"key\": 1}") {
                panic!("write error: {err}");
            }

            let adapter = FileReader::new();
            let content =
                adapter.read(&file_path).await.unwrap_or_else(|err| {
                    panic!("expected file load to succeed: {err}");
                });

            assert!(content.contains("key"));
        }
    );
}
