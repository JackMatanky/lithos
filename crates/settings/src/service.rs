//! Settings service facade.

use std::path::{Path, PathBuf};

use crate::{
    candidate::CandidatePath,
    config::{aggregate::AppConfig, error::ConfigError},
    discovery::{
        error::DiscoveryError, input::DiscoveryInput,
        outcome::DiscoveryOutcome, processor::DiscoveryProcessor,
    },
    env_var::SettingsEnvVars,
};

/// CLI/runtime inputs for settings discovery.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOptions {
    /// Directory or file path where discovery begins.
    anchor: PathBuf,
    /// Explicit config file supplied by the caller.
    config_file: Option<PathBuf>,
    /// Explicit vault directory supplied by the caller.
    vault_dir: Option<PathBuf>,
    /// Whether global config discovery should be skipped.
    suppress_global: bool,
}

impl DiscoveryOptions {
    /// Create new discovery options.
    #[must_use]
    #[inline]
    pub fn new(
        anchor: PathBuf,
        config_file: Option<PathBuf>,
        vault_dir: Option<PathBuf>,
        suppress_global: bool,
    ) -> Self {
        Self {
            anchor,
            config_file,
            vault_dir,
            suppress_global,
        }
    }

    /// Directory or file path where discovery begins.
    #[must_use]
    #[inline]
    pub fn anchor(&self) -> &Path {
        &self.anchor
    }

    /// Explicit config file supplied by the caller.
    #[must_use]
    #[inline]
    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    /// Explicit vault directory supplied by the caller.
    #[must_use]
    #[inline]
    pub fn vault_dir(&self) -> Option<&Path> {
        self.vault_dir.as_deref()
    }

    /// Whether global config discovery should be skipped.
    #[must_use]
    #[inline]
    pub fn suppress_global(&self) -> bool {
        self.suppress_global
    }
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
    trust_mode: TrustMode,
    /// Whether prompts should be accepted automatically.
    auto_confirm: bool,
}

impl ConfigBuilderOptions {
    /// Create new config builder options.
    #[must_use]
    #[inline]
    pub fn new(trust_mode: TrustMode, auto_confirm: bool) -> Self {
        Self {
            trust_mode,
            auto_confirm,
        }
    }

    /// Trust behavior used during config building.
    #[must_use]
    #[inline]
    pub fn trust_mode(&self) -> TrustMode {
        self.trust_mode
    }

    /// Whether prompts should be accepted automatically.
    #[must_use]
    #[inline]
    pub fn auto_confirm(&self) -> bool {
        self.auto_confirm
    }
}

/// Errors returned by the settings service boundary.
#[must_use]
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
        options: DiscoveryOptions,
    ) -> Result<DiscoveryOutcome, SettingsError> {
        let env = SettingsEnvVars::capture();
        let input = DiscoveryInput::from_options(&options, &env)?;
        Ok(DiscoveryProcessor::new(input)
            .collect_local()?
            .collect_global()
            .finish())
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
