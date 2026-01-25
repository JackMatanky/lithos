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
//! use lithos_adapters::spi::fs::FormatDispatcher; // Re-exported for clarity
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
//! - **Epic 5** (Configuration): Loads `Global`, `Vault` configs via TOML
//! - **Epic 7** (Schema): Loads `RawSchema`, `Property` definitions via
//!   YAML/JSON
//! - **Epic 12** (Templates): Loads `Template` definitions via YAML

use std::path::Path;

use serde::de::DeserializeOwned;
use tracing::{debug, error};

use crate::spi::errors::ParseError;

/// JSON parser strategy.
///
/// # Example
///
/// ```
/// use lithos_adapters::spi::fs::parsers::Json;
///
/// let is_json = Json::detect("{\"name\": \"lithos\"}");
/// assert!(is_json);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Json;

impl Json {
    /// Detect if content looks like JSON format.
    ///
    /// Checks for JSON-specific patterns:
    /// - Starts with `{` (object)
    /// - Starts with `[` (array)
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        trimmed.starts_with('{') || trimmed.starts_with('[')
    }

    /// Check if this parser can handle the given file path by extension.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Json;
    ///
    /// let path = Path::new("config.json");
    /// assert!(Json::is_supported(path));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    }

    /// Parse content string into type T.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Json;
    /// use serde_json::Value;
    ///
    /// let content = r#"{"name": "test"}"#;
    /// let result = Json::parse::<Value>(Path::new("test.json"), content);
    /// assert!(result.is_ok());
    /// ```
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
            path: path.into(),
            message: e.to_string().into(),
            line: Some(e.line()),
            column: Some(e.column()),
        })
    }
}

/// TOML parser strategy.
///
/// # Example
///
/// ```
/// use lithos_adapters::spi::fs::parsers::Toml;
///
/// let is_toml = Toml::detect("name = \"lithos\"");
/// assert!(is_toml);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Toml;

impl Toml {
    /// Detect if content looks like TOML format.
    ///
    /// Checks for TOML-specific patterns:
    /// - Contains `[table]` headers
    /// - Contains `key = value` assignments without colons
    /// - Does not start with JSON markers
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        // TOML: contains [ (table) or = (key-value) without : (YAML has :)
        // Avoid false positives with JSON
        !trimmed.starts_with('{')
            && (trimmed.contains('[')
                || (trimmed.contains('=') && !trimmed.contains(':')))
    }

    /// Check if this parser can handle the given file path by extension.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Toml;
    ///
    /// let path = Path::new("config.toml");
    /// assert!(Toml::is_supported(path));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    }

    /// Parse content string into type T.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Toml;
    /// use toml::Value;
    ///
    /// let content = "name = \"test\"";
    /// let result = Toml::parse::<Value>(Path::new("test.toml"), content);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    #[expect(
        clippy::string_slice,
        reason = "TOML parser guarantees span byte offsets fall on UTF-8 char \
                  boundaries. String slicing is safe here and avoids \
                  allocation overhead for error context extraction."
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
                path: path.into(),
                message: e.message().into(),
                line,
                column,
            }
        })
    }
}

/// YAML parser strategy.
///
/// # Example
///
/// ```
/// use lithos_adapters::spi::fs::parsers::Yaml;
///
/// let is_yaml = Yaml::detect("name: lithos");
/// assert!(is_yaml);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Yaml;

impl Yaml {
    /// Detect if content looks like YAML format.
    ///
    /// Checks for YAML-specific patterns:
    /// - Starts with `---` (document separator)
    /// - Contains `key: value` patterns without `=` (TOML has `=`)
    /// - Does not start with JSON markers
    #[inline]
    #[must_use]
    pub fn detect(content: &str) -> bool {
        let trimmed = content.trim_start();
        // YAML: contains --- (document separator) or key: patterns without =
        // (TOML has =)
        // Avoid false positives with JSON
        !trimmed.starts_with('{')
            && !trimmed.starts_with('[')
            && (trimmed.starts_with("---")
                || (trimmed.contains(':') && !trimmed.contains('=')))
    }

