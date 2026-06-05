//! Typed processing pipeline for building, fetching, or updating a
//! `BaseSchema`.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline with three paths:
//!
//! - **Missing path**: When no cached view exists, read and parse the file,
//!   construct a new `BaseSchema`, and persist it.
//! - **Fresh path**: When a view exists and timestamps match, fetch the cached
//!   `BaseSchema` without reading file content, then check whether a bank delta
//!   requires targeted re-expansion.
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
//!   │   → [Parsed] parse raw file
//!   │   → [Construction] construct domain from raw → Completed
//!   └─ View found (schema_id looked up once in run_present)
//!       → [Comparison] check timestamps
//!
//! Timestamp Check
//!   ├─ [match]
//!   │   → [Construction] check bank refs → fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Comparison] check content hash
//!
//! Content Check
//!   ├─ [match]
//!   │   → [Refresh] sync timestamps
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Parsed] parse raw schema
//!       → [Analysis] compute property/excludes/extends deltas
//!
//! Semantic Analysis
//!   ├─ [no changes]
//!   │   → [Refresh] sync timestamps + content hash
//!   │   → [Construction] fetch cached domain → Completed
//!   ├─ [changes]
//!   │   → [Construction] augment with bank delta → persist → Completed
//!   └─ [corrupt view]
//!       → [Construction] full rebuild fallback → Completed
//! ```
//!
//! # Design notes
//!
//! - `schema_id` is resolved once in `run_present` and carried through all
//!   downstream states, eliminating redundant repository lookups.
//! - `bank` is never cloned into state; it is threaded as a parameter to the
//!   methods that require it (`create`, `update`).
//! - `StaleReferences` handling is orthogonal: `Fresh` checks bank refs before
//!   fetching; `Changed` augments the property delta before persisting.

use std::marker::PhantomData;

use crate::{
    fs::{DirPath, FileReader, FsFile, PathKey},
    schema::{
        bank::PropertyBank,
        base::BaseSchema,
        delta::{
            ExcludesDelta, ExtendsDelta, PropertyDelta, PropertyDeltaEngine,
        },
        error::{
            PropertyRefError, SchemaError, SchemaIngestionError,
            SchemaLoaderError, SchemaRepositoryError,
        },
        expander::RefExpander,
        identifier::{SchemaId, SchemaName},
        property::{PropertyMap, PropertyName},
        property_bank_processor::PropertyBankResolution,
        raw::RawSchema,
        repository::{ReadRepository, Repository, WriteRepository},
        views::{
            HashRecord, RawView as _, RawViewRead, SchemaVersion,
            contracts::Version as _, raw::RawSchemaView,
        },
    },
    support::content_hash::Blake3Hash,
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
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
pub struct BaseSchemaProcessor<P, S> {
    file: FsFile,
    path_key: PathKey,
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> BaseSchemaProcessor<P, S> {
    #[inline]
    fn into_parts(self) -> (FsFile, PathKey, S) {
        (self.file, self.path_key, self.status)
    }

    #[inline]
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
    reason = "Stale carries explicit BaseSchema and deltas per issue 04 \
              requirement"
)]
pub enum BaseSchemaResolution {
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
pub struct Unknown;

/// Entry-point stage: processor created from discovery data.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
pub struct Init;

/// Entry-state operations that bootstrap the pipeline.
impl BaseSchemaProcessor<Init, Unknown> {
    #[inline]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    pub fn from_discovery(
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

    /// Run the full base schema pipeline.
    ///
    /// When `view` is `None`, the processor runs the missing path
    /// (read file → parse → construct → persist).
    /// When `view` is `Some(...)`, the processor runs the present path
    /// (check timestamps → check content → parse → analyze → refresh/update).
    ///
    /// `bank_resolution` carries the already-loaded [`PropertyBankResolution`]
    /// from the preceding `PropertyBankProcessor` run. When `None`, an empty
    /// bank is used and no `StaleReferences` check occurs.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    #[inline]
    pub fn run<R: Repository>(
        self,
        view: Option<&RawSchemaView>,
        source: &FileReader,
        repository: &R,
        bank_resolution: Option<&PropertyBankResolution>,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        let empty_bank;
        let bank = if let Some(r) = bank_resolution {
            r.bank()
        } else {
            empty_bank = PropertyBank::new();
            &empty_bank
        };

        if let Some(view) = view {
            let present = self.transition(Comparison, Present {
                view: view.clone(),
            });
            Self::run_present(present, source, repository, bank_resolution)
        } else {
            let (init_file, init_key, _) = self.into_parts();
            let parsed = Self::transition_from_parts::<Parsed, Missing>(
                init_file, init_key, Missing,
            )
            .parse(source)?;
            let schema_id = SchemaId::new();
            let (file, path_key, status) = parsed.into_parts();
            let new_proc = Self::transition_from_parts(file, path_key, New {
                id: schema_id,
                raw: status.raw,
                content_hash: status.content_hash,
            });
            let completed = new_proc.create(repository, bank)?;
            Ok(BaseSchemaResolution::New {
                base_schema: completed.into_base(),
            })
        }
    }

    /// Internal helper for the present path (view exists).
    ///
    /// Resolves `schema_id` once here and carries it through all downstream
    /// states, eliminating redundant repository lookups.
    fn run_present<R: Repository>(
        processor: BaseSchemaProcessor<Comparison, Present>,
        source: &FileReader,
        repository: &R,
        bank_resolution: Option<&PropertyBankResolution>,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        let bank = if let Some(r) = bank_resolution {
            r.bank()
        } else {
            // Internal bank needed for expansion during analyze()
            &PropertyBank::new()
        };

        let schema_id = repository
            .find_schema_id_by_path(processor.status.view.path())
            .map_err(SchemaLoaderError::Repository)?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundByPath(
                        processor.status.view.path().clone(),
                    ),
                )
            })?;

        match processor.check_timestamps(source, schema_id, bank_resolution)? {
            TimestampBranch::Match(fresh) => {
                let completed = fresh.fetch(repository)?;
                let (sid, base_schema) = completed.into_fresh_parts();
                Ok(BaseSchemaResolution::Fresh {
                    schema_id: sid,
                    base_schema,
                })
            }
            TimestampBranch::StaleRefs(stale) => {
                let parsed = stale.parse()?;
                Self::run_analysis(
                    parsed.analyze(bank)?,
                    repository,
                    bank_resolution,
                )
            }
            TimestampBranch::Mismatch(suspect) => Self::run_content_check(
                suspect,
                repository,
                bank,
                bank_resolution,
            ),
        }
    }

    /// Internal helper for content-hash comparison after a timestamp mismatch.
    fn run_content_check<R: Repository>(
        processor: BaseSchemaProcessor<Comparison, Suspect>,
        repository: &R,
        bank: &PropertyBank,
        bank_resolution: Option<&PropertyBankResolution>,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        match processor.check_content(bank_resolution) {
            ContentBranch::Match(stale_timestamps) => {
                let fresh = stale_timestamps.sync_metadata(repository)?;
                let completed = fresh.fetch(repository)?;
                let (schema_id, base_schema) = completed.into_fresh_parts();
                Ok(BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                })
            }
            ContentBranch::StaleRefs(stale) => {
                let parsed = stale.parse()?;
                Self::run_analysis(
                    parsed.analyze(bank)?,
                    repository,
                    bank_resolution,
                )
            }
            ContentBranch::Mismatch(stale) => {
                let parsed = stale.parse()?;
                Self::run_analysis(
                    parsed.analyze(bank)?,
                    repository,
                    bank_resolution,
                )
            }
        }
    }

    /// Internal helper for semantic-delta analysis after a content-hash
    /// mismatch.
    fn run_analysis<R: Repository>(
        branch: AnalysisBranch,
        repository: &R,
        bank_resolution: Option<&PropertyBankResolution>,
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
                match changed
                    .with_bank_delta_upserts(bank_resolution, repository)
                {
                    Ok(augmented) => {
                        let completed = augmented.update(repository)?;
                        Ok(completed.into_stale_resolution())
                    }
                    Err(resolved) => Ok(resolved),
                }
            }
            AnalysisBranch::Corrupt(new) => {
                let empty_bank = PropertyBank::new();
                let (file, path_key, status) = new.into_parts();
                let raw_for_fallback = status.raw.clone();
                let id = status.id;

                let new_proc = BaseSchemaProcessor::<Construction, New>::transition_from_parts(
                    file,
                    path_key,
                    New {
                        id,
                        raw: status.raw,
                        content_hash: status.content_hash,
                    },
                );

                match new_proc.create(repository, &empty_bank) {
                    Ok(completed) => Ok(BaseSchemaResolution::New {
                        base_schema: completed.into_base(),
                    }),
                    Err(SchemaLoaderError::Repository(e)) => {
                        Err(SchemaLoaderError::Repository(e))
                    }
                    Err(_) => {
                        let schema_name =
                            SchemaName::try_new(raw_for_fallback.name())
                                .unwrap_or_else(|_| {
                                    // SAFETY: "unknown" always satisfies
                                    // SchemaName validation
                                    #[expect(
                                        clippy::unwrap_used,
                                        reason = "'unknown' is a hardcoded \
                                                  literal that always \
                                                  satisfies SchemaName \
                                                  validation"
                                    )]
                                    SchemaName::try_new("unknown").unwrap()
                                });

                        Ok(BaseSchemaResolution::New {
                            base_schema: BaseSchema::new(
                                id,
                                schema_name,
                                PropertyMap::new(),
                                Vec::new(),
                                Vec::new(),
                            ),
                        })
                    }
                }
            }
        }
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
    schema_id: SchemaId,
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
    schema_id: SchemaId,
}

