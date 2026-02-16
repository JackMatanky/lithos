use crate::{errors::DomainError, validation};

/// Type-safe variable definition with validation constraints.
///
/// # Examples
/// ```
/// # use lithos_domain::InputSpec;
/// let definition = InputSpec::Number {
///     default: Some(3.0),
///     min: Some(1.0),
///     max: Some(5.0),
/// };
/// assert!(definition.has_default());
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::unsafe_derive_deserialize,
    reason = "No unsafe code in this enum, false positive"
)]
pub enum InputSpec {
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

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Function ordering optimized for logical flow over strict \
              alphabetical order"
)]
impl InputSpec {
    /// Checks string pattern constraints.
    /// Adversarial Review Fix: Use a cache to avoid recompiling the same regex.
    fn check_string_pattern(
        s: &str,
        pattern: Option<&str>,
    ) -> Result<(), DomainError> {
        let Some(p) = pattern else {
            return Ok(());
        };

        thread_local! {
            static CACHE:
                std::cell::RefCell<std::collections::HashMap<String, regex::Regex>> =
                std::cell::RefCell::new(std::collections::HashMap::new());
        }

        let is_match = CACHE.with(|cache| -> Result<bool, DomainError> {
            let mut cache = cache.borrow_mut();
            if let Some(re) = cache.get(p) {
                return Ok(re.is_match(s));
            }

            let re = regex::Regex::new(p)
                .map_err(|e| DomainError::InvalidRegex(e.to_string()))?;
            let res = re.is_match(s);
            cache.insert(p.to_owned(), re);
            Ok(res)
        })?;

        if !is_match {
            return Err(DomainError::ValidationFailed(format!(
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
        reason = "Mixed Copy (Boolean, Number) and non-Copy (String, Date, \
                  File) enum fields. Cannot use `match *self` without moving \
                  String/Date/File. Current pattern with `match self` is \
                  idiomatic for enums with mixed field types."
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
        reason = "Enum has mixed Copy (Boolean/Number: \
                  Option<bool>/Option<f64>) and non-Copy (String/Date/File: \
                  Option<String>) fields, requiring pattern matching on \
                  &self. All arms return is_some() but cannot be \
                  consolidated—each operates on different Option<T> types and \
                  dereferencing would move non-Copy String fields."
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
    fn validate_boolean(value: &serde_json::Value) -> Result<(), DomainError> {
        if !value.is_boolean() {
            return Err(DomainError::InvalidType {
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
    ) -> Result<(), DomainError> {
        let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "string (date)".to_owned(),
        })?;

        if let Some(fmt) = format {
            chrono::NaiveDate::parse_from_str(s, fmt)
                .map_err(|e| DomainError::InvalidDateFormat(e.to_string()))?;
        } else {
            s.parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|e| DomainError::InvalidDateFormat(e.to_string()))?;
        }
        Ok(())
    }

    /// Validates a value against this definition.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidType` if value type doesn't match.
    /// Returns `DomainError::ValidationFailed` for constraint violations.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::InputSpec;
    /// let definition = InputSpec::String {
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
        reason = "Enum variants have mixed Copy fields (min/max: Option<f64>) \
                  and non-Copy fields (format/pattern: Option<String>). \
                  Cannot dereference `self` without moving non-Copy String \
                  fields. Matching on `&self` with field binding is idiomatic."
    )]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
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
    ) -> Result<(), DomainError> {
        if let Some(allowed) = allowed_types {
            let ext = std::path::Path::new(s)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !allowed.iter().any(|a| a == ext) {
                return Err(DomainError::InvalidFileClass(s.to_owned()));
            }
        }
        Ok(())
    }

    #[inline]
    fn validate_file(
        value: &serde_json::Value,
        file_types: Option<&[String]>,
    ) -> Result<(), DomainError> {
        let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "string (file path)".to_owned(),
        })?;

        validation::validate_vault_path(s, None)?;
        Self::validate_file_extension_allowed(s, file_types)?;
        Ok(())
    }

