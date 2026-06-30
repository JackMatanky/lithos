//! Settings service facade.

use std::path::PathBuf;

use crate::{
    candidate::CandidatePath,
    config::{aggregate::AppConfig, error::ConfigError},
    discovery::{CacheRoot, error::DiscoveryError},
};

/// CLI/runtime inputs for settings discovery.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOptions {
    /// Directory or file path where discovery begins.
    pub anchor: PathBuf,
    /// Explicit config file supplied by the caller.
    pub config_file: Option<PathBuf>,
    /// Explicit vault directory supplied by the caller.
    pub vault_dir: Option<PathBuf>,
    /// Whether global config discovery should be skipped.
    pub suppress_global: bool,
}

/// Public discovery output for settings service callers.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOutcome {
    /// Vault-local candidate config paths.
    pub vault: Box<[CandidatePath]>,
    /// Global candidate config paths.
    pub global: Box<[CandidatePath]>,
    /// Resolved cache root for settings data.
    pub cache_root: CacheRoot,
}

/// Trust behavior for config building.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TrustMode {
    /// Verify config trust before using trusted inputs.
    Verify,
    /// Accept all config inputs without prompting.
    AcceptAll,
}

/// Inputs for building settings config after discovery.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfigBuilderOptions {
    /// Trust behavior used during config building.
    pub trust_mode: TrustMode,
    /// Whether prompts should be accepted automatically.
    pub auto_confirm: bool,
}

/// Errors returned by the settings service boundary.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// Discovery failed before config construction could begin.
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    /// App Config construction failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Service implementation has not been wired to the internal pipeline yet.
    #[error("settings service pipeline is not implemented yet")]
    PipelineNotImplemented,
}

/// Core service trait for the settings domain.
pub trait SettingsService {
    /// Discover settings candidate paths without building App Config.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] when discovery inputs are invalid or
    /// filesystem discovery fails.
    fn discover(
        &self,
        options: DiscoveryOptions,
    ) -> Result<DiscoveryOutcome, SettingsError>;

    /// Build App Config from discovered vault-local and global candidates.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] when trust, loading, validation, merge, or
    /// persistence fails while constructing App Config.
    fn build_config(
        &self,
        vault: &[CandidatePath],
        global: &[CandidatePath],
        options: ConfigBuilderOptions,
    ) -> Result<AppConfig, SettingsError>;
}

/// Primary implementation of the settings service.
pub struct Service;

impl SettingsService for Service {
    #[inline]
    fn discover(
        &self,
        _options: DiscoveryOptions,
    ) -> Result<DiscoveryOutcome, SettingsError> {
        Err(SettingsError::PipelineNotImplemented)
    }

    #[inline]
    fn build_config(
        &self,
        _vault: &[CandidatePath],
        _global: &[CandidatePath],
        _options: ConfigBuilderOptions,
    ) -> Result<AppConfig, SettingsError> {
        Err(SettingsError::PipelineNotImplemented)
    }
}
