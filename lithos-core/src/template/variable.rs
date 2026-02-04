//! Type-safe variable definitions.
#![allow(
    missing_docs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use super::error::TemplateError;
use crate::fs;

/// Type-safe variable definition with validation constraints.
///
/// # Examples
/// ```
/// # use lithos_core::template::variable::VariableDefinition;
/// let definition = VariableDefinition::Number {
///     default: Some(3.0),
///     min: Some(1.0),
///     max: Some(5.0),
/// };
/// assert!(
///     definition.has_default(),
///     "Variable with default should return true"
/// );
/// ```
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
#[expect(
    clippy::unsafe_derive_deserialize,
    reason = "No unsafe code in this enum, false positive"
)]
pub enum VariableDefinition {
    /// Boolean variable.
    Boolean {
        /// Default value.
        default: Option<bool>,
    },
    /// Date variable.
    Date {
        /// Default value.
        default: Option<String>,
        /// ISO 8601 format string.
        format: Option<String>,
    },
    /// File reference variable.
    File {
        /// Default value.
        default: Option<String>,
        /// Allowed file types.
        file_types: Option<Vec<String>>,
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
        default: Option<String>,
        /// Maximum length.
        max_length: Option<usize>,
        /// Minimum length.
        min_length: Option<usize>,
        /// Regex pattern.
        pattern: Option<String>,
    },
}

impl VariableDefinition {
    /// Checks string pattern constraints.
    /// Adversarial Review Fix: Use a cache to avoid recompiling the same regex.
    fn check_string_pattern(
        s: &str,
        pattern: Option<&str>,
    ) -> Result<(), TemplateError> {
        let Some(p) = pattern else {
            return Ok(());
        };

        thread_local! {
            static CACHE:
                std::cell::RefCell<std::collections::HashMap<String, regex::Regex>> =
                std::cell::RefCell::new(std::collections::HashMap::new());
        }

        let is_match = CACHE.with(|cache| -> Result<bool, TemplateError> {
            let mut cache = cache.borrow_mut();
            if let Some(re) = cache.get(p) {
                return Ok(re.is_match(s));
            }

            let re = regex::Regex::new(p)
                .map_err(|e| TemplateError::InvalidRegex(e.to_string()))?;
            let res = re.is_match(s);
            cache.insert(p.to_owned(), re);
            Ok(res)
        })?;

        if !is_match {
            return Err(TemplateError::ValidationFailed(format!(
                "String does not match pattern: {p}"
            )));
        }
        Ok(())
    }