/// Proven: bank references changed while content/timestamps matched.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct StaleReferences {
    content: String,
    content_hash: Blake3Hash,
    view: RawSchemaView,
    schema_id: SchemaId,
    ref_delta: Vec<PropertyName>,
}

#[derive(Debug)]
#[must_use = "timestamp branches must continue the pipeline"]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
enum TimestampBranch {
    Match(BaseSchemaProcessor<Construction, Fresh>),
    StaleRefs(BaseSchemaProcessor<Parsed, StaleReferences>),
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
        schema_id: SchemaId,
        bank_resolution: Option<&PropertyBankResolution>,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();
        let timestamps_match = status.view.is_timestamp_match(
            file.metadata().times().created_at(),
            file.metadata().times().modified_at(),
        );

        if timestamps_match {
            let ref_delta = relevant_bank_refs(&status.view, bank_resolution);

            if ref_delta.is_empty() {
                Ok(TimestampBranch::Match(Self::transition_from_parts(
                    file,
                    path_key,
                    Fresh {
                        schema_id,
                    },
                )))
            } else {
                let content = source
                    .read_to_string(file.path().as_path())
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                let content_hash = Blake3Hash::compute(content.as_bytes());

                Ok(TimestampBranch::StaleRefs(Self::transition_from_parts(
                    file,
                    path_key,
                    StaleReferences {
                        content,
                        content_hash,
                        view: status.view,
                        schema_id,
                        ref_delta,
                    },
                )))
            }
        } else {
            let content = source
                .read_to_string(file.path().as_path())
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
            Ok(TimestampBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Suspect {
                    view: status.view,
                    schema_id,
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
    StaleRefs(BaseSchemaProcessor<Parsed, StaleReferences>),
    Mismatch(BaseSchemaProcessor<Parsed, Stale>),
}

impl BaseSchemaProcessor<Comparison, Suspect> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn check_content(
        self,
        bank_resolution: Option<&PropertyBankResolution>,
    ) -> ContentBranch {
        let (file, path_key, status) = self.into_parts();
        let content_hash = Blake3Hash::compute(status.content.as_bytes());

        if status.view.is_content_match(&content_hash) {
            let ref_delta = relevant_bank_refs(&status.view, bank_resolution);

            if ref_delta.is_empty() {
                ContentBranch::Match(Self::transition_from_parts(
                    file,
                    path_key,
                    StaleTimestamps {
                        view: status.view,
                        schema_id: status.schema_id,
                    },
                ))
            } else {
                ContentBranch::StaleRefs(Self::transition_from_parts(
                    file,
                    path_key,
                    StaleReferences {
                        content: status.content,
                        content_hash,
                        view: status.view,
                        schema_id: status.schema_id,
                        ref_delta,
                    },
                ))
            }
        } else {
            ContentBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Stale {
                    content: status.content,
                    content_hash,
                    view: status.view,
                    schema_id: status.schema_id,
                },
            ))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Parsed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Parsing phase: file content has been parsed into a raw schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Parsed;

/// Proven: processor is on the missing path; no cached view exists.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Missing;

/// Proven: stale content parsed into a raw schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct ParsedMissing {
    raw: RawSchema,
    content_hash: Blake3Hash,
}

/// Proven: stale content (content-hash mismatch) parsed into a raw schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct ParsedStale {
    raw: RawSchema,
    content_hash: Blake3Hash,
    view: RawSchemaView,
    schema_id: SchemaId,
}

/// Proven: stale bank references (content matched) parsed into a raw schema.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct ParsedStaleReferences {
    raw: RawSchema,
    content_hash: Blake3Hash,
    view: RawSchemaView,
    schema_id: SchemaId,
    ref_delta: Vec<PropertyName>,
}

