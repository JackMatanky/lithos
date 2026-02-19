//! File type markers and parsing helpers for structured formats.
//!
//! This module defines file type helpers used to classify and parse
//! structured configuration files. JSON/TOML/YAML expose detect + parse
//! helpers; Markdown is represented as a file type without detect/parse
//! support. This keeps format identification centralized so reader pipelines
//! remain deterministic and adapter code can avoid ad-hoc extension checks.
//!
//! # Usage
//!
//! ```
//! use std::path::Path;
//!
//! use lithos_core::fs::Json;
//! use serde_json::Value;
//!
//! let content = r#"{"name": "lithos"}"#;
//! let value: Value = Json::parse(Path::new("config.json"), content)?;
//! println!("{:?}", value);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::Path;

use serde::de::DeserializeOwned;

use super::error::ParseError;

/// JSON parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Json;

/// TOML parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Toml;

/// YAML parser strategy.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Yaml;

/// Markdown file type marker.
///
/// Markdown does not use `detect`/`parse` here because parsing is delegated to
/// adapter-specific markdown implementations (e.g., pulldown-cmark).
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Markdown;

impl Json {
    /// Detect if content looks like JSON format.
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        trimmed.starts_with('{') || trimmed.starts_with('[')
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not JSON.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
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

impl Toml {
    /// Detect if content looks like TOML format.
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && (trimmed.contains('[')
                || (trimmed.contains('=') && !trimmed.contains(':')))
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not TOML.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
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

impl Yaml {
    /// Detect if content looks like YAML format.
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        !trimmed.starts_with('{')
            && !trimmed.starts_with('[')
            && (trimmed.starts_with("---")
                || (trimmed.contains(':') && !trimmed.contains('=')))
    }

    /// Check if this parser can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
        })
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    /// Returns `ParseError` if parsing fails or the extension is not YAML.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
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

impl Markdown {
    /// Check if this marker can handle the given file path by extension.
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md")
                || ext.eq_ignore_ascii_case("markdown")
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
        fn should_detect_valid_toml_content(#[case] content: &str) {
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
        fn should_reject_non_toml_content(#[case] content: &str) {
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
        fn should_detect_valid_json_content(#[case] content: &str) {
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
        fn should_reject_non_json_content(#[case] content: &str) {
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
        fn should_detect_valid_yaml_content(#[case] content: &str) {
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
        fn should_reject_non_yaml_content(#[case] content: &str) {
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
        fn should_parse_json() {
            let result: Result<serde_json::Value, _> =
                Json::parse(Path::new("test.json"), fixtures::VALID_JSON);
            assert!(result.is_ok(), "JSON parsing should succeed: {result:?}");
        }

        #[test]
        fn should_parse_toml() {
            let result: Result<toml::Value, _> =
                Toml::parse(Path::new("test.toml"), fixtures::VALID_TOML);
            assert!(result.is_ok(), "TOML parsing should succeed: {result:?}");
        }

        #[test]
        fn should_parse_yaml() {
            let result: Result<serde_yaml::Value, _> =
                Yaml::parse(Path::new("test.yaml"), fixtures::VALID_YAML);
            assert!(result.is_ok(), "YAML parsing should succeed: {result:?}");
        }

        #[test]
        fn should_reject_unsupported_json_extension() {
            let result: Result<serde_json::Value, _> =
                Json::parse(Path::new("test.toml"), fixtures::VALID_JSON);
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        fn should_reject_unsupported_toml_extension() {
            let result: Result<toml::Value, _> =
                Toml::parse(Path::new("test.yaml"), fixtures::VALID_TOML);
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        fn should_reject_unsupported_yaml_extension() {
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
        fn should_provide_toml_error_context() {
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
        fn should_provide_json_error_context() {
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
        fn should_provide_yaml_error_context() {
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
        fn should_recognize_valid_toml_extensions(#[case] path: &str) {
            assert!(
                Toml::is_supported(Path::new(path)),
                "TOML should support .toml extension (case-insensitive): \
                 {path}"
            );
        }

        #[test]
        fn should_reject_invalid_toml_extensions() {
            assert!(
                !Toml::is_supported(Path::new("config.json")),
                "TOML should reject non-.toml extensions"
            );
        }

        #[rstest]
        #[case::standard("config.json")]
        #[case::caps("config.JSON")]
        #[case::mixed("Config.Json")]
        fn should_recognize_valid_json_extensions(#[case] path: &str) {
            assert!(
                Json::is_supported(Path::new(path)),
                "JSON should support .json extension (case-insensitive): \
                 {path}"
            );
        }

        #[test]
        fn should_reject_invalid_json_extensions() {
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
        fn should_recognize_valid_yaml_extensions(#[case] path: &str) {
            assert!(
                Yaml::is_supported(Path::new(path)),
                "YAML should support .yaml/.yml extensions \
                 (case-insensitive): {path}"
            );
        }

        #[test]
        fn should_reject_invalid_yaml_extensions() {
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
        fn should_recognize_valid_markdown_extensions(#[case] path: &str) {
            assert!(
                Markdown::is_supported(Path::new(path)),
                "Markdown should support .md/.markdown extensions \
                 (case-insensitive): {path}"
            );
        }

        #[test]
        fn should_reject_invalid_markdown_extensions() {
            assert!(
                !Markdown::is_supported(Path::new("readme.txt")),
                "Markdown should reject non-.md/.markdown extensions"
            );
        }
    }
}
