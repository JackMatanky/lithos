//! Typed processing pipeline for building or updating a `PropertyBank`.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline that chooses the cheapest valid
//! path to a final `PropertyBank`. It uses two compile-time dimensions:
//!
//! - **Stage**: the current pipeline phase (`Comparison`, `Analysis`,
//!   `Refresh`, `Construction`, `Completed`).
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
//! 1. **Comparison**: compare timestamps, then content hash.
//! 2. **Analysis**: parse and compare per-property hashes.
//! 3. **Refresh**: early-commit metadata when only timestamps/content changed.
//! 4. **Construction**: create, update, or fetch the domain bank.
//! 5. **Completed**: produce the final `PropertyBank`.
//!
//! # Flow
//!
//! ```text
//! Entry
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
//!     AnalysisBranch, Comparison, ContentBranch,
//!     Init, PropertyBankProcessor, TimestampBranch, Unknown,
//! };
//!
//! let pipeline =
//!     PropertyBankProcessor::<Init, Unknown>::from_discovery(file, root)?;
//!
//! if let Some(view) = cached_view {
//!     let present = pipeline.transition(Comparison, Present::new(view));
//!     match present.check_timestamps(&source)? {
//!         TimestampBranch::Match(p) => p.fetch(&repo)?,
//!         TimestampBranch::Mismatch(p) => {
//!             match p.check_content() {
//!                 ContentBranch::Match(p) => p.sync_metadata(&repo)?,
//!                 ContentBranch::Mismatch(p) => {
//!                     let parsed = p.parse()?;
//!                     match parsed.analyze() {
//!                         AnalysisBranch::Empty(p) => p.sync_metadata(&repo)?,
//!                         AnalysisBranch::Delta(p) => p.update(&repo)?,
//!                         AnalysisBranch::Corrupt(p) => p.create(&repo)?,
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! } else {
//!     let missing = pipeline.transition(Parsed, Missing);
//!     missing.parse(&source)?.create(&repo)?;
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
    fs::{DirPath, FsFile, FsReader, PathKey},
    schema::{
        bank::PropertyBank,
        delta::{PropertyDelta, PropertyDeltaEngine},
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        property::PropertyName,
        raw::RawPropertyBank,
        repository::{ReadRepository, Repository, WriteRepository},
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
    file: FsFile,
    path_key: PathKey,
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> PropertyBankProcessor<P, S> {
    #[inline]
    fn into_parts(self) -> (FsFile, PathKey, S) {
        (self.file, self.path_key, self.status)
    }

    #[inline]
    fn transition_from_parts<NP, NS>(
        file: FsFile,
        path_key: PathKey,
        status: NS,
    ) -> PropertyBankProcessor<NP, NS> {
        PropertyBankProcessor {
            file,
            path_key,
            status,
            _stage: PhantomData,
        }
    }

    /// Internal constructor for state transitions.
    #[inline]
    fn transition<NP, NS>(
        self,
        _stage: NP,
        status: NS,
    ) -> PropertyBankProcessor<NP, NS> {
        let (file, path_key, _) = self.into_parts();
        Self::transition_from_parts(file, path_key, status)
    }
}

/// The result of a property bank resolution attempt.
pub(crate) struct PropertyBankResolution {
    bank: PropertyBank,
    delta: Option<HashSet<PropertyName>>,
}

impl PropertyBankResolution {
    /// Create a new resolution result.
    pub(crate) fn new(
        bank: PropertyBank,
        delta: Option<HashSet<PropertyName>>,
    ) -> Self {
        Self {
            bank,
            delta,
        }
    }

    /// Decompose the resolution into its constituent parts.
    pub(crate) fn into_parts(
        self,
    ) -> (PropertyBank, Option<HashSet<PropertyName>>) {
        (self.bank, self.delta)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Entry Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Initial state before any knowledge has been gathered.
#[derive(Debug)]
pub(crate) struct Unknown;

/// Entry-point stage: processor created from discovery data, not yet compared.
#[derive(Debug)]
pub(crate) struct Init;

/// Entry-state operations that bootstrap the comparison pipeline.
impl PropertyBankProcessor<Init, Unknown> {
    #[inline]
    pub(crate) fn from_discovery(
        file: FsFile,
        root: &DirPath,
    ) -> Result<Self, crate::fs::PathError> {
        let path_key = file.path().as_key(root)?;
        Ok(Self {
            file,
            path_key,
            status: Unknown,
            _stage: PhantomData,
        })
    }

    /// Run the full property bank pipeline.
    ///
    /// When `view` is `None`, the processor runs the missing path
    /// (parse from scratch → create).
    /// When `view` is `Some(...)`, the processor runs the present path
    /// (check timestamps → check content → analyze → create/update/fetch).
    pub(crate) fn run<R: Repository>(
        self,
        view: Option<&RawPropertyBankView>,
        source: &FsReader,
        repository: &R,
    ) -> Result<PropertyBankResolution, SchemaLoaderError> {
        if let Some(view) = view {
            let present =
                self.transition(Comparison, Present::new(view.clone()));
            Self::run_present(present, source, repository)
        } else {
            let constructed = self.transition(Parsed, Missing).parse(source)?;
            let completed = constructed.create(repository)?;
            Ok(PropertyBankResolution::new(completed.into_bank(), None))
        }
    }

    /// Internal helper for the present path (view exists).
    fn run_present<R: Repository>(
        processor: PropertyBankProcessor<Comparison, Present>,
        source: &FsReader,
        repository: &R,
    ) -> Result<PropertyBankResolution, SchemaLoaderError> {
        match processor.check_timestamps(source)? {
            TimestampBranch::Match(fresh) => {
                let completed = fresh.fetch(repository)?;
                Ok(PropertyBankResolution::new(completed.into_bank(), None))
            }
            TimestampBranch::Mismatch(suspect) => {
                Self::run_content_mismatch(suspect, repository)
            }
        }
    }

    /// Internal helper for content mismatch path.
    fn run_content_mismatch<R: Repository>(
        processor: PropertyBankProcessor<Comparison, Suspect>,
        repository: &R,
    ) -> Result<PropertyBankResolution, SchemaLoaderError> {
        match processor.check_content() {
            ContentBranch::Match(stale_ts) => {
                let fresh = stale_ts.sync_metadata(repository)?;
                let completed = fresh.fetch(repository)?;
                Ok(PropertyBankResolution::new(completed.into_bank(), None))
            }
            ContentBranch::Mismatch(stale) => {
                let parsed = stale.parse()?;
                Self::run_analysis(parsed.analyze(), repository)
            }
        }
    }

    /// Internal helper for analysis path.
    fn run_analysis<R: Repository>(
        branch: AnalysisBranch,
        repository: &R,
    ) -> Result<PropertyBankResolution, SchemaLoaderError> {
        match branch {
            AnalysisBranch::Empty(stale_content) => {
                let fresh = stale_content.sync_metadata(repository)?;
                let completed = fresh.fetch(repository)?;
                Ok(PropertyBankResolution::new(completed.into_bank(), None))
            }
            AnalysisBranch::Delta(changed) => {
                let completed = changed.update(repository)?;
                let (bank, delta) = completed.into_bank_with_changes();
                Ok(PropertyBankResolution::new(bank, Some(delta)))
            }
            AnalysisBranch::Corrupt(new) => {
                let completed = new.create(repository)?;
                Ok(PropertyBankResolution::new(completed.into_bank(), None))
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Comparison Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Identity phase: comparing file attributes with cached metadata.
#[derive(Debug)]
struct Comparison;

/// Proven: View does not exist in repository; carries file info.
#[derive(Debug)]
struct Missing;

/// Proven: View exists in repository; carries file info and cached view.
#[derive(Debug)]
struct Present {
    view: RawPropertyBankView,
}

impl Present {
    fn new(view: RawPropertyBankView) -> Self {
        Self {
            view,
        }
    }
}

/// Proven: binary identity has diverged; carries content for hashing/parsing.
#[derive(Debug)]
struct Suspect {
    view: RawPropertyBankView,
    content: String,
}

/// Proven: Content hash mismatch; file content retained for parsing.
/// Transitions to Analysis.
#[derive(Debug)]
struct Stale {
    content: String,
    content_hash: Blake3Hash,
    view: RawPropertyBankView,
}

/// Result of timestamp comparison in the Comparison stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
enum TimestampBranch {
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
    fn check_timestamps(
        self,
        source: &FsReader,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();
        let timestamps_match = status.view.is_timestamp_match(
            file.metadata().times().created_at(),
            file.metadata().times().modified_at(),
        );

        if timestamps_match {
            Ok(TimestampBranch::Match(Self::transition_from_parts(
                file, path_key, Fresh,
            )))
        } else {
            let content = source
                .read_to_string(file.path().as_path())
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

            Ok(TimestampBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Suspect {
                    view: status.view,
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
enum ContentBranch {
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
    fn check_content(self) -> ContentBranch {
        let (file, path_key, status) = self.into_parts();
        let content_hash = Blake3Hash::compute(status.content.as_bytes());

        let content_match = status.view.is_content_match(&content_hash);

        if content_match {
            ContentBranch::Match(Self::transition_from_parts(
                file,
                path_key,
                StaleTimestamps {
                    view: status.view,
                },
            ))
        } else {
            ContentBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Stale {
                    content: status.content,
                    content_hash,
                    view: status.view,
                },
            ))
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
struct Parsed;

/// Proven: Content has been parsed from Stale status.
/// Transitions to Analysis for property hash comparison.
#[derive(Debug)]
struct ParsedStale {
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
    fn parse(
        self,
        source: &FsReader,
    ) -> Result<PropertyBankProcessor<Construction, New>, SchemaLoaderError>
    {
        let (file, path_key, _status) = self.into_parts();
        let content = source
            .read_to_string(file.path().as_path())
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw: RawPropertyBank = FsReader::parse_structured_from_str(
            file.path().as_path(),
            &content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_metadata(file.metadata().clone());
        let content_hash = Blake3Hash::compute(content.as_bytes());

        Ok(Self::transition_from_parts(file, path_key, New {
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
    fn parse(
        self,
    ) -> Result<PropertyBankProcessor<Analysis, ParsedStale>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let raw: RawPropertyBank = FsReader::parse_structured_from_str(
            file.path().as_path(),
            &status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_metadata(file.metadata().clone());

        Ok(Self::transition_from_parts(file, path_key, ParsedStale {
            raw,
            content_hash: status.content_hash,
            view: status.view,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Analysis Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Semantic phase: computing the property delta between the file and view.
#[derive(Debug)]
struct Analysis;

/// Result of property delta analysis in the Analysis stage.
///
/// This enum fans out the next state for orchestration.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
enum AnalysisBranch {
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
    fn analyze(self) -> AnalysisBranch {
        let (file, path_key, status) = self.into_parts();
        let raw = status.raw;
        let content_hash = status.content_hash;
        let view = status.view;

        let Some(version) = view.current() else {
            return AnalysisBranch::Corrupt(Self::transition_from_parts(
                file,
                path_key,
                New {
                    raw,
                    content_hash,
                },
            ));
        };

        let Ok((delta, _property_hashes)) =
            PropertyDeltaEngine::for_property_bank(
                &raw,
                version.hashes().properties(),
            )
            .diff_property_bank()
        else {
            return AnalysisBranch::Corrupt(Self::transition_from_parts(
                file,
                path_key,
                New {
                    raw,
                    content_hash,
                },
            ));
        };
        if delta.is_empty() {
            AnalysisBranch::Empty(Self::transition_from_parts(
                file,
                path_key,
                StaleContent {
                    view,
                    content_hash,
                },
            ))
        } else {
            AnalysisBranch::Delta(Self::transition_from_parts(
                file,
                path_key,
                Changed {
                    raw,
                    delta,
                    content_hash,
                },
            ))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Refresh Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Maintenance phase: early commitment of proven metadata.
#[derive(Debug)]
struct Refresh;

/// Proven: content hashes match; only timestamps differ.
#[derive(Debug)]
struct StaleTimestamps {
    view: RawPropertyBankView,
}

/// Proven: property hashes match; content hash differs.
#[derive(Debug)]
struct StaleContent {
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
    fn sync_metadata<R: WriteRepository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        self.status.view.update_metadata(self.file.metadata().clone());

        repository
            .save_raw_property_bank_view(&self.path_key, &self.status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(self.transition(Construction, Fresh))
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
    fn sync_metadata<R: WriteRepository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        self.status.view.update_metadata(self.file.metadata().clone());
        self.status
            .view
            .update_content_hash(self.status.content_hash)
            .map_err(SchemaLoaderError::Repository)?;

        repository
            .save_raw_property_bank_view(&self.path_key, &self.status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(self.transition(Construction, Fresh))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Building phase: terminal domain construction.
#[derive(Debug)]
struct Construction;

/// Proven: initial ingestion path selected; carries raw bank and content hash.
#[derive(Debug)]
struct New {
    raw: RawPropertyBank,
    content_hash: Blake3Hash,
}

/// Proven: property divergence detected; carries raw bank, delta, and content
/// hash.
#[derive(Debug)]
struct Changed {
    raw: RawPropertyBank,
    delta: PropertyDelta,
    content_hash: Blake3Hash,
}

/// Proven: identity is fully synchronized; bank can be fetched without rebuild.
#[derive(Debug)]
struct Fresh;

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
    fn create<R: WriteRepository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, NewReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();

        let property_hashes = status.raw.properties().compute_hashes();
        let raw_hash =
            HashRecord::new(status.content_hash, property_hashes.into());
        let view = RawPropertyBankView::try_from_raw_with_hashes(
            &status.raw,
            path_key.clone(),
            raw_hash,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        let bank = PropertyBank::try_from(status.raw).map_err(|source| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                path: std::path::PathBuf::from("property_bank"),
                source,
            })
        })?;

        repository
            .save_property_bank(&bank)
            .map_err(SchemaLoaderError::Repository)?;

        repository
            .save_raw_property_bank_view(&path_key, &view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, NewReady {
            bank,
        }))
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
    fn update<R: Repository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, StaleReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let (raw, delta, content_hash) =
            (status.raw, status.delta, status.content_hash);

        let mut bank = repository
            .get_property_bank()
            .map_err(SchemaLoaderError::Repository)?
            .ok_or(SchemaLoaderError::Ingestion(
                SchemaIngestionError::Repository(
                    SchemaRepositoryError::PropertyBankNotFound,
                ),
            ))?;

        if !delta.is_empty() {
            let existing = bank.set_properties();
            let upserts = delta.upserts().clone().with_ids(existing);
            for (name, property) in upserts {
                existing.insert(name, property);
            }

            for name in delta.removals() {
                existing.remove(name);
            }

            *bank.set_recorded_at() = SystemTime::now();
        }

        let changed_names = delta.into_changed_name_set();

        let property_hashes = raw.properties().compute_hashes();
        let raw_hash = HashRecord::new(content_hash, property_hashes.into());
        let view = RawPropertyBankView::try_from_raw_with_hashes(
            &raw,
            path_key.clone(),
            raw_hash,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_property_bank(&bank)
            .map_err(SchemaLoaderError::Repository)?;

        repository
            .save_raw_property_bank_view(&path_key, &view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, StaleReady {
            bank,
            delta: changed_names,
        }))
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
    fn fetch<R: ReadRepository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed, FreshReady>, SchemaLoaderError>
    {
        let bank = repository
            .get_property_bank()
            .map_err(SchemaLoaderError::Repository)?
            .ok_or(SchemaIngestionError::Repository(
                SchemaRepositoryError::PropertyBankNotFound,
            ))?;

        Ok(self.transition(Completed, FreshReady {
            bank,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal phase: the `PropertyBank` is ready and owned.
#[derive(Debug)]
struct Completed;

/// Proven: terminal ingestion goal reached with fresh bank.
#[derive(Debug)]
struct FreshReady {
    bank: PropertyBank,
}

/// Proven: terminal ingestion goal reached with newly built bank.
#[derive(Debug)]
struct NewReady {
    bank: PropertyBank,
}

/// Proven: terminal ingestion goal reached with stale updates applied.
#[derive(Debug)]
struct StaleReady {
    bank: PropertyBank,
    delta: HashSet<PropertyName>,
}

/// Completed operations that expose the final property bank.
impl PropertyBankProcessor<Completed, FreshReady> {
    /// Extracts the completed `PropertyBank`.
    #[inline]
    #[must_use]
    fn into_bank(self) -> PropertyBank {
        self.status.bank
    }
}

impl PropertyBankProcessor<Completed, NewReady> {
    /// Extracts the completed `PropertyBank`.
    #[inline]
    #[must_use]
    fn into_bank(self) -> PropertyBank {
        self.status.bank
    }
}

impl PropertyBankProcessor<Completed, StaleReady> {
    /// Extracts the completed `PropertyBank` and changed property names.
    #[inline]
    #[must_use]
    fn into_bank_with_changes(self) -> (PropertyBank, HashSet<PropertyName>) {
        (self.status.bank, self.status.delta)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::{DirPath, FsFile},
        schema::storage::testing::InMemoryRepository,
    };

    mod run {
        use super::*;

        struct Fixture {
            repository: InMemoryRepository,
            source: FsReader,
            vault_root: DirPath,
            _vault_dir: TempDir,
            file: FsFile,
            key: PathKey,
            raw: RawPropertyBank,
            content_hash: Blake3Hash,
        }

        /// Helper to create a view with old timestamps (1 hour ago) for testing
        /// mismatch scenarios.
        fn make_stale_view(
            raw: &RawPropertyBank,
            key: PathKey,
            content_hash: Blake3Hash,
        ) -> RawPropertyBankView {
            use std::time::Duration;

            use crate::fs::metadata::{FileMetadata, FsTimes};

            let property_hashes = raw.properties().compute_hashes();
            let raw_hash =
                HashRecord::new(content_hash, property_hashes.into());

            // Create metadata with stale timestamps (1 hour ago)
            let old_time = SystemTime::now()
                .checked_sub(Duration::from_secs(3600))
                .expect("old time");
            let stale_times = FsTimes::new(Some(old_time), Some(old_time));
            let stale_metadata = FileMetadata::new(
                stale_times,
                raw.metadata().size(),
                raw.metadata().is_symlink(),
            );

            // Create a modified raw with stale metadata
            let stale_raw = raw.clone().with_metadata(stale_metadata);

            // Create view from the stale raw
            RawPropertyBankView::try_from_raw_with_hashes(
                &stale_raw, key, raw_hash,
            )
            .expect("view")
        }

        fn make_fixture() -> Fixture {
            let vault_dir = TempDir::new().expect("temp dir");
            let vault_root = DirPath::try_new(vault_dir.path().to_path_buf())
                .expect("vault root");
            let relative =
                std::path::PathBuf::from("schema/property-bank.json");
            let absolute = vault_dir.path().join(&relative);
            std::fs::create_dir_all(absolute.parent().expect("parent"))
                .expect("mkdir");
            let content = r#"{"$version":"1.0","properties":{"title":{"type":"string"}}}"#;
            std::fs::write(&absolute, content).expect("write file");

            let source = FsReader::new(vault_dir.path());
            let file_path = crate::fs::FilePath::try_new(absolute.clone())
                .expect("file path");
            let metadata = source
                .metadata(file_path.as_path())
                .expect("metadata")
                .as_file()
                .cloned()
                .expect("file metadata");
            let file = FsFile::new(file_path.clone(), metadata.clone());
            let key = file.path().as_key(&vault_root).expect("path key");
            let raw: RawPropertyBank = FsReader::parse_structured_from_str::<
                RawPropertyBank,
            >(
                file_path.as_path(), content
            )
            .expect("parse raw")
            .with_metadata(metadata);
            let content_hash = Blake3Hash::compute(content.as_bytes());

            Fixture {
                repository: InMemoryRepository::new(),
                source,
                vault_root,
                _vault_dir: vault_dir,
                file,
                key,
                raw,
                content_hash,
            }
        }

        #[test]
        fn run_missing_path_constructs_bank_with_title_property() {
            let fixture = make_fixture();
            let processor =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");
            let (bank, delta) = resolution.into_parts();

            assert!(
                bank.has(&"title".try_into().expect("property name")),
                "Expected title property in constructed bank"
            );
            assert!(delta.is_none(), "Missing path should not produce a delta");
        }

        #[test]
        fn run_fresh_path_returns_bank_without_delta_when_timestamps_match() {
            let fixture = make_fixture();

            let property_hashes = fixture.raw.properties().compute_hashes();
            let raw_hash =
                HashRecord::new(fixture.content_hash, property_hashes.into());
            let view = RawPropertyBankView::try_from_raw_with_hashes(
                &fixture.raw,
                fixture.key.clone(),
                raw_hash,
            )
            .expect("view");

            let seed_bank = PropertyBank::try_from(fixture.raw.clone())
                .expect("property bank");
            fixture
                .repository
                .save_property_bank(&seed_bank)
                .expect("seed bank");

            let processor =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");
            let (bank, delta) = resolution.into_parts();

            assert!(
                bank.has(&"title".try_into().expect("property name")),
                "Expected title property in fetched bank"
            );
            assert!(delta.is_none(), "Fresh path should not produce a delta");
        }

        #[test]
        fn run_content_match_path_syncs_and_returns_bank_without_delta() {
            let fixture = make_fixture();

            // Create view with stale timestamps but matching content hash
            let view = make_stale_view(
                &fixture.raw,
                fixture.key.clone(),
                fixture.content_hash,
            );

            let seed_bank = PropertyBank::try_from(fixture.raw.clone())
                .expect("property bank");
            fixture
                .repository
                .save_property_bank(&seed_bank)
                .expect("seed bank");

            let processor =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");
            let (bank, delta) = resolution.into_parts();

            assert!(
                bank.has(&"title".try_into().expect("property name")),
                "Expected title property in fetched bank"
            );
            assert!(
                delta.is_none(),
                "Content match path should not produce a delta"
            );
        }

        #[test]
        fn run_analysis_delta_path_returns_bank_with_delta() {
            let mut fixture = make_fixture();

            // Modify the file content to have a different property
            let modified_content = r#"{"$version":"1.0","properties":{"title":{"type":"number"}}}"#;
            std::fs::write(fixture.file.path().as_path(), modified_content)
                .expect("write modified file");

            // Create view with stale content (original property)
            let view = make_stale_view(
                &fixture.raw,
                fixture.key.clone(),
                fixture.content_hash,
            );

            let seed_bank = PropertyBank::try_from(fixture.raw.clone())
                .expect("property bank");
            fixture
                .repository
                .save_property_bank(&seed_bank)
                .expect("seed bank");

            // Reload file metadata after modification
            let modified_metadata = fixture
                .source
                .metadata(fixture.file.path().as_path())
                .expect("metadata")
                .as_file()
                .cloned()
                .expect("file metadata");
            fixture.file =
                FsFile::new(fixture.file.path().clone(), modified_metadata);

            let processor =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");
            let (bank, delta) = resolution.into_parts();

            assert!(
                bank.has(&"title".try_into().expect("property name")),
                "Expected title property in updated bank"
            );
            assert!(
                delta.is_some(),
                "Delta path should produce changed property set"
            );
            let changed_names = delta.expect("delta");
            let title_name: PropertyName =
                "title".try_into().expect("property name");
            assert!(
                changed_names.contains(&title_name),
                "Delta should include changed 'title' property"
            );
        }
    }

    mod constructor {
        use super::*;

        struct Fixture {
            repository: InMemoryRepository,
            source: FsReader,
            vault_root: DirPath,
            _vault_dir: TempDir,
            file: FsFile,
            key: PathKey,
            raw: RawPropertyBank,
            content_hash: Blake3Hash,
        }

        fn make_fixture() -> Fixture {
            let vault_dir = TempDir::new().expect("temp dir");
            let vault_root = DirPath::try_new(vault_dir.path().to_path_buf())
                .expect("vault root");
            let relative =
                std::path::PathBuf::from("schema/property-bank.json");
            let absolute = vault_dir.path().join(&relative);
            std::fs::create_dir_all(absolute.parent().expect("parent"))
                .expect("mkdir");
            let content = r#"{"$version":"1.0","properties":{"title":{"type":"string"}}}"#;
            std::fs::write(&absolute, content).expect("write file");

            let source = FsReader::new(vault_dir.path());
            let file_path = crate::fs::FilePath::try_new(absolute.clone())
                .expect("file path");
            let metadata = source
                .metadata(file_path.as_path())
                .expect("metadata")
                .as_file()
                .cloned()
                .expect("file metadata");
            let file = FsFile::new(file_path.clone(), metadata.clone());
            let key = file.path().as_key(&vault_root).expect("path key");
            let raw: RawPropertyBank = FsReader::parse_structured_from_str::<
                RawPropertyBank,
            >(
                file_path.as_path(), content
            )
            .expect("parse raw")
            .with_metadata(metadata);

            Fixture {
                repository: InMemoryRepository::new(),
                source,
                vault_root,
                _vault_dir: vault_dir,
                file,
                key,
                raw,
                content_hash: Blake3Hash::compute(content.as_bytes()),
            }
        }

        #[test]
        fn constructs_bank_with_title_property_when_new() {
            let fixture = make_fixture();
            let pipeline =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");
            let parsed = pipeline.transition(Parsed, Missing);
            let constructed = parsed.parse(&fixture.source).expect("parse");
            let completed =
                constructed.create(&fixture.repository).expect("create");
            let bank = completed.into_bank();

            assert!(
                bank.has(&"title".try_into().expect("property name")),
                "Expected title property in constructed bank"
            );
        }

        #[test]
        fn persists_view_with_rooted_path_key_when_constructing_new_bank() {
            let fixture = make_fixture();
            let key = fixture.key.clone();
            let pipeline =
                PropertyBankProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("pipeline");
            let parsed = pipeline.transition(Parsed, Missing);
            let constructed = parsed.parse(&fixture.source).expect("parse");
            let _completed =
                constructed.create(&fixture.repository).expect("create");

            let view = fixture
                .repository
                .get_raw_property_bank_view(&key)
                .expect("read view");
            assert!(view.is_some(), "Expected rooted path key to persist view");
        }

        #[test]
        fn update_persists_hashes_from_raw_property_bank_when_changed_content_hash_matches()
         {
            let fixture = make_fixture();
            let expected_hashes = HashRecord::new(
                fixture.content_hash,
                fixture.raw.properties().compute_hashes().into(),
            );

            let bank = PropertyBank::try_from(fixture.raw.clone())
                .expect("property bank");
            fixture.repository.save_property_bank(&bank).expect("seed bank");

            let changed = PropertyBankProcessor {
                file: fixture.file,
                path_key: fixture.key.clone(),
                status: Changed {
                    raw: fixture.raw,
                    delta: PropertyDelta::default(),
                    content_hash: fixture.content_hash,
                },
                _stage: PhantomData::<Construction>,
            };

            let _ = changed.update(&fixture.repository).expect("update");

            let view = fixture
                .repository
                .get_raw_property_bank_view(&fixture.key)
                .expect("read view")
                .expect("view exists");
            let actual_hashes =
                view.current().expect("current version").hashes();

            assert_eq!(
                actual_hashes, &expected_hashes,
                "Changed::persist should derive hashes from RawPropertyBank \
                 and content hash"
            );
        }

        #[test]
        fn create_and_update_persist_equivalent_hash_view_for_same_raw_property_bank()
         {
            let new_fixture = make_fixture();
            let changed_fixture = make_fixture();

            let new_processor = PropertyBankProcessor {
                file: new_fixture.file,
                path_key: new_fixture.key.clone(),
                status: New {
                    raw: new_fixture.raw.clone(),
                    content_hash: new_fixture.content_hash,
                },
                _stage: PhantomData::<Construction>,
            };
            let _ =
                new_processor.create(&new_fixture.repository).expect("create");

            let bank = PropertyBank::try_from(changed_fixture.raw.clone())
                .expect("property bank");
            changed_fixture
                .repository
                .save_property_bank(&bank)
                .expect("seed bank");

            let changed_processor = PropertyBankProcessor {
                file: changed_fixture.file,
                path_key: changed_fixture.key.clone(),
                status: Changed {
                    raw: changed_fixture.raw,
                    delta: PropertyDelta::default(),
                    content_hash: changed_fixture.content_hash,
                },
                _stage: PhantomData::<Construction>,
            };
            let _ = changed_processor
                .update(&changed_fixture.repository)
                .expect("update");

            let new_hashes = new_fixture
                .repository
                .get_raw_property_bank_view(&new_fixture.key)
                .expect("read new view")
                .expect("new view exists")
                .current()
                .expect("new current")
                .hashes()
                .clone();
            let changed_hashes = changed_fixture
                .repository
                .get_raw_property_bank_view(&changed_fixture.key)
                .expect("read changed view")
                .expect("changed view exists")
                .current()
                .expect("changed current")
                .hashes()
                .clone();

            assert_eq!(
                changed_hashes, new_hashes,
                "New and Changed persist paths should produce equivalent \
                 hashes"
            );
        }
    }
}
