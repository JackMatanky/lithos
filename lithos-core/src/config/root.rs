//! Config-side path handoff types.
//!
//! [`ConfigDiscoveryResult`] aggregates both local and global selected files
//! and will be consumed by [`crate::config::discovery`] as its input contract.
//! It lives here while discovery handoff remains isolated from parsing.

use std::path::PathBuf;

use crate::{
    discovery::engine::{GlobalDiscoveryResult, VaultDiscoveryResult},
    fs::format::StructuredFileFormat,
};

/// Typed representation of one discovered config file path.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DiscoveredConfigFile {
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
}

impl ConfigDiscoveryResult {
    /// Converts Discovery engine results into a [`ConfigDiscoveryResult`] for
    /// use by the config pipeline.
    ///
    /// Maps selected markers to path-only [`DiscoveredConfigFile`] values.
    pub(crate) fn from_discovery(
        vault: VaultDiscoveryResult,
        global: GlobalDiscoveryResult,
    ) -> Self {
        let global = global.marker.map(|marker| DiscoveredConfigFile {
            base: marker.base,
            path: marker.path,
            format: marker.format,
        });

        let local = vault.marker.map(|marker| DiscoveredConfigFile {
            base: marker.base,
            path: marker.path,
            format: marker.format,
        });

        Self {
            global,
            local,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod discovered_config_file {
        use super::*;

        mod equality {
            use super::*;

            #[test]
            fn returns_equal_when_path_base_and_format_match() {
                let a = DiscoveredConfigFile {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/lithos.toml"),
                    format: StructuredFileFormat::Toml,
                };
                let b = DiscoveredConfigFile {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/lithos.toml"),
                    format: StructuredFileFormat::Toml,
                };
                assert_eq!(a, b);
            }

            #[test]
            fn returns_not_equal_when_paths_differ() {
                let a = DiscoveredConfigFile {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/lithos.toml"),
                    format: StructuredFileFormat::Toml,
                };
                let b = DiscoveredConfigFile {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/.lithos.toml"),
                    format: StructuredFileFormat::Toml,
                };
                assert_ne!(a, b);
            }
        }
    }

    mod config_discovery_result {
        use super::*;
        use crate::discovery::engine::DiscoveredMarker;

        fn local_file() -> DiscoveredConfigFile {
            DiscoveredConfigFile {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/config.toml"),
                format: StructuredFileFormat::Toml,
            }
        }

        fn vault_result() -> VaultDiscoveryResult {
            VaultDiscoveryResult {
                root: Some(PathBuf::from("/vault")),
                marker: Some(DiscoveredMarker {
                    base: PathBuf::from("/vault"),
                    path: PathBuf::from("/vault/.lithos/config.toml"),
                    format: StructuredFileFormat::Toml,
                }),
                alternatives: Vec::new(),
                source: None,
                warnings: Vec::new(),
            }
        }

        fn global_result() -> GlobalDiscoveryResult {
            GlobalDiscoveryResult {
                marker: Some(DiscoveredMarker {
                    base: PathBuf::from("/config/lithos"),
                    path: PathBuf::from("/config/lithos/lithos.toml"),
                    format: StructuredFileFormat::Toml,
                }),
                alternatives: Vec::new(),
                source: None,
            }
        }

        mod equality {
            use super::*;

            #[test]
            fn returns_equal_when_global_and_local_match() {
                let a = ConfigDiscoveryResult {
                    global: None,
                    local: Some(local_file()),
                };
                let b = ConfigDiscoveryResult {
                    global: None,
                    local: Some(local_file()),
                };
                assert_eq!(a, b);
            }

            #[test]
            fn returns_not_equal_when_local_files_differ() {
                let with_local = ConfigDiscoveryResult {
                    global: None,
                    local: Some(local_file()),
                };
                let without_local = ConfigDiscoveryResult {
                    global: None,
                    local: None,
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
                    global_result(),
                );

                assert_eq!(
                    result.global,
                    Some(DiscoveredConfigFile {
                        base: PathBuf::from("/config/lithos"),
                        path: PathBuf::from("/config/lithos/lithos.toml"),
                        format: StructuredFileFormat::Toml,
                    })
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

            #[test]
            fn returns_global_file_without_source_identity() {
                let result = ConfigDiscoveryResult::from_discovery(
                    vault_result(),
                    GlobalDiscoveryResult {
                        marker: Some(DiscoveredMarker {
                            base: PathBuf::from("/config/lithos"),
                            path: PathBuf::from("/config/lithos/lithos.toml"),
                            format: StructuredFileFormat::Toml,
                        }),
                        alternatives: Vec::new(),
                        source: None,
                    },
                );

                assert_eq!(
                    result.global.map(|file| file.path),
                    Some(PathBuf::from("/config/lithos/lithos.toml"))
                );
            }
        }
    }
}
