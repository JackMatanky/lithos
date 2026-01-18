//! Predefined regex patterns for schema validation.

/// Basic email validation pattern.
pub const EMAIL: &str = r"^[^@]+@[^@]+\.[^@]+$";
/// Basic URL validation pattern.
pub const URL: &str = r"^https?://[^\s/$.?#].[^\s]*$";
/// `WikiLink` validation pattern.
pub const WIKILINK: &str = r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$";
/// UUID v4 validation pattern.
pub const UUID_V4: &str =
    "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
/// Slug validation pattern (kebab-case).
pub const SLUG: &str = "^[a-z0-9]+(-[a-z0-9]+)*$";
/// US Phone number validation pattern.
pub const PHONE_US: &str =
    r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$";
/// US Zip code validation pattern.
pub const ZIP_CODE: &str = r"^\d{5}(-\d{4})?$";
