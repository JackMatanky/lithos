//! Path configuration types.
//!
//! This module contains types for validated configuration paths,
//! including schemas directory, templates directory, and file names.

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived structs"
)]

use std::path::{Path, PathBuf};

use super::error::ConfigError;

// ============================================================================
// Public Domain Types (Most Important - User-Facing API)
// ============================================================================

/// Schema configuration (schemas directory + property bank filename).
///
/// Validated configuration for schema file storage. All paths are
/// vault-relative.
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
pub struct Schema {
    /// Directory containing schema files.
    pub schemas_dir: Option<RelativePath>,
    /// Property bank filename (stored in `schemas_dir`).
    pub property_bank_filename: Option<FileName>,
}

impl Default for Schema {
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        clippy::expect_used,
        reason = "Default values are guaranteed to be valid"
    )]
    fn default() -> Self {
        Self {
            schemas_dir: Some(RelativePath(PathBuf::from("schemas"))),
            property_bank_filename: Some(
                FileName::try_new("property_bank.json")
                    .expect("default filename must be valid"),
            ),
        }
    }
}

impl Schema {
    /// Create schema configuration.
    #[inline]
    #[must_use]
    pub const fn new(
        schemas_dir: Option<RelativePath>,
        property_bank_filename: Option<FileName>,
    ) -> Self {
        Self {
            schemas_dir,
            property_bank_filename,
        }
    }

    /// Return the schemas directory, if set.
    #[inline]
    #[must_use]
    pub fn schemas_dir(&self) -> Option<&RelativePath> {
        self.schemas_dir.as_ref()
    }

    /// Return the property bank file name, if set.
    #[inline]
    #[must_use]
    pub fn property_bank_filename(&self) -> Option<&FileName> {
        self.property_bank_filename.as_ref()
    }

    /// Get the full path to the property bank file.
    ///
    /// # Panics
    /// Panics if either `schemas_dir` or `property_bank_filename` is missing.
    /// This should only be called on resolved configurations.
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Internal invariant for resolved config"
    )]
    pub fn property_bank_path(&self) -> PathBuf {
        self.schemas_dir.as_ref().expect("resolved schemas_dir").as_path().join(
            self.property_bank_filename
                .as_ref()
                .expect("resolved property_bank_filename")
                .as_str(),
        )
    }

    /// Validate schema configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(dir) = self.schemas_dir.as_ref() {
            dir.validate()?;
        }
        Ok(())
    }
}

/// Template configuration (templates directory).
///
/// Validated configuration for template storage. All paths are vault-relative.
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
pub struct Template {
    /// Directory containing template files.
    pub templates_dir: Option<RelativePath>,
}

impl Default for Template {
    #[inline]
    fn default() -> Self {
        Self {
            templates_dir: Some(RelativePath(PathBuf::from("templates"))),
        }
    }
}

impl Template {
    /// Create template configuration.
    #[inline]
    #[must_use]
    pub const fn new(templates_dir: Option<RelativePath>) -> Self {
        Self {
            templates_dir,
        }
    }

    /// Return the templates directory, if set.
    #[inline]
    #[must_use]
    pub fn templates_dir(&self) -> Option<&RelativePath> {
        self.templates_dir.as_ref()
    }

    /// Validate template configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(dir) = self.templates_dir.as_ref() {
            dir.validate()?;
        }
        Ok(())
    }
}

/// Cache configuration (cache directory).
///
/// Validated configuration for cache storage. All paths are vault-relative.
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
pub struct Cache {
    /// Directory containing cache files.
    pub cache_dir: Option<RelativePath>,
}

impl Default for Cache {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: Some(RelativePath(PathBuf::from(".cache"))),
        }
    }
}

impl Cache {
    /// Create cache configuration.
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: Option<RelativePath>) -> Self {
        Self {
            cache_dir,
        }
    }

    /// Return the cache directory, if set.
    #[inline]
    #[must_use]
    pub fn cache_dir(&self) -> Option<&RelativePath> {
        self.cache_dir.as_ref()
    }

    /// Validate cache configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(dir) = self.cache_dir.as_ref() {
            dir.validate()?;
        }
        Ok(())
    }
}

// ============================================================================
// Building Block Types (Path Components)
// ============================================================================

