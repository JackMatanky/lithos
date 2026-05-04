//! File type markers and parsing helpers for structured formats.
//!
//! This module is internal to the `fs` crate. It defines zero-sized type
//! markers for JSON, TOML, YAML, and Markdown together with their
//! extension-detection and content-sniffing helpers. All parsing goes through
//! [`crate::fs::reader::Reader::parse_structured`]; these types are not part
//! of the public API.

use std::path::Path;

use serde::de::DeserializeOwned;

use super::error::ParseError;

/// Supported file formats for structured parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum FormatKind {
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

/// Markdown file type marker.
///
/// Markdown does not use `detect`/`parse` here because parsing is delegated to
/// adapter-specific markdown implementations (e.g., pulldown-cmark).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Markdown;

impl Markdown {
    /// Check if this marker can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub(crate) fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md")
                || ext.eq_ignore_ascii_case("markdown")
        })
    }
}

/// JSON parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Json;

impl Json {
    /// Detect if content looks like JSON format.
    #[inline]
    #[must_use]
    pub(crate) fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        trimmed.starts_with('{') || trimmed.starts_with('[')
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub(crate) fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not JSON.
    #[inline]
    pub(crate) fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !Self::is_supported(path) {
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

/// TOML parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Toml;

impl Toml {
    /// Detect if content looks like TOML format.
    #[inline]
    #[must_use]
    pub(crate) fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && (trimmed.contains('[')
                || (trimmed.contains('=') && !trimmed.contains(':')))
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub(crate) fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not TOML.
    #[inline]
    pub(crate) fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !Self::is_supported(path) {
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

/// YAML parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Yaml;

impl Yaml {
    /// Detect if content looks like YAML format.
    #[inline]
    #[must_use]
    pub(crate) fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && !trimmed.starts_with('[')
            && (trimmed.starts_with("---")
                || (trimmed.contains(':') && !trimmed.contains('=')))
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub(crate) fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
        })
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not YAML.
    #[inline]
    pub(crate) fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        if !Self::is_supported(path) {
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

/// Binary file type marker.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Binary;

impl Binary {
    /// Check if this marker can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub(crate) fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("png")
                || ext.eq_ignore_ascii_case("jpg")
                || ext.eq_ignore_ascii_case("jpeg")
                || ext.eq_ignore_ascii_case("pdf")
                || ext.eq_ignore_ascii_case("zip")
                || ext.eq_ignore_ascii_case("wasm")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixtures for reusable content.
    mod fixtures {
        pub(crate) const VALID_JSON: &str = r#"{"name": "test", "value": 42}"#;
        pub(crate) const VALID_TOML: &str = "name = \"test\"\nvalue = 42";
        pub(crate) const VALID_YAML: &str = "name: test\nvalue: 42";
        pub(crate) const INVALID_TOML: &str =
            "name = 'unclosed string\nvalue = 42";
        pub(crate) const INVALID_JSON: &str = r#"
{
  "name": "test",
  "value": 42,
}
"#;
    }

    mod detect {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::table_header("[package]")]
        #[case::key_value("name = \"test\"")]
        #[case::both("[package]\nname = \"test\"")]
        fn detects_valid_toml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                Toml::detect(trimmed),
                "TOML detector should recognize valid TOML content: \
                 {content:?}"
            );
        }

        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn rejects_non_toml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                !Toml::detect(trimmed),
                "TOML detector should reject non-TOML content: {content:?}"
            );
        }

