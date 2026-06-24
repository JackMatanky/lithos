//! Cache root location types for the discovery pipeline.
//!
//! These types represent the resolved cache root and its provenance — how and
//! where the cache directory was determined.

use std::path::PathBuf;

/// The resolved cache root with its provenance.
///
/// `path` is a directory path (not a file) and is not validated for existence
/// at construction — callers are responsible for creating it when needed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheRoot {
    /// The strategy that produced this cache root.
    location: CacheLocation,
    /// The resolved absolute directory path.
    ///
    /// Not validated for existence at construction — callers are responsible
    /// for creating the directory when needed.
    path: PathBuf,
}

impl CacheRoot {
    /// Creates a new `CacheRoot` from its provenance and resolved path.
    ///
    /// The path is not validated for existence at construction — callers are
    /// responsible for creating the directory when needed.
    #[inline]
    #[must_use]
    pub fn new(location: CacheLocation, path: PathBuf) -> Self {
        Self {
            location,
            path,
        }
    }

    /// Returns the cache location strategy that produced this root.
    #[inline]
    #[must_use]
    pub fn location(&self) -> &CacheLocation {
        &self.location
    }

    /// Returns the resolved absolute directory path.
    ///
    /// The directory is not guaranteed to exist — callers are responsible for
    /// creating it when needed.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Top-level branch: is the cache local to the vault or at a global/platform
/// level?
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CacheLocation {
    /// Cache is vault-scoped (local to the project).
    Local(LocalCacheLocation),
    /// Cache is at a global or platform level.
    Global(GlobalCacheLocation),
}

/// Local (vault-scoped) cache location variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalCacheLocation {
    /// `<vault_root>/.traces/cache/`
    ///
    /// Selected when a vault root is available and no env override is set.
    ProjectCacheDirectory,
}

/// Global (user/platform) cache location variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalCacheLocation {
    /// Sourced from the `TRACES_CACHE_DIR` environment variable.
    ///
    /// Used when the env var is set and non-empty, regardless of whether a
    /// vault root was found.
    EnvironmentOverride,
    /// Resolved from the OS platform user-cache convention:
    /// - Linux:   `$XDG_CACHE_HOME/traces/` or `~/.cache/traces/`
    /// - macOS:   `~/Library/Caches/traces/`
    /// - Windows: `%LOCALAPPDATA%\traces\Cache\`
    ///
    /// Used as a fallback when no vault root is available and no env var is
    /// set.
    PlatformUserCache,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Shared test fixture. Returns a `CacheRoot` with a synthetic
    /// `PlatformUserCache` location and `/tmp/placeholder-cache` path.
    /// Use wherever a `CacheRoot` is needed but its value is not under test.
    pub(crate) fn placeholder_cache_root() -> CacheRoot {
        CacheRoot::new(
            CacheLocation::Global(GlobalCacheLocation::PlatformUserCache),
            std::path::PathBuf::from("/tmp/placeholder-cache"),
        )
    }

    mod cache_root {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_path_and_location_as_provided() {
                let path = std::path::PathBuf::from("/tmp/my/cache");
                let location = CacheLocation::Local(
                    LocalCacheLocation::ProjectCacheDirectory,
                );
                let root = CacheRoot::new(location.clone(), path.clone());
                assert_eq!(root.path(), path.as_path());
                assert_eq!(root.location(), &location);
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_location_matching_provided_value() {
                let location = CacheLocation::Global(
                    GlobalCacheLocation::EnvironmentOverride,
                );
                let root = CacheRoot::new(
                    location.clone(),
                    std::path::PathBuf::from("/tmp/cache"),
                );
                assert_eq!(root.location(), &location);
            }

            #[test]
            fn returns_path_matching_provided_value() {
                let path = std::path::PathBuf::from("/does/not/exist/cache");
                let root = CacheRoot::new(
                    CacheLocation::Global(
                        GlobalCacheLocation::PlatformUserCache,
                    ),
                    path.clone(),
                );
                assert_eq!(root.path(), path.as_path());
            }

            #[test]
            fn does_not_validate_path_existence_at_construction() {
                // Non-existent path should not cause any error at construction
                let root = CacheRoot::new(
                    CacheLocation::Local(
                        LocalCacheLocation::ProjectCacheDirectory,
                    ),
                    std::path::PathBuf::from(
                        "/absolutely/nonexistent/path/cache",
                    ),
                );
                // Just verifying we reach here — construction didn't panic/fail
                assert_eq!(
                    root.path(),
                    std::path::Path::new("/absolutely/nonexistent/path/cache")
                );
            }
        }
    }

    mod cache_location {
        use super::*;

        #[test]
        fn local_variant_wraps_local_cache_location() {
            let loc =
                CacheLocation::Local(LocalCacheLocation::ProjectCacheDirectory);
            assert!(matches!(
                loc,
                CacheLocation::Local(LocalCacheLocation::ProjectCacheDirectory)
            ));
        }

        #[test]
        fn global_env_override_variant_is_constructible() {
            let loc =
                CacheLocation::Global(GlobalCacheLocation::EnvironmentOverride);
            assert!(matches!(
                loc,
                CacheLocation::Global(GlobalCacheLocation::EnvironmentOverride)
            ));
        }

        #[test]
        fn global_platform_user_cache_variant_is_constructible() {
            let loc =
                CacheLocation::Global(GlobalCacheLocation::PlatformUserCache);
            assert!(matches!(
                loc,
                CacheLocation::Global(GlobalCacheLocation::PlatformUserCache)
            ));
        }
    }
}
