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
    pub location: CacheLocation,
    /// The resolved absolute directory path.
    ///
    /// Not validated for existence at construction — callers are responsible
    /// for creating the directory when needed.
    pub path: PathBuf,
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
    /// `<vault_root>/.lithos/cache/`
    ///
    /// Selected when a vault root is available and no env override is set.
    ProjectCacheDirectory,
}

/// Global (user/platform) cache location variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalCacheLocation {
    /// Sourced from the `LITHOS_CACHE_DIR` environment variable.
    ///
    /// Used when the env var is set and non-empty, regardless of whether a
    /// vault root was found.
    EnvironmentOverride,
    /// Resolved from the OS platform user-cache convention:
    /// - Linux:   `$XDG_CACHE_HOME/lithos/` or `~/.cache/lithos/`
    /// - macOS:   `~/Library/Caches/lithos/`
    /// - Windows: `%LOCALAPPDATA%\lithos\Cache\`
    ///
    /// Used as a fallback when no vault root is available and no env var is
    /// set.
    PlatformUserCache,
}

#[cfg(test)]
mod tests {
    use super::*;

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
                let root = CacheRoot {
                    location: location.clone(),
                    path: path.clone(),
                };
                assert_eq!(root.path, path);
                assert_eq!(root.location, location);
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_location_matching_provided_value() {
                let location = CacheLocation::Global(
                    GlobalCacheLocation::EnvironmentOverride,
                );
                let root = CacheRoot {
                    location: location.clone(),
                    path: std::path::PathBuf::from("/tmp/cache"),
                };
                assert_eq!(root.location, location);
            }

            #[test]
            fn returns_path_matching_provided_value() {
                let path = std::path::PathBuf::from("/does/not/exist/cache");
                let root = CacheRoot {
                    location: CacheLocation::Global(
                        GlobalCacheLocation::PlatformUserCache,
                    ),
                    path: path.clone(),
                };
                assert_eq!(root.path, path);
            }

            #[test]
            fn does_not_validate_path_existence_at_construction() {
                // Non-existent path should not cause any error at construction
                let root = CacheRoot {
                    location: CacheLocation::Local(
                        LocalCacheLocation::ProjectCacheDirectory,
                    ),
                    path: std::path::PathBuf::from(
                        "/absolutely/nonexistent/path/cache",
                    ),
                };
                // Just verifying we reach here — construction didn't panic/fail
                assert_eq!(
                    root.path,
                    std::path::PathBuf::from(
                        "/absolutely/nonexistent/path/cache"
                    )
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
