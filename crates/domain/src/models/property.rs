//! Property and PropertySpec domain entities.
//!
//! This module defines the Property entity and its type-specific validation
//! specifications (PropertySpec).

#![expect(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming"
)]

/// Common regex patterns for property validation.
pub mod patterns {
    /// Email regex pattern.
    pub const EMAIL: &str = r"^[^@]+@[^@]+\.[^@]+$";
    /// URL regex pattern.
    pub const URL: &str = r"^https?://[^\s/$.?#].[^\s]*$";
}

use std::fmt::Debug;

use crate::errors::DomainError;

/// Reusable property definition with type-specific validation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Property {
    /// Whether property accepts array of values.
    pub array: bool,
    /// Deterministic ID: hash(name + spec content).
    pub id: String,
    /// Property name (e.g., "title", "status").
    pub name: String,
    /// Whether property is required.
    pub required: bool,
    /// Type-specific validation specification.
    pub spec: PropertySpec,
}

impl Property {
    /// Compute deterministic ID from name and spec using Blake3.
    #[inline]
    #[must_use]
    pub fn compute_id(name: &str, spec: &PropertySpec) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        // Use Debug representation for spec content to ensure determinism
        let spec_repr = format!("{spec:?}");
        hasher.update(spec_repr.as_bytes());
        let hash = hasher.finalize();
        let hex = hash.to_hex();
        hex.as_str().chars().take(16).collect()
    }

    /// Create a new property with validation and deterministic ID.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(
        name: String,
        required: bool,
        array: bool,
        spec: PropertySpec,
    ) -> Result<Self, DomainError> {
        let id = Self::compute_id(&name, &spec);
        let property = Self {
            array,
            id,
            name,
            required,
            spec,
        };
        property.validate()?;
        Ok(property)
    }

    /// Validate property structure and constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    ///
    /// # Panics
    /// Panics if internal regex fails to compile (should never happen).
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.is_empty() {
            return Err(DomainError::EmptyPropertyName);
        }
        if self.name.len() > 64 {
            return Err(DomainError::PropertyNameTooLong(self.name.len()));
        }

        // Validate name format: ^[a-z0-9_-]+$
        let re = regex::Regex::new("^[a-z0-9_-]+$")
            .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
        if !re.is_match(&self.name) {
            return Err(DomainError::InvalidPropertyName(self.name.clone()));
        }

        self.spec.validate_spec()?;
        Ok(())
    }
}

/// Core trait for property specifications.
///
/// Provides type-safe validation while maintaining flexibility.
pub trait PropertySpecTrait: Debug + Send + Sync {
    /// The value type this spec validates.
    type Value: Debug + Send + Sync;

    /// Get the spec type identifier.
    fn spec_type(&self) -> PropertySpecType;

    /// Validate a value against this spec's constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError>;

    /// Validate the spec's own structural constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if the spec definition is invalid.
    fn validate_spec(&self) -> Result<(), DomainError>;
}

/// Supported property specification types.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PropertySpecType {
    /// Boolean type.
    Bool,
    /// Date type.
    Date,
    /// File reference type.
    File,
    /// Numeric type.
    Number,
    /// String type.
    String,
}

/// Sum type for all supported property specifications.
///
/// This enum acts as the data container for persistence and equality,
/// while individual variants implement `PropertySpecTrait`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum PropertySpec {
    /// Boolean property (marker type).
    Bool(BoolSpec),
    /// Date property validation constraints.
    Date(DateSpec),
    /// File property validation constraints.
    File(FileSpec),
    /// Number property validation constraints.
    Number(NumberSpec),
    /// String property validation constraints.
    String(StringSpec),
}

impl PropertySpec {
    /// Validate a JSON value against this spec.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: self is &PropertySpec, variants bind implicitly. Consistent with frontmatter pattern."
    )]
    pub fn validate_json(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        match self {
            Self::Bool(s) => {
                if let Some(b) = value.as_bool() {
                    s.validate(&b)
                } else {
                    Err(DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "boolean".to_owned(),
                    })
                }
            }
            Self::Date(s) => {
                if let Some(st) = value.as_str() {
                    s.validate(&st.to_owned())
                } else {
                    Err(DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string (date)".to_owned(),
                    })
                }
            }
            Self::File(s) => {
                if let Some(st) = value.as_str() {
                    s.validate(&st.to_owned())
                } else {
                    Err(DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string (file path)".to_owned(),
                    })
                }
            }
            Self::Number(s) => {
                if let Some(f) = value.as_f64() {
                    s.validate(&f)
                } else {
                    Err(DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "number".to_owned(),
                    })
                }
            }
            Self::String(s) => {
                if let Some(st) = value.as_str() {
                    s.validate(&st.to_owned())
                } else {
                    Err(DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string".to_owned(),
                    })
                }
            }
        }
    }

    /// Validate the spec's own structural constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if the spec definition is invalid.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: self is &PropertySpec, variants bind implicitly. Consistent with frontmatter pattern."
    )]
    pub fn validate_spec(&self) -> Result<(), DomainError> {
        match self {
            Self::Bool(s) => s.validate_spec(),
            Self::Date(s) => s.validate_spec(),
            Self::File(s) => s.validate_spec(),
            Self::Number(s) => s.validate_spec(),
            Self::String(s) => s.validate_spec(),
        }
    }
}

