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
        default: Option<Box<str>>,
        /// ISO 8601 format string.
        format: Option<Box<str>>,
    },
    /// File reference variable.
    File {
        /// Default value.
        default: Option<Box<str>>,
        /// Allowed file types.
        file_types: Option<Vec<Box<str>>>,
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
        default: Option<Box<str>>,
        /// Maximum length.
        max_length: Option<usize>,
        /// Minimum length.
        min_length: Option<usize>,
        /// Regex pattern.
        pattern: Option<Box<str>>,
    },
}

impl VariableDefinition {
    /// Returns filter names to apply at render time.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on reference to enum variants"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Default behavior for unmatched variants is empty list"
    )]
    pub fn filter_chain(&self) -> Vec<&'static str> {
        match self {
            Self::String {
                pattern: Some(_),
                min_length: Some(_),
                ..
            }
            | Self::String {
                pattern: Some(_),
                max_length: Some(_),
                ..
            } => {
                vec!["validate_pattern", "validate_length"]
            }
            Self::String {
                pattern: Some(_),
                ..
            } => vec!["validate_pattern"],
            Self::String {
                min_length: Some(_),
                ..
            }
            | Self::String {
                max_length: Some(_),
                ..
            } => vec!["validate_length"],
            Self::Number {
                min: Some(_),
                ..
            }
            | Self::Number {
                max: Some(_),
                ..
            } => {
                vec!["validate_range"]
            }
            Self::File {
                file_types: Some(_),
                ..
            } => vec!["validate_file_type"],
            Self::Date {
                format: Some(_),
                ..
            } => vec!["date_format"],
            _ => vec![],
        }
    }

    /// Returns filter arguments as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on reference to enum variants"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Default behavior for unmatched variants is null"
    )]
    pub fn filter_args(&self) -> serde_json::Value {
        match self {
            Self::String {
                min_length,
                max_length,
                pattern,
                ..
            } => {
                let mut map = serde_json::Map::new();
                if let Some(min) = min_length {
                    map.insert("min".to_owned(), (*min).into());
                }
                if let Some(max) = max_length {
                    map.insert("max".to_owned(), (*max).into());
                }
                if let Some(p) = pattern {
                    map.insert("pattern".to_owned(), p.as_ref().into());
                }
                serde_json::Value::Object(map)
            }
            Self::Number {
                min,
                max,
                ..
            } => {
                let mut map = serde_json::Map::new();
                if let Some(min) = min {
                    map.insert("min".to_owned(), (*min).into());
                }
                if let Some(max) = max {
                    map.insert("max".to_owned(), (*max).into());
                }
                serde_json::Value::Object(map)
            }
            Self::File {
                file_types: Some(types),
                ..
            } => {
                let mut map = serde_json::Map::new();
                map.insert(
                    "extensions".to_owned(),
                    serde_json::Value::Array(
                        types
                            .iter()
                            .map(|s| serde_json::Value::String(s.to_string()))
                            .collect(),
                    ),
                );
                serde_json::Value::Object(map)
            }
            Self::Date {
                format: Some(f),
                ..
            } => {
                let mut map = serde_json::Map::new();
                map.insert("format".to_owned(), f.as_ref().into());
                serde_json::Value::Object(map)
            }
            _ => serde_json::Value::Null,
        }
    }

    /// Returns default value as `serde_json::Value`.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Mixed Copy and non-Copy enum fields."
    )]
    pub fn default_value(&self) -> Option<serde_json::Value> {
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

    /// Checks string pattern constraints.
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
    /// Returns `TemplateError` if validation fails.
    #[inline]
    #[deprecated(
        since = "0.2.0",
        note = "Variable validation is handled by MiniJinja filters."
    )]
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
        allowed_types: Option<&[Box<str>]>,
    ) -> Result<(), TemplateError> {
        if let Some(allowed) = allowed_types {
            let ext = std::path::Path::new(s)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !allowed.iter().any(|a| a.as_ref() == ext) {
                return Err(TemplateError::InvalidFileClass(s.to_owned()));
            }
        }
        Ok(())
    }

    #[inline]
    fn validate_file(
        value: &serde_json::Value,
        file_types: Option<&[Box<str>]>,
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
#[expect(deprecated, reason = "Legacy validation tests")]
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

    fn boolean_def() -> VariableDefinition {
        VariableDefinition::Boolean {
            default: None,
        }
    }

    fn number_def() -> VariableDefinition {
        VariableDefinition::Number {
            default: None,
            max: Some(10.0f64),
            min: Some(1.0f64),
        }
    }

    fn string_def_with_default() -> VariableDefinition {
        VariableDefinition::String {
            default: Some("Title".into()),
            max_length: None,
            min_length: None,
            pattern: None,
        }
    }

    fn date_def_iso() -> VariableDefinition {
        VariableDefinition::Date {
            default: None,
            format: None,
        }
    }

    fn date_def_custom() -> VariableDefinition {
        VariableDefinition::Date {
            default: None,
            format: Some("%Y-%m-%d".into()),
        }
    }

    fn file_def_md() -> VariableDefinition {
        VariableDefinition::File {
            default: None,
            file_types: Some(vec!["md".into()]),
        }
    }

    fn file_def_any() -> VariableDefinition {
        VariableDefinition::File {
            default: None,
            file_types: None,
        }
    }

    fn file_def_multi() -> VariableDefinition {
        VariableDefinition::File {
            default: None,
            file_types: Some(vec!["md".into(), "txt".into()]),
        }
    }

    fn string_def_range(min: usize, max: usize) -> VariableDefinition {
        VariableDefinition::String {
            default: None,
            max_length: Some(max),
            min_length: Some(min),
            pattern: None,
        }
    }

    fn string_def_min_only(min: usize) -> VariableDefinition {
        VariableDefinition::String {
            default: None,
            max_length: None,
            min_length: Some(min),
            pattern: None,
        }
    }

    fn string_def_max_only(max: usize) -> VariableDefinition {
        VariableDefinition::String {
            default: None,
            max_length: Some(max),
            min_length: None,
            pattern: None,
        }
    }

    fn string_def_pattern() -> VariableDefinition {
        VariableDefinition::String {
            default: None,
            max_length: None,
            min_length: None,
            pattern: Some(r"^\d+$".into()),
        }
    }

    #[test]
    fn accepts_boolean_values() {
        let def = boolean_def();
        let result = def.validate_value(&serde_json::Value::Bool(true));

        assert!(
            result.is_ok(),
            "Boolean value should be accepted, but got: {result:?}"
        );
    }

    #[test]
    fn rejects_non_boolean_values() {
        let def = boolean_def();
        let result = def.validate_value(&str_value("true"));

        assert!(
            result.is_err(),
            "String value should be rejected for boolean field, got: \
             {result:?}"
        );
    }

    #[test]
    fn accepts_number_in_range() {
        let def = number_def();
        let result = def.validate_value(&f64_value(5.0f64));

        assert!(
            result.is_ok(),
            "Number in range should be accepted, but got: {result:?}"
        );
    }

    #[test]
    fn rejects_number_below_min() {
        let def = number_def();
        let result = def.validate_value(&f64_value(0.5f64));

        assert!(
            result.is_err(),
            "Value below min should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn rejects_number_above_max() {
        let def = number_def();
        let result = def.validate_value(&f64_value(10.5f64));

        assert!(
            result.is_err(),
            "Value above max should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn has_default_returns_true_when_default_present() {
        let def = string_def_with_default();

        assert!(
            def.has_default(),
            "Variable definition should indicate that a default value is set"
        );
    }

    #[test]
    fn get_default_value_returns_configured_value() {
        let def = string_def_with_default();

        assert_eq!(
            def.default_value(),
            Some(str_value("Title")),
            "Default value should match the configured value"
        );
    }

    #[test]
    fn accepts_iso8601_date_values() {
        let def = date_def_iso();
        let result = def.validate_value(&str_value("2024-01-15T10:00:00Z"));

        assert!(
            result.is_ok(),
            "Valid ISO8601 date should be accepted, got: {result:?}"
        );
    }

    #[test]
    fn rejects_invalid_iso8601_dates() {
        let def = date_def_iso();
        let result = def.validate_value(&str_value("invalid"));

        assert!(
            result.is_err(),
            "Invalid date string should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn accepts_custom_format_dates() {
        let def = date_def_custom();
        let result = def.validate_value(&str_value("2024-01-15"));

        assert!(
            result.is_ok(),
            "Valid custom format date should be accepted, got: {result:?}"
        );
    }

    #[test]
    fn rejects_mismatched_custom_format_dates() {
        let def = date_def_custom();
        let result = def.validate_value(&str_value("2024/01/15"));

        assert!(
            result.is_err(),
            "Date not matching custom format should be rejected, got: \
             {result:?}"
        );
    }

    #[test]
    fn accepts_file_with_allowed_extension() {
        let def = file_def_md();
        let result = def.validate_value(&str_value("note.md"));

        assert!(
            result.is_ok(),
            "Valid file with correct extension should be accepted, got: \
             {result:?}"
        );
    }

    #[test]
    fn rejects_file_with_disallowed_extension() {
        let def = file_def_md();
        let result = def.validate_value(&str_value("img.png"));

        assert!(
            result.is_err(),
            "File with wrong extension should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn rejects_absolute_file_paths() {
        let def = file_def_md();
        let result = def.validate_value(&str_value("/abs/path"));

        assert!(
            result.is_err(),
            "Absolute path should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn accepts_files_when_no_type_restriction() {
        let def = file_def_any();
        let result = def.validate_value(&str_value("any.file"));

        assert!(
            result.is_ok(),
            "File with no type restriction should accept any extension, got: \
             {result:?}"
        );
    }

    #[test]
    fn accepts_md_extension_when_allowed() {
        let def = file_def_multi();
        let result = def.validate_value(&str_value("test.md"));

        assert!(
            result.is_ok(),
            "File with .md extension should be accepted, got: {result:?}"
        );
    }

    #[test]
    fn accepts_txt_extension_when_allowed() {
        let def = file_def_multi();
        let result = def.validate_value(&str_value("test.txt"));

        assert!(
            result.is_ok(),
            "File with .txt extension should be accepted, got: {result:?}"
        );
    }

    #[test]
    fn rejects_unlisted_file_extensions() {
        let def = file_def_multi();
        let result = def.validate_value(&str_value("test.png"));

        assert!(
            result.is_err(),
            "File with .png extension should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn accepts_strings_within_length_constraints() {
        let def = string_def_range(2, 5);
        let result = def.validate_value(&str_value("abc"));

        assert!(
            result.is_ok(),
            "String within length constraints should be accepted, got: \
             {result:?}"
        );
    }

    #[test]
    fn rejects_strings_below_min_length() {
        let def = string_def_range(2, 5);
        let result = def.validate_value(&str_value("a"));

        assert!(
            result.is_err(),
            "String below min length should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn rejects_strings_above_max_length() {
        let def = string_def_range(2, 5);
        let result = def.validate_value(&str_value("abcdef"));

        assert!(
            result.is_err(),
            "String above max length should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn rejects_strings_below_min_length_when_only_min_set() {
        let def = string_def_min_only(3);
        let result = def.validate_value(&str_value("ab"));

        assert!(
            result.is_err(),
            "String below min_length should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn rejects_strings_above_max_length_when_only_max_set() {
        let def = string_def_max_only(3);
        let result = def.validate_value(&str_value("abcd"));

        assert!(
            result.is_err(),
            "String above max_length should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn accepts_strings_matching_pattern() {
        let def = string_def_pattern();
        let result = def.validate_value(&str_value("123"));

        assert!(
            result.is_ok(),
            "String matching pattern should be accepted, got: {result:?}"
        );
    }

    #[test]
    fn rejects_strings_not_matching_pattern() {
        let def = string_def_pattern();
        let result = def.validate_value(&str_value("abc"));

        assert!(
            result.is_err(),
            "String not matching pattern should be rejected, got: {result:?}"
        );
    }

    #[test]
    fn filter_chain_for_string_with_pattern_and_length() {
        let var = VariableDefinition::String {
            default: None,
            min_length: Some(5),
            max_length: Some(10),
            pattern: Some("^[A-Z]".into()),
        };

        let chain = var.filter_chain();
        assert_eq!(chain, vec!["validate_pattern", "validate_length"]);
    }

    #[test]
    fn filter_chain_for_number_with_range() {
        let var = VariableDefinition::Number {
            default: None,
            min: Some(1.0f64),
            max: Some(10.0f64),
        };

        let chain = var.filter_chain();
        assert_eq!(chain, vec!["validate_range"]);
    }

    #[test]
    fn filter_args_for_string() {
        let var = VariableDefinition::String {
            default: None,
            min_length: Some(5),
            max_length: Some(10),
            pattern: Some("^[A-Z]".into()),
        };

        let args = var.filter_args();
        assert_eq!(
            args.get("min").and_then(serde_json::Value::as_u64),
            Some(5)
        );
        assert_eq!(
            args.get("max").and_then(serde_json::Value::as_u64),
            Some(10)
        );
        assert_eq!(
            args.get("pattern").and_then(serde_json::Value::as_str),
            Some("^[A-Z]")
        );
    }
}
