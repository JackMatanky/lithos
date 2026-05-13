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
//!   `Changed`, `Fresh`, `FreshReady`, `NewReady`, `StaleReady`).
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
//!         p.parse(&source, &config_path)?.create(filename, &repo)?
//!     }
//!     ComparisonBranch::Present(p) => {
//!         match p.check_timestamps(&source, &config_path)? {
//!             TimestampBranch::Match(p) => p.fetch(&repo)?,
//!             TimestampBranch::Mismatch(p) => {
//!                 match p.check_content() {
//!                     ContentBranch::Match(p) => p.sync_metadata(&repo)?,
//!                     ContentBranch::Mismatch(p) => {
//!                         let parsed = p.parse(&config_path)?;
//!                         match parsed.analyze() {
//!                             AnalysisBranch::Empty(p) => p.sync_metadata(&repo)?,
//!                             AnalysisBranch::Delta(p) => p.update(filename, &repo)?,
//!                             AnalysisBranch::Corrupt(p) => p.create(filename, &repo)?,
//!                         }
//!                     }
//!                 }
//!             }
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
    fs::{FileInfo, FsReader, RelativePath},
    schema::{
        bank::PropertyBank,
        delta::{PropertyDelta, PropertyDeltaEngine},
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
            SchemaStorageError,
        },
        property::PropertyName,
        raw::RawPropertyBank,
        repository::{SchemaReadRepository, SchemaWriteRepository},
        views::{
            HashRecord, RawPropertyBankView, RawView as _, RawViewRead as _,
            contracts::Version as _,
        },
    },
    support::hash::Blake3Hash,
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
    pub(crate) fn transition<NP, NS>(
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
    Missing(PropertyBankProcessor<Parsed, Missing>),
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

/// Proven: View does not exist in repository; carries file info.
#[derive(Debug)]
pub(crate) struct Missing {
    info: FileInfo,
}

impl Missing {
    #[expect(dead_code, reason = "reserved for future use")]
    pub(crate) fn info(&self) -> &FileInfo {
        &self.info
    }

    pub(crate) fn new(info: FileInfo) -> Self {
        Self {
            info,
        }
    }
}

/// Proven: View exists in repository; carries file info and cached view.
#[derive(Debug)]
pub(crate) struct Present {
    info: FileInfo,
    view: RawPropertyBankView,
}

impl Present {
    #[expect(dead_code, reason = "reserved for future use")]
    pub(crate) fn info(&self) -> &FileInfo {
        &self.info
    }

    #[expect(dead_code, reason = "reserved for future use")]
    pub(crate) fn view(&self) -> &RawPropertyBankView {
        &self.view
    }

    pub(crate) fn new(info: FileInfo, view: RawPropertyBankView) -> Self {
        Self {
            info,
            view,
        }
    }
}

/// Proven: binary identity has diverged; carries content for hashing/parsing.
#[derive(Debug)]
pub(crate) struct Suspect {
    info: FileInfo,
    view: RawPropertyBankView,
    content: String,
}

/// Proven: Content hash mismatch; file content retained for parsing.
/// Transitions to Analysis.
#[derive(Debug)]
pub(crate) struct Stale {
    info: FileInfo,
    content: String,
    content_hash: Blake3Hash,
    view: RawPropertyBankView,
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

/// Present-view operations that compare timestamps against the cache.
impl PropertyBankProcessor<Comparison, Present> {
    /// Checks if file timestamps match the cached view.
    ///
    /// Reads the file content internally using the provided `FsReader` and
    /// path. Content is only retained on mismatch to avoid unnecessary
    /// allocation.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be read.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn check_timestamps(
        self,
        source: &FsReader,
        config_path: &std::path::Path,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let timestamps_match = self.status.view.is_timestamp_match(
            self.status.info.created_at(),
            self.status.info.modified_at(),
        );

        if timestamps_match {
            Ok(TimestampBranch::Match(Self::transition(Construction, Fresh)))
        } else {
            let content = source
                .read_to_string(config_path)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

            Ok(TimestampBranch::Mismatch(Self::transition(
                Comparison,
                Suspect {
                    info: self.status.info,
                    view: self.status.view,
                    content,
                },
            )))
        }
    }
}

/// Result of content hash comparison in the Comparison stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ContentBranch {
    /// Content hash matches; only timestamps need updating.
    Match(PropertyBankProcessor<Refresh, StaleTimestamps>),
    /// Content hash mismatches; proceed to Parsed stage for parsing.
    Mismatch(PropertyBankProcessor<Parsed, Stale>),
}

