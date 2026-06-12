//! Cache configuration types.
//!
//! This module owns the cache portion of resolved configuration:
//! [`CacheDir`] stores the declarative relative cache directory,
//! [`CacheConfig`] stores the validated resolved config value, and
//! [`CacheConfigSpec`] exposes the narrowed contract consumed by cache-facing
//! code.
//!
//! Config paths remain declarative here. They use [`RelativeDirPath`] instead
//! of filesystem-validated paths so configuration can be parsed before the
//! cache directory exists on disk. Consumers that need filesystem paths should
//! use [`CacheConfigSpec`] to resolve the declaration against a vault root.
//!
//! # Examples
//!
//! ```rust
//! use lithos_core::{
//!     config::cache::{CacheConfig, CacheConfigSpec, CacheDir},
//!     fs::{DirPath, path::RelativeDirPath},
//! };
//!
//! # fn example(root: DirPath) -> Result<(), Box<dyn std::error::Error>> {
//! let cache_dir = CacheDir::new(RelativeDirPath::try_new(".cache")?);
//! let config = CacheConfig::new(cache_dir);
//! let spec = CacheConfigSpec::new(
//!     root,
//!     config.cache_dir().as_relative_dir().clone(),
//! );
//!
//! assert_eq!(spec.as_relative_dir().as_str(), ".cache");
//! # Ok(())
//! # }
//! ```

use rkyv::{Archive, Deserialize, Serialize};

use super::error::ConfigError;
use crate::fs::{DirPath, PathKey, path::RelativeDirPath};

/// Declarative cache directory configuration.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct CacheDir(RelativeDirPath);

impl CacheDir {
    /// Creates a `CacheDir` from a validated [`RelativeDirPath`].
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: RelativeDirPath) -> Self {
        Self(cache_dir)
    }

    /// Validates and constructs a `CacheDir` from a raw path.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationFailed`] if the path is empty,
    /// absolute, contains parent traversal components (`..`), or is not
    /// valid UTF-8.
    #[inline]
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError> {
        let value =
            path.to_str().ok_or_else(|| ConfigError::ValidationFailed {
                field: "cache_dir".into(),
                message: "Non-UTF-8 path".into(),
            })?;

        RelativeDirPath::try_new(value).map(Self).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "cache_dir".into(),
                message: error.to_string().into(),
            }
        })
    }

    /// Returns the inner validated relative directory path.
    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.0
    }
}

impl Default for CacheDir {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Default directory literal is guaranteed valid"
    )]
    fn default() -> Self {
        Self(
            RelativeDirPath::try_new(".cache")
                .expect("default path literal must be valid"),
        )
    }
}

/// Resolved cache configuration.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct CacheConfig {
    cache_dir: CacheDir,
}

impl CacheConfig {
    /// Creates a `CacheConfig` with the given cache directory.
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: CacheDir) -> Self {
        Self {
            cache_dir,
        }
    }

    /// Returns the configured cache directory.
    #[inline]
    #[must_use]
    pub const fn cache_dir(&self) -> &CacheDir {
        &self.cache_dir
    }
}

impl Default for CacheConfig {
    #[inline]
    fn default() -> Self {
        Self::new(CacheDir::default())
    }
}

/// Cache configuration specification for cache consumers.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CacheConfigSpec {
    root: DirPath,
    directory: RelativeDirPath,
}

impl CacheConfigSpec {
    /// Creates a `CacheConfigSpec` from a vault root and a relative cache
    /// directory.
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

