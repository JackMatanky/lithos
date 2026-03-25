//! Typed processing pipeline for building or updating a `PropertyBank`.
//!
//! This module encodes the property bank load/update workflow as a typestate
//! pipeline. Each state is a compile-time step with the inputs required for the
//! next transition, so every branch is explicit and no state relies on runtime
//! `expect()`.
//!
//! ## What the pipeline does
//! - Reads file timestamps and cached view metadata.
//! - Decides whether to reuse the stored bank, update only view metadata, or
//!   parse and apply a property delta.
//! - Produces a ready `PropertyBank` and persists any updated view/bank data.
//!
//! ## Pipeline flow
//! ```text
//! Discovery
//!   ├─ New
//!   │   └─ IsNew -> NewConstruction -> Completed
//!   └─ FreshTimestamp
//!       └─ IsFreshTimestamp
//!           ├─ [timestamps match]
//!           │   └─ FetchConstruction -> Completed
//!           └─ [timestamps mismatch]
//!               └─ IsFreshContent
//!                   ├─ [content match]
//!                   │   └─ UpdateRawViewTime -> FetchConstruction -> Completed
//!                   └─ [content mismatch]
//!                       └─ IsStale
//!                           ├─ [delta empty]
//!                           │   └─ UpdateStaleRawView -> FetchConstruction -> Completed
//!                           └─ [delta non-empty]
//!                               └─ UpdateConstruction -> Completed
//! ```
//!
//! The terminal `Completed` state owns the finished `PropertyBank`.

use std::{collections::HashSet, time::SystemTime};

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
//  Processor Wrapper
// ─────────────────────────────────────────────────────────────────────────────

/// Core state machine tracking the current stage via compile-time types.
#[derive(Debug)]
#[must_use]
pub struct PropertyBankProcessor<S> {
    state: S,
}

impl<S> PropertyBankProcessor<S> {
    /// Internal constructor for state transitions.
    #[inline]
    fn transition<N>(state: N) -> PropertyBankProcessor<N> {
        PropertyBankProcessor {
            state,
        }
    }
}

impl Default for PropertyBankProcessor<Discovery> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Discovery
// ─────────────────────────────────────────────────────────────────────────────

/// Result of checking if a raw view exists.
#[derive(Debug)]
#[non_exhaustive]
pub enum DiscoveryBranch {
    /// No view exists in DB.
    New(PropertyBankProcessor<IsNew>),
    /// View exists, proceed to timestamp check.
    FreshTimestamp(PropertyBankProcessor<IsFreshTimestamp>),
}

/// Entry state for the property bank pipeline.
///
/// # Invariants
/// - No cached view has been loaded yet.
/// - File timestamps are not read until this state transitions.
#[derive(Debug)]
#[non_exhaustive]
pub struct Discovery;

impl PropertyBankProcessor<Discovery> {
    /// Start a new processor.
    #[inline]
    pub fn new() -> Self {
        PropertyBankProcessor {
            state: Discovery,
        }
    }

    /// Check if a raw view exists in the repository.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn has_raw_view<R: Repository>(
        self,
        filename: &str,
        source: &FsReader,
        config_path: &std::path::Path,
        repository: &R,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let cached_view = repository
            .get_raw_property_bank_view(filename)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let times = RawFileTimes {
            created_at: source.created_at(config_path),
            modified_at: source.modified_at(config_path),
        };

