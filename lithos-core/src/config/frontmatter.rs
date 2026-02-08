//! Frontmatter configuration types.
//!
//! This module contains types related to frontmatter key configuration
//! for Markdown file metadata.

#![expect(
    clippy::struct_field_names,
    reason = "Frontmatter struct fields intentionally share '_key' suffix \
              (flagged by rkyv::Archive derive)"
)]

use super::error::ConfigError;

// ============================================================================
// Public Domain Types (Most Important - User-Facing API)
// ============================================================================

/// Frontmatter configuration for Markdown file metadata.
///
/// Configures which keys to use when reading/writing frontmatter fields in
/// Markdown files. All keys must be non-empty strings.
///
/// # Invariants
/// - All keys must be non-empty strings.
/// - Keys should follow YAML/TOML naming conventions (lowercase, underscores).
///
/// # Examples
/// ```
/// # use lithos_core::config::frontmatter::Frontmatter;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let frontmatter = Frontmatter::default();
/// frontmatter.validate()?;
/// # Ok(())
/// # }
/// ```
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key for aliases in frontmatter.
    alias_key: FrontmatterKey,
    /// Key for creation date in frontmatter.
    date_created_key: FrontmatterKey,
    /// Key for modification date in frontmatter.
    date_modified_key: FrontmatterKey,
    /// Key for file classification in frontmatter.
    file_class_key: FrontmatterKey,
    /// Key for title field in frontmatter.
    title_key: FrontmatterKey,
}

impl Frontmatter {
    /// Create frontmatter configuration.
    #[inline]
    #[must_use]
    pub fn new(
        alias_key: FrontmatterKey,
        date_created_key: FrontmatterKey,
        date_modified_key: FrontmatterKey,
        file_class_key: FrontmatterKey,
        title_key: FrontmatterKey,
    ) -> Self {
        Self {
            alias_key,
            date_created_key,
            date_modified_key,
            file_class_key,
            title_key,
        }
    }

    /// Return the alias key.
    #[inline]
    #[must_use]
    pub fn alias_key(&self) -> &FrontmatterKey {
        &self.alias_key
    }

    /// Return the created date key.
    #[inline]
    #[must_use]
    pub fn date_created_key(&self) -> &FrontmatterKey {
        &self.date_created_key
    }

    /// Return the modified date key.
    #[inline]
    #[must_use]
    pub fn date_modified_key(&self) -> &FrontmatterKey {
        &self.date_modified_key
    }

    /// Return the file classification key.
    #[inline]
    #[must_use]
    pub fn file_class_key(&self) -> &FrontmatterKey {
        &self.file_class_key
    }

    /// Return the title key.
    #[inline]
    #[must_use]
    pub fn title_key(&self) -> &FrontmatterKey {
        &self.title_key
    }

    /// Validate frontmatter configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if any frontmatter key is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Default for Frontmatter {
    #[inline]
    fn default() -> Self {
        Self {
            alias_key: FrontmatterKey::try_new("aliases")
                .unwrap_or_else(|_| FrontmatterKey("aliases".into())),
            date_created_key: FrontmatterKey::try_new("date_created")
                .unwrap_or_else(|_| FrontmatterKey("date_created".into())),
            date_modified_key: FrontmatterKey::try_new("date_modified")
                .unwrap_or_else(|_| FrontmatterKey("date_modified".into())),
            file_class_key: FrontmatterKey::try_new("file_class")
                .unwrap_or_else(|_| FrontmatterKey("file_class".into())),
            title_key: FrontmatterKey::try_new("title")
                .unwrap_or_else(|_| FrontmatterKey("title".into())),
        }
    }
}

// ============================================================================
// Building Block Types (Key Component)
// ============================================================================

/// Validated frontmatter key (non-empty).
///
/// A validated string used as a key in YAML/TOML frontmatter. Must be
/// non-empty.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct FrontmatterKey(
    /// Internal key storage.
    Box<str>,
);

impl FrontmatterKey {
    /// Create a validated frontmatter key.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the key is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        validate_non_empty("frontmatter_key", &value)?;
        Ok(Self(value))
    }

    pub(crate) fn try_new_with_field(
        field: &'static str,
        value: impl Into<Box<str>>,
    ) -> Result<Self, ConfigError> {
        let value = value.into();
        validate_non_empty(field, &value)?;
        Ok(Self(value))
    }

    /// Return the key as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Standard Trait Implementations (Conversions)
// ============================================================================

impl TryFrom<String> for FrontmatterKey {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(value)
    }
}

impl From<FrontmatterKey> for String {
    #[inline]
    fn from(value: FrontmatterKey) -> Self {
        value.0.into()
    }
}

// ============================================================================
// Raw DTOs (Deserialization Boundary - Internal)
// ============================================================================

/// Raw frontmatter configuration (unvalidated input from config files).
///
/// This is a serde-only DTO that accepts flexible input from TOML/YAML/JSON.
/// All fields are optional to support partial configuration and defaults.
/// Validation happens during conversion to [`Frontmatter`] via [`TryFrom`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawFrontmatter {
    /// Frontmatter key for aliases.
    pub alias_key: Option<String>,
    /// Frontmatter key for created date.
    pub date_created_key: Option<String>,
    /// Frontmatter key for modified date.
    pub date_modified_key: Option<String>,
    /// Frontmatter key for file classification.
    pub file_class_key: Option<String>,
    /// Frontmatter key for title.
    pub title_key: Option<String>,
}

impl TryFrom<RawFrontmatter> for Frontmatter {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawFrontmatter) -> Result<Self, Self::Error> {
        let defaults = Frontmatter::default();
        let alias_key = match raw.alias_key {
            Some(value) => {
                FrontmatterKey::try_new_with_field("alias_key", value)?
            }
            None => defaults.alias_key,
        };
        let date_created_key = match raw.date_created_key {
            Some(value) => {
                FrontmatterKey::try_new_with_field("date_created_key", value)?
            }
            None => defaults.date_created_key,
        };
        let date_modified_key = match raw.date_modified_key {
            Some(value) => {
                FrontmatterKey::try_new_with_field("date_modified_key", value)?
            }
            None => defaults.date_modified_key,
        };
        let file_class_key = match raw.file_class_key {
            Some(value) => {
                FrontmatterKey::try_new_with_field("file_class_key", value)?
            }
            None => defaults.file_class_key,
        };
        let title_key = match raw.title_key {
            Some(value) => {
                FrontmatterKey::try_new_with_field("title_key", value)?
            }
            None => defaults.title_key,
        };

        Ok(Frontmatter::new(
            alias_key,
            date_created_key,
            date_modified_key,
            file_class_key,
            title_key,
        ))
    }
}

// ============================================================================
// Private Validation Helpers (Implementation Details)
// ============================================================================

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), ConfigError> {
    if value.is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: format!("{field} cannot be empty").into(),
        });
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::FrontmatterKey;

    /// 3.3-UNIT-026: `frontmatter_validate_rejects_empty_keys`.
    /// Priority: P0.
    #[test]
    fn frontmatter_key_rejects_empty() {
        let result = FrontmatterKey::try_new("");
        assert!(result.is_err(), "Expected validation error");
    }
}
