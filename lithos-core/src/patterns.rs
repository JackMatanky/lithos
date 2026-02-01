//! Predefined regex patterns for domain validation.
//!
//! This module provides reusable regex pattern constants that can be used
//! across all bounded contexts (Config, Note, Schema, Template) for consistent
//! validation of common data formats.
//!
//! # Visibility
//! All items are `pub`.
//!
//! # Available Patterns
//!
//! ## Domain-Specific Patterns (Currently Used)
//! - **`ALPHANUMERIC_NAME`**: Name validation for schemas, properties, and
//!   templates
//! - **`IDENTIFIER_NAME`**: Variable name validation (programming identifier
//!   style)
//!
//! ## General Patterns (Future Use)
//! - Email validation
//! - URL validation
//! - `WikiLink` validation (Obsidian-style links)
//! - UUID v4 validation
//! - Slug validation (kebab-case)
//! - US Phone number validation
//! - US Zip code validation

#![allow(
    dead_code,
    reason = "Patterns provided for future use across contexts"
)]

// ============================================================================
// Domain-Specific Patterns (Actively Used)
// ============================================================================

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

// ============================================================================
// General Patterns (Available for Future Use)
// ============================================================================

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
