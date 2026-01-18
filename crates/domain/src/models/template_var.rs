use chrono::{DateTime, Utc};

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
    /// Gets default value as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
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
            } => default.as_ref().map(|v| serde_json::Value::from(v.as_str())),
        }
    }

    /// Checks if variable has a default value.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
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

    fn validate_boolean(value: &serde_json::Value) -> Result<(), DomainError> {
        if !value.is_boolean() {
            return Err(DomainError::InvalidType {
                value: value.to_string(),
                expected: "boolean".to_owned(),
            });
        }
        Ok(())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn validate_date(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        if let Self::Date {
            format,
            ..
        } = self
        {
            let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
                value: value.to_string(),
                expected: "string (date)".to_owned(),
            })?;
            if let Some(fmt) = format {
                chrono::NaiveDate::parse_from_str(s, fmt).map_err(|e| {
                    DomainError::InvalidDateFormat(e.to_string())
                })?;
            } else {
                s.parse::<DateTime<Utc>>().map_err(|e| {
                    DomainError::InvalidDateFormat(e.to_string())
                })?;
            }
            Ok(())
        } else {
            Err(DomainError::ValidationFailed(
                "Expected Date variant".to_owned(),
            ))
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn validate_file(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        if let Self::File {
            file_types,
            ..
        } = self
        {
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
        } else {
            Err(DomainError::ValidationFailed(
                "Expected File variant".to_owned(),
            ))
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn validate_number(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        if let Self::Number {
            min,
            max,
            ..
        } = self
        {
            let n = value.as_f64().ok_or_else(|| DomainError::InvalidType {
                value: value.to_string(),
                expected: "number".to_owned(),
            })?;

            if let Some(min_val) = min
                && n < *min_val
            {
                return Err(DomainError::NumberOutOfRange {
                    value: n,
                    min: Some(*min_val),
                    max: *max,
                });
            }

            if let Some(max_val) = max
                && n > *max_val
            {
                return Err(DomainError::NumberOutOfRange {
                    value: n,
                    min: *min,
                    max: Some(*max_val),
                });
            }
            Ok(())
        } else {
            Err(DomainError::ValidationFailed(
                "Expected Number variant".to_owned(),
            ))
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn validate_string(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        if let Self::String {
            min_length,
            max_length,
            pattern,
            ..
        } = self
        {
            let s = value.as_str().ok_or_else(|| DomainError::InvalidType {
                value: value.to_string(),
                expected: "string".to_owned(),
            })?;

            if let Some(min) = min_length
                && s.len() < *min
            {
                return Err(DomainError::StringTooShort {
                    min: *min,
                    actual: s.len(),
                });
            }

            if let Some(max) = max_length
                && s.len() > *max
            {
                return Err(DomainError::StringTooLong {
                    max: *max,
                    actual: s.len(),
                });
            }

            if let Some(p) = pattern {
                let re = regex::Regex::new(p)
                    .map_err(|e| DomainError::InvalidRegex(e.to_string()))?;
                if !re.is_match(s) {
                    return Err(DomainError::ValidationFailed(format!(
                        "String does not match pattern: {p}"
                    )));
                }
            }
            Ok(())
        } else {
            Err(DomainError::ValidationFailed(
                "Expected String variant".to_owned(),
            ))
        }
    }

    /// Validates a value against this definition.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidType` if type mismatch.
    /// Returns `DomainError::StringTooShort` or `DomainError::StringTooLong` if length constraints violated.
    /// Returns `DomainError::NumberOutOfRange` if range constraints violated.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
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
                ..
            } => self.validate_date(value),
            Self::File {
                ..
            } => self.validate_file(value),
            Self::Number {
                ..
            } => self.validate_number(value),
            Self::String {
                ..
            } => self.validate_string(value),
        }
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "Test logic")]
#[expect(clippy::assertions_on_result_states, reason = "Test logic")]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::models::template::{Metadata, Template};

    #[test]
    fn rejects_invalid_variable_names() {
        let names = vec![
            String::new(),
            "123var".to_owned(),
            "var-name".to_owned(),
            "var name".to_owned(),
            "if".to_owned(),
            "for".to_owned(),
        ];
        for name in names {
            let mut variables = HashMap::new();
            variables.insert(
                name.clone(),
                VariableDefinition::Boolean {
                    default: None,
                },
            );
            let result = Template::new(
                "test".to_owned(),
                "content".to_owned(),
                variables,
                None,
                Metadata::default(),
            );
            assert!(
                result.is_err(),
                "Expected variable name '{name}' to be rejected"
            );
        }
    }

    #[test]
    fn validates_string_constraints() {
        let def = VariableDefinition::String {
            default: None,
            max_length: Some(10),
            min_length: Some(3),
            pattern: Some("^[a-z]+$".to_owned()),
        };

        assert!(def.validate_value(&serde_json::json!("abc")).is_ok());
        assert!(def.validate_value(&serde_json::json!("ab")).is_err());
        assert!(def.validate_value(&serde_json::json!("abcdefghijk")).is_err());
        assert!(def.validate_value(&serde_json::json!("ABC")).is_err());
    }

    #[test]
    fn validates_number_constraints() {
        let def = VariableDefinition::Number {
            default: None,
            max: Some(10.0f64),
            min: Some(1.0f64),
        };

        assert!(def.validate_value(&serde_json::json!(5.0f64)).is_ok());
        assert!(def.validate_value(&serde_json::json!(0.5f64)).is_err());
        assert!(def.validate_value(&serde_json::json!(10.5f64)).is_err());
    }

    #[test]
    fn validates_boolean_constraints() {
        let def = VariableDefinition::Boolean {
            default: None,
        };
        assert!(def.validate_value(&serde_json::json!(true)).is_ok());
        assert!(def.validate_value(&serde_json::json!("true")).is_err());
    }
}
