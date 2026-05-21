//! Validated path configuration management.
//!
//! This module defines how Lithos manages its filesystem locations (cache,
//! schemas, templates). It distinguishes between the fully-resolved [`Paths`]
//! and the partial overrides used during construction.
//!
//! # Always Valid Invariants
//! - **Relative Paths**: Most paths must be vault-relative and cannot use `..`
//!   to escape the vault root.
//! - **File Names**: FileNames must not contain path separators.
//! - **Non-Empty**: Paths and filenames cannot be empty. Construction of these
//!   types will fail if these invariants are violated.

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived structs"
)]

use std::path::PathBuf;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::ConfigError;
use crate::fs::{DirPath, FileName, FilePath, FsPath, RelativePath};

#[expect(
    clippy::expect_used,
    reason = "Static defaults are compile-time literals expected to remain \
              valid"
)]
fn default_relative_path(value: &'static str) -> RelativePath {
    RelativePath::try_from(value).expect("default path literal must be valid")
}

// ----------------------------------------------------------- //
//                   Resolved Path Aggregate                   //
// ----------------------------------------------------------- //

/// Fully resolved paths configuration.
///
/// This struct contains all path-related settings after defaults and
/// overrides have been merged. All fields are guaranteed to be present
/// and validated.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::paths::Paths;
///
/// let paths = Paths::default();
/// assert_eq!(paths.property_bank.as_str(), "property_bank.json");
/// ```
#[derive(Debug, Clone, PartialEq, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Paths {
    /// Resolved cache settings.
    pub cache: Cache,
    /// Resolved template settings.
    pub template: Template,
    /// Resolved schema settings.
    pub schema: Schema,
    /// Resolved property bank filename.
    pub property_bank: PropertyBank,
}

impl Paths {
    /// Creates fully resolved paths.
    #[inline]
    #[must_use]
    pub const fn new(
        cache: Cache,
        template: Template,
        schema: Schema,
        property_bank: PropertyBank,
    ) -> Self {
        Self {
            cache,
            template,
            schema,
            property_bank,
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

impl ArchivedPaths {
    /// Return the cache configuration.
    #[inline]
    #[must_use]
    pub const fn cache(&self) -> &ArchivedCache {
        &self.cache
    }
}

impl TryFrom<&super::raw::RawPathsConfig> for Paths {
    type Error = ConfigError;

    /// Create Paths from raw configuration, applying defaults for missing
    /// values.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if any path is invalid.
    #[inline]
    fn try_from(raw: &super::raw::RawPathsConfig) -> Result<Self, Self::Error> {
        let cache = raw
            .cache_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Cache::try_new(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "cache_dir".into(),
                        message: format!("invalid cache_dir: {e}").into(),
                    }
                })
            })
            .transpose()?
            .unwrap_or_default();

        let template = raw
            .templates_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Template::try_new(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "templates_dir".into(),
                        message: format!("invalid templates_dir: {e}").into(),
                    }
                })
            })
            .transpose()?
            .unwrap_or_default();

        let schema = raw
            .schemas_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Schema::try_new(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "schemas_dir".into(),
                        message: format!("invalid schemas_dir: {e}").into(),
                    }
                })
            })
            .transpose()?
            .unwrap_or_default();

        let property_bank = raw
            .property_bank_file
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                PropertyBank::try_new(s.clone()).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "property_bank_file".into(),
                        message: format!("invalid property_bank_file: {e}")
                            .into(),
                    }
                })
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self::new(cache, template, schema, property_bank))
    }
}

// ----------------------------------------------------------- //
//                        Domain Types                         //
// ----------------------------------------------------------- //

/// Schema storage configuration.
///
/// This type manages the location where Lithos looks for note schemas.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
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
            schemas_dir: default_relative_path("schemas"),
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

    /// Creates a validated schema directory path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is absolute,
    /// empty, or contains parent directory traversal.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            schemas_dir: RelativePath::try_from(path).map_err(|e| {
                ConfigError::ValidationFailed {
                    field: "schemas_dir".into(),
                    message: e.to_string().into(),
                }
            })?,
        })
    }

    /// Return the schemas directory.
    #[inline]
    #[must_use]
    pub const fn schemas_dir(&self) -> &RelativePath {
        &self.schemas_dir
    }
}