/// Boolean property (marker type).
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct BoolSpec;

impl PropertySpecTrait for BoolSpec {
    type Value = bool;

    #[inline]
    fn spec_type(&self) -> PropertySpecType {
        PropertySpecType::Bool
    }

    #[inline]
    fn validate(&self, _value: &Self::Value) -> Result<(), DomainError> {
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Date property validation constraints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct DateSpec {
    /// Date format string.
    pub format: String,
}

impl PropertySpecTrait for DateSpec {
    type Value = String;

    #[inline]
    fn spec_type(&self) -> PropertySpecType {
        PropertySpecType::Date
    }

    #[inline]
    fn validate(&self, _value: &Self::Value) -> Result<(), DomainError> {
        // Domain only validates string format pattern roughly
        // Actual chrono parsing happens in adapter layer, but we can check if format is non-empty
        if self.format.is_empty() {
            return Err(DomainError::InvalidDateFormat(
                "Format cannot be empty".to_owned(),
            ));
        }
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        if self.format.is_empty() {
            return Err(DomainError::InvalidDateFormat(
                "Format cannot be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

/// File property validation constraints.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct FileSpec {
    /// Optional directory restriction.
    pub directory: Option<String>,
    /// Optional file class restriction.
    pub file_class: Option<String>,
}

impl PropertySpecTrait for FileSpec {
    type Value = String;

    #[inline]
    fn spec_type(&self) -> PropertySpecType {
        PropertySpecType::File
    }

    #[inline]
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        if let Some(class) = self.file_class.as_deref() {
            let valid_classes = ["image", "pdf", "note", "audio", "video"];
            if !valid_classes.contains(&class) {
                return Err(DomainError::InvalidFileClass(class.to_owned()));
            }

            // Rough check for extension based on class
            let ext = value.split('.').next_back().unwrap_or("");
            match class {
                "image" => {
                    let images = ["png", "jpg", "jpeg", "gif", "webp"];
                    if !images.contains(&ext.to_lowercase().as_str()) {
                        return Err(DomainError::InvalidFileClass(format!(
                            "File {value} is not an image"
                        )));
                    }
                }
                "pdf" => {
                    if ext.to_lowercase() != "pdf" {
                        return Err(DomainError::InvalidFileClass(format!(
                            "File {value} is not a pdf"
                        )));
                    }
                }
                "note" => {
                    if ext.to_lowercase() != "md" {
                        return Err(DomainError::InvalidFileClass(format!(
                            "File {value} is not a note (.md)"
                        )));
                    }
                }
                _ => {}
            }
        }

        if self.directory.as_deref().is_some_and(|dir| !value.starts_with(dir))
        {
            return Err(DomainError::InvalidDirectoryPath(format!(
                "File {value} must be in directory {}",
                self.directory.as_deref().unwrap_or_default()
            )));
        }

        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        if self.file_class.as_deref().is_some_and(|class| {
            !["image", "pdf", "note", "audio", "video"].contains(&class)
        }) {
            return Err(DomainError::InvalidFileClass(
                self.file_class.clone().unwrap_or_default(),
            ));
        }
        Ok(())
    }
}

/// Number property validation constraints.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct NumberSpec {
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

impl PropertySpecTrait for NumberSpec {
    type Value = f64;

    #[inline]
    fn spec_type(&self) -> PropertySpecType {
        PropertySpecType::Number
    }

    #[inline]
    #[expect(
        clippy::float_arithmetic,
        clippy::modulo_arithmetic,
        reason = "Core numeric validation logic"
    )]
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        if self.min.is_some_and(|min| *value < min) {
            return Err(DomainError::NumberOutOfRange {
                value: *value,
                min: self.min,
                max: self.max,
            });
        }
        if self.max.is_some_and(|max| *value > max) {
            return Err(DomainError::NumberOutOfRange {
                value: *value,
                min: self.min,
                max: self.max,
            });
        }
        if let Some(step) = self.step {
            if step <= 0.0f64 {
                return Err(DomainError::InvalidStepValue {
                    value: *value,
                    step,
                });
            }
            let base = self.min.unwrap_or(0.0f64);
            let diff = (value - base).abs();
            let rem = diff % step;
            // Use a small epsilon for float comparison
            if rem > 1e-10f64 && (step - rem) > 1e-10f64 {
                return Err(DomainError::InvalidStepValue {
                    value: *value,
                    step,
                });
            }
        }
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        if self.min.is_some_and(|min| self.max.is_some_and(|max| min > max)) {
            return Err(DomainError::ValidationFailed(
                "min cannot be greater than max".to_owned(),
            ));
        }
        if self.step.is_some_and(|step| step <= 0.0f64) {
            return Err(DomainError::ValidationFailed(
                "step must be positive".to_owned(),
            ));
        }
        Ok(())
    }
}

/// String property validation constraints.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct StringSpec {
    /// Optional enum of allowed values.
    pub enum_values: Option<Vec<String>>,
    /// Optional max length.
    pub max_length: Option<usize>,
    /// Optional min length.
    pub min_length: Option<usize>,
    /// Optional regex pattern.
    pub pattern: Option<String>,
}

impl PropertySpecTrait for StringSpec {
    type Value = String;