/// Missing-path parse: reads the file and parses it into a `ParsedMissing`.
///
/// This is Path A (missing): no cached view exists. The file must be read and
/// parsed to obtain property content. Transitions directly to `Construction`
/// via the `New` status returned by `run()`.
impl BaseSchemaProcessor<Parsed, Missing> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn parse(
        self,
        source: &FileReader,
    ) -> Result<BaseSchemaProcessor<Parsed, ParsedMissing>, SchemaLoaderError>
    {
        let (file, path_key, _) = self.into_parts();
        let content = source
            .read_to_string(file.path().as_path())
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let schema_name =
            SchemaName::try_from(file.path().basename().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::InvalidFileName {
                        path: file.path().as_path().to_path_buf(),
                        reason: "missing file stem".into(),
                    },
                ))
            })?)
            .map_err(SchemaLoaderError::Resolution)?;

        let raw: RawSchema = FileReader::parse_structured_from_str(
            file.path().as_path(),
            &content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw
            .with_name(schema_name.as_str().into())
            .with_metadata(file.metadata().clone());

        let content_hash = Blake3Hash::compute(content.as_bytes());

        Ok(Self::transition_from_parts(file, path_key, ParsedMissing {
            raw,
            content_hash,
        }))
    }
}

/// Stale-path parse: parses already-read content into a `ParsedStale`.
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

        let schema_name =
            SchemaName::try_from(file.path().basename().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::InvalidFileName {
                        path: file.path().as_path().to_path_buf(),
                        reason: "missing file stem".into(),
                    },
                ))
            })?)
            .map_err(SchemaLoaderError::Resolution)?;

        let raw: RawSchema = FileReader::parse_structured_from_str(
            file.path().as_path(),
            &status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw
            .with_name(schema_name.as_str().into())
            .with_metadata(file.metadata().clone());

        Ok(Self::transition_from_parts(file, path_key, ParsedStale {
            raw,
            content_hash: status.content_hash,
            view: status.view,
            schema_id: status.schema_id,
        }))
    }
}

/// Stale bank-reference parse: parses already-read content into a
/// `ParsedStaleReferences`.
impl BaseSchemaProcessor<Parsed, StaleReferences> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn parse(
        self,
    ) -> Result<
        BaseSchemaProcessor<Analysis, ParsedStaleReferences>,
        SchemaLoaderError,
    > {
        let (file, path_key, status) = self.into_parts();

        let schema_name =
            SchemaName::try_from(file.path().basename().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::InvalidFileName {
                        path: file.path().as_path().to_path_buf(),
                        reason: "missing file stem".into(),
                    },
                ))
            })?)
            .map_err(SchemaLoaderError::Resolution)?;

        let raw: RawSchema = FileReader::parse_structured_from_str(
            file.path().as_path(),
            &status.content,
        )
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        let raw = raw
            .with_name(schema_name.as_str().into())
            .with_metadata(file.metadata().clone());

        Ok(Self::transition_from_parts(file, path_key, ParsedStaleReferences {
            raw,
            content_hash: status.content_hash,
            view: status.view,
            schema_id: status.schema_id,
            ref_delta: status.ref_delta,
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
    fn analyze(
        self,
        bank: &PropertyBank,
    ) -> Result<AnalysisBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();

        let Some(version) = status.view.current() else {
            return Ok(AnalysisBranch::Corrupt(Self::transition_from_parts(
                file,
                path_key,
                New {
                    id: SchemaId::new(),
                    raw: status.raw,
                    content_hash: status.content_hash,
                },
            )));
        };

        let expander = RefExpander::new(bank);
        let property_delta = PropertyDeltaEngine::for_schema(
            &status.raw,
            version.hashes().properties(),
        )
        .diff_schema(&expander, &[])?;

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
                    schema_id: status.schema_id,
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
                    schema_id: status.schema_id,
                    content_hash: status.content_hash,
                    property_delta,
                    excludes_delta,
                    extends_delta,
                },
            )))
        }
    }
}

