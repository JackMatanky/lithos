//! Type-safe input specifications.
#![allow(
    missing_docs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};

use crate::bounds::Bounds;

/// Type-safe input specification with validation constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum InputSpec {
    /// Boolean input.
    Boolean {
        /// Default value.
        default: Option<bool>,
    },
    /// Date input.
    Date {
        /// Default value.
        default: Option<Box<str>>,
        /// ISO 8601 format string.
        format: Option<Box<str>>,
    },
    /// File reference input.
    File {
        /// Default value.
        default: Option<Box<str>>,
        /// Allowed file types.
        file_types: Option<Vec<Box<str>>>,
    },
    /// Number input.
    Number {
        /// Default value.
        default: Option<f64>,
        /// Value bounds.
        bounds: Bounds<f64>,
    },
    /// String input.
    String {
        /// Default value.
        default: Option<Box<str>>,
        /// Length bounds.
        length: Bounds<usize>,
        /// Regex pattern.
        pattern: Option<Box<str>>,
    },
}

impl InputSpec {
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

    /// Checks if input has a default value.
    #[inline]
    #[must_use]
    #[expect(
        clippy::match_same_arms,
        reason = "Bindings have different types (Option<f64> vs \
                  Option<Box<str>>)"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Mixed Copy and non-Copy enum fields."
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
        let spec = InputSpec::String {
            default: Some("Title".into()),
            length: Bounds::Unbounded,
            pattern: None,
        };

        assert!(
            spec.has_default(),
            "Input spec should indicate that a default value is set"
        );
    }

    #[test]
    fn get_default_value_returns_configured_value() {
        let spec = InputSpec::String {
            default: Some("Title".into()),
            length: Bounds::Unbounded,
            pattern: None,
        };

        assert_eq!(
            spec.default_value(),
            Some(serde_json::Value::String("Title".to_owned())),
            "Default value should match the configured value"
        );
    }
}
