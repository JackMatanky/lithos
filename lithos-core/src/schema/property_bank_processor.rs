//! Typed processing pipeline for building or updating a `PropertyBank`.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline that chooses the cheapest valid
//! path to a final `PropertyBank`. It uses two compile-time dimensions:
//!
//! - **Stage**: the current pipeline phase (`Discovery`, `Comparison`,
//!   `Analysis`, `Refresh`, `Construction`, `Completed`).
//! - **Status**: the knowledge state carrying data and invariants (`Unknown`,
//!   `Missing`, `Present`, `Suspect`, `StaleTimestamps`, `StaleContent`, `New`,
//!   `Changed`, `Fresh`, `Ready`).
//!
//! The dual-typestate design prevents invalid transitions at compile time and
//! keeps orchestration in the [`Builder`](crate::schema::Builder).
//!
//! # Design
//!
//! The pipeline follows a "cheap to expensive" hierarchy:
//!
//! 1. **Discovery**: determine if a cached view exists.
//! 2. **Comparison**: compare timestamps, then content hash.
//! 3. **Analysis**: parse and compare per-property hashes.
//! 4. **Refresh**: early-commit metadata when only timestamps/content changed.
//! 5. **Construction**: create, update, or fetch the domain bank.
//! 6. **Completed**: produce the final `PropertyBank`.
//!
//! # Flow
//!
//! ```text
//! Discovery
//!   ├─ No view
//!   │   → [Comparison] parse raw file
//!   │   → [Construction] construct domain from raw → Completed
//!   └─ View found
//!       → [Comparison] check timestamps (content retained)
//!
//! Timestamp Check
//!   ├─ [match]
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Comparison] check content hash
//!
//! Content Check
//!   ├─ [match]
//!   │   → [Refresh] sync timestamps
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Analysis] analyze property hashes
//!
//! Property Analysis
//!   ├─ [no changes]
//!   │   → [Refresh] sync timestamps + content hash
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [changes]
//!       → [Construction] update domain from delta → Completed
//! ```
//!
//! # Invariants
//!
//! - `Present` guarantees a loaded `RawPropertyBankView`.
//! - `Suspect` guarantees file content for hashing/parsing.
//! - `StaleTimestamps` means content is identical; only timestamps changed.
//! - `StaleContent` means property hashes match; content hash differs.
//! - `Fresh` means the stored bank can be fetched without rebuilding.
//!
//! # Usage
//!
//! ```ignore
//! use lithos_core::schema::property_bank_processor::{
//!     AnalysisBranch, ComparisonBranch, ContentBranch, Discovery,
//!     PropertyBankProcessor, TimestampBranch, Unknown,
//! };
//!
//! let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
//! let branch = pipeline.discover(filename, &source, &config_path, &repo)?;
//!
//! match branch {
//!     ComparisonBranch::Missing(p) => {
//!         let content = source.read_to_string(&config_path)?;
//!         p.parse(&config_path, &content)?.create(filename, &repo)?
//!     }
//!     ComparisonBranch::Present(p) => {
//!         let content = source.read_to_string(&config_path)?;
//!         match p.check_timestamps(&content) {
//!             TimestampBranch::Match(p) => p.fetch(&repo)?,
//!             TimestampBranch::Mismatch(p) => match p.check_content(&config_path) {
//!                 ContentBranch::Match(p) => p.sync_metadata(&repo)?,
//!                 ContentBranch::Mismatch(p) => match p.analyze(&config_path)? {
//!                     AnalysisBranch::Empty(p) => p.sync_metadata(&repo)?,
//!                     AnalysisBranch::Delta(p) => p.update(filename, &repo)?,
//!                 },
//!             },
//!         }
//!     }
//! }
//! ```
//!
//! # Maintenance Notes
//!
//! - Add new stages/statuses only when they introduce a new invariant or reduce
//!   work; each state must carry the data needed to satisfy its invariant.
//! - `sync_metadata` is an early-commit checkpoint and intentionally
//!   side-effecting to avoid repeated parsing on retries.

use std::{collections::HashSet, marker::PhantomData, time::SystemTime};