/// Stale bank-reference analysis: computes property deltas for the specific
/// changed bank references.
impl BaseSchemaProcessor<Analysis, ParsedStaleReferences> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn analyze(
        self,
        bank: &PropertyBank,
    ) -> Result<AnalysisBranch, SchemaLoaderError> {
        let (file, path_key, status) = self.into_parts();

        let Some(version) = status.view.current() else {
            return Ok(AnalysisBranch::Corrupt(Self::transition_from_parts(
                file,
                path_key,
                New {
                    id: SchemaId::new(),
                    raw: status.raw,
                    content_hash: status.content_hash,
                },
            )));
        };

        let expander = RefExpander::new(bank);
        let property_delta = match PropertyDeltaEngine::for_schema(
            &status.raw,
            version.hashes().properties(),
        )
        .diff_schema(&expander, &status.ref_delta)
        {
            Ok(delta) => delta,
            Err(SchemaLoaderError::Resolution(SchemaError::PropertyRef(
                PropertyRefError::NotFound {
                    ..
                },
            ))) => {
                // Structural conflict: bank target missing. Escalate to full
                // rebuild.
                return Ok(AnalysisBranch::Corrupt(
                    Self::transition_from_parts(file, path_key, New {
                        id: SchemaId::new(),
                        raw: status.raw,
                        content_hash: status.content_hash,
                    }),
                ));
            }
            Err(e) => return Err(e),
        };

        Ok(AnalysisBranch::Delta(Self::transition_from_parts(
            file,
            path_key,
            Changed {
                raw: status.raw,
                view: status.view,
                schema_id: status.schema_id,
                content_hash: status.content_hash,
                property_delta,
                excludes_delta: ExcludesDelta::default(),
                extends_delta: ExtendsDelta::default(),
            },
        )))
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
    schema_id: SchemaId,
}

/// Proven: content hash changed but semantic state did not.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct StaleContent {
    view: RawSchemaView,
    schema_id: SchemaId,
    content_hash: Blake3Hash,
}

impl BaseSchemaProcessor<Refresh, StaleTimestamps> {
    /// Syncs file timestamps to the cached view. Only the view is written;
    /// the `BaseSchema` aggregate is not touched.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn sync_metadata<R: WriteRepository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Construction, Fresh>, SchemaLoaderError>
    {
        let (file, path_key, mut status) = self.into_parts();
        status.view.update_metadata(file.metadata().clone());
        repository
            .save_raw_schema_view(status.schema_id, &status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Fresh {
            schema_id: status.schema_id,
        }))
    }
}

impl BaseSchemaProcessor<Refresh, StaleContent> {
    /// Syncs timestamps and content hash to the cached view. Only the view is
    /// written; the `BaseSchema` aggregate is not touched.
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
            .save_raw_schema_view(status.schema_id, &status.view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Fresh {
            schema_id: status.schema_id,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Building phase: terminal domain construction.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Construction;

/// Proven: missing path; carries parsed raw schema and content hash.
///
/// `SchemaId` is generated by the caller before constructing this state.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct New {
    id: SchemaId,
    raw: RawSchema,
    content_hash: Blake3Hash,
}

/// Proven: identity is fully synchronized; schema can be fetched without
/// rebuild.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "Builder integration deferred to Phase 3")
)]
struct Fresh {
    schema_id: SchemaId,
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
    schema_id: SchemaId,
    content_hash: Blake3Hash,
    property_delta: PropertyDelta,
    excludes_delta: ExcludesDelta,
    extends_delta: ExtendsDelta,
}

/// Construction operations that build the base schema from a new (missing)
/// file.
impl BaseSchemaProcessor<Construction, New> {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn create<R: WriteRepository>(
        self,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<BaseSchemaProcessor<Completed, NewReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();

        let expander = RefExpander::new(bank);
        let mut properties = expander
            .expand_properties(&status.raw.properties().ref_entries())
            .map_err(SchemaLoaderError::Resolution)?;
        let inline_entries = status.raw.properties().inline_entries();
        if !inline_entries.is_empty() {
            properties.extend(
                PropertyMap::try_from(inline_entries)
                    .map_err(SchemaLoaderError::Resolution)?,
            );
        }

        let schema_name = SchemaName::try_new(status.raw.name())
            .map_err(SchemaLoaderError::Resolution)?;

        let base = BaseSchema::new(
            status.id,
            schema_name,
            properties,
            status.raw.extends().to_vec(),
            status.raw.excludes().to_vec(),
        );

        let property_hashes = status.raw.properties().compute_hashes();
        let hashes =
            HashRecord::new(status.content_hash, property_hashes.into());
        let view = RawSchemaView::try_from_raw_with_hashes(
            &status.raw,
            path_key.clone(),
            hashes,
        )
        .map_err(SchemaLoaderError::Ingestion)?;

        repository
            .save_base_schema(&base)
            .map_err(SchemaLoaderError::Repository)?;
        repository
            .save_raw_schema_view(status.id, &view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, NewReady {
            base,
        }))
    }
}

/// Construction operations that fetch the cached base schema.
impl BaseSchemaProcessor<Construction, Fresh> {
    /// Retrieves the already-current schema from the repository.
    ///
    /// `schema_id` is already known from the carried state; no additional
    /// repository lookup is required.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Builder integration deferred to Phase 3")
    )]
    fn fetch<R: ReadRepository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Completed, FreshReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();
        let base = repository
            .find_base_schema_by_id(status.schema_id)?
            .ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundById(status.schema_id),
                )
            })?;

        Ok(Self::transition_from_parts(file, path_key, FreshReady {
            schema_id: status.schema_id,
            base,
        }))
    }
}

/// Construction operations that apply property deltas.
impl BaseSchemaProcessor<Construction, Changed> {
    /// Applies incremental schema updates via property deltas.
    ///
    /// Returns the terminal `Completed` state on success.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] if construction or repository access
    /// fails.
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
        let schema_id = status.schema_id;

