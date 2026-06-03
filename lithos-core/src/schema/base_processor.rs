//! Typed processing pipeline for building, fetching, or updating a
//! `BaseSchema`.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline with three paths:
//!
//! - **Missing path**: When no cached view exists, construct a new `BaseSchema`
//!   and persist it.
//! - **Fresh path**: When a view exists and timestamps match, fetch the cached
//!   `BaseSchema` without reading file content.
//! - **Stale path**: When timestamps differ, read file content, compare the
//!   stored content hash, parse stale content only when needed, compute
//!   property/excludes/extends semantic deltas, then either refresh metadata
//!   for no-op changes or persist a stale semantic update.
//!
//! # Flow
//!
//! ```text
//! Entry
//!   ├─ No view
//!   │   → [Construction] construct defaults → Completed(NewReady)
//!   └─ View found
//!       → [Comparison] check timestamps
//!
//! Timestamp Check
//!   ├─ [match]
//!   │   → [Construction] fetch cached base schema → Completed(FreshReady)
//!   └─ [mismatch]
//!       → [Comparison] read content and check content hash
//!
//! Content Check
//!   ├─ [match]
//!   │   → [Refresh] sync metadata
//!   │   → [Construction] fetch cached base schema → Completed(FreshReady)
//!   └─ [mismatch]
//!       → [Parsed] parse raw schema
//!       → [Analysis] compute property/excludes/extends deltas
//!
//! Semantic Analysis
//!   ├─ [no changes]
//!   │   → [Refresh] sync metadata + content hash
//!   │   → [Construction] fetch cached base schema → Completed(FreshReady)
//!   ├─ [changes]
//!   │   → [Construction] persist updated BaseSchema + RawSchemaView
//!   │   → Completed(StaleReady)
//!   └─ [corrupt view]
//!       → [Construction] full rebuild fallback → Completed(NewReady)
//! ```

use std::marker::PhantomData;

