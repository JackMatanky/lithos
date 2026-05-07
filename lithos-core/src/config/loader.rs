//! Configuration loading orchestration with hybrid staleness detection.
//!
//! This module provides the [`Loader`] struct which orchestrates the complete
//! configuration loading pipeline:
//!
//! 1. **Load raw views** from database (cached file state)
//! 2. **Ingest raw configs** from filesystem
//! 3. **Detect staleness** via timestamp + content hash comparison
//! 4. **Merge configs** using Figment when stale
//! 5. **Build domain** via `Config::build`
//! 6. **Persist** updated views and config to database
//!
//! The loader implements efficient staleness tracking by comparing:
//! - File modification timestamps (fast check)
//! - BLAKE3 content hashes (accurate change detection)
//!
//! This hybrid approach avoids unnecessary rebuilds while detecting all actual
//! file changes.

use std::path::PathBuf;

use tracing::instrument;

use crate::config::{
    aggregate::Config,
    error::ConfigError,
    ingestor::Ingestor,
    merger::ConfigMerger,
    processor::{self, ConfigFileProcessor, GlobalConfig, VaultConfig},
    storage::Repository,
    vault::{VaultId, VaultRoot},
};

/// Configuration loader with hybrid staleness detection.
///
/// Coordinates the full configuration loading pipeline:
/// - File discovery and ingestion
/// - Staleness detection (timestamps + content hash)
/// - Figment-based merging
/// - Domain validation
/// - Database persistence
///
/// # Architecture
///
/// The loader owns the orchestration pipeline but delegates to:
/// - `Ingestor`: File discovery and raw config parsing
/// - `Repository`: Database persistence and retrieval
/// - `Config::build`: Domain validation and construction
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
///
/// use lithos_core::config::{loader::Loader, storage::RedbStorage};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vault_root = PathBuf::from("/vault");
/// // let repo = RedbStorage::new(...);
/// // let loader = Loader::new(vault_root, repo);
/// // let config = loader.load()?;
/// # Ok(())
/// # }
/// ```
pub struct Loader<R> {
    /// Vault root path for config resolution.
    vault_root: PathBuf,
    /// Configuration ingestor for file operations.
    ingestor: Ingestor,
    /// Repository for database persistence.
    repository: R,
}

impl<R> Loader<R>
where
    R: Repository,
{
    /// Create a new loader for the given vault root.
    ///
    /// # Parameters
    ///
    /// - `vault_root`: Vault root directory for config resolution
    /// - `repository`: Database repository for persistence
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::{loader::Loader, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let loader = Loader::new(vault_root, repo);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(vault_root: P, repository: R) -> Self {
        let vault_root = vault_root.into();
        let ingestor = Ingestor::new(&vault_root);

        Self {
            vault_root,
            ingestor,
            repository,
        }
    }

    /// Load configuration with hybrid staleness detection.
    ///
    /// Pipeline:
    /// 1. Load raw views from DB (if exist)
    /// 2. Ingest raw configs from files
    /// 3. Detect staleness (timestamps + content hash)
    /// 4. If stale: merge → build → save
    /// 5. If fresh: load cached config from DB
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - File ingestion fails (I/O error, parse error)
    /// - Figment merge fails (incompatible layers)
    /// - Domain validation fails (invalid config)
    /// - Database operations fail
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::{loader::Loader, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let loader = Loader::new(vault_root, repo);
    /// // let config = loader.load()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(vault_root = %self.vault_root.display())
    )]
    pub fn load(&self) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Step 1: Validate and get vault root
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        // Step 2: Get or create vault ID
        let vault_id = self.get_or_create_vault_id()?;

        // Step 3: Ingest raw configs from files
        let global_raw = self.ingestor.global_config()?;
        let vault_raw = self.ingestor.vault_config(&vault_root)?;

        // Step 4: Get cached views from repository
        let global_view =
            self.repository.get_raw_global_view().map_err(Into::into)?;
        let vault_view =
            self.repository.get_raw_vault_view(vault_id).map_err(Into::into)?;

        // Step 5: Process global config
        let global_processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(
            global_raw,
            global_view,
        );
        let global_outcome = match global_processor.compare()? {
            processor::ComparisonBranch::Fresh(proc) => proc.finalize(),
            processor::ComparisonBranch::Stale(proc) => match proc.analyze()? {
                processor::AnalysisBranch::NoChanges(proc) => proc.finalize(),
                processor::AnalysisBranch::PropertyChanges(proc) => {
                    proc.finalize()
                }
            },
        };

        // Step 6: Process vault config
        let vault_processor = ConfigFileProcessor::<VaultConfig, _, _>::new(
            vault_raw, vault_view,
        );
        let vault_outcome = match vault_processor.compare()? {
            processor::ComparisonBranch::Fresh(proc) => proc.finalize(),
            processor::ComparisonBranch::Stale(proc) => match proc.analyze()? {
                processor::AnalysisBranch::NoChanges(proc) => proc.finalize(),
                processor::AnalysisBranch::PropertyChanges(proc) => {
                    proc.finalize()
                }
            },
        };

        // Step 7: Merge outcomes into final config
        let merger = ConfigMerger::new(vault_id, vault_root, &self.repository);
        merger.merge(global_outcome, vault_outcome)
    }

    /// Get or create vault ID for the vault root.
    ///
    /// Looks up existing vault ID from path mapping, or creates a new one.
    fn get_or_create_vault_id(&self) -> Result<VaultId, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        if let Some(existing_id) = self
            .repository
            .find_vault_id_by_path(&vault_root)
            .map_err(Into::into)?
        {
            Ok(existing_id)
        } else {
            let new_id = VaultId::new();
            self.repository
                .save_vault_path_mapping(new_id, &vault_root)
                .map_err(Into::into)?;
            Ok(new_id)
        }
    }
}
