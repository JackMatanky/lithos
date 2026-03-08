//! String property validation constraints.

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{error::SchemaError, formats::StringFormat};

/// A validated option entry with optional display label.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct OptionEntry {
    /// The option value used in validation.
    pub value: Box<str>,
    /// Optional display label for UI consumers.
    pub label: Option<Box<str>>,
}

/// String property validation constraints.
///
/// # Invariants
/// - `format` and `pattern` are mutually exclusive (only one can be set).
/// - If `pattern` is set, it must be a valid regex.
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::StringSpec;
///
/// let spec = StringSpec::try_new(None, None, None)?;
/// let _ = spec;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StringSpec {
    options: Option<Vec<OptionEntry>>,
    pattern: Option<Box<str>>,
    format: Option<StringFormat>,
}

impl Default for StringSpec {
    #[inline]
    fn default() -> Self {
        Self {
            options: None,
            pattern: None,
            format: None,
        }
    }
}

impl StringSpec {
    /// Create a validated `StringSpec`.
    ///
    /// # Errors
    /// Returns `SchemaError` if:
    /// - `pattern` is present but not a valid regex.
    /// - Both `pattern` and `format` are specified (mutually exclusive).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::StringSpec;
    ///
    /// let _spec = StringSpec::try_new(None, None, None)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_new(
        pattern: Option<Box<str>>,
        format: Option<StringFormat>,
        options: Option<Vec<OptionEntry>>,
    ) -> Result<Self, SchemaError> {
        // Validate mutual exclusivity
        if pattern.is_some() && format.is_some() {
            return Err(SchemaError::ValidationFailed(
                "pattern and format are mutually exclusive".into(),
            ));
        }

        // Validate pattern if present (compile to check validity, then discard)
        if let Some(p) = pattern.as_ref() {
            regex::Regex::new(p).map_err(|e| {
                SchemaError::InvalidRegex(format!("Invalid pattern {p}: {e}"))
            })?;
        }

        Ok(Self {
            options,
            pattern,
            format,
        })
    }

    #[inline]
    pub(super) fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        self.validate_options(value)?;
        self.validate_pattern(value)?;
        Ok(())
    }

    fn validate_options(&self, value: &str) -> Result<(), SchemaError> {
        if let Some(entries) = self.options.as_ref()
            && !entries.iter().any(|e| e.value.as_ref() == value)
        {
            return Err(SchemaError::InvalidEnumValue {
                value: value.into(),
                allowed: entries
                    .iter()
                    .map(|e| e.value.as_ref().into())
                    .collect(),
            });
        }
        Ok(())
    }

    fn validate_pattern(&self, value: &str) -> Result<(), SchemaError> {
        // Use format regex if specified (pre-compiled static)
        if let Some(format) = self.format {
            let re = format.regex();
            if !re.is_match(value) {
                return Err(SchemaError::ValidationFailed(format!(
                    "Value {value} does not match format '{format}' (pattern: \
                     {})",
                    format.pattern()
                )));
            }
            return Ok(());
        }

        // Otherwise use custom pattern if specified (cached compilation)
        if let Some(pattern) = self.pattern.as_ref() {
            let re = get_or_compile_pattern(pattern);
            if !re.is_match(value) {
                return Err(SchemaError::ValidationFailed(format!(
                    "Value {value} does not match pattern {pattern}"
                )));
            }
        }
        Ok(())
    }

    /// Apply overrides from a raw string spec.
    ///
    /// Fields that are `None` in the overrides preserve the base values.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{property_spec::StringSpec, raw::RawStringSpec};
    ///
    /// let base = StringSpec::try_new(None, None, None)?;
    /// let overrides = RawStringSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawStringSpec,
    ) -> Result<Self, SchemaError> {
        let pattern = overrides.pattern.clone().or(self.pattern);
        let format = overrides.format.or(self.format);
        let options = overrides
            .options
            .as_ref()
            .map(|o| o.clone().into_entries())
            .or(self.options);
        Self::try_new(pattern, format, options)
    }
}

