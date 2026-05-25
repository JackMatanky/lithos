//! File format detection and classification.

use std::{ffi::OsStr, path::Path};

use rkyv::{Archive, Deserialize, Serialize};
use serde::de::DeserializeOwned;

use super::error::ParseError;

/// Supported file formats for structured parsing and classification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FileFormat {
    /// JSON format.
    Json,
    /// TOML format.
    Toml,
    /// YAML format.
    Yaml,
    /// Markdown format.
    Markdown,
    /// Image format (png, jpg, jpeg, gif, webp, svg, bmp, ico).
    Image,
    /// PDF format.
    Pdf,
    /// Text-based document format (doc, docx, odt, rtf, txt).
    Document,
    /// Archive format (zip, tar, gz, rar, 7z, wasm).
    Archive,
    /// Fallback for other binary formats.
    Binary,
    /// Unknown or unrecognized format.
    Unknown,
}

impl FileFormat {
    /// Detect format from file extension.
    #[inline]
    #[must_use]
    pub fn from_extension(ext: &OsStr) -> Self {
        let ext_str = ext.to_str().unwrap_or_default().to_ascii_lowercase();
        match ext_str.as_str() {
            "json" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "md" | "markdown" => Self::Markdown,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => {
                Self::Image
            }
            "pdf" => Self::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "txt" => Self::Document,
            "zip" | "tar" | "gz" | "rar" | "7z" | "wasm" => Self::Archive,
            _ => Self::Unknown,
        }
    }

    /// Returns `true` if this is a markdown format.
    #[inline]
    #[must_use]
    pub fn is_markdown(&self) -> bool {
        matches!(self, Self::Markdown)
    }

    /// Returns `true` if this is a structured data format.
    #[inline]
    #[must_use]
    pub fn is_structured(&self) -> bool {
        matches!(self, Self::Json | Self::Toml | Self::Yaml)
    }

    /// Returns a stable lowercase storage key for this format.
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Markdown => "markdown",
            Self::Image => "image",
            Self::Pdf => "pdf",
            Self::Document => "document",
            Self::Archive => "archive",
            Self::Binary => "binary",
            Self::Unknown => "unknown",
        }
    }
}

#[inline]
fn extension_is_supported(path: &Path, extensions: &[&str]) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        extensions.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate))
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
struct JsonParser;

