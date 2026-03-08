//! Property specification variants and validation logic.
//!
//! This module provides validated property type specifications for schema
//! properties. Each spec type enforces domain invariants at construction time
//! and provides validation methods for property values.

#![expect(
    clippy::pub_use,
    clippy::exhaustive_enums,
    reason = "Submodule re-exports required for clean public API. rkyv \
              Archive derive generates exhaustive variants despite \
              #[non_exhaustive]"
)]

#[path = "property_spec/bool.rs"]
mod bool;
#[path = "property_spec/date.rs"]
mod date;
#[path = "property_spec/file.rs"]
mod file;
#[path = "property_spec/number.rs"]
mod number;
#[path = "property_spec/string.rs"]
mod string;

// Re-export public types
use rkyv::{Archive, Deserialize, Serialize};

pub use self::{
    bool::BoolSpec,
    date::DateSpec,
    file::FileSpec,
    number::NumberSpec,
    string::{OptionEntry, StringSpec},
};
use super::error::SchemaError;

/// Validated sum type for all supported property specifications.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::{BoolSpec, PropertySpec};
///
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// match spec {
///     PropertySpec::Bool(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PropertySpec {
    /// Boolean property constraints.
    Bool(BoolSpec),
    /// Date property constraints.
    Date(DateSpec),
    /// File property constraints.
    File(FileSpec),
    /// Number property constraints.
    Number(NumberSpec),
    /// String property constraints.
    String(StringSpec),
}

impl PropertySpec {
    /// Validate a value against this spec's constraints.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate
    /// Representation (IR) for metadata values, allowing validation of data
    /// loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::schema::raw::{RawPropertySpec, RawBoolSpec};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let def = RawPropertySpec::Bool(RawBoolSpec);
    /// let spec = def.try_into_validated()?;
    /// spec.validate(&serde_json::json!(true))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub fn validate(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        match self {
            Self::Bool(_) => {
                if !value.is_boolean() {
                    return Err(Self::invalid_type(value, "boolean"));
                }
                Ok(())
            }
            Self::Date(s) => {
                let val = Self::expect_str(value, "string (date)")?;
                s.validate_str(val)
            }
            Self::File(s) => {
                let val = Self::expect_str(value, "string (file path)")?;
                s.validate_str(val)
            }
            Self::Number(s) => {
                let n = Self::expect_f64(value, "number")?;
                s.validate_value(n)
            }
            Self::String(s) => {
                let val = Self::expect_str(value, "string")?;
                s.validate_str(val)
            }
        }
    }

    #[inline]
    fn invalid_type(
        value: &serde_json::Value,
        expected: &'static str,
    ) -> SchemaError {
        SchemaError::InvalidType {
            value: value.to_string(),
            expected: expected.into(),
        }
    }

    #[inline]
    fn expect_str<'value>(
        value: &'value serde_json::Value,
        expected: &'static str,
    ) -> Result<&'value str, SchemaError> {
        value.as_str().ok_or_else(|| Self::invalid_type(value, expected))
    }

    #[inline]
    fn expect_f64(
        value: &serde_json::Value,
        expected: &'static str,
    ) -> Result<f64, SchemaError> {
        value.as_f64().ok_or_else(|| Self::invalid_type(value, expected))
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::raw::{RawBoolSpec, RawPropertySpec};

    #[test]
    fn validate_dispatches_to_bool_spec() {
        let spec = RawPropertySpec::Bool(RawBoolSpec)
            .try_into_validated()
            .expect("Expected default BoolSpec to validate");
        let result = spec.validate(&serde_json::Value::Bool(true));
        assert!(
            result.is_ok(),
            "Bool validation should succeed, got error: {:?}",
            result.err()
        );
    }
}
