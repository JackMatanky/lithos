//! Schema configuration types.
//!
//! This module owns the schema portion of resolved configuration:
//! [`SchemaDir`] stores the declarative relative schema directory,
//! [`PropertyBankFile`] stores the property bank filename, [`SchemaConfig`]
//! combines both values, and [`SchemaConfigSpec`] exposes the narrowed
//! contract used by schema discovery.
//!
//! Schema paths stay declarative at the config boundary. The directory uses
//! [`RelativeDirPath`] and the property bank path is projected to
//! [`RelativeFilePath`] only when a consumer asks for a config spec. Filesystem
//! validation happens when [`SchemaConfigSpec`] resolves those declarations
//! against a vault root.
//!
//! # Examples
//!
//! ```rust
//! use lithos_core::{
//!     config::schema::{PropertyBankFile, SchemaConfig, SchemaConfigSpec, SchemaDir},
//!     fs::{DirPath, path::RelativeFilePath},
//! };
//!
//! # fn example(root: DirPath) -> Result<(), Box<dyn std::error::Error>> {
//! let config = SchemaConfig::new(
//!     SchemaDir::default(),
//!     PropertyBankFile::try_new("property_bank.json")?,
//! );
//! let property_bank = RelativeFilePath::try_new(
//!     config.property_bank_relative_path().to_string_lossy().as_ref(),
//! )?;
//! let spec = SchemaConfigSpec::new(
//!     root,
//!     config.schema_dir().as_relative_dir().clone(),
//!     property_bank,
//! );
//!
//! assert_eq!(spec.directory_relative().as_str(), "schemas");
//! assert_eq!(spec.property_bank_relative().as_str(), "schemas/property_bank.json");
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::ConfigError;
use crate::fs::{
    DirPath, FileName, FilePath, PathKey,
    path::{RelativeDirPath, RelativeFilePath},
};

// ----------------------------------------------------------- //
//                       SchemaDir                             //
// ----------------------------------------------------------- //

/// Declarative schema directory configuration.
///
/// Validates that the supplied path is a non-empty, relative (non-absolute)
/// path that does not escape the vault root via `..`.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SchemaDir(RelativeDirPath);

impl SchemaDir {
    /// Creates a `SchemaDir` from an already-validated relative directory path.
    #[inline]
    #[must_use]
    pub const fn new(schema_dir: RelativeDirPath) -> Self {
        Self(schema_dir)
    }

    /// Creates a validated schema directory path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is absolute,
    /// empty, or contains parent directory traversal.
    #[inline]
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError> {
        let value =
            path.to_str().ok_or_else(|| ConfigError::ValidationFailed {
                field: "schemas_dir".into(),
                message: "Non-UTF-8 path".into(),
            })?;

        RelativeDirPath::try_new(value).map(Self).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "schemas_dir".into(),
                message: error.to_string().into(),
            }
        })
    }

    /// Returns the relative directory declaration.
    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.0
    }
}

impl Default for SchemaDir {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Default directory literal is guaranteed valid"
    )]
    fn default() -> Self {
        Self(
            RelativeDirPath::try_new("schemas")
                .expect("default path literal must be valid"),
        )
    }
}

// ----------------------------------------------------------- //
//                     PropertyBankFile                        //
// ----------------------------------------------------------- //

/// Property bank filename configuration.
///
/// The property bank is a central registry of all properties used across
/// a vault's notes. The filename must not contain path separators.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct PropertyBankFile(FileName);

impl PropertyBankFile {
    /// Creates a validated property bank filename.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty or
    /// contains path separators.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let boxed: Box<str> = value.into();
        if boxed.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "property_bank_file".into(),
                message: "filename must not be empty".into(),
            });
        }
        if boxed.contains('/') || boxed.contains('\\') {
            return Err(ConfigError::ValidationFailed {
                field: "property_bank_file".into(),
                message: "filename must not contain path separators".into(),
            });
        }
        Ok(Self(FileName::new(boxed)))
    }

    /// Returns the filename as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for PropertyBankFile {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Default filename is guaranteed valid"
    )]
    fn default() -> Self {
        Self::try_new("property_bank.json")
            .expect("default property bank filename must be valid")
    }
}

