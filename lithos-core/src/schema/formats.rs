//! User-facing string format specifications.
//!
//! Provides named validation patterns for common string formats, allowing
//! users to reference `"email"` instead of writing raw regex patterns.

#![allow(
    clippy::exhaustive_enums,
    clippy::missing_inline_in_public_items,
    reason = "rkyv Archive derive generates exhaustive archived types; const \
              fn pattern/name are trivial getters"
)]

use std::sync::OnceLock;

use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};

/// Named string formats for common validation patterns.
///
/// These formats are mutually exclusive with custom `pattern` field in
/// `StringSpec`. Users can reference these by name (e.g., `"email"`, `"url"`)
/// instead of writing raw regex patterns.
///
/// # Examples
///
/// ```
/// use lithos_core::schema::formats::StringFormat;
///
/// let email_format = StringFormat::Email;
/// assert!(email_format.pattern().contains("@"));
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
#[non_exhaustive]
pub enum StringFormat {
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
}

impl StringFormat {
    /// Returns the regex pattern for this format.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::formats::StringFormat;
    ///
    /// assert_eq!(StringFormat::Slug.pattern(), "^[a-z0-9]+(-[a-z0-9]+)*$");
    /// ```
    #[must_use]
    pub const fn pattern(self) -> &'static str {
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
        }
    }

    /// Returns the human-readable name of this format.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::formats::StringFormat;
    ///
    /// assert_eq!(StringFormat::Email.name(), "email");
    /// assert_eq!(StringFormat::PhoneUs.name(), "phone_us");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Url => "url",
            Self::PhoneUs => "phone_us",
            Self::Slug => "slug",
            Self::UuidV4 => "uuid_v4",
            Self::WikiLink => "wikilink",
            Self::ZipCode => "zipcode",
        }
    }

    /// Returns a pre-compiled `Regex` for this format.
    ///
    /// Regexes are compiled once and cached in static `OnceLock` cells for
    /// zero runtime overhead on subsequent calls.
    ///
    /// # Panics
    /// Panics if the format's built-in pattern is invalid.
    /// This should never happen and indicates a bug in the pattern definitions.
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Built-in patterns are hardcoded and tested; panic indicates \
                  programmer error"
    )]
    pub fn regex(self) -> &'static regex::Regex {
        static EMAIL_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static URL_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static PHONE_US_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static SLUG_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static UUID_V4_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static WIKILINK_REGEX: OnceLock<regex::Regex> = OnceLock::new();
        static ZIPCODE_REGEX: OnceLock<regex::Regex> = OnceLock::new();

        match self {
            Self::Email => EMAIL_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in Email regex pattern is valid")
            }),
            Self::Url => URL_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in Url regex pattern is valid")
            }),
            Self::PhoneUs => PHONE_US_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in PhoneUs regex pattern is valid")
            }),
            Self::Slug => SLUG_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in Slug regex pattern is valid")
            }),
            Self::UuidV4 => UUID_V4_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in UuidV4 regex pattern is valid")
            }),
            Self::WikiLink => WIKILINK_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in WikiLink regex pattern is valid")
            }),
            Self::ZipCode => ZIPCODE_REGEX.get_or_init(|| {
                regex::Regex::new(self.pattern())
                    .expect("built-in ZipCode regex pattern is valid")
            }),
        }
    }
}

impl std::fmt::Display for StringFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_format_has_correct_pattern() {
        assert_eq!(StringFormat::Email.pattern(), r"^[^@]+@[^@]+\.[^@]+$");
    }

    #[test]
    fn url_format_has_correct_pattern() {
        assert_eq!(
            StringFormat::Url.pattern(),
            r"^https?://[^\s/$.?#].[^\s]*$"
        );
    }

    #[test]
    fn phone_us_format_has_correct_pattern() {
        assert_eq!(
            StringFormat::PhoneUs.pattern(),
            r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$"
        );
    }

    #[test]
    fn slug_format_has_correct_pattern() {
        assert_eq!(StringFormat::Slug.pattern(), "^[a-z0-9]+(-[a-z0-9]+)*$");
    }

    #[test]
    fn uuid_v4_format_has_correct_pattern() {
        assert_eq!(
            StringFormat::UuidV4.pattern(),
            "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
        );
    }

    #[test]
    fn wikilink_format_has_correct_pattern() {
        assert_eq!(
            StringFormat::WikiLink.pattern(),
            r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$"
        );
    }

    #[test]
    fn zipcode_format_has_correct_pattern() {
        assert_eq!(StringFormat::ZipCode.pattern(), r"^\d{5}(-\d{4})?$");
    }

    #[test]
    fn format_name_matches_serde_representation() {
        assert_eq!(StringFormat::Email.name(), "email");
        assert_eq!(StringFormat::PhoneUs.name(), "phone_us");
    }

    #[test]
    fn format_display_uses_name() {
        assert_eq!(StringFormat::Email.to_string(), "email");
        assert_eq!(StringFormat::Slug.to_string(), "slug");
    }
}
