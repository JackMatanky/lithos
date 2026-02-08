//! Frontmatter configuration types.
//!
//! This module contains types related to frontmatter key configuration
//! for Markdown file metadata.

#![expect(
    clippy::struct_field_names,
    reason = "Frontmatter struct fields intentionally share '_key' suffix \
              (flagged by rkyv::Archive derive)"
)]

use super::{error::ConfigError, raw::RawFrontmatter};

/// Validated frontmatter key (non-empty).
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

/// Frontmatter configuration for Markdown file metadata.
///
/// # Invariants
/// - All keys must be non-empty strings.
/// - Keys should follow YAML/TOML naming conventions (lowercase, underscores).
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

    #[inline]
    #[must_use]
    /// Return the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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

impl Frontmatter {
    #[inline]
    #[must_use]
    /// Create frontmatter configuration.
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

    #[inline]
    #[must_use]
    /// Return the alias key.
    pub fn alias_key(&self) -> &FrontmatterKey {
        &self.alias_key
    }

    #[inline]
    #[must_use]
    /// Return the created date key.
    pub fn date_created_key(&self) -> &FrontmatterKey {
        &self.date_created_key
    }

    #[inline]
    #[must_use]
    /// Return the modified date key.
    pub fn date_modified_key(&self) -> &FrontmatterKey {
        &self.date_modified_key
    }

    #[inline]
    #[must_use]
    /// Return the file classification key.
    pub fn file_class_key(&self) -> &FrontmatterKey {
        &self.file_class_key
    }

    #[inline]
    #[must_use]
    /// Return the title key.
    pub fn title_key(&self) -> &FrontmatterKey {
        &self.title_key
    }

    /// Validate frontmatter configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if any frontmatter key is
    /// empty.
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
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
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
