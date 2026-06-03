//! Config-side root handoff types.
//!
//! Receives [`crate::discovery::FoundRootMarker`] output from the Discovery
//! context and classifies it into [`DiscoveredConfigFile`] with the appropriate
//! [`crate::config::location::LocalConfigLocation`] variant.
//!
//! [`ConfigDiscoveryResult`] aggregates both local and global classified files
//! and will be consumed by [`crate::config::discovery`] as its input contract.
//! It lives here until the global classification side is built in Phase 2.

use std::path::{Component, Path, PathBuf};

use crate::{
    config::{
        diagnostics::ConfigWarning,
        location::{ConfigLocation, LocalConfigLocation},
    },
    discovery::engine::VaultDiscoveryResult,
    fs::format::StructuredFileFormat,
};

/// Typed representation of one discovered config file with location metadata.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DiscoveredConfigFile {
    /// Source location classification (e.g. `SystemConfig`,
    /// `HiddenRootConfigFile`).
    pub(crate) location: ConfigLocation,
    /// Base directory used for interpreting relative paths in the config file.
    ///
    /// For local vault configs, this is the vault root.
    pub(crate) base: PathBuf,
    /// Absolute canonicalized path to the discovered config file.
    pub(crate) path: PathBuf,
    /// The detected or assumed structured format of the file.
    pub(crate) format: StructuredFileFormat,
}

/// Combined discovery output for global + local config files.
///
/// `None` for `global` or `local` represents an absent file, not an error.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConfigDiscoveryResult {
    /// Selected global environment config candidate, if any.
    pub(crate) global: Option<DiscoveredConfigFile>,
    /// Selected local vault config candidate, if any.
    pub(crate) local: Option<DiscoveredConfigFile>,
    /// Non-fatal diagnostics collected during the discovery process.
    pub(crate) warnings: Vec<ConfigWarning>,
}

impl ConfigDiscoveryResult {
    /// Converts a [`VaultDiscoveryResult`] (from the Discovery engine) into a
    /// [`ConfigDiscoveryResult`] for use by the config pipeline.
    ///
    /// Maps the root marker to a [`DiscoveredConfigFile`] with the appropriate
    /// [`LocalConfigLocation`] variant. Global config is `None` until the
    /// `find_global` pipeline is implemented.
    pub(crate) fn from_vault_discovery(result: VaultDiscoveryResult) -> Self {
        let local = result.marker.map(|marker| {
            let location =
                classify_local_config_location(&marker.base, &marker.path);
            DiscoveredConfigFile {
                location: ConfigLocation::Local(location),
                base: marker.base,
                path: marker.path,
                format: marker.format,
            }
        });

        Self {
            global: None,
            local,
            warnings: Vec::new(),
        }
    }
}

/// Determines the [`LocalConfigLocation`] variant for a marker file found at
/// `path` relative to its `base` directory.
///
/// The classification uses path components, not string matching, for
/// cross-platform safety.
fn classify_local_config_location(
    base: &Path,
    path: &Path,
) -> LocalConfigLocation {
    let Ok(relative) = path.strip_prefix(base) else {
        // This should not happen: VaultRootProbe always sets base = dir and
        // path = canonical path under that dir. If it does, default to root.
        debug_assert!(
            false,
            "classify_local_config_location: path {} is not under base {}",
            path.display(),
            base.display(),
        );
        return LocalConfigLocation::RootConfigFile;
    };

    let mut components = relative.components();
    match components.next() {
        Some(Component::Normal(first)) => {
            let first_str = first.to_string_lossy();
            if first_str == ".lithos" && components.next().is_some() {
                // Path has `.lithos` as a directory component with a child:
                // e.g. `.lithos/config.toml` → ConfigDirectoryFile
                LocalConfigLocation::ConfigDirectoryFile
            } else if first_str.starts_with(".lithos.") {
                // Single-component file like `.lithos.toml` →
                // HiddenRootConfigFile
                LocalConfigLocation::HiddenRootConfigFile
            } else {
                // Any other pattern (e.g. `lithos.toml`) → RootConfigFile
                LocalConfigLocation::RootConfigFile
            }
        }
        _ => LocalConfigLocation::RootConfigFile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        diagnostics::LocalDiscoveryWarning,
        location::{GlobalConfigLocation, LocalConfigLocation},
    };

    mod discovered_config_file {
        use super::*;

        #[test]
        fn returns_true_when_files_have_same_location_path_and_format() {
            let a = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(a, b);
        }

        #[test]
        fn returns_false_when_locations_differ() {
            let a = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::HiddenRootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_ne!(a, b);
        }
    }

    mod config_discovery_result {
        use super::*;

        fn local_file() -> DiscoveredConfigFile {
            DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::ConfigDirectoryFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/config.toml"),
                format: StructuredFileFormat::Toml,
            }
        }

        #[test]
        fn returns_true_when_results_match() {
            let warning =
                ConfigWarning::Local(LocalDiscoveryWarning::Ambiguity {
                    candidates: vec![PathBuf::from("/vault/lithos.toml")],
                });
            let a = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: vec![warning],
            };
            let b = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: vec![ConfigWarning::Local(
                    LocalDiscoveryWarning::Ambiguity {
                        candidates: vec![PathBuf::from("/vault/lithos.toml")],
                    },
                )],
            };
            assert_eq!(a, b);
        }

        #[test]
        fn returns_false_when_local_files_differ() {
            let with_local = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: Vec::new(),
            };
            let without_local = ConfigDiscoveryResult {
                global: None,
                local: None,
                warnings: Vec::new(),
            };
            assert_ne!(with_local, without_local);
        }
    }

    mod config_location {
        use super::*;

        #[test]
        fn returns_false_when_global_and_local_variants_compared() {
            let global =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let local = ConfigLocation::Local(
                LocalConfigLocation::HiddenRootConfigFile,
            );
            assert_ne!(global, local);
        }

        #[test]
        fn returns_true_when_global_locations_match() {
            let first = ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let second =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            assert_eq!(first, second);
        }
    }
}
