//! Precedence and behavioral policies for configuration discovery.
//!
//! This module defines the [`DiscoveryPolicy`] which configures how the
//! [`crate::discovery::engine::DiscoveryEngine`] prioritizes different
//! configuration sources. It provides the ranking and taxonomy for vault and
//! global configuration origins.
//!
//! # Precedence Tiers
//!
//! Precedence is modeled as a list of [`VaultSourceType`] or
//! [`GlobalSourceType`] variants. The engine probes these sources in the
//! order they appear in the policy.
//!
//! # Defaults
//!
//! The [`Default`] implementation for `DiscoveryPolicy` provides the standard
//! Lithos precedence:
//! - **Vault**: Explicit Flag > Environment Variable > Ascending Walk.
//! - **Global**: Environment Variable > XDG Config > User Config > System
//!   Config.

/// Naming pattern used to identify a marker file family.
#[allow(
    dead_code,
    reason = "Contract slice; traversal still uses legacy probe"
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MarkerPattern {
    /// Filename prefix before a structured file extension is applied.
    pub(crate) prefix: &'static str,
    /// Whether the pattern is nested below a config directory.
    pub(crate) is_nested: bool,
}

/// Standard marker patterns used for vault root resolution.
#[allow(
    dead_code,
    reason = "Contract slice; traversal still uses legacy probe"
)]
pub(crate) const VAULT_MARKER_PATTERNS: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".lithos/config",
        is_nested: true,
    },
];

/// Standard marker patterns used for global config resolution.
#[allow(
    dead_code,
    reason = "Contract slice; traversal still uses legacy probe"
)]
pub(crate) const GLOBAL_MARKER_PATTERNS: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: "lithos/config",
        is_nested: true,
    },
];

/// Standard project boundary directory names that stop ascending traversal.
///
/// When the ascending walk encounters one of these directory names, it
/// treats that directory as a project boundary and stops before or at
/// the boundary (depending on `allow_marker_at_ceiling`).
#[allow(
    dead_code,
    reason = "Contract slice; wired in once orchestration lands"
)]
pub(crate) const BOUNDARY_MARKER_PATTERNS: &[&str] = &[".git", ".workspace"];

/// Defines the behavior and precedence rules for configuration discovery.
///
/// This policy controls which sources are checked, in what order, and how
/// boundary conditions (like discovery ceilings) are handled.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoveryPolicy {
    /// Ordered list of sources to check for vault roots.
    ///
    /// The engine iterates through this list and stops at the first source
    /// that yields a valid root.
    pub(crate) vault_precedence: Vec<VaultSourceType>,
    /// Ordered list of sources to check for global config roots.
    pub(crate) global_precedence: Vec<GlobalSourceType>,
    /// Whether a marker file located exactly at a ceiling directory is valid.
    ///
    /// If true, the walk stops *after* probing the ceiling. If false, it
    /// stops *before* probing.
    pub(crate) allow_marker_at_ceiling: bool,
    /// Whether discovery should fail immediately if an explicit path is
    /// invalid.
    pub(crate) strict_overrides: bool,
}

impl Default for DiscoveryPolicy {
    fn default() -> Self {
        Self {
            vault_precedence: VaultSourceType::PRECEDENCE.to_vec(),
            global_precedence: GlobalSourceType::PRECEDENCE.to_vec(),
            allow_marker_at_ceiling: true,
            strict_overrides: false,
        }
    }
}

/// Enumerates the possible origins for a vault configuration root.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum VaultSourceType {
    /// Provided via `--vault` CLI flag.
    ExplicitFlag,
    /// Provided via `LITHOS_VAULT` environment variable.
    EnvVar,
    /// Discovered by walking up parent directories from CWD.
    AscendingWalk,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl VaultSourceType {
    /// Stable precedence used by deterministic candidate selection.
    pub(crate) const PRECEDENCE: [Self; 3] =
        [Self::ExplicitFlag, Self::EnvVar, Self::AscendingWalk];

    /// Returns the precedence rank (lower is higher priority).
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::ExplicitFlag => 0,
            Self::EnvVar => 1,
            Self::AscendingWalk => 2,
        }
    }

    /// Returns a human-readable description of the source.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ExplicitFlag => "explicit CLI flag",
            Self::EnvVar => "environment variable",
            Self::AscendingWalk => "ascending directory walk",
        }
    }
}

/// Enumerates the possible origins for global system/user configuration.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GlobalSourceType {
    /// Provided via `LITHOS_CONFIG_FILE` environment variable.
    EnvVar,
    /// Discovered via a configured global source directory.
    Directory(GlobalSourceDirectory),
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl GlobalSourceType {
    /// Stable precedence used by deterministic global source resolution.
    pub(crate) const PRECEDENCE: [Self; 4] = [
        Self::EnvVar,
        Self::Directory(GlobalSourceDirectory::XdgConfig),
        Self::Directory(GlobalSourceDirectory::UserConfig),
        Self::Directory(GlobalSourceDirectory::SystemConfig),
    ];

    /// Returns the precedence rank (lower is higher priority).
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::EnvVar => 0,
            Self::Directory(GlobalSourceDirectory::XdgConfig) => 1,
            Self::Directory(GlobalSourceDirectory::UserConfig) => 2,
            Self::Directory(GlobalSourceDirectory::SystemConfig) => 3,
        }
    }
}

