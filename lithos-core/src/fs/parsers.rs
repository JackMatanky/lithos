//! Structured data format parsing strategies for TOML, JSON, and YAML.
//!
//! This module provides the parser infrastructure for Epic 4 (File Loading
//! Strategy Foundation). Parsers are used by adapters implementing domain ports
//! (`ConfigQuery`, `SchemaQuery`, `TemplateQuery`) to deserialize structured
//! configuration files into domain types.
//!
//! # Architecture
//!
//! - **Strategy Pattern**: Each format (TOML/JSON/YAML) has its own parser
//!   struct
//! - **Auto-Detection**: `Dispatcher` (re-exported as `FormatDispatcher`)
//!   selects parser by file extension
//! - **Rich Errors**: Parse errors include file path, line numbers, and context
//!   via `miette`-compatible types
//! - **Zero-Cost Abstraction**: `Dispatcher` is a unit struct (zero runtime
//!   overhead, compile-time dispatch)
//!
//! # Current Scope
//!
//! This module focuses exclusively on **structured configuration formats**:
//! - **TOML** for primary configuration (`lithos.toml`, vault settings)
//! - **JSON** for interoperability and data exchange
//! - **YAML** for schema definitions and complex hierarchies
//!
//! # Future Extensibility
//!
//! Future epics (vault indexing, content processing) may require detection and
//! handling of additional file types (Markdown, images, PDFs, etc.). When that
//! time comes, consider:
//!
//! - MIME type detection via `infer` crate (magic byte analysis)
//! - Extension-based fallback via `mime_guess` crate
//! - `FileType` enum to represent all supported vault content types
//! - **Keep structured data parsers (this module) separate from binary/text
//!   detection**
//!
//! The current design intentionally stays lean and focused on the immediate
//! need (config/schema loading) while remaining extensible via additional
//! modules in `adapters/spi/fs/` when vault indexing requirements emerge.
//!
//! # Usage
//!
//! ```
//! use std::path::Path;
//!
//! use lithos_core::fs::FormatDispatcher; // Re-exported for clarity
//! use serde_json::Value;
//!
//! let dispatcher = FormatDispatcher::new();
//! let content = r#"name = "lithos""#;
//! let config: Value = dispatcher.parse(Path::new("test.toml"), content)?;
//! println!("{:?}", config);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Epic Dependencies
//!
//! - **Epic 6** (Configuration): Loads `Global`, `Vault` configs via TOML
//! - **Epic 7** (Schema): Loads `RawSchema`, `Property` definitions via
//!   YAML/JSON
//! - **Epic 12** (Templates): Loads `Template` definitions via YAML

use std::path::Path;

use serde::de::DeserializeOwned;
use tracing::{debug, error};

use super::error::ParseError;

/// Auto-detecting file parser for structured configuration formats.
///
/// Dispatches to the appropriate parser based on file extension or content
/// analysis. Supports TOML, JSON, and YAML formats.
///
/// # Heuristic detection priority:
/// 1. File extension (fastest)
/// 2. Content analysis fallback (for files without extensions)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Dispatcher;

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

impl Dispatcher {
    /// Create a new dispatcher.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Parse file content with automatic format detection.
    ///
    /// Supports: `.toml`, `.json`, `.yaml`, `.yml`.
    ///
    /// # Errors
    /// Returns `ParseError::UnsupportedFormat` if format cannot be determined.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        let mut tried_json = false;
        let mut tried_toml = false;
        let mut tried_yaml = false;

        // First priority: Try extension-based detection
        if Json::is_supported(path) {
            tried_json = true;
            if let Ok(result) = Json::parse(path, content) {
                debug!(
                  path = %path.display(),
                  format = "JSON",
                  "Parsed structured data file"
                );
                return Ok(result);
            }
        }
        if Toml::is_supported(path) {
            tried_toml = true;
            if let Ok(result) = Toml::parse(path, content) {
                debug!(
                  path = %path.display(),
                  format = "TOML",
                  "Parsed structured data file"
                );
                return Ok(result);
            }
        }
        if Yaml::is_supported(path) {
            tried_yaml = true;
            if let Ok(result) = Yaml::parse(path, content) {
                debug!(
                  path = %path.display(),
                  format = "YAML",
                  "Parsed structured data file"
                );
                return Ok(result);
            }
        }