/// Template storage configuration.
///
/// This type manages the location where Lithos looks for note templates.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
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
            templates_dir: default_relative_path("templates"),
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

    /// Creates a validated template directory path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is absolute,
    /// empty, or contains parent directory traversal.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            templates_dir: RelativePath::try_from(path).map_err(|e| {
                ConfigError::ValidationFailed {
                    field: "templates_dir".into(),
                    message: e.to_string().into(),
                }
            })?,
        })
    }

    /// Return the templates directory.
    #[inline]
    #[must_use]
    pub const fn templates_dir(&self) -> &RelativePath {
        &self.templates_dir
    }
}

/// Cache storage configuration.
///
/// This type manages the location where Lithos stores its performance cache.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
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
            cache_dir: default_relative_path(".cache"),
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

    /// Creates a validated cache directory path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is absolute,
    /// empty, or contains parent directory traversal.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        Ok(Self {
            cache_dir: RelativePath::try_from(path).map_err(|e| {
                ConfigError::ValidationFailed {
                    field: "cache_dir".into(),
                    message: e.to_string().into(),
                }
            })?,
        })
    }

    /// Return the cache directory.
    #[inline]
    #[must_use]
    pub const fn cache_dir(&self) -> &RelativePath {
        &self.cache_dir
    }
}

impl ArchivedCache {
    /// Return the cache directory.
    #[inline]
    #[must_use]
    pub const fn cache_dir(&self) -> &rkyv::Archived<RelativePath> {
        &self.cache_dir
    }
}

/// Property bank filename configuration.
///
/// The property bank is a central registry of all properties used across
/// a vault's notes.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct PropertyBank(FileName);

impl Default for PropertyBank {
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

impl PropertyBank {
    /// Creates a validated property bank filename.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty or
    /// contains path separators.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        Ok(Self(FileName::new(value.into())))
    }

    /// Return the filename as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<FileName> for PropertyBank {
    #[inline]
    fn from(value: FileName) -> Self {
        Self(value)
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

impl std::fmt::Display for PropertyBank {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ----------------------------------------------------------- //
//                    Schema Config Spec                       //
// ----------------------------------------------------------- //

/// Schema configuration specification for discovery engine.
///
/// This type provides a minimal, filesystem-focused view of schema
/// configuration for the discovery engine. It contains only the paths
/// needed for file scanning and discovery.
///
/// # Design Rationale
///
/// - **Type-safe paths**: Uses `DirPath`, `FilePath`, and `RelativePath` for
///   compile-time enforcement
/// - **Vault-rooted**: Stores vault root and relative paths, constructing
///   absolute paths via `join_dir`/`join_file`
/// - **Discovery-focused**: Used by `DiscoveryEngine::run()` instead of full
///   `Config`
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
///
/// use lithos_core::{
///     config::paths::SchemaConfigSpec,
///     fs::{DirPath, RelativePath},
/// };
///
/// let root = DirPath::try_from(PathBuf::from("/vault")).unwrap();
/// let dir_rel = RelativePath::try_from("schemas").unwrap();
/// let bank_rel =
///     RelativePath::try_from("schemas/property_bank.json").unwrap();
///
/// let spec = SchemaConfigSpec::new(root, dir_rel, bank_rel);
/// assert_eq!(
///     spec.directory().as_path(),
///     std::path::Path::new("/vault/schemas")
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SchemaConfigSpec {
    /// Vault root directory (e.g., "/vault").
    root: DirPath,
    /// Relative path to schema directory from vault root (e.g., "schemas").
    directory: RelativePath,
    /// Relative path to property bank file from vault root (e.g.,
    /// "`schemas/property_bank.json`").
    property_bank: RelativePath,
}

impl SchemaConfigSpec {
    /// Creates a new schema configuration specification.
    ///
    /// # Arguments
    ///
    /// * `root` - Vault root directory
    /// * `directory` - Relative path to schema directory from vault root
    /// * `property_bank` - Relative path to property bank file from vault root
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::{
    ///     config::paths::SchemaConfigSpec,
    ///     fs::{DirPath, RelativePath},
    /// };
    ///
    /// let root = DirPath::try_from(PathBuf::from("/vault")).unwrap();
    /// let dir = RelativePath::try_from("schemas").unwrap();
    /// let bank = RelativePath::try_from("schemas/bank.json").unwrap();
    /// let spec = SchemaConfigSpec::new(root, dir, bank);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(
        root: DirPath,
        directory: RelativePath,
        property_bank: RelativePath,
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
    #[expect(
        clippy::unreachable,
        reason = "directory paths are relative without extensions by invariant"
    )]
    #[inline]
    #[must_use]
    pub fn directory(&self) -> DirPath {
        match self.root.join_path(&self.directory) {
            FsPath::Dir(path) => path,
            FsPath::File(_) => unreachable!(
                "directory relative path unexpectedly classified as file"
            ),
        }
    }

    /// Returns the absolute property bank file path.
    #[expect(
        clippy::unreachable,
        reason = "property bank path includes a file extension by invariant"
    )]
    #[inline]
    #[must_use]
    pub fn property_bank(&self) -> FilePath {
        match self.root.join_path(&self.property_bank) {
            FsPath::File(path) => path,
            FsPath::Dir(_) => unreachable!(
                "property bank relative path unexpectedly classified as \
                 directory"
            ),
        }
    }