        #[rstest]
        #[case::object_start("{")]
        #[case::array_start("[")]
        #[case::full_object("{\"name\": \"test\"}")]
        fn detects_valid_json_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                Json::detect(trimmed),
                "JSON detector should recognize valid JSON content: \
                 {content:?}"
            );
        }

        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::toml_key_value("name = \"test\"")]
        #[case::plain_text("plain text")]
        fn rejects_non_json_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                !Json::detect(trimmed),
                "JSON detector should reject non-JSON content: {content:?}"
            );
        }

        #[rstest]
        #[case::document_separator("---")]
        #[case::key_value("name: test")]
        #[case::both("---\nname: test")]
        fn detects_valid_yaml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                Yaml::detect(trimmed),
                "YAML detector should recognize valid YAML content: \
                 {content:?}"
            );
        }

        #[rstest]
        #[case::toml_key_value("name = \"test\"")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn rejects_non_yaml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(
                !Yaml::detect(trimmed),
                "YAML detector should reject non-YAML content: {content:?}"
            );
        }
    }

    mod parse {
        use super::*;

        #[test]
        fn parses_valid_json() {
            let result: Result<serde_json::Value, _> =
                Json::parse(Path::new("test.json"), fixtures::VALID_JSON);
            assert!(result.is_ok(), "JSON parsing should succeed: {result:?}");
        }

        #[test]
        fn parses_valid_toml() {
            let result: Result<toml::Value, _> =
                Toml::parse(Path::new("test.toml"), fixtures::VALID_TOML);
            assert!(result.is_ok(), "TOML parsing should succeed: {result:?}");
        }

        #[test]
        fn parses_valid_yaml() {
            let result: Result<serde_yaml::Value, _> =
                Yaml::parse(Path::new("test.yaml"), fixtures::VALID_YAML);
            assert!(result.is_ok(), "YAML parsing should succeed: {result:?}");
        }

        #[test]
        fn rejects_unsupported_json_extension() {
            let result: Result<serde_json::Value, _> =
                Json::parse(Path::new("test.toml"), fixtures::VALID_JSON);
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        fn rejects_unsupported_toml_extension() {
            let result: Result<toml::Value, _> =
                Toml::parse(Path::new("test.yaml"), fixtures::VALID_TOML);
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        fn rejects_unsupported_yaml_extension() {
            let result: Result<serde_yaml::Value, _> =
                Yaml::parse(Path::new("test.toml"), fixtures::VALID_YAML);
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }
    }

    mod errors {
        use super::*;

        #[test]
        fn provides_toml_error_context() {
            let result = Toml::parse::<toml::Value>(
                Path::new("test.toml"),
                fixtures::INVALID_TOML,
            );
            assert!(
                matches!(result, Err(ParseError::Toml { .. })),
                "Invalid TOML should return Toml error variant, got: \
                 {result:?}"
            );
        }

        #[test]
        fn provides_json_error_context() {
            let result = Json::parse::<serde_json::Value>(
                Path::new("test.json"),
                fixtures::INVALID_JSON,
            );
            assert!(
                matches!(result, Err(ParseError::Json { .. })),
                "Invalid JSON should return Json error variant, got: \
                 {result:?}"
            );
        }

        #[test]
        fn provides_yaml_error_context() {
            let result = Yaml::parse::<serde_yaml::Value>(
                Path::new("test.yaml"),
                "name: test\n  invalid: indent",
            );
            assert!(
                matches!(result, Err(ParseError::Yaml { .. })),
                "Invalid YAML should return Yaml error variant, got: \
                 {result:?}"
            );
        }
    }

    mod extensions {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::standard("config.toml")]
        #[case::caps("config.TOML")]
        #[case::mixed("Config.Toml")]
        fn recognizes_valid_toml_extensions(#[case] path: &str) {
            assert!(
                Toml::is_supported(Path::new(path)),
                "TOML should support .toml extension (case-insensitive): \
                 {path}"
            );
        }

        #[test]
        fn rejects_invalid_toml_extensions() {
            assert!(
                !Toml::is_supported(Path::new("config.json")),
                "TOML should reject non-.toml extensions"
            );
        }

        #[rstest]
        #[case::standard("config.json")]
        #[case::caps("config.JSON")]
        #[case::mixed("Config.Json")]
        fn recognizes_valid_json_extensions(#[case] path: &str) {
            assert!(
                Json::is_supported(Path::new(path)),
                "JSON should support .json extension (case-insensitive): \
                 {path}"
            );
        }

        #[test]
        fn rejects_invalid_json_extensions() {
            assert!(
                !Json::is_supported(Path::new("config.toml")),
                "JSON should reject non-.json extensions"
            );
        }

        #[rstest]
        #[case::standard_yaml("config.yaml")]
        #[case::standard_yml("config.yml")]
        #[case::caps("config.YAML")]
        #[case::mixed("Config.Yml")]
        fn recognizes_valid_yaml_extensions(#[case] path: &str) {
            assert!(
                Yaml::is_supported(Path::new(path)),
                "YAML should support .yaml/.yml extensions \
                 (case-insensitive): {path}"
            );
        }

        #[test]
        fn rejects_invalid_yaml_extensions() {
            assert!(
                !Yaml::is_supported(Path::new("config.toml")),
                "YAML should reject non-.yaml/.yml extensions"
            );
        }

        #[rstest]
        #[case::standard_md("readme.md")]
        #[case::standard_markdown("readme.markdown")]
        #[case::caps("README.MD")]
        #[case::mixed("Readme.Markdown")]
        fn recognizes_valid_markdown_extensions(#[case] path: &str) {
            assert!(
                Markdown::is_supported(Path::new(path)),
                "Markdown should support .md/.markdown extensions \
                 (case-insensitive): {path}"
            );
        }

        #[test]
        fn rejects_invalid_markdown_extensions() {
            assert!(
                !Markdown::is_supported(Path::new("readme.txt")),
                "Markdown should reject non-.md/.markdown extensions"
            );
        }
    }
}
