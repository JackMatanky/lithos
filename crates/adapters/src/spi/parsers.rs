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

    #[test]
    fn toml_recognizes_toml_extension() {
        assert!(Toml::can_parse(Path::new("config.toml")));
        assert!(Toml::can_parse(Path::new("config.TOML")));
        assert!(!Toml::can_parse(Path::new("config.json")));
    }

    #[test]
    fn json_recognizes_json_extension() {
        assert!(Json::can_parse(Path::new("config.json")));
        assert!(Json::can_parse(Path::new("config.JSON")));
        assert!(!Json::can_parse(Path::new("config.toml")));
    }

    #[test]
    fn yaml_recognizes_yaml_extensions() {
        assert!(Yaml::can_parse(Path::new("config.yaml")));
        assert!(Yaml::can_parse(Path::new("config.yml")));
        assert!(Yaml::can_parse(Path::new("config.YAML")));
        assert!(!Yaml::can_parse(Path::new("config.toml")));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses unwrap for clarity"
    )]
    fn dispatcher_dispatches_to_correct_parser() {
        let dispatcher = Dispatcher::new();

        // JSON
        let json_content = r#"{"name": "test", "value": 42}"#;
        let json_res: Result<serde_json::Value, _> =
            dispatcher.parse(Path::new("test.json"), json_content);
        let _: serde_json::Value = json_res.unwrap();

        // TOML
        let toml_content = "
name = \"test\"
value = 42
";
        let toml_res: Result<toml::Value, _> =
            dispatcher.parse(Path::new("test.toml"), toml_content);
        let _: toml::Value = toml_res.unwrap();

        // YAML
        let yaml_content = "
name: test
value: 42
";
        let yaml_res: Result<serde_yaml::Value, _> =
            dispatcher.parse(Path::new("test.yaml"), yaml_content);
        let _: serde_yaml::Value = yaml_res.unwrap();
    }

    #[test]
    fn dispatcher_rejects_unsupported_format() {
        let dispatcher = Dispatcher::new();
        let result =
            dispatcher.parse::<serde_json::Value>(Path::new("test.xml"), "");

        assert!(matches!(result, Err(ParseError::UnsupportedFormat { .. })));
    }

    #[test]
    fn toml_provides_error_context() {
        let invalid_toml = "
name = 'unclosed string
value = 42
";
        let result =
            Toml::parse::<toml::Value>(Path::new("test.toml"), invalid_toml);

        assert!(result.is_err());
        if let Err(ParseError::Toml {
            message,
            ..
        }) = result
        {
            assert!(message.contains("string"));
        }
    }

    #[test]
    fn json_provides_line_and_column() {
        let invalid_json = r#"
{
  "name": "test",
  "value": 42,
}
"#;
        let result = Json::parse::<serde_json::Value>(
            Path::new("test.json"),
            invalid_json,
        );

        assert!(result.is_err());
        if let Err(ParseError::Json {
            line,
            column,
            ..
        }) = result
        {
            assert!(line.is_some());
            assert!(column.is_some());
        }
    }
}
