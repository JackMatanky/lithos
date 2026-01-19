//! Predefined regex patterns for domain validation.
//!
//! This module provides reusable regex pattern constants that can be used
//! across all bounded contexts (Config, Note, Schema, Template) for consistent
//! validation of common data formats.
//!
//! # Visibility
//! All items are `pub(crate)` - internal to domain crate only. These patterns
//! are not part of the public API.
//!
//! # Available Patterns
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

/// Basic email validation pattern.
pub(crate) const EMAIL: &str = r"^[^@]+@[^@]+\.[^@]+$";
/// Basic URL validation pattern.
pub(crate) const URL: &str = r"^https?://[^\s/$.?#].[^\s]*$";
/// `WikiLink` validation pattern.
pub(crate) const WIKILINK: &str = r"^\[\[([^\]|]+)(\|[^\]]+)?\]\]$";
/// UUID v4 validation pattern.
pub(crate) const UUID_V4: &str =
    "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
/// Slug validation pattern (kebab-case).
pub(crate) const SLUG: &str = "^[a-z0-9]+(-[a-z0-9]+)*$";
/// US Phone number validation pattern.
pub(crate) const PHONE_US: &str =
    r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$";
/// US Zip code validation pattern.
pub(crate) const ZIP_CODE: &str = r"^\d{5}(-\d{4})?$";
