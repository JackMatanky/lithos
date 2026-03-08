//! String property validation constraints.

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::error::SchemaError;

// ============================================================================
// Public API - Primary Types (ordered by importance to developers)
// ============================================================================

/// String property validation constraints.
///
/// # Invariants
/// - If `pattern` is set, it must be a valid regex (enforced at construction).
/// - `pattern` and `options` are independent constraints (both checked if
///   present).
///
/// # Examples
/// ```
/// use lithos_core::schema::property_spec::{StringPattern, StringSpec};
///
/// // No pattern, only options
/// let spec = StringSpec::try_new(None, None)?;
///
/// // With predefined pattern
/// let spec = StringSpec::try_new(Some(StringPattern::Email), None)?;
///
/// // With custom pattern
/// let pattern = StringPattern::try_custom(r"^\d{3}-\d{4}$")?;
/// let spec = StringSpec::try_new(Some(pattern), None)?;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StringSpec {
    options: Option<Vec<OptionEntry>>,
    pattern: Option<StringPattern>,
}

impl Default for StringSpec {
    #[inline]
    fn default() -> Self {
        Self {
            options: None,
            pattern: None,
        }
    }
}

impl StringSpec {
    /// Create a validated `StringSpec`.
    ///
    /// Pattern is already validated at `StringPattern` construction time,
    /// so this constructor is infallible for valid inputs.
    ///
    /// # Errors
    /// This function is currently infallible but returns `Result` for
    /// future extensibility and API consistency.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::{StringPattern, StringSpec};
    ///
    /// let spec = StringSpec::try_new(None, None)?;
    ///
    /// let pattern = StringPattern::try_custom(r"^\d+$")?;
    /// let spec = StringSpec::try_new(Some(pattern), None)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_new(
        pattern: Option<StringPattern>,
        options: Option<Vec<OptionEntry>>,
    ) -> Result<Self, SchemaError> {
        Ok(Self {
            options,
            pattern,
        })
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
    /// let base = StringSpec::try_new(None, None)?;
    /// let overrides = RawStringSpec::default();
    /// let _updated = base.apply_overrides(&overrides)?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawStringSpec,
    ) -> Result<Self, SchemaError> {
        use crate::schema::raw::RawStringFormat;

        // Convert RawStringSpec (pattern/format separate) to unified
        // StringPattern
        let pattern = match (overrides.pattern.as_ref(), overrides.format) {
            (Some(p), None) => Some(StringPattern::try_custom(p.clone())?),
            (None, Some(f)) => {
                // Convert RawStringFormat to StringPattern
                let sp = match f {
                    RawStringFormat::Email => StringPattern::Email,
                    RawStringFormat::Url => StringPattern::Url,
                    RawStringFormat::PhoneUs => StringPattern::PhoneUs,
                    RawStringFormat::Slug => StringPattern::Slug,
                    RawStringFormat::UuidV4 => StringPattern::UuidV4,
                    RawStringFormat::WikiLink => StringPattern::WikiLink,
                    RawStringFormat::ZipCode => StringPattern::ZipCode,
                };
                Some(sp)
            }
            (None, None) => self.pattern,
            (Some(_), Some(_)) => {
                return Err(SchemaError::ValidationFailed(
                    "pattern and format are mutually exclusive".into(),
                ));
            }
        };

        let options = overrides
            .options
            .as_ref()
            .map(|o| {
                o.clone().into_entries().into_iter().map(Into::into).collect()
            })
            .or(self.options);

        Self::try_new(pattern, options)
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
        if let Some(pattern) = self.pattern.as_ref() {
            pattern.validate(value)?;
        }
        Ok(())
    }
}

// ============================================================================
// Public API - Supporting Types
// ============================================================================

/// String validation pattern (predefined format or custom regex).
///
/// This enum unifies predefined formats (like `Email`, `Url`) with
/// user-defined custom regex patterns into a single type-safe API.
///
/// # Examples
///
/// ```
/// use lithos_core::schema::property_spec::StringPattern;
///
/// // Predefined format
/// let email = StringPattern::Email;
/// assert!(email.pattern().contains("@"));
///
/// // Custom pattern
/// let custom = StringPattern::try_custom(r"^\d{3}-\d{3}-\d{4}$")?;
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
#[non_exhaustive]
pub enum StringPattern {
    /// Email address validation (RFC 5322 simplified).
    ///
    /// Pattern: `^[^@]+@[^@]+\.[^@]+$`.
    Email,

    /// URL validation (HTTP/HTTPS).
    ///
    /// Pattern: `^https?://[^\s/$.?#].[^\s]*$`.
    Url,

    /// US phone number validation.
    ///
    /// Pattern: `^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?
    /// ([0-9]{4})$`.
    PhoneUs,

