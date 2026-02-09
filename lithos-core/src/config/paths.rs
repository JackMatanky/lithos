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
// Fully Resolved Configuration (Aggregate Components)
// ============================================================================

/// Fully resolved paths configuration.
///
/// This struct contains all path-related settings after defaults and
/// overrides have been merged. All fields are guaranteed to be present.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Default,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Paths {
    /// Resolved cache settings.
    pub cache: Cache,
    /// Resolved schema settings.
    pub schema: Schema,
    /// Resolved property bank filename.
    pub property_bank: PropertyBank,
    /// Resolved template settings.
    pub template: Template,
}

impl Paths {
    /// Create fully resolved paths.
    #[inline]
    #[must_use]
    pub const fn new(
        cache: Cache,
        schema: Schema,
        property_bank: PropertyBank,
        template: Template,
    ) -> Self {
        Self {
            cache,
            schema,
            property_bank,
            template,
        }
    }

    /// Get the full path to the property bank file.
    ///
    /// Combines the schemas directory with the property bank filename.
    #[inline]
    #[must_use]
    pub fn property_bank_path(&self) -> PathBuf {
        self.schema.schemas_dir().as_path().join(self.property_bank.as_str())
    }
}

// ============================================================================
// Domain Types (Building Blocks)
// ============================================================================

/// Schema storage configuration (directory).
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
    schemas_dir: RelativePath,
}

impl Default for Schema {
    #[inline]
    fn default() -> Self {
        Self {
            schemas_dir: RelativePath(PathBuf::from("schemas")),
        }
    }
}

impl Schema {
    /// Create schema configuration.
    #[inline]
    #[must_use]
    pub const fn new(schemas_dir: RelativePath) -> Self {
        Self {
            schemas_dir,
        }
    }

    /// Create a validated schema directory path.
    ///
    /// # Errors
    /// Returns `ConfigError` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            schemas_dir: RelativePath::try_new(path)?,
        })
    }

    /// Return the schemas directory.
    #[inline]
    #[must_use]
    pub const fn schemas_dir(&self) -> &RelativePath {
        &self.schemas_dir
    }
}

/// Template storage configuration (directory).
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
    pub templates_dir: RelativePath,
}

impl Default for Template {
    #[inline]
    fn default() -> Self {
        Self {
            templates_dir: RelativePath(PathBuf::from("templates")),
        }
    }
}

impl Template {
    /// Create template configuration.
    #[inline]
    #[must_use]
    pub const fn new(templates_dir: RelativePath) -> Self {
        Self {
            templates_dir,
        }
    }

    /// Create a validated template directory path.
    ///
    /// # Errors
    /// Returns `ConfigError` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            templates_dir: RelativePath::try_new(path)?,
        })
    }

    /// Return the templates directory.
    #[inline]
    #[must_use]
    pub const fn templates_dir(&self) -> &RelativePath {
        &self.templates_dir
    }
}

/// Cache storage configuration (directory).
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
    pub cache_dir: RelativePath,
}

impl Default for Cache {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: RelativePath(PathBuf::from(".cache")),
        }
    }
}

impl Cache {
    /// Create cache configuration.
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: RelativePath) -> Self {
        Self {
            cache_dir,
        }
    }

    /// Create a validated cache directory path.
    ///
    /// # Errors
    /// Returns `ConfigError` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            cache_dir: RelativePath::try_new(path)?,
        })
    }

    /// Return the cache directory.
    #[inline]
    #[must_use]
    pub const fn cache_dir(&self) -> &RelativePath {
        &self.cache_dir
    }
}

/// Property bank filename configuration.
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
pub struct PropertyBank(FileName);

impl Default for PropertyBank {
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        clippy::expect_used,
        reason = "Default filename is guaranteed valid"
    )]
    fn default() -> Self {
        Self::try_new("property_bank.json")
            .expect("default property bank filename must be valid")
    }
}

impl PropertyBank {
    /// Create a validated property bank filename.
    ///
    /// # Errors
    /// Returns `ConfigError` if the name is invalid.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        Ok(Self(FileName::try_new(value)?))
    }

    /// Return the filename as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// ============================================================================
// Low-Level Implementation Types
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
// Standard Trait Implementations (Conversions)
// ============================================================================

impl From<FileName> for PropertyBank {
    #[inline]
    fn from(value: FileName) -> Self {
        Self(value)
    }
}

impl From<PropertyBank> for FileName {
    #[inline]
    fn from(value: PropertyBank) -> Self {
        value.0
    }
}

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

impl TryFrom<String> for PropertyBank {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<PropertyBank> for String {
    #[inline]
    fn from(value: PropertyBank) -> Self {
        value.0.into()
    }
}

impl std::fmt::Display for RelativePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

impl std::fmt::Display for PropertyBank {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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

        use super::super::{PropertyBank, Schema};

        pub fn sample_schema() -> Schema {
            Schema::try_new(PathBuf::from("schemas"))
                .expect("valid dir for fixture")
        }

        pub fn sample_property_bank() -> PropertyBank {
            PropertyBank::try_new("props.json").expect("valid file for fixture")
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

        use super::super::{Cache, Paths, Template};

        /// 3.3-UNIT-034: `constructs_valid_property_bank_path`.
        /// Priority: P1.
        #[test]
        fn schema_property_bank_path_logic_works() {
            let schema = super::fixtures::sample_schema();
            let property_bank = super::fixtures::sample_property_bank();
            let paths = Paths::new(
                Cache::default(),
                schema,
                property_bank,
                Template::default(),
            );

            let path = paths.property_bank_path();

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
            let schemas_dir =
                RelativePath::try_new(std::path::PathBuf::from(""));
            let file_name = PropertyBank::try_new("");
            assert!(schemas_dir.is_err(), "Expected invalid schemas_dir");
            assert!(file_name.is_err(), "Expected invalid file name");
        }
    }
}
