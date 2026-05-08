//! Config resolution for processor outcomes.
//!
//! This module combines outcomes from parallel config file processors
//! (environment + local) and resolves them into an executable plan.

use crate::config::{
    error::ConfigError,
    processor::{GlobalConfig, ProcessorOutcome, VaultConfig},
    raw::{RawGlobalConfig, RawVaultConfig},
};

/// Action plan emitted by [`ConfigResolver`].
#[non_exhaustive]
pub enum ResolutionPlan {
    /// Both inputs are fresh; load cached config.
    UseCached,
    /// Raw content is unchanged but views must be updated.
    UpdateViews {
        /// Environment config raw layer when view sync is required.
        global: Option<RawGlobalConfig>,
        /// Local config raw layer when view sync is required.
        vault: Option<RawVaultConfig>,
    },
    /// At least one source changed semantically; rebuild final config.
    Rebuild {
        /// Environment config raw layer for rebuild.
        global: Option<RawGlobalConfig>,
        /// Local config raw layer for rebuild.
        vault: Option<RawVaultConfig>,
    },
}

/// Config resolver for combining processor outcomes.
#[derive(Default)]
#[non_exhaustive]
pub struct ConfigResolver;

impl ConfigResolver {
    /// Create a new resolver.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolve processor outcomes into an executable plan.
    ///
    /// Handles all 9 combinations of (`UseCached` | `UpdateViewOnly` |
    /// `Rebuild`) × 2.
    ///
    /// # Errors
    ///
    /// Currently no error branches are emitted, but this keeps the API
    /// extensible for future validation.
    #[inline]
    pub fn resolve(
        &self,
        global_outcome: ProcessorOutcome<GlobalConfig>,
        vault_outcome: ProcessorOutcome<VaultConfig>,
    ) -> Result<ResolutionPlan, ConfigError> {
        use ProcessorOutcome::{Rebuild, UpdateViewOnly, UseCached};

        let plan = match (global_outcome, vault_outcome) {
            (UseCached, UseCached) => ResolutionPlan::UseCached,
            (
                UpdateViewOnly {
                    raw: global,
                },
                UpdateViewOnly {
                    raw: vault,
                },
            ) => ResolutionPlan::UpdateViews {
                global: Some(global),
                vault: Some(vault),
            },
            (
                UpdateViewOnly {
                    raw: global,
                },
                UseCached,
            ) => ResolutionPlan::UpdateViews {
                global: Some(global),
                vault: None,
            },
            (
                UseCached,
                UpdateViewOnly {
                    raw: vault,
                },
            ) => ResolutionPlan::UpdateViews {
                global: None,
                vault: Some(vault),
            },
            (
                UseCached,
                Rebuild {
                    raw: vault,
                    ..
                },
            ) => ResolutionPlan::Rebuild {
                global: None,
                vault: Some(vault),
            },
            (
                UpdateViewOnly {
                    raw: global,
                },
                Rebuild {
                    raw: vault,
                    ..
                },
            )
            | (
                Rebuild {
                    raw: global,
                    ..
                },
                Rebuild {
                    raw: vault,
                    ..
                },
            )
            | (
                Rebuild {
                    raw: global,
                    ..
                },
                UpdateViewOnly {
                    raw: vault,
                },
            ) => ResolutionPlan::Rebuild {
                global: Some(global),
                vault: Some(vault),
            },
            (
                Rebuild {
                    raw: global,
                    ..
                },
                UseCached,
            ) => ResolutionPlan::Rebuild {
                global: Some(global),
                vault: None,
            },
        };

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::config::{
        processor::ProcessorOutcome,
        raw::{RawGlobalConfig, RawGlobalPaths, RawVaultConfig, RawVaultPaths},
    };

    fn create_test_global_config() -> RawGlobalConfig {
        RawGlobalConfig {
            logging: None,
            paths: RawGlobalPaths {
                templates_dir: Some("global-templates".into()),
                schemas_dir: Some("global-schemas".into()),
                property_bank_file: None,
            },
            trusted_vaults: None,
            frontmatter: None,
            task: None,
            metadata: None,
        }
    }

    fn create_test_vault_config() -> RawVaultConfig {
        RawVaultConfig {
            vault_path: "/tmp/test-vault".into(),
            name: None,
            version: None,
            logging: None,
            paths: RawVaultPaths {
                templates_dir: Some("vault-templates".into()),
                schemas_dir: None,
                cache_dir: Some(".cache".into()),
                property_bank_file: None,
            },
            frontmatter: None,
            task: None,
            metadata: None,
        }
    }

    #[test]
    fn resolve_both_use_cached_returns_use_cached() {
        let resolver = ConfigResolver::new();
        let result = resolver
            .resolve(ProcessorOutcome::UseCached, ProcessorOutcome::UseCached);
        assert!(matches!(result.unwrap(), ResolutionPlan::UseCached));
    }

    #[test]
    fn resolve_both_rebuild_returns_rebuild_with_both_layers() {
        let resolver = ConfigResolver::new();
        let expected_global = create_test_global_config();
        let expected_vault = create_test_vault_config();

        let result = resolver.resolve(
            ProcessorOutcome::Rebuild {
                raw: expected_global,
                changed_fields: HashSet::new(),
            },
            ProcessorOutcome::Rebuild {
                raw: expected_vault,
                changed_fields: HashSet::new(),
            },
        );

        assert!(matches!(result.unwrap(), ResolutionPlan::Rebuild {
            global: Some(_),
            vault: Some(_),
        }));
    }
}
