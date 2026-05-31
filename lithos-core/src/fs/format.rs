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

/// Discovery selector for structured file formats.
///
/// This keeps extension-level distinctions used by discovery/candidate
/// resolution while allowing parse-time handoff into [`FileFormat`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum StructuredFileFormat {
    /// TOML format (`.toml`).
    Toml,
    /// JSON format (`.json`).
    Json,
    /// YAML format (`.yaml`).
    Yaml,
    /// YAML format via `.yml` extension.
    Yml,
}

impl StructuredFileFormat {
    /// Stable precedence used by deterministic candidate selection.
    pub const PRECEDENCE: [Self; 4] =
        [Self::Toml, Self::Json, Self::Yaml, Self::Yml];

    /// Canonical extension (without leading dot).
    #[inline]
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Yml => "yml",
        }
    }

    /// Stable precedence rank where lower is higher priority.
    #[inline]
    #[must_use]
    pub const fn precedence_rank(self) -> usize {
        match self {
            Self::Toml => 0,
            Self::Json => 1,
            Self::Yaml => 2,
            Self::Yml => 3,
        }
    }

    /// Converts a file extension into a structured selector variant.
    #[inline]
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        if extension.eq_ignore_ascii_case("toml") {
            return Some(Self::Toml);
        }
        if extension.eq_ignore_ascii_case("json") {
            return Some(Self::Json);
        }
        if extension.eq_ignore_ascii_case("yaml") {
            return Some(Self::Yaml);
        }
        if extension.eq_ignore_ascii_case("yml") {
            return Some(Self::Yml);
        }
        None
    }

    /// Detects selector variant from a path's extension.
    #[inline]
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension().and_then(OsStr::to_str).and_then(Self::from_extension)
    }
}

impl From<StructuredFileFormat> for FileFormat {
    #[inline]
    fn from(value: StructuredFileFormat) -> Self {
        match value {
            StructuredFileFormat::Toml => Self::Toml,
            StructuredFileFormat::Json => Self::Json,
            StructuredFileFormat::Yaml | StructuredFileFormat::Yml => {
                Self::Yaml
            }
        }
    }
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
    fn structured_file_format_precedence_is_stable() {
        assert_eq!(
            StructuredFileFormat::PRECEDENCE[0],
            StructuredFileFormat::Toml
        );
        assert_eq!(
            StructuredFileFormat::PRECEDENCE[1],
            StructuredFileFormat::Json
        );
        assert_eq!(
            StructuredFileFormat::PRECEDENCE[2],
            StructuredFileFormat::Yaml
        );
        assert_eq!(
            StructuredFileFormat::PRECEDENCE[3],
            StructuredFileFormat::Yml
        );
        assert!(
            StructuredFileFormat::Toml.precedence_rank()
                < StructuredFileFormat::Json.precedence_rank()
        );
        assert!(
            StructuredFileFormat::Json.precedence_rank()
                < StructuredFileFormat::Yaml.precedence_rank()
        );
        assert!(
            StructuredFileFormat::Yaml.precedence_rank()
                < StructuredFileFormat::Yml.precedence_rank()
        );
    }

    #[test]
    fn structured_file_format_maps_extensions_case_insensitively() {
        assert_eq!(
            StructuredFileFormat::from_extension("TOML"),
            Some(StructuredFileFormat::Toml)
        );
        assert_eq!(
            StructuredFileFormat::from_extension("Json"),
            Some(StructuredFileFormat::Json)
        );
        assert_eq!(
            StructuredFileFormat::from_extension("YAML"),
            Some(StructuredFileFormat::Yaml)
        );
        assert_eq!(
            StructuredFileFormat::from_extension("YmL"),
            Some(StructuredFileFormat::Yml)
        );
        assert_eq!(StructuredFileFormat::from_extension("md"), None);
    }

    #[test]
    fn structured_file_format_maps_paths_and_unknowns() {
        assert_eq!(
            StructuredFileFormat::from_path(Path::new("schema.toml")),
            Some(StructuredFileFormat::Toml)
        );
        assert_eq!(
            StructuredFileFormat::from_path(Path::new("schema.JSON")),
            Some(StructuredFileFormat::Json)
        );
        assert_eq!(
            StructuredFileFormat::from_path(Path::new("schema.yaml")),
            Some(StructuredFileFormat::Yaml)
        );
        assert_eq!(
            StructuredFileFormat::from_path(Path::new("schema.YML")),
            Some(StructuredFileFormat::Yml)
        );
        assert_eq!(
            StructuredFileFormat::from_path(Path::new("schema.md")),
            None
        );
        assert_eq!(StructuredFileFormat::from_path(Path::new("schema")), None);
    }

    #[test]
    fn structured_file_format_converts_to_parse_format() {
        assert_eq!(
            FileFormat::from(StructuredFileFormat::Toml),
            FileFormat::Toml
        );
        assert_eq!(
            FileFormat::from(StructuredFileFormat::Json),
            FileFormat::Json
        );
        assert_eq!(
            FileFormat::from(StructuredFileFormat::Yaml),
            FileFormat::Yaml
        );
        assert_eq!(
            FileFormat::from(StructuredFileFormat::Yml),
            FileFormat::Yaml
        );
    }

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