/// Enumerates global configuration directories probed after environment file
/// overrides have been considered.
#[allow(dead_code, reason = "Phase-2 seam; wired in once orchestration lands")]
#[expect(
    clippy::enum_variant_names,
    reason = "GlobalSourceDirectory variants mirror the approved source names"
)]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GlobalSourceDirectory {
    /// Discovered via XDG Base Directory specification.
    XdgConfig,
    /// Discovered in standard user home configuration path (~/.config/lithos).
    UserConfig,
    /// Discovered in system-wide configuration path (/etc/lithos).
    SystemConfig,
}

#[allow(dead_code, reason = "Phase-2 seam; wired in once orchestration lands")]
impl GlobalSourceDirectory {
    /// Returns the corresponding discovery source identity.
    pub(crate) fn source_type(self) -> GlobalSourceType {
        GlobalSourceType::Directory(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rank {
        use super::*;

        #[test]
        fn returns_explicit_flag_before_vault_env_var() {
            assert!(
                VaultSourceType::ExplicitFlag.rank()
                    < VaultSourceType::EnvVar.rank()
            );
        }

        #[test]
        fn returns_vault_env_var_before_ascending_walk() {
            assert!(
                VaultSourceType::EnvVar.rank()
                    < VaultSourceType::AscendingWalk.rank()
            );
        }

        #[test]
        fn returns_env_var_before_xdg_config() {
            assert!(
                GlobalSourceType::EnvVar.rank()
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::XdgConfig,
                    )
                    .rank()
            );
        }

        #[test]
        fn returns_xdg_config_before_user_config() {
            assert!(
                GlobalSourceType::Directory(GlobalSourceDirectory::XdgConfig)
                    .rank()
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::UserConfig,
                    )
                    .rank()
            );
        }

        #[test]
        fn returns_user_config_before_system_config() {
            assert!(
                GlobalSourceType::Directory(GlobalSourceDirectory::UserConfig)
                    .rank()
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::SystemConfig,
                    )
                    .rank()
            );
        }
    }

    mod description {
        use super::*;

        #[test]
        fn returns_explicit_flag_description() {
            assert_eq!(
                VaultSourceType::ExplicitFlag.description(),
                "explicit CLI flag"
            );
        }

        #[test]
        fn returns_env_var_description() {
            assert_eq!(
                VaultSourceType::EnvVar.description(),
                "environment variable"
            );
        }

        #[test]
        fn returns_ascending_walk_description() {
            assert_eq!(
                VaultSourceType::AscendingWalk.description(),
                "ascending directory walk"
            );
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn declares_vault_marker_pattern_contract_prefix() {
            assert_eq!(
                VAULT_MARKER_PATTERNS.first().expect("vault pattern").prefix,
                "lithos"
            );
        }

        #[test]
        fn declares_global_marker_pattern_contract_prefix() {
            assert_eq!(
                GLOBAL_MARKER_PATTERNS.first().expect("global pattern").prefix,
                "lithos"
            );
        }

        #[test]
        fn declares_boundary_marker_patterns() {
            let patterns: Vec<&str> = BOUNDARY_MARKER_PATTERNS.to_vec();
            assert!(
                !patterns.is_empty(),
                "boundary patterns must not be empty"
            );
            assert!(
                patterns.contains(&".git"),
                "expected .git boundary marker"
            );
        }

        #[test]
        fn returns_standard_vault_precedence() {
            let policy = DiscoveryPolicy::default();

            assert_eq!(
                policy.vault_precedence,
                VaultSourceType::PRECEDENCE.to_vec()
            );
        }

        #[test]
        fn returns_standard_global_precedence() {
            let policy = DiscoveryPolicy::default();

            assert_eq!(
                policy.global_precedence,
                GlobalSourceType::PRECEDENCE.to_vec()
            );
        }

        #[test]
        fn allows_markers_at_ceiling() {
            let policy = DiscoveryPolicy::default();

            assert!(
                policy.allow_marker_at_ceiling,
                "default policy should allow marker files at ceiling \
                 directories"
            );
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn orders_explicit_flag_before_vault_env_var() {
            assert!(VaultSourceType::ExplicitFlag < VaultSourceType::EnvVar);
        }

        #[test]
        fn orders_vault_env_var_before_ascending_walk() {
            assert!(VaultSourceType::EnvVar < VaultSourceType::AscendingWalk);
        }

        #[test]
        fn orders_env_var_before_xdg_config() {
            assert!(
                GlobalSourceType::EnvVar
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::XdgConfig,
                    )
            );
        }

        #[test]
        fn orders_xdg_config_before_user_config() {
            assert!(
                GlobalSourceType::Directory(GlobalSourceDirectory::XdgConfig)
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::UserConfig,
                    )
            );
        }

        #[test]
        fn orders_user_config_before_system_config() {
            assert!(
                GlobalSourceType::Directory(GlobalSourceDirectory::UserConfig)
                    < GlobalSourceType::Directory(
                        GlobalSourceDirectory::SystemConfig,
                    )
            );
        }
    }
}