    /// Check if this parser can handle the given file path by extension.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Yaml;
    ///
    /// let path = Path::new("config.yaml");
    /// assert!(Yaml::is_supported(path));
    ///
    /// let path_yml = Path::new("config.yml");
    /// assert!(Yaml::is_supported(path_yml));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_supported(path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
        })
    }

    /// Parse content string into type T.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_adapters::spi::fs::parsers::Yaml;
    /// use serde_yaml::Value;
    ///
    /// let content = "name: test";
    /// let result = Yaml::parse::<Value>(Path::new("test.yaml"), content);
    /// assert!(result.is_ok());
    /// ```
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
                path: path.into(),
                message: e.to_string().into(),
                line,
                column,
            }
        })
    }
}

/// Auto-detecting file parser for structured configuration formats.
///
/// Dispatches to the appropriate parser based on file extension or content
/// analysis. Supports TOML, JSON, and YAML formats.
///
/// # Design Philosophy
///
/// - **Zero-sized type**: No runtime state, pure dispatch logic
/// - **Enum dispatch**: Uses compile-time polymorphism (no vtables)
/// - **Extension first**: Leverages `std::path::Path` for fast extension checks
/// - **Content fallback**: Heuristic detection for extensionless files
///
/// Format detection follows this priority:
/// 1. File extension (fastest)
/// 2. Content analysis fallback (for files without extensions)
///
/// # Future Extensibility
///
/// This type remains `#[non_exhaustive]` to allow future addition of methods
/// for vault file type detection (markdown, images, PDFs) without breaking
/// changes. The zero-sized design means adding methods has no cost.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Dispatcher;

impl Dispatcher {
    /// Create a new dispatcher.
    ///
    /// # Example
    ///
    /// ```
    /// use lithos_adapters::spi::fs::parsers::Dispatcher;
    ///
    /// let dispatcher = Dispatcher::new();
    /// # let _ = dispatcher;
    /// ```
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Parse file content with automatic format detection.
    ///
    /// Format detection uses this priority:
    /// 1. File extension (fastest, most reliable)
    /// 2. Content analysis fallback (for files without extensions)
    ///
    /// Supports:
    /// - `.toml` → TOML
    /// - `.json` → JSON
    /// - `.yaml`, `.yml` → YAML
    /// - Content starting with `{` or `[` → JSON
    /// - Content with `---` or `key:` patterns → YAML
    /// - Content with `[` or `key =` patterns → TOML
    ///
    /// # Errors
    ///
    /// Returns `ParseError::UnsupportedFormat` if format cannot be determined,
    /// or format-specific parse error if deserialization fails.
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

        // Second priority: Try content-based detection (skip already tried
        // formats)
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
            let result = Toml::is_supported(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as TOML", path.display());
        }

        // [4.1-U-01] TOML extension rejection
        #[test]
        fn should_reject_invalid_toml_extensions() {
            // GIVEN a path with a non-TOML extension (json)
            let path = Path::new("config.json");

            // WHEN checking if the Toml parser can handle it
            let result = Toml::is_supported(path);

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
            let result = Json::is_supported(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as JSON", path.display());
        }

        // [4.1-U-02] JSON extension rejection
        #[test]
        fn should_reject_invalid_json_extensions() {
            // GIVEN a path with a non-JSON extension (toml)
            let path = Path::new("config.toml");

            // WHEN checking if the Json parser can handle it
            let result = Json::is_supported(path);

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
            let result = Yaml::is_supported(path);

            // THEN it should return true
            assert!(result, "Should recognize {} as YAML", path.display());
        }

        // [4.1-U-03] YAML extension rejection
        #[test]
        fn should_reject_invalid_yaml_extensions() {
            // GIVEN a path with a non-YAML extension (toml)
            let path = Path::new("config.toml");

            // WHEN checking if the Yaml parser can handle it
            let result = Yaml::is_supported(path);

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
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
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
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }

        // [4.1-U-05] Dispatcher TOML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
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
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }

        // [4.1-U-06] Dispatcher YAML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
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
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
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

