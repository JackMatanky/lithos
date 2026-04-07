//! String property validation constraints.

use std::sync::Arc;

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::error::SchemaError;

// ============================================================================
// Public API - Primary Types
// ============================================================================

/// String property validation constraints.
///
/// A `StringSpec` supports four operational states:
/// - **Neither**: Accepts any string.
/// - **Only `options`**: Acts as an enum-style restricted list.
/// - **Only `pattern`**: Validates the string against a custom regex or
///   predefined format.
/// - **Both**: Provides a restricted list where *every element* must
///   additionally satisfy the pattern constraint.
///
/// # Invariants
/// - If `pattern` is set and is a custom regex, it must be valid (enforced at
///   construction).
/// - If both `pattern` and `options` are set, every option must match the
///   pattern (enforced at construction).
/// - `options` (if present) must be non-empty.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub struct StringSpec {
    /// Optional allowed values (enum-like).
    options: Option<OptionEntries>,
    /// Optional regex pattern or predefined format.
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
    /// Create a new `StringSpec`, validating consistency between constraints.
    ///
    /// # Errors
    /// Returns `SchemaError` if:
    /// 1. Options list is empty (but present).
    /// 2. Any option value does not match the provided pattern.
    #[inline]
    pub fn try_new(
        pattern: Option<StringPattern>,
        options: Option<OptionEntries>,
    ) -> Result<Self, SchemaError> {
        // 1. Validate options are not empty if present
        if let Some(opts) = options.as_ref()
            && opts.is_empty()
        {
            return Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::OptionsEmpty,
            ));
        }

        let spec = Self {
            options,
            pattern,
        };

        // 2. Validate consistency: Options must match pattern
        if let (Some(p), Some(opts)) = (spec.pattern(), spec.options()) {
            for opt in opts {
                if p.validate(opt.value()).is_err() {
                    return Err(SchemaError::PropertySpec(
                        crate::schema::error::PropertySpecError::OptionPatternMismatch {
                            value: opt.value().into(),
                            pattern: p.pattern().into(),
                        },
                    ));
                }
            }
        }

        Ok(spec)
    }

    /// Returns the allowed options for this string if defined.
    #[inline]
    #[must_use]
    pub fn options(&self) -> Option<&[OptionEntry]> {
        self.options.as_ref().map(OptionEntries::as_slice)
    }

    /// Returns the validation pattern for this string if defined.
    #[inline]
    #[must_use]
    pub fn pattern(&self) -> Option<&StringPattern> {
        self.pattern.as_ref()
    }

    /// Apply overrides from a raw string spec.
    ///
    /// # Errors
    /// Returns `SchemaError` if override values are invalid or inconsistent.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::spec_string::RawStringSpec,
    ) -> Result<Self, SchemaError> {
        let pattern = match overrides.pattern.as_ref() {
            Some(
                crate::schema::raw::spec_string::RawStringPattern::Custom(p),
            ) => Some(StringPattern::try_custom(p.clone())?),
            Some(crate::schema::raw::spec_string::RawStringPattern::Named(
                f,
            )) => Some(StringPattern::from(*f)),
            None => self.pattern,
        };

        let options = overrides
            .options
            .as_ref()
            .map(|o| {
                let entries = o
                    .clone()
                    .into_entries()
                    .into_iter()
                    .map(OptionEntry::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                OptionEntries::try_new(entries)
            })
            .transpose()?
            .or(self.options);

        Self::try_new(pattern, options)
    }

    /// Validate a runtime string value against all constraints in this spec.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub(super) fn validate(&self, value: &str) -> Result<(), SchemaError> {
        // 1. Validate options (enum check)
        if let Some(entries) = self.options.as_ref()
            && !entries.iter().any(|e| e.value() == value)
        {
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::InvalidEnumValue {
                    value: value.into(),
                    allowed: entries.iter().map(|e| e.value().into()).collect(),
                },
            ));
        }

        // 2. Validate pattern (regex/format check)
        if let Some(pattern) = self.pattern.as_ref() {
            pattern.validate(value)?;
        }

        Ok(())
    }
}

impl ArchivedStringSpec {
    /// Validate a runtime string value against all constraints in this archived
    /// spec directly from the database without deserialization.
    ///
    /// This is a zero-copy validation method that operates on the archived
    /// representation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        // 1. Validate options (enum check)
        if let Some(entries) = self.options.as_ref()
            && !entries.iter().any(|e| e.value.as_ref() == value)
        {
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::InvalidEnumValue {
                    value: value.into(),
                    allowed: entries
                        .iter()
                        .map(|e| (*e.value).into())
                        .collect(),
                },
            ));
        }

        // 2. Validate pattern (regex/format check)
        if let Some(pattern) = self.pattern.as_ref() {
            pattern.validate(value)?;
        }

        Ok(())
    }
}