    #[inline]
    fn spec_type(&self) -> PropertySpecType {
        PropertySpecType::String
    }

    #[inline]
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        if self.min_length.is_some_and(|min| value.len() < min) {
            return Err(DomainError::StringTooShort {
                min: self.min_length.unwrap_or_default(),
                actual: value.len(),
            });
        }
        if self.max_length.is_some_and(|max| value.len() > max) {
            return Err(DomainError::StringTooLong {
                max: self.max_length.unwrap_or_default(),
                actual: value.len(),
            });
        }
        if self.enum_values.as_ref().is_some_and(|enums| !enums.contains(value))
        {
            return Err(DomainError::InvalidEnumValue {
                value: value.clone(),
                allowed: self.enum_values.clone().unwrap_or_default(),
            });
        }
        if let Some(pattern) = self.pattern.as_ref() {
            let re = regex::Regex::new(pattern).map_err(|e| {
                DomainError::InvalidRegex(format!(
                    "Invalid pattern {pattern}: {e}"
                ))
            })?;
            if !re.is_match(value) {
                return Err(DomainError::ValidationFailed(format!(
                    "Value {value} does not match pattern {pattern}"
                )));
            }
        }
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        if self
            .min_length
            .is_some_and(|min| self.max_length.is_some_and(|max| min > max))
        {
            return Err(DomainError::ValidationFailed(
                "min_length cannot be greater than max_length".to_owned(),
            ));
        }
        if let Some(pattern) = self.pattern.as_ref() {
            regex::Regex::new(pattern).map_err(|e| {
                DomainError::InvalidRegex(format!(
                    "Invalid pattern {pattern}: {e}"
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod property {
        use super::*;

        #[test]
        fn id_is_deterministic_using_blake3() {
            let spec = PropertySpec::String(StringSpec::default());
            let id1 = Property::compute_id("title", &spec);
            let id2 = Property::compute_id("title", &spec);
            let id3 = Property::compute_id("other", &spec);

            assert_eq!(id1, id2);
            assert_ne!(id1, id3);
            assert!(!id1.is_empty());
        }

        #[test]
        fn rejects_invalid_property_names() {
            let spec = PropertySpec::String(StringSpec::default());
            let res =
                Property::new("Invalid Name".to_owned(), true, false, spec);
            assert!(matches!(res, Err(DomainError::InvalidPropertyName(_))));
        }

        #[test]
        fn validates_regex_patterns_safely() {
            let spec = PropertySpec::String(StringSpec {
                pattern: Some("[invalid regex".to_owned()),
                ..Default::default()
            });

            let res = Property::new("test".to_owned(), true, false, spec);
            assert!(matches!(res, Err(DomainError::InvalidRegex(_))));
        }
    }

    mod specs {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
        fn string_spec_validates_enums() {
            let spec = StringSpec {
                enum_values: Some(vec!["A".to_owned(), "B".to_owned()]),
                ..Default::default()
            };

            spec.validate(&"A".to_owned()).expect("valid");
            assert!(matches!(
                spec.validate(&"C".to_owned()),
                Err(DomainError::InvalidEnumValue { .. })
            ));
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
        fn number_spec_validates_steps() {
            let spec = NumberSpec {
                min: Some(0.0f64),
                step: Some(2.0f64),
                ..Default::default()
            };

            spec.validate(&2.0f64).expect("valid");
            assert!(
                spec.validate(&3.0f64).is_err(),
                "3.0 is not a multiple of step 2.0 from min 0.0"
            );
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
        fn file_spec_validates_file_classes() {
            let spec = FileSpec {
                file_class: Some("image".to_owned()),
                ..Default::default()
            };

            spec.validate(&"test.png".to_owned()).expect("valid");
            assert!(matches!(
                spec.validate(&"test.txt".to_owned()),
                Err(DomainError::InvalidFileClass(_))
            ));
        }
    }
}