/// Cache for user-defined custom regex patterns.
///
/// Built-in formats use static `OnceLock` per format. Custom patterns use this
/// shared cache to avoid recompiling on every validation.
///
/// Design: Simple unbounded cache since:
/// 1. Patterns are validated at schema load time (guaranteed valid)
/// 2. Number of unique patterns is bounded by number of properties (~100s)
/// 3. Cache is per-process, shared across all validations
type CustomPatternCache = HashMap<Box<str>, Arc<regex::Regex>>;

static CUSTOM_PATTERN_CACHE: OnceLock<RwLock<CustomPatternCache>> =
    OnceLock::new();

/// Get or compile a custom regex pattern.
///
/// Uses a simple cache to avoid recompiling patterns on every validation.
/// Patterns are guaranteed valid (validated at construction time).
#[expect(
    clippy::expect_used,
    reason = "Pattern validated at StringSpec construction, expect documents \
              invariant"
)]
fn get_or_compile_pattern(pattern: &str) -> Arc<regex::Regex> {
    let cache =
        CUSTOM_PATTERN_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    // Fast path: read lock
    {
        let guard = cache.read().unwrap_or_else(|e| {
            tracing::warn!(
                "Regex cache RwLock poisoned (recovered from panic), \
                 proceeding with recovery"
            );
            e.into_inner()
        });
        if let Some(re) = guard.get(pattern) {
            return Arc::clone(re);
        }
    }

    // Slow path: compile and cache
    // Pattern is guaranteed valid (validated in try_new), so expect is safe
    let compiled =
        Arc::new(regex::Regex::new(pattern).expect(
            "Custom pattern should be valid (validated at construction)",
        ));

    let mut guard = cache.write().unwrap_or_else(|e| {
        tracing::warn!(
            "Regex cache RwLock poisoned during write (recovered from panic), \
             proceeding with recovery"
        );
        e.into_inner()
    });
    // Check again in case another thread inserted while we compiled
    if let Some(re) = guard.get(pattern) {
        return Arc::clone(re);
    }
    guard.insert(pattern.into(), Arc::clone(&compiled));
    compiled
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::schema::raw::{RawOptions, RawStringSpec};

    /// 3.3-UNIT-011: String Specification Validation Matrix.
    /// Priority: P1.
    #[rstest]
    #[case::options_match(
        RawStringSpec {
            options: Some(RawOptions::List(vec!["A".into(), "B".into()])),
            ..Default::default()
        },
        "A",
        Ok(())
    )]
    #[case::options_mismatch(
        RawStringSpec {
            options: Some(RawOptions::List(vec!["A".into(), "B".into()])),
            ..Default::default()
        },
        "C",
        Err(SchemaError::InvalidEnumValue {
            value: "C".to_owned(),
            allowed: vec!["A".to_owned(), "B".to_owned()]
        })
    )]
    #[case::regex_match(
        RawStringSpec { pattern: Some(r"^\d+$".into()), ..Default::default() },
        "123",
        Ok(())
    )]
    #[case::regex_mismatch(
        RawStringSpec { pattern: Some(r"^\d+$".into()), ..Default::default() },
        "abc",
        Err(SchemaError::ValidationFailed("Value abc does not match pattern ^\\d+$".to_owned()))
    )]
    fn string_spec_validation_matrix(
        #[case] def: RawStringSpec,
        #[case] value: &str,
        #[case] expected: Result<(), SchemaError>,
    ) {
        fn validated_spec(def: RawStringSpec) -> StringSpec {
            StringSpec::try_new(
                def.pattern,
                def.format,
                def.options.map(RawOptions::into_entries),
            )
            .expect("Expected valid RawStringSpec")
        }

        let spec = validated_spec(def);

        // WHEN: validating a string value
        let result = spec.validate_str(value);

        // THEN: the result matches the expectation
        assert_eq!(
            result, expected,
            "String validation failed for value='{value}': expected \
             {expected:?}, got {result:?}"
        );
    }
}
