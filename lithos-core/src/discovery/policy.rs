#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum VaultSourceType {
    ExplicitFlag,
    EnvVar,
    AscendingWalk,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl VaultSourceType {
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::ExplicitFlag => 0,
            Self::EnvVar => 1,
            Self::AscendingWalk => 2,
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ExplicitFlag => "explicit CLI flag",
            Self::EnvVar => "environment variable",
            Self::AscendingWalk => "ascending directory walk",
        }
    }
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GlobalSourceType {
    EnvVar,
    XdgConfig,
    UserConfig,
    SystemConfig,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl GlobalSourceType {
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::EnvVar => 0,
            Self::XdgConfig => 1,
            Self::UserConfig => 2,
            Self::SystemConfig => 3,
        }
    }
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveryPolicy {
    pub(crate) precedence: Vec<VaultSourceType>,
    pub(crate) allow_marker_at_ceiling: bool,
    pub(crate) strict_overrides: bool,
}

impl Default for DiscoveryPolicy {
    fn default() -> Self {
        Self {
            precedence: vec![
                VaultSourceType::ExplicitFlag,
                VaultSourceType::EnvVar,
                VaultSourceType::AscendingWalk,
            ],
            allow_marker_at_ceiling: true,
            strict_overrides: false,
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
