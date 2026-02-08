//! Configuration types shared between vault and global contexts.
//!
//! This module contains the fundamental configuration types that are used
//! by both vault and global configuration contexts.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv::Archive derive generates exhaustive archived enums"
)]
#![expect(
    clippy::struct_field_names,
    reason = "Frontmatter struct fields intentionally share '_key' suffix \
              (flagged by rkyv::Archive derive)"
)]

use std::path::{Path, PathBuf};

use super::{
    error::ConfigError,
    raw::{RawFrontmatter, RawLogging, RawSchemaPaths, RawTemplatePaths},
};

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

/// Logging verbosity level.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LogLevel {
    /// Error-level logging only.
    Error,
    /// Warning and error logging.
    Warn,
    /// Informational logging.
    #[default]
    Info,
    /// Debug logging.
    Debug,
    /// Trace-level logging.
    Trace,
}

/// Vault-relative schemas directory.
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

/// Vault-relative templates directory.
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

/// File name without path separators.
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

impl LogLevel {
    #[inline]
    #[must_use]
    /// Return the lowercase string form.
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl TryFrom<RawLogging> for Logging {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawLogging) -> Result<Self, Self::Error> {
        match raw.log_level {
            Some(value) => Ok(Logging::new(LogLevel::try_from(value)?)),
            None => Ok(Logging::default()),
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        match value.as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(ConfigError::InvalidEnumValue {
                field: "log_level".to_owned().into(),
                value: value.into(),
                allowed: ["error", "warn", "info", "debug", "trace"]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            }),
        }
    }
}

impl From<LogLevel> for String {
    #[inline]
    fn from(value: LogLevel) -> Self {
        value.as_str().to_owned()
    }
}

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

    #[inline]
    #[must_use]
    /// Return the directory path.
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

    #[inline]
    #[must_use]
    /// Return the directory path.
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

impl FileName {
    #[inline]
    /// Create a validated file name.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the name is empty or contains
    /// path separators.
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

    #[inline]
    #[must_use]
    /// Return the file name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
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

/// Logging configuration with validation.
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
pub struct Logging {
    /// Log level (debug, info, warn, error).
    log_level: LogLevel,
}

/// Schema configuration (schemas directory).
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

/// Template configuration (templates directory).
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

impl Default for Logging {
    #[inline]
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
        }
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

impl Default for Template {
    #[inline]
    fn default() -> Self {
        Self {
            templates_dir: TemplatesDir::default(),
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
    /// # use lithos_core::config::types::Frontmatter;
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

impl Logging {
    #[inline]
    #[must_use]
    /// Create logging configuration.
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            log_level,
        }
    }

    #[inline]
    #[must_use]
    /// Return the log level.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    #[inline]
    #[must_use]
    /// Return the log level as a string.
    pub fn log_level_str(&self) -> &'static str {
        self.log_level.as_str()
    }

    /// Validate logging configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::InvalidEnumValue` if `log_level` is not one
    /// of: debug, info, warn, error.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::config::types::Logging;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let logging = Logging::default();
    /// logging.validate()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Schema {
    #[inline]
    #[must_use]
    /// Create schema configuration.
    pub fn new(
        schemas_dir: SchemasDir,
        property_bank_filename: FileName,
    ) -> Self {
        Self {
            schemas_dir,
            property_bank_filename,
        }
    }

    #[inline]
    #[must_use]
    /// Return the schemas directory.
    pub fn schemas_dir(&self) -> &SchemasDir {
        &self.schemas_dir
    }

    #[inline]
    #[must_use]
    /// Return the property bank file name.
    pub fn property_bank_filename(&self) -> &FileName {
        &self.property_bank_filename
    }

    /// Get the full path to the property bank file
    /// (`schemas_dir/property_bank_filename`).
    ///
    /// The property bank is always stored in the schemas directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_core::config::types::Schema;
    /// let schema = Schema::default();
    ///
    /// assert_eq!(
    ///     schema.property_bank_path(),
    ///     std::path::PathBuf::from("schemas").join("property_bank.json"),
    ///     "Property bank path should use schema directory"
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub fn property_bank_path(&self) -> PathBuf {
        self.schemas_dir.as_path().join(self.property_bank_filename.as_str())
    }

    /// Validate schema configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if `schemas_dir` or
    /// `property_bank_filename` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Template {
    #[inline]
    #[must_use]
    /// Create template configuration.
    pub fn new(templates_dir: TemplatesDir) -> Self {
        Self {
            templates_dir,
        }
    }

    #[inline]
    #[must_use]
    /// Return the templates directory.
    pub fn templates_dir(&self) -> &TemplatesDir {
        &self.templates_dir
    }

    /// Validate template configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if `templates_dir` is
    /// empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{FileName, FrontmatterKey, LogLevel, SchemasDir, TemplatesDir};

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

    /// 3.3-UNIT-026: `frontmatter_validate_rejects_empty_keys`.
    /// Priority: P0.
    #[test]
    fn frontmatter_key_rejects_empty() {
        let result = FrontmatterKey::try_new("");
        assert!(result.is_err(), "Expected validation error");
    }

    /// 3.3-UNIT-027: `logging_rejects_invalid_levels`.
    /// Priority: P0.
    #[test]
    fn logging_rejects_invalid_levels() {
        let result = LogLevel::try_from("verbose".to_owned());
        assert!(result.is_err(), "Expected validation error");
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
