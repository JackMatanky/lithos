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
// Legacy re-export for backward compatibility
#[expect(
    deprecated,
    reason = "Re-exporting deprecated type for compatibility"
)]
pub use string::StringFormat;

pub use self::{
    bool::BoolSpec,
    date::DateSpec,
    file::FileSpec,
    number::NumberSpec,
    string::{OptionEntry, StringPattern, StringSpec},
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
    /// # use lithos_core::schema::{raw::{RawPropertySpec, RawBoolSpec}, property_spec::PropertySpec};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let def = RawPropertySpec::Bool(RawBoolSpec);
    /// let spec = PropertySpec::try_from(def)?;
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
                s.validate(val)
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

// ============================================================================
// Conversions from Raw Types (Syntax → Domain)
// ============================================================================

impl TryFrom<crate::schema::raw::RawPropertySpec> for PropertySpec {
    type Error = SchemaError;

    /// Convert raw property spec (syntax layer) to validated domain spec.
    ///
    /// This is the correct direction of dependency: `property_spec` depends on
    /// `raw`, not the other way around.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails (invalid pattern, missing
    /// required fields, etc.).
    #[inline]
    fn try_from(
        raw: crate::schema::raw::RawPropertySpec,
    ) -> Result<Self, Self::Error> {
        use crate::schema::raw::RawPropertySpec;

        match raw {
            RawPropertySpec::Bool(_) => Ok(Self::Bool(BoolSpec::default())),
            RawPropertySpec::Date(def) => {
                let format = def.format.ok_or_else(|| {
                    SchemaError::ValidationFailed(
                        "date format is required".into(),
                    )
                })?;
                Ok(Self::Date(DateSpec::try_new(&format)?))
            }
            RawPropertySpec::File(def) => Ok(Self::File(FileSpec::try_new(
                def.directory.as_deref(),
                def.file_class.as_deref(),
            )?)),
            RawPropertySpec::Number(def) => Ok(Self::Number(
                NumberSpec::try_new(def.min, def.max, def.step)?,
            )),
            RawPropertySpec::String(def) => {
                use crate::schema::raw::RawStringPattern;

                // Convert pattern
                let pattern = match def.pattern {
                    Some(RawStringPattern::Custom(p)) => {
                        Some(StringPattern::try_custom(p)?)
                    }
                    Some(RawStringPattern::Named(f)) => {
                        Some(StringPattern::from(f))
                    }
                    None => None,
                };
                Ok(Self::String(StringSpec::try_new(
                    pattern,
                    def.options
                        .map(|o| {
                            o.into_entries()
                                .into_iter()
                                .map(TryInto::try_into)
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .transpose()?,
                )?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PropertySpec;
    use crate::schema::raw::{RawBoolSpec, RawPropertySpec};

    #[test]
    fn validate_dispatches_to_bool_spec() {
        let spec = PropertySpec::try_from(RawPropertySpec::Bool(RawBoolSpec))
            .expect("Expected default BoolSpec to validate");
        let result = spec.validate(&serde_json::Value::Bool(true));
        assert!(
            result.is_ok(),
            "Bool validation should succeed, got error: {:?}",
            result.err()
        );
    }
}
