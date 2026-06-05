//! Precedence and behavioral policies for configuration discovery.

/// Configuration defining the behavior and precedence rules for the discovery
/// engine.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveryPolicy {
    /// Ordered list of sources to check for vault roots.
    pub(crate) precedence: Vec<VaultSourceType>,
    /// Whether a marker file located exactly at a ceiling directory is valid.
    pub(crate) allow_marker_at_ceiling: bool,
    /// Whether discovery should fail immediately if an explicit path is
    /// invalid.
    pub(crate) strict_overrides: bool,
}

impl Default for DiscoveryPolicy {
    fn default() -> Self {
        Self {
            precedence: VaultSourceType::PRECEDENCE.to_vec(),
            allow_marker_at_ceiling: true,
            strict_overrides: false,
        }
    }
}

/// Enumerates the possible origins for a vault configuration root.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GlobalSourceType {
    /// Provided via `LITHOS_CONFIG_FILE` environment variable.
    EnvVar,
    /// Discovered via XDG Base Directory specification.
    XdgConfig,
    /// Discovered in standard user home configuration path (~/.lithos).
    UserConfig,
    /// Discovered in system-wide configuration path (/etc/lithos).
    SystemConfig,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl GlobalSourceType {
    /// Stable precedence used by deterministic candidate selection.
    pub(crate) const PRECEDENCE: [Self; 4] =
        [Self::EnvVar, Self::XdgConfig, Self::UserConfig, Self::SystemConfig];

    /// Returns the precedence rank (lower is higher priority).
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::EnvVar => 0,
            Self::XdgConfig => 1,
            Self::UserConfig => 2,
            Self::SystemConfig => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_source_types_have_correct_rank_order() {
        assert!(
            VaultSourceType::ExplicitFlag.rank()
                < VaultSourceType::EnvVar.rank()
        );
        assert!(
            VaultSourceType::EnvVar.rank()
                < VaultSourceType::AscendingWalk.rank()
        );
    }

    #[test]
    fn global_source_types_have_correct_rank_order() {
        assert!(
            GlobalSourceType::EnvVar.rank()
                < GlobalSourceType::XdgConfig.rank()
        );
        assert!(
            GlobalSourceType::XdgConfig.rank()
                < GlobalSourceType::UserConfig.rank()
        );
        assert!(
            GlobalSourceType::UserConfig.rank()
                < GlobalSourceType::SystemConfig.rank()
        );
    }

    #[test]
    fn vault_source_type_description_is_human_readable() {
        assert_eq!(
            VaultSourceType::ExplicitFlag.description(),
            "explicit CLI flag"
        );
        assert_eq!(
            VaultSourceType::EnvVar.description(),
            "environment variable"
        );
        assert_eq!(
            VaultSourceType::AscendingWalk.description(),
            "ascending directory walk"
        );
    }

    #[test]
    fn default_policy_uses_standard_precedence() {
        let policy = DiscoveryPolicy::default();
        assert_eq!(policy.precedence.len(), 3);
        assert_eq!(
            policy.precedence.first(),
            Some(&VaultSourceType::ExplicitFlag)
        );
        assert_eq!(policy.precedence.get(1), Some(&VaultSourceType::EnvVar));
        assert_eq!(
            policy.precedence.get(2),
            Some(&VaultSourceType::AscendingWalk)
        );
        assert!(policy.allow_marker_at_ceiling);
    }

    #[test]
    fn vault_source_types_derive_ord() {
        assert!(VaultSourceType::ExplicitFlag < VaultSourceType::EnvVar);
        assert!(VaultSourceType::EnvVar < VaultSourceType::AscendingWalk);
    }

    #[test]
    fn global_source_types_derive_ord() {
        assert!(GlobalSourceType::EnvVar < GlobalSourceType::XdgConfig);
        assert!(GlobalSourceType::XdgConfig < GlobalSourceType::UserConfig);
        assert!(GlobalSourceType::UserConfig < GlobalSourceType::SystemConfig);
    }
}
