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
    pub fn anchor(&self) -> &PathBuf {
        &self.anchor
    }

    /// Explicit config file supplied by the caller.
    #[must_use]
    #[inline]
    pub fn config_file(&self) -> Option<&PathBuf> {
        self.config_file.as_ref()
    }

    /// Explicit vault directory supplied by the caller.
    #[must_use]
    #[inline]
    pub fn vault_dir(&self) -> Option<&PathBuf> {
        self.vault_dir.as_ref()
    }

    /// Whether global config discovery should be skipped.
    #[must_use]
    #[inline]
    pub fn suppress_global(&self) -> bool {
        self.suppress_global
    }
}

/// Public discovery output for settings service callers.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiscoveryOutcome {
    /// Vault-local candidate config paths.
    vault: Box<[CandidatePath]>,
    /// Global candidate config paths.
    global: Box<[CandidatePath]>,
    /// Resolved cache root for settings data.
    cache_root: CacheRoot,
}

impl DiscoveryOutcome {
    /// Create new discovery outcome.
    #[must_use]
    #[inline]
    pub fn new(
        vault: Box<[CandidatePath]>,
        global: Box<[CandidatePath]>,
        cache_root: CacheRoot,
    ) -> Self {
        Self {
            vault,
            global,
            cache_root,
        }
    }

    /// Vault-local candidate config paths.
    #[must_use]
    #[inline]
    pub fn vault(&self) -> &[CandidatePath] {
        &self.vault
    }

    /// Global candidate config paths.
    #[must_use]
    #[inline]
    pub fn global(&self) -> &[CandidatePath] {
        &self.global
    }

    /// Resolved cache root for settings data.
    #[must_use]
    #[inline]
    pub fn cache_root(&self) -> &CacheRoot {
        &self.cache_root
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
