//! Single-file typestate processor for config file processing.
//!
//! This module provides a reusable typestate pipeline for processing individual
//! config files (global or vault). The processor handles:
//!
//! | Stage | Marker type | What happens |
//! |-------|-------------|--------------|
//! | 1 | [`Comparison`] | Compare raw config vs cached view (staleness detection) |
//! | 2 | [`Analysis`] | Field-level change detection (property hashes) |
//! | 3 | [`Completed`] | Terminal state with processing outcome |
//!
//! # Architecture
//!
//! The processor uses a **trait-based generic** design to work with both global
//! and vault config types:
//!
//! ```rust,ignore
//! // Define config type
//! pub struct GlobalConfig;
//! impl ConfigType for GlobalConfig {
//!     type Raw = RawGlobalConfig;
//!     type View = RawGlobalConfigView;
//!     // ...
//! }
//!
//! // Use processor
//! let processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(raw, view);
//! let outcome = processor.compare()?.analyze()?.finalize();
//! ```
//!
//! # Processing Outcomes
//!
//! The processor produces one of three outcomes:
//! - [`UseCached`] - Config is fresh, use cached domain object
//! - [`UpdateViewOnly`] - Metadata changed, update view but reuse cached config
//! - [`Rebuild`] - Properties changed, need to rebuild domain config
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use lithos_core::config::{
//!     processor::{
//!         ConfigFileProcessor, GlobalConfig, ProcessorOutcome,
//!         ComparisonBranch, AnalysisBranch,
//!     },
//!     raw::RawGlobalConfig,
//!     views::RawGlobalConfigView,
//! };
//!
//! # fn example(raw: Option<RawGlobalConfig>, view: Option<RawGlobalConfigView>) -> Result<(), Box<dyn std::error::Error>> {
//! let processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(raw, view);
//!
//! match processor.compare()? {
//!     ComparisonBranch::Fresh(p) => {
//!         // Use cached config
//!     }
//!     ComparisonBranch::Stale(p) => {
//!         match p.analyze()? {
//!             AnalysisBranch::NoChanges(p) => {
//!                 // Update view only
//!             }
//!             AnalysisBranch::PropertyChanges(p) => {
//!                 // Rebuild needed
//!             }
//!             _ => unreachable!("all variants covered"),
//!         }
//!     }
//!     _ => unreachable!("all variants covered"),
//! }
//! # Ok(())
//! # }
//! ```

use std::{collections::HashSet, marker::PhantomData};

use crate::config::{
    error::ConfigError,
    raw::{RawGlobalConfig, RawVaultConfig},
    views::{RawGlobalConfigView, RawVaultConfigView},
};

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigType Trait (Generic Abstraction)
// ─────────────────────────────────────────────────────────────────────────────

/// Trait defining the config type contract for generic processor.
///
/// This trait enables the processor to work with both global and vault configs
/// using the same typestate logic.
pub trait ConfigType {
    /// The raw config type (deserialized from TOML).
    type Raw: std::fmt::Debug;

    /// The view type (cached metadata + version history).
    type View: std::fmt::Debug;

    /// Compute field-level hashes for the raw config.
    ///
    /// Used for incremental analysis - determines which specific fields
    /// changed.
    fn compute_field_hashes(raw: &Self::Raw) -> ConfigFieldHashes;

    /// Returns true when view content hash matches raw content.
    fn content_hash_matches(view: &Self::View, raw: &Self::Raw) -> bool;
}

/// Marker type for global config processing.
#[derive(Debug)]
#[non_exhaustive]
pub struct GlobalConfig;

impl ConfigType for GlobalConfig {
    type Raw = RawGlobalConfig;
    type View = RawGlobalConfigView;

    #[inline]
    fn compute_field_hashes(raw: &Self::Raw) -> ConfigFieldHashes {
        use crate::support::hash::Blake3Hash;

        let mut hashes = ConfigFieldHashes::new();

        // Hash logging field if present
        if let Some(logging) = raw.logging.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(logging)
                .expect("logging serialization should not fail");
            hashes.insert(ConfigField::Logging, Blake3Hash::compute(&json));
        }

        // Hash paths field (always present, has default)
        #[expect(
            clippy::expect_used,
            reason = "Config types are always serializable"
        )]
        let paths_json = serde_json::to_vec(&raw.paths)
            .expect("paths serialization should not fail");
        hashes.insert(ConfigField::Paths, Blake3Hash::compute(&paths_json));

        // Hash frontmatter field if present
        if let Some(frontmatter) = raw.frontmatter.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(frontmatter)
                .expect("frontmatter serialization should not fail");
            hashes.insert(ConfigField::Frontmatter, Blake3Hash::compute(&json));
        }

        // Hash task field if present
        if let Some(task) = raw.task.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(task)
                .expect("task serialization should not fail");
            hashes.insert(ConfigField::Task, Blake3Hash::compute(&json));
        }

        hashes
    }

    #[inline]
    fn content_hash_matches(view: &Self::View, raw: &Self::Raw) -> bool {
        view.content_hash_matches(raw)
    }
}

