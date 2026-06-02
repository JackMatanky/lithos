//! Discovery location taxonomy for config resolution.
//!
//! This module defines location-oriented contracts only. It does not perform
//! filesystem traversal or candidate selection.

use std::path::{Path, PathBuf};

use crate::fs::format::StructuredFileFormat;

/// Global configuration discovery locations.
///
/// Discovery order: `ExplicitOverride` > `EnvironmentOverride` > `XdgConfig`
/// > `UserConfig` > `SystemConfig`.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GlobalConfigLocation {
    /// Explicit path from `--config`.
    ExplicitOverride(PathBuf),
    /// Path from `$LITHOS_CONFIG_FILE`.
    EnvironmentOverride(PathBuf),
    /// `$XDG_CONFIG_HOME/lithos/lithos.{toml,json,yaml,yml}`
    XdgConfig,
    /// `~/.config/lithos/lithos.{toml,json,yaml,yml}`
    UserConfig,
    /// `/etc/lithos/lithos.{toml,json,yaml,yml}`
    SystemConfig,
}

/// Local configuration discovery locations.
///
/// Supported file patterns per location:
/// - `RootConfigFile`: `<root>/lithos.{toml,json,yaml,yml}`
/// - `HiddenRootConfigFile`: `<root>/.lithos.{toml,json,yaml,yml}`
/// - `ConfigDirectoryFile`: `<root>/.lithos/config.{toml,json,yaml,yml}`
#[allow(
    clippy::enum_variant_names,
    reason = "Variant names are fixed by the approved PRD contract"
)]
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LocalConfigLocation {
    RootConfigFile,
    HiddenRootConfigFile,
    ConfigDirectoryFile,
}

impl LocalConfigLocation {
    /// Locations recognized as valid vault root markers.
    pub(crate) const MARKERS: &'static [Self] = &[
        Self::RootConfigFile,
        Self::HiddenRootConfigFile,
        Self::ConfigDirectoryFile,
    ];

    /// Generates a concrete candidate path for a location/format pair.
    #[allow(
        dead_code,
        reason = "Phase-1 contracts are defined before full pipeline \
                  integration"
    )]
    pub(crate) fn candidate_path(
        self,
        root: &Path,
        format: StructuredFileFormat,
    ) -> PathBuf {
        let ext = format.extension();
        match self {
            Self::RootConfigFile => root.join(format!("lithos.{ext}")),
            Self::HiddenRootConfigFile => root.join(format!(".lithos.{ext}")),
            Self::ConfigDirectoryFile => {
                root.join(".lithos").join(format!("config.{ext}"))
            }
        }
    }
}

/// Unified config location taxonomy.
///
/// This wrapper preserves source class information so downstream resolution can
/// apply precedence rules (`Global` before `Local`) without string matching.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConfigLocation {
    Global(GlobalConfigLocation),
    Local(LocalConfigLocation),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod candidate_path {
        use super::*;

        #[test]
        fn returns_root_config_file_path() {
            let got = LocalConfigLocation::RootConfigFile.candidate_path(
                Path::new("/vault"),
                StructuredFileFormat::Toml,
            );

            assert_eq!(got, PathBuf::from("/vault/lithos.toml"));
        }

        #[test]
        fn returns_hidden_root_config_file_path() {
            let got = LocalConfigLocation::HiddenRootConfigFile.candidate_path(
                Path::new("/vault"),
                StructuredFileFormat::Json,
            );

            assert_eq!(got, PathBuf::from("/vault/.lithos.json"));
        }

        #[test]
        fn returns_config_directory_file_path() {
            let got = LocalConfigLocation::ConfigDirectoryFile.candidate_path(
                Path::new("/vault"),
                StructuredFileFormat::Yaml,
            );

            assert_eq!(got, PathBuf::from("/vault/.lithos/config.yaml"));
        }

        #[test]
        fn returns_yml_path_for_hidden_root_config_file() {
            let got = LocalConfigLocation::HiddenRootConfigFile
                .candidate_path(Path::new("/vault"), StructuredFileFormat::Yml);

            assert_eq!(got, PathBuf::from("/vault/.lithos.yml"));
        }
    }

    mod markers {
        use super::*;

        #[test]
        fn returns_all_supported_root_marker_locations() {
            assert_eq!(LocalConfigLocation::MARKERS.len(), 3);
        }

        #[test]
        fn returns_markers_in_discovery_order() {
            assert_eq!(LocalConfigLocation::MARKERS, &[
                LocalConfigLocation::RootConfigFile,
                LocalConfigLocation::HiddenRootConfigFile,
                LocalConfigLocation::ConfigDirectoryFile,
            ]);
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn returns_true_when_explicit_override_locations_match() {
            let first = GlobalConfigLocation::ExplicitOverride(PathBuf::from(
                "/tmp/lithos.toml",
            ));
            let second = GlobalConfigLocation::ExplicitOverride(PathBuf::from(
                "/tmp/lithos.toml",
            ));

            assert_eq!(first, second);
        }

        #[test]
        fn returns_true_when_environment_override_locations_match() {
            let first = GlobalConfigLocation::EnvironmentOverride(
                PathBuf::from("/env/lithos.toml"),
            );
            let second = GlobalConfigLocation::EnvironmentOverride(
                PathBuf::from("/env/lithos.toml"),
            );

            assert_eq!(first, second);
        }

        #[test]
        fn returns_local_variant_when_wrapping_local_location() {
            let wrapped =
                ConfigLocation::Local(LocalConfigLocation::ConfigDirectoryFile);

            assert!(matches!(wrapped, ConfigLocation::Local(_)));
        }
    }
}