/// A validated vault-relative path.
///
/// Validated path that must be:
/// - Non-empty
/// - Vault-relative (not absolute)
/// - No parent directory traversal (`..`)
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct RelativePath(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

impl RelativePath {
    /// Create a validated relative path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Self::validate_relative_path_invariants("path", &path)?;
        Ok(Self(path))
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Validate the path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.0.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "path".to_owned().into(),
                message: "path cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }

    /// Private helper to validate relative path invariants.
    fn validate_relative_path_invariants(
        field: &'static str,
        path: &Path,
    ) -> Result<(), ConfigError> {
        if path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: format!("{field} cannot be empty").into(),
            });
        }
        if path.is_absolute() {
            return Err(ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: format!("{field} must be vault-relative").into(),
            });
        }
        if path
            .components()
            .any(|component| component == std::path::Component::ParentDir)
        {
            return Err(ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: format!("{field} must not contain parent components")
                    .into(),
            });
        }
        Ok(())
    }
}

/// File name without path separators.
///
/// Validated filename that must be:
/// - Non-empty
/// - No forward slashes (`/`)
/// - No backslashes (`\`)
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
pub struct FileName(
    /// Internal filename storage.
    Box<str>,
);

impl FileName {
    /// Create a validated file name.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the name is empty or contains
    /// path separators.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        Self::validate_non_empty("file_name", &value)?;
        if value.contains('/') || value.contains('\\') {
            return Err(ConfigError::ValidationFailed {
                field: "file_name".to_owned().into(),
                message: "file name must not contain path separators"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(value))
    }

    /// Return the file name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
}

// ============================================================================
// Tests
// ============================================================================

impl TryFrom<String> for RelativePath {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<RelativePath> for String {
    #[inline]
    fn from(value: RelativePath) -> Self {
        value.0.to_string_lossy().into_owned()
    }
}

impl TryFrom<String> for FileName {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<FileName> for String {
    #[inline]
    fn from(value: FileName) -> Self {
        value.0.into()
    }
}

impl std::fmt::Display for RelativePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test modules have relaxed unwrap/expect rules"
)]
mod tests {
    mod fixtures {
        use std::path::PathBuf;

        use super::super::{FileName, RelativePath, Schema};

        pub fn sample_schema() -> Schema {
            Schema::new(
                Some(
                    RelativePath::try_new(PathBuf::from("schemas"))
                        .expect("valid dir for fixture"),
                ),
                Some(
                    FileName::try_new("props.json")
                        .expect("valid file for fixture"),
                ),
            )
        }
    }

    mod constructor {
        use std::path::PathBuf;

        use super::super::*;

        #[test]
        fn relative_path_rejects_empty() {
            let result = RelativePath::try_new(PathBuf::from(""));
            assert!(result.is_err(), "Expected validation error");
        }

        #[test]
        fn relative_path_rejects_absolute() {
            let result = RelativePath::try_new(PathBuf::from("/abs"));
            assert!(result.is_err(), "Expected validation error");
        }

        #[test]
        fn relative_path_rejects_parent_traversal() {
            let result = RelativePath::try_new(PathBuf::from("a/../b"));
            assert!(result.is_err(), "Expected validation error");
        }
    }

    mod accessors {
        use std::path::PathBuf;

        /// 3.3-UNIT-034: `constructs_valid_property_bank_path`.
        /// Priority: P1.
        #[test]
        fn schema_property_bank_path_logic_works() {
            let schema = super::fixtures::sample_schema();

            let path = schema.property_bank_path();

            assert_eq!(
                path,
                PathBuf::from("schemas").join("props.json"),
                "property_bank_path logic works"
            );
        }
    }

    mod validation {
        use super::super::*;

        /// 3.3-UNIT-028: `schema_validate_rejects_empty_paths`.
        /// Priority: P0.
        #[test]
        fn schema_rejects_empty_paths() {
            let schemas_dir = RelativePath(std::path::PathBuf::from(""));
            let file_name = FileName::try_new("");
            assert!(
                schemas_dir.validate().is_err(),
                "Expected invalid schemas_dir"
            );
            assert!(file_name.is_err(), "Expected invalid file name");
        }
    }
}
