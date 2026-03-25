//! `PropertyBank` state machine for incremental loading and staleness
//! detection.
//!
//! This module implements a **strong typestate pattern** to orchestrate the
//! `PropertyBank` loading pipeline. Unlike the previous implementation, this
//! version uses distinct structs for each state, ensuring that required data
//! is present at compile time and eliminating the need for runtime `expect()`
//! calls.
//!
//! # States and Transitions
//!
//! 1. **Discovery**: Entry point. Checks for existence of a cached view.
//!     - `to_new` -> `IsNew`
//!     - `to_fresh_timestamp` -> `IsFreshTimestamp`
//!
//! 2. **IsNew**: Handles completely new files.
//!     - `parse` -> `NewConstruction`
//!
//! 3. **IsFreshTimestamp**: Tier 2 check (metadata matching).
//!     - `to_fetch_construction` -> `FetchConstruction` (Fastest path)
//!     - `to_fresh_content` -> `IsFreshContent`
//!
//! 4. **IsFreshContent**: Tier 3 check (hash matching).
//!     - `to_update_raw_view_time` -> `UpdateRawViewTime`
//!     - `to_stale` -> `IsStale`
//!
//! 5. **UpdateRawViewTime**: Syncs metadata when content matches but timestamps
//!    differ.
//!     - `update` -> `FetchConstruction`
//!
//! 6. **UpdateStaleRawView**: Content changed but properties did not.
//!     - `update` -> `FetchConstruction`
//!
//! 7. **IsStale**: Content changed.
//!     - `filter_changed_properties` -> `UpdateStaleRawView` or
//!       `UpdateConstruction`
//!
//! 8. **NewConstruction**: Builds a fresh `PropertyBank` and persists view.
//!     - `create` -> `Completed`
//!
//! 9. **UpdateConstruction**: Applies delta to `PropertyBank` and persists
//!    view.
//!     - `update` -> `Completed`
//!
//! 10. **FetchConstruction**: Fetches cached `PropertyBank`.
//!     - `fetch` -> `Completed`
//!
//! 11. **Completed**: Terminal state.

use std::time::SystemTime;

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
//  State Definitions
// ─────────────────────────────────────────────────────────────────────────────

/// Initial state - preparing to check for cached views.
#[derive(Debug)]
#[non_exhaustive]
pub struct Discovery;

/// No cached view exists - must perform full ingestion.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsNew {
    times: RawFileTimes,
}

/// Cached view exists - checking if timestamps match.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsFreshTimestamp {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

/// Timestamps mismatched - checking if content hash matches.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsFreshContent<'source> {
    times: RawFileTimes,
    view: RawPropertyBankView,
    content: &'source str,
}

/// Content hash matched but timestamps differ - update view only.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateRawViewTime {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

/// Content changed but properties did not - update content hash.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateStaleRawView {
    times: RawFileTimes,
    content_hash: [u8; 32],
    view: RawPropertyBankView,
}

/// Content changed - must compute delta and update incrementally.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsStale<'source> {
    raw: RawPropertyBank,
    view: RawPropertyBankView,
    content: &'source str,
    content_hash: [u8; 32],
}

/// Ready to build a new `PropertyBank` from scratch.
#[derive(Debug)]
#[non_exhaustive]
pub struct NewConstruction {
    raw: RawPropertyBank,
    content: String,
}

/// Ready to update an existing `PropertyBank` with property delta.
#[derive(Debug)]
#[non_exhaustive]
pub struct UpdateConstruction {
    raw: RawPropertyBank,
    content: String,
    delta: PropertyDelta,
}

/// Ready to fetch the cached `PropertyBank` from storage.
#[derive(Debug)]
#[non_exhaustive]
pub struct FetchConstruction;

/// Terminal state - `PropertyBank` is ready.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed {
    bank: PropertyBank,
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

// ─────────────────────────────────────────────────────────────────────────────
//  Branching Enums
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

/// Result of checking if content matches.
#[derive(Debug)]
#[non_exhaustive]
pub enum ContentBranch<'source> {
    /// Hash matches - just update timestamps.
    Match(PropertyBankProcessor<UpdateRawViewTime>),
    /// Hash mismatches - compute delta.
    Mismatch(PropertyBankProcessor<IsStale<'source>>),
}

/// Result of matching timestamps.
#[derive(Debug)]
#[non_exhaustive]
pub enum TimestampBranch<'source> {
    /// Timestamps match - fetch cached bank.
    Fetch(PropertyBankProcessor<FetchConstruction>),
    /// Timestamps mismatch - check content hash.
    Content(PropertyBankProcessor<IsFreshContent<'source>>),
}

/// Result of filtering changed properties.
#[derive(Debug)]
#[non_exhaustive]
pub enum DeltaBranch {
    /// Content changed but properties did not.
    ContentOnly(PropertyBankProcessor<UpdateStaleRawView>),
    /// Properties changed - proceed with delta update.
    PropertiesChanged(PropertyBankProcessor<UpdateConstruction>),
}

// ─────────────────────────────────────────────────────────────────────────────
//  Transitions: Discovery
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: IsFreshTimestamp
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: IsFreshContent
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: UpdateRawViewTime
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: UpdateStaleRawView
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: IsNew
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: IsStale
// ─────────────────────────────────────────────────────────────────────────────

impl PropertyBankProcessor<IsStale<'_>> {
    /// Filter changed properties and transition to the appropriate state.
    ///
    /// The file is already parsed in this state, so we only need to compare
    /// hashes with the cached view.
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    pub fn filter_changed_properties(self) -> DeltaBranch {
        let new_hashes =
            HashMetadata::compute_property_hashes(self.state.raw.properties());
        let (mut upserts, mut removals) =
            self.state.view.current().map_or_else(
                || {
                    let upserts = self
                        .state
                        .raw
                        .properties()
                        .iter()
                        .map(|(name, entry)| (name.clone(), entry.clone()))
                        .collect::<Vec<_>>();
                    (upserts, Vec::new())
                },
                |version| {
                    let prev_hashes = version.hashes().properties();
                    let mut upserts = Vec::new();

                    for (name, entry) in self.state.raw.properties().iter() {
                        let Some(new_hash) = new_hashes.get(name) else {
                            continue;
                        };
                        if prev_hashes.get(name) != Some(new_hash) {
                            upserts.push((name.clone(), entry.clone()));
                        }
                    }
                    let removals = prev_hashes
                        .keys()
                        .filter(|name| !new_hashes.contains_key(*name))
                        .cloned()
                        .collect::<Vec<_>>();

                    (upserts, removals)
                },
            );

        if upserts.is_empty() && removals.is_empty() {
            return DeltaBranch::ContentOnly(self.into_update_stale_view());
        }

        upserts.sort_by(|left, right| left.0.cmp(&right.0));
        removals.sort();

        let delta = PropertyDelta {
            upserts,
            removals,
        };

        DeltaBranch::PropertiesChanged(self.into_update_construction(delta))
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
//  Transitions: NewConstruction
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: UpdateConstruction
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: FetchConstruction
// ─────────────────────────────────────────────────────────────────────────────

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
//  Transitions: Completed
// ─────────────────────────────────────────────────────────────────────────────

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