        if let Some(view) = cached_view {
            Ok(DiscoveryBranch::FreshTimestamp(Self::transition(
                IsFreshTimestamp {
                    times,
                    view,
                },
            )))
        } else {
            Ok(DiscoveryBranch::New(Self::transition(IsNew {
                times,
            })))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  IsNew
// ─────────────────────────────────────────────────────────────────────────────

/// Branch when no cached view exists.
///
/// # Invariants
/// - No cached view is available in the repository.
/// - File timestamps are captured and stored in `times`.
/// - The next step must parse content into a raw bank.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsNew {
    times: RawFileTimes,
}

impl PropertyBankProcessor<IsNew> {
    /// Parse the file content into a raw property bank.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn parse(
        self,
        config_path: &std::path::Path,
        content: &str,
    ) -> Result<PropertyBankProcessor<NewConstruction>, SchemaLoaderError> {
        let raw: RawPropertyBank =
            FsReader::parse_structured_from_str(config_path, content)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_file_times(self.state.times);

        Ok(Self::transition(NewConstruction {
            raw,
            content: content.into(),
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  IsFreshTimestamp
// ─────────────────────────────────────────────────────────────────────────────

/// Result of matching timestamps.
#[derive(Debug)]
#[non_exhaustive]
pub enum TimestampBranch<'source> {
    /// Timestamps match - fetch cached bank.
    Fetch(PropertyBankProcessor<FetchConstruction>),
    /// Timestamps mismatch - check content hash.
    Content(PropertyBankProcessor<IsFreshContent<'source>>),
}

/// Branch when a cached view exists and timestamps can be compared.
///
/// # Invariants
/// - A cached view is present in `view`.
/// - File timestamps are captured in `times`.
/// - Content has not been hashed yet.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsFreshTimestamp {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

impl PropertyBankProcessor<IsFreshTimestamp> {
    /// Match timestamps and branch to the next state.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn is_match(self, content: &str) -> TimestampBranch<'_> {
        let timestamps_match = self.state.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                self.state.times.created_at,
                self.state.times.modified_at,
            )
        });

        if timestamps_match {
            TimestampBranch::Fetch(self.to_fetch_construction())
        } else {
            TimestampBranch::Content(self.to_fresh_content(content))
        }
    }

    /// Transition to content check if timestamps mismatch.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn to_fresh_content(
        self,
        content: &str,
    ) -> PropertyBankProcessor<IsFreshContent<'_>> {
        Self::transition(IsFreshContent {
            times: self.state.times,
            view: self.state.view,
            content,
        })
    }

    /// Transition to fetch the cached bank from storage.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn to_fetch_construction(
        self,
    ) -> PropertyBankProcessor<FetchConstruction> {
        Self::transition(FetchConstruction)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  IsFreshContent
// ─────────────────────────────────────────────────────────────────────────────

/// Result of checking if content matches.
#[derive(Debug)]
#[non_exhaustive]
pub enum ContentBranch<'source> {
    /// Hash matches - just update timestamps.
    Match(PropertyBankProcessor<UpdateRawViewTime>),
    /// Hash mismatches - compute delta.
    Mismatch(PropertyBankProcessor<IsStale<'source>>),
}

/// State for content hashing when timestamps do not match.
///
/// # Invariants
/// - A cached view is present in `view`.
/// - File timestamps are captured in `times`.
/// - Full file content is available in `content` for hashing.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsFreshContent<'source> {
    times: RawFileTimes,
    view: RawPropertyBankView,
    content: &'source str,
}

