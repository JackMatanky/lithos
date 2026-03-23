#![expect(clippy::missing_errors_doc, reason = "State machine transitions")]
#![expect(
    clippy::missing_inline_in_public_items,
    reason = "State machine methods"
)]
#![expect(
    clippy::missing_panics_doc,
    reason = "Typestate invariants enforced at compile time"
)]
#![expect(
    clippy::expect_used,
    reason = "Typestate invariants enforced at compile time"
)]

//! `PropertyBank` state machine for incremental loading and staleness
//! detection.
//!
//! This module implements a **compile-time state machine** using the typestate
//! pattern to orchestrate the `PropertyBank` loading pipeline. The state
//! machine enforces correct operation ordering at compile time and eliminates
//! the possibility of invalid state transitions.
//!
//! # Overview
//!
//! The `PropertyBank` pipeline handles four scenarios based on file staleness:
//!
//! 1. **NEW**: No cached view exists - parse file and build from scratch.
//! 2. **`FreshTimestamp`**: Timestamps match - fetch from DB, skip all I/O.
//! 3. **`FreshContent`**: Content hash matches but timestamps differ - update
//!    view timestamps only.
//! 4. **STALE**: Content changed - compute delta and apply incrementally.
//!
//! # State Machine States
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │                              Discovery                               │
//! │                   (Query DB for RawPropertyBankView)                 │
//! │             (Determine NEW/FreshTimestamp/FreshContent/STALE)        │
//! └───────┬─────────────────┬────────────────┬─────────────────┬─────────┘
//!         │                 │                │                 │
//!         ▼                 ▼                ▼                 ▼
//!     ┌───────┐     ┌──────────────┐  ┌────────────┐       ┌───────┐
//!     │  NEW  │     │FreshTimestamp│  │FreshContent│       │ STALE │
//!     └───┬───┘     └───────┬──────┘  └──────┬─────┘       └───┬───┘
//!         │                 │                │                 │
//!         ▼                 │                │                 ▼
//!   ┌───────────┐           │                │           ┌───────────┐
//!   │FileParsed │           │                │           │FileParsed │
//!   │  (+view)  │           │                │           │  (+view)  │
//!   └─────┬─────┘           │                │           └─────┬─────┘
//!         │                 │                │                 │
//!         │                 │                │                 ▼
//!         │                 │                │         ┌──────────────┐
//!         │                 │                │         │PropertyDelta │
//!         │                 │                │         └──────┬───────┘
//!         │                 │                │                │
//!         ▼                 ▼                ▼                ▼
//!  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  ┌──────────────┐
//!  │BaseConstructed││BaseConstructed││BaseConstructed│ │BaseConstructed│
//!  │(from scratch)│ │  (from DB)   │ │ (+upd times) │  │  (from DB)   │
//!  └──────┬───────┘ └───────┬──────┘ └───────┬──────┘  └───────┬──────┘
//!         │                 │                │                 │
//!         │                 │                │                 ▼
//!         │                 │                │           ┌────────────┐
//!         │                 │                │           │DeltaApplied│
//!         │                 │                │           │  (+view)   │
//!         │                 │                │           └─────┬──────┘
//!         │                 │                │                 │
//!         ▼                 ▼                ▼                ▼
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │                              Completed                               │
//! └──────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Branching Paths
//!
//! **Path 1: NEW** (no cached view)
//! - Discovery → `FileParsed` → `BaseConstructed` → `Completed`
//! - Reads and parses file, builds `PropertyBank` from scratch, persists both.
//!
//! **Path 2: `FreshTimestamp`** (fastest path)
//! - Discovery → `BaseConstructed` → `Completed`
//! - No file I/O, no parsing, fetches from DB, no persistence.
//!
//! **Path 3: `FreshContent`** (timestamp skew)
//! - Discovery → `BaseConstructed` → `Completed`
//! - Reads file for hash, no parsing, fetches from DB, persists view only.
//!
//! **Path 4: STALE** (content changed)
//! - Discovery → `FileParsed` → `PropertyDelta` → `BaseConstructed` →
//!   `DeltaApplied` → `Completed`
//! - Reads and parses file, computes delta, fetches from DB, applies updates,
//!   persists both.
//!
//! # Design Notes
//!
//! **Typestate Pattern**: Uses zero-sized types (ZSTs) to encode state at
//! compile time. Invalid transitions are type errors, not runtime errors.
//!
//! **Zero-Cost Abstraction**: State markers compile away completely - no
//! runtime overhead.
//!
//! **Two-Tier Staleness**: Minimizes I/O by checking timestamps first (metadata
//! only), then content hash (single file read) before full parsing.
//!
//! **Why Not Sealed State Markers?**: State markers are simple ZSTs without a
//! sealed trait. This is intentional for internal APIs. Sealing is only
//! necessary for public library APIs that need evolution flexibility (see Rust
//! API Guidelines C-SEALED).
//!
//! # Usage Example
//!
//! ```rust,ignore
//! // 1. Discover which path to take
//! let branch = PropertyBankPipeline::discover(&config_path, &source, &repository)?;
//!
//! // 2. Drive to completion (handles all internal transitions)
//! let completed = branch.into_completed(&repository)?;
//!
//! // 3. Extract the final PropertyBank
//! let bank = completed.into_bank();
//! ```