impl TryFrom<crate::schema::raw::spec_string::RawStringSpec> for StringSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        raw: crate::schema::raw::spec_string::RawStringSpec,
    ) -> Result<Self, Self::Error> {
        let pattern = match raw.pattern {
            Some(
                crate::schema::raw::spec_string::RawStringPattern::Custom(p),
            ) => Some(StringPattern::try_custom(p)?),
            Some(crate::schema::raw::spec_string::RawStringPattern::Named(
                f,
            )) => Some(StringPattern::from(f)),
            None => None,
        };

        let options = raw
            .options
            .map(|o| {
                let entries = o
                    .into_entries()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?;
                OptionEntries::try_new(entries)
            })
            .transpose()?;

        Self::try_new(pattern, options)
    }
}

// ============================================================================
// Public API - Supporting Types
// ============================================================================

/// String validation pattern (predefined format or custom regex).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub enum StringPattern {
    /// Email address validation (RFC 5322 simplified).
    Email,
    /// URL validation (HTTP/HTTPS).
    Url,
    /// US phone number validation.
    PhoneUs,
    /// Slug validation (kebab-case).
    Slug,
    /// UUID v4 validation.
    UuidV4,
    /// `WikiLink` validation (Obsidian-style).
    WikiLink,
    /// US ZIP code validation (5 or 9 digits).
    ZipCode,
    /// User-defined custom regex pattern.
    Custom(Box<str>),
}

impl StringPattern {
    /// Create a custom pattern, validating that the regex is compilable.
    ///
    /// # Errors
    /// Returns `SchemaError::PropertySpec` if the pattern is invalid.
    #[inline]
    pub fn try_custom<S: Into<Box<str>>>(
        pattern: S,
    ) -> Result<Self, SchemaError> {
        let pattern = pattern.into();
        regex::Regex::new(&pattern).map_err(|e| {
            SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::InvalidRegex {
                    pattern: pattern.clone(),
                    reason: e.to_string().into(),
                },
            )
        })?;
        Ok(Self::Custom(pattern))
    }

    /// Returns the regex pattern string for this pattern.
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
    /// Returns `SchemaError::PropertyValue` if the value doesn't match the
    /// pattern.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        let matches = match self {
            Self::Email => Self::static_regex_email().is_match(value),
            Self::Url => Self::static_regex_url().is_match(value),
            Self::PhoneUs => Self::static_regex_phone_us().is_match(value),
            Self::Slug => Self::static_regex_slug().is_match(value),
            Self::UuidV4 => Self::static_regex_uuid_v4().is_match(value),
            Self::WikiLink => Self::static_regex_wikilink().is_match(value),
            Self::ZipCode => Self::static_regex_zipcode().is_match(value),
            Self::Custom(pattern) => {
                Self::get_or_compile_pattern(pattern).is_match(value)
            }
        };

        if !matches {
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::PatternMismatch {
                    value: value.into(),
                    pattern: self.pattern().into(),
                },
            ));
        }
        Ok(())
    }

    /// Get or compile a custom regex pattern from an internal global cache.
    #[expect(
        clippy::expect_used,
        reason = "Pattern validated at StringPattern construction, expect \
                  documents invariant"
    )]
    fn get_or_compile_pattern(pattern: &str) -> Arc<regex::Regex> {
        use std::{
            collections::HashMap,
            sync::{OnceLock, RwLock},
        };

        type CacheMap = HashMap<Box<str>, Arc<regex::Regex>>;
        static CUSTOM_PATTERN_CACHE: OnceLock<RwLock<CacheMap>> =
            OnceLock::new();

        let cache =
            CUSTOM_PATTERN_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        // Fast path: read lock
        {
            let guard =
                cache.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(re) = guard.get(pattern) {
                return Arc::clone(re);
            }
        }

        // Slow path: compile and cache
        let compiled = Arc::new(regex::Regex::new(pattern).expect(
            "Custom pattern should be valid (validated at construction)",
        ));

        let mut guard =
            cache.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(re) = guard.get(pattern) {
            return Arc::clone(re);
        }
        guard.insert(pattern.into(), Arc::clone(&compiled));
        compiled
    }

    // Static regex accessors for predefined formats
    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_email() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::Email.pattern())
                .expect("built-in Email regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_url() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::Url.pattern())
                .expect("built-in Url regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_phone_us() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::PhoneUs.pattern())
                .expect("built-in PhoneUs regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_slug() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::Slug.pattern())
                .expect("built-in Slug regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_uuid_v4() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::UuidV4.pattern())
                .expect("built-in UuidV4 regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_wikilink() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::WikiLink.pattern())
                .expect("built-in WikiLink regex pattern is valid")
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    fn static_regex_zipcode() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            regex::Regex::new(StringPattern::ZipCode.pattern())
                .expect("built-in ZipCode regex pattern is valid")
        })
    }
}

impl ArchivedStringPattern {
    /// Validate a string value against this archived pattern directly from the
    /// database without deserialization.
    ///
    /// This is a zero-copy validation method that operates on the archived
    /// representation.
    ///
    /// # Errors
    /// Returns `SchemaError::PropertyValue` if the value doesn't match the
    /// pattern.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &Self with value patterns is more readable than \
                  matching *self"
    )]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        let matches = match self {
            Self::Email => StringPattern::static_regex_email().is_match(value),
            Self::Url => StringPattern::static_regex_url().is_match(value),
            Self::PhoneUs => {
                StringPattern::static_regex_phone_us().is_match(value)
            }
            Self::Slug => StringPattern::static_regex_slug().is_match(value),
            Self::UuidV4 => {
                StringPattern::static_regex_uuid_v4().is_match(value)
            }
            Self::WikiLink => {
                StringPattern::static_regex_wikilink().is_match(value)
            }
            Self::ZipCode => {
                StringPattern::static_regex_zipcode().is_match(value)
            }
            Self::Custom(pattern) => {
                StringPattern::get_or_compile_pattern(pattern.as_ref())
                    .is_match(value)
            }
        };

        if !matches {
            return Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::PatternMismatch {
                    value: value.into(),
                    pattern: "archived".into(),
                },
            ));
        }
        Ok(())
    }
}

