//! Frontmatter configuration types.

#![allow(clippy::struct_field_names, reason = "Fields share '_key' suffix")]

use super::error::ConfigError;

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
    String,
);

impl FrontmatterKey {
    /// Create a validated frontmatter key.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the key is empty.
    #[inline]
    pub fn try_new<T: Into<String>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        validate_non_empty("frontmatter_key", &value)?;
        Ok(Self(value))
    }

    pub(crate) fn try_new_with_field(
        field: &'static str,
        value: impl Into<String>,
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

/// Frontmatter configuration with validation.
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
    /// Key used for aliases in frontmatter.
    alias_key: FrontmatterKey,
    /// Key used for creation date in frontmatter.
    date_created_key: FrontmatterKey,
    /// Key used for modification date in frontmatter.
    date_modified_key: FrontmatterKey,
    /// Key used for file class/type in frontmatter.
    file_class_key: FrontmatterKey,
    /// Key used for title in frontmatter.
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
    pub const fn alias_key(&self) -> &FrontmatterKey {
        &self.alias_key
    }

    /// Return the date created key.
    #[inline]
    #[must_use]
    pub const fn date_created_key(&self) -> &FrontmatterKey {
        &self.date_created_key
    }

    /// Return the date modified key.
    #[inline]
    #[must_use]
    pub const fn date_modified_key(&self) -> &FrontmatterKey {
        &self.date_modified_key
    }

    /// Return the file class key.
    #[inline]
    #[must_use]
    pub const fn file_class_key(&self) -> &FrontmatterKey {
        &self.file_class_key
    }

    /// Return the title key.
    #[inline]
    #[must_use]
    pub const fn title_key(&self) -> &FrontmatterKey {
        &self.title_key
    }
}

impl Default for Frontmatter {
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        clippy::expect_used,
        reason = "Default values are guaranteed to be valid"
    )]
    fn default() -> Self {
        Self {
            alias_key: FrontmatterKey::try_new("aliases")
                .expect("default alias key must be valid"),
            date_created_key: FrontmatterKey::try_new("date_created")
                .expect("default created key must be valid"),
            date_modified_key: FrontmatterKey::try_new("date_modified")
                .expect("default modified key must be valid"),
            file_class_key: FrontmatterKey::try_new("file_class")
                .expect("default file class key must be valid"),
            title_key: FrontmatterKey::try_new("title")
                .expect("default title key must be valid"),
        }
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
    fn from(key: FrontmatterKey) -> Self {
        key.0
    }
}

/// Raw frontmatter configuration (unvalidated input).
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
    fn try_from(raw: RawFrontmatter) -> Result<Self, ConfigError> {
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

        Ok(Self {
            alias_key,
            date_created_key,
            date_modified_key,
            file_class_key,
            title_key,
        })
    }
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test modules have relaxed unwrap/expect rules"
)]
mod tests {
    mod fixtures {
        use super::super::FrontmatterKey;

        pub fn sample_key() -> FrontmatterKey {
            FrontmatterKey::try_new("author").expect("valid key for fixture")
        }
    }

    mod constructor {
        use super::super::*;

        /// 3.3-UNIT-026: `frontmatter_validate_rejects_empty_keys`.
        /// Priority: P0.
        #[test]
        fn frontmatter_key_rejects_empty() {
            let result = FrontmatterKey::try_new("");
            assert!(result.is_err(), "Expected validation error");
        }
    }

    mod accessors {
        #[test]
        fn frontmatter_key_as_str_returns_expected() {
            let key = super::fixtures::sample_key();
            assert_eq!(key.as_str(), "author");
        }
    }
}