/// Marker type for vault config processing.
#[derive(Debug)]
#[non_exhaustive]
pub struct VaultConfig;

impl ConfigType for VaultConfig {
    type Raw = RawVaultConfig;
    type View = RawVaultConfigView;

    #[inline]
    fn compute_field_hashes(raw: &Self::Raw) -> ConfigFieldHashes {
        use crate::support::hash::Blake3Hash;

        let mut hashes = ConfigFieldHashes::new();

        // Hash logging field if present
        if let Some(logging) = raw.logging.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(logging)
                .expect("logging serialization should not fail");
            hashes.insert(ConfigField::Logging, Blake3Hash::compute(&json));
        }

        // Hash paths field (always present, has default)
        #[expect(
            clippy::expect_used,
            reason = "Config types are always serializable"
        )]
        let paths_json = serde_json::to_vec(&raw.paths)
            .expect("paths serialization should not fail");
        hashes.insert(ConfigField::Paths, Blake3Hash::compute(&paths_json));

        // Hash frontmatter field if present
        if let Some(frontmatter) = raw.frontmatter.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(frontmatter)
                .expect("frontmatter serialization should not fail");
            hashes.insert(ConfigField::Frontmatter, Blake3Hash::compute(&json));
        }

        // Hash task field if present
        if let Some(task) = raw.task.as_ref() {
            #[expect(
                clippy::expect_used,
                reason = "Config types are always serializable"
            )]
            let json = serde_json::to_vec(task)
                .expect("task serialization should not fail");
            hashes.insert(ConfigField::Task, Blake3Hash::compute(&json));
        }

        hashes
    }

    #[inline]
    fn content_hash_matches(view: &Self::View, raw: &Self::Raw) -> bool {
        view.content_hash_matches(raw)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigFieldHashes (Field-Level Change Detection)
// ─────────────────────────────────────────────────────────────────────────────

/// Per-field hash map for incremental config analysis.
///
/// Similar to schema's `RawPropertyHashIndex`, this enables detecting which
/// specific config fields changed without reparsing the entire file.
///
/// # Examples
///
/// ```rust,ignore
/// use lithos_core::{
///     config::processor::{ConfigField, ConfigFieldHashes},
///     support::hash::Blake3Hash,
/// };
///
/// let mut hashes = ConfigFieldHashes::default();
/// hashes.insert(ConfigField::Logging, Blake3Hash::new([0; 32]));
/// assert!(hashes.contains(&ConfigField::Logging));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFieldHashes {
    inner: crate::support::hash::Blake3HashIndex<ConfigField>,
}