use crate::{
    fs::FsReader,
    schema::{
        bank::PropertyBank,
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
            SchemaStorageError,
        },
        property::{Property, PropertyName},
        raw::{RawFileTimes, RawPropertyBank, RawPropertyBankEntry},
        storage::Repository,
        views::{
            FileTimesMetadata, RawPropertyBankView, metadata::HashMetadata,
        },
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Processor Core
// ─────────────────────────────────────────────────────────────────────────────

/// Core state machine tracking the current pipeline stage and knowledge status
/// via compile-time types.
///
/// This struct uses a dimensional typestate pattern with two generic
/// parameters, where stages are markers and statuses carry data:
///
/// - `P` (Stage): the current phase of the pipeline.
/// - `S` (Status): the knowledge state carrying data and invariants.
#[derive(Debug)]
#[must_use]
pub(crate) struct PropertyBankProcessor<P, S> {
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> PropertyBankProcessor<P, S> {
    /// Internal constructor for state transitions.
    #[inline]
    fn transition<NP, NS>(
        _stage: NP,
        status: NS,
    ) -> PropertyBankProcessor<NP, NS> {
        PropertyBankProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Discovery Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Entry phase: checking the repository for a cached view.
#[derive(Debug)]
pub(crate) struct Discovery;

/// Initial state before any knowledge has been gathered.
#[derive(Debug)]
pub(crate) struct Unknown;

/// Result of the Discovery stage, determining the next branch in the pipeline.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ComparisonBranch {
    /// No cached view exists; the file is new.
    Missing(PropertyBankProcessor<Comparison, Missing>),
    /// A cached view exists; proceed to comparison checks.
    Present(PropertyBankProcessor<Comparison, Present>),
}

/// Entry-state operations that decide whether a cached view exists.
impl PropertyBankProcessor<Discovery, Unknown> {
    /// Creates a new processor in the initial state.
    ///
    /// ```ignore
    /// # use lithos_core::schema::property_bank_processor::{
    /// #     Discovery, PropertyBankProcessor, Unknown,
    /// # };
    /// let processor = PropertyBankProcessor::<Discovery, Unknown>::new();
    /// ```
    #[inline]
    pub(crate) fn new() -> Self {
        PropertyBankProcessor {
            status: Unknown,
            _stage: PhantomData,
        }
    }

    /// Initial entry point: checks the repository for an existing view.
    ///
    /// This method gathers file times and queries the repository to decide
    /// whether the pipeline starts as a `Missing` (new ingestion) or a
    /// `Present` (incremental update) branch.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn discover<R: Repository>(
        self,
        filename: &str,
        source: &FsReader,
        config_path: &std::path::Path,
        repository: &R,
    ) -> Result<ComparisonBranch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        drop(self);
        let cached_view = repository
            .get_raw_property_bank_view(filename)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let times = RawFileTimes {
            created_at: source.created_at(config_path),
            modified_at: source.modified_at(config_path),
        };

        if let Some(view) = cached_view {
            Ok(ComparisonBranch::Present(Self::transition(
                Comparison,
                Present {
                    times,
                    view,
                },
            )))
        } else {
            Ok(ComparisonBranch::Missing(Self::transition(
                Comparison,
                Missing {
                    times,
                },
            )))
        }
    }
}

impl Default for PropertyBankProcessor<Discovery, Unknown> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Comparison Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Identity phase: comparing file attributes with cached metadata.
#[derive(Debug)]
pub(crate) struct Comparison;

/// Proven: View does not exist in repository; carries file timestamps.
#[derive(Debug)]
pub(crate) struct Missing {
    times: RawFileTimes,
}

/// Proven: View exists in repository; carries timestamps and cached view.
#[derive(Debug)]
pub(crate) struct Present {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

/// Proven: binary identity has diverged; carries content for hashing/parsing.
#[derive(Debug)]
pub(crate) struct Suspect {
    times: RawFileTimes,
    view: RawPropertyBankView,
    content: String,
}

/// Result of timestamp comparison in the Comparison stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum TimestampBranch {
    /// Timestamps match; the cached bank is fresh.
    Match(PropertyBankProcessor<Construction, Fresh>),
    /// Timestamps mismatch; proceed to content hash check.
    Mismatch(PropertyBankProcessor<Comparison, Suspect>),
}

/// Result of content hash comparison in the Comparison stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ContentBranch {
    /// Content hash matches; only timestamps need updating.
    Match(PropertyBankProcessor<Refresh, StaleTimestamps>),
    /// Content hash mismatches; proceed to delta analysis.
    Mismatch(PropertyBankProcessor<Analysis, Suspect>),
}

