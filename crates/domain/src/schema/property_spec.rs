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

use crate::{errors::DomainError, validation};

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
    /// Get the spec type identifier.
    #[inline]
    #[must_use]
    pub fn spec_type(&self) -> PropertySpecType {
        match *self {
            Self::Bool(_) => PropertySpecType::Bool,
            Self::Date(_) => PropertySpecType::Date,
            Self::File(_) => PropertySpecType::File,
            Self::Number(_) => PropertySpecType::Number,
            Self::String(_) => PropertySpecType::String,
        }
    }

    /// Validate a value against this spec's constraints.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate Representation (IR)
    /// for metadata values, allowing validation of data loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{PropertySpec, BoolSpec};
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// spec.validate(&serde_json::json!(true)).unwrap();
    /// ```
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: self is &PropertySpec, variants bind implicitly. Consistent with frontmatter pattern."
    )]
    pub fn validate(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        match self {
            Self::Bool(s) => {
                let b = value.as_bool().ok_or_else(|| {
                    DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "boolean".to_owned(),
                    }
                })?;
                s.validate(&b)
            }
            Self::Date(s) => {
                let val =
                    value.as_str().ok_or_else(|| DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string (date)".to_owned(),
                    })?;
                s.validate(&val.to_owned())
            }
            Self::File(s) => {
                let val =
                    value.as_str().ok_or_else(|| DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string (file path)".to_owned(),
                    })?;
                s.validate(&val.to_owned())
            }
            Self::Number(s) => {
                let n =
                    value.as_f64().ok_or_else(|| DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "number".to_owned(),
                    })?;
                s.validate(&n)
            }
            Self::String(s) => {
                let val =
                    value.as_str().ok_or_else(|| DomainError::InvalidType {
                        value: value.to_string(),
                        expected: "string".to_owned(),
                    })?;
                s.validate(&val.to_owned())
            }
        }
    }

    /// Validate the spec's own structural constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if the spec definition is invalid.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{PropertySpec, StringSpec};
    /// let spec = PropertySpec::String(StringSpec::default());
    /// spec.validate_spec().unwrap();
    /// ```
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
    /// Date format string (using chrono format tokens).
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
        // Use chrono format tokens directly
        let is_valid =
            chrono::NaiveDateTime::parse_from_str(value, &self.format).is_ok()
                || chrono::NaiveDate::parse_from_str(value, &self.format)
                    .is_ok();

        if !is_valid {
            return Err(DomainError::InvalidDateFormat(format!(
                "Value {value} does not match format {}",
                self.format
            )));
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
        if let Some(dir) = self.directory.as_ref()
            && !value.starts_with(dir)
        {
            return Err(DomainError::InvalidDirectoryPath(format!(
                "File {value} must be in directory {dir}"
            )));
        }
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        if let Some(fc) = self.file_class.as_ref()
            && fc.is_empty()
        {
            return Err(DomainError::InvalidFileClass(
                "File class cannot be empty".to_owned(),
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
    fn validate(&self, value: &Self::Value) -> Result<(), DomainError> {
        validation::validate_numeric_range(*value, self.min, self.max)?;
        self.validate_step(*value)?;
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        self.check_range()?;
        self.check_step()?;
        Ok(())
    }
}

impl NumberSpec {
    fn check_range(&self) -> Result<(), DomainError> {
        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(DomainError::ValidationFailed(
                "min cannot be greater than max".to_owned(),
            ));
        }
        Ok(())
    }

    fn check_step(&self) -> Result<(), DomainError> {
        if self.step.is_some_and(|step| step <= 0.0f64) {
            return Err(DomainError::ValidationFailed(
                "step must be positive".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_step(&self, value: f64) -> Result<(), DomainError> {
        if let Some(step) = self.step {
            // Note: step positivity is guaranteed by check_step() in validate_spec
            let base = self.min.unwrap_or(0.0f64);
            validation::validate_numeric_step(value, base, step)?;
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
        validation::validate_string_length(
            value,
            self.min_length,
            self.max_length,
        )?;
        self.validate_enum(value)?;
        self.validate_pattern(value)?;
        Ok(())
    }

    #[inline]
    fn validate_spec(&self) -> Result<(), DomainError> {
        self.check_length_range()?;
        self.check_pattern()?;
        Ok(())
    }
}

impl StringSpec {
    fn check_length_range(&self) -> Result<(), DomainError> {
        if let (Some(min), Some(max)) = (self.min_length, self.max_length)
            && min > max
        {
            return Err(DomainError::ValidationFailed(
                "min_length cannot be greater than max_length".to_owned(),
            ));
        }
        Ok(())
    }

    fn check_pattern(&self) -> Result<(), DomainError> {
        if let Some(pattern) = self.pattern.as_ref() {
            get_cached_regex(pattern)?;
        }
        Ok(())
    }

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

    fn validate_pattern(&self, value: &str) -> Result<(), DomainError> {
        if let Some(pattern) = self.pattern.as_ref() {
            // Note: pattern compilation is guaranteed by check_pattern() in validate_spec
            let re = get_cached_regex(pattern)?;
            if !re.is_match(value) {
                return Err(DomainError::ValidationFailed(format!(
                    "Value {value} does not match pattern {pattern}"
                )));
            }
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
    // # LINT_DISABLE_REASON: Standard test utilities and behavioral verification patterns.
    use lithos_test_utils::assert_err_kind;
    use rstest::rstest;

    use super::*;

    /// 3.3-UNIT-011: String Specification Validation Matrix.
    /// Priority: P1.
    #[rstest]
    #[case::enum_match(
        StringSpec { enum_values: Some(vec!["A".to_owned(), "B".to_owned()]), ..Default::default() },
        "A",
        Ok(())
    )]
    #[case::enum_mismatch(
        StringSpec { enum_values: Some(vec!["A".to_owned(), "B".to_owned()]), ..Default::default() },
        "C",
        Err(DomainError::InvalidEnumValue { value: "C".to_owned(), allowed: vec!["A".to_owned(), "B".to_owned()] })
    )]
    #[case::regex_match(
        StringSpec { pattern: Some(r"^\d+$".to_owned()), ..Default::default() },
        "123",
        Ok(())
    )]
    #[case::regex_mismatch(
        StringSpec { pattern: Some(r"^\d+$".to_owned()), ..Default::default() },
        "abc",
        Err(DomainError::ValidationFailed("Value abc does not match pattern ^\\d+$".to_owned()))
    )]
    #[case::length_match(
        StringSpec { min_length: Some(2), max_length: Some(5), ..Default::default() },
        "abc",
        Ok(())
    )]
    #[case::too_short(
        StringSpec { min_length: Some(2), ..Default::default() },
        "a",
        Err(DomainError::StringTooShort { min: 2, actual: 1 })
    )]
    #[case::too_long(
        StringSpec { max_length: Some(5), ..Default::default() },
        "abcdef",
        Err(DomainError::StringTooLong { max: 5, actual: 6 })
    )]
    fn string_spec_validation_matrix(
        #[case] spec: StringSpec,
        #[case] value: &str,
        #[case] expected: Result<(), DomainError>,
    ) {
        // WHEN: validating a string value
        let result = spec.validate(&value.to_owned());

        // THEN: the result matches the expectation
        assert_eq!(result, expected);
    }

    /// 3.3-UNIT-012: Number Specification Validation Matrix.
    /// Priority: P1.
    #[rstest]
    #[case::in_range(NumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
      5.0f64,
      Ok(()))]
    #[case::at_min(NumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
      0.0f64,
      Ok(()))]
    #[case::at_max(NumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
      10.0f64,
      Ok(()))]
    #[case::below_min(NumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
      -1.0f64,
      Err(DomainError::NumberOutOfRange { value: -1.0f64, min: Some(0.0f64), max: Some(10.0f64) }))]
    #[case::above_max(NumberSpec { min: Some(0.0f64), max: Some(10.0f64), step: None },
      11.0f64,
      Err(DomainError::NumberOutOfRange { value: 11.0f64, min: Some(0.0f64), max: Some(10.0f64) }))]
    #[case::valid_step(NumberSpec { min: Some(0.0f64), step: Some(0.5f64), ..Default::default() },
      5.5f64,
      Ok(()))]
    #[case::invalid_step(NumberSpec { min: Some(0.0f64), step: Some(0.5f64), ..Default::default() },
      5.2f64,
      Err(DomainError::InvalidStepValue { value: 5.2f64, step: 0.5f64 }))]
    fn number_spec_validation_matrix(
        #[case] spec: NumberSpec,
        #[case] value: f64,
        #[case] expected: Result<(), DomainError>,
    ) {
        // WHEN: validating a numeric value
        let result = spec.validate(&value);

        // THEN: the result matches the expectation
        assert_eq!(result, expected);
    }

    /// 3.3-UNIT-013: File Specification Validation Matrix.
    /// Priority: P1.
    #[rstest]
    #[case::in_dir("notes/my_note.md", "notes/", Ok(()))]
    #[case::out_dir(
      "other/note.md",
      "notes/",
      Err(DomainError::InvalidDirectoryPath("File other/note.md must be in directory notes/".to_owned()))
    )]
    fn file_spec_validation_matrix(
        #[case] path: &str,
        #[case] dir: &str,
        #[case] expected: Result<(), DomainError>,
    ) {
        // GIVEN: a FileSpec with a directory restriction
        let spec = FileSpec {
            directory: Some(dir.to_owned()),
            file_class: None,
        };

        // WHEN: validating file paths
        let result = spec.validate(&path.to_owned());

        // THEN: the result matches the expectation
        assert_eq!(result, expected);
    }

    #[test]
    fn file_spec_validates_file_class_format() {
        // GIVEN: a valid file_class spec
        let spec = FileSpec {
            directory: None,
            file_class: Some("any-schema-name".to_owned()),
        };
        // THEN: it should be valid
        spec.validate_spec().unwrap();
    }

    #[test]
    fn file_spec_rejects_empty_file_class() {
        // GIVEN: an empty file_class spec
        let invalid_spec = FileSpec {
            directory: None,
            file_class: Some(String::new()),
        };

        // WHEN: validating the spec
        let result = invalid_spec.validate_spec();

        // THEN: it should be invalid
        assert!(result.is_err());
    }

    #[test]
    fn bool_spec_validates_type() {
        // GIVEN: a BoolSpec
        let spec = BoolSpec::default();

        // THEN: it accepts booleans
        spec.validate(&true).unwrap();
        spec.validate(&false).unwrap();
    }

    #[test]
    fn date_spec_validates_iso8601() {
        // GIVEN: a DateSpec with RFC3339-like format
        let spec = DateSpec {
            format: "%Y-%m-%dT%H:%M:%SZ".to_owned(),
        };

        // THEN: it accepts matching strings
        spec.validate(&"2024-01-15T14:30:00Z".to_owned()).unwrap();

        // AND: rejects invalid dates
        let result = spec.validate(&"not-a-date".to_owned());
        assert_err_kind!(result, DomainError::InvalidDateFormat(_));
    }

    #[test]
    fn number_spec_validates_spec_definition() {
        // GIVEN: an invalid NumberSpec (min > max)
        let invalid = NumberSpec {
            min: Some(10.0f64),
            max: Some(5.0f64),
            step: None,
        };

        // THEN: it fails spec validation
        let result = invalid.validate_spec();
        assert_err_kind!(result, DomainError::ValidationFailed(_));

        // AND: valid specs pass
        let valid = NumberSpec {
            min: Some(5.0f64),
            max: Some(10.0f64),
            step: Some(1.0f64),
        };
        valid.validate_spec().unwrap();
    }

    #[test]
    fn property_spec_dispatch_works() {
        // GIVEN: various spec variants
        let b = PropertySpec::Bool(BoolSpec::default());
        let s = PropertySpec::String(StringSpec::default());
        let n = PropertySpec::Number(NumberSpec::default());

        // THEN: spec_type returns correct discriminant
        assert_eq!(b.spec_type(), PropertySpecType::Bool);
        assert_eq!(s.spec_type(), PropertySpecType::String);
        assert_eq!(n.spec_type(), PropertySpecType::Number);

        // AND: validate dispatches to inner spec (tested via successful bool parse)
        b.validate(&serde_json::json!(true)).unwrap();
    }
}
