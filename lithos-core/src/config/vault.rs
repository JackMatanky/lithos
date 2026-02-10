//! Vault-specific overrides and metadata.
//!
//! This module defines the [`Vault`] configuration, which contains
//! vault-specific settings and overrides for global defaults. It also
//! manages [`VaultId`] and [`VaultRoot`].

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived structs"
)]

use std::path::{Path, PathBuf};

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{Cache, PropertyBank, Schema, Template},
    task::Task,
};

// ----------------------------------------------------------- //
//                     Public Domain Types                     //
// ----------------------------------------------------------- //

/// Vault-specific configuration overrides.
///
/// `Vault` contains settings that are specific to a single vault and
/// override the global defaults. It covers paths, frontmatter, and
/// logging settings.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::vault::Vault;
///
/// let vault = Vault::default();
/// assert!(vault.logging().is_none());
/// ```
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
pub struct Vault {
    /// Overridden logging settings.
    logging: Option<Logging>,
    /// Overridden paths settings.
    paths: Paths,
    /// Overridden frontmatter settings.
    frontmatter: Option<Frontmatter>,
    /// Overridden task settings.
    task: Option<Task>,
}

impl Vault {
    /// Create vault-specific configuration.
    #[inline]
    #[must_use]
    pub const fn new(
        logging: Option<Logging>,
        paths: Paths,
        frontmatter: Option<Frontmatter>,
        task: Option<Task>,
    ) -> Self {
        Self {
            logging,
            paths,
            frontmatter,
            task,
        }
    }

    /// Return the overridden paths settings.
    #[inline]
    #[must_use]
    pub const fn paths(&self) -> &Paths {
        &self.paths
    }

    /// Return the overridden frontmatter settings, if set.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Return the overridden task settings, if set.
    #[inline]
    #[must_use]
    pub fn task(&self) -> Option<&Task> {
        self.task.as_ref()
    }

    /// Return the overridden logging settings, if set.
    #[inline]
    #[must_use]
    pub fn logging(&self) -> Option<&Logging> {
        self.logging.as_ref()
    }
}

/// Metadata for a specific vault.
///
/// This struct holds the identity, root path, and versioning information
/// for a vault. It is stored as part of the [`Config`] aggregate.
///
/// # Invariants
///
/// - **Vault Identity**: Every vault must have a unique UUID v7 identifier.
/// - **Vault Root**: The root path of a vault must be a non-empty directory
///   path.
/// - **Schema Version**: Every vault is associated with a specific schema
///   version.
///
/// # Examples
///
/// ```rust
/// # use lithos_core::config::vault::{Metadata, VaultId, VaultRoot};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = Metadata::new(
///     VaultId::new(),
///     VaultRoot::try_new("/vaults/work".into())?,
///     Some("Work".into()),
///     None,
/// )?;
///
/// assert_eq!(metadata.name(), "Work");
/// # Ok(())
/// # }
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
pub struct Metadata {
    /// Unique identity of the vault.
    id: VaultId,
    /// Absolute path to the vault root on disk.
    root: VaultRoot,
    /// Human-readable name of the vault.
    name: VaultName,
    /// Version of the vault schema/Lithos that created it.
    version: AppVersion,
}

impl Metadata {
    /// Creates vault metadata.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the vault root validation fails or if the
    /// default schema version cannot be constructed.
    #[inline]
    pub fn new(
        id: VaultId,
        root: VaultRoot,
        name: Option<VaultName>,
        version: Option<AppVersion>,
    ) -> Result<Self, ConfigError> {
        let name = name.unwrap_or_else(|| VaultName::from_root(&root));
        let version =
            match version {
                Some(v) => v,
                None => AppVersion::try_new(env!("CARGO_PKG_VERSION"))
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "version".to_owned().into(),
                        message: format!("default version invalid: {e}").into(),
                    })?,
            };

        Ok(Self {
            id,
            root,
            name,
            version,
        })
    }

    /// Return the vault identity.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> VaultId {
        self.id
    }

    /// Return the vault root path.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> &VaultRoot {
        &self.root
    }

    /// Return the vault name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the vault version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &AppVersion {
        &self.version
    }
}

/// Vault-specific paths configuration (overrides).
///
/// Unlike the resolved [`crate::config::paths::Paths`], this struct uses
/// [`Option`] for all fields to represent partial overrides of global path
/// settings.
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
    /// Overridden cache settings.
    pub cache: Option<Cache>,
    /// Overridden template settings.
    pub template: Option<Template>,
    /// Overridden schema settings.
    pub schema: Option<Schema>,
    /// Overridden property bank filename.
    pub property_bank: Option<PropertyBank>,
}

impl Paths {
    /// Create vault-specific paths settings.
    #[inline]
    #[must_use]
    pub const fn new(
        cache: Option<Cache>,
        template: Option<Template>,
        schema: Option<Schema>,
        property_bank: Option<PropertyBank>,
    ) -> Self {
        Self {
            cache,
            template,
            schema,
            property_bank,
        }
    }
}

// ----------------------------------------------------------- //
//                    Building Block Types                     //
// ----------------------------------------------------------- //

/// Vault unique identity using UUID v7.
///
/// UUID v7 is used for its time-ordered properties, which helps with
/// database indexing and debugging.
#[derive(
    Debug,
    Clone,
    Copy,
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
#[non_exhaustive]
pub struct VaultId(uuid::Uuid);

