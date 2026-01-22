//! File parsing strategies for TOML, JSON, and YAML.
//!
//! This module provides the parser infrastructure for Epic 4 (File Loading
//! Strategy Foundation). Parsers are used by adapters implementing domain ports
//! (`ConfigQuery`, `SchemaQuery`, `TemplateQuery`) to deserialize files
//! into domain types.
//!
//! # Architecture
//!
//! - **Strategy Pattern**: Each format (TOML/JSON/YAML) has its own parser
//! - **Auto-Detection**: `Dispatcher` selects parser by file extension
//! - **Rich Errors**: Parse errors include file path, line numbers, and context
//!
//! # Usage
//!
//! ```ignore
//! use lithos_adapters::spi::parsers::Dispatcher;
//! use lithos_domain::Global;
//!
//! let dispatcher = Dispatcher::new();
//! let content = tokio::fs::read_to_string("lithos.toml").await?;
//! let config: Global = dispatcher.parse(Path::new("lithos.toml"), &content)?;
//! ```
//!
//! # Epic Dependencies
//!
//! - **Epic 5** (Configuration): Loads `Global`, `Vault` configs
//! - **Epic 6** (Schema): Loads `RawSchema`, `Property` definitions
//! - **Epic 11** (Templates): Loads `Template` definitions

use std::path::Path;

use serde::de::DeserializeOwned;

use super::errors::ParseError;

/// Parser strategy enum for different config file formats.
///
/// Uses enum dispatch instead of trait objects for better type safety and
/// performance.
#[expect(
    clippy::exhaustive_enums,
    reason = "Known set of supported configuration formats"
)]
#[derive(Debug, Clone)]
pub enum ParserStrategy {
    /// JSON parser.
    Json(Json),
    /// TOML parser.
    Toml(Toml),
    /// YAML parser.
    Yaml(Yaml),
}

impl ParserStrategy {
    /// Check if this parser can handle the given file path.
    #[inline]
    #[must_use]
    pub fn can_parse(&self, path: &Path) -> bool {
        match *self {
            Self::Json(_) => Json::can_parse(path),
            Self::Toml(_) => Toml::can_parse(path),
            Self::Yaml(_) => Yaml::can_parse(path),
        }
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        match *self {
            Self::Json(_) => Json::parse(path, content),
            Self::Toml(_) => Toml::parse(path, content),
            Self::Yaml(_) => Yaml::parse(path, content),
        }
    }
}

/// TOML parser strategy.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Toml;

impl Toml {
    /// Check if this parser can handle the given file path.
    #[inline]
    #[must_use]
    pub fn can_parse(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    #[expect(
        clippy::string_slice,
        reason = "TOML spans are byte offsets guaranteed to be on char \
                  boundaries"
    )]
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        toml::from_str(content).map_err(|e| {
            let (line, column) = e.span().map_or((None, None), |span| {
                // Convert byte offset to line/column (approximation)
                let lines: Vec<&str> = content[..span.start].lines().collect();
                let line = lines.len();
                let column = lines.last().map_or(0, |l| l.len());
                (Some(line), Some(column))
            });

            ParseError::Toml {
                path: path.to_path_buf(),
                message: e.message().to_owned(),
                line,
                column,
            }
        })
    }
}

/// JSON parser strategy.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Json;

impl Json {
    /// Check if this parser can handle the given file path.
    #[inline]
    #[must_use]
    pub fn can_parse(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        serde_json::from_str(content).map_err(|e| ParseError::Json {
            path: path.to_path_buf(),
            message: e.to_string(),
            line: Some(e.line()),
            column: Some(e.column()),
        })
    }
}

/// YAML parser strategy.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Yaml;

impl Yaml {
    /// Check if this parser can handle the given file path.
    #[inline]
    #[must_use]
    pub fn can_parse(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
        })
    }

    /// Parse content string into type T.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        serde_yaml::from_str(content).map_err(|e| {
            let (line, column) = e.location().map_or((None, None), |loc| {
                (Some(loc.line()), Some(loc.column()))
            });

            ParseError::Yaml {
                path: path.to_path_buf(),
                message: e.to_string(),
                line,
                column,
            }
        })
    }
}