impl From<FileName> for PropertyBankFile {
    #[inline]
    fn from(value: FileName) -> Self {
        Self(value)
    }
}

impl TryFrom<String> for PropertyBankFile {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<PropertyBankFile> for String {
    #[inline]
    fn from(value: PropertyBankFile) -> Self {
        value.0.into()
    }
}

impl std::fmt::Display for PropertyBankFile {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ----------------------------------------------------------- //
//                       SchemaConfig                          //
// ----------------------------------------------------------- //

/// Resolved schema configuration.
///
/// Combines the schema directory location and property bank filename into
/// a single validated configuration object.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct SchemaConfig {
    schema_dir: SchemaDir,
    property_bank_file: PropertyBankFile,
}

impl SchemaConfig {
    /// Creates a resolved schema configuration.
    #[inline]
    #[must_use]
    pub const fn new(
        schema_dir: SchemaDir,
        property_bank_file: PropertyBankFile,
    ) -> Self {
        Self {
            schema_dir,
            property_bank_file,
        }
    }

    /// Returns the schema directory configuration.
    #[inline]
    #[must_use]
    pub const fn schema_dir(&self) -> &SchemaDir {
        &self.schema_dir
    }

    /// Returns the property bank filename configuration.
    #[inline]
    #[must_use]
    pub const fn property_bank_file(&self) -> &PropertyBankFile {
        &self.property_bank_file
    }

    /// Returns the property bank file path relative to the vault root.
    ///
    /// Combines the schema directory with the property bank filename.
    #[inline]
    #[must_use]
    pub fn property_bank_relative_path(&self) -> PathBuf {
        std::path::Path::new(self.schema_dir.as_relative_dir().as_str())
            .join(self.property_bank_file.as_str())
    }
}

impl Default for SchemaConfig {
    #[inline]
    fn default() -> Self {
        Self::new(SchemaDir::default(), PropertyBankFile::default())
    }
}

// ----------------------------------------------------------- //
//                    SchemaConfigSpec                         //
// ----------------------------------------------------------- //

/// Schema configuration specification for discovery engine.
///
/// This type provides a minimal, filesystem-focused view of schema
/// configuration for the discovery engine. It contains only the paths
/// needed for file scanning and discovery.
///
/// # Design Rationale
///
/// - **Type-safe paths**: Uses `DirPath`, `FilePath`, `RelativeDirPath`, and
///   `RelativeFilePath` for compile-time enforcement.
/// - **Vault-rooted**: Stores vault root and relative paths, constructing
///   absolute paths via `append_dir`/`append_file`.
/// - **Discovery-focused**: Used by `DiscoveryEngine::run()` instead of full
///   `Config`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SchemaConfigSpec {
    /// Vault root directory (e.g., "/vault").
    root: DirPath,
    /// Relative path to schema directory from vault root (e.g., "schemas").
    directory: RelativeDirPath,
    /// Relative path to property bank file from vault root (e.g.,
    /// `"schemas/property_bank.json"`).
    property_bank: RelativeFilePath,
}

impl SchemaConfigSpec {
    /// Creates a new schema configuration specification.
    #[inline]
    #[must_use]
    pub const fn new(
        root: DirPath,
        directory: RelativeDirPath,
        property_bank: RelativeFilePath,
    ) -> Self {
        Self {
            root,
            directory,
            property_bank,
        }
    }