impl From<crate::schema::raw::spec_string::RawStringFormat> for StringPattern {
    #[inline]
    fn from(format: crate::schema::raw::spec_string::RawStringFormat) -> Self {
        use crate::schema::raw::spec_string::RawStringFormat;
        match format {
            RawStringFormat::Email => Self::Email,
            RawStringFormat::Url => Self::Url,
            RawStringFormat::PhoneUs => Self::PhoneUs,
            RawStringFormat::Slug => Self::Slug,
            RawStringFormat::UuidV4 => Self::UuidV4,
            RawStringFormat::WikiLink => Self::WikiLink,
            RawStringFormat::ZipCode => Self::ZipCode,
        }
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
#[deprecated(
    since = "0.1.0",
    note = "Use StringPattern instead. StringFormat has been unified with \
            custom patterns."
)]
pub type StringFormat = StringPattern;

/// A validated option entry with optional display label.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub struct OptionEntry {
    /// The option value used in validation.
    value: Box<str>,
    /// Optional display label for UI consumers.
    label: Option<Box<str>>,
}

/// A wrapper for ordered option entries with helper sorting APIs.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub struct OptionEntries(Box<[OptionEntry]>);

impl OptionEntries {
    /// Create a new `OptionEntries`, validating non-empty constraints.
    ///
    /// # Errors
    /// Returns `SchemaError` if the entries list is empty.
    #[inline]
    pub fn try_new(entries: Vec<OptionEntry>) -> Result<Self, SchemaError> {
        if entries.is_empty() {
            return Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::OptionsEmpty,
            ));
        }
        Ok(Self(entries.into_boxed_slice()))
    }

    /// Returns the ordered entries as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[OptionEntry] {
        &self.0
    }

    /// Iterate over entries in stored order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, OptionEntry> {
        self.0.iter()
    }

    /// Returns entries sorted by label (fallback to value when label missing).
    #[must_use]
    pub fn sorted_by_label(&self) -> Vec<&OptionEntry> {
        let mut items: Vec<_> = self.0.iter().collect();
        items.sort_by(|a, b| {
            let a_key = a.label().unwrap_or_else(|| a.value());
            let b_key = b.label().unwrap_or_else(|| b.value());
            a_key.cmp(b_key)
        });
        items
    }

    /// Returns entries sorted by value.
    #[must_use]
    pub fn sorted_by_value(&self) -> Vec<&OptionEntry> {
        let mut items: Vec<_> = self.0.iter().collect();
        items.sort_by(|a, b| a.value().cmp(b.value()));
        items
    }

    /// Returns true if there are no entries.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ArchivedOptionEntries {
    /// Iterate over archived entries in stored order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, ArchivedOptionEntry> {
        self.0.iter()
    }

    /// Returns the ordered archived entries as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[ArchivedOptionEntry] {
        self.0.as_ref()
    }
}