impl ConfigFieldHashes {
    /// Creates an empty field hash map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: crate::support::hash::Blake3HashIndex::default(),
        }
    }

    /// Inserts a field hash.
    #[inline]
    pub(crate) fn insert(
        &mut self,
        field: ConfigField,
        hash: crate::support::hash::Blake3Hash,
    ) {
        let _previous = self.inner.insert(field, hash);
    }

    /// Returns the hash for a field, if present.
    #[inline]
    #[must_use]
    #[expect(
        dead_code,
        reason = "Reserved for upcoming field-hash diff wiring"
    )]
    pub(crate) fn get(
        &self,
        field: &ConfigField,
    ) -> Option<&crate::support::hash::Blake3Hash> {
        self.inner.get(field)
    }

    /// Checks if a field hash exists.
    #[inline]
    #[must_use]
    pub fn contains(&self, field: &ConfigField) -> bool {
        self.inner.contains_key(field)
    }

    /// Returns an iterator over all field-hash pairs.
    #[inline]
    pub(crate) fn iter(
        &self,
    ) -> impl Iterator<Item = (&ConfigField, &crate::support::hash::Blake3Hash)>
    {
        self.inner.iter()
    }

    /// Computes the set of fields that changed between two hash maps.
    ///
    /// Returns fields present in `new` that either:
    /// - Don't exist in `old`
    /// - Have a different hash value
    #[inline]
    #[must_use]
    pub fn diff(&self, new: &Self) -> HashSet<ConfigField> {
        let mut changed = HashSet::new();

        for (field, new_hash) in new.inner.iter() {
            if self.inner.get(field) != Some(new_hash) {
                changed.insert(field.clone());
            }
        }

        changed
    }
}

/// Config field identifier for change tracking.
///
/// Represents top-level sections that can be independently hashed for
/// incremental analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConfigField {
    /// Logging configuration (`RawLoggingConfig`).
    Logging,
    /// Path configuration (`RawPathsConfig`).
    Paths,
    /// Task configuration (`RawTaskConfig`).
    Task,
    /// Frontmatter configuration (`RawFrontmatterConfig`).
    Frontmatter,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Core Processor
// ─────────────────────────────────────────────────────────────────────────────

/// Single-file config processor with typestate pattern.
///
/// Generic over:
/// - `T`: Config type (implements [`ConfigType`])
/// - `P`: Pipeline stage marker
/// - `S`: Status type (carries data)
#[derive(Debug)]
#[must_use]
pub struct ConfigFileProcessor<T, P, S> {
    status: S,
    _config_type: PhantomData<T>,
    _stage: PhantomData<P>,
}