impl VaultId {
    /// Create a new unique vault identity.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Return the raw UUID.
    #[inline]
    #[must_use]
    pub const fn uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for VaultId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VaultId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated, absolute path to a vault root.
///
/// # Invariants
///
/// - Must be a non-empty path.
/// - Should ideally be an absolute path (checked at the application level).
///
/// # Errors
///
/// Returns [`ConfigError::ValidationFailed`] if the path is empty.
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
pub struct VaultRoot(#[rkyv(with = rkyv::with::AsString)] PathBuf);

impl VaultRoot {
    /// Creates a validated vault root path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is empty.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        if path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_root".to_owned().into(),
                message: "path cannot be empty".to_owned().into(),
            });
        }
        Ok(Self(path))
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Return this path as a string suitable for DB key lookups.
    #[inline]
    #[must_use]
    pub fn as_key(&self) -> String {
        self.as_path().to_string_lossy().into_owned()
    }
}

impl TryFrom<String> for VaultRoot {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<VaultRoot> for String {
    #[inline]
    fn from(root: VaultRoot) -> Self {
        root.0.to_string_lossy().into_owned()
    }
}

impl Default for VaultRoot {
    #[inline]
    #[expect(
        clippy::expect_used,
        clippy::disallowed_methods,
        reason = "Root path is guaranteed non-empty"
    )]
    fn default() -> Self {
        Self::try_new(PathBuf::from("/")).expect("root path is non-empty")
    }
}

/// Version identifier for the Lithos application.
///
/// This type ensures that version strings are not empty and represent
/// a valid application version.
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
pub struct AppVersion(Box<str>);

impl AppVersion {
    /// Creates a new application version.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the version string is
    /// empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "version".to_owned().into(),
                message: "version cannot be empty".to_owned().into(),
            });
        }
        Ok(Self(value))
    }

    /// Return the version as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppVersion {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AppVersion> for String {
    #[inline]
    fn from(version: AppVersion) -> Self {
        version.0.into_string()
    }
}

impl std::fmt::Display for AppVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated vault name.
///
/// This type ensures that vault names are not empty and are user-friendly.
/// The name can be explicitly provided or derived from the vault root path.
///
/// # Invariants
///
/// - Must not be empty.
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
pub struct VaultName(Box<str>);

impl VaultName {
    /// Creates a validated vault name from explicit user input.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "name".into(),
                message: "vault name cannot be empty".into(),
            });
        }
        Ok(Self(value))
    }

    /// Derives a vault name from the last component of the vault root path.
    #[inline]
    #[must_use]
    pub fn from_root(root: &VaultRoot) -> Self {
        let name = root.as_path().file_name().map_or_else(
            || "unnamed".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        Self(name.into_boxed_str())
    }

    /// Return the name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for VaultName {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<&str> for VaultName {
    #[inline]
    fn from(value: &str) -> Self {
        // Safe because &str is always non-empty when converted from string
        // literal
        Self(value.to_owned().into_boxed_str())
    }
}

impl From<VaultName> for String {
    #[inline]
    fn from(name: VaultName) -> Self {
        name.0.into_string()
    }
}

impl std::fmt::Display for VaultName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::disallowed_methods,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::logging::{LogLevel, Logging};

    mod fixtures {
        use std::path::PathBuf;

        use super::super::*;

        pub fn vault_root(path: &str) -> VaultRoot {
            VaultRoot::try_new(PathBuf::from(path)).expect("valid vault root")
        }

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn vault_new_constructs_with_given_values() {
            let paths = Paths::default();
            let logging = Logging::new(LogLevel::Debug);
            let vault =
                Vault::new(Some(logging.clone()), paths.clone(), None, None);

            assert_eq!(vault.paths(), &paths);
            assert_eq!(vault.logging(), Some(&logging));
        }

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn metadata_new_derives_from_vault_path()
        -> Result<(), Box<dyn std::error::Error>> {
            // GIVEN: a vault path
            let vault_root = fixtures::vault_root("/vaults/work");

            // WHEN: building metadata from the path
            let metadata = Metadata::new(
                fixtures::vault_id(),
                vault_root.clone(),
                None,
                None,
            )?;

            // THEN: version and name defaults are applied
            assert_eq!(
                metadata.version().as_str(),
                env!("CARGO_PKG_VERSION"),
                "Expected version default to be set"
            );
            assert_eq!(
                metadata.name(),
                "work",
                "Expected vault name to default to directory basename"
            );
            assert_eq!(
                metadata.root().as_path(),
                vault_root.as_path(),
                "Expected path to match input"
            );

            Ok(())
        }

        #[test]
        fn cache_dir_rejects_empty() {
            // GIVEN: empty cache_dir
            use crate::config::paths::RelativePath;
            let result = RelativePath::try_new(PathBuf::from(""));

            // THEN: validation fails for cache_dir
            assert!(
                result.is_err(),
                "Expected validation failure for cache_dir"
            );
        }

        #[test]
        fn vault_root_rejects_empty() {
            // GIVEN: an empty vault path
            let vault_path = "";

            // WHEN: building metadata from the empty path
            let result = VaultRoot::try_new(PathBuf::from(vault_path));

            // THEN: validation fails with a required field error
            assert!(
                result.is_err(),
                "Expected validation failure for empty vault_path"
            );
        }
    }

    mod validation {
        use super::*;
        use crate::config::paths::RelativePath;

        #[test]
        fn templates_dir_rejects_absolute() {
            // GIVEN: invalid template dir (absolute)
            let result = RelativePath::try_new(PathBuf::from("/abs"));

            // THEN: it fails
            assert!(
                result.is_err(),
                "RelativePath should reject absolute paths"
            );
        }
    }
}