/// Suspect operations that compare content hashes after timestamp drift.
impl PropertyBankProcessor<Comparison, Suspect> {
    /// Checks if the content hash matches the cached view.
    ///
    /// Transitions to Parsed stage with Stale on mismatch.
    #[inline]
    pub(crate) fn check_content(self) -> ContentBranch {
        let content_hash = Blake3Hash::compute(self.status.content.as_bytes());

        let content_match = self.status.view.is_content_match(&content_hash);

        if content_match {
            ContentBranch::Match(Self::transition(Refresh, StaleTimestamps {
                info: self.status.info,
                view: self.status.view,
            }))
        } else {
            ContentBranch::Mismatch(Self::transition(Parsed, Stale {
                info: self.status.info,
                content: self.status.content,
                content_hash,
                view: self.status.view,
            }))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Parsed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Parsing phase: file content has been parsed into a raw bank.
/// This stage sits between Comparison and Analysis.
///
/// Path A (Missing): Missing → Parsed (Missing) → Construction (New).
/// Path B (Content mismatch): Suspect → Parsed (Stale) → Analysis.
#[derive(Debug)]
pub(crate) struct Parsed;

/// Proven: Content has been parsed from Stale status.
/// Transitions to Analysis for property hash comparison.
#[derive(Debug)]
pub(crate) struct ParsedStale {
    raw: RawPropertyBank,
    content_hash: Blake3Hash,
    view: RawPropertyBankView,
}

/// Missing operations in the Parsed stage: parse new file content.
impl PropertyBankProcessor<Parsed, Missing> {
    /// Parses new file content into a raw bank and transitions directly to
    /// Construction (New).
    ///
    /// This is Path A: Missing → Parsed (Missing) → Construction (New).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn parse(
        self,
        source: &FsReader,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankProcessor<Construction, New>, SchemaLoaderError>
    {
        let content = source
            .read_to_string(config_path)
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw: RawPropertyBank =
            FsReader::parse_structured_from_str(config_path, &content)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_info(self.status.info);
        let content_hash = Blake3Hash::compute(content.as_bytes());

        Ok(Self::transition(Construction, New {
            raw,
            content_hash,
        }))
    }
}

/// Stale operations: parse content and transition to `ParsedStale`.
impl PropertyBankProcessor<Parsed, Stale> {
    /// Parses content and transitions to `ParsedStale`.
    ///
    /// This is the entry point for Path B where content was already read
    /// in the Comparison stage.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if parsing fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn parse(
        self,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankProcessor<Analysis, ParsedStale>, SchemaLoaderError>
    {
        let raw: RawPropertyBank = FsReader::parse_structured_from_str(
            config_path,
            &self.status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_info(self.status.info);

        Ok(Self::transition(Analysis, ParsedStale {
            raw,
            content_hash: self.status.content_hash,
            view: self.status.view,
        }))
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
    /// View has no current version (corruption); treat as new.
    Corrupt(PropertyBankProcessor<Construction, New>),
}

/// Analysis operations that compute property-level deltas.
impl PropertyBankProcessor<Analysis, ParsedStale> {
    /// Compares property-level hashes and produces the analysis result.
    ///
    /// An empty delta transitions to `StaleContent`; a non-empty delta
    /// transitions to `Changed`. If view has no current version (corruption),
    /// transitions to `New`.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub(crate) fn analyze(self) -> AnalysisBranch {
        let raw = &self.status.raw;
        let content_hash = self.status.content_hash;
        let view = self.status.view;

        let Some(version) = view.current() else {
            return AnalysisBranch::Corrupt(Self::transition(
                Construction,
                New {
                    raw: raw.clone(),
                    content_hash,
                },
            ));
        };

        let Ok((delta, property_hashes)) =
            PropertyDeltaEngine::for_property_bank(
                raw,
                version.hashes().properties(),
            )
            .diff_property_bank()
        else {
            return AnalysisBranch::Corrupt(Self::transition(
                Construction,
                New {
                    raw: raw.clone(),
                    content_hash,
                },
            ));
        };
        let raw_hash = HashRecord::new(content_hash, property_hashes);

        if delta.is_empty() {
            AnalysisBranch::Empty(Self::transition(Refresh, StaleContent {
                info: *raw.info(),
                view,
                content_hash,
            }))
        } else {
            AnalysisBranch::Delta(Self::transition(Construction, Changed {
                raw: raw.clone(),
                delta,
                raw_hash,
            }))
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
    info: FileInfo,
    view: RawPropertyBankView,
}

/// Proven: property hashes match; content hash differs.
#[derive(Debug)]
pub(crate) struct StaleContent {
    info: FileInfo,
    view: RawPropertyBankView,
    content_hash: Blake3Hash,
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
    pub(crate) fn sync_metadata<R: SchemaWriteRepository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        self.status.view.update_file_info(self.status.info);

        repository
            .save_raw_property_bank_view(
                self.status.view.file_path(),
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
    pub(crate) fn sync_metadata<R: SchemaWriteRepository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        self.status.view.update_file_info(self.status.info);
        self.status
            .view
            .update_content_hash(self.status.content_hash)
            .map_err(SchemaRepositoryError::Storage)
            .map_err(SchemaLoaderError::Repository)?;

        repository
            .save_raw_property_bank_view(
                self.status.view.file_path(),
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

/// Proven: initial ingestion path selected; carries raw bank and content hash.
#[derive(Debug)]
pub(crate) struct New {
    raw: RawPropertyBank,
    content_hash: Blake3Hash,
}

/// Proven: property divergence detected; carries raw bank, delta, and raw
/// hashes.
#[derive(Debug)]
pub(crate) struct Changed {
    raw: RawPropertyBank,
    delta: PropertyDelta,
    raw_hash: HashRecord,
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
    pub(crate) fn create<R: SchemaWriteRepository>(
        self,
        path: &RelativePath,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, NewReady>, SchemaLoaderError>
    {
        let bank = PropertyBank::try_from(self.status.raw.clone()).map_err(
            |source| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                    path: std::path::PathBuf::from("property_bank"),
                    source,
                })
            },
        )?;
        self.persist(path, repository, &bank)?;

        Ok(Self::transition(Completed, NewReady {
            bank,
        }))
    }

    #[inline]
    fn persist<R: SchemaWriteRepository>(
        &self,
        path: &RelativePath,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<(), SchemaLoaderError> {
        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let property_hashes = self.status.raw.properties().compute_hashes();
        let raw_hash =
            HashRecord::new(self.status.content_hash, property_hashes.into());

        let view = RawPropertyBankView::try_from_raw_with_hashes(
            &self.status.raw,
            path.clone(),
            raw_hash,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(path, &view)
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
    pub(crate) fn update<R: SchemaReadRepository + SchemaWriteRepository>(
        self,
        path: &RelativePath,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, StaleReady>, SchemaLoaderError>
    {
        let mut bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaLoaderError::Ingestion(
                SchemaIngestionError::Storage(
                    SchemaStorageError::PropertyBankNotFound,
                ),
            ))?;

        self.apply_delta(&mut bank);
        self.persist(path, repository, &bank)?;
        Ok(Self::transition(Completed, StaleReady {
            bank,
            delta: self.status.delta.into_changed_name_set(),
        }))
    }

    fn apply_delta(&self, bank: &mut PropertyBank) {
        if self.status.delta.is_empty() {
            return;
        }

        let existing = bank.set_properties();
        let upserts = self.status.delta.upserts().clone().with_ids(existing);
        for (name, property) in upserts {
            existing.insert(name, property);
        }

        for name in self.status.delta.removals() {
            existing.remove(name);
        }

        *bank.set_recorded_at() = SystemTime::now();
    }

    #[inline]
    fn persist<R: SchemaWriteRepository>(
        &self,
        path: &RelativePath,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<(), SchemaLoaderError> {
        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let view = RawPropertyBankView::try_from_raw_with_hashes(
            &self.status.raw,
            path.clone(),
            self.status.raw_hash.clone(),
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(path, &view)
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
    pub(crate) fn fetch<R: SchemaReadRepository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, FreshReady>, SchemaLoaderError>
    {
        drop(self);
        let bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaIngestionError::Storage(
                SchemaStorageError::PropertyBankNotFound,
            ))?;

        Ok(Self::transition(Completed, FreshReady {
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

/// Proven: terminal ingestion goal reached with fresh bank.
#[derive(Debug)]
pub(crate) struct FreshReady {
    bank: PropertyBank,
}

/// Proven: terminal ingestion goal reached with newly built bank.
#[derive(Debug)]
pub(crate) struct NewReady {
    bank: PropertyBank,
}

/// Proven: terminal ingestion goal reached with stale updates applied.
#[derive(Debug)]
pub(crate) struct StaleReady {
    bank: PropertyBank,
    delta: HashSet<PropertyName>,
}

/// Completed operations that expose the final property bank.
impl PropertyBankProcessor<Completed, FreshReady> {
    /// Extracts the completed `PropertyBank`.
    #[inline]
    #[must_use]
    pub(crate) fn into_bank(self) -> PropertyBank {
        self.status.bank
    }
}

impl PropertyBankProcessor<Completed, NewReady> {
    /// Extracts the completed `PropertyBank`.
    #[inline]
    #[must_use]
    pub(crate) fn into_bank(self) -> PropertyBank {
        self.status.bank
    }
}

impl PropertyBankProcessor<Completed, StaleReady> {
    /// Extracts the completed `PropertyBank` and changed property names.
    #[inline]
    #[must_use]
    pub(crate) fn into_bank_with_changes(
        self,
    ) -> (PropertyBank, HashSet<PropertyName>) {
        (self.status.bank, self.status.delta)
    }
}