    /// Returns the declarative relative cache directory (does not require the
    /// directory to exist).
    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.directory
    }

    /// Resolves the cache directory against the vault root, requiring the
    /// directory to exist.
    ///
    /// # Errors
    ///
    /// Returns [`crate::fs::PathError`] if the resolved path does not exist or
    /// is not a directory.
    #[inline]
    pub fn to_dir_path(&self) -> Result<DirPath, crate::fs::PathError> {
        self.root.append_dir(&self.directory)
    }

    /// Returns the vault-relative path key for the cache directory.
    ///
    /// # Errors
    ///
    /// Returns [`crate::fs::PathError`] if the resolved path does not exist, is
    /// not a directory, or cannot be expressed as a relative key from the vault
    /// root.
    #[inline]
    pub fn to_path_key(&self) -> Result<PathKey, crate::fs::PathError> {
        self.to_dir_path()?.as_key(self.root())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::path::RelativeDirPath;

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_cache_dir() {
            let cache_dir = CacheDir::default();

            assert_eq!(
                cache_dir.as_relative_dir().as_str(),
                ".cache",
                "default cache dir should match the documented default"
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_cache_dir_when_relative_path_is_valid() {
            let result =
                CacheDir::try_new(std::path::Path::new(".lithos-cache"));

            assert!(
                result.is_ok(),
                "valid relative cache path should construct successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result
                    .expect("result checked as ok")
                    .as_relative_dir()
                    .as_str(),
                ".lithos-cache",
                "cache dir should preserve the validated relative declaration"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = CacheDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty cache path should be rejected");
        }

        #[test]
        fn rejects_absolute_path() {
            let result = CacheDir::try_new(std::path::Path::new("/tmp/cache"));

            assert!(result.is_err(), "absolute cache path should be rejected");
        }

        #[test]
        fn rejects_parent_traversal() {
            let result = CacheDir::try_new(std::path::Path::new("../cache"));

            assert!(
                result.is_err(),
                "cache path escaping the vault root should be rejected"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_cache_dir_from_relative_dir_path() {
            let relative = RelativeDirPath::try_new("cache")
                .expect("fixture relative path should be valid");

            let cache_dir = CacheDir::new(relative);

            assert_eq!(
                cache_dir.as_relative_dir().as_str(),
                "cache",
                "constructor should retain the validated relative dir"
            );
        }
    }

    mod cache_config {
        use super::*;

        #[test]
        fn returns_default_cache_dir() {
            let config = CacheConfig::default();

            assert_eq!(
                config.cache_dir().as_relative_dir().as_str(),
                ".cache",
                "cache config should expose the default cache directory"
            );
        }

        #[test]
        fn returns_configured_cache_dir() {
            let cache_dir =
                CacheDir::try_new(std::path::Path::new("custom-cache"))
                    .expect("fixture cache dir should be valid");

            let config = CacheConfig::new(cache_dir);

            assert_eq!(
                config.cache_dir().as_relative_dir().as_str(),
                "custom-cache",
                "cache config should retain the configured cache directory"
            );
        }
    }

    mod cache_config_spec {
        use super::*;
        use crate::fs::DirPath;

        #[test]
        fn returns_relative_dir_without_requiring_target_to_exist() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new(".cache")
                .expect("fixture relative path should be valid");

            let spec = CacheConfigSpec::new(root, directory);

            assert_eq!(
                spec.as_relative_dir().as_str(),
                ".cache",
                "cache spec should retain declarative relative directory"
            );
        }

        #[test]
        fn returns_dir_path_when_root_and_relative_dir_are_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let cache_path = root.path().join(".cache");
            std::fs::create_dir_all(&cache_path)
                .expect("cache dir fixture should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new(".cache")
                .expect("fixture relative path should be valid");

            let spec = CacheConfigSpec::new(root, directory);

            let result = spec.to_dir_path();

            assert!(
                result.is_ok(),
                "existing cache dir should resolve successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_path(),
                cache_path.as_path(),
                "cache spec should resolve relative directory against vault \
                 root"
            );
        }

        #[test]
        fn returns_path_key_when_root_scoped_dir_is_valid() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let cache_path = root.path().join(".cache");
            std::fs::create_dir_all(&cache_path)
                .expect("cache dir fixture should be created");
            let root = DirPath::try_from(root.path().to_path_buf())
                .expect("temp root should be valid");
            let directory = RelativeDirPath::try_new(".cache")
                .expect("fixture relative path should be valid");

            let spec = CacheConfigSpec::new(root, directory);

            let result = spec.to_path_key();

            assert!(
                result.is_ok(),
                "existing cache dir should convert to path key: {:?}",
                result.err()
            );
            assert_eq!(
                result.expect("result checked as ok").as_str(),
                ".cache",
                "cache spec should return vault-relative path key"
            );
        }
    }
}
