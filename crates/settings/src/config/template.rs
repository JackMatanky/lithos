//! Template configuration types.
//!
//! This module owns the template portion of resolved configuration:
//! [`TemplateDir`] stores the declarative relative template directory,
//! [`TemplateConfig`] stores the validated resolved config value, and
//! [`TemplateConfigSpec`] exposes the narrowed contract used by template
//! discovery and loading code.
//!
//! Template paths are intentionally declarative. They use [`RelativeDirPath`]
//! rather than a filesystem-validated directory so configuration can be loaded
//! even before templates exist. Consumers resolve the declaration against a
//! vault root through [`TemplateConfigSpec`].
//!
//! # Examples
//!
//! ```rust
//! use trace_settings::config::template::{TemplateConfig, TemplateConfigSpec, TemplateDir};
//! use trace_fs::{DirPath, path::RelativeDirPath};
//!
//! # fn example(root: DirPath) -> Result<(), Box<dyn std::error::Error>> {
//! let template_dir = TemplateDir::new(RelativeDirPath::try_new("templates")?);
//! let config = TemplateConfig::new(template_dir);
//! let spec = TemplateConfigSpec::new(
//!     root,
//!     config.template_dir().as_relative_dir().clone(),
//! );
//!
//! assert_eq!(spec.as_relative_dir().as_str(), "templates");
//! # Ok(())
//! # }
//! ```

use rkyv::{Archive, Deserialize, Serialize};
use trace_fs::{DirPath, PathKey, path::RelativeDirPath};

use super::error::ConfigError;

/// Declarative template directory configuration.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TemplateDir(RelativeDirPath);

impl TemplateDir {
    /// Creates a `TemplateDir` from an already-validated relative directory
    /// path.
    #[inline]
    #[must_use]
    pub const fn new(templates_dir: RelativeDirPath) -> Self {
        Self(templates_dir)
    }

    /// Creates a validated template directory path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is absolute,
    /// empty, or contains parent directory traversal.
    #[inline]
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError> {
        let value =
            path.to_str().ok_or_else(|| ConfigError::ValidationFailed {
                field: "templates_dir".into(),
                message: "Non-UTF-8 path".into(),
            })?;

        RelativeDirPath::try_new(value).map(Self).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "templates_dir".into(),
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

impl Default for TemplateDir {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Default directory literal is guaranteed valid"
    )]
    fn default() -> Self {
        Self(
            RelativeDirPath::try_new("templates")
                .expect("default path literal must be valid"),
        )
    }
}

/// Resolved template configuration.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct TemplateConfig {
    template_dir: TemplateDir,
}

impl TemplateConfig {
    /// Creates a resolved template configuration.
    #[inline]
    #[must_use]
    pub const fn new(template_dir: TemplateDir) -> Self {
        Self {
            template_dir,
        }
    }

    /// Returns the template directory.
    #[inline]
    #[must_use]
    pub const fn template_dir(&self) -> &TemplateDir {
        &self.template_dir
    }
}

impl Default for TemplateConfig {
    #[inline]
    fn default() -> Self {
        Self::new(TemplateDir::default())
    }
}

/// Template configuration specification for template discovery.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TemplateConfigSpec {
    /// Vault root directory.
    root: DirPath,
    /// Relative path to template directory from vault root.
    directory: RelativeDirPath,
}

impl TemplateConfigSpec {
    /// Creates a new template configuration specification.
    #[inline]
    #[must_use]
    pub const fn new(root: DirPath, directory: RelativeDirPath) -> Self {
        Self {
            root,
            directory,
        }
    }

    /// Returns the vault root directory.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> &DirPath {
        &self.root
    }

    /// Returns the relative template directory declaration.
    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.directory
    }

    /// Returns the absolute template directory path.
    ///
    /// # Errors
    /// Returns an error if the derived directory path does not currently exist.
    #[inline]
    pub fn to_dir_path(&self) -> Result<DirPath, trace_fs::PathError> {
        self.root.append_dir(&self.directory)
    }

    /// Returns the template directory persistence key.
    ///
    /// # Errors
    /// Returns an error when the absolute template directory path cannot be
    /// derived or key conversion fails.
    #[inline]
    pub fn to_path_key(&self) -> Result<PathKey, trace_fs::PathError> {
        self.to_dir_path()?.as_key(self.root())
    }
}

