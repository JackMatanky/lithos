use crate::errors::DomainError;

/// Type-safe variable definition with validation constraints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    /// Checks if a number is within the specified range.
    fn check_number_range(
        n: f64,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), DomainError> {
        if let Some(min_val) = min
            && n < min_val
        {
            return Err(DomainError::NumberOutOfRange {
                value: n,
                min: Some(min_val),
                max,
            });
        }
        if let Some(max_val) = max
            && n > max_val
        {
            return Err(DomainError::NumberOutOfRange {
                value: n,
                min,
                max: Some(max_val),
            });
        }
        Ok(())
    }

    /// Checks string length constraints.
    fn check_string_length(
        s: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<(), DomainError> {
        if let Some(m) = min
            && s.len() < m
        {
            return Err(DomainError::StringTooShort {
                min: m,
                actual: s.len(),
            });
        }
        if let Some(m) = max
            && s.len() > m
        {
            return Err(DomainError::StringTooLong {
                max: m,
                actual: s.len(),
            });
        }
        Ok(())
    }

    /// Checks string pattern constraints.
    /// Adversarial Review Fix: Use a cache to avoid recompiling the same regex.
    fn check_string_pattern(
        s: &str,
        pattern: Option<&str>,
    ) -> Result<(), DomainError> {
        if let Some(p) = pattern {
            thread_local! {
                static CACHE: std::cell::RefCell<std::collections::HashMap<String, regex::Regex>> =
                    std::cell::RefCell::new(std::collections::HashMap::new());
            }

            let is_match =
                CACHE.with(|cache| -> Result<bool, DomainError> {
                    let mut cache = cache.borrow_mut();
                    if let Some(re) = cache.get(p) {
                        Ok(re.is_match(s))
                    } else {
                        let re = regex::Regex::new(p).map_err(|e| {
                            DomainError::InvalidRegex(e.to_string())
                        })?;
                        let res = re.is_match(s);
                        cache.insert(p.to_owned(), re);
                        Ok(res)
                    }
                })?;

            if !is_match {
                return Err(DomainError::ValidationFailed(format!(
                    "String does not match pattern: {p}"
                )));
            }
        }
        Ok(())
    }

    /// Gets default value as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(clippy::pattern_type_mismatch, reason = "Enum reference matching")]
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
    #[expect(clippy::pattern_type_mismatch, reason = "Enum reference matching")]
    #[expect(clippy::match_same_arms, reason = "Divergent types in arms")]
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

    /// Validates a value against this definition.
    #[inline]
    #[expect(clippy::pattern_type_mismatch, reason = "Enum reference matching")]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        match self {
            Self::Boolean {
                ..
            } => self.validate_boolean(value),
            Self::Date {
                format,
                ..
            } => self.validate_date(value, format.as_deref()),
            Self::File {
                file_types,
                ..
            } => self.validate_file(value, file_types.as_deref()),
            Self::Number {
                min,
                max,
                ..
            } => self.validate_number(value, *min, *max),
            Self::String {
                min_length,
                max_length,
                pattern,
                ..
            } => self.validate_string(
                value,
                *min_length,
                *max_length,
                pattern.as_deref(),
            ),
        }
    }

    #[inline]
    fn validate_boolean(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
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
        &self,
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

    #[inline]
    fn validate_file(
        &self,
        value: &serde_json::Value,
        file_types: Option<&[String]>,
    ) -> Result<(), DomainError> {
        let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "string (file path)".to_owned(),
        })?;

        if s.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        if let Some(allowed) = file_types {
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
    fn validate_number(
        &self,
        value: &serde_json::Value,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), DomainError> {
        let n = value.as_f64().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "number".to_owned(),
        })?;
        Self::check_number_range(n, min, max)
    }

    #[inline]
    fn validate_string(
        &self,
        value: &serde_json::Value,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<&str>,
    ) -> Result<(), DomainError> {
        let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
            value: value.to_string(),
            expected: "string".to_owned(),
        })?;
        Self::check_string_length(s, min_length, max_length)?;
        Self::check_string_pattern(s, pattern)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_boolean_constraints() {
        let def = VariableDefinition::Boolean {
            default: None,
        };
        def.validate_value(&serde_json::json!(true)).expect("Valid boolean");
        assert!(def.validate_value(&serde_json::json!("true")).is_err());
    }

    #[test]
    fn should_validate_number_constraints() {
        let def = VariableDefinition::Number {
            default: None,
            max: Some(10.0f64),
            min: Some(1.0f64),
        };
        def.validate_value(&serde_json::json!(5.0f64)).expect("Valid number");
        assert!(def.validate_value(&serde_json::json!(0.5f64)).is_err());
        assert!(def.validate_value(&serde_json::json!(10.5f64)).is_err());
    }
}
