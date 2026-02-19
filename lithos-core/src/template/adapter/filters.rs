use std::{cell::RefCell, collections::HashMap};

use chrono::NaiveDate;
use minijinja::{Environment, value::Kwargs};
use regex::Regex;

use super::FilterName;

/// Registry for custom `MiniJinja` filters that enforce input constraints.
#[non_exhaustive]
pub struct FilterRegistry;

impl FilterRegistry {
    /// Registers all custom filters in the given environment.
    #[inline]
    pub fn register_all(env: &mut Environment) {
        env.add_filter(
            FilterName::VALIDATE_LENGTH.as_str(),
            Self::validate_length,
        );
        env.add_filter(
            FilterName::VALIDATE_PATTERN.as_str(),
            Self::validate_pattern,
        );
        env.add_filter(
            FilterName::VALIDATE_RANGE.as_str(),
            Self::validate_range,
        );
        env.add_filter(
            FilterName::VALIDATE_FILE_TYPE.as_str(),
            Self::validate_file_type,
        );
        env.add_filter(FilterName::DATE_FORMAT.as_str(), Self::date_format);
        env.add_filter(FilterName::VAULT_PATH.as_str(), Self::vault_path);
    }

    /// String length validation filter.
    ///
    /// Usage: `{{ title | validate_length(min=5, max=100) }}`.
    #[expect(clippy::needless_pass_by_value, reason = "MiniJinja signature")]
    fn validate_length(
        value: String,
        kwargs: Kwargs,
    ) -> Result<String, minijinja::Error> {
        let min: Option<usize> = kwargs.get("min")?;
        let max: Option<usize> = kwargs.get("max")?;
        let len = value.len();

        if let Some(min) = min
            && len < min
        {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("String too short: min {min}, got {len}"),
            ));
        }

        if let Some(max) = max
            && len > max
        {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("String too long: max {max}, got {len}"),
            ));
        }

        Ok(value)
    }

    /// Regex pattern validation filter (with thread-local cache).
    ///
    /// Usage: `{{ name | validate_pattern(pattern="^[A-Z]") }}`.
    #[expect(clippy::needless_pass_by_value, reason = "MiniJinja signature")]
    fn validate_pattern(
        value: String,
        kwargs: Kwargs,
    ) -> Result<String, minijinja::Error> {
        let pattern: String = kwargs.get("pattern")?;

        thread_local! {
            static CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
        }

        let is_match =
            CACHE.with(|cache| -> Result<bool, minijinja::Error> {
                let mut cache = cache.borrow_mut();

                if let Some(re) = cache.get(&pattern) {
                    return Ok(re.is_match(&value));
                }

                let re = Regex::new(&pattern).map_err(|e| {
                    minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid regex pattern: {e}"),
                    )
                })?;

                let result = re.is_match(&value);
                cache.insert(pattern.clone(), re);
                Ok(result)
            })?;

        if !is_match {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("String does not match pattern: {pattern}"),
            ));
        }

        Ok(value)
    }

    /// Number range validation filter.
    ///
    /// Usage: `{{ priority | validate_range(min=1, max=10) }}`.
    #[expect(clippy::needless_pass_by_value, reason = "MiniJinja signature")]
    fn validate_range(
        value: f64,
        kwargs: Kwargs,
    ) -> Result<f64, minijinja::Error> {
        let min: Option<f64> = kwargs.get("min")?;
        let max: Option<f64> = kwargs.get("max")?;

        if !value.is_finite() {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("Value {value} is not finite"),
            ));
        }

        if let Some(min) = min
            && value < min
        {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("Value {value} is below min {min}"),
            ));
        }

        if let Some(max) = max
            && value > max
        {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("Value {value} is above max {max}"),
            ));
        }

        Ok(value)
    }

    /// File type validation filter.
    ///
    /// Usage: `{{ path | validate_file_type(types=["md", "txt"]) }}`.
    #[expect(clippy::needless_pass_by_value, reason = "MiniJinja signature")]
    fn validate_file_type(
        path: String,
        kwargs: Kwargs,
    ) -> Result<String, minijinja::Error> {
        let types: Vec<String> = kwargs.get("types")?;

        let ext = std::path::Path::new(&path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !types.iter().any(|t| t == ext) {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!(
                    "File extension '{ext}' not allowed. Expected: {types:?}"
                ),
            ));
        }

        Ok(path)
    }

    /// Date formatting filter.
    ///
    /// Usage: `{{ date | date_format(format="%Y-%m-%d") }}`.
    #[expect(clippy::needless_pass_by_value, reason = "MiniJinja signature")]
    fn date_format(
        date: String,
        kwargs: Kwargs,
    ) -> Result<String, minijinja::Error> {
        let format: Option<String> = kwargs.get("format")?;

        if let Some(fmt) = format {
            let parsed =
                NaiveDate::parse_from_str(&date, &fmt).map_err(|e| {
                    minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid date format: {e}"),
                    )
                })?;
            Ok(parsed.format(&fmt).to_string())
        } else {
            // ISO 8601 pass-through
            let _parsed =
                date.parse::<chrono::DateTime<chrono::Utc>>().map_err(|e| {
                    minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid ISO 8601 date: {e}"),
                    )
                })?;
            Ok(date)
        }
    }

    /// Vault path validation filter.
    ///
    /// Usage: `{{ path | vault_path }}`.
    fn vault_path(path: String) -> Result<String, minijinja::Error> {
        crate::fs::PathValidator::validate_vault_path(&path, None).map_err(
            |e| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("Invalid vault path: {e}"),
                )
            },
        )?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use minijinja::Environment;

    use super::*;

    #[test]
    fn filter_validate_length_passes() {
        let mut env = Environment::new();
        FilterRegistry::register_all(&mut env);

        env.add_template("test", "{{ text | validate_length(min=3, max=10) }}")
            .unwrap();

        let result = env
            .get_template("test")
            .unwrap()
            .render(minijinja::context! { text => "hello" });

        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn filter_validate_length_fails_too_short() {
        let mut env = Environment::new();
        FilterRegistry::register_all(&mut env);

        env.add_template("test", "{{ text | validate_length(min=5) }}")
            .unwrap();

        let result = env
            .get_template("test")
            .unwrap()
            .render(minijinja::context! { text => "hi" });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn filter_validate_pattern_passes() {
        let mut env = Environment::new();
        FilterRegistry::register_all(&mut env);

        env.add_template(
            "test",
            r#"{{ text | validate_pattern(pattern="^[A-Z]") }}"#,
        )
        .unwrap();

        let result = env
            .get_template("test")
            .unwrap()
            .render(minijinja::context! { text => "Hello" });

        let _: String = result.unwrap();
    }

    #[test]
    fn filter_validate_pattern_caches_regex() {
        // Test that pattern is cached (compile once, use many times)
        let mut env = Environment::new();
        FilterRegistry::register_all(&mut env);

        env.add_template("test", r#"{{ a | validate_pattern(pattern="^[a-z]+$") }} {{ b | validate_pattern(pattern="^[a-z]+$") }}"#).unwrap();

        let result = env
            .get_template("test")
            .unwrap()
            .render(minijinja::context! { a => "foo", b => "bar" });

        assert_eq!(result.unwrap(), "foo bar");
    }
}