use crate::{
    fs::{DirPath, FileReader, FsFile, PathKey},
    schema::{
        base::BaseSchema,
        delta::{
            ExcludesDelta, ExtendsDelta, PropertyDelta, PropertyDeltaEngine,
        },
        error::{
            SchemaFileError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError,
        },
        expander::RefExpander,
        identifier::{SchemaId, SchemaName},
        property::{PropertyMap, PropertyName},
        raw::RawSchema,
        repository::{ReadRepository, Repository, WriteRepository},
        views::{
            HashRecord, RawPropertyHashIndex, RawView as _, RawViewRead,
            SchemaVersion, contracts::Version as _, raw::RawSchemaView,
        },
    },
    support::content_hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  Processor Core
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
#[must_use]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
pub(crate) struct BaseSchemaProcessor<P, S> {
    file: FsFile,
    path_key: PathKey,
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> BaseSchemaProcessor<P, S> {
    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn into_parts(self) -> (FsFile, PathKey, S) {
        (self.file, self.path_key, self.status)
    }

    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn transition_from_parts<NP, NS>(
        file: FsFile,
        path_key: PathKey,
        status: NS,
    ) -> BaseSchemaProcessor<NP, NS> {
        BaseSchemaProcessor {
            file,
            path_key,
            status,
            _stage: PhantomData,
        }
    }

    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn transition<NP, NS>(
        self,
        _stage: NP,
        status: NS,
    ) -> BaseSchemaProcessor<NP, NS> {
        let (file, path_key, _) = self.into_parts();
        Self::transition_from_parts(file, path_key, status)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Resolution
// ─────────────────────────────────────────────────────────────────────────────

/// The result of a base schema resolution attempt.
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
#[expect(
    clippy::large_enum_variant,
    reason = "Issue 04 requires Stale to carry explicit BaseSchema and deltas"
)]
pub(crate) enum BaseSchemaResolution {
    /// Base schema was already fresh in the repository.
    Fresh {
        /// The schema identifier.
        schema_id: SchemaId,
        /// The cached base schema.
        base_schema: BaseSchema,
    },
    /// Base schema was newly constructed.
    New {
        /// The newly constructed base schema.
        base_schema: BaseSchema,
    },
    /// Base schema changed semantically and was incrementally updated.
    Stale {
        /// The reused schema identifier.
        schema_id: SchemaId,
        /// The updated base schema.
        base_schema: BaseSchema,
        /// Direct property semantic changes.
        property_delta: PropertyDelta,
        /// Exclude-list semantic changes.
        excludes_delta: ExcludesDelta,
        /// Extends-list semantic changes.
        extends_delta: ExtendsDelta,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
//  Entry Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Initial state before any knowledge has been gathered.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
pub(crate) struct Unknown;

/// Entry-point stage: processor created from discovery data.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
pub(crate) struct Init;

/// Entry-state operations that bootstrap the pipeline.
impl BaseSchemaProcessor<Init, Unknown> {
    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
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

    /// Derive a `SchemaName` from the file's basename.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn schema_name_from_path(
        file: &FsFile,
    ) -> Result<SchemaName, SchemaLoaderError> {
        let basename = file.path().basename().ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                SchemaFileError::InvalidFileName {
                    path: file.path().as_path().to_path_buf(),
                    reason: "missing file stem".into(),
                },
            ))
        })?;
        Ok(SchemaName::try_new(basename.as_str())?)
    }

    /// Run the full base schema pipeline.
    ///
    /// When `view` is `None`, the processor runs the missing path
    /// (construct from defaults → persist).
    /// When `view` is `Some(...)`, the processor runs the present path
    /// (check timestamps → check content → parse → analyze → refresh/update).
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    pub(crate) fn run<R: Repository>(
        self,
        view: Option<&RawSchemaView>,
        source: &FileReader,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        if let Some(view) = view {
            let present = self.transition(Comparison, Present {
                view: view.clone(),
            });
            Self::run_present(present, source, repository)
        } else {
            self.run_missing(repository)
        }
    }

    /// Internal helper for the missing path (no cached view exists).
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn run_missing<R: Repository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        let (file, path_key, _status) = self.into_parts();
        let schema_name = Self::schema_name_from_path(&file)?;
        let constructed = Self::transition_from_parts(file, path_key, New {
            id: SchemaId::new(),
            schema_name,
            properties: PropertyMap::new(),
            extends: Vec::new(),
            excludes: Vec::new(),
        });
        let completed = constructed.create(repository)?;
        Ok(BaseSchemaResolution::New {
            base_schema: completed.into_base(),
        })
    }

    /// Internal helper for the present path (cached view exists).
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn run_present<R: Repository>(
        processor: BaseSchemaProcessor<Comparison, Present>,
        source: &FileReader,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        match processor.check_timestamps(source)? {
            TimestampBranch::Match(fresh) => {
                let completed = fresh.fetch(repository)?;
                let (schema_id, base_schema) = completed.into_fresh_parts();
                Ok(BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                })
            }
            TimestampBranch::Mismatch(suspect) => {
                Self::run_content_check(suspect, repository)
            }
        }
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn run_content_check<R: Repository>(
        processor: BaseSchemaProcessor<Comparison, Suspect>,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        match processor.check_content() {
            ContentBranch::Match(stale_timestamps) => {
                let fresh = stale_timestamps.sync_metadata(repository)?;
                let completed = fresh.fetch(repository)?;
                let (schema_id, base_schema) = completed.into_fresh_parts();
                Ok(BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                })
            }
            ContentBranch::Mismatch(stale) => {
                let parsed = stale.parse()?;
                Self::run_analysis(parsed.analyze(repository)?, repository)
            }
        }
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn run_analysis<R: Repository>(
        branch: AnalysisBranch,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        match branch {
            AnalysisBranch::Empty(stale_content) => {
                let fresh = stale_content.sync_metadata(repository)?;
                let completed = fresh.fetch(repository)?;
                let (schema_id, base_schema) = completed.into_fresh_parts();
                Ok(BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                })
            }
            AnalysisBranch::Delta(changed) => {
                let completed = changed.update(repository)?;
                Ok(completed.into_stale_resolution())
            }
            AnalysisBranch::Corrupt(new) => {
                let completed = new.create(repository)?;
                Ok(BaseSchemaResolution::New {
                    base_schema: completed.into_base(),
                })
            }
        }
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn diff_properties<R: Repository>(
        raw: &RawSchema,
        previous_hashes: &RawPropertyHashIndex,
        repository: &R,
    ) -> Result<PropertyDelta, SchemaLoaderError> {
        let engine = PropertyDeltaEngine::for_schema(raw, previous_hashes);
        if let Some(bank) = repository
            .get_property_bank()
            .map_err(SchemaLoaderError::Repository)?
        {
            engine.diff_schema(&RefExpander::new(&bank))
        } else {
            engine.diff_schema_without_bank()
        }
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn updated_base_schema<R: Repository>(
        schema_id: SchemaId,
        raw: &RawSchema,
        repository: &R,
    ) -> Result<BaseSchema, SchemaLoaderError> {
        let properties = if let Some(bank) = repository
            .get_property_bank()
            .map_err(SchemaLoaderError::Repository)?
        {
            let expander = RefExpander::new(&bank);
            let mut resolved_properties = expander
                .expand_properties(&raw.properties().ref_entries())
                .map_err(SchemaLoaderError::Resolution)?;
            let inline_entries = raw.properties().inline_entries();
            if !inline_entries.is_empty() {
                resolved_properties.extend(
                    PropertyMap::try_from(inline_entries)
                        .map_err(SchemaLoaderError::Resolution)?,
                );
            }
            resolved_properties
        } else {
            let refs = raw.properties().ref_entries();
            if !refs.is_empty() {
                tracing::warn!(
                    ref_count = refs.len(),
                    "property bank missing; treating schema refs as unexpanded"
                );
            }
            PropertyMap::try_from(raw.properties().inline_entries())
                .map_err(SchemaLoaderError::Resolution)?
        };

        let schema_name = SchemaName::try_new(raw.name())
            .map_err(SchemaLoaderError::Resolution)?;
        Ok(BaseSchema::new(
            schema_id,
            schema_name,
            properties,
            raw.extends().to_vec(),
            raw.excludes().to_vec(),
        ))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Comparison Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Identity phase: compare timestamps and content hashes.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Comparison;

/// Proven: cached view exists for this path.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Present {
    view: RawSchemaView,
}

/// Proven: timestamps differ; content has been read for hash comparison.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Suspect {
    view: RawSchemaView,
    content: String,
}