    /// Slug validation (kebab-case).
    ///
    /// Pattern: `^[a-z0-9]+(-[a-z0-9]+)*$`.
    Slug,

    /// UUID v4 validation.
    ///
    /// Pattern: `^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`.
    UuidV4,

    /// `WikiLink` validation (Obsidian-style).
    ///
    /// Pattern: `^\[\[([^\]|]+)(\|[^\]]+)?\]\]$`.
    WikiLink,

    /// US ZIP code validation (5 or 9 digits).
    ///
    /// Pattern: `^\d{5}(-\d{4})?$`.
    ZipCode,

    /// User-defined custom regex pattern.
    ///
    /// The pattern is validated at construction time via `try_custom()`.
    Custom(Box<str>),
}

impl StringPattern {
    /// Create a custom pattern, validating the regex is compilable.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidRegex` if the pattern is invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::StringPattern;
    ///
    /// let pattern = StringPattern::try_custom(r"^\d+$")?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_custom<S: Into<Box<str>>>(
        pattern: S,
    ) -> Result<Self, SchemaError> {
        let pattern = pattern.into();
        // Validate pattern compiles (then discard compiled regex)
        regex::Regex::new(&pattern).map_err(|e| {
            SchemaError::InvalidRegex(format!("Invalid pattern {pattern}: {e}"))
        })?;
        Ok(Self::Custom(pattern))
    }

    /// Returns the regex pattern string for this pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::property_spec::StringPattern;
    ///
    /// assert_eq!(StringPattern::Slug.pattern(), "^[a-z0-9]+(-[a-z0-9]+)*$");
    ///
    /// let custom = StringPattern::try_custom(r"^\d+$")?;
    /// assert_eq!(custom.pattern(), r"^\d+$");
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn pattern(&self) -> &str {
        match self {
            Self::Email => r"^[^@]+@[^@]+\.[^@]+$",
            Self::Url => r"^https?://[^\s/$.?#].[^\s]*$",
            Self::PhoneUs => {
                r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$"
            }
            Self::Slug => "^[a-z0-9]+(-[a-z0-9]+)*$",
            Self::UuidV4 => {
                "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
            }
            Self::WikiLink => r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$",
            Self::ZipCode => r"^\d{5}(-\d{4})?$",
            Self::Custom(pattern) => pattern.as_ref(),
        }
    }

    /// Returns the human-readable name of this pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::property_spec::StringPattern;
    ///
    /// assert_eq!(StringPattern::Email.name(), "email");
    /// assert_eq!(StringPattern::PhoneUs.name(), "phone_us");
    ///
    /// let custom = StringPattern::try_custom(r"^\d+$")?;
    /// assert_eq!(custom.name(), "custom");
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn name(&self) -> &str {
        match self {
            Self::Email => "email",
            Self::Url => "url",
            Self::PhoneUs => "phone_us",
            Self::Slug => "slug",
            Self::UuidV4 => "uuid_v4",
            Self::WikiLink => "wikilink",
            Self::ZipCode => "zipcode",
            Self::Custom(_) => "custom",
        }
    }

    /// Validate a string value against this pattern.
    ///
    /// # Errors
    /// Returns `SchemaError::ValidationFailed` if the value doesn't match the
    /// pattern.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property_spec::StringPattern;
    ///
    /// let pattern = StringPattern::Email;
    /// assert!(pattern.validate("user@example.com").is_ok());
    /// assert!(pattern.validate("invalid").is_err());
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        let re = self.regex();
        if !re.is_match(value) {
            return Err(SchemaError::ValidationFailed(format!(
                "Value {value} does not match pattern '{self}' ({})",
                self.pattern()
            )));
        }
        Ok(())
    }

    /// Returns a compiled `Regex` for this pattern.
    ///
    /// Predefined formats use static `OnceLock` cells for zero-cost caching.
    /// Custom patterns use a shared cache to avoid recompilation.
    ///
    /// # Panics
    /// Panics if a built-in pattern is invalid (indicates programmer error).
    /// Custom patterns are validated at construction, so they never panic.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn regex(&self) -> Arc<regex::Regex> {
        match self {
            Self::Email => Arc::new(Self::static_regex_email().clone()),
            Self::Url => Arc::new(Self::static_regex_url().clone()),
            Self::PhoneUs => Arc::new(Self::static_regex_phone_us().clone()),
            Self::Slug => Arc::new(Self::static_regex_slug().clone()),
            Self::UuidV4 => Arc::new(Self::static_regex_uuid_v4().clone()),
            Self::WikiLink => Arc::new(Self::static_regex_wikilink().clone()),
            Self::ZipCode => Arc::new(Self::static_regex_zipcode().clone()),
            Self::Custom(pattern) => get_or_compile_pattern(pattern),
        }
    }

    // Static regex accessors for predefined formats
    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_email() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::Email.pattern())
                .expect("built-in Email regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_url() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::Url.pattern())
                .expect("built-in Url regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_phone_us() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::PhoneUs.pattern())
                .expect("built-in PhoneUs regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_slug() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::Slug.pattern())
                .expect("built-in Slug regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_uuid_v4() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::UuidV4.pattern())
                .expect("built-in UuidV4 regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_wikilink() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::WikiLink.pattern())
                .expect("built-in WikiLink regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_zipcode() -> &'static regex::Regex {
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(Self::ZipCode.pattern())
                .expect("built-in ZipCode regex pattern is valid")
        })
    }
}

impl std::fmt::Display for StringPattern {
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(pattern) => write!(f, "custom({pattern})"),
            Self::Email
            | Self::Url
            | Self::PhoneUs
            | Self::Slug
            | Self::UuidV4
            | Self::WikiLink
            | Self::ZipCode => write!(f, "{}", self.name()),
        }
    }
}

