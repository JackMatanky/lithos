//! Type-safe variable definitions.
#![allow(
    missing_docs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

/// Type-safe variable definition with validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum VariableDefinition {
    /// Boolean variable.
    Boolean {
        /// Default value.
        default: Option<bool>,
    },
    /// Date variable.
    Date {
        /// Default value.
        default: Option<Box<str>>,
        /// ISO 8601 format string.
        format: Option<Box<str>>,
    },
    /// File reference variable.
    File {
        /// Default value.
        default: Option<Box<str>>,
        /// Allowed file types.
        file_types: Option<Vec<Box<str>>>,
    },
    /// Number variable.
    Number {
        /// Default value.
        default: Option<f64>,
        /// Maximum value.
        max: Option<f64>,
        /// Minimum value.
        min: Option<f64>,
    },
    /// String variable.
    String {
        /// Default value.
        default: Option<Box<str>>,
        /// Maximum length.
        max_length: Option<usize>,
        /// Minimum length.
        min_length: Option<usize>,
        /// Regex pattern.
        pattern: Option<Box<str>>,
    },
}

impl VariableDefinition {
    /// Returns filter names to apply at render time.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on reference to enum variants"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Default behavior for unmatched variants is empty list"
    )]
    pub fn filter_chain(&self) -> Vec<&'static str> {
        match self {
            Self::String {
                pattern: Some(_),
                min_length: Some(_),
                ..
            }
            | Self::String {
                pattern: Some(_),
                max_length: Some(_),
                ..
            } => {
                vec!["validate_pattern", "validate_length"]
            }
            Self::String {
                pattern: Some(_),
                ..
            } => vec!["validate_pattern"],
            Self::String {
                min_length: Some(_),
                ..
            }
            | Self::String {
                max_length: Some(_),
                ..
            } => vec!["validate_length"],
            Self::Number {
                min: Some(_),
                ..
            }
            | Self::Number {
                max: Some(_),
                ..
            } => {
                vec!["validate_range"]
            }
            Self::File {
                file_types: Some(_),
                ..
            } => vec!["validate_file_type"],
            Self::Date {
                format: Some(_),
                ..
            } => vec!["date_format"],
            _ => vec![],
        }
    }

    /// Returns filter arguments as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on reference to enum variants"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Default behavior for unmatched variants is null"
    )]
    pub fn filter_args(&self) -> serde_json::Value {
        match self {
            Self::String {
                min_length,
                max_length,
                pattern,
                ..
            } => {
                let mut map = serde_json::Map::new();
                if let Some(min) = min_length {
                    map.insert("min".to_owned(), (*min).into());
                }
                if let Some(max) = max_length {
                    map.insert("max".to_owned(), (*max).into());
                }
                if let Some(p) = pattern {
                    map.insert("pattern".to_owned(), p.as_ref().into());
                }
                serde_json::Value::Object(map)
            }
            Self::Number {
                min,
                max,
                ..
            } => {
                let mut map = serde_json::Map::new();
                if let Some(min) = min {
                    map.insert("min".to_owned(), (*min).into());
                }
                if let Some(max) = max {
                    map.insert("max".to_owned(), (*max).into());
                }
                serde_json::Value::Object(map)
            }
            Self::File {
                file_types: Some(types),
                ..
            } => {
                let mut map = serde_json::Map::new();
                map.insert(
                    "extensions".to_owned(),
                    serde_json::Value::Array(
                        types
                            .iter()
                            .map(|s| serde_json::Value::String(s.to_string()))
                            .collect(),
                    ),
                );
                serde_json::Value::Object(map)
            }
            Self::Date {
                format: Some(f),
                ..
            } => {
                let mut map = serde_json::Map::new();
                map.insert("format".to_owned(), f.as_ref().into());
                serde_json::Value::Object(map)
            }
            _ => serde_json::Value::Null,
        }
    }

    /// Returns default value as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Mixed Copy and non-Copy enum fields."
    )]
    pub fn default_value(&self) -> Option<serde_json::Value> {
        match self {
            Self::Boolean {
                default,
            } => default.map(serde_json::Value::from),
            Self::Number {
                default,
                ..
            } => default.map(serde_json::Value::from),
            Self::Date {
                default,
                ..
            }
            | Self::File {
                default,
                ..
            }
            | Self::String {
                default,
                ..
            } => default.as_deref().map(serde_json::Value::from),
        }
    }

    /// Checks if variable has a default value.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        clippy::match_same_arms,
        reason = "Enum has mixed Copy and non-Copy fields."
    )]
    pub fn has_default(&self) -> bool {
        match self {
            Self::Boolean {
                default,
            } => default.is_some(),
            Self::Date {
                default,
                ..
            } => default.is_some(),
            Self::File {
                default,
                ..
            } => default.is_some(),
            Self::Number {
                default,
                ..
            } => default.is_some(),
            Self::String {
                default,
                ..
            } => default.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_default_returns_true_when_default_present() {
        let def = VariableDefinition::String {
            default: Some("Title".into()),
            max_length: None,
            min_length: None,
            pattern: None,
        };

        assert!(
            def.has_default(),
            "Variable definition should indicate that a default value is set"
        );
    }

    #[test]
    fn get_default_value_returns_configured_value() {
        let def = VariableDefinition::String {
            default: Some("Title".into()),
            max_length: None,
            min_length: None,
            pattern: None,
        };

        assert_eq!(
            def.default_value(),
            Some(serde_json::Value::String("Title".to_owned())),
            "Default value should match the configured value"
        );
    }

    #[test]
    fn filter_chain_for_string_with_pattern_and_length() {
        let var = VariableDefinition::String {
            default: None,
            min_length: Some(5),
            max_length: Some(10),
            pattern: Some("^[A-Z]".into()),
        };

        let chain = var.filter_chain();
        assert_eq!(chain, vec!["validate_pattern", "validate_length"]);
    }

    #[test]
    fn filter_chain_for_number_with_range() {
        let var = VariableDefinition::Number {
            default: None,
            min: Some(1.0f64),
            max: Some(10.0f64),
        };

        let chain = var.filter_chain();
        assert_eq!(chain, vec!["validate_range"]);
    }

    #[test]
    fn filter_args_for_string() {
        let var = VariableDefinition::String {
            default: None,
            min_length: Some(5),
            max_length: Some(10),
            pattern: Some("^[A-Z]".into()),
        };

        let args = var.filter_args();
        assert_eq!(
            args.get("min").and_then(serde_json::Value::as_u64),
            Some(5)
        );
        assert_eq!(
            args.get("max").and_then(serde_json::Value::as_u64),
            Some(10)
        );
        assert_eq!(
            args.get("pattern").and_then(serde_json::Value::as_str),
            Some("^[A-Z]")
        );
    }
}