/// Proven: content hash differs; content is retained for parsing.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Stale {
    content: String,
    content_hash: Blake3Hash,
    view: RawSchemaView,
}

#[derive(Debug)]
#[must_use = "timestamp branches must continue the pipeline"]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
enum TimestampBranch {
    Match(BaseSchemaProcessor<Construction, Fresh>),
    Mismatch(BaseSchemaProcessor<Comparison, Suspect>),
}

impl BaseSchemaProcessor<Comparison, Present> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn check_timestamps(
        self,
        source: &FileReader,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();
        let timestamps_match = status.view.is_timestamp_match(
            file.metadata().times().created_at(),
            file.metadata().times().modified_at(),
        );

        if timestamps_match {
            Ok(TimestampBranch::Match(Self::transition_from_parts(
                file,
                path_key,
                Fresh {
                    view: status.view,
                },
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

#[derive(Debug)]
#[must_use = "content branches must continue the pipeline"]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
enum ContentBranch {
    Match(BaseSchemaProcessor<Refresh, StaleTimestamps>),
    Mismatch(BaseSchemaProcessor<Parsed, Stale>),
}

impl BaseSchemaProcessor<Comparison, Suspect> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn check_content(self) -> ContentBranch {
        let (file, path_key, status) = self.into_parts();
        let content_hash = Blake3Hash::compute(status.content.as_bytes());

        if status.view.is_content_match(&content_hash) {
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

/// Parsing phase for stale file content.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Parsed;

/// Proven: stale content parsed into a raw schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct ParsedStale {
    raw: RawSchema,
    content_hash: Blake3Hash,
    view: RawSchemaView,
}

impl BaseSchemaProcessor<Parsed, Stale> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn parse(
        self,
    ) -> Result<BaseSchemaProcessor<Analysis, ParsedStale>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let raw: RawSchema = FileReader::parse_structured_from_str(
            file.path().as_path(),
            &status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
        let raw = raw
            .with_name(
                BaseSchemaProcessor::<Init, Unknown>::schema_name_from_path(
                    &file,
                )?
                .as_str()
                .into(),
            )
            .with_metadata(file.metadata().clone());

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

/// Semantic phase: compute property, exclude-list, and extends-list deltas.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Analysis;

#[derive(Debug)]
#[must_use = "analysis branches must continue the pipeline"]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
#[expect(
    clippy::large_enum_variant,
    reason = "Typestate branch carries exact next-state payloads"
)]
enum AnalysisBranch {
    Empty(BaseSchemaProcessor<Refresh, StaleContent>),
    Delta(BaseSchemaProcessor<Construction, Changed>),
    Corrupt(BaseSchemaProcessor<Construction, New>),
}

impl BaseSchemaProcessor<Analysis, ParsedStale> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn analyze<R: Repository>(
        self,
        repository: &R,
    ) -> Result<AnalysisBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();

        let Some(version) = status.view.current() else {
            let schema_name =
                BaseSchemaProcessor::<Init, Unknown>::schema_name_from_path(
                    &file,
                )?;
            return Ok(AnalysisBranch::Corrupt(Self::transition_from_parts(
                file,
                path_key,
                New {
                    id: SchemaId::new(),
                    schema_name,
                    properties: PropertyMap::new(),
                    extends: Vec::new(),
                    excludes: Vec::new(),
                },
            )));
        };

        let property_delta =
            BaseSchemaProcessor::<Init, Unknown>::diff_properties(
                &status.raw,
                version.hashes().properties(),
                repository,
            )?;
        let excludes_delta = ExcludesDelta::from_slices(
            version.excludes(),
            status.raw.excludes(),
        );
        let extends_delta =
            ExtendsDelta::from_slices(version.extends(), status.raw.extends());

        if property_delta.is_empty()
            && excludes_delta.is_empty()
            && extends_delta.is_empty()
        {
            Ok(AnalysisBranch::Empty(Self::transition_from_parts(
                file,
                path_key,
                StaleContent {
                    view: status.view,
                    content_hash: status.content_hash,
                },
            )))
        } else {
            Ok(AnalysisBranch::Delta(Self::transition_from_parts(
                file,
                path_key,
                Changed {
                    raw: status.raw,
                    view: status.view,
                    content_hash: status.content_hash,
                    property_delta,
                    excludes_delta,
                    extends_delta,
                },
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Refresh Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Maintenance phase: persist metadata-only freshness changes.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Refresh;

/// Proven: only timestamps changed.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct StaleTimestamps {
    view: RawSchemaView,
}

/// Proven: content hash changed but semantic state did not.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct StaleContent {
    view: RawSchemaView,
    content_hash: Blake3Hash,
}

impl BaseSchemaProcessor<Refresh, StaleTimestamps> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn sync_metadata<R: Repository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        let (file, path_key, mut status) = self.into_parts();
        let schema_id = repository
            .find_schema_id_by_path(status.view.path())?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundByPath(
                        status.view.path().clone(),
                    ),
                )
            })?;
        status.view.update_metadata(file.metadata().clone());
        repository
            .save_raw_schema_view(schema_id, &status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Fresh {
            view: status.view,
        }))
    }
}

impl BaseSchemaProcessor<Refresh, StaleContent> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn sync_metadata<R: Repository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        let (file, path_key, mut status) = self.into_parts();
        let schema_id = repository
            .find_schema_id_by_path(status.view.path())?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundByPath(
                        status.view.path().clone(),
                    ),
                )
            })?;
        let version = {
            let current = status.view.current().ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::EmptyVersionHistory(
                        path_key.clone(),
                    ),
                )
            })?;
            let hashes = HashRecord::new(
                status.content_hash,
                current.hashes().properties().clone(),
            );
            current.with_metadata(file.metadata().clone(), hashes)
        };
        status.view.add_version(version);
        repository
            .save_raw_schema_view(schema_id, &status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Fresh {
            view: status.view,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Building phase: terminal domain construction.
///
/// Activated after the entry stage resolves into either a missing (`New`) or
/// present (`Fresh`) path. Constructing the `BaseSchema` requires a
/// `WriteRepository`.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Construction;

/// Raw construction inputs for building a new `BaseSchema` from scratch.
///
/// Entered from the missing-view path or stale-timestamp path when no cached
/// view exists or timestamps have drifted. Produces a `Completed<NewReady>`.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct New {
    id: SchemaId,
    schema_name: SchemaName,
    properties: PropertyMap,
    extends: Vec<SchemaName>,
    excludes: Vec<PropertyName>,
}

/// Proven: identity is fully synchronized; schema can be fetched without
/// rebuild.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Fresh {
    view: RawSchemaView,
}