        let property_hashes = status.raw.properties().compute_hashes();
        let hashes =
            HashRecord::new(status.content_hash, property_hashes.into());
        let version =
            SchemaVersion::new(file.metadata().clone(), hashes, &status.raw)
                .map_err(SchemaLoaderError::Ingestion)?;
        let mut updated_view = status.view;
        updated_view.add_version(version);

        // Retrieve existing schema to build updated property map. The bank is
        // not needed here: all ref-expansion happened in analyze() and any bank
        // delta augmentation happened in with_bank_delta_upserts(). The
        // property_delta already contains all required upserts.
        let existing_base =
            repository.find_base_schema_by_id(schema_id)?.ok_or_else(|| {
                SchemaLoaderError::Repository(
                    SchemaRepositoryError::NotFoundById(schema_id),
                )
            })?;

        // Apply upserts with ID preservation, then apply removals.
        let mut properties = existing_base.properties().clone();

        let mut property_delta = status.property_delta;
        let upserts_with_ids =
            property_delta.upserts().clone().with_ids(&properties);

        // Update the delta so it carries the preserved IDs into StaleReady
        property_delta = PropertyDelta::new(
            upserts_with_ids.clone(),
            property_delta.removals().to_vec(),
        );

        for (name, prop) in upserts_with_ids {
            properties.insert(name, prop);
        }
        for name in property_delta.removals() {
            properties.remove(name);
        }

        let schema_name = SchemaName::try_new(status.raw.name())
            .map_err(SchemaLoaderError::Resolution)?;
        let updated_base = BaseSchema::new(
            schema_id,
            schema_name,
            properties,
            status.raw.extends().to_vec(),
            status.raw.excludes().to_vec(),
        );