impl<'source> PropertyBankProcessor<IsFreshContent<'source>> {
    /// Check if the content hash matches the cached view.
    ///
    /// If content mismatches, it transitions to `IsStale` and parses the file
    /// immediately to ensure the state carries valid raw data.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn is_match(
        self,
        config_path: &std::path::Path,
    ) -> Result<ContentBranch<'source>, SchemaLoaderError> {
        let content_hash = blake3::hash(self.state.content.as_bytes());
        let content_match = self.state.view.current().is_some_and(|v| {
            v.hashes().is_content_match(content_hash.as_bytes())
        });

        if content_match {
            Ok(ContentBranch::Match(self.into_update_view()))
        } else {
            let raw: RawPropertyBank = FsReader::parse_structured_from_str(
                config_path,
                self.state.content,
            )
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

            let raw = raw.with_file_times(self.state.times.clone());

            Ok(ContentBranch::Mismatch(self.into_stale(raw, content_hash)))
        }
    }

    #[inline]
    fn into_update_view(self) -> PropertyBankProcessor<UpdateRawViewTime> {
        Self::transition(UpdateRawViewTime {
            times: self.state.times,
            view: self.state.view,
        })
    }

    #[inline]
    fn into_stale(
        self,
        raw: RawPropertyBank,
        content_hash: blake3::Hash,
    ) -> PropertyBankProcessor<IsStale<'source>> {
        Self::transition(IsStale {
            raw,
            view: self.state.view,
            content: self.state.content,
            content_hash: *content_hash.as_bytes(),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  UpdateRawViewTime
// ─────────────────────────────────────────────────────────────────────────────

/// State for updating view timestamps after a content hash match.
///
/// # Invariants
/// - Content hash matches the cached view.
/// - The bank does not need to be rebuilt.
/// - Only view timestamps must be updated.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateRawViewTime {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

impl PropertyBankProcessor<UpdateRawViewTime> {
    /// Update timestamps in the cached view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn update<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<FetchConstruction>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let new_file_times = FileTimesMetadata::new(
            self.state.times.created_at,
            self.state.times.modified_at,
        );
        self.state.view.update_timestamps(new_file_times);

        repository
            .save_raw_property_bank_view(
                self.state.view.file_path().as_str(),
                &self.state.view,
            )
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(Self::transition(FetchConstruction))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  IsStale
// ─────────────────────────────────────────────────────────────────────────────

/// Result of filtering changed properties.
#[derive(Debug)]
#[non_exhaustive]
pub enum DeltaBranch {
    /// Content changed but properties did not.
    ContentOnly(PropertyBankProcessor<UpdateStaleRawView>),
    /// Properties changed - proceed with delta update.
    PropertiesChanged(PropertyBankProcessor<UpdateConstruction>),
}

/// Property delta between cached view and new raw bank.
#[derive(Debug)]
struct PropertyDelta {
    upserts: Vec<(PropertyName, RawPropertyBankEntry)>,
    removals: Vec<PropertyName>,
}

impl PropertyDelta {
    #[inline]
    fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.removals.is_empty()
    }
}

/// State for computing a property delta from parsed raw content.
///
/// # Invariants
/// - Content hash differs from the cached view.
/// - Raw content is parsed and stored in `raw`.
/// - `content_hash` corresponds to `content`.
/// - `raw` includes file times captured for this run.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsStale<'source> {
    raw: RawPropertyBank,
    view: RawPropertyBankView,
    content: &'source str,
    content_hash: [u8; 32],
}

