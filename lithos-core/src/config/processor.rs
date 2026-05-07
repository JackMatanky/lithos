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
//! ```rust,no_run
//! use lithos_core::config::{
//!     processor::{
//!         ConfigFileProcessor, GlobalConfig,
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
}

/// Marker type for global config processing.
#[derive(Debug)]
#[non_exhaustive]
pub struct GlobalConfig;

impl ConfigType for GlobalConfig {
    type Raw = RawGlobalConfig;
    type View = RawGlobalConfigView;

    #[inline]
    fn compute_field_hashes(_raw: &Self::Raw) -> ConfigFieldHashes {
        // TODO: Implement field-level hashing
        ConfigFieldHashes::default()
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
    fn compute_field_hashes(_raw: &Self::Raw) -> ConfigFieldHashes {
        // TODO: Implement field-level hashing
        ConfigFieldHashes::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigFieldHashes (Field-Level Change Detection)
// ─────────────────────────────────────────────────────────────────────────────

/// Per-field hash map for incremental config analysis.
///
/// Similar to schema's `RawPropertyMapHash`, this enables detecting which
/// specific config fields changed without reparsing the entire file.
///
/// # Examples
///
/// ```rust
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
    inner: std::collections::HashMap<
        ConfigField,
        crate::support::hash::Blake3Hash,
    >,
}

impl ConfigFieldHashes {
    /// Creates an empty field hash map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: std::collections::HashMap::new(),
        }
    }

    /// Inserts a field hash.
    #[inline]
    pub fn insert(
        &mut self,
        field: ConfigField,
        hash: crate::support::hash::Blake3Hash,
    ) {
        self.inner.insert(field, hash);
    }

    /// Returns the hash for a field, if present.
    #[inline]
    #[must_use]
    pub fn get(
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
    pub fn iter(
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

        #[expect(
            clippy::iter_over_hash_type,
            reason = "Field order irrelevant for change detection"
        )]
        for (field, new_hash) in &new.inner {
            if self.inner.get(field) != Some(new_hash) {
                changed.insert(field.clone());
            }
        }

        changed
    }
}

/// Config field identifier for change tracking.
///
/// Represents the top-level fields in `RawConfig` that can be independently
/// hashed for incremental analysis.
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
        // Compute field hashes for new config
        let _new_hashes = T::compute_field_hashes(&self.status.raw);

        // Get old field hashes from view (if exists)
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Borrowing semantics are clear in context"
        )]
        let changed_fields: HashSet<ConfigField> = match &self.status.view {
            Some(_view) => {
                // TODO: Extract field hashes from view's latest version
                // For now, assume all fields changed if view exists
                // This will be implemented when we add field_hashes to
                // RawFileVersion
                let _old_hashes = ConfigFieldHashes::default();
                // changed_fields = old_hashes.diff(&new_hashes);

                // Temporary: assume all fields changed
                vec![
                    ConfigField::Logging,
                    ConfigField::Paths,
                    ConfigField::Task,
                    ConfigField::Frontmatter,
                ]
                .into_iter()
                .collect()
            }
            None => {
                // No view means first load - all fields are "changed"
                vec![
                    ConfigField::Logging,
                    ConfigField::Paths,
                    ConfigField::Task,
                    ConfigField::Frontmatter,
                ]
                .into_iter()
                .collect()
            }
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
    use super::*;

    #[test]
    fn processor_new_creates_unknown_status() {
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
        // Compilation test - if it compiles, types are correct
        drop(processor);
    }

    #[test]
    fn comparison_branch_both_none_returns_fresh() {
        let processor =
            ConfigFileProcessor::<GlobalConfig, _, _>::new(None, None);
        let result = processor.compare();
        // Both None = no config exists = Fresh (nothing to do)
        assert!(matches!(result, Ok(ComparisonBranch::Fresh(_))));
    }
}