/// Proven: semantic divergence exists and must be persisted.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Changed {
    raw: RawSchema,
    view: RawSchemaView,
    content_hash: Blake3Hash,
    property_delta: PropertyDelta,
    excludes_delta: ExcludesDelta,
    extends_delta: ExtendsDelta,
}

/// Construction operations that build the base schema.
impl BaseSchemaProcessor<Construction, New> {
    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn create<R: WriteRepository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Completed, NewReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();

        let base = BaseSchema::new(
            status.id,
            status.schema_name,
            status.properties,
            status.extends,
            status.excludes,
        );

        repository
            .save_base_schema(&base)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, NewReady {
            base,
        }))
    }
}

impl BaseSchemaProcessor<Construction, Changed> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn update<R: Repository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Completed, StaleReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let schema_id = repository
            .find_schema_id_by_path(status.view.path())?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundByPath(
                        status.view.path().clone(),
                    ),
                )
            })?;
        let property_hashes = status.raw.properties().compute_hashes();
        let hashes =
            HashRecord::new(status.content_hash, property_hashes.into());
        let version =
            SchemaVersion::new(file.metadata().clone(), hashes, &status.raw)
                .map_err(SchemaLoaderError::Ingestion)?;
        let mut updated_view = status.view;
        updated_view.add_version(version);
        let updated_base =
            BaseSchemaProcessor::<Init, Unknown>::updated_base_schema(
                schema_id,
                &status.raw,
                repository,
            )?;

        repository
            .save_base_schema(&updated_base)
            .map_err(SchemaLoaderError::Repository)?;
        repository
            .save_raw_schema_view(schema_id, &updated_view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, StaleReady {
            schema_id,
            base: updated_base,
            property_delta: status.property_delta,
            excludes_delta: status.excludes_delta,
            extends_delta: status.extends_delta,
        }))
    }
}