        // [4.1-U-08] Dispatcher Content Analysis JSON
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
        )]
        fn should_dispatch_json_by_content() {
            // GIVEN a dispatcher, valid JSON content, and a path without
            // extension
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_JSON;
            let path = Path::new("config"); // No extension

            // WHEN parsing the content via the dispatcher
            let result: Result<serde_json::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the JSON via content analysis
            assert!(result.is_ok(), "Should parse JSON by content");
            let value = result.unwrap();
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }

        // [4.1-U-09] Dispatcher Content Analysis YAML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
        )]
        fn should_dispatch_yaml_by_content() {
            // GIVEN a dispatcher, valid YAML content, and a path without
            // extension
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_YAML;
            let path = Path::new("config"); // No extension

            // WHEN parsing the content via the dispatcher
            let result: Result<serde_yaml::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the YAML via content analysis
            assert!(result.is_ok(), "Should parse YAML by content");
            let value = result.unwrap();
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }

        // [4.1-U-10] Dispatcher Content Analysis TOML
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
        )]
        fn should_dispatch_toml_by_content() {
            // GIVEN a dispatcher, valid TOML content, and a path without
            // extension
            let dispatcher = Dispatcher::new();
            let content = fixtures::VALID_TOML;
            let path = Path::new("config"); // No extension

            // WHEN parsing the content via the dispatcher
            let result: Result<toml::Value, _> =
                dispatcher.parse(path, content);

            // THEN it should successfully parse the TOML via content analysis
            assert!(result.is_ok(), "Should parse TOML by content");
            let value = result.unwrap();
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }

        // [4.1-U-11] Dispatcher Extension Takes Priority
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions intentionally use unwrap() for clear \
                      failure messages. See clippy.toml allow-unwrap-in-tests."
        )]
        fn should_prioritize_extension_over_content() {
            // GIVEN a dispatcher, JSON content but TOML extension
            let dispatcher = Dispatcher::new();
            let json_content = fixtures::VALID_JSON;
            let path = Path::new("config.toml"); // TOML extension

            // WHEN parsing the content
            let result: Result<serde_json::Value, _> =
                dispatcher.parse(path, json_content);

            // THEN it should try TOML first (extension), fail, then try JSON
            // (content), succeed
            assert!(result.is_ok(), "Should fall back to content analysis");
            let value = result.unwrap();
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }
    }

    mod errors {
        use super::*;

        // [4.1-U-08] TOML Error Context
        #[test]
        #[expect(
            clippy::panic,
            reason = "panic!() used to fail test immediately if wrong error \
                      variant received. This is intentional test-only \
                      behavior for explicit failure messaging."
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
            reason = "panic!() used to fail test immediately if wrong error \
                      variant received. This is intentional test-only \
                      behavior for explicit failure messaging."
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

        // [4.1-U-18] YAML Error Context
        #[test]
        #[expect(
            clippy::panic,
            reason = "panic!() used to fail test immediately if wrong error \
                      variant received. This is intentional test-only \
                      behavior for explicit failure messaging."
        )]
        fn should_provide_yaml_error_context() {
            // GIVEN invalid YAML content with invalid indentation
            let invalid_yaml = "name: test\n  invalid: indent";
            let path = Path::new("test.yaml");

            // WHEN attempting to parse it
            let result = Yaml::parse::<serde_yaml::Value>(path, invalid_yaml);

            // THEN it should return a Yaml error with line and column
            // information
            assert!(result.is_err());
            if let Err(ParseError::Yaml {
                message,
                line,
                column,
                ..
            }) = result
            {
                assert!(line.is_some(), "Should provide line number");
                assert!(column.is_some(), "Should provide column number");
                assert!(
                    message.to_lowercase().contains("indent")
                        || message.contains("mapping"),
                    "Error message should indicate indentation or mapping \
                     issue"
                );
            } else {
                panic!("Expected ParseError::Yaml");
            }
        }
    }

    mod detect {
        use rstest::rstest;

        use super::*;

        // [4.1-U-12] TOML Content Detection
        #[rstest]
        #[case::table_header("[package]")]
        #[case::key_value("name = \"test\"")]
        #[case::both("[package]\nname = \"test\"")]
        fn should_detect_valid_toml_content(#[case] content: &str) {
            // GIVEN valid TOML content patterns
            let trimmed = content.trim_start();

            // WHEN checking if Toml can detect it
            let result = Toml::detect(trimmed);

            // THEN it should return true
            assert!(result, "Should detect {content} as TOML");
        }

        // [4.1-U-12] TOML Content Rejection
        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn should_reject_non_toml_content(#[case] content: &str) {
            // GIVEN non-TOML content
            let trimmed = content.trim_start();

            // WHEN checking if Toml can detect it
            let result = Toml::detect(trimmed);

            // THEN it should return false
            assert!(!result, "Should not detect {content} as TOML");
        }

        // [4.1-U-13] JSON Content Detection
        #[rstest]
        #[case::object_start("{")]
        #[case::array_start("[")]
        #[case::full_object("{\"name\": \"test\"}")]
        fn should_detect_valid_json_content(#[case] content: &str) {
            // GIVEN valid JSON content patterns
            let trimmed = content.trim_start();

            // WHEN checking if Json can detect it
            let result = Json::detect(trimmed);

            // THEN it should return true
            assert!(result, "Should detect {content} as JSON");
        }

        // [4.1-U-13] JSON Content Rejection
        #[rstest]
        #[case::yaml_key_value("name: test")]
        #[case::toml_key_value("name = \"test\"")]
        #[case::plain_text("plain text")]
        fn should_reject_non_json_content(#[case] content: &str) {
            // GIVEN non-JSON content
            let trimmed = content.trim_start();

            // WHEN checking if Json can detect it
            let result = Json::detect(trimmed);

            // THEN it should return false
            assert!(!result, "Should not detect {content} as JSON");
        }

        // [4.1-U-14] YAML Content Detection
        #[rstest]
        #[case::document_separator("---")]
        #[case::key_value("name: test")]
        #[case::both("---\nname: test")]
        fn should_detect_valid_yaml_content(#[case] content: &str) {
            // GIVEN valid YAML content patterns
            let trimmed = content.trim_start();

            // WHEN checking if Yaml can detect it
            let result = Yaml::detect(trimmed);

            // THEN it should return true
            assert!(result, "Should detect {content} as YAML");
        }

        // [4.1-U-14] YAML Content Rejection
        #[rstest]
        #[case::toml_key_value("name = \"test\"")]
        #[case::json_object("{\"name\": \"test\"}")]
        #[case::plain_text("plain text")]
        fn should_reject_non_yaml_content(#[case] content: &str) {
            // GIVEN non-YAML content
            let trimmed = content.trim_start();

            // WHEN checking if Yaml can detect it
            let result = Yaml::detect(trimmed);

            // THEN it should return false
            assert!(!result, "Should not detect {content} as YAML");
        }
    }

    mod edge_cases {
        use rstest::rstest;

        use super::*;

        // [4.1-U-15] Empty Content Handling
        #[rstest]
        #[case::toml("toml")]
        #[case::json("json")]
        #[case::yaml("yaml")]
        fn should_handle_empty_content_gracefully(#[case] format: &str) {
            // GIVEN empty content and appropriate path
            let content = "";
            let path_str = format!("test.{format}");
            let path = Path::new(&path_str);

            // WHEN attempting to parse
            let result =
                Dispatcher::new().parse::<serde_json::Value>(path, content);

            // THEN JSON fails, TOML/YAML succeed (empty is valid)
            if format == "json" {
                assert!(
                    result.is_err(),
                    "Empty content should fail to parse as {format}"
                );
            } else {
                assert!(
                    result.is_ok(),
                    "Empty content should succeed to parse as {format}"
                );
            }
        }

        // [4.1-U-16] Malformed Content Error Context
        #[rstest]
        #[case::toml_invalid_key("invalid key = value", "toml")]
        #[case::json_invalid_syntax("{\"name\": }", "json")]
        #[case::yaml_invalid_indent("name:\n  - item\n- invalid", "yaml")]
        fn should_provide_error_context_for_malformed_content(
            #[case] content: &str,
            #[case] ext: &str,
        ) {
            // GIVEN malformed content
            let path_str = format!("test.{ext}");
            let path = Path::new(&path_str);

            // WHEN attempting to parse
            let result =
                Dispatcher::new().parse::<serde_json::Value>(path, content);

            // THEN it should return an error with context
            assert!(result.is_err(), "Malformed {ext} content should fail");
        }

        // [4.1-U-17] Mixed Line Endings
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test unwrap for assertion. See clippy.toml \
                      allow-unwrap-in-tests."
        )]
        fn should_handle_mixed_line_endings() {
            // GIVEN TOML content with mixed line endings (\r\n and \n)
            let mixed_endings =
                "name = \"test\"\r\nversion = 1\nenabled = true";
            let path = Path::new("test.toml");

            // WHEN parsing the content
            let result: Result<toml::Value, _> =
                Dispatcher::new().parse(path, mixed_endings);

            // THEN it should successfully parse despite mixed endings
            assert!(result.is_ok(), "Should handle mixed line endings");
            let value = result.unwrap();
            assert_eq!(
                value.get("name").and_then(|v| v.as_str()),
                Some("test")
            );
        }
    }
}