impl JsonParser {
    #[inline]
    fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        trimmed.starts_with('{') || trimmed.starts_with('[')
    }

    #[inline]
    fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !extension_is_supported(path, &["json"]) {
            return Err(ParseError::UnsupportedFormat {
                path: path.to_path_buf(),
                supported: &["json"],
            });
        }

        serde_json::from_str(content).map_err(|error| ParseError::Json {
            path: path.to_path_buf(),
            message: error.to_string().into(),
            line: Some(error.line()),
            column: Some(error.column()),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct TomlParser;

impl TomlParser {
    #[inline]
    fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && (trimmed.contains('[')
                || (trimmed.contains('=') && !trimmed.contains(':')))
    }

    #[inline]
    fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !extension_is_supported(path, &["toml"]) {
            return Err(ParseError::UnsupportedFormat {
                path: path.to_path_buf(),
                supported: &["toml"],
            });
        }

        toml::from_str(content).map_err(|error| {
            let (line, column) = error
                .span()
                .and_then(|span| content.get(..span.start))
                .map_or((None, None), |prefix| {
                    let line_no = prefix.lines().count();
                    let col = prefix.lines().last().map_or(0, str::len);
                    (Some(line_no), Some(col))
                });

            ParseError::Toml {
                path: path.to_path_buf(),
                message: error.message().into(),
                line,
                column,
            }
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct YamlParser;

impl YamlParser {
    #[inline]
    fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && !trimmed.starts_with('[')
            && (trimmed.starts_with("---")
                || (trimmed.contains(':') && !trimmed.contains('=')))
    }

    #[inline]
    fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !extension_is_supported(path, &["yaml", "yml"]) {
            return Err(ParseError::UnsupportedFormat {
                path: path.to_path_buf(),
                supported: &["yaml", "yml"],
            });
        }

        serde_yaml::from_str(content).map_err(|error| {
            let (line, column) = error.location().map_or((None, None), |loc| {
                (Some(loc.line()), Some(loc.column()))
            });

            ParseError::Yaml {
                path: path.to_path_buf(),
                message: error.to_string().into(),
                line,
                column,
            }
        })
    }
}

/// Attempts to classify structured content from its textual shape.
///
/// Returns `Some(FileFormat)` for JSON/TOML/YAML-like payloads and `None`
/// when no structured format signature is detected.
#[inline]
#[must_use]
pub fn sniff_structured_format(content: &str) -> Option<FileFormat> {
    if JsonParser::detect(content) {
        return Some(FileFormat::Json);
    }
    if YamlParser::detect(content) {
        return Some(FileFormat::Yaml);
    }
    if TomlParser::detect(content) {
        return Some(FileFormat::Toml);
    }
    None
}

/// Parses structured content using an explicit file format classification.
///
/// # Errors
/// Returns `ParseError::UnsupportedFormat` when `format` is not JSON/TOML/YAML,
/// or parser-specific errors for malformed content.
#[inline]
pub(crate) fn parse_from_format<T: DeserializeOwned>(
    path: &Path,
    content: &str,
    format: FileFormat,
) -> Result<T, ParseError> {
    match format {
        FileFormat::Json => JsonParser::parse(path, content),
        FileFormat::Toml => TomlParser::parse(path, content),
        FileFormat::Yaml => YamlParser::parse(path, content),
        FileFormat::Markdown
        | FileFormat::Image
        | FileFormat::Pdf
        | FileFormat::Document
        | FileFormat::Archive
        | FileFormat::Binary
        | FileFormat::Unknown => Err(ParseError::UnsupportedFormat {
            path: path.to_path_buf(),
            supported: &["json", "toml", "yaml", "yml"],
        }),
    }
}

/// Borrowed extension view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileExtensionRef<'a>(pub(crate) &'a OsStr);

impl FileExtensionRef<'_> {
    /// Get the format for this extension.
    #[inline]
    #[must_use]
    pub fn format(&self) -> FileFormat {
        FileFormat::from_extension(self.0)
    }

    /// Get the extension as a string slice if valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn should_detect_various_formats() {
        assert_eq!(
            FileFormat::from_extension(OsStr::new("md")),
            FileFormat::Markdown
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("PNG")),
            FileFormat::Image
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("pdf")),
            FileFormat::Pdf
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("txt")),
            FileFormat::Document
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("zip")),
            FileFormat::Archive
        );
        assert_eq!(
            FileFormat::from_extension(OsStr::new("unknown")),
            FileFormat::Unknown
        );
    }

    #[test]
    fn should_identify_structured_formats() {
        assert!(FileFormat::Json.is_structured());
        assert!(FileFormat::Toml.is_structured());
        assert!(FileFormat::Yaml.is_structured());
        assert!(!FileFormat::Markdown.is_structured());
        assert!(!FileFormat::Image.is_structured());
    }

    #[test]
    fn should_sniff_structured_format_by_content() {
        assert_eq!(
            sniff_structured_format("{\"k\":1}"),
            Some(FileFormat::Json)
        );
        assert_eq!(
            sniff_structured_format("name: test\nvalue: 42"),
            Some(FileFormat::Yaml)
        );
        assert_eq!(
            sniff_structured_format("name = \"test\""),
            Some(FileFormat::Toml)
        );
        assert_eq!(sniff_structured_format("plain text"), None);
    }

    #[test]
    fn parse_from_format_rejects_unsupported_format() {
        let result: Result<serde_json::Value, ParseError> = parse_from_format(
            Path::new("note.md"),
            "# title",
            FileFormat::Markdown,
        );
        assert!(matches!(result, Err(ParseError::UnsupportedFormat { .. })));
    }
}