/// Auto-detecting file parser.
///
/// Dispatches to the appropriate parser based on file extension.
/// Supports TOML, JSON, and YAML formats.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Dispatcher {
    parsers: Vec<ParserStrategy>,
}

impl Default for Dispatcher {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    /// Create a new dispatcher with all format strategies.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            parsers: vec![
                ParserStrategy::Json(Json),
                ParserStrategy::Toml(Toml),
                ParserStrategy::Yaml(Yaml),
            ],
        }
    }

    /// Parse file content with automatic format detection.
    ///
    /// Format is determined by file extension. Supports:
    /// - `.toml` → TOML
    /// - `.json` → JSON
    /// - `.yaml`, `.yml` → YAML
    ///
    /// # Errors
    ///
    /// Returns `ParseError::UnsupportedFormat` if file extension is not
    /// recognized, or format-specific parse error if deserialization fails.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        for parser in &self.parsers {
            if parser.can_parse(path) {
                return parser.parse(path, content);
            }
        }

        Err(ParseError::UnsupportedFormat {
            path: path.to_path_buf(),
            supported: vec!["toml", "json", "yaml", "yml"],
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

    mod extensions {
        use rstest::rstest;

        use super::*;

        // [4.1-U-01] TOML extension detection
        #[rstest]
        #[case::standard("config.toml")]
        #[case::caps("config.TOML")]
        #[case::mixed("Config.Toml")]
        fn should_recognize_valid_toml_extensions(#[case] path: &str) {
            // GIVEN a path with a valid TOML extension variant
            let path = Path::new(path);

            // WHEN checking if the Toml parser can handle it
            let result = Toml::can_parse(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as TOML", path.display());
        }

        // [4.1-U-01] TOML extension rejection
        #[test]
        fn should_reject_invalid_toml_extensions() {
            // GIVEN a path with a non-TOML extension (json)
            let path = Path::new("config.json");

            // WHEN checking if the Toml parser can handle it
            let result = Toml::can_parse(path);

            // THEN it should return false
            assert!(!result, "Should not recognize JSON as TOML");
        }

        // [4.1-U-02] JSON extension detection
        #[rstest]
        #[case::standard("config.json")]
        #[case::caps("config.JSON")]
        #[case::mixed("Config.Json")]
        fn should_recognize_valid_json_extensions(#[case] path: &str) {
            // GIVEN a path with a valid JSON extension variant
            let path = Path::new(path);

            // WHEN checking if the Json parser can handle it
            let result = Json::can_parse(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as JSON", path.display());
        }

        // [4.1-U-02] JSON extension rejection
        #[test]
        fn should_reject_invalid_json_extensions() {
            // GIVEN a path with a non-JSON extension (toml)
            let path = Path::new("config.toml");

            // WHEN checking if the Json parser can handle it
            let result = Json::can_parse(path);

            // THEN it should return false
            assert!(!result, "Should not recognize TOML as JSON");
        }

        // [4.1-U-03] YAML extension detection
        #[rstest]
        #[case::standard_yaml("config.yaml")]
        #[case::standard_yml("config.yml")]
        #[case::caps("config.YAML")]
        #[case::mixed("Config.Yml")]
        fn should_recognize_valid_yaml_extensions(#[case] path: &str) {
            // GIVEN a path with a valid YAML extension variant
            let path = Path::new(path);

            // WHEN checking if the Yaml parser can handle it
            let result = Yaml::can_parse(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as YAML", path.display());
        }

        // [4.1-U-03] YAML extension rejection
        #[test]
        fn should_reject_invalid_yaml_extensions() {
            // GIVEN a path with a non-YAML extension (toml)
            let path = Path::new("config.toml");

            // WHEN checking if the Yaml parser can handle it
            let result = Yaml::can_parse(path);

            // THEN it should return false
            assert!(!result, "Should not recognize TOML as YAML");
        }
    }

    mod dispatch {
        use super::*;

        // [4.1-U-04] Dispatcher JSON
        #[test]
        #[expect(
            clippy::disallowed_methods,
            clippy::indexing_slicing,
            reason = "Test setup uses unwrap for clarity and assertions are \
                      performed on known JSON structure"
        )]
        fn should_dispatch_json_correctly() {
            // GIVEN a dispatcher, valid JSON content, and a .json path
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_JSON;
            let path = Path::new("test.json");

            // WHEN parsing the content via the dispatcher
            let result: Result<serde_json::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the JSON
            assert!(result.is_ok(), "Should parse JSON successfully");
            let value = result.unwrap();
            assert_eq!(value["name"], "test");
        }

        // [4.1-U-05] Dispatcher TOML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            clippy::indexing_slicing,
            reason = "Test setup uses unwrap for clarity and assertions are \
                      performed on known TOML structure"
        )]
        fn should_dispatch_toml_correctly() {
            // GIVEN a dispatcher, valid TOML content, and a .toml path
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_TOML;
            let path = Path::new("test.toml");

            // WHEN parsing the content via the dispatcher
            let result: Result<toml::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the TOML
            assert!(result.is_ok(), "Should parse TOML successfully");
            let value = result.unwrap();
            assert_eq!(value["name"].as_str(), Some("test"));
        }

        // [4.1-U-06] Dispatcher YAML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            clippy::indexing_slicing,
            reason = "Test setup uses unwrap for clarity and assertions are \
                      performed on known YAML structure"
        )]
        fn should_dispatch_yaml_correctly() {
            // GIVEN a dispatcher, valid YAML content, and a .yaml path
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_YAML;
            let path = Path::new("test.yaml");

            // WHEN parsing the content via the dispatcher
            let result: Result<serde_yaml::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the YAML
            assert!(result.is_ok(), "Should parse YAML successfully");
            let value = result.unwrap();
            assert_eq!(value["name"].as_str(), Some("test"));
        }

        // [4.1-U-07] Dispatcher Unsupported Format
        #[test]
        fn should_reject_unsupported_format() {
            // GIVEN a dispatcher and a path with an unsupported extension
            // (.xml)
            let dispatcher = Dispatcher::new();
            let path = Path::new("test.xml");

            // WHEN attempting to parse content
            let result = dispatcher.parse::<serde_json::Value>(path, "");

            // THEN it should return an UnsupportedFormat error
            assert!(
                matches!(result, Err(ParseError::UnsupportedFormat { .. })),
                "Should return UnsupportedFormat error"
            );
        }
    }

    mod errors {
        use super::*;

        // [4.1-U-08] TOML Error Context
        #[test]
        #[expect(
            clippy::panic,
            reason = "Panic used to fail test on unexpected error variant"
        )]
        fn should_provide_toml_error_context() {
            // GIVEN invalid TOML content with an unclosed string
            let invalid_toml = fixtures::INVALID_TOML;
            let path = Path::new("test.toml");

            // WHEN attempting to parse it
            let result = Toml::parse::<toml::Value>(path, invalid_toml);

            // THEN it should return a Toml error containing context about the
            // string issue
            assert!(result.is_err());
            if let Err(ParseError::Toml {
                message,
                ..
            }) = result
            {
                assert!(
                    message.contains("string"),
                    "Error message should mention string issue"
                );
            } else {
                panic!("Expected ParseError::Toml");
            }
        }

        // [4.1-U-09] JSON Error Context
        #[test]
        #[expect(
            clippy::panic,
            reason = "Panic used to fail test on unexpected error variant"
        )]
        fn should_provide_json_error_context() {
            // GIVEN invalid JSON content with a trailing comma
            let invalid_json = fixtures::INVALID_JSON;
            let path = Path::new("test.json");

            // WHEN attempting to parse it
            let result = Json::parse::<serde_json::Value>(path, invalid_json);

            // THEN it should return a Json error with line and column
            // information
            assert!(result.is_err());
            if let Err(ParseError::Json {
                line,
                column,
                ..
            }) = result
            {
                assert!(line.is_some(), "Should provide line number");
                assert!(column.is_some(), "Should provide column number");
            } else {
                panic!("Expected ParseError::Json");
            }
        }
    }
}