        repository
            .save_base_schema(&updated_base)
            .map_err(SchemaLoaderError::Repository)?;
        repository
            .save_raw_schema_view(schema_id, &updated_view)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, StaleReady {
            schema_id,
            base: updated_base,
            property_delta,
            excludes_delta: status.excludes_delta,
            extends_delta: status.extends_delta,
        }))
    }

    /// Folds bank delta re-expansions into this processor's `property_delta`,
    /// if any of the changed bank targets are referenced by this schema.
    ///
    /// Returns `Ok(self)` when augmentation succeeds or no augmentation is
    /// needed. Returns `Err(BaseSchemaResolution::New { .. })` when a
    /// structural conflict is detected (bank target missing), having emitted a
    /// diagnostic. The caller should propagate the `Err` as a resolution
    /// directly.
    #[expect(
        clippy::result_large_err,
        reason = "BaseSchemaResolution::Stale is intentionally large per \
                  issue 04; the Err variant is used only for early-exit \
                  escalation"
    )]
    fn with_bank_delta_upserts<R: Repository>(
        mut self,
        bank_resolution: Option<&PropertyBankResolution>,
        repository: &R,
    ) -> Result<Self, BaseSchemaResolution> {
        let changed_refs =
            relevant_bank_refs(&self.status.view, bank_resolution);
        if changed_refs.is_empty() {
            return Ok(self);
        }

        let Some(bank_res) = bank_resolution else {
            return Ok(self);
        };
        let expander = RefExpander::new(bank_res.bank());
        let ref_entries = self.status.raw.properties().ref_entries();
        let mut bank_props = PropertyMap::new();

        for prop_name in &changed_refs {
            let Some(entry) = ref_entries.get(prop_name) else {
                continue;
            };

            if let Ok(prop) = expander.expand_property(entry) {
                bank_props.insert(prop_name.clone(), prop);
            } else {
                tracing::warn!(
                    property = %prop_name,
                    "StaleReferences(analysis): bank target for '{}' \
                     absent; escalating to full rebuild",
                    prop_name
                );
                let (file, path_key, status) = self.into_parts();
                let schema_name_result = SchemaName::try_new(status.raw.name());
                let new_proc = BaseSchemaProcessor::<Construction, New>::transition_from_parts(
                    file,
                    path_key,
                    New {
                        id: SchemaId::new(),
                        raw: status.raw,
                        content_hash: status.content_hash,
                    },
                );
                let empty_bank = PropertyBank::new();
                // SAFETY: create() with empty bank only fails on repository
                // error or name error. We ignore those here and return a
                // minimal New resolution if it fails, matching previous
                // behavior.
                if let Ok(completed) = new_proc.create(repository, &empty_bank)
                {
                    return Err(BaseSchemaResolution::New {
                        base_schema: completed.into_base(),
                    });
                }
                let schema_name = schema_name_result.unwrap_or_else(|_| {
                    // SAFETY: "unknown" always satisfies SchemaName
                    // validation
                    #[expect(
                        clippy::unwrap_used,
                        reason = "'unknown' is a hardcoded literal that \
                                  always satisfies SchemaName validation"
                    )]
                    SchemaName::try_new("unknown").unwrap()
                });
                return Err(BaseSchemaResolution::New {
                    base_schema: BaseSchema::new(
                        SchemaId::new(),
                        schema_name,
                        PropertyMap::new(),
                        Vec::new(),
                        Vec::new(),
                    ),
                });
            }
        }

        if bank_props.is_empty() {
            return Ok(self);
        }

        // Merge bank upserts into the existing property delta.
        let existing_upserts = self.status.property_delta.upserts().clone();
        let existing_removals = self.status.property_delta.removals().to_vec();
        let mut merged_upserts = existing_upserts;
        for (name, prop) in bank_props {
            merged_upserts.insert(name, prop);
        }
        self.status.property_delta =
            PropertyDelta::new(merged_upserts, existing_removals);
        Ok(self)
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
    schema_id: SchemaId,
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
    fn into_fresh_parts(self) -> (SchemaId, BaseSchema) {
        (self.status.schema_id, self.status.base)
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

// ─────────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the subset of a schema's `$ref` property names whose bank
/// targets appear in the bank delta.
///
/// Returns an empty `Vec` when `bank_resolution` is `None`, the delta is
/// `None`, the delta is empty, or the schema has no current version.
/// The returned names are sorted for deterministic iteration order.
fn relevant_bank_refs(
    view: &RawSchemaView,
    bank_resolution: Option<&PropertyBankResolution>,
) -> Vec<PropertyName> {
    let bank_delta = bank_resolution.and_then(|r| r.delta());
    let mut refs: Vec<PropertyName> = bank_delta
        .filter(|d| !d.is_empty())
        .and_then(|d| view.current().map(|v| v.changed_bank_references(d)))
        .unwrap_or_default();
    refs.sort();
    refs
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::{DirPath, FsFile},
        schema::{
            bank::PropertyBank,
            property::{
                Multiplicity, Optionality, Property, PropertyId, PropertyMap,
                PropertyName,
            },
            property_bank_processor::PropertyBankResolution,
            property_spec::{PropertySpec, StringSpec},
            repository::{ReadRepository, WriteRepository},
            storage::testing::InMemoryRepository,
            views::RawView,
        },
    };

    // ─────────────────────────────────────────────────────────────────────────
    //  Macros
    // ─────────────────────────────────────────────────────────────────────────

    macro_rules! expect_new {
        ($resolution:expr) => {{
            let resolution = $resolution;
            let BaseSchemaResolution::New {
                base_schema,
            } = resolution
            else {
                panic!("Expected New resolution, got {resolution:?}");
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
                panic!("Expected Fresh resolution, got {resolution:?}");
            };
            (schema_id, base_schema)
        }};
    }

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
                panic!("Expected Stale resolution, got {:?}", $resolution);
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

    // ─────────────────────────────────────────────────────────────────────────
    //  Test fixtures
    // ─────────────────────────────────────────────────────────────────────────

    struct Fixture {
        repository: InMemoryRepository,
        source: FileReader,
        vault_root: DirPath,
        _vault_dir: TempDir,
        file: FsFile,
        key: PathKey,
        content: String,
    }

    fn make_fixture() -> Fixture {
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
        let file_path =
            crate::fs::FilePath::try_new(absolute.clone()).expect("file path");
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

    fn write_schema(fixture: &mut Fixture, content: &str) {
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

    fn parse_raw_schema(fixture: &Fixture, content: &str) -> RawSchema {
        FileReader::parse_structured_from_str::<RawSchema>(
            fixture.file.path().as_path(),
            content,
        )
        .expect("raw schema")
        .with_name("test-schema".into())
        .with_metadata(fixture.file.metadata().clone())
    }

    /// Create a view whose timestamps match the current file metadata.
    fn matching_view(fixture: &Fixture) -> RawSchemaView {
        matching_view_for_content(fixture, &fixture.content)
    }

    fn matching_view_for_content(
        fixture: &Fixture,
        content: &str,
    ) -> RawSchemaView {
        let raw = parse_raw_schema(fixture, content);
        let hash = Blake3Hash::compute(content.as_bytes());
        let hashes = crate::schema::views::hashes::HashRecord::new(
            hash,
            raw.properties().compute_hashes().into(),
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

    /// Create a view with timestamps one hour in the past (triggers mismatch).
    fn stale_view(fixture: &Fixture, content: &str) -> RawSchemaView {
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
        fixture: &Fixture,
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

    /// Build a `PropertyBank` containing a single `status` property of type
    /// string.
    fn bank_with_status() -> PropertyBank {
        use crate::schema::raw::RawPropertyBank;
        let content =
            r#"{"$version":"1.0","properties":{"status":{"type":"string"}}}"#;
        let raw = FileReader::parse_structured_from_str::<RawPropertyBank>(
            std::path::Path::new("property-bank.json"),
            content,
        )
        .expect("parse bank");
        PropertyBank::try_from(raw).expect("bank")
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: missing path
    // ─────────────────────────────────────────────────────────────────────────

    mod missing {
        use super::*;

        #[test]
        fn returns_new_resolution() {
            let fixture = make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository, None)
                .expect("run");

            assert!(matches!(resolution, BaseSchemaResolution::New { .. }));
        }

        #[test]
        fn derives_name_from_file_basename() {
            let fixture = make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository, None)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert_eq!(base_schema.name().as_str(), "test-schema");
        }

        #[test]
        fn constructs_with_empty_extends_and_excludes() {
            let fixture = make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository, None)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert!(base_schema.extends().is_empty());
            assert!(base_schema.excludes().is_empty());
        }

        #[test]
        fn persists_base_schema_to_repository() {
            let fixture = make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository, None)
                .expect("run");

            let base_schema = expect_new!(resolution);
            let stored = fixture
                .repository
                .find_base_schema_by_id(*base_schema.id())
                .expect("query")
                .expect("stored");
            assert_eq!(stored.name(), base_schema.name());
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: present → fresh path
    // ─────────────────────────────────────────────────────────────────────────

    mod fresh {
        use super::*;

        #[test]
        fn returns_fresh_when_timestamps_match() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            seed_base_and_view(&fixture, schema_id, &view);
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (sid, _) = expect_fresh!(resolution);
            assert_eq!(sid, schema_id);
        }

        #[test]
        fn returns_correct_base_schema_when_fresh() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            let base = seed_base_and_view(&fixture, schema_id, &view);
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (_, fetched) = expect_fresh!(resolution);
            assert_eq!(fetched, base);
        }

        #[test]
        fn does_not_write_to_repository_when_fresh() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            seed_base_and_view(&fixture, schema_id, &view);
            fixture.repository.harness().counters().reset();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let snapshot = fixture.repository.harness().counters().snapshot();
            assert_eq!(snapshot.writes, 0, "fresh path must not write");
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: normalization (stale timestamps / stale content)
    // ─────────────────────────────────────────────────────────────────────────

    mod normalization {
        use super::*;

        #[test]
        fn returns_fresh_when_content_matches_after_timestamp_mismatch() {
            let fixture = make_fixture();
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
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (fresh_id, _) = expect_fresh!(resolution);
            assert_eq!(fresh_id, schema_id);
        }

        #[test]
        fn updates_view_timestamps_when_normalizing_stale_timestamps() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, &fixture.content);
            seed_base_and_view(&fixture, schema_id, &view);
            let fixture_times = fixture.file.metadata().times().clone();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let saved = fixture
                .repository
                .get_raw_schema_view(schema_id)
                .expect("get view")
                .expect("view");
            assert!(saved.is_timestamp_match(
                fixture_times.created_at(),
                fixture_times.modified_at(),
            ));
        }

        #[test]
        fn does_not_write_base_schema_when_normalizing_stale_timestamps() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, &fixture.content);
            seed_base_and_view(&fixture, schema_id, &view);
            fixture.repository.harness().counters().reset();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let snapshot = fixture.repository.harness().counters().snapshot();
            assert_eq!(
                snapshot.writes, 2,
                "only view write expected (schema_id path + view)"
            );
        }

        #[test]
        fn returns_fresh_when_semantic_state_is_unchanged_after_content_mismatch()
         {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(&mut fixture, "properties: {}\n");
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (fresh_id, _) = expect_fresh!(resolution);
            assert_eq!(fresh_id, schema_id);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: analysis (stale content with semantic changes)
    // ─────────────────────────────────────────────────────────────────────────

    mod analysis {
        use super::*;

        #[test]
        fn returns_stale_when_property_delta_detected() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(
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
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (_, _, property_delta, _, _) = expect_stale!(resolution);
            assert!(!property_delta.is_empty());
        }

        #[test]
        fn reuses_schema_id_when_returning_stale() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(
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
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (stale_id, base, _, _, _) = expect_stale!(resolution);
            assert_eq!(stale_id, schema_id);
            assert_eq!(base.id(), &schema_id);
        }

        #[test]
        fn returns_stale_when_extends_delta_detected() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(&mut fixture, "extends: parent\nproperties: {}\n");
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (_, _, _, _, extends_delta) = expect_stale!(resolution);
            assert!(!extends_delta.is_empty());
        }

        #[test]
        fn returns_stale_when_excludes_delta_detected() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(
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
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (_, _, _, excludes_delta, _) = expect_stale!(resolution);
            assert!(!excludes_delta.is_empty());
        }

        #[test]
        fn appends_view_version_when_returning_stale() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(
                &mut fixture,
                "properties:\n  title:\n    type: string\n",
            );
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let saved = fixture
                .repository
                .get_raw_schema_view(schema_id)
                .expect("get view")
                .expect("view");
            assert_eq!(saved.version_count(), view.version_count() + 1);
        }

        #[test]
        fn returns_error_when_ref_property_not_found_in_bank() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(
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

            // Empty bank: $ref cannot be resolved → resolution error.
            let result = processor.run(
                Some(&view),
                &fixture.source,
                &fixture.repository,
                None,
            );

            assert!(result.is_err(), "Expected error for unresolvable $ref");
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: fallback (corrupt view)
    // ─────────────────────────────────────────────────────────────────────────

    mod fallback {
        use super::*;

        #[test]
        fn returns_new_when_view_has_no_current_version() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, &fixture.content);
            seed_base_and_view(&fixture, schema_id, &view);
            let corrupt_view =
                RawSchemaView::empty_for_test(fixture.key.clone());
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
                    None,
                )
                .expect("run");

            assert!(matches!(resolution, BaseSchemaResolution::New { .. }));
        }

        #[test]
        fn returns_error_when_parse_fails() {
            let mut fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, "properties: {}");
            seed_base_and_view(&fixture, schema_id, &view);
            write_schema(&mut fixture, "properties: [");
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
                None,
            );

            assert!(result.is_err(), "Expected parse error");
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: StaleReferences
    // ─────────────────────────────────────────────────────────────────────────

    mod stale_references {
        use super::*;

        #[test]
        fn skips_file_read_when_bank_delta_is_none() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            seed_base_and_view(&fixture, schema_id, &view);
            // bank_resolution = None → no StaleReferences check
            let bank = PropertyBank::new();
            let resolution = PropertyBankResolution::new(bank, None);
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (fresh_id, _) = expect_fresh!(result);
            assert_eq!(fresh_id, schema_id);
        }

        #[test]
        fn skips_file_read_when_bank_delta_is_empty() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            seed_base_and_view(&fixture, schema_id, &view);
            let bank = PropertyBank::new();
            let resolution =
                PropertyBankResolution::new(bank, Some(HashSet::new()));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (fresh_id, _) = expect_fresh!(result);
            assert_eq!(fresh_id, schema_id);
        }

        #[test]
        fn skips_file_read_when_no_referencing_properties_exist() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            // Schema has no $ref properties → no re-expansion even with a delta
            let view = matching_view(&fixture);
            seed_base_and_view(&fixture, schema_id, &view);
            let bank = bank_with_status();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (fresh_id, _) = expect_fresh!(result);
            assert_eq!(fresh_id, schema_id);
        }

        #[test]
        fn returns_stale_with_relevant_upserts_when_fresh_and_bank_changed() {
            let mut fixture = make_fixture();
            let content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
"##;
            write_schema(&mut fixture, content);
            let schema_id = SchemaId::new();
            let view = matching_view_for_content(&fixture, content);
            seed_base_and_view(&fixture, schema_id, &view);
            let bank = bank_with_status();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (_, _, property_delta, _, _) = expect_stale!(result);
            assert!(
                property_delta.contains_upsert(
                    &PropertyName::try_new("my_prop").expect("my_prop")
                ),
                "property_delta should contain my_prop from bank re-expansion"
            );
        }

        #[test]
        fn preserves_unaffected_property_ids_via_with_ids() {
            let mut fixture = make_fixture();
            let content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
  inline_prop:
    type: string
"##;
            write_schema(&mut fixture, content);
            let schema_id = SchemaId::new();
            let view = matching_view_for_content(&fixture, content);
            let inline_id = PropertyId::new();
            let ref_id = PropertyId::new();
            let bank = bank_with_status();

            let mut existing_props = PropertyMap::new();
            existing_props.insert(
                PropertyName::try_new("my_prop").expect("my_prop"),
                Property::new(
                    ref_id,
                    Optionality::Required,
                    Multiplicity::Single,
                    PropertySpec::String(StringSpec::default()),
                ),
            );
            existing_props.insert(
                PropertyName::try_new("inline_prop").expect("inline"),
                Property::new(
                    inline_id,
                    Optionality::Optional,
                    Multiplicity::Single,
                    PropertySpec::String(StringSpec::default()),
                ),
            );
            let base = BaseSchema::new(
                schema_id,
                SchemaName::try_new("test-schema").expect("name"),
                existing_props,
                Vec::new(),
                Vec::new(),
            );
            fixture.repository.save_base_schema(&base).expect("save");
            fixture
                .repository
                .save_raw_schema_view(schema_id, &view)
                .expect("save view");

            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (_, base_schema, _, _, _) = expect_stale!(result);
            assert_eq!(
                base_schema
                    .properties()
                    .get(&PropertyName::try_new("inline_prop").expect("inline"))
                    .map(Property::id),
                Some(inline_id),
                "inline property ID should be preserved"
            );
            assert_eq!(
                base_schema
                    .properties()
                    .get(&PropertyName::try_new("my_prop").expect("my_prop"))
                    .map(Property::id),
                Some(ref_id),
                "re-expanded reference property ID should be preserved"
            );
        }

        #[test]
        fn escalates_to_full_rebuild_when_bank_target_missing_fresh_path() {
            let mut fixture = make_fixture();
            let content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
"##;
            write_schema(&mut fixture, content);
            let schema_id = SchemaId::new();
            let view = matching_view_for_content(&fixture, content);
            seed_base_and_view(&fixture, schema_id, &view);
            // Empty bank: "status" does not exist → structural conflict
            let bank = PropertyBank::new();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            assert!(
                matches!(result, BaseSchemaResolution::New { .. }),
                "Expected New (full rebuild) when bank target missing on \
                 fresh path"
            );
        }

        #[test]
        fn analysis_path_folds_bank_delta_into_property_delta() {
            let mut fixture = make_fixture();
            let old_content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
"##;
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, old_content);
            seed_base_and_view(&fixture, schema_id, &view);
            let new_content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
  new_title:
    type: string
