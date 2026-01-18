//! Property and `PropertySpec` domain entities.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Property prefix is descriptive"
)]

use std::fmt::{Debug, Display};

use crate::errors::DomainError;

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$`
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct PropertyName(String);

impl PropertyName {
    /// Get string reference.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a new `PropertyName` with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(name: String) -> Result<Self, DomainError> {
        Self::validate_non_empty(&name)?;
        Self::validate_length(&name)?;
        Self::validate_format(&name)?;
        Ok(Self(name))
    }

    fn validate_format(name: &str) -> Result<(), DomainError> {
        let re = regex::Regex::new("^[a-z0-9_-]+$")
            .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
        if !re.is_match(name) {
            return Err(DomainError::InvalidPropertyName(name.to_owned()));
        }
        Ok(())
    }

    fn validate_length(name: &str) -> Result<(), DomainError> {
        if name.len() > 64 {
            return Err(DomainError::PropertyNameTooLong(name.len()));
        }
        Ok(())
    }

    fn validate_non_empty(name: &str) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptyPropertyName);
        }
        Ok(())
    }
}

impl Display for PropertyName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for PropertyName {
    type Error = DomainError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PropertyName> for String {
    #[inline]
    fn from(val: PropertyName) -> Self {
        val.0
    }
}

impl AsRef<str> for PropertyName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Reusable property definition with type-specific validation.
///
/// This is the resolved entity used in the Domain layer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Property {
    /// Whether property accepts array of values.
    pub array: bool,
    /// Deterministic ID: hash(name + spec content).
    pub id: String,
    /// Property name.
    pub name: PropertyName,
    /// Whether property is required.
    pub required: bool,
    /// Type-specific validation specification.
    pub spec: PropertySpec,
}

impl Property {
    /// Compute deterministic ID from name and spec using Blake3.
    ///
    /// # Errors
    /// Returns `DomainError` if canonicalization fails.
    #[inline]
    pub fn compute_id(
        name: &str,
        spec: &PropertySpec,
    ) -> Result<String, DomainError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        // Use canonical JSON representation for spec content to ensure absolute determinism
        let spec_json = serde_json::to_string(spec).map_err(|e| {
            DomainError::ValidationFailed(format!(
                "Failed to canonicalize spec: {e}"
            ))
        })?;
        hasher.update(spec_json.as_bytes());
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Create a new property with validation.
    ///
    /// # Identity Integrity
    /// The `id` must be provided by the caller but will be validated against the
    /// computed hash of the property's definition (name + spec) using Blake3.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails or the provided ID does not match
    /// the computed definition hash.
    #[inline]
    pub fn new(
        id: String,
        name: PropertyName,
        required: bool,
        array: bool,
        spec: PropertySpec,
    ) -> Result<Self, DomainError> {
        Self::validate_id_integrity(&id, name.as_str(), &spec)?;
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
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        // Name validation is handled by PropertyName type.
        // We only validate the spec constraints here.
        self.spec.validate_spec()?;
        Ok(())
    }

    fn validate_id_integrity(
        id: &str,
        name: &str,
        spec: &PropertySpec,
    ) -> Result<(), DomainError> {
        let computed_id = Self::compute_id(name, spec)?;
        if id != computed_id {
            return Err(DomainError::ValidationFailed(format!(
                "Property ID mismatch for {name}. Expected {computed_id}, got {id}"
            )));
        }
        Ok(())
    }
}

/// Raw property input definition (Inline or Ref).
///
/// Matches the `PropertyOrRef` schema definition used in input files.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming"
)]
pub enum RawProperty {
    /// An inline property definition.
    Inline(RawPropertyInline),
    /// A reference to a property in the `PropertyBank`.
    Ref(RawPropertyRef),
}

/// Reference variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference string (e.g., "#/properties/title").
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

/// Inline variant of a raw property.
///
/// Corresponds to the `Property` definition in the JSON schema but used as input.
/// Differs from `Property` entity by missing the `id` (which is computed later)
/// and using raw types before validation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Property name.
    pub name: String,
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

/// Core trait for property specifications.
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
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        self.validate_format_string(value)
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

impl DateSpec {
    fn validate_format_string(&self, _value: &str) -> Result<(), DomainError> {
        // Here we would check if the value matches the format string.
        // For MVP, we ensure the spec itself is valid.
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
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<String>,
    /// Optional file class restriction (schema name).
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
        self.validate_directory(value)
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        self.validate_file_class_validity()
    }
}

impl FileSpec {
    fn validate_directory(&self, value: &str) -> Result<(), DomainError> {
        if let Some(dir) = self.directory.as_ref()
            && !value.starts_with(dir)
        {
            return Err(DomainError::InvalidDirectoryPath(format!(
                "File {value} must be in directory {dir}"
            )));
        }
        Ok(())
    }

    fn validate_file_class_validity(&self) -> Result<(), DomainError> {
        if let Some(fc) = self.file_class.as_ref() {
            // File class is a schema name reference, so it must be a valid schema name.
            // Using standard schema name regex: ^[a-z0-9]+(-[a-z0-9]+)*$
            let re = regex::Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$")
                .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
            if !re.is_match(fc) {
                return Err(DomainError::InvalidFileClass(fc.clone()));
            }
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
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        self.validate_min(*value)?;
        self.validate_max(*value)?;
        self.validate_step(*value)?;
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        self.validate_range_validity()?;
        self.validate_step_validity()?;
        Ok(())
    }
}

impl NumberSpec {
    fn validate_max(&self, value: f64) -> Result<(), DomainError> {
        if let Some(max) = self.max
            && value > max
        {
            return Err(DomainError::NumberOutOfRange {
                value,
                min: self.min,
                max: self.max,
            });
        }
        Ok(())
    }

    fn validate_min(&self, value: f64) -> Result<(), DomainError> {
        if let Some(min) = self.min
            && value < min
        {
            return Err(DomainError::NumberOutOfRange {
                value,
                min: self.min,
                max: self.max,
            });
        }
        Ok(())
    }

    fn validate_range_validity(&self) -> Result<(), DomainError> {
        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(DomainError::ValidationFailed(
                "min cannot be greater than max".to_owned(),
            ));
        }
        Ok(())
    }

    #[expect(
        clippy::float_arithmetic,
        clippy::modulo_arithmetic,
        reason = "Core numeric validation logic"
    )]
    fn validate_step(&self, value: f64) -> Result<(), DomainError> {
        if let Some(step) = self.step {
            if step <= 0.0f64 {
                return Err(DomainError::InvalidStepValue {
                    value,
                    step,
                });
            }
            let base = self.min.unwrap_or(0.0f64);
            let diff = (value - base).abs();
            let rem = diff % step;
            if rem > 1e-10f64 && (step - rem) > 1e-10f64 {
                return Err(DomainError::InvalidStepValue {
                    value,
                    step,
                });
            }
        }
        Ok(())
    }

    fn validate_step_validity(&self) -> Result<(), DomainError> {
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
        self.validate_min_length(value)?;
        self.validate_max_length(value)?;
        self.validate_enum(value)?;
        self.validate_pattern(value)?;
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        self.validate_length_range_validity()?;
        self.validate_pattern_validity()?;
        Ok(())
    }
}

impl StringSpec {
    fn validate_enum(&self, value: &str) -> Result<(), DomainError> {
        if let Some(enums) = self.enum_values.as_ref()
            && !enums.contains(&value.to_owned())
        {
            return Err(DomainError::InvalidEnumValue {
                value: value.to_owned(),
                allowed: enums.clone(),
            });
        }
        Ok(())
    }

    fn validate_length_range_validity(&self) -> Result<(), DomainError> {
        if let (Some(min), Some(max)) = (self.min_length, self.max_length)
            && min > max
        {
            return Err(DomainError::ValidationFailed(
                "min_length cannot be greater than max_length".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_max_length(&self, value: &str) -> Result<(), DomainError> {
        if let Some(max) = self.max_length
            && value.len() > max
        {
            return Err(DomainError::StringTooLong {
                max,
                actual: value.len(),
            });
        }
        Ok(())
    }

    fn validate_min_length(&self, value: &str) -> Result<(), DomainError> {
        if let Some(min) = self.min_length
            && value.len() < min
        {
            return Err(DomainError::StringTooShort {
                min,
                actual: value.len(),
            });
        }
        Ok(())
    }

    fn validate_pattern(&self, value: &str) -> Result<(), DomainError> {
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

    fn validate_pattern_validity(&self) -> Result<(), DomainError> {
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
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    use super::*;

    mod property {
        use super::*;

        /// 3.3-UNIT-005: `id_is_deterministic_using_blake3_and_canonical_json`.
        #[test]
        fn id_is_deterministic_using_blake3_and_canonical_json() {
            let spec = PropertySpec::String(StringSpec::default());
            let id1 = Property::compute_id("title", &spec).unwrap();
            let id2 = Property::compute_id("title", &spec).unwrap();
            let id3 = Property::compute_id("other", &spec).unwrap();

            assert_eq!(id1, id2, "Identical definitions must produce same ID");
            assert_ne!(id1, id3, "Different names must produce different IDs");
        }

        /// 3.3-UNIT-006: `returns_error_when_property_name_format_is_invalid`.
        #[test]
        fn returns_error_when_property_name_format_is_invalid() {
            let spec = PropertySpec::String(StringSpec::default());
            let invalid_names = vec!["Invalid Name", "invalid.name", ""];
            for name_str in invalid_names {
                // PropertyName::new checks validation
                assert!(
                    PropertyName::new(name_str.to_owned()).is_err(),
                    "Should reject invalid name: {name_str}"
                );
            }
            // Use _spec to suppress warning about unused variable if the loop didn't consume it
            let _: PropertySpec = spec;
        }
    }

    mod property_name {
        use super::*;

        #[test]
        fn validates_regex_and_length() {
            PropertyName::new("valid_name".into()).unwrap();
            PropertyName::new("valid-name-123".into()).unwrap();
            PropertyName::new(String::new()).unwrap_err();
            PropertyName::new("Invalid Name".into()).unwrap_err();
            PropertyName::new("a".repeat(65)).unwrap_err();
        }

        #[test]
        fn validates_format() {
            PropertyName::new("invalid_name!".into()).unwrap_err();
            PropertyName::new("valid_name".into()).unwrap();
        }

        #[test]
        fn validates_length() {
            let long_name = "a".repeat(65);
            assert!(matches!(
                PropertyName::new(long_name),
                Err(DomainError::PropertyNameTooLong(_))
            ));
        }

        #[test]
        fn validates_non_empty() {
            assert!(matches!(
                PropertyName::new(String::new()),
                Err(DomainError::EmptyPropertyName)
            ));
        }
    }

    mod specs {
        use super::*;

        #[test]
        fn string_spec_validates_enums_and_patterns() {
            let spec = StringSpec {
                enum_values: Some(vec!["A".to_owned(), "B".to_owned()]),
                ..Default::default()
            };

            spec.validate(&"A".to_owned()).unwrap();
            assert!(spec.validate(&"C".to_owned()).is_err());
        }

        #[test]
        fn number_spec_validates_min_max_step() {
            let spec = NumberSpec {
                min: Some(0.0f64),
                max: Some(10.0f64),
                step: Some(0.5f64),
            };
            spec.validate(&0.0f64).unwrap();
            spec.validate(&10.0f64).unwrap();
            spec.validate(&5.5f64).unwrap();
            assert!(spec.validate(&-1.0f64).is_err());
            assert!(spec.validate(&11.0f64).is_err());
            assert!(spec.validate(&5.2f64).is_err());
        }

        #[test]
        fn file_spec_validates_directory() {
            let spec = FileSpec {
                directory: Some("notes/".to_owned()),
                file_class: None,
            };
            spec.validate(&"notes/my_note.md".to_owned()).unwrap();
            assert!(spec.validate(&"other/note.md".to_owned()).is_err());
        }

        #[test]
        fn file_spec_validates_file_class_format() {
            let spec = FileSpec {
                directory: None,
                file_class: Some("valid-schema".to_owned()),
            };
            spec.validate_spec().unwrap();

            let invalid_spec = FileSpec {
                directory: None,
                file_class: Some("Invalid Schema!".to_owned()),
            };
            assert!(invalid_spec.validate_spec().is_err());
        }
    }

    mod proptests {
        use proptest::prelude::*;

        use super::super::*;

        proptest! {
            #[test]
            fn validates_property_name_format(name in "[a-z0-9_-]{1,64}") {
                PropertyName::new(name).unwrap();
            }

            #[test]
            fn rejects_invalid_property_name_characters(name in ".*[^a-z0-9_-].*") {
                // Ensure the string isn't empty and length is valid, as those are different errors
                if !name.is_empty() && name.len() <= 64 {
                    PropertyName::new(name).unwrap_err();
                }
            }

            #[test]
            fn compute_id_is_deterministic(name in "[a-z0-9_-]{1,64}") {
                let spec = PropertySpec::Bool(BoolSpec);
                let id1 = Property::compute_id(&name, &spec).unwrap();
                let id2 = Property::compute_id(&name, &spec).unwrap();
                assert_eq!(id1, id2);
            }
        }
    }
}
