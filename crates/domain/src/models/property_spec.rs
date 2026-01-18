//! Property specification variants and validation logic.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Spec suffix is descriptive"
)]

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Mutex, OnceLock},
};

use crate::errors::DomainError;

static REGEX_CACHE: OnceLock<Mutex<HashMap<String, regex::Regex>>> =
    OnceLock::new();

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
    fn validate_format_string(&self, value: &str) -> Result<(), DomainError> {
        if self.format.is_empty() {
            return Err(DomainError::InvalidDateFormat(
                "Format cannot be empty".to_owned(),
            ));
        }

        // Map common moment-style tokens to chrono format strings
        let chrono_format = self
            .format
            .replace("YYYY", "%Y")
            .replace("MM", "%m")
            .replace("DD", "%d")
            .replace("HH", "%H")
            .replace("mm", "%M")
            .replace("SS", "%S");

        // Try parsing as DateTime first, then as Date
        let is_valid =
            chrono::NaiveDateTime::parse_from_str(value, &chrono_format)
                .is_ok()
                || chrono::NaiveDate::parse_from_str(value, &chrono_format)
                    .is_ok();

        if !is_valid {
            return Err(DomainError::InvalidDateFormat(format!(
                "Value {value} does not match format {}",
                self.format
            )));
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
            let re = get_cached_regex(pattern)?;
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
            get_cached_regex(pattern)?;
        }
        Ok(())
    }
}

fn get_cached_regex(pattern: &str) -> Result<regex::Regex, DomainError> {
    let cache = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().map_err(|e| {
        DomainError::ValidationFailed(format!("Regex cache poison: {e}"))
    })?;

    if let Some(re) = guard.get(pattern) {
        return Ok(re.clone());
    }

    let re = regex::Regex::new(pattern).map_err(|e| {
        DomainError::InvalidRegex(format!("Invalid pattern {pattern}: {e}"))
    })?;

    guard.insert(pattern.to_owned(), re.clone());
    Ok(re)
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    use super::*;

    /// 3.3-UNIT-011: `string_spec_validates_enums_and_patterns`.
    /// Priority: P1.
    #[test]
    fn string_spec_validates_enums_and_patterns() {
        // GIVEN a StringSpec with enum values
        let spec = StringSpec {
            enum_values: Some(vec!["A".to_owned(), "B".to_owned()]),
            ..Default::default()
        };

        // WHEN validating values
        // THEN it should accept values in the enum
        spec.validate(&"A".to_owned()).unwrap();
        // AND reject values not in the enum
        assert!(spec.validate(&"C".to_owned()).is_err());
    }

    /// 3.3-UNIT-012: `number_spec_validates_min_max_step`.
    /// Priority: P1.
    #[test]
    fn number_spec_validates_min_max_step() {
        // GIVEN a NumberSpec with range and step constraints
        let spec = NumberSpec {
            min: Some(0.0f64),
            max: Some(10.0f64),
            step: Some(0.5f64),
        };

        // WHEN validating numeric values
        // THEN it should accept valid values
        spec.validate(&0.0f64).unwrap();
        spec.validate(&10.0f64).unwrap();
        spec.validate(&5.5f64).unwrap();
        // AND reject values out of range
        assert!(spec.validate(&-1.0f64).is_err());
        assert!(spec.validate(&11.0f64).is_err());
        // AND reject values not matching the step
        assert!(spec.validate(&5.2f64).is_err());
    }

    /// 3.3-UNIT-013: `file_spec_validates_directory`.
    /// Priority: P1.
    #[test]
    fn file_spec_validates_directory() {
        // GIVEN a FileSpec with a directory restriction
        let spec = FileSpec {
            directory: Some("notes/".to_owned()),
            file_class: None,
        };

        // WHEN validating file paths
        // THEN it should accept paths within the directory
        spec.validate(&"notes/my_note.md".to_owned()).unwrap();
        // AND reject paths outside the directory
        assert!(spec.validate(&"other/note.md".to_owned()).is_err());
    }

    /// 3.3-UNIT-014: `file_spec_validates_file_class_format`.
    /// Priority: P2.
    #[test]
    fn file_spec_validates_file_class_format() {
        // GIVEN a valid file_class spec
        let spec = FileSpec {
            directory: None,
            file_class: Some("valid-schema".to_owned()),
        };
        // THEN it should be valid
        spec.validate_spec().unwrap();

        // GIVEN an invalid file_class spec
        let invalid_spec = FileSpec {
            directory: None,
            file_class: Some("Invalid Schema!".to_owned()),
        };
        // THEN it should be invalid
        assert!(invalid_spec.validate_spec().is_err());
    }
}