impl OptionEntry {
    /// Create a new `OptionEntry`, validating the value.
    ///
    /// # Errors
    /// Returns `SchemaError` if the value is empty or only whitespace.
    #[inline]
    pub fn try_new<V: Into<Box<str>>>(
        value: V,
        label: Option<Box<str>>,
    ) -> Result<Self, SchemaError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SchemaError::PropertySpec(
                crate::schema::error::PropertySpecError::OptionValueEmpty,
            ));
        }
        Ok(Self {
            value,
            label,
        })
    }

    /// Returns the option value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the display label if set.
    #[inline]
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

impl TryFrom<crate::schema::raw::spec_string::RawEntryLabeled> for OptionEntry {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        raw: crate::schema::raw::spec_string::RawEntryLabeled,
    ) -> Result<Self, Self::Error> {
        Self::try_new(raw.value, raw.label)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(
        clippy::indexing_slicing,
        clippy::missing_asserts_for_indexing,
        reason = "Test code: indexing is safe after known-length construction"
    )]
    mod option_entries {
        use super::*;

        #[test]
        fn preserves_order_and_sorts_by_label() {
            let entries = OptionEntries::try_new(vec![
                OptionEntry::try_new("b", Some("bee".into())).unwrap(),
                OptionEntry::try_new("a", None).unwrap(),
                OptionEntry::try_new("c", Some("see".into())).unwrap(),
            ])
            .unwrap();

            let ordered = entries.as_slice();
            assert_eq!(ordered[0].value(), "b");
            assert_eq!(ordered[1].value(), "a");
            assert_eq!(ordered[2].value(), "c");

            let sorted = entries.sorted_by_label();
            assert_eq!(sorted[0].value(), "a");
            assert_eq!(sorted[1].value(), "b");
            assert_eq!(sorted[2].value(), "c");
        }

        #[test]
        fn sorts_by_value() {
            let entries = OptionEntries::try_new(vec![
                OptionEntry::try_new("z", None).unwrap(),
                OptionEntry::try_new("m", None).unwrap(),
                OptionEntry::try_new("a", None).unwrap(),
            ])
            .unwrap();

            let sorted = entries.sorted_by_value();
            assert_eq!(sorted[0].value(), "a");
            assert_eq!(sorted[1].value(), "m");
            assert_eq!(sorted[2].value(), "z");
        }

        #[test]
        fn rejects_empty_entries() {
            let result = OptionEntries::try_new(vec![]);
            assert!(matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::OptionsEmpty
                ))
            ));
        }
    }

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
        use crate::schema::raw::spec_string::{
            RawOptions, RawOptionsPlain, RawStringSpec,
        };

        #[rstest]
        #[case::options_match(
            RawStringSpec {
                options: Some(RawOptions::Plain(RawOptionsPlain::from(vec![
                    "A".into(),
                    "B".into(),
                ]))),
                ..Default::default()
            },
            "A",
            Ok(())
        )]
        #[case::options_mismatch(
            RawStringSpec {
                options: Some(RawOptions::Plain(RawOptionsPlain::from(vec![
                    "A".into(),
                    "B".into(),
                ]))),
                ..Default::default()
            },
            "C",
            Err(SchemaError::PropertyValue(
                crate::schema::error::PropertyValueError::InvalidEnumValue {
                    value: "C".into(),
                    allowed: vec!["A".into(), "B".into()]
                }
            ))
        )]
        #[case::regex_match(
            RawStringSpec { pattern: Some(crate::schema::raw::spec_string::RawStringPattern::Custom(r"^\d+$".into())), ..Default::default() },
            "123",
            Ok(())
        )]
        fn string_spec_validation_matrix(
            #[case] def: RawStringSpec,
            #[case] value: &str,
            #[case] expected: Result<(), SchemaError>,
        ) {
            let spec = StringSpec::default().apply_overrides(&def).unwrap();
            let result = spec.validate(value);
            assert_eq!(result, expected);
        }

        #[test]
        fn empty_spec_accepts_any_string() {
            let spec = StringSpec::try_new(None, None).unwrap();
            let result = spec.validate("any arbitrary string");
            result.unwrap();
        }

        #[test]
        fn try_new_rejects_inconsistent_options() {
            let pattern = StringPattern::try_custom(r"^\d+$").unwrap();
            let options = OptionEntries::try_new(vec![
                OptionEntry::try_new("not-a-number", None).unwrap(),
            ])
            .unwrap();

            let result = StringSpec::try_new(Some(pattern), Some(options));
            assert!(matches!(
                result,
                Err(SchemaError::PropertySpec(
                    crate::schema::error::PropertySpecError::OptionPatternMismatch { .. }
                ))
            ));
        }
    }
}