    #[inline]
    fn validate_number(
        value: &serde_json::Value,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), DomainError> {
        let n = value.as_f64().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "number".to_owned(),
        })?;
        validation::validate_numeric_range(n, min, max)
    }

    #[inline]
    fn validate_string(
        value: &serde_json::Value,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<&str>,
    ) -> Result<(), DomainError> {
        let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "string".to_owned(),
        })?;
        validation::validate_string_length(s, min_length, max_length)?;
        Self::check_string_pattern(s, pattern)?;
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::expect() for deterministic setup and \
              validation of variable definitions. Unreachable paths are \
              acceptable in domain unit tests."
)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_boolean_constraints() {
        // GIVEN: a boolean variable definition
        let def = InputSpec::Boolean {
            default: None,
        };

        // WHEN: validating boolean and non-boolean values
        let valid_result = def.validate_value(&serde_json::json!(true));
        let invalid_result = def.validate_value(&serde_json::json!("true"));

        // THEN: only the boolean value is accepted
        assert!(
            valid_result.is_ok(),
            "Boolean value should be accepted, but got: {:?}",
            valid_result.err()
        );
        assert!(invalid_result.is_err());
    }

    #[test]
    fn should_validate_number_constraints() {
        // GIVEN: a numeric variable definition with range constraints
        let def = InputSpec::Number {
            default: None,
            max: Some(10.0f64),
            min: Some(1.0f64),
        };

        // WHEN: validating values within and outside the range
        let valid_result = def.validate_value(&serde_json::json!(5.0f64));
        let too_low = def.validate_value(&serde_json::json!(0.5f64));
        let too_high = def.validate_value(&serde_json::json!(10.5f64));

        // THEN: range constraints are enforced
        assert!(
            valid_result.is_ok(),
            "Number in range should be accepted, but got: {:?}",
            valid_result.err()
        );
        assert!(too_low.is_err());
        assert!(too_high.is_err());
    }

    #[test]
    fn accessors_expose_defaults() {
        // GIVEN: a variable definition with a default value
        let def = InputSpec::String {
            default: Some("Title".to_owned()),
            max_length: None,
            min_length: None,
            pattern: None,
        };

        // WHEN: checking for defaults
        let has_default = def.has_default();
        let default_val = def.get_default_value();

        // THEN: the default value is correctly exposed
        assert!(has_default);
        assert_eq!(default_val, Some(serde_json::json!("Title")));
    }

    #[test]
    fn should_validate_date_variable() {
        // GIVEN: a date variable definition
        let def = InputSpec::Date {
            default: None,
            format: None,
        };

        // THEN: it validates ISO8601 strings
        def.validate_value(&serde_json::json!("2024-01-15T10:00:00Z")).unwrap();
        assert!(def.validate_value(&serde_json::json!("invalid")).is_err());

        // AND: handles custom formats
        let def_fmt = InputSpec::Date {
            default: None,
            format: Some("%Y-%m-%d".to_owned()),
        };
        def_fmt.validate_value(&serde_json::json!("2024-01-15")).unwrap();
        assert!(
            def_fmt.validate_value(&serde_json::json!("2024/01/15")).is_err()
        );
    }

    #[test]
    fn should_validate_file_variable() {
        // GIVEN: a file variable definition
        let def = InputSpec::File {
            default: None,
            file_types: Some(vec!["md".to_owned()]),
        };

        // THEN: it validates vault-relative paths and extensions
        def.validate_value(&serde_json::json!("note.md")).unwrap();
        assert!(def.validate_value(&serde_json::json!("img.png")).is_err());
        assert!(def.validate_value(&serde_json::json!("/abs/path")).is_err());

        // AND: handles no type restriction
        let def_any = InputSpec::File {
            default: None,
            file_types: None,
        };
        def_any.validate_value(&serde_json::json!("any.file")).unwrap();
    }

    #[test]
    fn should_validate_file_variable_extensions() {
        // GIVEN: a file variable with multiple allowed types
        let def = InputSpec::File {
            default: None,
            file_types: Some(vec!["md".to_owned(), "txt".to_owned()]),
        };

        // THEN: it accepts matching extensions
        def.validate_value(&serde_json::json!("test.md")).unwrap();
        def.validate_value(&serde_json::json!("test.txt")).unwrap();

        // AND: rejects others
        assert!(def.validate_value(&serde_json::json!("test.png")).is_err());
    }

    #[test]
    fn should_validate_string_variable_constraints() {
        // GIVEN: a string variable with length constraints
        let def = InputSpec::String {
            default: None,
            max_length: Some(5),
            min_length: Some(2),
            pattern: None,
        };

        // THEN: it enforces length limits
        def.validate_value(&serde_json::json!("abc")).unwrap();
        assert!(def.validate_value(&serde_json::json!("a")).is_err());
        assert!(def.validate_value(&serde_json::json!("abcdef")).is_err());

        // AND: handles min/max independently
        let def_min = InputSpec::String {
            default: None,
            min_length: Some(3),
            max_length: None,
            pattern: None,
        };
        assert!(def_min.validate_value(&serde_json::json!("ab")).is_err());

        let def_max = InputSpec::String {
            default: None,
            min_length: None,
            max_length: Some(3),
            pattern: None,
        };
        assert!(def_max.validate_value(&serde_json::json!("abcd")).is_err());
    }

    #[test]
    fn should_validate_string_pattern_with_caching() {
        // GIVEN: a string variable with a pattern
        let def = InputSpec::String {
            default: None,
            max_length: None,
            min_length: None,
            pattern: Some(r"^\d+$".to_owned()),
        };

        // THEN: it validates matching strings (multiple times to trigger cache)
        def.validate_value(&serde_json::json!("123")).unwrap();
        def.validate_value(&serde_json::json!("456")).unwrap();

        // AND: rejects non-matching strings
        assert!(def.validate_value(&serde_json::json!("abc")).is_err());
    }
}