"##;
            write_schema(&mut fixture, new_content);
            let bank = bank_with_status();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (_, _, property_delta, _, _) = expect_stale!(result);
            assert!(
                property_delta.contains_upsert(
                    &PropertyName::try_new("my_prop").expect("my_prop")
                ),
                "property_delta should contain my_prop from bank delta"
            );
            assert!(
                property_delta.contains_upsert(
                    &PropertyName::try_new("new_title").expect("title")
                ),
                "property_delta should contain new_title from content diff"
            );
        }

        #[test]
        fn analysis_path_ignores_bank_delta_when_no_refs_match() {
            let mut fixture = make_fixture();
            let old_content = "properties:\n  title:\n    type: string\n";
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, old_content);
            seed_base_and_view(&fixture, schema_id, &view);
            let new_content = "properties:\n  title:\n    type: string\n  \
                               body:\n    type: string\n";
            write_schema(&mut fixture, new_content);
            let bank = bank_with_status();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            let (_, _, property_delta, _, _) = expect_stale!(result);
            assert!(
                property_delta.contains_upsert(
                    &PropertyName::try_new("body").expect("body")
                ),
                "should contain body from content diff"
            );
            assert!(
                !property_delta.contains_upsert(
                    &PropertyName::try_new("status").expect("status")
                ),
                "should NOT contain status (no ref in schema)"
            );
        }

        #[test]
        fn analysis_path_escalates_to_full_rebuild_when_bank_target_missing() {
            let mut fixture = make_fixture();
            let old_content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
"##;
            let schema_id = SchemaId::new();
            let view = stale_view(&fixture, old_content);
            seed_base_and_view(&fixture, schema_id, &view);
            // New content still has the ref — triggers analysis path
            let new_content = r##"properties:
  my_prop:
    $ref: "#property_bank/status"
    required: true
  body:
    type: string
"##;
            write_schema(&mut fixture, new_content);
            // Bank does NOT contain "status" — structural conflict
            let bank = PropertyBank::new();
            let mut delta = HashSet::new();
            delta.insert(PropertyName::try_new("status").expect("status"));
            let resolution = PropertyBankResolution::new(bank, Some(delta));
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let result = processor
                .run(
                    Some(&view),
                    &fixture.source,
                    &fixture.repository,
                    Some(&resolution),
                )
                .expect("run");

            assert!(
                matches!(result, BaseSchemaResolution::New { .. }),
                "Expected New (full rebuild) when bank target missing on \
                 analysis path, got {result:?}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Tests: terminal extractors
    // ─────────────────────────────────────────────────────────────────────────

    mod terminal {
        use super::*;

        #[test]
        fn new_resolution_carries_constructed_base_schema() {
            let fixture = make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository, None)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert_eq!(base_schema.name().as_str(), "test-schema");
        }

        #[test]
        fn fresh_resolution_carries_cached_base_schema() {
            let fixture = make_fixture();
            let schema_id = SchemaId::new();
            let view = matching_view(&fixture);
            let expected = seed_base_and_view(&fixture, schema_id, &view);
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository, None)
                .expect("run");

            let (_, base_schema) = expect_fresh!(resolution);
            assert_eq!(base_schema, expected);
        }
    }
}
