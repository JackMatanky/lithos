//! Config-side root handoff types.
//!
//! Receives [`crate::discovery::FoundRootMarker`] output from the Discovery
//! context and classifies it into [`DiscoveredConfigFile`] with the appropriate
//! Config-owned location variant.
//!
//! [`ConfigDiscoveryResult`] aggregates both local and global classified files
//! and will be consumed by [`crate::config::discovery`] as its input contract.
//! It lives here while discovery handoff remains isolated from parsing.

use std::path::{Component, Path, PathBuf};

use crate::{
    config::{
        diagnostics::ConfigWarning,
        location::{ConfigLocation, GlobalConfigLocation, LocalConfigLocation},
    },
    discovery::{
        engine::{GlobalDiscoveryResult, VaultDiscoveryResult},
        policy::GlobalSourceType,
    },
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
    /// Converts Discovery engine results into a [`ConfigDiscoveryResult`] for
    /// use by the config pipeline.
    ///
    /// Maps selected markers to [`DiscoveredConfigFile`] values with
    /// Config-owned location classification.
    pub(crate) fn from_discovery(
        vault: VaultDiscoveryResult,
        global: GlobalDiscoveryResult,
    ) -> Self {
        let global = match (global.marker, global.source) {
            (Some(marker), Some(source)) => Some(DiscoveredConfigFile {
                location: ConfigLocation::Global(
                    classify_global_config_location(source),
                ),
                base: marker.base,
                path: marker.path,
                format: marker.format,
            }),
            (Some(_), None) => {
                debug_assert!(
                    false,
                    "from_discovery: GlobalDiscoveryResult has marker but no \
                     source; dropping global config"
                );
                None
            }
            _ => None,
        };

        let local = vault.marker.map(|marker| {
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
            global,
            local,
            warnings: Vec::new(),
        }
    }
}

/// Determines the [`GlobalConfigLocation`] variant for the global source
/// returned by Discovery.
fn classify_global_config_location(
    source: GlobalSourceType,
) -> GlobalConfigLocation {
    match source {
        GlobalSourceType::EnvVar => GlobalConfigLocation::EnvironmentOverride,
        GlobalSourceType::XdgConfig => GlobalConfigLocation::XdgConfig,
        GlobalSourceType::UserConfig => GlobalConfigLocation::UserConfig,
        GlobalSourceType::SystemConfig => GlobalConfigLocation::SystemConfig,
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

        mod equality {
            use super::*;

            #[test]
            fn returns_equal_when_location_path_base_and_format_match() {
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
            fn returns_not_equal_when_locations_differ() {
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
    }

    mod config_discovery_result {
        use super::*;
        use crate::discovery::{
            engine::FoundRootMarker,
            policy::{GlobalSourceType, VaultSourceType},
        };

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

        fn vault_result() -> VaultDiscoveryResult {
            VaultDiscoveryResult {
                root: Some(PathBuf::from("/vault")),
                marker: Some(FoundRootMarker {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/.lithos/config.toml"),
                    format: StructuredFileFormat::Toml,
                }),
                alternatives: Vec::new(),
                source: Some(VaultSourceType::AscendingWalk),
                warnings: Vec::new(),
            }
        }

        fn global_result_with_source(
            source: GlobalSourceType,
        ) -> GlobalDiscoveryResult {
            GlobalDiscoveryResult {
                marker: Some(FoundRootMarker {
                    base: PathBuf::from("/config/lithos"),
                    path: PathBuf::from("/config/lithos/lithos.toml"),
                    format: StructuredFileFormat::Toml,
                }),
                alternatives: Vec::new(),
                source: Some(source),
                warnings: Vec::new(),
            }
        }

        mod equality {
            use super::*;

            #[test]
            fn returns_equal_when_global_local_and_warnings_match() {
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
                            candidates: vec![PathBuf::from(
                                "/vault/lithos.toml",
                            )],
                        },
                    )],
                };
                assert_eq!(a, b);
            }

            #[test]
            fn returns_not_equal_when_local_files_differ() {
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

        mod from_discovery {
            use super::*;

            #[test]
            fn returns_global_file_when_global_marker_is_xdg_config() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    global_result_with_source(GlobalSourceType::XdgConfig),
                );

                assert_eq!(
                    result.global,
                    Some(DiscoveredConfigFile {
                        location: ConfigLocation::Global(
                            GlobalConfigLocation::XdgConfig,
                        ),
                        base: PathBuf::from("/config/lithos"),
                        path: PathBuf::from("/config/lithos/lithos.toml"),
                        format: StructuredFileFormat::Toml,
                    })
                );
            }

            #[test]
            fn returns_environment_override_location_when_source_is_env_var() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    global_result_with_source(GlobalSourceType::EnvVar),
                );

                assert_eq!(
                    result.global.map(|f| f.location),
                    Some(ConfigLocation::Global(
                        GlobalConfigLocation::EnvironmentOverride
                    ))
                );
            }

            #[test]
            fn returns_user_config_location_when_source_is_user_config() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    global_result_with_source(GlobalSourceType::UserConfig),
                );

                assert_eq!(
                    result.global.map(|f| f.location),
                    Some(ConfigLocation::Global(
                        GlobalConfigLocation::UserConfig
                    ))
                );
            }

            #[test]
            fn returns_system_config_location_when_source_is_system_config() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    global_result_with_source(GlobalSourceType::SystemConfig),
                );

                assert_eq!(
                    result.global.map(|f| f.location),
                    Some(ConfigLocation::Global(
                        GlobalConfigLocation::SystemConfig
                    ))
                );
            }

            #[test]
            fn returns_none_global_when_global_marker_is_absent() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    GlobalDiscoveryResult::default(),
                );

                assert_eq!(result.global, None);
            }
        }
    }

    mod config_location {
        use super::*;

        mod equality {
            use super::*;

            #[test]
            fn returns_not_equal_when_global_and_local_variants_compared() {
                let global =
                    ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
                let local = ConfigLocation::Local(
                    LocalConfigLocation::HiddenRootConfigFile,
                );
                assert_ne!(global, local);
            }

            #[test]
            fn returns_equal_when_global_location_variants_match() {
                let first =
                    ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
                let second =
                    ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
                assert_eq!(first, second);
            }
        }
    }
}