/// Missing-view operations that parse a new file into a raw bank.
impl PropertyBankProcessor<Comparison, Missing> {
    /// Parses new file content into a raw bank.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn parse(
        self,
        config_path: &std::path::Path,
        content: &str,
    ) -> Result<PropertyBankProcessor<Construction, New>, SchemaLoaderError>
    {
        let raw: RawPropertyBank =
            FsReader::parse_structured_from_str(config_path, content)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_file_times(self.status.times.clone());

        Ok(Self::transition(Construction, New {
            raw,
            content: content.into(),
        }))
    }
}

/// Present-view operations that compare timestamps against the cache.
impl PropertyBankProcessor<Comparison, Present> {
    /// Checks if file timestamps match the cached view.
    ///
    /// The `content` is only retained on mismatch to avoid re-reading the file.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn check_timestamps(self, content: &str) -> TimestampBranch {
        let timestamps_match = self.status.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                self.status.times.created_at,
                self.status.times.modified_at,
            )
        });

        if timestamps_match {
            TimestampBranch::Match(Self::transition(Construction, Fresh))
        } else {
            TimestampBranch::Mismatch(Self::transition(Comparison, Suspect {
                times: self.status.times,
                view: self.status.view,
                content: content.into(),
            }))
        }
    }
}

/// Suspect operations that compare content hashes after timestamp drift.
impl PropertyBankProcessor<Comparison, Suspect> {
    /// Checks if the content hash matches the cached view.
    ///
    /// The `config_path` is unused today but kept for transition parity with
    /// other methods that require it.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn check_content(
        self,
        _config_path: &std::path::Path,
    ) -> ContentBranch {
        let content_hash = blake3::hash(self.status.content.as_bytes());
        let content_match = self.status.view.current().is_some_and(|v| {
            v.hashes().is_content_match(content_hash.as_bytes())
        });

        if content_match {
            ContentBranch::Match(Self::transition(Refresh, StaleTimestamps {
                times: self.status.times,
                view: self.status.view,
            }))
        } else {
            ContentBranch::Mismatch(Self::transition(Analysis, self.status))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Analysis Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Semantic phase: computing the property delta between the file and view.
#[derive(Debug)]
pub(crate) struct Analysis;

/// Result of property delta analysis in the Analysis stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum AnalysisBranch {
    /// Content changed but properties did not.
    Empty(PropertyBankProcessor<Refresh, StaleContent>),
    /// Properties changed; proceed with delta update.
    Delta(PropertyBankProcessor<Construction, Changed>),
}

/// Property delta between the cached view and the new raw bank.
#[derive(Debug)]
struct PropertyDelta {
    /// New or changed properties.
    upserts: Vec<(PropertyName, RawPropertyBankEntry)>,
    /// Removed properties.
    removals: Vec<PropertyName>,
}

impl PropertyDelta {
    /// Returns `true` if there are no changes.
    #[inline]
    fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.removals.is_empty()
    }
}