    /// Gets default value as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Mixed Copy and non-Copy enum fields."
    )]
    pub fn get_default_value(&self) -> Option<serde_json::Value> {
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

    #[inline]
    fn validate_boolean(
        value: &serde_json::Value,
    ) -> Result<(), TemplateError> {
        if !value.is_boolean() {
            return Err(TemplateError::InvalidType {
                value: value.to_string(),
                expected: "boolean".to_owned(),
            });
        }
        Ok(())
    }

    #[inline]
    fn validate_date(
        value: &serde_json::Value,
        format: Option<&str>,
    ) -> Result<(), TemplateError> {
        let s = value.as_str().ok_or_else(|| TemplateError::InvalidType {
            value: value.to_string(),
            expected: "string (date)".to_owned(),
        })?;

        if let Some(fmt) = format {
            chrono::NaiveDate::parse_from_str(s, fmt)
                .map_err(|e| TemplateError::InvalidDateFormat(e.to_string()))?;
        } else {
            s.parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|e| TemplateError::InvalidDateFormat(e.to_string()))?;
        }
        Ok(())
    }

    /// Validates a value against this definition.
    ///
    /// # Errors
    /// Returns `TemplateError::InvalidType` if value type doesn't match.
    /// Returns `TemplateError::ValidationFailed` for constraint violations.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::template::variable::VariableDefinition;
    /// let definition = VariableDefinition::String {
    ///     default: None,
    ///     min_length: Some(1),
    ///     max_length: Some(5),
    ///     pattern: None,
    /// };
    /// definition.validate_value(&serde_json::json!("note")).unwrap();
    /// ```
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Enum variants have mixed Copy and non-Copy fields."
    )]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), TemplateError> {
        match self {
            Self::Boolean {
                ..
            } => Self::validate_boolean(value),
            Self::Date {
                format,
                ..
            } => Self::validate_date(value, format.as_deref()),
            Self::File {
                file_types,
                ..
            } => Self::validate_file(value, file_types.as_deref()),
            Self::Number {
                min,
                max,
                ..
            } => Self::validate_number(value, *min, *max),
            Self::String {
                min_length,
                max_length,
                pattern,
                ..
            } => Self::validate_string(
                value,
                *min_length,
                *max_length,
                pattern.as_deref(),
            ),
        }
    }

    fn validate_file_extension_allowed(
        s: &str,
        allowed_types: Option<&[String]>,
    ) -> Result<(), TemplateError> {
        if let Some(allowed) = allowed_types {
            let ext = std::path::Path::new(s)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !allowed.iter().any(|a| a == ext) {
                return Err(TemplateError::InvalidFileClass(s.to_owned()));
            }
        }
        Ok(())
    }

    #[inline]
    fn validate_file(
        value: &serde_json::Value,
        file_types: Option<&[String]>,
    ) -> Result<(), TemplateError> {
        let s = value.as_str().ok_or_else(|| TemplateError::InvalidType {
            value: value.to_string(),
            expected: "string (file path)".to_owned(),
        })?;

        fs::validate_vault_path(s, None)
            .map_err(TemplateError::ValidationFailed)?;
        Self::validate_file_extension_allowed(s, file_types)?;
        Ok(())
    }

    #[inline]
    fn validate_number(
        value: &serde_json::Value,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), TemplateError> {
        let n = value.as_f64().ok_or_else(|| TemplateError::InvalidType {
            value: value.to_string(),
            expected: "number".to_owned(),
        })?;

        if !n.is_finite() {
            return Err(TemplateError::ValidationFailed(format!(
                "Value {n} is not finite"
            )));
        }
        for (field, v) in [("min", min), ("max", max)] {
            if v.is_some_and(|v| !v.is_finite()) {
                return Err(TemplateError::ValidationFailed(format!(
                    "{field} must be finite"
                )));
            }
        }

        if let (Some(min), Some(max)) = (min, max)
            && min > max
        {
            return Err(TemplateError::ValidationFailed(
                "min cannot be greater than max".to_owned(),
            ));
        }

        if let Some(min) = min
            && n < min
        {
            return Err(TemplateError::ValidationFailed(format!(
                "Value {n} is below min {min}"
            )));
        }
        if let Some(max) = max
            && n > max
        {
            return Err(TemplateError::ValidationFailed(format!(
                "Value {n} is above max {max}"
            )));
        }

        Ok(())
    }

    #[inline]
    fn validate_string(
        value: &serde_json::Value,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<&str>,
    ) -> Result<(), TemplateError> {
        let s = value.as_str().ok_or_else(|| TemplateError::InvalidType {
            value: value.to_string(),
            expected: "string".to_owned(),
        })?;

        if let (Some(min), Some(max)) = (min_length, max_length)
            && min > max
        {
            return Err(TemplateError::ValidationFailed(
                "min_length cannot be greater than max_length".to_owned(),
            ));
        }

        // NOTE: Length is in UTF-8 bytes (value.len()), matching the schema
        // semantics.
        let len = s.len();
        if let Some(min) = min_length
            && len < min
        {
            return Err(TemplateError::ValidationFailed(format!(
                "String too short: min {min}, got {len}"
            )));
        }
        if let Some(max) = max_length
            && len > max
        {
            return Err(TemplateError::ValidationFailed(format!(
                "String too long: max {max}, got {len}"
            )));
        }

        Self::check_string_pattern(s, pattern)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_value(value: &str) -> serde_json::Value {
        serde_json::Value::String(value.to_owned())
    }

    fn f64_value(value: f64) -> serde_json::Value {
        let num_opt = serde_json::Number::from_f64(value);
        assert!(
            num_opt.is_some(),
            "Expected finite f64 for JSON number, got: {value}"
        );
        let Some(num) = num_opt else {
            return serde_json::Value::Null;
        };
        serde_json::Value::Number(num)
    }

    #[test]
    fn should_validate_boolean_constraints() {
        // GIVEN: a boolean variable definition
        let def = VariableDefinition::Boolean {
            default: None,
        };

        // WHEN: validating boolean and non-boolean values
        let valid_result = def.validate_value(&serde_json::Value::Bool(true));
        let invalid_result = def.validate_value(&str_value("true"));

        // THEN: only the boolean value is accepted
        assert!(
            valid_result.is_ok(),
            "Boolean value should be accepted, but got: {valid_result:?}"
        );
        assert!(
            invalid_result.is_err(),
            "String value should be rejected for boolean field, got: \
             {invalid_result:?}"
        );
    }

    #[test]
    fn should_validate_number_constraints() {
        // GIVEN: a numeric variable definition with range constraints
        let def = VariableDefinition::Number {
            default: None,
            max: Some(10.0f64),
            min: Some(1.0f64),
        };

        // WHEN: validating values within and outside the range
        let valid_result = def.validate_value(&f64_value(5.0f64));
        let too_low = def.validate_value(&f64_value(0.5f64));
        let too_high = def.validate_value(&f64_value(10.5f64));

        // THEN: range constraints are enforced
        assert!(
            valid_result.is_ok(),
            "Number in range should be accepted, but got: {valid_result:?}"
        );
        assert!(
            too_low.is_err(),
            "Value below min should be rejected, got: {too_low:?}"
        );
        assert!(
            too_high.is_err(),
            "Value above max should be rejected, got: {too_high:?}"
        );
    }

    #[test]
    fn accessors_expose_defaults() {
        // GIVEN: a variable definition with a default value
        let def = VariableDefinition::String {
            default: Some("Title".to_owned()),
            max_length: None,
            min_length: None,
            pattern: None,
        };

        // WHEN: checking for defaults
        let has_default = def.has_default();
        let default_val = def.get_default_value();

        // THEN: the default value is correctly exposed
        assert!(
            has_default,
            "Variable definition should indicate that a default value is set"
        );
        assert_eq!(
            default_val,
            Some(str_value("Title")),
            "Default value should match the configured value"
        );
    }

    #[test]
    fn should_validate_date_variable() {
        // GIVEN: a date variable definition
        let def = VariableDefinition::Date {
            default: None,
            format: None,
        };

        // THEN: it validates ISO8601 strings
        let valid = def.validate_value(&str_value("2024-01-15T10:00:00Z"));
        assert!(
            valid.is_ok(),
            "Valid ISO8601 date should be accepted, got: {valid:?}"
        );

        let invalid = def.validate_value(&str_value("invalid"));
        assert!(
            invalid.is_err(),
            "Invalid date string should be rejected, got: {invalid:?}"
        );

        // AND: handles custom formats
        let def_fmt = VariableDefinition::Date {
            default: None,
            format: Some("%Y-%m-%d".to_owned()),
        };
        let valid_fmt = def_fmt.validate_value(&str_value("2024-01-15"));
        assert!(
            valid_fmt.is_ok(),
            "Valid custom format date should be accepted, got: {valid_fmt:?}"
        );

        let invalid_fmt = def_fmt.validate_value(&str_value("2024/01/15"));
        assert!(
            invalid_fmt.is_err(),
            "Date not matching custom format should be rejected, got: \
             {invalid_fmt:?}"
        );
    }

    #[test]
    fn should_validate_file_variable() {
        // GIVEN: a file variable definition
        let def = VariableDefinition::File {
            default: None,
            file_types: Some(vec!["md".to_owned()]),
        };

        // THEN: it validates vault-relative paths and extensions
        let valid = def.validate_value(&str_value("note.md"));
        assert!(
            valid.is_ok(),
            "Valid file with correct extension should be accepted, got: \
             {valid:?}"
        );

        let wrong_ext = def.validate_value(&str_value("img.png"));
        assert!(
            wrong_ext.is_err(),
            "File with wrong extension should be rejected, got: {wrong_ext:?}"
        );

        let abs_path = def.validate_value(&str_value("/abs/path"));
        assert!(
            abs_path.is_err(),
            "Absolute path should be rejected, got: {abs_path:?}"
        );

        // AND: handles no type restriction
        let def_any = VariableDefinition::File {
            default: None,
            file_types: None,
        };
        let any_file = def_any.validate_value(&str_value("any.file"));
        assert!(
            any_file.is_ok(),
            "File with no type restriction should accept any extension, got: \
             {any_file:?}"
        );
    }

    #[test]
    fn should_validate_file_variable_extensions() {
        // GIVEN: a file variable with multiple allowed types
        let def = VariableDefinition::File {
            default: None,
            file_types: Some(vec!["md".to_owned(), "txt".to_owned()]),
        };

        // THEN: it accepts matching extensions
        let md_file = def.validate_value(&str_value("test.md"));
        assert!(
            md_file.is_ok(),
            "File with .md extension should be accepted, got: {md_file:?}"
        );

        let txt_file = def.validate_value(&str_value("test.txt"));
        assert!(
            txt_file.is_ok(),
            "File with .txt extension should be accepted, got: {txt_file:?}"
        );

        // AND: rejects others
        let png_file = def.validate_value(&str_value("test.png"));
        assert!(
            png_file.is_err(),
            "File with .png extension should be rejected, got: {png_file:?}"
        );
    }

    #[test]
    fn should_validate_string_variable_constraints() {
        // GIVEN: a string variable with length constraints
        let def = VariableDefinition::String {
            default: None,
            max_length: Some(5),
            min_length: Some(2),
            pattern: None,
        };

        // THEN: it enforces length limits
        let valid = def.validate_value(&str_value("abc"));
        assert!(
            valid.is_ok(),
            "String within length constraints should be accepted, got: \
             {valid:?}"
        );

        let too_short = def.validate_value(&str_value("a"));
        assert!(
            too_short.is_err(),
            "String below min length should be rejected, got: {too_short:?}"
        );

        let too_long = def.validate_value(&str_value("abcdef"));
        assert!(
            too_long.is_err(),
            "String above max length should be rejected, got: {too_long:?}"
        );

        // AND: handles min/max independently
        let def_min = VariableDefinition::String {
            default: None,
            min_length: Some(3),
            max_length: None,
            pattern: None,
        };
        let below_min = def_min.validate_value(&str_value("ab"));
        assert!(
            below_min.is_err(),
            "String below min_length should be rejected, got: {below_min:?}"
        );

        let def_max = VariableDefinition::String {
            default: None,
            min_length: None,
            max_length: Some(3),
            pattern: None,
        };
        let above_max = def_max.validate_value(&str_value("abcd"));
        assert!(
            above_max.is_err(),
            "String above max_length should be rejected, got: {above_max:?}"
        );
    }

    #[test]
    fn should_validate_string_pattern_with_caching() {
        // GIVEN: a string variable with a pattern
        let def = VariableDefinition::String {
            default: None,
            max_length: None,
            min_length: None,
            pattern: Some(r"^\d+$".to_owned()),
        };

        // THEN: it validates matching strings (multiple times to trigger cache)
        let match1 = def.validate_value(&str_value("123"));
        assert!(
            match1.is_ok(),
            "String matching pattern should be accepted, got: {match1:?}"
        );

        let match2 = def.validate_value(&str_value("456"));
        assert!(
            match2.is_ok(),
            "Second matching string should be accepted (tests cache), got: \
             {match2:?}"
        );

        // AND: rejects non-matching strings
        let no_match = def.validate_value(&str_value("abc"));
        assert!(
            no_match.is_err(),
            "String not matching pattern should be rejected, got: {no_match:?}"
        );
    }
}
