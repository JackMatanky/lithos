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
//!     - `parse` -> `NewRawView`
//!
//! 3. **IsFreshTimestamp**: Tier 2 check (metadata matching).
//!     - `build` -> `Completed` (Fastest path: fetch from DB)
//!     - `to_fresh_content` -> `IsFreshContent`
//!
//! 4. **IsFreshContent**: Tier 3 check (hash matching).
//!     - `to_raw_time_update` -> `RawViewTimeUpdate`
//!     - `to_stale` -> `IsStale`
//!
//! 5. **RawViewTimeUpdate**: Syncs metadata when content matches but timestamps
//!    differ.
//!     - `update` -> `Completed`
//!
//! 6. **IsStale**: Content changed.
//!     - `parse` -> `NewRawView` (with delta)
//!
//! 7. **NewRawView**: Persists new version and builds domain object.
//!     - `save` -> `Completed`
//!
//! 8. **Completed**: Terminal state.

use crate::{
    fs::FsReader,
    schema::{
        bank::PropertyBank,
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
            SchemaStorageError,
        },
        property::PropertyName,
        raw::{RawFileTimes, RawPropertyBank},
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
pub struct RawViewTimeUpdate {
    times: RawFileTimes,
    view: RawPropertyBankView,
}

/// Content changed - must compute delta and update incrementally.
#[derive(Debug)]
#[non_exhaustive]
pub struct IsStale<'source> {
    raw: RawPropertyBank,
    view: RawPropertyBankView,
    content: &'source str,
}

/// Ready to create a new version view and build/update the bank.
#[derive(Debug)]
#[non_exhaustive]
pub struct NewRawView {
    raw: RawPropertyBank,
    content: String, // Kept for view creation (compression)
    delta: Option<Vec<PropertyName>>,
}

/// Terminal state - `PropertyBank` is ready.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed {
    bank: PropertyBank,
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
    Match(PropertyBankProcessor<RawViewTimeUpdate>),
    /// Hash mismatches - compute delta.
    Mismatch(PropertyBankProcessor<IsStale<'source>>),
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
    /// Check if file timestamps match the cached view without reading content.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(&self) -> bool {
        self.state.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                self.state.times.created_at,
                self.state.times.modified_at,
            )
        })
    }

    /// Transition to content check if timestamps mismatch.
    #[inline]
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

    /// Path 2: Fastest path - fetch bank from DB and complete.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails or the bank
    /// is missing.
    #[inline]
    pub fn build<R: Repository>(
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
    pub fn match_content(
        self,
        config_path: &std::path::Path,
    ) -> Result<ContentBranch<'source>, SchemaLoaderError> {
        let content_hash = blake3::hash(self.state.content.as_bytes());
        let is_match = self.state.view.current().is_some_and(|v| {
            v.hashes().is_content_match(content_hash.as_bytes())
        });

        if is_match {
            Ok(ContentBranch::Match(Self::transition(RawViewTimeUpdate {
                times: self.state.times,
                view: self.state.view,
            })))
        } else {
            let raw: RawPropertyBank = FsReader::parse_structured_from_str(
                config_path,
                self.state.content,
            )
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

            let raw = raw.with_file_times(self.state.times);

            Ok(ContentBranch::Mismatch(Self::transition(IsStale {
                raw,
                view: self.state.view,
                content: self.state.content,
            })))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Transitions: RawViewTimeUpdate
// ─────────────────────────────────────────────────────────────────────────────

impl PropertyBankProcessor<RawViewTimeUpdate> {
    /// Update timestamps in the cached view.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails.
    #[inline]
    pub fn update<R: Repository>(
        mut self,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError>
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

        // After updating timestamps, we still need to fetch the bank
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
//  Transitions: IsNew
// ─────────────────────────────────────────────────────────────────────────────

impl PropertyBankProcessor<IsNew> {
    /// Parse the file content into a raw property bank.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the file cannot be parsed.
    #[inline]
    pub fn parse(
        self,
        config_path: &std::path::Path,
        content: &str,
    ) -> Result<PropertyBankProcessor<NewRawView>, SchemaLoaderError> {
        let raw: RawPropertyBank =
            FsReader::parse_structured_from_str(config_path, content)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw.with_file_times(self.state.times);

        Ok(Self::transition(NewRawView {
            raw,
            content: content.to_owned(),
            delta: None, // NEW path has no delta (full build)
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Transitions: IsStale
// ─────────────────────────────────────────────────────────────────────────────

impl PropertyBankProcessor<IsStale<'_>> {
    /// Compute the property delta and transition to `NewRawView`.
    ///
    /// The file is already parsed in this state, so we only need to compare
    /// hashes with the cached view.
    #[inline]
    pub fn compute_delta(self) -> PropertyBankProcessor<NewRawView> {
        let new_hashes =
            HashMetadata::compute_property_hashes(self.state.raw.properties());
        let delta = self.state.view.current().map_or_else(
            || new_hashes.keys().cloned().collect(),
            |v| {
                v.hashes().changed_properties(&new_hashes).into_iter().collect()
            },
        );

        Self::transition(NewRawView {
            raw: self.state.raw,
            content: self.state.content.to_owned(),
            delta: Some(delta),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Transitions: NewRawView
// ─────────────────────────────────────────────────────────────────────────────

impl PropertyBankProcessor<NewRawView> {
    /// Save the new raw view and build/update the domain `PropertyBank`.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if the repository access fails or
    /// ingestion fails.
    #[inline]
    pub fn build<R: Repository>(
        self,
        filename: &str,
        repository: &R,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        // 1. Create and save new view
        let new_view = RawPropertyBankView::try_from_raw_with_content(
            &self.state.raw,
            filename,
            &self.state.content,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_raw_property_bank_view(filename, &new_view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // 2. Build or update PropertyBank
        let bank = if let Some(delta) = self.state.delta {
            // STALE path: fetch old and update
            let mut bank = repository
                .get_property_bank()
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?
                .unwrap_or_default();

            bank.update_from_raw(&self.state.raw, &delta).map_err(
                |source| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                        path: std::path::PathBuf::from("property_bank"),
                        source,
                    })
                },
            )?;
            bank
        } else {
            // NEW path: build from scratch
            PropertyBank::try_from(self.state.raw).map_err(|source| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                    path: std::path::PathBuf::from("property_bank"),
                    source,
                })
            })?
        };

        // 3. Save the bank
        repository
            .save_property_bank(&bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

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

    mod is_timestamp_match {
        use std::path::Path;

        use rstest::rstest;
        use tempfile::TempDir;

        use super::fixtures::{create_test_view, repo, temp_dir, write_config};
        use crate::{
            fs::FsReader,
            schema::{
                property_bank_processor::{
                    ContentBranch, Discovery, DiscoveryBranch,
                    PropertyBankProcessor,
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
                assert!(p.is_timestamp_match(), "Expected timestamp match");
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
            let branch = branch.unwrap();
            assert!(
                matches!(branch, DiscoveryBranch::FreshTimestamp(_)),
                "Expected FreshTimestamp branch, found: {branch:?}"
            );

            if let DiscoveryBranch::FreshTimestamp(p) = branch {
                assert!(!p.is_timestamp_match(), "Expected timestamp mismatch");
                let next = p.to_fresh_content(content);
                let content_branch = next.match_content(config_path);
                assert!(
                    content_branch.is_ok(),
                    "Expected success, found: {:?}",
                    content_branch.err()
                );
                assert!(
                    matches!(content_branch.unwrap(), ContentBranch::Match(_)),
                    "Expected Match branch"
                );
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
                    let completed = res.unwrap().build(filename, &repo);
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
}