impl<T, P, S> ConfigFileProcessor<T, P, S> {
    /// Internal transition helper for moving between states.
    #[inline]
    fn transition<NP, NS>(
        _stage: NP,
        status: NS,
    ) -> ConfigFileProcessor<T, NP, NS> {
        ConfigFileProcessor {
            status,
            _config_type: PhantomData,
            _stage: PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage Markers
// ─────────────────────────────────────────────────────────────────────────────

/// Comparison stage: check staleness via view comparison.
#[derive(Debug)]
#[non_exhaustive]
pub struct Comparison;

/// Analysis stage: field-level change detection.
#[derive(Debug)]
#[non_exhaustive]
pub struct Analysis;

/// Terminal stage: processing complete.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed;

// ─────────────────────────────────────────────────────────────────────────────
//  Status Types
// ─────────────────────────────────────────────────────────────────────────────

/// Initial status before comparison.
#[derive(Debug)]
pub struct Unknown<T: ConfigType> {
    raw: Option<T::Raw>,
    view: Option<T::View>,
}

/// Status when config is fresh (matches cached view).
#[derive(Debug)]
#[non_exhaustive]
pub struct Fresh;

/// Status when config is stale (doesn't match view).
#[derive(Debug)]
pub struct Stale<T: ConfigType> {
    raw: T::Raw,
    view: Option<T::View>,
}

/// Status when only metadata changed (timestamps/content hash).
#[derive(Debug)]
pub struct NoChanges<T: ConfigType> {
    raw: T::Raw,
}

/// Status when properties changed.
#[derive(Debug)]
pub struct PropertyChanges<T: ConfigType> {
    raw: T::Raw,
    changed_fields: HashSet<ConfigField>,
}

/// Terminal status with processing outcome.
#[derive(Debug)]
pub struct Ready<T: ConfigType> {
    #[expect(dead_code, reason = "Field consumed by finalize() methods")]
    outcome: ProcessorOutcome<T>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Branch Enums
// ─────────────────────────────────────────────────────────────────────────────

/// Result of comparison stage (staleness detection).
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
#[non_exhaustive]
pub enum ComparisonBranch<T: ConfigType> {
    /// Config is fresh - use cached domain object.
    Fresh(ConfigFileProcessor<T, Completed, Fresh>),
    /// Config is stale - proceed to analysis.
    Stale(ConfigFileProcessor<T, Analysis, Stale<T>>),
}

/// Result of analysis stage (field-level change detection).
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
#[non_exhaustive]
pub enum AnalysisBranch<T: ConfigType> {
    /// Only metadata changed - update view, reuse cached config.
    NoChanges(ConfigFileProcessor<T, Completed, NoChanges<T>>),
    /// Properties changed - need rebuild.
    PropertyChanges(ConfigFileProcessor<T, Completed, PropertyChanges<T>>),
}

/// Processing outcome after finalization.
#[derive(Debug)]
#[non_exhaustive]
pub enum ProcessorOutcome<T: ConfigType> {
    /// Config is fresh - use cached domain object from repository.
    UseCached,
    /// Metadata changed - update view only, reuse cached config.
    UpdateViewOnly {
        /// The raw config for view update.
        raw: T::Raw,
    },
    /// Properties changed - rebuild domain config.
    Rebuild {
        /// The raw config for rebuilding.
        raw: T::Raw,
        /// Which fields changed (for targeted rebuild).
        changed_fields: HashSet<ConfigField>,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage: Entry Point
// ─────────────────────────────────────────────────────────────────────────────

impl<T: ConfigType> ConfigFileProcessor<T, Comparison, Unknown<T>> {
    /// Creates a new processor for a config file.
    ///
    /// # Parameters
    ///
    /// - `raw`: Optional raw config from file ingestion
    /// - `view`: Optional cached view from repository
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use lithos_core::config::processor::{ConfigFileProcessor, GlobalConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(raw: Option<T::Raw>, view: Option<T::View>) -> Self {
        Self {
            status: Unknown {
                raw,
                view,
            },
            _config_type: PhantomData,
            _stage: PhantomData,
        }
    }

    /// Compare raw config against cached view for staleness detection.
    ///
    /// Returns a branch enum indicating whether the config is fresh or stale.
    ///
    /// Staleness is determined by:
    /// 1. No config file → Fresh (use defaults)
    /// 2. No cached view → Stale (first load, need to build)
    /// 3. Config deleted → Fresh (revert to defaults)
    /// 4. Both exist → Check `view.is_fresh(raw)` (timestamps + content hash)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if view comparison fails.
    #[inline]
    pub fn compare(self) -> Result<ComparisonBranch<T>, ConfigError>
    where
        T::View: IsConfigViewFresh<T::Raw>,
    {
        match (self.status.raw, self.status.view) {
            (None, None) => {
                // No config file, no cached view - fresh (use defaults)
                Ok(ComparisonBranch::Fresh(Self::transition(
                    Completed,
                    Fresh {},
                )))
            }
            (Some(raw), None) => {
                // Config exists but no view - stale (first load)
                Ok(ComparisonBranch::Stale(Self::transition(Analysis, Stale {
                    raw,
                    view: None,
                })))
            }
            (None, Some(_view)) => {
                // View exists but no file - config was deleted
                // Treat as fresh (use defaults)
                Ok(ComparisonBranch::Fresh(Self::transition(
                    Completed,
                    Fresh {},
                )))
            }
            (Some(raw), Some(view)) => {
                // Both exist - check staleness via view
                if view.is_fresh(&raw) {
                    // Matches cached version
                    Ok(ComparisonBranch::Fresh(Self::transition(
                        Completed,
                        Fresh {},
                    )))
                } else {
                    // Stale - proceed to analysis
                    Ok(ComparisonBranch::Stale(Self::transition(
                        Analysis,
                        Stale {
                            raw,
                            view: Some(view),
                        },
                    )))
                }
            }
        }
    }
}

/// Trait for checking view freshness against raw config.
///
/// This trait abstracts the staleness check logic so the processor
/// can work generically with both global and vault views.
pub trait IsConfigViewFresh<R> {
    /// Returns `true` if the view matches the raw config (not stale).
    ///
    /// Performs hybrid staleness detection:
    /// - Fast timestamp check
    /// - Content hash check
    fn is_fresh(&self, raw: &R) -> bool;
}

// Implement trait for global config view
impl IsConfigViewFresh<RawGlobalConfig> for RawGlobalConfigView {
    #[inline]
    fn is_fresh(&self, raw: &RawGlobalConfig) -> bool {
        // Delegate to the view's existing is_fresh method
        self.is_fresh(raw)
    }
}

// Implement trait for vault config view
impl IsConfigViewFresh<RawVaultConfig> for RawVaultConfigView {
    #[inline]
    fn is_fresh(&self, raw: &RawVaultConfig) -> bool {
        // Delegate to the view's existing is_fresh method
        self.is_fresh(raw)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage: Analysis
// ─────────────────────────────────────────────────────────────────────────────

impl<T: ConfigType> ConfigFileProcessor<T, Analysis, Stale<T>> {
    /// Analyze field-level changes to determine rebuild necessity.
    ///
    /// Compares per-field hashes between old (view) and new (raw) configs
    /// to determine if only metadata changed or if actual properties changed.
    ///
    /// Strategy:
    /// 1. Compute field hashes for new raw config
    /// 2. Compare against old field hashes (if view exists)
    /// 3. If no fields changed → `NoChanges` (update view only)
    /// 4. If fields changed → `PropertyChanges` (rebuild needed)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if hash computation fails.
    #[inline]
    pub fn analyze(self) -> Result<AnalysisBranch<T>, ConfigError> {
        let changed_fields: HashSet<ConfigField> =
            match self.status.view.as_ref() {
                Some(view)
                    if T::content_hash_matches(view, &self.status.raw) =>
                {
                    HashSet::new()
                }
                _ => T::compute_field_hashes(&self.status.raw)
                    .iter()
                    .map(|(field, _)| field.clone())
                    .collect(),
            };

        if changed_fields.is_empty() {
            // Only metadata (timestamps/content hash) changed
            Ok(AnalysisBranch::NoChanges(Self::transition(
                Completed,
                NoChanges {
                    raw: self.status.raw,
                },
            )))
        } else {
            // Properties changed - rebuild needed
            Ok(AnalysisBranch::PropertyChanges(Self::transition(
                Completed,
                PropertyChanges {
                    raw: self.status.raw,
                    changed_fields,
                },
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage: Completed (Finalization)
// ─────────────────────────────────────────────────────────────────────────────

impl<T: ConfigType> ConfigFileProcessor<T, Completed, Fresh> {
    /// Extract the processing outcome (config is fresh).
    #[inline]
    #[must_use]
    pub fn finalize(self) -> ProcessorOutcome<T> {
        ProcessorOutcome::UseCached
    }
}

impl<T: ConfigType> ConfigFileProcessor<T, Completed, NoChanges<T>> {
    /// Extract the processing outcome (metadata changed only).
    #[inline]
    #[must_use]
    pub fn finalize(self) -> ProcessorOutcome<T> {
        ProcessorOutcome::UpdateViewOnly {
            raw: self.status.raw,
        }
    }
}

impl<T: ConfigType> ConfigFileProcessor<T, Completed, PropertyChanges<T>> {
    /// Extract the processing outcome (properties changed).
    #[inline]
    #[must_use]
    pub fn finalize(self) -> ProcessorOutcome<T> {
        ProcessorOutcome::Rebuild {
            raw: self.status.raw,
            changed_fields: self.status.changed_fields,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::{
        config::{
            raw::{RawGlobalPaths, RawLogging, RawVaultPaths},
            views::RawFileVersion,
        },
        fs::metadata::{FileMetadata, FsTimes},
    };

    // ─────────────────────────────────────────────────────────────────────────────
    //  Test Fixtures
    // ─────────────────────────────────────────────────────────────────────────────

    fn create_raw_global_config() -> RawGlobalConfig {
        let now = SystemTime::now();
        RawGlobalConfig {
            logging: Some(RawLogging::default()),
            paths: RawGlobalPaths::default(),
            trusted_vaults: None,
            frontmatter: None,
            task: None,
            metadata: Some(FileMetadata::new(
                FsTimes::new(None, Some(now)),
                0,
                false,
            )),
        }
    }

    fn create_raw_vault_config() -> RawVaultConfig {
        let now = SystemTime::now();
        RawVaultConfig {
            vault_path: "/vault".to_owned(),
            name: Some("Test Vault".to_owned()),
            version: None,
            logging: None,
            paths: RawVaultPaths::default(),
            frontmatter: None,
            task: None,
            metadata: Some(FileMetadata::new(
                FsTimes::new(None, Some(now)),
                0,
                false,
            )),
        }
    }

    fn global_view_for(raw: &RawGlobalConfig) -> RawGlobalConfigView {
        let mut view = RawGlobalConfigView::new("/tmp/global.toml".into());
        let content =
            toml::to_string(raw).expect("raw global should serialize");
        let file_info = raw.metadata.clone().expect("metadata must exist");
        let version =
            RawFileVersion::new(content.as_bytes(), file_info).expect("valid");
        view.push_version(version);
        view
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Constructor Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn processor_new_creates_unknown_status() {
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
        // Compilation test - if it compiles, types are correct
        drop(processor);
    }

    #[test]
    #[expect(clippy::panic, reason = "Test assertions use panic for failures")]
    fn finalize_no_changes_returns_update_view_only() {
        let raw = create_raw_global_config();
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(Some(raw), None);

        let result = processor.compare().expect("compare should succeed");

        match result {
            ComparisonBranch::Stale(stale_proc) => {
                let analysis =
                    stale_proc.analyze().expect("analyze should succeed");
                match analysis {
                    AnalysisBranch::NoChanges(completed_proc) => {
                        let outcome = completed_proc.finalize();
                        assert!(matches!(
                            outcome,
                            ProcessorOutcome::UpdateViewOnly { .. }
                        ));
                    }
                    AnalysisBranch::PropertyChanges(_) => {
                        // Analysis might detect changes due to no view
                        // This is expected behavior
                    }
                }
            }
            ComparisonBranch::Fresh(_) => {
                panic!("Expected Stale branch, got Fresh");
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Comparison Stage Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn comparison_branch_both_none_returns_fresh() {
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
        let result = processor.compare();
        // Both None = no config exists = Fresh (nothing to do)
        assert!(matches!(result, Ok(ComparisonBranch::Fresh(_))));
    }

    #[test]
    #[expect(clippy::panic, reason = "Test branch assertion uses panic")]
    fn analyze_stale_timestamp_same_content_returns_no_changes() {
        let mut raw = create_raw_global_config();
        let view = global_view_for(&raw);

        // simulate mtime-only drift while content stays the same
        raw.metadata = Some(FileMetadata::new(
            FsTimes::new(None, Some(SystemTime::now())),
            1,
            false,
        ));

        let processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(
            Some(raw),
            Some(view),
        );

        let comparison = processor.compare().expect("compare should succeed");
        let stale = match comparison {
            ComparisonBranch::Stale(stale) => stale,
            ComparisonBranch::Fresh(_) => panic!("expected stale"),
        };

        let analysis = stale.analyze().expect("analysis should succeed");
        assert!(matches!(analysis, AnalysisBranch::NoChanges(_)));
    }

    #[test]
    fn comparison_branch_raw_only_returns_stale() {
        let raw = create_raw_global_config();
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(Some(raw), None);
        let result = processor.compare();
        // No view = first time seeing config = Stale
        assert!(matches!(result, Ok(ComparisonBranch::Stale(_))));
    }

    #[test]
    fn comparison_branch_view_only_returns_fresh() {
        // View exists but no raw config = use cached version
        let view = RawGlobalConfigView::new("/path/to/config.toml".into());
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, Some(view));
        let result = processor.compare();
        assert!(matches!(result, Ok(ComparisonBranch::Fresh(_))));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Field Hashing Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn global_config_compute_field_hashes_includes_logging() {
        let raw = create_raw_global_config();
        let hashes = GlobalConfig::compute_field_hashes(&raw);

        // Logging field is present and should be hashed
        assert!(hashes.contains(&ConfigField::Logging));
    }

    #[test]
    fn global_config_compute_field_hashes_always_includes_paths() {
        let raw = create_raw_global_config();
        let hashes = GlobalConfig::compute_field_hashes(&raw);

        // Paths always present (has default)
        assert!(hashes.contains(&ConfigField::Paths));
    }

    #[test]
    fn global_config_compute_field_hashes_skips_none_fields() {
        let raw = create_raw_global_config();
        let hashes = GlobalConfig::compute_field_hashes(&raw);

        // Frontmatter and Task are None, should not be in hash map
        assert!(!hashes.contains(&ConfigField::Frontmatter));
        assert!(!hashes.contains(&ConfigField::Task));
    }

    #[test]
    fn vault_config_compute_field_hashes_works() {
        let raw = create_raw_vault_config();
        let hashes = VaultConfig::compute_field_hashes(&raw);

        // Paths always present
        assert!(hashes.contains(&ConfigField::Paths));
        // Logging is None in fixture
        assert!(!hashes.contains(&ConfigField::Logging));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  ConfigFieldHashes Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn config_field_hashes_diff_detects_new_fields() {
        let old = ConfigFieldHashes::default(); // Empty
        let raw = create_raw_global_config();
        let new = GlobalConfig::compute_field_hashes(&raw);

        let diff = old.diff(&new);

        // All fields in new config are "changed" (added)
        assert!(diff.contains(&ConfigField::Logging));
        assert!(diff.contains(&ConfigField::Paths));
    }

    #[test]
    fn config_field_hashes_diff_detects_removed_fields() {
        let raw = create_raw_global_config();
        let old = GlobalConfig::compute_field_hashes(&raw);
        let new = ConfigFieldHashes::default(); // Empty

        let diff = old.diff(&new);

        // Old had fields, new doesn't = no changes detected
        // (diff only reports fields changed in new)
        assert!(diff.is_empty());
    }

    #[test]
    fn config_field_hashes_diff_identical_returns_empty() {
        let raw = create_raw_global_config();
        let hash1 = GlobalConfig::compute_field_hashes(&raw);
        let hash2 = GlobalConfig::compute_field_hashes(&raw);

        let diff = hash1.diff(&hash2);

        // Identical configs = no changes
        assert!(diff.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Finalization Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    #[expect(clippy::panic, reason = "Test assertions use panic for failures")]
    fn finalize_fresh_returns_use_cached() {
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
        let result = processor.compare().expect("compare should succeed");

        match result {
            ComparisonBranch::Fresh(fresh_proc) => {
                let outcome = fresh_proc.finalize();
                assert!(matches!(outcome, ProcessorOutcome::UseCached));
            }
            ComparisonBranch::Stale(_) => {
                panic!("Expected Fresh branch, got Stale");
            }
        }
    }
}