impl PropertyBankProcessor<IsStale<'_>> {
    /// Filter changed properties and transition to the appropriate state.
    ///
    /// The file is already parsed in this state, so we only need to compare
    /// hashes with the cached view.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn filter_changed_properties(self) -> DeltaBranch {
        let delta = self.state.view.current().map_or_else(
            || self.delta_from_new_file(),
            |version| {
                self.delta_from_cached_view(version.hashes().properties())
            },
        );

        if delta.is_empty() {
            return DeltaBranch::ContentOnly(self.into_update_stale_view());
        }

        DeltaBranch::PropertiesChanged(self.into_update_construction(delta))
    }

    #[inline]
    fn delta_from_new_file(&self) -> PropertyDelta {
        let mut upserts = self
            .state
            .raw
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
        &self,
        prev_hashes: &std::collections::HashMap<PropertyName, [u8; 32]>,
    ) -> PropertyDelta {
        let mut upserts = Vec::new();
        let mut seen =
            HashSet::with_capacity(self.state.raw.properties().len());

        for (name, entry) in self.state.raw.properties().iter() {
            let new_hash = HashMetadata::hash_entry(entry);
            if prev_hashes.get(name) != Some(&new_hash) {
                upserts.push((name.clone(), entry.clone()));
            }
            seen.insert(name.clone());
        }

        let mut removals = prev_hashes
            .keys()
            .filter(|name| !seen.contains(*name))
            .cloned()
            .collect::<Vec<_>>();

        upserts.sort_by(|left, right| left.0.cmp(&right.0));
        removals.sort();

        PropertyDelta {
            upserts,
            removals,
        }
    }

    #[inline]
    fn into_update_stale_view(
        self,
    ) -> PropertyBankProcessor<UpdateStaleRawView> {
        Self::transition(UpdateStaleRawView {
            times: self.state.raw.file_times().clone(),
            content_hash: self.state.content_hash,
            view: self.state.view,
        })
    }

    #[inline]
    fn into_update_construction(
        self,
        delta: PropertyDelta,
    ) -> PropertyBankProcessor<UpdateConstruction> {
        Self::transition(UpdateConstruction {
            raw: self.state.raw,
            content: self.state.content.into(),
            delta,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  UpdateStaleRawView
// ─────────────────────────────────────────────────────────────────────────────

/// State for updating view hashes/timestamps when content changed but
/// properties did not.
///
/// # Invariants
/// - Content hash differs from the cached view.
/// - Property delta is empty (no property changes).
/// - Only view metadata must be updated.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateStaleRawView {
    times: RawFileTimes,
    content_hash: [u8; 32],
    view: RawPropertyBankView,
}

impl PropertyBankProcessor<UpdateStaleRawView> {
    /// Update timestamps and content hash in the cached view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails or the
    /// cached view cannot be reconstructed.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn update<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<FetchConstruction>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let new_file_times = FileTimesMetadata::new(
            self.state.times.created_at,
            self.state.times.modified_at,
        );
        self.state.view.update_timestamps(new_file_times);
        self.state
            .view
            .update_content_hash(self.state.content_hash)
            .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(
                self.state.view.file_path().as_str(),
                &self.state.view,
            )
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(Self::transition(FetchConstruction))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  NewConstruction
// ─────────────────────────────────────────────────────────────────────────────

/// State for building a new `PropertyBank` from raw content.
///
/// # Invariants
/// - `raw` is parsed from current content.
/// - `content` matches the raw bank.
/// - The bank does not exist in storage for this run.
#[derive(Debug)]
#[non_exhaustive]
pub struct NewConstruction {
    raw: RawPropertyBank,
    content: String,
}

impl PropertyBankProcessor<NewConstruction> {
    /// Build a new `PropertyBank`, persist it, then save the raw view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if construction or repository access
    /// fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn create<R: Repository>(
        self,
        filename: &str,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = self.build_bank()?;
        self.persist(filename, repository, &bank)?;

        Ok(Self::transition(Completed {
            bank,
        }))
    }

    #[inline]
    fn build_bank(&self) -> Result<PropertyBank, SchemaLoaderError> {
        let mut bank = PropertyBank::new();

        let mut entries: Vec<_> = self.state.raw.properties().iter().collect();
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
            &self.state.raw,
            filename,
            &self.state.content,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(filename, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  UpdateConstruction
// ─────────────────────────────────────────────────────────────────────────────

/// State for applying a property delta to an existing `PropertyBank`.
///
/// # Invariants
/// - `raw` is parsed from current content.
/// - `delta` captures all property upserts/removals.
/// - Existing property IDs must be preserved on updates.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateConstruction {
    raw: RawPropertyBank,
    content: String,
    delta: PropertyDelta,
}

impl PropertyBankProcessor<UpdateConstruction> {
    /// Update the cached `PropertyBank`, persist it, then save the raw view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if construction or repository access
    /// fails.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn update<R: Repository>(
        self,
        filename: &str,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let UpdateConstruction {
            raw,
            content,
            delta,
        } = self.state;
        let mut bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaLoaderError::Ingestion(
                SchemaIngestionError::Storage(
                    SchemaStorageError::PropertyBankNotFound,
                ),
            ))?;
        Self::apply_delta(delta, &mut bank)?;
        Self::persist(&raw, &content, filename, repository, &bank)?;

        Ok(Self::transition(Completed {
            bank,
        }))
    }

    #[inline]
    fn apply_delta(
        delta: PropertyDelta,
        bank: &mut PropertyBank,
    ) -> Result<(), SchemaLoaderError> {
        use std::collections::hash_map::Entry;

        let any_changed = !delta.is_empty();
        let PropertyDelta {
            upserts,
            removals,
        } = delta;

        for (name, entry) in upserts {
            let property = Property::try_from((name.clone(), entry)).map_err(
                |source| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                        path: std::path::PathBuf::from("property_bank"),
                        source,
                    })
                },
            )?;
            match bank.set_properties().entry(name) {
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

        for name in removals {
            bank.set_properties().remove(&name);
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
        raw: &RawPropertyBank,
        content: &str,
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
            raw, filename, content,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(filename, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  FetchConstruction
// ─────────────────────────────────────────────────────────────────────────────

/// State for fetching the cached `PropertyBank` without rebuilding.
///
/// # Invariants
/// - Cached view metadata is up to date for this run.
/// - The bank is expected to exist in storage.
#[derive(Debug)]
#[non_exhaustive]
pub struct FetchConstruction;

impl PropertyBankProcessor<FetchConstruction> {
    /// Fetch the cached `PropertyBank` from storage.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails or the bank
    /// is missing.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn fetch<R: Repository>(
        self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let bank = repository
            .get_property_bank()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or(SchemaIngestionError::Storage(
                SchemaStorageError::PropertyBankNotFound,
            ))?;

        Ok(Self::transition(Completed {
            bank,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal state containing the ready `PropertyBank`.
///
/// # Invariants
/// - The bank is fully constructed and owned by this state.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed {
    bank: PropertyBank,
}

impl PropertyBankProcessor<Completed> {
    /// Extract the completed `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn into_bank(self) -> PropertyBank {
        self.state.bank
    }
}

#[cfg(test)]
mod tests {
    mod fixtures {
        use std::{collections::HashMap, path::Path};

        use rstest::fixture;
        use tempfile::TempDir;

        use crate::schema::{
            raw::RawPropertyBank,
            testing::InMemoryRepository,
            views::{
                FileTimesMetadata, Filename, PropertyBankVersion,
                RawPropertyBankView, metadata::HashMetadata,
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
                serde_json::from_value(raw_json).expect("Invalid JSON");
            let version = PropertyBankVersion::new(times, hashes, &raw)
                .expect("Version error");
            let filename = Filename::new("properties.yaml".into());
            RawPropertyBankView::new(filename, version)
        }
    }

    mod has_raw_view {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::fixtures::{create_test_view, repo, temp_dir, write_config};
        use crate::{
            fs::FsReader,
            schema::{
                property_bank_processor::{
                    Discovery, DiscoveryBranch, PropertyBankProcessor,
                },
                storage::Repository as _,
                testing::InMemoryRepository,
            },
        };

        #[rstest]
        fn returns_new_when_no_view_exists(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(
                branch.is_ok(),
                "Discovery should succeed, found: {:?}",
                branch.err()
            );
            assert!(
                matches!(branch.unwrap(), DiscoveryBranch::New(_)),
                "Expected New branch"
            );
        }

        #[rstest]
        fn returns_fresh_timestamp_when_view_exists(
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
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(
                branch.is_ok(),
                "Discovery should succeed, found: {:?}",
                branch.err()
            );
            assert!(
                matches!(branch.unwrap(), DiscoveryBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch"
            );
        }
    }

    mod parse_errors {
        use rstest::rstest;

        use crate::schema::{
            error::{SchemaIngestionError, SchemaLoaderError},
            property_bank_processor::{IsNew, PropertyBankProcessor},
            raw::RawFileTimes,
        };

        #[rstest]
        fn parse_returns_error_on_invalid_yaml() {
            let processor = PropertyBankProcessor {
                state: IsNew {
                    times: RawFileTimes {
                        created_at: None,
                        modified_at: None,
                    },
                },
            };

            let result = processor
                .parse(std::path::Path::new("properties.yaml"), "invalid: [");

            assert!(matches!(
                result,
                Err(SchemaLoaderError::Ingestion(SchemaIngestionError::Parse(
                    _
                )))
            ));
        }
    }

    mod match_timestamp {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::fixtures::{create_test_view, repo, temp_dir, write_config};
        use crate::{
            fs::FsReader,
            schema::{
                property_bank_processor::{
                    ContentBranch, Discovery, DiscoveryBranch,
                    PropertyBankProcessor, TimestampBranch,
                },
                storage::Repository as _,
                testing::InMemoryRepository,
            },
        };

        #[rstest]
        fn returns_true_on_same_timestamps(
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
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(branch.is_ok(), "Expected success");
            let branch = branch.unwrap();
            assert!(
                matches!(branch, DiscoveryBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch, found: {branch:?}"
            );

            if let DiscoveryBranch::FreshTimestamp(p) = branch {
                let content_branch = p.is_match(content);
                assert!(
                    matches!(content_branch, TimestampBranch::Fetch(_)),
                    "Expected Fetch branch"
                );
            }
        }

        #[rstest]
        fn returns_false_on_different_timestamps(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            // Use a future time to ensure mismatch
            let future_time = std::time::SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .expect("Time error");
            let view =
                create_test_view(content, Some(future_time), Some(future_time));
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(branch.is_ok(), "Expected success");
            let discovery_branch = branch.unwrap();
            assert!(
                matches!(discovery_branch, DiscoveryBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch, found: {discovery_branch:?}"
            );

            if let DiscoveryBranch::FreshTimestamp(p) = discovery_branch {
                let timestamp_branch = p.is_match(content);
                assert!(
                    matches!(timestamp_branch, TimestampBranch::Content(_)),
                    "Expected Content branch"
                );
                if let TimestampBranch::Content(next) = timestamp_branch {
                    let content_branch = next.is_match(config_path);
                    assert!(
                        content_branch.is_ok(),
                        "Expected success, found: {:?}",
                        content_branch.err()
                    );
                    assert!(
                        matches!(
                            content_branch.unwrap(),
                            ContentBranch::Match(_)
                        ),
                        "Expected Match branch"
                    );
                }
            }
        }
    }

    mod content_hash {
        use std::{path::Path, time::SystemTime};

        use rstest::rstest;
        use tempfile::TempDir;

        use super::fixtures::{create_test_view, repo, temp_dir, write_config};
        use crate::{
            fs::FsReader,
            schema::{
                error::{
                    SchemaIngestionError, SchemaLoaderError, SchemaStorageError,
                },
                property_bank_processor::{
                    ContentBranch, Discovery, DiscoveryBranch,
                    FetchConstruction, PropertyBankProcessor, TimestampBranch,
                },
                storage::Repository as _,
                testing::InMemoryRepository,
            },
        };

        #[rstest]
        fn content_match_updates_view_timestamps(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let future_time = SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .expect("Time error");
            let view =
                create_test_view(content, Some(future_time), Some(future_time));
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(branch.is_ok(), "Expected success");
            let branch = branch.unwrap();
            assert!(matches!(branch, DiscoveryBranch::FreshTimestamp(_)));
            let DiscoveryBranch::FreshTimestamp(processor) = branch else {
                return;
            };

            let timestamp_branch = processor.is_match(content);
            assert!(matches!(timestamp_branch, TimestampBranch::Content(_)));
            let TimestampBranch::Content(next) = timestamp_branch else {
                return;
            };

            let content_branch = next.is_match(config_path);
            assert!(content_branch.is_ok(), "Expected success");
            assert!(matches!(content_branch, Ok(ContentBranch::Match(_))));
            let ContentBranch::Match(matcher) = content_branch.unwrap() else {
                return;
            };

            let _fetch = matcher.update(&repo).expect("Update should succeed");
            let updated_view = repo
                .get_raw_property_bank_view(filename)
                .expect("Fetch error")
                .expect("Expected view");

            let matches = updated_view.current().is_some_and(|v| {
                v.file_times().is_timestamp_match(
                    source.created_at(config_path),
                    source.modified_at(config_path),
                )
            });
            assert!(matches, "Expected timestamps to match current file");
        }

        #[rstest]
        fn content_mismatch_transitions_to_stale(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let old_content = "properties:\n  title:\n    type: string";
            let new_content = "properties:\n  title:\n    type: number";
            write_config(temp_dir.path(), filename, new_content);
            let config_path = Path::new(filename);

            let future_time = SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .expect("Time error");
            let view = create_test_view(
                old_content,
                Some(future_time),
                Some(future_time),
            );
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(branch.is_ok(), "Expected success");
            let branch = branch.unwrap();
            assert!(matches!(branch, DiscoveryBranch::FreshTimestamp(_)));
            let DiscoveryBranch::FreshTimestamp(processor) = branch else {
                return;
            };

            let timestamp_branch = processor.is_match(new_content);
            assert!(matches!(timestamp_branch, TimestampBranch::Content(_)));
            let TimestampBranch::Content(next) = timestamp_branch else {
                return;
            };

            let content_branch = next.is_match(config_path);
            assert!(content_branch.is_ok(), "Expected success");
            assert!(
                matches!(content_branch.unwrap(), ContentBranch::Mismatch(_)),
                "Expected Mismatch branch"
            );
        }

        #[rstest]
        fn fetch_returns_error_when_bank_missing(repo: InMemoryRepository) {
            let processor = PropertyBankProcessor {
                state: FetchConstruction,
            };

            let result = processor.fetch(&repo);

            assert!(matches!(
                result,
                Err(SchemaLoaderError::Ingestion(
                    SchemaIngestionError::Storage(
                        SchemaStorageError::PropertyBankNotFound
                    )
                ))
            ));
        }
    }

    mod integration {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::fixtures::{repo, temp_dir, write_config};
        use crate::{
            fs::FsReader,
            schema::{
                property_bank_processor::{
                    Discovery, DiscoveryBranch, PropertyBankProcessor,
                },
                testing::InMemoryRepository,
            },
        };

        #[rstest]
        #[expect(clippy::panic, reason = "Tests")]
        fn completes_new_path_from_discovery(
            temp_dir: TempDir,
            repo: InMemoryRepository,
        ) {
            let source = FsReader::new(temp_dir.path());
            let filename = "properties.yaml";
            let content = "properties:\n  title:\n    type: string";
            write_config(temp_dir.path(), filename, content);
            let config_path = Path::new(filename);

            let pipeline = PropertyBankProcessor::<Discovery>::new();
            let branch =
                pipeline.has_raw_view(filename, &source, config_path, &repo);

            assert!(branch.is_ok(), "Expected success");
            let branch = branch.unwrap();
            assert!(
                matches!(branch, DiscoveryBranch::New(_)),
                "Expected New branch, found: {branch:?}"
            );

            match branch {
                DiscoveryBranch::New(p) => {
                    let res = p.parse(config_path, content);
                    assert!(res.is_ok(), "Parse error: {:?}", res.err());
                    let completed = res.unwrap().create(filename, &repo);
                    assert!(
                        completed.is_ok(),
                        "Build error: {:?}",
                        completed.err()
                    );
                    let bank = completed.unwrap().into_bank();
                    assert_eq!(bank.all().count(), 1, "Expected 1 property");
                }
                DiscoveryBranch::FreshTimestamp(_) => {
                    panic!("Expected New branch");
                }
            }
        }
    }

    mod stale_view_update {
        use rstest::rstest;

        use super::fixtures::repo;
        use crate::schema::{
            property_bank_processor::{
                DeltaBranch, IsStale, PropertyBankProcessor,
            },
            raw::RawPropertyBank,
            storage::Repository as _,
            testing::InMemoryRepository,
            views::RawPropertyBankView,
        };

        #[rstest]
        fn updates_content_hash_when_delta_empty(repo: InMemoryRepository) {
            let filename = "properties.yaml";
            let old_content = "properties:\n  title:\n    type: string";
            let new_content = "properties:\n  title:\n    type: string\n";
            let old_raw: RawPropertyBank =
                serde_yaml::from_str(old_content).expect("Invalid YAML");
            let new_raw: RawPropertyBank =
                serde_yaml::from_str(new_content).expect("Invalid YAML");

            let view = RawPropertyBankView::try_from_raw_with_content(
                &old_raw,
                filename,
                old_content,
            )
            .expect("View error");
            repo.save_raw_property_bank_view(filename, &view)
                .expect("Save error");

            let content_hash = blake3::hash(new_content.as_bytes());
            let processor = PropertyBankProcessor {
                state: IsStale {
                    raw: new_raw,
                    view,
                    content: new_content,
                    content_hash: *content_hash.as_bytes(),
                },
            };

            let branch = processor.filter_changed_properties();
            assert!(matches!(branch, DeltaBranch::ContentOnly(_)));
            let DeltaBranch::ContentOnly(next) = branch else {
                return;
            };

            let _fetch = next.update(&repo).expect("Update should succeed");
            let updated_view = repo
                .get_raw_property_bank_view(filename)
                .expect("Fetch error")
                .expect("Expected view");
            let matches = updated_view.current().is_some_and(|v| {
                v.hashes().is_content_match(content_hash.as_bytes())
            });
            assert!(matches, "Expected content hash to be updated");
        }
    }

    mod update_construction {
        use rstest::rstest;

        use super::fixtures::repo;
        use crate::schema::{
            bank::PropertyBank,
            property::{Property, PropertyName},
            property_bank_processor::{
                DeltaBranch, IsStale, PropertyBankProcessor,
            },
            raw::RawPropertyBank,
            storage::Repository as _,
            testing::InMemoryRepository,
            views::RawPropertyBankView,
        };

        #[rstest]
        fn removes_properties_present_only_in_cached_view(
            repo: InMemoryRepository,
        ) {
            let filename = "properties.yaml";
            let old_content = "properties:\n  title:\n    type: string";
            let new_content = "properties: {}";

            let old_raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": {
                        "multi": false,
                        "type": "string"
                    }
                }
            });
            let new_raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            let old_raw: RawPropertyBank =
                serde_json::from_value(old_raw_json).unwrap();
            let new_raw: RawPropertyBank =
                serde_json::from_value(new_raw_json).unwrap();

            let view = RawPropertyBankView::try_from_raw_with_content(
                &old_raw,
                filename,
                old_content,
            )
            .unwrap();

            let content_hash = blake3::hash(new_content.as_bytes());
            let processor = PropertyBankProcessor {
                state: IsStale {
                    raw: new_raw,
                    view,
                    content: new_content,
                    content_hash: *content_hash.as_bytes(),
                },
            };

            let branch = processor.filter_changed_properties();

            assert!(
                matches!(branch, DeltaBranch::PropertiesChanged(_)),
                "Expected PropertiesChanged branch"
            );
            let DeltaBranch::PropertiesChanged(next) = branch else {
                return;
            };

            let mut bank = PropertyBank::new();
            let (existing_name, entry) =
                old_raw.properties().iter().next().unwrap();
            let property =
                Property::try_from((existing_name.clone(), entry.clone()))
                    .unwrap();
            bank.register(property).unwrap();
            repo.save_property_bank(&bank).unwrap();

            let completed = next.update(filename, &repo).unwrap();
            let updated_bank = completed.into_bank();
            let title_name = PropertyName::try_new("title").unwrap();
            assert!(
                updated_bank.get(&title_name).is_none(),
                "Expected removal"
            );
        }
    }
}