/// Analysis operations that compute property-level deltas.
impl PropertyBankProcessor<Analysis, Suspect> {
    /// Parses the file and compares property-level hashes.
    ///
    /// An empty delta transitions to `StaleContent`; a non-empty delta
    /// transitions to `Changed`.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn analyze(
        self,
        config_path: &std::path::Path,
    ) -> Result<AnalysisBranch, SchemaLoaderError> {
        let raw: RawPropertyBank = FsReader::parse_structured_from_str(
            config_path,
            &self.status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_file_times(self.status.times.clone());
        let content_hash =
            *blake3::hash(self.status.content.as_bytes()).as_bytes();

        let delta = self.status.view.current().map_or_else(
            || Self::delta_from_new_file(&raw),
            |version| {
                Self::delta_from_cached_view(
                    &raw,
                    version.hashes().properties(),
                )
            },
        );

        if delta.is_empty() {
            Ok(AnalysisBranch::Empty(Self::transition(Refresh, StaleContent {
                times: self.status.times,
                view: self.status.view,
                content_hash,
            })))
        } else {
            Ok(AnalysisBranch::Delta(Self::transition(Construction, Changed {
                raw,
                delta,
                content: self.status.content,
            })))
        }
    }

    #[inline]
    fn delta_from_new_file(raw: &RawPropertyBank) -> PropertyDelta {
        let mut upserts = raw
            .properties()
            .iter()
            .map(|(name, entry)| (name.clone(), entry.clone()))
            .collect::<Vec<_>>();
        upserts.sort_by(|left, right| left.0.cmp(&right.0));

        PropertyDelta {
            upserts,
            removals: Vec::new(),
        }
    }

    #[inline]
    fn delta_from_cached_view(
        raw: &RawPropertyBank,
        prev_hashes: &std::collections::HashMap<PropertyName, [u8; 32]>,
    ) -> PropertyDelta {
        let mut upserts = Vec::new();
        let mut seen = HashSet::with_capacity(raw.properties().len());

        for (name, entry) in raw.properties().iter() {
            let new_hash = HashMetadata::hash_entry(entry);
            if prev_hashes.get(name) != Some(&new_hash) {
                upserts.push((name.clone(), entry.clone()));
            }
            seen.insert(name.clone());
        }

        let mut removals = prev_hashes
            .keys()
            .filter(|&name| !seen.contains(name))
            .cloned()
            .collect::<Vec<_>>();

        upserts.sort_by(|left, right| left.0.cmp(&right.0));
        removals.sort();

        PropertyDelta {
            upserts,
            removals,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Refresh Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Maintenance phase: early commitment of proven metadata.
#[derive(Debug)]
pub(crate) struct Refresh;

/// Proven: content hashes match; only timestamps differ.
#[derive(Debug)]
pub(crate) struct StaleTimestamps {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

/// Proven: property hashes match; content hash differs.
#[derive(Debug)]
pub(crate) struct StaleContent {
    times: RawFileTimes,
    view: RawPropertyBankView,
    content_hash: [u8; 32],
}

/// Refresh operations that sync only file timestamps.
impl PropertyBankProcessor<Refresh, StaleTimestamps> {
    /// Syncs file timestamps to the cached view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn sync_metadata<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let new_file_times = FileTimesMetadata::new(
            self.status.times.created_at,
            self.status.times.modified_at,
        );
        self.status.view.update_timestamps(new_file_times);

        repository
            .save_raw_property_bank_view(
                self.status.view.file_path().as_str(),
                &self.status.view,
            )
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(Self::transition(Construction, Fresh))
    }
}

/// Refresh operations that sync timestamps plus content hash.
impl PropertyBankProcessor<Refresh, StaleContent> {
    /// Syncs timestamps and content hash to the cached view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn sync_metadata<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let new_file_times = FileTimesMetadata::new(
            self.status.times.created_at,
            self.status.times.modified_at,
        );
        self.status.view.update_timestamps(new_file_times);
        self.status
            .view
            .update_content_hash(self.status.content_hash)
            .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(
                self.status.view.file_path().as_str(),
                &self.status.view,
            )
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(Self::transition(Construction, Fresh))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Building phase: terminal domain construction.
#[derive(Debug)]
pub(crate) struct Construction;

/// Proven: initial ingestion path selected; carries raw bank and content.
#[derive(Debug)]
pub(crate) struct New {
    raw: RawPropertyBank,
    content: String,
}

/// Proven: property divergence detected; carries raw bank, delta, content.
#[derive(Debug)]
pub(crate) struct Changed {
    raw: RawPropertyBank,
    delta: PropertyDelta,
    content: String,
}

/// Proven: identity is fully synchronized; bank can be fetched without rebuild.
#[derive(Debug)]
pub(crate) struct Fresh;

/// Construction operations that build the initial property bank.
impl PropertyBankProcessor<Construction, New> {
    /// Performs the initial full bank construction.
    ///
    /// Returns the terminal `Completed` state on success.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if construction or repository access
    /// fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn create<R: Repository>(
        self,
        filename: &str,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, Ready>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = self.build_bank()?;
        self.persist(filename, repository, &bank)?;

        Ok(Self::transition(Completed, Ready {
            bank,
        }))
    }

    #[inline]
    fn build_bank(&self) -> Result<PropertyBank, SchemaLoaderError> {
        let mut bank = PropertyBank::new();

        let mut entries: Vec<_> = self.status.raw.properties().iter().collect();
        entries.sort_by(|left, right| left.0.cmp(right.0));

        for (name, entry) in entries {
            let property = Property::try_from((name.clone(), entry.clone()))
                .map_err(|source| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                        path: std::path::PathBuf::from("property_bank"),
                        source,
                    })
                })?;
            bank.register(property).map_err(|source| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                    path: std::path::PathBuf::from("property_bank"),
                    source,
                })
            })?;
        }

        Ok(bank)
    }

    #[inline]
    fn persist<R: Repository>(
        &self,
        filename: &str,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<(), SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let view = RawPropertyBankView::try_from_raw_with_content(
            &self.status.raw,
            filename,
            &self.status.content,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(filename, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }
}

/// Construction operations that apply property deltas.
impl PropertyBankProcessor<Construction, Changed> {
    /// Applies incremental bank updates via property deltas.
    ///
    /// Returns the terminal `Completed` state on success.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if construction or repository access
    /// fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn update<R: Repository>(
        self,
        filename: &str,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, Ready>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let mut bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaLoaderError::Ingestion(
                SchemaIngestionError::Storage(
                    SchemaStorageError::PropertyBankNotFound,
                ),
            ))?;

        self.apply_delta(&mut bank)?;
        self.persist(filename, repository, &bank)?;

        Ok(Self::transition(Completed, Ready {
            bank,
        }))
    }

    #[inline]
    fn apply_delta(
        &self,
        bank: &mut PropertyBank,
    ) -> Result<(), SchemaLoaderError> {
        use std::collections::hash_map::Entry;

        let any_changed = !self.status.delta.is_empty();

        for pair in &self.status.delta.upserts {
            let name = &pair.0;
            let entry = &pair.1;
            let property = Property::try_from((name.clone(), entry.clone()))
                .map_err(|source| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                        path: std::path::PathBuf::from("property_bank"),
                        source,
                    })
                })?;
            match bank.set_properties().entry(name.clone()) {
                Entry::Occupied(mut occupied) => {
                    let existing_id = occupied.get().id();
                    let property = property.with_id(existing_id);
                    occupied.insert(property);
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(property);
                }
            }
        }

        for name in &self.status.delta.removals {
            bank.set_properties().remove(name);
        }

        if any_changed {
            let next = bank.version().increment();
            *bank.set_version() = next;
            *bank.set_recorded_at() = SystemTime::now();
        }

        Ok(())
    }

    #[inline]
    fn persist<R: Repository>(
        &self,
        filename: &str,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<(), SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let view = RawPropertyBankView::try_from_raw_with_content(
            &self.status.raw,
            filename,
            &self.status.content,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(filename, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }
}

/// Construction operations that fetch the cached bank as-is.
impl PropertyBankProcessor<Construction, Fresh> {
    /// Retrieves the already-current bank from the repository.
    ///
    /// Returns the terminal `Completed` state on success.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails or the bank
    /// is missing.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn fetch<R: Repository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, Ready>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        drop(self);
        let bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaIngestionError::Storage(
                SchemaStorageError::PropertyBankNotFound,
            ))?;

        Ok(Self::transition(Completed, Ready {
            bank,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal phase: the `PropertyBank` is ready and owned.
#[derive(Debug)]
pub(crate) struct Completed;

/// Proven: terminal ingestion goal reached; owns the final bank.
#[derive(Debug)]
pub(crate) struct Ready {
    bank: PropertyBank,
}

/// Completed operations that expose the final property bank.
impl PropertyBankProcessor<Completed, Ready> {
    /// Extracts the completed `PropertyBank`.
    #[inline]
    #[must_use]
    pub(crate) fn into_bank(self) -> PropertyBank {
        self.status.bank
    }
}