    /// Returns the relative schema directory path.
    #[inline]
    #[must_use]
    pub const fn directory_relative(&self) -> &RelativePath {
        &self.directory
    }

    /// Returns the relative property bank path.
    #[inline]
    #[must_use]
    pub const fn property_bank_relative(&self) -> &RelativePath {
        &self.property_bank
    }
}

#[cfg(test)]
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
                Template::default(),
                schema,
                property_bank,
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
                RelativePath::try_from(std::path::PathBuf::from(""));
            let file_name = FileName::try_from(std::path::Path::new(""));
            assert!(schemas_dir.is_err(), "Expected invalid schemas_dir");
            assert!(file_name.is_err(), "Expected invalid file name");
        }
    }

    mod schema_config_spec {
        use super::super::*;

        /// Test `SchemaConfigSpec` construction with vault root and relative
        /// paths.
        #[test]
        fn constructs_with_valid_paths() {
            use crate::fs::{DirPath, RelativePath};

            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_dir = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_dir)
                .expect("schemas dir should be created");
            std::fs::write(schemas_dir.join("bank.json"), "{}")
                .expect("property bank fixture should be writable");

            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should convert");
            let dir_rel = RelativePath::try_from("schemas").unwrap();
            let bank_rel = RelativePath::try_from("schemas/bank.json").unwrap();

            let spec = SchemaConfigSpec::new(root, dir_rel, bank_rel);

            assert_eq!(spec.directory().as_path(), schemas_dir.as_path());
            assert_eq!(
                spec.property_bank().as_path(),
                schemas_dir.join("bank.json").as_path()
            );
        }

        /// Test `SchemaConfigSpec` accessors return correct absolute and
        /// relative paths.
        #[test]
        fn accessors_return_correct_values() {
            use crate::fs::{DirPath, RelativePath};

            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_dir = root.path().join("my-schemas");
            std::fs::create_dir_all(&schemas_dir)
                .expect("schemas dir should be created");
            std::fs::write(schemas_dir.join("props.json"), "{}")
                .expect("property bank fixture should be writable");

            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should convert");
            let dir_rel = RelativePath::try_from("my-schemas").unwrap();
            let bank_rel =
                RelativePath::try_from("my-schemas/props.json").unwrap();

            let spec = SchemaConfigSpec::new(
                root.clone(),
                dir_rel.clone(),
                bank_rel.clone(),
            );

            assert_eq!(spec.root().as_path(), root.as_path());
            assert_eq!(spec.directory_relative(), &dir_rel);
            assert_eq!(spec.property_bank_relative(), &bank_rel);
            assert_eq!(spec.directory().as_path(), schemas_dir.as_path());
            assert_eq!(
                spec.property_bank().as_path(),
                schemas_dir.join("props.json").as_path()
            );
        }

        /// TDD Test 1.1: `SchemaConfigSpec` stores vault root and relative
        /// paths, constructs absolute paths via `join_dir/join_file`.
        #[test]
        fn schema_config_spec_accepts_absolute_paths() {
            use crate::fs::{DirPath, RelativePath};

            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_dir = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_dir)
                .expect("schemas dir should be created");
            let bank_path = schemas_dir.join("property_bank.json");
            std::fs::write(&bank_path, "{}")
                .expect("property bank fixture should be writable");

            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should convert");
            let dir_rel = RelativePath::try_from("schemas").unwrap();
            let bank_rel =
                RelativePath::try_from("schemas/property_bank.json").unwrap();

            let spec = SchemaConfigSpec::new(root, dir_rel, bank_rel);

            assert_eq!(spec.directory().as_path(), schemas_dir.as_path());
            assert_eq!(spec.property_bank().as_path(), bank_path.as_path());
        }
    }
}