#[cfg(test)]
mod tests {
    use trace_fs::path::RelativeDirPath;

    use super::*;

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_template_dir() {
            let template_dir = TemplateDir::default();

            assert_eq!(
                template_dir.as_relative_dir().as_str(),
                "templates",
                "default template dir should match the documented default"
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_template_dir_when_relative_path_is_valid() {
            let result =
                TemplateDir::try_new(std::path::Path::new("custom-templates"));

            assert!(
                result.is_ok(),
                "valid relative template path should construct successfully: \
                 {:?}",
                result.err()
            );
            assert_eq!(
                result
                    .expect("result checked as ok")
                    .as_relative_dir()
                    .as_str(),
                "custom-templates",
                "template dir should preserve the validated relative \
                 declaration"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = TemplateDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty template path should be rejected");
        }

        #[test]
        fn rejects_absolute_path() {
            let result =
                TemplateDir::try_new(std::path::Path::new("/tmp/templates"));

            assert!(
                result.is_err(),
                "absolute template path should be rejected"
            );
        }

        #[test]
        fn rejects_parent_traversal() {
            let result =
                TemplateDir::try_new(std::path::Path::new("../templates"));

            assert!(
                result.is_err(),
                "template path escaping the vault root should be rejected"
            );
        }
    }

    mod template_config {
        use super::*;

        #[test]
        fn returns_default_template_dir() {
            let config = TemplateConfig::default();

            assert_eq!(
                config.template_dir().as_relative_dir().as_str(),
                "templates",
                "template config should expose the default template directory"
            );
        }

        #[test]
        fn returns_configured_template_dir() {
            let template_dir =
                TemplateDir::try_new(std::path::Path::new("custom-templates"))
                    .expect("fixture template dir should be valid");

            let config = TemplateConfig::new(template_dir);

            assert_eq!(
                config.template_dir().as_relative_dir().as_str(),
                "custom-templates",
                "template config should retain the configured template \
                 directory"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_template_dir_from_relative_dir_path() {
            let relative = RelativeDirPath::try_new("templates")
                .expect("fixture relative path should be valid");

            let template_dir = TemplateDir::new(relative);

            assert_eq!(
                template_dir.as_relative_dir().as_str(),
                "templates",
                "constructor should retain the validated relative dir"
            );
        }
    }

    mod template_config_spec {
        use trace_fs::DirPath;

        use super::*;

        #[test]
        fn returns_relative_dir_without_requiring_target_to_exist() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("templates")
                .expect("fixture relative path should be valid");

            let spec = TemplateConfigSpec::new(root, directory);

            assert_eq!(
                spec.as_relative_dir().as_str(),
                "templates",
                "template spec should retain declarative relative directory"
            );
        }

        #[test]
        fn returns_dir_path_when_root_and_relative_dir_are_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let template_path = root.path().join("templates");
            std::fs::create_dir_all(&template_path)
                .expect("template dir fixture should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("templates")
                .expect("fixture relative path should be valid");

            let spec = TemplateConfigSpec::new(root, directory);

            let result = spec.to_dir_path();

            assert!(
                result.is_ok(),
                "existing template dir should resolve successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_path(),
                template_path.as_path(),
                "template spec should resolve relative directory against \
                 vault root"
            );
        }

        #[test]
        fn returns_path_key_when_root_scoped_dir_is_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let template_path = root.path().join("templates");
            std::fs::create_dir_all(&template_path)
                .expect("template dir fixture should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new("templates")
                .expect("fixture relative path should be valid");

            let spec = TemplateConfigSpec::new(root, directory);

            let result = spec.to_path_key();

            assert!(
                result.is_ok(),
                "existing template dir should convert to path key: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_str(),
                "templates",
                "template spec should return vault-relative path key"
            );
        }
    }
}