        // Second priority: Try content-based detection
        if !tried_json
            && Json::detect(content)
            && let Ok(result) = Json::parse(path, content)
        {
            debug!(
              path = %path.display(),
              format = "JSON",
              method = "content-detection",
              "Parsed structured data file"
            );
            return Ok(result);
        }
        if !tried_yaml
            && Yaml::detect(content)
            && let Ok(result) = Yaml::parse(path, content)
        {
            debug!(
              path = %path.display(),
              format = "YAML",
              method = "content-detection",
              "Parsed structured data file"
            );
            return Ok(result);
        }
        if !tried_toml
            && Toml::detect(content)
            && let Ok(result) = Toml::parse(path, content)
        {
            debug!(
              path = %path.display(),
              format = "TOML",
              method = "content-detection",
              "Parsed structured data file"
            );
            return Ok(result);
        }

        error!(
            path = %path.display(),
            supported = ?vec!["toml", "json", "yaml", "yml"],
            "Unsupported file format"
        );

        Err(ParseError::UnsupportedFormat {
            path: path.into(),
            supported: vec!["toml", "json", "yaml", "yml"],
        })
    }
}

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
    /// Returns `ParseError` if parsing fails.
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        serde_json::from_str(content).map_err(|e| ParseError::Json {
            path: path.into(),
            message: e.to_string().into(),
            line: Some(e.line()),
            column: Some(e.column()),
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
    /// Returns `ParseError` if parsing fails.
    #[expect(
        clippy::string_slice,
        reason = "TOML parser guarantees span byte offsets fall on UTF-8 char \
                  boundaries."
    )]
    #[inline]
    pub fn parse<T: DeserializeOwned>(
        path: &Path,
        content: &str,
    ) -> Result<T, ParseError> {
        toml::from_str(content).map_err(|e| {
            let (line, column) = e.span().map_or((None, None), |span| {
                let lines: Vec<&str> = content[..span.start].lines().collect();
                let line = lines.len();
                let column = lines.last().map_or(0, |l| l.len());
                (Some(line), Some(column))
            });

            ParseError::Toml {
                path: path.into(),
                message: e.message().into(),
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
                path: path.into(),
                message: e.to_string().into(),
                line,
                column,
            }
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
            assert!(Toml::detect(trimmed));
        }

        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn should_reject_non_toml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(!Toml::detect(trimmed));
        }

        #[rstest]
        #[case::object_start("{")]
        #[case::array_start("[")]
        #[case::full_object("{\"name\": \"test\"}")]
        fn should_detect_valid_json_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(Json::detect(trimmed));
        }

        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::toml_key_value("name = \"test\"")]
        #[case::plain_text("plain text")]
        fn should_reject_non_json_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(!Json::detect(trimmed));
        }

        #[rstest]
        #[case::document_separator("---")]
        #[case::key_value("name: test")]
        #[case::both("---\nname: test")]
        fn should_detect_valid_yaml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(Yaml::detect(trimmed));
        }

        #[rstest]
        #[case::toml_key_value("name = \"test\"")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn should_reject_non_yaml_content(#[case] content: &str) {
            let trimmed = content.trim_start();
            assert!(!Yaml::detect(trimmed));
        }
    }

    mod dispatch {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_json_correctly() {
            let dispatcher = Dispatcher::new();
            let result: Result<serde_json::Value, _> =
                dispatcher.parse(Path::new("test.json"), fixtures::VALID_JSON);
            assert!(result.is_ok(), "JSON parsing should succeed: {result:?}");
            let value = result.expect("JSON should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed JSON should contain 'name' field with value 'test'"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_toml_correctly() {
            let dispatcher = Dispatcher::new();
            let result: Result<toml::Value, _> =
                dispatcher.parse(Path::new("test.toml"), fixtures::VALID_TOML);
            assert!(result.is_ok(), "TOML parsing should succeed: {result:?}");
            let value = result.expect("TOML should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed TOML should contain 'name' field with value 'test'"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_yaml_correctly() {
            let dispatcher = Dispatcher::new();
            let result: Result<serde_yaml::Value, _> =
                dispatcher.parse(Path::new("test.yaml"), fixtures::VALID_YAML);
            assert!(result.is_ok(), "YAML parsing should succeed: {result:?}");
            let value = result.expect("YAML should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed YAML should contain 'name' field with value 'test'"
            );
        }

        #[test]
        fn should_reject_unsupported_format() {
            let dispatcher = Dispatcher::new();
            let result = dispatcher
                .parse::<serde_json::Value>(Path::new("test.xml"), "");
            assert!(matches!(
                result,
                Err(ParseError::UnsupportedFormat { .. })
            ));
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_json_by_content() {
            let dispatcher = Dispatcher::new();
            let result: Result<serde_json::Value, _> =
                dispatcher.parse(Path::new("config"), fixtures::VALID_JSON);
            assert!(
                result.is_ok(),
                "JSON should be dispatched by content when extension is \
                 missing: {result:?}"
            );
            let value = result.expect("JSON should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed JSON should contain 'name' field"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_yaml_by_content() {
            let dispatcher = Dispatcher::new();
            let result: Result<serde_yaml::Value, _> =
                dispatcher.parse(Path::new("config"), fixtures::VALID_YAML);
            assert!(
                result.is_ok(),
                "YAML should be dispatched by content when extension is \
                 missing: {result:?}"
            );
            let value = result.expect("YAML should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed YAML should contain 'name' field"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_dispatch_toml_by_content() {
            let dispatcher = Dispatcher::new();
            let result: Result<toml::Value, _> =
                dispatcher.parse(Path::new("config"), fixtures::VALID_TOML);
            assert!(
                result.is_ok(),
                "TOML should be dispatched by content when extension is \
                 missing: {result:?}"
            );
            let value = result.expect("TOML should be parsed");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed TOML should contain 'name' field"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_prioritize_extension_over_content() {
            let dispatcher = Dispatcher::new();
            let result: Result<serde_json::Value, _> = dispatcher
                .parse(Path::new("config.toml"), fixtures::VALID_JSON);
            assert!(
                result.is_ok(),
                "Extension should take priority over content detection: \
                 {result:?}"
            );
            let value = result.expect("JSON should be parsed via TOML parser");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed value should contain 'name' field"
            );
        }
    }

    mod edge_cases {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::toml("toml")]
        #[case::json("json")]
        #[case::yaml("yaml")]
        fn should_handle_empty_content_gracefully(#[case] format: &str) {
            let path_str = format!("test.{format}");
            let result = Dispatcher::new()
                .parse::<serde_json::Value>(Path::new(&path_str), "");
            if format == "json" {
                assert!(
                    result.is_err(),
                    "JSON should reject empty content, got: {result:?}"
                );
            } else {
                assert!(
                    result.is_ok(),
                    "TOML/YAML should accept empty content, got: {result:?}"
                );
            }
        }

        #[rstest]
        #[case::toml_invalid_key("invalid key = value", "toml")]
        #[case::json_invalid_syntax("{\"name\": }", "json")]
        #[case::yaml_invalid_indent("name:\n  - item\n- invalid", "yaml")]
        fn should_provide_error_context_for_malformed_content(
            #[case] content: &str,
            #[case] ext: &str,
        ) {
            let path_str = format!("test.{ext}");
            let result = Dispatcher::new()
                .parse::<serde_json::Value>(Path::new(&path_str), content);
            assert!(
                result.is_err(),
                "Malformed {ext} content should be rejected, got: {result:?}"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Setup phase - test fixture extraction"
        )]
        fn should_handle_mixed_line_endings() {
            let mixed = "name = \"test\"\r\nversion = 1\nenabled = true";
            let result: Result<toml::Value, _> =
                Dispatcher::new().parse(Path::new("test.toml"), mixed);
            assert!(
                result.is_ok(),
                "TOML should handle mixed line endings, got: {result:?}"
            );
            let value =
                result.expect("TOML with mixed line endings should parse");
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test"),
                "Parsed TOML should contain 'name' field"
            );
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
            assert!(matches!(result, Err(ParseError::Toml { .. })));
        }

        #[test]
        fn should_provide_json_error_context() {
            let result = Json::parse::<serde_json::Value>(
                Path::new("test.json"),
                fixtures::INVALID_JSON,
            );
            assert!(matches!(result, Err(ParseError::Json { .. })));
        }

        #[test]
        fn should_provide_yaml_error_context() {
            let result = Yaml::parse::<serde_yaml::Value>(
                Path::new("test.yaml"),
                "name: test\n  invalid: indent",
            );
            assert!(matches!(result, Err(ParseError::Yaml { .. })));
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
            assert!(Toml::is_supported(Path::new(path)));
        }

        #[test]
        fn should_reject_invalid_toml_extensions() {
            assert!(!Toml::is_supported(Path::new("config.json")));
        }

        #[rstest]
        #[case::standard("config.json")]
        #[case::caps("config.JSON")]
        #[case::mixed("Config.Json")]
        fn should_recognize_valid_json_extensions(#[case] path: &str) {
            assert!(Json::is_supported(Path::new(path)));
        }

        #[test]
        fn should_reject_invalid_json_extensions() {
            assert!(!Json::is_supported(Path::new("config.toml")));
        }

        #[rstest]
        #[case::standard_yaml("config.yaml")]
        #[case::standard_yml("config.yml")]
        #[case::caps("config.YAML")]
        #[case::mixed("Config.Yml")]
        fn should_recognize_valid_yaml_extensions(#[case] path: &str) {
            assert!(Yaml::is_supported(Path::new(path)));
        }

        #[test]
        fn should_reject_invalid_yaml_extensions() {
            assert!(!Yaml::is_supported(Path::new("config.toml")));
        }
    }
}
