//! Path configuration types.
//!
//! This module contains types for validated configuration paths,
//! including schemas directory, templates directory, and file names.

use std::path::{Path, PathBuf};

use super::error::ConfigError;

// ============================================================================
// Public Domain Types (Most Important - User-Facing API)
// ============================================================================

/// Schema configuration (schemas directory + property bank filename).
///
/// Validated configuration for schema file storage. All paths are
/// vault-relative.
///
/// # Examples
///
/// ```rust
/// # use lithos_core::config::paths::Schema;
/// let schema = Schema::default();
///
/// assert_eq!(
///     schema.property_bank_path(),
///     std::path::PathBuf::from("schemas").join("property_bank.json"),
///     "Property bank path should use schema directory"
/// );
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
pub struct Schema {
    /// Directory containing schema files.
    schemas_dir: SchemasDir,
    /// Property bank filename (stored in `schemas_dir`).
    property_bank_filename: FileName,
}

impl Schema {
    /// Create schema configuration.
    #[inline]
    #[must_use]
    pub fn new(
        schemas_dir: SchemasDir,
        property_bank_filename: FileName,
    ) -> Self {
        Self {
            schemas_dir,
            property_bank_filename,
        }
    }

    /// Return the schemas directory.
    #[inline]
    #[must_use]
    pub fn schemas_dir(&self) -> &SchemasDir {
        &self.schemas_dir
    }

    /// Return the property bank file name.
    #[inline]
    #[must_use]
    pub fn property_bank_filename(&self) -> &FileName {
        &self.property_bank_filename
    }

    /// Get the full path to the property bank file
    /// (`schemas_dir/property_bank_filename`).
    ///
    /// The property bank is always stored in the schemas directory.
    #[inline]
    #[must_use]
    pub fn property_bank_path(&self) -> PathBuf {
        self.schemas_dir.as_path().join(self.property_bank_filename.as_str())
    }

    /// Validate schema configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `schemas_dir` or
    /// `property_bank_filename` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schemas_dir.validate()?;
        // FileName is validated at construction
        Ok(())
    }
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
            schemas_dir: SchemasDir::default(),
            property_bank_filename: FileName::try_new("property_bank.json")
                .expect("default filename must be valid"),
        }
    }
}

/// Template configuration (templates directory).
///
/// Validated configuration for template storage. All paths are vault-relative.
///
/// # Examples
///
/// ```rust
/// # use lithos_core::config::paths::Template;
/// let template = Template::default();
///
/// assert_eq!(
///     template.templates_dir().as_path(),
///     std::path::Path::new("templates"),
///     "Template directory should match default"
/// );
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
pub struct Template {
    /// Directory containing template files.
    templates_dir: TemplatesDir,
}

impl Template {
    /// Create template configuration.
    #[inline]
    #[must_use]
    pub const fn new(templates_dir: TemplatesDir) -> Self {
        Self {
            templates_dir,
        }
    }

    /// Return the templates directory.
    #[inline]
    #[must_use]
    pub const fn templates_dir(&self) -> &TemplatesDir {
        &self.templates_dir
    }

    /// Validate template configuration.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `templates_dir` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.templates_dir.validate()?;
        Ok(())
    }
}

impl Default for Template {
    #[inline]
    fn default() -> Self {
        Self {
            templates_dir: TemplatesDir::default(),
        }
    }
}

// ============================================================================
// Building Block Types (Path Components)
// ============================================================================

/// Vault-relative schemas directory.
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
pub struct SchemasDir(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

impl SchemasDir {
    /// Create a validated schemas directory path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        validate_relative_path("schemas_dir", &path)?;
        Ok(Self(path))
    }

    /// Return the directory path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Validate the schemas directory.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("schemas_dir", &self.0.to_string_lossy())
    }
}

impl Default for SchemasDir {
    #[inline]
    fn default() -> Self {
        Self(PathBuf::from("schemas"))
    }
}

/// Vault-relative templates directory.
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
pub struct TemplatesDir(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

impl TemplatesDir {
    /// Create a validated templates directory path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        validate_relative_path("templates_dir", &path)?;
        Ok(Self(path))
    }

    /// Return the directory path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Validate the templates directory.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("templates_dir", &self.0.to_string_lossy())
    }
}

impl Default for TemplatesDir {
    #[inline]
    fn default() -> Self {
        Self(PathBuf::from("templates"))
    }
}

/// Vault-relative cache directory.
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
pub struct CacheDir(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

impl CacheDir {
    /// Create a validated cache directory path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is invalid.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        validate_relative_path("cache_dir", &path)?;
        Ok(Self(path))
    }

    /// Return the cache directory path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl Default for CacheDir {
    #[inline]
    fn default() -> Self {
        Self(PathBuf::from(".cache"))
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
        validate_non_empty("file_name", &value)?;
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
}

// ============================================================================
// Standard Trait Implementations (Conversions)
// ============================================================================

impl TryFrom<String> for SchemasDir {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<SchemasDir> for String {
    #[inline]
    fn from(value: SchemasDir) -> Self {
        value.0.to_string_lossy().into_owned()
    }
}

impl TryFrom<String> for TemplatesDir {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<TemplatesDir> for String {
    #[inline]
    fn from(value: TemplatesDir) -> Self {
        value.0.to_string_lossy().into_owned()
    }
}

impl TryFrom<String> for CacheDir {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<CacheDir> for String {
    #[inline]
    fn from(value: CacheDir) -> Self {
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

fn validate_relative_path(
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

        use super::super::{FileName, Schema, SchemasDir};

        pub fn sample_schema() -> Schema {
            Schema::new(
                SchemasDir::try_new(PathBuf::from("schemas"))
                    .expect("valid dir for fixture"),
                FileName::try_new("props.json")
                    .expect("valid file for fixture"),
            )
        }
    }

    mod constructor {
        use std::path::PathBuf;

        use super::super::*;

        #[test]
        fn templates_dir_rejects_empty() {
            let result = TemplatesDir::try_new(PathBuf::from(""));
            assert!(result.is_err(), "Expected validation error");
        }

        #[test]
        fn cache_dir_rejects_empty() {
            let result = CacheDir::try_new(PathBuf::from(""));
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
        use std::path::PathBuf;

        use super::super::*;

        /// 3.3-UNIT-028: `schema_validate_rejects_empty_paths`.
        /// Priority: P0.
        #[test]
        fn schema_rejects_empty_paths() {
            let schemas_dir = SchemasDir::try_new(PathBuf::from(""));
            let file_name = FileName::try_new("");
            assert!(schemas_dir.is_err(), "Expected invalid schemas_dir");
            assert!(file_name.is_err(), "Expected invalid file name");
        }
    }
}