    /// Returns the vault root directory.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> &DirPath {
        &self.root
    }

    /// Returns the absolute schemas directory path.
    ///
    /// # Errors
    /// Returns an error if the derived directory path does not currently exist.
    #[inline]
    pub fn schema_directory_path(
        &self,
    ) -> Result<DirPath, crate::fs::PathError> {
        self.root.append_dir(&self.directory)
    }

    /// Returns the absolute property bank file path.
    ///
    /// # Errors
    /// Returns an error if the derived file path does not currently exist.
    #[inline]
    pub fn property_bank_file_path(
        &self,
    ) -> Result<FilePath, crate::fs::PathError> {
        self.root.append_file(&self.property_bank)
    }

    /// Returns the schema directory persistence key.
    ///
    /// # Errors
    /// Returns an error when the absolute schema directory path cannot be
    /// derived or key conversion fails.
    #[inline]
    pub fn schema_directory_key(
        &self,
    ) -> Result<PathKey, crate::fs::PathError> {
        self.schema_directory_path()?.as_key(self.root())
    }

    /// Returns the property bank persistence key.
    ///
    /// # Errors
    /// Returns an error when the absolute property bank file path cannot be
    /// derived or key conversion fails.
    #[inline]
    pub fn property_bank_key(&self) -> Result<PathKey, crate::fs::PathError> {
        self.property_bank_file_path()?.as_key(self.root())
    }

    /// Returns the relative schema directory declaration.
    #[inline]
    #[must_use]
    pub const fn directory_relative(&self) -> &RelativeDirPath {
        &self.directory
    }

    /// Returns the relative property bank declaration.
    #[inline]
    #[must_use]
    pub const fn property_bank_relative(&self) -> &RelativeFilePath {
        &self.property_bank
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FileName;

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_schema_dir() {
            let schema_dir = SchemaDir::default();

            assert_eq!(
                schema_dir.as_relative_dir().as_str(),
                "schemas",
                "default schema dir should match the documented default"
            );
        }

        #[test]
        fn returns_default_property_bank_file() {
            let property_bank = PropertyBankFile::default();

            assert_eq!(
                property_bank.as_str(),
                "property_bank.json",
                "default property bank file should match the documented \
                 default"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_schema_dir() {
            let result = SchemaDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty schema path should be rejected");
        }

        #[test]
        fn rejects_absolute_schema_dir() {
            let result =
                SchemaDir::try_new(std::path::Path::new("/tmp/schemas"));

            assert!(result.is_err(), "absolute schema path should be rejected");
        }

        #[test]
        fn rejects_parent_traversal_schema_dir() {
            let result = SchemaDir::try_new(std::path::Path::new("../schemas"));

            assert!(
                result.is_err(),
                "schema path escaping the vault root should be rejected"
            );
        }

        #[test]
        fn rejects_property_bank_file_with_path_separator() {
            let result = PropertyBankFile::try_new("schemas/bank.json");

            assert!(
                result.is_err(),
                "property bank file should reject path separators"
            );
        }
    }

    mod schema_config {
        use super::*;

        #[test]
        fn returns_default_schema_dir_and_property_bank_file() {
            let config = SchemaConfig::default();

            assert_eq!(
                config.schema_dir().as_relative_dir().as_str(),
                "schemas",
                "schema config should expose the default schema directory"
            );
            assert_eq!(
                config.property_bank_file().as_str(),
                "property_bank.json",
                "schema config should expose the default property bank file"
            );
        }

        #[test]
        fn returns_property_bank_relative_path_under_schema_dir() {
            let schema_dir =
                SchemaDir::try_new(std::path::Path::new("custom-schemas"))
                    .expect("fixture schema dir should be valid");
            let property_bank = PropertyBankFile::try_new("bank.json")
                .expect("fixture property bank file should be valid");
            let config = SchemaConfig::new(schema_dir, property_bank);

            let result = config.property_bank_relative_path();

            assert_eq!(
                result,
                std::path::PathBuf::from("custom-schemas").join("bank.json"),
                "schema config should derive property bank path under schema \
                 dir"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_property_bank_file_from_file_name() {
            let file_name =
                FileName::try_from(std::path::Path::new("bank.json"))
                    .expect("fixture filename should be valid");

            let property_bank = PropertyBankFile::from(file_name);

            assert_eq!(
                property_bank.as_str(),
                "bank.json",
                "property bank file should retain the validated file name"
            );
        }

        #[test]
        fn returns_string_from_property_bank_file() {
            let property_bank = PropertyBankFile::try_new("bank.json")
                .expect("fixture property bank file should be valid");

            let value = String::from(property_bank);

            assert_eq!(
                value, "bank.json",
                "property bank file should convert back to owned string"
            );
        }
    }

    mod schema_config_spec {
        use super::*;
        use crate::fs::{
            DirPath,
            path::{RelativeDirPath, RelativeFilePath},
        };

        #[test]
        fn returns_relative_paths_without_requiring_targets_to_exist() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("schemas")
                .expect("fixture relative dir should be valid");
            let property_bank = RelativeFilePath::try_new("schemas/bank.json")
                .expect("fixture relative file should be valid");

            let spec = SchemaConfigSpec::new(root, directory, property_bank);

            assert_eq!(
                spec.directory_relative().as_str(),
                "schemas",
                "schema spec should retain schema directory declaration"
            );
            assert_eq!(
                spec.property_bank_relative().as_str(),
                "schemas/bank.json",
                "schema spec should retain property bank declaration"
            );
        }

        #[test]
        fn returns_schema_directory_path_when_root_and_relative_dir_are_valid()
        {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_path = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_path)
                .expect("schemas dir fixture should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("schemas")
                .expect("fixture relative dir should be valid");
            let property_bank = RelativeFilePath::try_new("schemas/bank.json")
                .expect("fixture relative file should be valid");

            let spec = SchemaConfigSpec::new(root, directory, property_bank);

            let result = spec.schema_directory_path();

            assert!(
                result.is_ok(),
                "existing schema dir should resolve successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_path(),
                schemas_path.as_path(),
                "schema spec should resolve schema directory against vault \
                 root"
            );
        }

        #[test]
        fn returns_property_bank_file_path_when_root_and_relative_file_are_valid()
         {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_path = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_path)
                .expect("schemas dir fixture should be created");
            let bank_path = schemas_path.join("bank.json");
            std::fs::write(&bank_path, "{}")
                .expect("property bank fixture should be writable");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("schemas")
                .expect("fixture relative dir should be valid");
            let property_bank = RelativeFilePath::try_new("schemas/bank.json")
                .expect("fixture relative file should be valid");

            let spec = SchemaConfigSpec::new(root, directory, property_bank);

            let result = spec.property_bank_file_path();

            assert!(
                result.is_ok(),
                "existing property bank file should resolve successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_path(),
                bank_path.as_path(),
                "schema spec should resolve property bank file against vault \
                 root"
            );
        }

        #[test]
        fn returns_schema_directory_key_when_root_scoped_dir_is_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_path = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_path)
                .expect("schemas dir fixture should be created");
            let bank_path = schemas_path.join("bank.json");
            std::fs::write(&bank_path, "{}")
                .expect("property bank fixture should be writable");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("schemas")
                .expect("fixture relative dir should be valid");
            let property_bank = RelativeFilePath::try_new("schemas/bank.json")
                .expect("fixture relative file should be valid");

            let spec = SchemaConfigSpec::new(root, directory, property_bank);

            let result = spec.schema_directory_key();

            assert!(
                result.is_ok(),
                "existing schema dir should convert to path key: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_str(),
                "schemas",
                "schema directory key should be vault-relative"
            );
        }

        #[test]
        fn returns_property_bank_key_when_root_scoped_file_is_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_path = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_path)
                .expect("schemas dir fixture should be created");
            let bank_path = schemas_path.join("bank.json");
            std::fs::write(&bank_path, "{}")
                .expect("property bank fixture should be writable");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("schemas")
                .expect("fixture relative dir should be valid");
            let property_bank = RelativeFilePath::try_new("schemas/bank.json")
                .expect("fixture relative file should be valid");

            let spec = SchemaConfigSpec::new(root, directory, property_bank);

            let result = spec.property_bank_key();

            assert!(
                result.is_ok(),
                "existing property bank file should convert to path key: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_str(),
                "schemas/bank.json",
                "property bank key should be vault-relative"
            );
        }
    }
}
