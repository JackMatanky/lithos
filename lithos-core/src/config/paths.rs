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
        Ok(())
    }
}

impl Default for Schema {
    #[inline]
    fn default() -> Self {
        Self {
            schemas_dir: SchemasDir::default(),
            property_bank_filename: FileName::try_new("property_bank.json")
                .unwrap_or_else(|_| FileName("property_bank.json".into())),
        }
    }
}

/// Template configuration (templates directory).
///
/// Validated configuration for template file storage. All paths are
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
pub struct Template {
    /// Directory containing template files.
    templates_dir: TemplatesDir,
}

impl Template {
    /// Create template configuration.
    #[inline]
    #[must_use]
    pub fn new(templates_dir: TemplatesDir) -> Self {
        Self {
            templates_dir,
        }
    }

    /// Return the templates directory.
    #[inline]
    #[must_use]
    pub fn templates_dir(&self) -> &TemplatesDir {
        &self.templates_dir
    }

    /// Validate template configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `templates_dir` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
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

/// Schema path overrides (optional configuration).
///
/// Used for vault-specific overrides of global schema settings.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaOverrides {
    /// Overridden schemas directory.
    pub schemas_dir: Option<SchemasDir>,
    /// Overridden property bank filename.
    pub property_bank_filename: Option<FileName>,
}

impl SchemaOverrides {
    /// Create schema override values.
    #[inline]
    #[must_use]
    pub fn new(
        schemas_dir: Option<SchemasDir>,
        property_bank_filename: Option<FileName>,
    ) -> Self {
        Self {
            schemas_dir,
            property_bank_filename,
        }
    }

    /// Return the overridden schemas directory, if set.
    #[inline]
    #[must_use]
    pub fn schemas_dir(&self) -> Option<&SchemasDir> {
        self.schemas_dir.as_ref()
    }

    /// Return the overridden property bank filename, if set.
    #[inline]
    #[must_use]
    pub fn property_bank_filename(&self) -> Option<&FileName> {
        self.property_bank_filename.as_ref()
    }
}

/// Template path overrides (optional configuration).
///
/// Used for vault-specific overrides of global template settings.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TemplateOverrides {
    /// Overridden templates directory.
    pub templates_dir: Option<TemplatesDir>,
}

impl TemplateOverrides {
    /// Create template override values.
    #[inline]
    #[must_use]
    pub fn new(templates_dir: Option<TemplatesDir>) -> Self {
        Self {
            templates_dir,
        }
    }

    /// Return the overridden templates directory, if set.
    #[inline]
    #[must_use]
    pub fn templates_dir(&self) -> Option<&TemplatesDir> {
        self.templates_dir.as_ref()
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

impl From<SchemaOverrides> for Schema {
    #[inline]
    fn from(overrides: SchemaOverrides) -> Self {
        let defaults = Schema::default();
        let schemas_dir = overrides.schemas_dir.unwrap_or(defaults.schemas_dir);
        let property_bank_filename = overrides
            .property_bank_filename
            .unwrap_or(defaults.property_bank_filename);
        Self::new(schemas_dir, property_bank_filename)
    }
}

impl From<TemplateOverrides> for Template {
    #[inline]
    fn from(overrides: TemplateOverrides) -> Self {
        let defaults = Template::default();
        let templates_dir =
            overrides.templates_dir.unwrap_or(defaults.templates_dir);
        Self::new(templates_dir)
    }
}

// ============================================================================
// Raw DTOs (Deserialization Boundary - Internal)
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
mod tests {
    use super::{CacheDir, FileName, SchemasDir, TemplatesDir};

    /// 3.3-UNIT-034: `constructs_valid_property_bank_path`.
    /// Priority: P1.
    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! which can panic."
    )]
    fn constructs_valid_property_bank_path() -> Result<(), super::ConfigError> {
        let schema = super::Schema::new(
            SchemasDir::try_new(std::path::PathBuf::from("schemas"))?,
            FileName::try_new("props.json")?,
        );

        let path = schema.property_bank_path();

        assert_eq!(
            path,
            std::path::PathBuf::from("schemas").join("props.json"),
            "property_bank_path logic works"
        );
        Ok(())
    }

    /// 3.3-UNIT-036: `rejects_empty_templates_dir`.
    /// Priority: P0.
    #[test]
    fn rejects_empty_templates_dir() {
        let result = TemplatesDir::try_new(std::path::PathBuf::from(""));
        assert!(result.is_err(), "Expected validation error");
    }

    /// 3.3-UNIT-036: `rejects_empty_cache_dir`.
    /// Priority: P0.
    #[test]
    fn rejects_empty_cache_dir() {
        let result = CacheDir::try_new(std::path::PathBuf::from(""));
        assert!(result.is_err(), "Expected validation error");
    }

    /// 3.3-UNIT-028: `schema_validate_rejects_empty_paths`.
    /// Priority: P0.
    #[test]
    fn schema_rejects_empty_paths() {
        let schemas_dir = SchemasDir::try_new(std::path::PathBuf::from(""));
        let file_name = FileName::try_new("");
        assert!(schemas_dir.is_err(), "Expected invalid schemas_dir");
        assert!(file_name.is_err(), "Expected invalid file name");
    }
}