use std::marker::PhantomData;

use crate::{
    fs::FsReader,
    schema::{
        bank::PropertyBank,
        error::{
            SchemaFileError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError, SchemaStorageError,
        },
        property::PropertyName,
        raw::{RawFileTimes, RawPropertyBank},
        storage::Repository,
        views::{RawPropertyBankView, metadata::HashMetadata},
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Shared Data Payload
// ─────────────────────────────────────────────────────────────────────────────

/// Shared state payload carried through the pipeline.
///
/// Fields are optional because different branching paths skip different stages:
/// - **`FreshTimestamp`**: Only `bank` is populated (skips parsing entirely).
/// - **`FreshContent`**: Only `bank` is populated (skips parsing, updates view
///   externally).
/// - **NEW**: All fields except `delta` are populated.
/// - **STALE**: All fields are populated.
///
/// The state machine enforces that required fields are present at each state
/// via `expect()` calls, which are safe due to typestate invariants.
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct PropertyBankData {
    /// Raw parsed property bank (`FileParsed` state, NEW and STALE paths).
    pub raw: Option<RawPropertyBank>,

    /// Domain property bank (`BaseConstructed` state, all paths).
    pub bank: Option<PropertyBank>,

    /// Changed property names for incremental updates (`PropertyDelta` state,
    /// STALE path only).
    pub delta: Option<Vec<PropertyName>>,

    /// View for persistence (created at different times depending on path).
    /// NEW path: `FileParsed` state | STALE path: `DeltaApplied` state |
    /// `FreshContent` path: Discovery state.
    pub view: Option<RawPropertyBankView>,

    /// Filename for persistence (Discovery state).
    pub filename: Option<String>,

    /// File content needed for STALE path view creation (`FileParsed` state).
    pub content: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  State Machine Wrapper
// ─────────────────────────────────────────────────────────────────────────────

/// Core state machine tracking the current stage via compile-time types.
///
/// The generic parameter `S` is a zero-sized state marker that exists only at
/// compile time, enabling type-safe state transitions with zero runtime cost.
///
/// Each state has its own `impl PropertyBankPipeline<State>` block defining
/// valid operations. Invalid operations are prevented by the type system.
#[non_exhaustive]
#[derive(Debug)]
pub struct PropertyBankPipeline<S> {
    /// Carried data payload (boxed to keep state machine type small).
    data: Box<PropertyBankData>,

    /// Zero-sized state marker (no runtime representation).
    _state: PhantomData<S>,
}

impl<S> PropertyBankPipeline<S> {
    /// Internal constructor used during state transitions.
    #[must_use]
    fn new(data: PropertyBankData) -> Self {
        Self {
            data: Box::new(data),
            _state: PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Branching Enum
// ─────────────────────────────────────────────────────────────────────────────

/// Branching paths determined by the discovery phase.
///
/// Each variant contains the state machine at the appropriate starting state
/// for that path, along with any additional data needed for processing.
#[non_exhaustive]
#[derive(Debug)]
pub enum PropertyBankBranch {
    /// NEW path: No view exists in DB - parse file and build from scratch.
    /// Contains state machine at `FileParsed` state (file already parsed).
    New(PropertyBankPipeline<FileParsed>),

    /// `FreshTimestamp` path: Timestamps match - fetch from DB, skip
    /// persistence. Contains state machine at `BaseConstructed` state (bank
    /// will be fetched from DB).
    FreshTimestamp(PropertyBankPipeline<BaseConstructed>),

    /// `FreshContent` path: Content hash matches but timestamps differ - update
    /// view timestamps only. Contains: state machine at `BaseConstructed`
    /// state + updated view + filename.
    FreshContent(
        PropertyBankPipeline<BaseConstructed>,
        RawPropertyBankView,
        String,
    ),

    /// STALE path: Content changed - compute delta and apply incrementally.
    /// Contains: state machine at `FileParsed` state + cached view + filename.
    Stale(PropertyBankPipeline<FileParsed>, RawPropertyBankView, String),
}

impl PropertyBankBranch {
    /// Drive the path to completion, handling all branching logic internally.
    ///
    /// Abstracts over the four different paths and executes the appropriate
    /// state transitions for each:
    /// - **NEW**: `FileParsed` → `BaseConstructed` → `Completed` (persist both)
    /// - **`FreshTimestamp`**: `BaseConstructed` → `Completed` (no persistence)
    /// - **`FreshContent`**: `BaseConstructed` → `Completed` (persist view
    ///   only)
    /// - **STALE**: `FileParsed` → `PropertyDelta` → `BaseConstructed` →
    ///   `DeltaApplied` → `Completed` (persist both)
    pub fn into_completed<R: Repository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankPipeline<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        match self {
            PropertyBankBranch::New(pipeline) => pipeline
                .construct_base_from_scratch()?
                .into_completed_persist_all(repository),
            PropertyBankBranch::FreshTimestamp(pipeline) => pipeline
                .fetch_from_db(repository)
                .map(PropertyBankPipeline::into_completed_no_persistence),
            PropertyBankBranch::FreshContent(pipeline, view, filename) => {
                pipeline
                    .fetch_from_db(repository)?
                    .into_completed_persist_view_only(
                        repository, &filename, &view,
                    )
            }
            PropertyBankBranch::Stale(pipeline, cached_view, filename) => {
                pipeline
                    .compute_delta(&cached_view)
                    .fetch_from_db(repository)?
                    .apply_delta()?
                    .into_completed_persist_all(repository, &filename)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  State Transitions (one struct + impl block per state)
// ─────────────────────────────────────────────────────────────────────────────

/// Discovery state - entry point for staleness checking.
///
/// Queries the database for cached views and determines which of the four
/// branching paths to take (NEW/`FreshTimestamp`/`FreshContent`/STALE).
///
/// **Valid transitions**: Discovery → `FileParsed` (NEW/STALE) or Discovery →
/// `BaseConstructed` (`FreshTimestamp`/`FreshContent`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Discovery;

impl PropertyBankPipeline<Discovery> {
    /// Entry point: Discover staleness and return appropriate branch.
    ///
    /// Uses a two-tier check to minimize I/O:
    /// 1. Tier 1 (no file I/O): Check if cached view exists → NEW path if not.
    /// 2. Tier 2 (metadata only): Compare file timestamps → `FreshTimestamp` if
    ///    match.
    /// 3. Tier 3 (single file read): Compare content hash → `FreshContent` if
    ///    match, STALE otherwise.
    pub fn discover<R: Repository>(
        config_path: &std::path::Path,
        source: &FsReader,
        repository: &R,
    ) -> Result<PropertyBankBranch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let filename = config_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                SchemaIngestionError::File(SchemaFileError::InvalidFilename {
                    path: config_path.to_path_buf(),
                    reason: "missing filename".into(),
                })
            })?
            .to_owned();

        let cached_view = repository
            .get_raw_property_bank_view(&filename)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let Some(cached_view) = cached_view else {
            // Branch 1: NEW
            let content = source
                .read_to_string(config_path)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
            return Ok(PropertyBankBranch::New(Self::parse_file(
                source,
                config_path,
                &content,
                &filename,
                true, // create_view
            )?));
        };

        let created_at = source.created_at(config_path);
        let modified_at = source.modified_at(config_path);

        let is_timestamp_match = cached_view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(created_at, modified_at)
        });

        if is_timestamp_match {
            // Branch 2: FreshTimestamp
            let data = PropertyBankData::default();
            return Ok(PropertyBankBranch::FreshTimestamp(
                PropertyBankPipeline::new(data),
            ));
        }

        // Branches 3 & 4 require reading the file
        let content = source
            .read_to_string(config_path)
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let content_hash = blake3::hash(content.as_bytes());
        let is_content_match = cached_view.current().is_some_and(|v| {
            v.hashes().is_content_match(content_hash.as_bytes())
        });

        if is_content_match {
            // Branch 3: FreshContent
            let mut updated_view = cached_view.clone();
            let new_file_times =
                super::views::FileTimesMetadata::new(created_at, modified_at);
            updated_view.update_timestamps(new_file_times);
            let data = PropertyBankData::default();
            return Ok(PropertyBankBranch::FreshContent(
                PropertyBankPipeline::new(data),
                updated_view,
                filename,
            ));
        }

        // Branch 4: STALE
        Ok(PropertyBankBranch::Stale(
            Self::parse_file(
                source,
                config_path,
                &content,
                &filename,
                false, // don't create view yet (need delta first)
            )?,
            cached_view,
            filename,
        ))
    }

    /// Helper: Parse file and optionally create view (NEW path only).
    fn parse_file(
        source: &FsReader,
        config_path: &std::path::Path,
        content: &str,
        filename: &str,
        create_view: bool,
    ) -> Result<PropertyBankPipeline<FileParsed>, SchemaLoaderError> {
        let raw: RawPropertyBank =
            FsReader::parse_structured_from_str(config_path, content)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_file_times(RawFileTimes {
            created_at: source.created_at(config_path),
            modified_at: source.modified_at(config_path),
        });

        let view = if create_view {
            Some(
                RawPropertyBankView::try_from_raw_with_content(&raw, content)
                    .map_err(SchemaLoaderError::Ingestion)?,
            )
        } else {
            None
        };

        let data = PropertyBankData {
            raw: Some(raw),
            bank: None,
            delta: None,
            view,
            filename: Some(filename.to_owned()),
            content: Some(content.to_owned()),
        };

        Ok(PropertyBankPipeline::new(data))
    }
}

/// File parsed state - file has been read, parsed, and validated.
///
/// **Used by**: NEW path (first time) and STALE path (re-parsed after content
/// change).
///
/// **Valid transitions**: `FileParsed` → `BaseConstructed` (NEW path) or
/// `FileParsed` → `PropertyDelta` (STALE path).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct FileParsed;

impl PropertyBankPipeline<FileParsed> {
    /// Transition: NEW path - build from scratch.
    ///
    /// Converts the parsed `RawPropertyBank` into a domain-level `PropertyBank`
    /// by validating and transforming all properties.
    pub fn construct_base_from_scratch(
        mut self,
    ) -> Result<PropertyBankPipeline<BaseConstructed>, SchemaLoaderError> {
        let raw = self
            .data
            .raw
            .take()
            .expect("RawPropertyBank missing in FileParsed state");
        let bank = PropertyBank::try_from(raw).map_err(|source| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                path: std::path::PathBuf::from("property_bank"),
                source,
            })
        })?;

        self.data.bank = Some(bank);
        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Transition: STALE path - compute delta.
    ///
    /// Computes which properties changed by comparing per-property hashes
    /// between the cached view and the newly parsed file. Identifies
    /// new/modified/removed properties for incremental updates.
    #[must_use]
    pub fn compute_delta(
        mut self,
        cached_view: &RawPropertyBankView,
    ) -> PropertyBankPipeline<PropertyDelta> {
        let raw = self
            .data
            .raw
            .as_ref()
            .expect("RawPropertyBank missing in FileParsed state");

        let new_hashes =
            HashMetadata::compute_property_hashes(raw.properties());
        let changed = cached_view.current().map_or_else(
            || new_hashes.keys().cloned().collect(),
            |v| {
                v.hashes().changed_properties(&new_hashes).into_iter().collect()
            },
        );

        self.data.delta = Some(changed);

        PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        }
    }
}

/// Delta computed state - property differences have been calculated.
///
/// **Used by**: STALE path only (incremental update needed).
///
/// **Valid transitions**: `PropertyDelta` → `BaseConstructed` (fetch old bank
/// from DB before applying delta).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct PropertyDelta;

impl PropertyBankPipeline<PropertyDelta> {
    /// Transition: STALE path - fetch old bank from DB.
    pub fn fetch_from_db<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankPipeline<BaseConstructed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .unwrap_or_default();

        self.data.bank = Some(bank);
        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

/// Base constructed state - initial `PropertyBank` loaded or created.
///
/// This is the convergence point where all four branching paths meet.
///
/// **Valid transitions**: `BaseConstructed` → `DeltaApplied` (STALE path) or
/// `BaseConstructed` → `Completed` (all other paths).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct BaseConstructed;

impl PropertyBankPipeline<BaseConstructed> {
    /// Transition: FreshTimestamp/FreshContent paths - fetch bank from DB.
    ///
    /// Note: Self-returning type used for optional fetching scenarios in
    /// `FreshTimestamp`/`FreshContent` paths where bank may already be
    /// populated.
    pub fn fetch_from_db<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankPipeline<BaseConstructed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaIngestionError::Storage(
                SchemaStorageError::PropertyBankNotFound,
            ))?;

        self.data.bank = Some(bank);
        Ok(self)
    }

    /// Transition: STALE path - apply delta to old bank.
    ///
    /// Applies incremental updates by processing only the properties that
    /// changed. Updates existing properties, adds new properties, and
    /// removes deleted properties. Increments `BankVersion` if any changes
    /// were made.
    pub fn apply_delta(
        mut self,
    ) -> Result<PropertyBankPipeline<DeltaApplied>, SchemaLoaderError> {
        let mut bank = self
            .data
            .bank
            .take()
            .expect("Bank missing in BaseConstructed state");
        let raw = self
            .data
            .raw
            .as_ref()
            .expect("Raw missing in BaseConstructed state");
        let delta = self
            .data
            .delta
            .as_ref()
            .expect("Delta missing in BaseConstructed state");

        bank.update_from_raw(raw, delta).map_err(|source| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                path: std::path::PathBuf::from("property_bank"),
                source,
            })
        })?;

        self.data.bank = Some(bank);
        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Terminal: NEW path - persist bank and view.
    fn into_completed_persist_all<R: Repository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankPipeline<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = self.data.bank.as_ref().expect("Bank missing");
        let view = self.data.view.as_ref().expect("View missing");
        let filename = self.data.filename.as_ref().expect("Filename missing");

        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        repository
            .save_raw_property_bank_view(filename, view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }

    /// Terminal: `FreshTimestamp` path - skip persistence.
    #[must_use]
    fn into_completed_no_persistence(self) -> PropertyBankPipeline<Completed> {
        PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Terminal: `FreshContent` path - persist view only.
    fn into_completed_persist_view_only<R: Repository>(
        self,
        repository: &R,
        filename: &str,
        view: &RawPropertyBankView,
    ) -> Result<PropertyBankPipeline<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        repository
            .save_raw_property_bank_view(filename, view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

/// Delta applied state - `PropertyBank` updated with changes.
///
/// **Used by**: STALE path only (content changed, incremental update applied).
///
/// **Valid transitions**: `DeltaApplied` → `Completed` (persist both bank and
/// view).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct DeltaApplied;

impl PropertyBankPipeline<DeltaApplied> {
    /// Terminal: STALE path - persist bank and create/persist view.
    fn into_completed_persist_all<R: Repository>(
        self,
        repository: &R,
        filename: &str,
    ) -> Result<PropertyBankPipeline<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let raw = self.data.raw.as_ref().expect("Raw missing");
        let bank = self.data.bank.as_ref().expect("Bank missing");
        let content = self.data.content.as_ref().expect("Content missing");

        // Create view now (after delta applied)
        let new_view =
            RawPropertyBankView::try_from_raw_with_content(raw, content)
                .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_property_bank(bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        repository
            .save_raw_property_bank_view(filename, &new_view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

/// `Completed` state - pipeline is done.
///
/// Terminal state where the pipeline has finished all operations. The
/// `PropertyBank` is ready for extraction and persistence is complete (if
/// needed).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Completed;

impl PropertyBankPipeline<Completed> {
    /// Extract the completed `PropertyBank`.
    ///
    /// Consumes the state machine and returns the final `PropertyBank`. The
    /// returned bank is guaranteed to be fully constructed, updated with any
    /// incremental changes, and persisted to the database (except
    /// `FreshTimestamp` path).
    #[must_use]
    pub fn into_bank(self) -> PropertyBank {
        self.data.bank.expect("Bank missing in Completed state")
    }

    /// Borrow the completed `PropertyBank`.
    ///
    /// Returns a reference without consuming the state machine. Useful when
    /// you need to inspect the bank but still want access to the state machine
    /// afterward (e.g., for calling `into_data()`).
    #[must_use]
    pub fn bank(&self) -> &PropertyBank {
        self.data.bank.as_ref().expect("Bank missing in Completed state")
    }

    /// Extract the underlying data for advanced use cases.
    ///
    /// Consumes the state machine and returns the raw `PropertyBankData`
    /// payload. Primarily useful for testing, debugging, and advanced
    /// workflows that need access to intermediate data. Most users should
    /// use `into_bank()` instead.
    #[must_use]
    pub fn into_data(self) -> PropertyBankData {
        *self.data
    }
}

#[cfg(test)]
mod tests {
    mod fixtures {
        use std::{collections::HashMap, path::Path};

        use rstest::fixture;
        use tempfile::TempDir;

        use super::super::*;
        use crate::schema::{
            property::{Multiplicity, Optionality, Property, PropertyId},
            property_spec::{PropertySpec, StringSpec},
            testing::InMemoryRepository,
            views::{
                FileTimesMetadata, PropertyBankVersion, RawPropertyBankView,
                metadata::HashMetadata,
            },
        };

        #[fixture]
        pub fn repo() -> InMemoryRepository {
            InMemoryRepository::new()
        }

        #[fixture]
        pub fn temp_dir() -> TempDir {
            TempDir::new().expect("Failed to create temp dir")
        }

        pub fn write_config(root: &Path, name: &str, content: &str) {
            let path = root.join(name);
            std::fs::write(path, content).expect("Failed to write config file");
        }

        pub fn sample_bank() -> PropertyBank {
            let mut bank = PropertyBank::new();
            let prop = Property::new(
                PropertyId::new(),
                PropertyName::try_new("title").unwrap(),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::String(StringSpec::default()),
            );
            bank.register(prop).unwrap();
            bank
        }

        pub fn create_test_view(
            content: &str,
            created: Option<std::time::SystemTime>,
            modified: Option<std::time::SystemTime>,
        ) -> RawPropertyBankView {
            let times = FileTimesMetadata::new(created, modified);
            let hashes = HashMetadata::new(
                *blake3::hash(content.as_bytes()).as_bytes(),
                HashMap::new(),
            );
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            let raw: RawPropertyBank =
                serde_json::from_value(raw_json).unwrap();
            let version =
                PropertyBankVersion::new(times, hashes, &raw).unwrap();
            RawPropertyBankView::new(version)
        }
    }

    mod base_constructed {
        use rstest::rstest;

        use super::{fixtures::*, *};
        use crate::schema::{
            property::{
                Multiplicity, Optionality, Property, PropertyId, PropertyName,
            },
            property_spec::{PropertySpec, StringSpec},
            testing::InMemoryRepository,
        };

        #[test]
        fn should_apply_delta_to_bank() {
            let mut data = PropertyBankData::default();
            let bank = PropertyBank::new();
            let title_name = PropertyName::try_new("title").unwrap();

            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": { "multi": false, "type": "string" }
                }
            });
            data.raw =
                Some(serde_json::from_value(raw_json).expect("Invalid JSON"));
            data.delta = Some(vec![title_name.clone()]);
            data.bank = Some(bank);

            let pipeline = PropertyBankPipeline::<BaseConstructed>::new(data);

            let next = pipeline.apply_delta().expect("Apply delta failed");

            let updated_bank = next.data.bank.as_ref().expect("Bank missing");
            assert!(
                updated_bank.has(&title_name),
                "Bank should have title property"
            );
            assert_eq!(
                updated_bank.version().as_u64(),
                1,
                "Version should be 1"
            );
        }

        #[test]
        fn should_handle_property_removal_in_apply_delta() {
            let mut data = PropertyBankData::default();
            let mut bank = PropertyBank::new();
            let title_name = PropertyName::try_new("title").unwrap();

            // Register it first
            let prop = Property::new(
                PropertyId::new(),
                title_name.clone(),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::String(StringSpec::default()),
            );
            bank.register(prop).unwrap();
            assert_eq!(bank.version().as_u64(), 1);

            // Raw has NO properties
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            data.raw =
                Some(serde_json::from_value(raw_json).expect("Invalid JSON"));
            data.delta = Some(vec![title_name.clone()]);
            data.bank = Some(bank);

            let pipeline = PropertyBankPipeline::<BaseConstructed>::new(data);
            let next = pipeline.apply_delta().unwrap();

            let updated_bank = next.data.bank.as_ref().unwrap();
            assert!(
                !updated_bank.has(&title_name),
                "Property should be removed"
            );
            assert_eq!(
                updated_bank.version().as_u64(),
                2,
                "Version should increment on removal"
            );
        }

        #[rstest]
        fn should_persist_all_for_new_path(repo: InMemoryRepository) {
            let mut data = PropertyBankData::default();
            let bank = sample_bank();
            data.bank = Some(bank);

            let view = create_test_view("", None, None);
            data.view = Some(view);
            data.filename = Some("props.yaml".to_owned());
            let pipeline = PropertyBankPipeline::<BaseConstructed>::new(data);

            pipeline
                .into_completed_persist_all(&repo)
                .expect("Persist all failed");

            assert!(
                repo.get_property_bank().expect("Read failed").is_some(),
                "Bank should be persisted"
            );
            assert!(
                repo.get_raw_property_bank_view("props.yaml")
                    .expect("Read failed")
                    .is_some(),
                "View should be persisted"
            );
        }

        #[rstest]
        fn should_fail_if_bank_missing_for_fresh_paths(
            repo: InMemoryRepository,
        ) {
            // No bank in repo
            let pipeline = PropertyBankPipeline::<BaseConstructed>::new(
                PropertyBankData::default(),
            );

            let res = pipeline.fetch_from_db(&repo);

            res.expect_err("Expected error when fetching missing bank");
        }
    }

    mod discovery {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::{fixtures::*, *};
        use crate::schema::testing::InMemoryRepository;

        #[rstest]
        fn should_return_new_when_no_view_exists(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let branch =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery should succeed");

            assert!(
                matches!(branch, PropertyBankBranch::New(_)),
                "Expected New branch, found {branch:?}"
            );
        }

        #[rstest]
        fn should_return_fresh_timestamp_when_timestamps_match(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let view = create_test_view(
                content,
                source.created_at(config_path),
                source.modified_at(config_path),
            );
            repo.save_raw_property_bank_view(filename, &view).unwrap();

            let branch =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery should succeed");

            assert!(
                matches!(branch, PropertyBankBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch, found {branch:?}"
            );
        }

        #[rstest]
        fn should_return_fresh_content_when_content_matches_but_timestamps_differ(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            // Use a future time to ensure mismatch (using checked_add for
            // safety)
            let future_time = std::time::SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .expect("Time overflow");
            let view =
                create_test_view(content, Some(future_time), Some(future_time));
            repo.save_raw_property_bank_view(filename, &view).unwrap();

            let branch =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery should succeed");

            assert!(
                matches!(branch, PropertyBankBranch::FreshContent(_, _, _)),
                "Expected FreshContent branch, got {branch:?}"
            );

            if let PropertyBankBranch::FreshContent(_, updated_view, _) = branch
            {
                assert!(
                    updated_view
                        .current()
                        .expect("Missing current version in view")
                        .file_times()
                        .is_timestamp_match(
                            source.created_at(config_path),
                            source.modified_at(config_path)
                        ),
                    "Updated view should match current file times"
                );
            }
        }

        #[rstest]
        fn should_return_stale_when_content_differs(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let old_content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, old_content);
            let config_path = Path::new(filename);

            // Create and save a view with old content hash
            let view = create_test_view(old_content, None, None);
            repo.save_raw_property_bank_view(filename, &view).unwrap();

            let new_content = "properties:\n  title:\n    type: string\n  \
                               description:\n    type: string";
            write_config(temp_dir.path(), filename, new_content);

            let branch =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery should succeed");

            assert!(
                matches!(branch, PropertyBankBranch::Stale(_, _, _)),
                "Expected Stale branch, found {branch:?}"
            );
        }

        #[rstest]
        fn should_fail_when_filename_invalid(repo: InMemoryRepository) {
            let source = FsReader::new("/");
            // An empty path has no filename
            let config_path = Path::new("");

            let res =
                PropertyBankPipeline::discover(config_path, &source, &repo);

            res.expect_err("Expected error for missing filename");
        }

        #[rstest]
        fn should_fail_when_config_file_missing(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            // Path that definitely doesn't exist
            let config_path = Path::new("absolutely_missing.yaml");

            let res =
                PropertyBankPipeline::discover(config_path, &source, &repo);

            res.expect_err("Expected error for missing file");
        }

        #[rstest]
        fn should_fail_when_config_is_malformed(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "bad.yaml";
            // YAML syntax error: unclosed quote
            let content = "properties:\n  title:\n    type: \"unclosed";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let res =
                PropertyBankPipeline::discover(config_path, &source, &repo);

            res.expect_err("Expected error for malformed YAML");
        }
    }

    mod file_parsed {
        use super::{fixtures::*, *};

        #[test]
        fn should_construct_base_from_scratch() {
            let mut data = PropertyBankData::default();
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": { "multi": false, "type": "string" }
                }
            });
            data.raw =
                Some(serde_json::from_value(raw_json).expect("Invalid JSON"));
            let pipeline = PropertyBankPipeline::<FileParsed>::new(data);

            let next = pipeline
                .construct_base_from_scratch()
                .expect("Construction should succeed");

            assert!(
                next.data.bank.is_some(),
                "Bank should be populated in next state"
            );
            assert_eq!(
                next.data.bank.as_ref().expect("Bank missing").all().count(),
                1,
                "Bank should have 1 property"
            );
        }

        #[test]
        fn should_compute_delta_with_changes() {
            let mut data = PropertyBankData::default();
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": { "multi": false, "type": "string" }
                }
            });
            let raw: RawPropertyBank =
                serde_json::from_value(raw_json).expect("Invalid JSON");
            data.raw = Some(raw);
            let pipeline = PropertyBankPipeline::<FileParsed>::new(data);

            // Empty view means everything is new
            let view = create_test_view("", None, None);

            let next = pipeline.compute_delta(&view);

            let delta = next.data.delta.as_ref().expect("Delta missing");
            assert_eq!(delta.len(), 1);
            assert_eq!(
                delta.first().expect("Missing element").as_str(),
                "title"
            );
        }

        #[test]
        fn should_detect_property_removal_in_delta() {
            let mut data = PropertyBankData::default();
            // New raw has NO properties
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            data.raw =
                Some(serde_json::from_value(raw_json).expect("Invalid JSON"));
            let pipeline = PropertyBankPipeline::<FileParsed>::new(data);

            // Cached view HAS "title"
            let view_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": { "multi": false, "type": "string" }
                }
            });
            let view_raw: RawPropertyBank =
                serde_json::from_value(view_json).unwrap();
            let view =
                RawPropertyBankView::try_from_raw_with_content(&view_raw, "")
                    .unwrap();

            let next = pipeline.compute_delta(&view);

            let delta = next.data.delta.as_ref().expect("Delta missing");
            assert_eq!(delta.len(), 1, "Should detect 1 change (removal)");
            assert_eq!(delta.first().unwrap().as_str(), "title");
        }
    }

    mod integration {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::{fixtures::*, *};
        use crate::schema::{
            property::PropertyName, testing::InMemoryRepository,
        };

        #[rstest]
        fn should_drive_new_path_to_completion(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let branch =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery failed");
            let completed =
                branch.into_completed(&repo).expect("Should complete NEW path");
            let bank = completed.into_bank();

            assert_eq!(bank.all().count(), 1);
            assert!(bank.has(&PropertyName::try_new("title").unwrap()));
            assert!(repo.get_property_bank().unwrap().is_some());
        }

        #[rstest]
        fn should_drive_fresh_timestamp_path_to_completion(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            // 1. Setup initial state
            let branch_init =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .expect("Discovery failed");
            branch_init.into_completed(&repo).expect("Setup failed");

            // 2. Discover again (should be fresh timestamp)
            let branch_fresh =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap();
            assert!(
                matches!(branch_fresh, PropertyBankBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch"
            );

            let completed = branch_fresh
                .into_completed(&repo)
                .expect("Should complete FreshTimestamp path");
            assert_eq!(completed.bank().all().count(), 1);
        }

        #[rstest]
        fn should_drive_fresh_content_path_to_completion(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            // 1. Setup initial state
            let branch_init =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap();
            branch_init.into_completed(&repo).unwrap();

            // Capture initial created_at
            let initial_created = source.created_at(config_path);

            // 2. Modify timestamps but not content
            let future_time = std::time::SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .expect("Time overflow");

            let file = std::fs::File::options()
                .write(true)
                .open(temp_dir.path().join(filename))
                .expect("Failed to open file");
            file.set_times(
                std::fs::FileTimes::new()
                    .set_modified(future_time)
                    .set_accessed(future_time),
            )
            .expect("Failed to set file times");
            drop(file);

            // 3. Discover again (should be fresh content due to timestamp
            //    mismatch)
            let branch_fresh =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap();
            assert!(
                matches!(&branch_fresh, PropertyBankBranch::FreshContent(..)),
                "Expected FreshContent branch, got {branch_fresh:?}"
            );

            let completed = branch_fresh
                .into_completed(&repo)
                .expect("Should complete FreshContent path");
            assert_eq!(completed.bank().all().count(), 1);

            // Verify view was updated in repo
            let view =
                repo.get_raw_property_bank_view(filename).unwrap().unwrap();
            let current = view.current().unwrap();
            assert!(
                current
                    .file_times()
                    .is_timestamp_match(initial_created, Some(future_time)),
                "View should have updated modified_at and original \
                 created_at. Expected: ({initial_created:?}, \
                 {future_time:?}), Found: ({:?}, {:?})",
                current.file_times().created_at(),
                current.file_times().modified_at()
            );
        }

        #[rstest]
        fn should_drive_stale_path_to_completion(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let old_content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, old_content);
            let config_path = Path::new(filename);

            let branch_init =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap();
            branch_init.into_completed(&repo).unwrap();

            let new_content = "properties:\n  title:\n    type: string\n  \
                               description:\n    type: string";
            write_config(temp_dir.path(), filename, new_content);

            let branch_stale =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap();
            assert!(
                matches!(branch_stale, PropertyBankBranch::Stale(_, _, _)),
                "Expected Stale branch"
            );

            let completed = branch_stale
                .into_completed(&repo)
                .expect("Should complete Stale path");
            let bank = completed.into_bank();

            assert_eq!(bank.all().count(), 2, "Bank should have 2 properties");
            assert!(
                bank.has(&PropertyName::try_new("description").unwrap()),
                "Bank should have description property"
            );
            assert_eq!(
                bank.version().as_u64(),
                2,
                "Version should increment to 2"
            );
        }

        #[rstest]
        fn should_handle_sequential_stale_updates(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let config_path = Path::new(filename);

            // v1: title
            write_config(
                temp_dir.path(),
                filename,
                "properties:\n  title: { type: string }",
            );
            PropertyBankPipeline::discover(config_path, &source, &repo)
                .unwrap()
                .into_completed(&repo)
                .unwrap();

            // v2: title, desc
            write_config(
                temp_dir.path(),
                filename,
                "properties:\n  title: { type: string }\n  desc: { type: \
                 string }",
            );
            PropertyBankPipeline::discover(config_path, &source, &repo)
                .unwrap()
                .into_completed(&repo)
                .unwrap();

            // v3: desc (title removed)
            write_config(
                temp_dir.path(),
                filename,
                "properties:\n  desc: { type: string }",
            );
            let completed =
                PropertyBankPipeline::discover(config_path, &source, &repo)
                    .unwrap()
                    .into_completed(&repo)
                    .unwrap();

            let bank = completed.into_bank();
            assert_eq!(bank.all().count(), 1, "Should have 1 property");
            assert!(bank.has(&PropertyName::try_new("desc").unwrap()));
            assert!(!bank.has(&PropertyName::try_new("title").unwrap()));
            assert_eq!(bank.version().as_u64(), 3, "Version should be 3");
        }
    }

    mod property_delta {
        use rstest::rstest;

        use super::{fixtures::*, *};
        use crate::schema::testing::InMemoryRepository;

        #[rstest]
        fn should_fetch_bank_from_db(repo: InMemoryRepository) {
            let bank = sample_bank();
            repo.save_property_bank(&bank).expect("Persist failed");
            let pipeline = PropertyBankPipeline::<PropertyDelta>::new(
                PropertyBankData::default(),
            );

            let next = pipeline
                .fetch_from_db(&repo)
                .expect("Fetching from DB should succeed");

            assert_eq!(
                next.data.bank.as_ref().expect("Bank missing").version(),
                bank.version(),
                "Fetched bank version should match"
            );
        }

        #[rstest]
        fn should_use_default_bank_if_missing_during_stale_path(
            repo: InMemoryRepository,
        ) {
            // No bank in repo
            let pipeline = PropertyBankPipeline::<PropertyDelta>::new(
                PropertyBankData::default(),
            );

            let next = pipeline.fetch_from_db(&repo).expect("Fetch failed");

            assert_eq!(
                next.data
                    .bank
                    .as_ref()
                    .expect("Bank missing")
                    .version()
                    .as_u64(),
                0,
                "Should use default (v0) bank if missing"
            );
        }
    }

    use super::*;
}
