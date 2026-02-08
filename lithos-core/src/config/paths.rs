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

    pub(crate) fn try_new_with_field(
        field: &'static str,
        path: PathBuf,
    ) -> Result<Self, ConfigError> {
        validate_relative_path(field, &path)?;
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

    pub(crate) fn try_new_with_field(
        field: &'static str,
        path: PathBuf,
    ) -> Result<Self, ConfigError> {
        validate_relative_path(field, &path)?;
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

    pub(crate) fn try_new_with_field(
        field: &'static str,
        value: impl Into<Box<str>>,
    ) -> Result<Self, ConfigError> {
        let value = value.into();
        validate_non_empty(field, &value)?;
        if value.contains('/') || value.contains('\\') {
            return Err(ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: format!("{field} must not contain path separators")
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
// Raw DTOs (Deserialization Boundary - Internal)
// ============================================================================

/// Raw schema filesystem configuration (unvalidated input from config files).
///
/// This is a serde-only DTO that accepts flexible input from TOML/YAML/JSON.
/// All fields are optional to support partial configuration and defaults.
/// Validation happens during conversion to [`Schema`] via [`TryFrom`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawSchemaPaths {
    /// Directory containing schema files.
    pub schemas_dir: Option<String>,
    /// Property bank filename.
    pub property_bank_filename: Option<String>,
}

impl TryFrom<RawSchemaPaths> for Schema {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawSchemaPaths) -> Result<Self, Self::Error> {
        let defaults = Schema::default();
        let schemas_dir = match raw.schemas_dir {
            Some(value) => SchemasDir::try_new_with_field(
                "schemas_dir",
                PathBuf::from(value),
            )?,
            None => defaults.schemas_dir,
        };
        let property_bank_filename = match raw.property_bank_filename {
            Some(value) => {
                FileName::try_new_with_field("property_bank_filename", value)?
            }
            None => defaults.property_bank_filename,
        };
        Ok(Schema::new(schemas_dir, property_bank_filename))
    }
}

/// Raw template filesystem configuration (unvalidated input from config files).
///
/// This is a serde-only DTO that accepts flexible input from TOML/YAML/JSON.
/// All fields are optional to support partial configuration and defaults.
/// Validation happens during conversion to [`Template`] via [`TryFrom`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawTemplatePaths {
    /// Directory containing template files.
    pub templates_dir: Option<String>,
}

impl TryFrom<RawTemplatePaths> for Template {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTemplatePaths) -> Result<Self, Self::Error> {
        let defaults = Template::default();
        let templates_dir = match raw.templates_dir {
            Some(value) => TemplatesDir::try_new_with_field(
                "templates_dir",
                PathBuf::from(value),
            )?,
            None => defaults.templates_dir,
        };
        Ok(Template::new(templates_dir))
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
mod tests {
    use super::{FileName, SchemasDir, TemplatesDir};

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