/// Construction operations that fetch the cached base schema.
///
/// Used after direct timestamp matches and after stale timestamp/content
/// normalization paths have refreshed view metadata without semantic changes.
impl BaseSchemaProcessor<Construction, Fresh> {
    #[inline]
    fn fetch<R: ReadRepository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Completed, FreshReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let schema_id = repository
            .find_schema_id_by_path(status.view.path())?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundByPath(
                        status.view.path().clone(),
                    ),
                )
            })?;
        let base =
            repository.find_base_schema_by_id(schema_id)?.ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundById(schema_id),
                )
            })?;

        Ok(Self::transition_from_parts(file, path_key, FreshReady {
            id: schema_id,
            base,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal phase: the `BaseSchema` is ready and owned.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Completed;

/// Proven: terminal ingestion goal reached with newly built schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct NewReady {
    base: BaseSchema,
}

/// Proven: terminal ingestion goal reached with freshly fetched schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct FreshReady {
    id: SchemaId,
    base: BaseSchema,
}

/// Proven: terminal ingestion goal reached with stale updates applied.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct StaleReady {
    schema_id: SchemaId,
    base: BaseSchema,
    property_delta: PropertyDelta,
    excludes_delta: ExcludesDelta,
    extends_delta: ExtendsDelta,
}

/// Completed operations that expose the final base schema.
impl BaseSchemaProcessor<Completed, NewReady> {
    #[inline]
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn into_base(self) -> BaseSchema {
        self.status.base
    }
}

impl BaseSchemaProcessor<Completed, FreshReady> {
    #[inline]
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn into_base(self) -> BaseSchema {
        self.status.base
    }

    #[inline]
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn into_fresh_parts(self) -> (SchemaId, BaseSchema) {
        (self.status.id, self.status.base)
    }
}

