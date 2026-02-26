//! Predefined regex patterns for domain validation across bounded contexts.

// ----------------------------------------------------------- //
//                  Domain-Specific Patterns                   //
// ----------------------------------------------------------- //

/// Alphanumeric name pattern: letters, numbers, underscores, and dashes.
///
/// Used by:
/// - Template names (`Template::validate_name`)
/// - Schema names (`SchemaName::validate_format`)
/// - Property names (`PropertyName::validate_format`)
///
/// Pattern: `^[a-zA-Z0-9_-]+$`.
///
/// # Examples
/// - Valid: `daily-note`, `project_schema`, `MyTemplate123`
/// - Invalid: `invalid name`, `name!`, `name.txt`
pub const ALPHANUMERIC_NAME: &str = "^[a-zA-Z0-9_-]+$";

/// Lowercase alphanumeric name pattern: lowercase letters, numbers,
/// underscores, and dashes.
///
/// Used by:
/// - Schema names (`SchemaName::validate_format`)
///
/// Pattern: `^[a-z0-9_-]+$`.
///
/// # Examples
/// - Valid: `daily-note`, `project_schema`, `schema123`
/// - Invalid: `MySchema`, `invalid name`, `name!`
pub const ALPHANUMERIC_NAME_LOWER: &str = "^[a-z0-9_-]+$";

/// Basic email validation pattern.
pub const EMAIL: &str = r"^[^@]+@[^@]+\.[^@]+$";

/// Identifier name pattern: programming-style identifiers.
///
/// Used by:
/// - Template variable names (`Template::validate_variable_name`)
///
/// Pattern: `^[a-zA-Z_][a-zA-Z0-9_]*$`.
///
/// # Examples
/// - Valid: `title`, `my_var`, `_private`, `camelCase`
/// - Invalid: `123var`, `my-var`, `var!`
pub const IDENTIFIER_NAME: &str = "^[a-zA-Z_][a-zA-Z0-9_]*$";

/// Property name pattern: mixed-case letters, underscores, and hyphens.
///
/// Used by:
/// - Property names (`PropertyName::validate`)
///
/// Pattern: `^[A-Za-z_][A-Za-z0-9_-]*$`.
///
/// Must start with a letter (uppercase or lowercase) or underscore.
/// May contain letters, digits, underscores, and hyphens.
///
/// # Examples
/// - Valid: `status`, `MyProperty`, `_internal`, `tag-name`, `Priority1`
/// - Invalid: `123prop`, `-prop`, `prop!`, `my prop`
pub const PROPERTY_NAME: &str = "^[A-Za-z_][A-Za-z0-9_-]*$";

/// US Phone number validation pattern.
pub const PHONE_US: &str =
    r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$";

/// Slug validation pattern (kebab-case).
pub const SLUG: &str = "^[a-z0-9]+(-[a-z0-9]+)*$";

/// Basic URL validation pattern.
pub const URL: &str = r"^https?://[^\s/$.?#].[^\s]*$";

/// UUID v4 validation pattern.
pub const UUID_V4: &str =
    "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";

/// `WikiLink` validation pattern (Obsidian-style).
pub const WIKILINK: &str = r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$";

/// US Zip code validation pattern.
pub const ZIP_CODE: &str = r"^\d{5}(-\d{4})?$";