/// Legacy compatibility: Re-export as `StringFormat` for existing code.
///
/// **Deprecated**: Use `StringPattern` directly. This alias exists only
/// for backward compatibility during migration.
#[deprecated(
    since = "0.1.0",
    note = "Use StringPattern instead. StringFormat has been unified with \
            custom patterns."
)]
pub type StringFormat = StringPattern;

/// A validated option entry with optional display label.
///
/// This is created from `RawOptionEntry` after ordering is applied.
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

impl From<crate::schema::raw::RawOptionEntry> for OptionEntry {
    #[inline]
    fn from(raw: crate::schema::raw::RawOptionEntry) -> Self {
        Self {
            value: raw.value,
            label: raw.label,
        }
    }
}

// ============================================================================
// Internal Implementation - Regex Cache
// ============================================================================

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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod string_pattern {
        use super::*;

        #[test]
        fn email_format_has_correct_pattern() {
            assert_eq!(StringPattern::Email.pattern(), r"^[^@]+@[^@]+\.[^@]+$");
        }

        #[test]
        fn url_format_has_correct_pattern() {
            assert_eq!(
                StringPattern::Url.pattern(),
                r"^https?://[^\s/$.?#].[^\s]*$"
            );
        }

        #[test]
        fn phone_us_format_has_correct_pattern() {
            assert_eq!(
                StringPattern::PhoneUs.pattern(),
                r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$"
            );
        }

        #[test]
        fn slug_format_has_correct_pattern() {
            assert_eq!(
                StringPattern::Slug.pattern(),
                "^[a-z0-9]+(-[a-z0-9]+)*$"
            );
        }

        #[test]
        fn uuid_v4_format_has_correct_pattern() {
            assert_eq!(
                StringPattern::UuidV4.pattern(),
                "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
            );
        }

        #[test]
        fn wikilink_format_has_correct_pattern() {
            assert_eq!(
                StringPattern::WikiLink.pattern(),
                r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$"
            );
        }

        #[test]
        fn zipcode_format_has_correct_pattern() {
            assert_eq!(StringPattern::ZipCode.pattern(), r"^\d{5}(-\d{4})?$");
        }

        #[test]
        fn format_name_matches_serde_representation() {
            assert_eq!(StringPattern::Email.name(), "email");
            assert_eq!(StringPattern::PhoneUs.name(), "phone_us");
        }

        #[test]
        fn format_display_uses_name() {
            assert_eq!(StringPattern::Email.to_string(), "email");
            assert_eq!(StringPattern::Slug.to_string(), "slug");
        }

        #[test]
        fn custom_pattern_validates_and_stores() {
            let pattern = StringPattern::try_custom(r"^\d{3}-\d{4}$")
                .expect("Valid pattern");
            assert_eq!(pattern.pattern(), r"^\d{3}-\d{4}$");
            assert_eq!(pattern.name(), "custom");
        }

        #[test]
        fn custom_pattern_rejects_invalid_regex() {
            let result = StringPattern::try_custom("(?P<unclosed");
            result.unwrap_err();
        }
    }

    mod string_spec {
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
            Err(SchemaError::ValidationFailed("Value abc does not match pattern 'custom(^\\d+$)' (^\\d+$)".to_owned()))
        )]
        fn string_spec_validation_matrix(
            #[case] def: RawStringSpec,
            #[case] value: &str,
            #[case] expected: Result<(), SchemaError>,
        ) {
            fn validated_spec(def: &RawStringSpec) -> StringSpec {
                // Use apply_overrides which handles conversion and validation
                let base = StringSpec::default();
                base.apply_overrides(def).expect("Test data should be valid")
            }

            let spec = validated_spec(&def);

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
}