impl BaseSchemaProcessor<Completed, StaleReady> {
    #[inline]
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn into_stale_resolution(self) -> BaseSchemaResolution {
        BaseSchemaResolution::Stale {
            schema_id: self.status.schema_id,
            base_schema: self.status.base,
            property_delta: self.status.property_delta,
            excludes_delta: self.status.excludes_delta,
            extends_delta: self.status.extends_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::{DirPath, FsFile},
        schema::{
            repository::ReadRepository, storage::testing::InMemoryRepository,
            views::RawView,
        },
    };

    /// Helper to create a [`RawSchemaView`] with the fixture file's timestamps.
    fn matching_view(fixture: &fixtures::Fixture) -> RawSchemaView {
        let metadata = fixture.file.metadata().clone();
        let raw = crate::schema::raw::RawSchema {
            version: crate::schema::raw::RawSchemaVersion::default(),
            name: "test-schema".into(),
            extends: vec![],
            excludes: vec![],
            properties: crate::schema::raw::property::RawPropertyMap::from_map(
                std::collections::HashMap::new(),
            ),
            metadata: metadata.clone(),
        };
        let hash = crate::support::content_hash::Blake3Hash::compute(
            fixture.content.as_bytes(),
        );
        let hashes = crate::schema::views::hashes::HashRecord::new(
            hash,
            crate::schema::views::hashes::RawPropertyHashIndex::default(),
        );
        let mut view = RawSchemaView::try_from_raw_with_hashes(
            &raw,
            fixture.key.clone(),
            hashes,
        )
        .expect("view");
        view.update_metadata(fixture.file.metadata().clone());
        view
    }

    fn stale_view(fixture: &fixtures::Fixture, content: &str) -> RawSchemaView {
        use crate::fs::metadata::{FileMetadata, FsTimes};

        let current = fixture.file.metadata().clone();
        let old_time = SystemTime::now()
            .checked_sub(Duration::from_secs(3600))
            .expect("old time");
        let stale_metadata = FileMetadata::new(
            FsTimes::new(Some(old_time), Some(old_time)),
            current.size(),
            current.is_symlink(),
        );
        let raw =
            parse_raw_schema(fixture, content).with_metadata(stale_metadata);
        let content_hash = Blake3Hash::compute(content.as_bytes());
        let hashes = crate::schema::views::hashes::HashRecord::new(
            content_hash,
            raw.properties().compute_hashes().into(),
        );

        RawSchemaView::try_from_raw_with_hashes(
            &raw,
            fixture.key.clone(),
            hashes,
        )
        .expect("view")
    }

    fn parse_raw_schema(
        fixture: &fixtures::Fixture,
        content: &str,
    ) -> RawSchema {
        FileReader::parse_structured_from_str::<RawSchema>(
            fixture.file.path().as_path(),
            content,
        )
        .expect("raw schema")
        .with_name("test-schema".into())
        .with_metadata(fixture.file.metadata().clone())
    }

    fn base_schema_with_id(schema_id: SchemaId) -> BaseSchema {
        BaseSchema::new(
            schema_id,
            SchemaName::try_new("test-schema").expect("name"),
            PropertyMap::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn seed_base_and_view(
        fixture: &fixtures::Fixture,
        schema_id: SchemaId,
        view: &RawSchemaView,
    ) -> BaseSchema {
        let base = base_schema_with_id(schema_id);
        fixture.repository.save_base_schema(&base).expect("save base");
        fixture
            .repository
            .save_raw_schema_view(schema_id, view)
            .expect("save view");
        base
    }

    mod fixtures {
        use super::*;

        pub(super) struct Fixture {
            pub(super) repository: InMemoryRepository,
            pub(super) source: FileReader,
            pub(super) vault_root: DirPath,
            pub(super) _vault_dir: TempDir,
            pub(super) file: FsFile,
            pub(super) key: PathKey,
            pub(super) content: String,
        }

        pub(super) fn make_fixture() -> Fixture {
            let vault_dir = TempDir::new().expect("temp dir");
            let vault_root = DirPath::try_new(vault_dir.path().to_path_buf())
                .expect("vault root");
            let relative = std::path::PathBuf::from("schemas/test-schema.yaml");
            let absolute = vault_dir.path().join(&relative);
            std::fs::create_dir_all(absolute.parent().expect("parent"))
                .expect("mkdir");
            let content = "properties: {}".to_owned();
            std::fs::write(&absolute, &content).expect("write file");

            let source = FileReader::new(vault_dir.path());
            let file_path = crate::fs::FilePath::try_new(absolute.clone())
                .expect("file path");
            let metadata =
                crate::fs::metadata::FsMetadata::from_path(file_path.as_path())
                    .expect("metadata")
                    .as_file()
                    .cloned()
                    .expect("file metadata");
            let file = FsFile::new(file_path.clone(), metadata.clone());
            let key = file.path().as_key(&vault_root).expect("path key");

            Fixture {
                repository: InMemoryRepository::new(),
                source,
                vault_root,
                _vault_dir: vault_dir,
                file,
                key,
                content,
            }
        }

        pub(super) fn write_schema(fixture: &mut Fixture, content: &str) {
            std::fs::write(fixture.file.path().as_path(), content)
                .expect("write schema");
            let metadata = crate::fs::metadata::FsMetadata::from_path(
                fixture.file.path().as_path(),
            )
            .expect("metadata")
            .as_file()
            .cloned()
            .expect("file metadata");
            fixture.file = FsFile::new(fixture.file.path().clone(), metadata);
            fixture.content = content.to_owned();
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn from_discovery_returns_processor_with_unknown() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            assert!(matches!(processor.status, Unknown));
        }
    }

    mod run {
        use super::*;

        macro_rules! expect_new {
            ($resolution:expr) => {{
                let resolution = $resolution;
                let BaseSchemaResolution::New {
                    base_schema,
                } = resolution
                else {
                    panic!("Expected New resolution");
                };
                base_schema
            }};
        }

        macro_rules! expect_fresh {
            ($resolution:expr) => {{
                let resolution = $resolution;
                let BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                } = resolution
                else {
                    panic!("Expected Fresh resolution");
                };
                (schema_id, base_schema)
            }};
        }

        #[test]
        fn missing_constructs_and_persists_base_schema() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            assert!(matches!(resolution, BaseSchemaResolution::New { .. }));
        }

        #[test]
        fn new_resolution_derives_name_from_file_basename() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert_eq!(base_schema.name().as_str(), "test-schema");
        }

        #[test]
        fn new_resolution_constructs_with_empty_defaults() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert!(base_schema.properties().is_empty());
            assert!(base_schema.extends().is_empty());
            assert!(base_schema.excludes().is_empty());
        }

        #[test]
        fn present_returns_fresh_when_timestamps_match() {
            let fixture = fixtures::make_fixture();

            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            fixture.repository.save_base_schema(&base).expect("save base");

            let view = matching_view(&fixture);
            fixture
                .repository
                .save_raw_schema_view(schema_id, &view)
                .expect("save view");

            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");

            let (sid, bs) = expect_fresh!(resolution);
            assert_eq!(sid, schema_id);
            assert_eq!(bs.name().as_str(), "test-schema");
        }

        #[test]
        fn present_does_not_write_when_fresh() {
            let fixture = fixtures::make_fixture();

            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            fixture.repository.save_base_schema(&base).expect("save base");

            let view = matching_view(&fixture);
            fixture
                .repository
                .save_raw_schema_view(schema_id, &view)
                .expect("save view");

            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");

            let (_sid, bs) = expect_fresh!(resolution);
            assert_eq!(bs, base);
        }

        #[test]
        fn missing_persisted_base_schema_is_retrievable() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);

            let found = fixture
                .repository
                .find_base_schema_by_id(*base_schema.id())
                .expect("find base")
                .expect("base schema should exist");

            assert_eq!(found, base_schema);
        }

        mod normalization {
            use super::*;

            #[test]
            fn returns_fresh_when_content_matches_after_timestamp_mismatch() {
                let fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, &fixture.content);
                seed_base_and_view(&fixture, schema_id, &view);
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (fresh_id, _) = expect_fresh!(resolution);
                assert_eq!(fresh_id, schema_id);
            }

            #[test]
            fn persists_view_when_normalizing_stale_timestamps() {
                let fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, &fixture.content);
                seed_base_and_view(&fixture, schema_id, &view);
                fixture.repository.harness().counters().reset();
                let fixture_times = fixture.file.metadata().times().clone();
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                assert!(matches!(
                    resolution,
                    BaseSchemaResolution::Fresh { .. }
                ));
                let saved = fixture
                    .repository
                    .get_raw_schema_view(schema_id)
                    .expect("get view")
                    .expect("view");
                assert!(saved.is_timestamp_match(
                    fixture_times.created_at(),
                    fixture_times.modified_at(),
                ));
                let snapshot =
                    fixture.repository.harness().counters().snapshot();
                assert_eq!(snapshot.writes, 2);
            }

            #[test]
            fn returns_fresh_when_semantic_state_is_unchanged_after_content_mismatch()
             {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let old_content = "properties: {}";
                let view = stale_view(&fixture, old_content);
                let base = seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(&mut fixture, "properties: {}\n");
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (fresh_id, fresh_base) = expect_fresh!(resolution);
                assert_eq!(fresh_id, schema_id);
                assert_eq!(fresh_base, base);
            }

            #[test]
            fn appends_version_without_mutating_prior_metadata_when_content_normalizes()
             {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let old_content = "properties: {}";
                let view = stale_view(&fixture, old_content);
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(&mut fixture, "properties: {}\n");
                let new_metadata = fixture.file.metadata().clone();
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                assert!(matches!(
                    resolution,
                    BaseSchemaResolution::Fresh { .. }
                ));
                let saved = fixture
                    .repository
                    .get_raw_schema_view(schema_id)
                    .expect("get view")
                    .expect("view");
                assert_eq!(saved.version_count(), view.version_count() + 1);
                assert_eq!(
                    saved.current().expect("new current").metadata(),
                    &new_metadata
                );
            }
        }

        mod analysis {
            use super::*;

            macro_rules! expect_stale {
                ($resolution:expr) => {{
                    let BaseSchemaResolution::Stale {
                        schema_id,
                        base_schema,
                        property_delta,
                        excludes_delta,
                        extends_delta,
                    } = $resolution
                    else {
                        panic!("Expected Stale resolution");
                    };
                    (
                        schema_id,
                        base_schema,
                        property_delta,
                        excludes_delta,
                        extends_delta,
                    )
                }};
            }

            #[test]
            fn returns_stale_when_property_delta_detected() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    "properties:\n  title:\n    type: string\n",
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (_, _, property_delta, _, _) = expect_stale!(resolution);
                assert!(!property_delta.is_empty());
            }

            #[test]
            fn returns_stale_when_extends_delta_detected() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    "extends: parent\nproperties: {}\n",
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (_, _, _, _, extends_delta) = expect_stale!(resolution);
                assert!(!extends_delta.is_empty());
            }

            #[test]
            fn returns_stale_when_excludes_delta_detected() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    "excludes:\n  - title\nproperties: {}\n",
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (_, _, _, excludes_delta, _) = expect_stale!(resolution);
                assert!(!excludes_delta.is_empty());
            }

            #[test]
            fn reuses_schema_id_when_returning_stale() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    "properties:\n  title:\n    type: string\n",
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (stale_id, base_schema, _, _, _) =
                    expect_stale!(resolution);
                assert_eq!(stale_id, schema_id);
                assert_eq!(base_schema.id(), &schema_id);
            }

            #[test]
            fn appends_view_version_when_returning_stale() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    "properties:\n  title:\n    type: string\n",
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                assert!(matches!(
                    resolution,
                    BaseSchemaResolution::Stale { .. }
                ));
                let saved = fixture
                    .repository
                    .get_raw_schema_view(schema_id)
                    .expect("get view")
                    .expect("view");
                assert_eq!(saved.version_count(), view.version_count() + 1);
            }

            #[test]
            fn logs_and_treats_refs_as_empty_when_property_bank_is_missing() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(
                    &mut fixture,
                    r##"properties:
  from_bank:
    $ref: "#property_bank/title"
"##,
                );
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(Some(&view), &fixture.source, &fixture.repository)
                    .expect("run");

                let (_, base_schema) = expect_fresh!(resolution);
                assert!(base_schema.properties().is_empty());
            }
        }

        mod fallback {
            use super::*;

            #[test]
            fn returns_new_when_view_has_no_current_version() {
                let fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, &fixture.content);
                seed_base_and_view(&fixture, schema_id, &view);
                let corrupt_view = RawSchemaView::empty_for_test(fixture.key);
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let resolution = processor
                    .run(
                        Some(&corrupt_view),
                        &fixture.source,
                        &fixture.repository,
                    )
                    .expect("run");

                assert!(matches!(resolution, BaseSchemaResolution::New { .. }));
            }

            #[test]
            fn returns_error_when_parse_fails() {
                let mut fixture = fixtures::make_fixture();
                let schema_id = SchemaId::new();
                let view = stale_view(&fixture, "properties: {}");
                seed_base_and_view(&fixture, schema_id, &view);
                fixtures::write_schema(&mut fixture, "properties: [");
                let processor =
                    BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                        fixture.file,
                        &fixture.vault_root,
                    )
                    .expect("processor");

                let result = processor.run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                );

                assert!(result.is_err(), "Expected parse error");
            }
        }
    }

    mod terminal {
        use super::*;

        #[test]
        fn new_ready_into_base_returns_constructed_schema() {
            let fixture = fixtures::make_fixture();
            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            let expected = base.clone();

            let processor = BaseSchemaProcessor::<Completed, NewReady> {
                file: fixture.file,
                path_key: fixture.key,
                status: NewReady {
                    base: expected.clone(),
                },
                _stage: PhantomData,
            };

            let result = processor.into_base();
            assert_eq!(result, expected);
        }

        #[test]
        fn fresh_ready_into_base_returns_fetched_schema() {
            let fixture = fixtures::make_fixture();
            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            let expected = base.clone();

            let processor = BaseSchemaProcessor::<Completed, FreshReady> {
                file: fixture.file,
                path_key: fixture.key,
                status: FreshReady {
                    id: schema_id,
                    base: expected.clone(),
                },
                _stage: PhantomData,
            };

            let result = processor.into_base();
            assert_eq!(result, expected);
        }
    }
}
