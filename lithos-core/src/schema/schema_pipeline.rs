//! Single-state-machine schema ingestion pipeline.
//!
//! # Architecture
//!
//! This module implements a single batch processor with per-file status enums
//! to coordinate:
//!
//! ```text
//! Discovery -> Comparison -> TreeGraphed -> PropertyAnalysis -> Refresh
//! -> Construction -> Completion
//! ```
//!
//! Per-file staleness checks are handled in `Comparison`, while graph patching
//! and structural validation happen in `TreeGraphed` (fail fast). Property and
//! bank-reference deltas are computed in `PropertyAnalysis`. Metadata refreshes
//! happen in `Refresh`, and final schema construction + persistence complete
//! the pipeline.

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        bank::PropertyBank,
        error::{
            SchemaFileError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError, SchemaStorageError,
        },
        expander::RefExpander,
        graph::{DagBuilder, InheritanceNode, TopologicalGraph},
        property::{Property, PropertyName},
        raw::{
            RawFileTimes, RawProperty, RawPropertyInline, RawPropertyRef,
            RawSchema,
        },
        storage::Repository,
        views::{
            FileTimesMetadata, Filename, HashMetadata, RawSchemaView,
            SchemaVersion,
        },
    },
};

const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

// ═════════════════════════════════════════════════════════════════════════════
//  PROCESSOR TYPE
// ═════════════════════════════════════════════════════════════════════════════

/// Global state machine for schema ingestion.
#[derive(Debug)]
#[must_use]
pub(crate) struct SchemaTreeProcessor<Stage, Status> {
    status: Status,
    _stage: PhantomData<Stage>,
}

impl<S, T> SchemaTreeProcessor<S, T> {
    #[inline]
    #[expect(
        clippy::unused_self,
        reason = "transition consumes self for typestate ergonomics"
    )]
    fn transition<NS, NT>(self, status: NT) -> SchemaTreeProcessor<NS, NT> {
        SchemaTreeProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGES + STATUS TYPES
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct Discovery;

#[derive(Debug)]
pub(crate) struct Comparison;

#[derive(Debug)]
pub(crate) struct TreeGraphed;

#[derive(Debug)]
pub(crate) struct PropertyAnalysis;

#[derive(Debug)]
pub(crate) struct Refresh;

#[derive(Debug)]
pub(crate) struct Construction;

#[derive(Debug)]
pub(crate) struct Completion;

/// Entry marker before any scanning.
#[derive(Debug)]
pub(crate) struct Unknown;

#[derive(Debug)]
pub(crate) struct DiscoveryState {
    files: Vec<PathBuf>,
    id_by_path: HashMap<PathBuf, SchemaId>,
    view_by_id: HashMap<SchemaId, RawSchemaView>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct ComparisonState {
    statuses: HashMap<SchemaId, FileStatus>,
    #[expect(dead_code, reason = "retained for future refresh optimizations")]
    fresh_ids: Vec<SchemaId>,
    #[expect(dead_code, reason = "retained for future refresh optimizations")]
    stale_ids: Vec<SchemaId>,
    new_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct TreeGraphedState {
    graph: PipelineGraph,
    statuses: HashMap<SchemaId, FileStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,
    affected_subtrees: HashSet<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct PropertyAnalysisState {
    graph: PipelineGraph,
    statuses: HashMap<SchemaId, FileStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    #[expect(dead_code, reason = "retained for future construction stages")]
    deltas_by_id: HashMap<SchemaId, SchemaPropertyDelta>,
    #[expect(dead_code, reason = "retained for future construction stages")]
    excludes_by_id: HashMap<SchemaId, ExcludesDelta>,
    rebuild_ids: HashSet<SchemaId>,
    #[expect(dead_code, reason = "retained for future cleanup stage")]
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct ConstructionState {
    graph: PipelineGraph,
    statuses: HashMap<SchemaId, FileStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    rebuild_ids: HashSet<SchemaId>,
    #[expect(dead_code, reason = "retained for future persistence stage")]
    changed_ids: HashSet<SchemaId>,
    schemas: Vec<Arc<Schema>>,
}

#[derive(Debug)]
pub(crate) struct CompletionState {
    schemas: Vec<Arc<Schema>>,
    graph: PipelineGraph,
    changed_ids: HashSet<SchemaId>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  FILE STATUS
// ═════════════════════════════════════════════════════════════════════════════

#[expect(dead_code, reason = "retained for future diagnostics")]
#[derive(Debug, Clone)]
pub(crate) enum FileStatus {
    Fresh {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
    },
    StaleTimestamps {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
        times: RawFileTimes,
    },
    StaleContent {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
        raw: RawSchema,
        content_hash: [u8; 32],
        times: RawFileTimes,
    },
    New {
        id: SchemaId,
        path: PathBuf,
        raw: RawSchema,
        content_hash: [u8; 32],
        times: RawFileTimes,
    },
}

impl FileStatus {
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics for &FileStatus"
    )]
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Fresh {
                path,
                ..
            }
            | Self::StaleTimestamps {
                path,
                ..
            }
            | Self::StaleContent {
                path,
                ..
            }
            | Self::New {
                path,
                ..
            } => path.as_path(),
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics for &FileStatus"
    )]
    pub(crate) fn view(&self) -> Option<&RawSchemaView> {
        match self {
            Self::Fresh {
                view,
                ..
            }
            | Self::StaleTimestamps {
                view,
                ..
            }
            | Self::StaleContent {
                view,
                ..
            } => Some(view),
            Self::New {
                ..
            } => None,
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics for &FileStatus"
    )]
    pub(crate) fn raw(&self) -> Option<&RawSchema> {
        match self {
            Self::StaleContent {
                raw,
                ..
            }
            | Self::New {
                raw,
                ..
            } => Some(raw),
            Self::Fresh {
                ..
            }
            | Self::StaleTimestamps {
                ..
            } => None,
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics for &FileStatus"
    )]
    pub(crate) fn content_hash(&self) -> Option<[u8; 32]> {
        match self {
            Self::StaleContent {
                content_hash,
                ..
            }
            | Self::New {
                content_hash,
                ..
            } => Some(*content_hash),
            Self::Fresh {
                ..
            }
            | Self::StaleTimestamps {
                ..
            } => None,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DELTAS + SUPPORT STRUCTURES
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExtendsDelta {
    old_parent: Option<SchemaName>,
    new_parent: Option<SchemaName>,
}

impl ExtendsDelta {
    #[inline]
    pub(crate) fn changed(&self) -> bool {
        self.old_parent != self.new_parent
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ExcludesDelta {
    added: Vec<PropertyName>,
    removed: Vec<PropertyName>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyDelta {
    upserts: SchemaPropertyUpserts,
    removed: Vec<PropertyName>,
}

impl SchemaPropertyDelta {
    #[inline]
    fn is_empty(&self) -> bool {
        self.upserts.inline.is_empty()
            && self.upserts.refs.is_empty()
            && self.removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyUpserts {
    inline: HashMap<PropertyName, RawPropertyInline>,
    refs: HashMap<PropertyName, RawPropertyRef>,
}

pub(crate) type PipelineGraph = TopologicalGraph<InheritanceNode>;

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: DISCOVERY
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<Discovery, Unknown> {
    #[inline]
    pub(crate) fn new() -> Self {
        SchemaTreeProcessor {
            status: Unknown,
            _stage: PhantomData,
        }
    }

    /// Discovers schema files, loads cached graph, and deletes missing schemas.
    #[expect(
        clippy::iter_over_hash_type,
        reason = "iteration order irrelevant for view hydration"
    )]
    pub(crate) fn discover_tree<R: Repository>(
        self,
        source: &FsReader,
        repository: &R,
        schemas_dir: &Path,
        property_bank_filename: &str,
    ) -> Result<
        SchemaTreeProcessor<Comparison, DiscoveryState>,
        SchemaLoaderError,
    >
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let pattern = format!("{}/**/*", schemas_dir.display());
        let all_files = source.list_files(&pattern).map_err(|e| {
            SchemaIngestionError::File(SchemaFileError::Io {
                path: schemas_dir.to_path_buf(),
                source: std::io::Error::other(e),
            })
        })?;

        let files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|path| {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                else {
                    return false;
                };
                if file_name == property_bank_filename {
                    return false;
                }
                let Some(ext) = path.extension().and_then(|e| e.to_str())
                else {
                    return false;
                };
                SCHEMA_EXTENSIONS
                    .iter()
                    .any(|allowed| ext.eq_ignore_ascii_case(allowed))
            })
            .collect();

        let id_by_path = repository
            .find_schema_ids_by_paths(&files)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let views_by_path =
            repository
                .find_raw_schema_views_by_paths(&files)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let mut view_by_id = HashMap::new();
        for (path, view) in views_by_path {
            if let Some(id) = id_by_path.get(&path) {
                view_by_id.insert(*id, view);
            }
        }

        let db_pairs = repository
            .list_schema_path_id_pairs()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        let file_set: HashSet<PathBuf> = files.iter().cloned().collect();

        let mut deleted_ids = Vec::new();
        for (path, id) in db_pairs {
            if !file_set.contains(&path) {
                deleted_ids.push(id);
            }
        }

        if !deleted_ids.is_empty() {
            for id in &deleted_ids {
                repository
                    .delete_schema(*id)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            }
        }

        Ok(self.transition(DiscoveryState {
            files,
            id_by_path,
            view_by_id,
            deleted_ids,
        }))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: COMPARISON
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<Comparison, DiscoveryState> {
    /// Performs timestamp + content hash checks per file.
    #[expect(
        clippy::excessive_nesting,
        reason = "state machine branches are explicit for readability"
    )]
    pub(crate) fn compare_files(
        self,
        source: &FsReader,
    ) -> Result<
        SchemaTreeProcessor<TreeGraphed, ComparisonState>,
        SchemaLoaderError,
    > {
        let SchemaTreeProcessor {
            status:
                DiscoveryState {
                    files,
                    id_by_path,
                    view_by_id,
                    deleted_ids,
                },
            ..
        } = self;

        let mut statuses = HashMap::new();
        let mut fresh_ids = Vec::new();
        let mut stale_ids = Vec::new();
        let mut new_ids = Vec::new();

        for path in &files {
            let id =
                id_by_path.get(path).copied().unwrap_or_else(SchemaId::new);

            let times = RawFileTimes {
                created_at: source.created_at(path),
                modified_at: source.modified_at(path),
            };

            let status = if let Some(view) = view_by_id.get(&id) {
                let timestamps_match = view.current().is_some_and(|v| {
                    v.file_times()
                        .is_timestamp_match(times.created_at, times.modified_at)
                });

                if timestamps_match {
                    fresh_ids.push(id);
                    FileStatus::Fresh {
                        id,
                        path: path.clone(),
                        view: view.clone(),
                    }
                } else {
                    let content = source
                        .read_to_string(path)
                        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                    let content_hash =
                        *blake3::hash(content.as_bytes()).as_bytes();

                    let content_match = view.current().is_some_and(|v| {
                        v.hashes().is_content_match(&content_hash)
                    });

                    if content_match {
                        stale_ids.push(id);
                        FileStatus::StaleTimestamps {
                            id,
                            path: path.clone(),
                            view: view.clone(),
                            times,
                        }
                    } else {
                        let raw =
                            parse_raw_schema_from_str(path, &content, &times)?;
                        stale_ids.push(id);
                        FileStatus::StaleContent {
                            id,
                            path: path.clone(),
                            view: view.clone(),
                            raw,
                            content_hash,
                            times,
                        }
                    }
                }
            } else {
                let content = source
                    .read_to_string(path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                let content_hash = *blake3::hash(content.as_bytes()).as_bytes();
                let raw = parse_raw_schema_from_str(path, &content, &times)?;
                new_ids.push(id);
                FileStatus::New {
                    id,
                    path: path.clone(),
                    raw,
                    content_hash,
                    times,
                }
            };

            statuses.insert(id, status);
        }

        Ok(SchemaTreeProcessor {
            status: ComparisonState {
                statuses,
                fresh_ids,
                stale_ids,
                new_ids,
                deleted_ids,
            },
            _stage: PhantomData,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: TREE GRAPHED
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<TreeGraphed, ComparisonState> {
    /// Patch or build the inheritance graph and fail fast on structural issues.
    #[expect(
        clippy::iter_over_hash_type,
        reason = "iteration order irrelevant for graph hydration"
    )]
    pub(crate) fn graph_structure(
        self,
    ) -> Result<
        SchemaTreeProcessor<PropertyAnalysis, TreeGraphedState>,
        SchemaLoaderError,
    > {
        let SchemaTreeProcessor {
            status:
                ComparisonState {
                    statuses,
                    new_ids,
                    deleted_ids,
                    ..
                },
            ..
        } = self;

        let mut raw_by_id = HashMap::new();
        let mut extends_deltas = HashMap::new();

        for (id, status) in &statuses {
            if let Some(raw) = status.raw() {
                raw_by_id.insert(*id, raw.clone());
                let old_parent =
                    status.view().and_then(|view| view.extends().cloned());
                let new_parent = raw.extends().cloned();
                extends_deltas.insert(*id, ExtendsDelta {
                    old_parent,
                    new_parent,
                });
            }
        }

        let graph = DagBuilder::new(&statuses).build()?;

        let mut changed_extends_ids = HashSet::new();
        for (id, delta) in &extends_deltas {
            if delta.changed() {
                changed_extends_ids.insert(*id);
            }
        }

        let mut seed_ids: HashSet<SchemaId> = changed_extends_ids;
        seed_ids.extend(new_ids.iter().copied());

        let affected_subtrees = if seed_ids.is_empty() {
            HashSet::new()
        } else {
            graph.affected_subtree(&seed_ids)
        };

        Ok(SchemaTreeProcessor {
            status: TreeGraphedState {
                graph,
                statuses,
                raw_by_id,
                extends_deltas,
                affected_subtrees,
                deleted_ids,
            },
            _stage: PhantomData,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: PROPERTY ANALYSIS
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<PropertyAnalysis, TreeGraphedState> {
    /// Compute excludes/property deltas and incorporate property bank changes.
    #[expect(
        clippy::excessive_nesting,
        reason = "nested deltas mirror processing stages"
    )]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "iteration order irrelevant for delta computation"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "pipeline stage kept together for readability"
    )]
    pub(crate) fn analyze_properties(
        self,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<
        SchemaTreeProcessor<Refresh, PropertyAnalysisState>,
        SchemaLoaderError,
    > {
        let SchemaTreeProcessor {
            status:
                TreeGraphedState {
                    graph,
                    statuses,
                    raw_by_id,
                    extends_deltas,
                    affected_subtrees,
                    deleted_ids,
                },
            ..
        } = self;

        let mut deltas_by_id = HashMap::new();
        let mut excludes_by_id = HashMap::new();
        let mut raw_by_id = raw_by_id;
        let mut rebuild_ids = affected_subtrees.clone();

        for (id, status) in &statuses {
            let view = status.view();
            let has_raw = status.raw().is_some();

            if has_raw {
                let raw = status.raw().ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                        SchemaStorageError::NotFound {
                            name: status.path().to_string_lossy().into(),
                        },
                    ))
                })?;
                let (excludes, properties) = if let Some(view) = view {
                    compute_deltas_from_cached(raw, view)
                } else {
                    let excludes = diff_excludes(&[], raw.excludes());
                    let properties = property_delta_from_new(raw);
                    (excludes, properties)
                };
                if !properties.is_empty()
                    || !excludes.added.is_empty()
                    || !excludes.removed.is_empty()
                {
                    rebuild_ids.insert(*id);
                    deltas_by_id.insert(*id, properties);
                    excludes_by_id.insert(*id, excludes);
                }
            }

            if let (Some(view), Some(bank_delta)) = (view, property_bank_delta)
            {
                let current = view.current();
                if let Some(version) = current {
                    let mut is_affected = false;
                    for bank_prop in version.bank_references().values() {
                        if bank_delta.contains(bank_prop) {
                            is_affected = true;
                            break;
                        }
                    }

                    if is_affected {
                        rebuild_ids.insert(*id);
                        deltas_by_id.entry(*id).or_insert_with(|| {
                            SchemaPropertyDelta {
                                upserts: SchemaPropertyUpserts::default(),
                                removed: Vec::new(),
                            }
                        });
                        excludes_by_id
                            .entry(*id)
                            .or_insert_with(ExcludesDelta::default);
                    }
                }
            }

            if rebuild_ids.contains(id) && !raw_by_id.contains_key(id) {
                let raw = status
                    .view()
                    .and_then(|raw_view| raw_view.to_raw().ok().flatten())
                    .ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::Storage(
                                SchemaStorageError::NotFound {
                                    name: status
                                        .path()
                                        .to_string_lossy()
                                        .into(),
                                },
                            ),
                        )
                    })?;
                raw_by_id.insert(*id, raw);
            }

            if let Some(delta) = extends_deltas.get(id)
                && delta.changed()
            {
                rebuild_ids.insert(*id);
            }
        }

        Ok(SchemaTreeProcessor {
            status: PropertyAnalysisState {
                graph,
                statuses,
                raw_by_id,
                deltas_by_id,
                excludes_by_id,
                rebuild_ids,
                deleted_ids,
            },
            _stage: PhantomData,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: REFRESH
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<Refresh, PropertyAnalysisState> {
    /// Refreshes metadata for stale timestamps or stale content-only changes.
    #[expect(
        clippy::excessive_nesting,
        reason = "refresh branches are explicit for error context"
    )]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "iteration order irrelevant for refresh"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "refresh stage kept together for readability"
    )]
    pub(crate) fn refresh_metadata<R: Repository>(
        self,
        repository: &R,
    ) -> Result<
        SchemaTreeProcessor<Construction, ConstructionState>,
        SchemaLoaderError,
    >
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let SchemaTreeProcessor {
            status:
                PropertyAnalysisState {
                    graph,
                    statuses,
                    raw_by_id,
                    rebuild_ids,
                    ..
                },
            ..
        } = self;

        let mut refreshed_statuses = HashMap::new();

        for (id, status) in statuses {
            let refreshed = match status {
                FileStatus::StaleTimestamps {
                    path,
                    mut view,
                    times,
                    ..
                } => {
                    let raw = view
                        .to_raw()
                        .map_err(SchemaLoaderError::Ingestion)?
                        .ok_or_else(|| {
                            SchemaLoaderError::Ingestion(
                                SchemaIngestionError::Storage(
                                    SchemaStorageError::NotFound {
                                        name: path.to_string_lossy().into(),
                                    },
                                ),
                            )
                        })?;
                    let current = view.current().ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::Storage(
                                SchemaStorageError::NotFound {
                                    name: path.to_string_lossy().into(),
                                },
                            ),
                        )
                    })?;
                    let file_times = FileTimesMetadata::new(
                        times.created_at,
                        times.modified_at,
                    );
                    let hashes = HashMetadata::new(
                        *current.hashes().content(),
                        current.hashes().properties().clone(),
                    );
                    let version = SchemaVersion::new(file_times, hashes, &raw)
                        .map_err(SchemaLoaderError::Ingestion)?;
                    view.add_version(version);
                    repository
                        .save_raw_schema_view(id, &view)
                        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
                    FileStatus::Fresh {
                        id,
                        path,
                        view,
                    }
                }
                FileStatus::StaleContent {
                    path,
                    view,
                    raw,
                    content_hash,
                    times,
                    ..
                } => {
                    if rebuild_ids.contains(&id) {
                        FileStatus::StaleContent {
                            id,
                            path,
                            view,
                            raw,
                            content_hash,
                            times,
                        }
                    } else {
                        let new_view =
                            build_view_from_raw(&raw, &path, content_hash)?;
                        repository
                            .save_raw_schema_view(id, &new_view)
                            .map_err(|e| {
                                SchemaLoaderError::Repository(e.into())
                            })?;
                        FileStatus::Fresh {
                            id,
                            path,
                            view: new_view.clone(),
                        }
                    }
                }
                status @ (FileStatus::Fresh {
                    ..
                }
                | FileStatus::New {
                    ..
                }) => status,
            };

            refreshed_statuses.insert(id, refreshed);
        }

        Ok(SchemaTreeProcessor {
            status: ConstructionState {
                graph,
                statuses: refreshed_statuses,
                raw_by_id,
                rebuild_ids,
                changed_ids: HashSet::new(),
                schemas: Vec::new(),
            },
            _stage: PhantomData,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: CONSTRUCTION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<Construction, ConstructionState> {
    /// Build resolved schemas via ref expansion and inheritance merge.
    #[expect(
        clippy::iter_over_hash_type,
        reason = "iteration order irrelevant for construction"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "construction stage kept together for readability"
    )]
    pub(crate) fn construct_schemas<R: Repository>(
        self,
        repository: &R,
        bank: &PropertyBank,
    ) -> Result<
        SchemaTreeProcessor<Completion, CompletionState>,
        SchemaLoaderError,
    >
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let SchemaTreeProcessor {
            mut status,
            ..
        } = self;
        let mut changed_ids = HashSet::new();
        let rebuild_ids = &status.rebuild_ids;

        let expanded_by_id = if rebuild_ids.is_empty() {
            HashMap::new()
        } else {
            let raw_vec: Vec<(SchemaId, RawSchema)> = rebuild_ids
                .iter()
                .filter_map(|id| {
                    status.raw_by_id.get(id).map(|raw| (*id, raw.clone()))
                })
                .collect();
            let expanded = RefExpander::new(bank)
                .expand_all(raw_vec)
                .map_err(SchemaLoaderError::Resolution)?;
            expanded.into_iter().collect()
        };

        let mut parent_ids = HashSet::new();
        for node in status.graph.nodes.values() {
            for parent_id in &node.parents {
                if !status.graph.nodes.contains_key(parent_id) {
                    parent_ids.insert(*parent_id);
                }
            }
        }

        let mut fetch_ids: Vec<SchemaId> = parent_ids.into_iter().collect();
        for id in &status.graph.order {
            if !rebuild_ids.contains(id) {
                fetch_ids.push(*id);
            }
        }

        let fetched = repository
            .find_schemas_by_ids(&fetch_ids)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        let mut fetched_by_id: HashMap<SchemaId, Schema> = HashMap::new();
        for schema in fetched {
            fetched_by_id.insert(*schema.id(), schema);
        }

        let mut resolved_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in &status.graph.order {
            let node = status
                .graph
                .nodes
                .get(id)
                .ok_or(SchemaLoaderError::Resolution(
                crate::schema::error::SchemaError::Resolution(
                    crate::schema::error::SchemaResolutionError::MissingNode {
                        id: *id,
                    },
                ),
            ))?;

            let status_entry = status.statuses.get(id);
            let status_name: Box<str> = status_entry.map_or_else(
                || id.to_string().into(),
                |s| s.path().to_string_lossy().into(),
            );

            let children = node.children.clone();

            if !rebuild_ids.contains(id) {
                let schema = fetched_by_id.get(id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                        SchemaStorageError::NotFound {
                            name: status_name.clone(),
                        },
                    ))
                })?;
                resolved_cache.insert(*id, schema.clone());
                status.schemas.push(Arc::new(schema.clone()));
                continue;
            }

            changed_ids.insert(*id);

            let expanded = expanded_by_id.get(id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                    SchemaStorageError::NotFound {
                        name: status_name.clone(),
                    },
                ))
            })?;

            let parent_props = if node.parents.is_empty() {
                HashMap::new()
            } else {
                collect_parent_properties(
                    *id,
                    &node.parents,
                    &resolved_cache,
                    &fetched_by_id,
                )
            };

            let merged = merge_properties(
                Some(&parent_props),
                &expanded.properties,
                &expanded.excludes,
            );

            let schema = Schema::new(
                *id,
                expanded.name.clone(),
                node.parents.first().copied(),
                children,
                merged,
            );

            if let Some(file_status) = status_entry
                && matches!(
                    file_status,
                    FileStatus::StaleContent { .. } | FileStatus::New { .. }
                )
            {
                let raw = status.raw_by_id.get(id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                        SchemaStorageError::NotFound {
                            name: status_name.clone(),
                        },
                    ))
                })?;
                let content_hash =
                    file_status.content_hash().ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::Storage(
                                SchemaStorageError::NotFound {
                                    name: status_name.clone(),
                                },
                            ),
                        )
                    })?;
                let view =
                    build_view_from_raw(raw, file_status.path(), content_hash)?;
                repository
                    .save_raw_schema_view(*id, &view)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            }

            resolved_cache.insert(*id, schema.clone());
            status.schemas.push(Arc::new(schema));
        }

        Ok(SchemaTreeProcessor {
            status: CompletionState {
                schemas: status.schemas,
                graph: status.graph,
                changed_ids,
            },
            _stage: PhantomData,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE: COMPLETION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaTreeProcessor<Completion, CompletionState> {
    /// Persist schemas, graph, and inheritance metadata.
    pub(crate) fn persist<R: Repository>(
        self,
        repository: &R,
    ) -> Result<Vec<Schema>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        if !self.status.changed_ids.is_empty() {
            let schemas: Vec<Schema> = self
                .status
                .schemas
                .iter()
                .filter(|schema| self.status.changed_ids.contains(schema.id()))
                .map(|schema| (**schema).clone())
                .collect();
            if !schemas.is_empty() {
                repository
                    .save_schemas(&schemas)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            }
        }

        repository
            .save_topological_graph(&self.status.graph)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(self
            .status
            .schemas
            .into_iter()
            .map(|schema| (*schema).clone())
            .collect())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HELPERS
// ═════════════════════════════════════════════════════════════════════════════

fn parse_raw_schema_from_str(
    path: &Path,
    content: &str,
    times: &RawFileTimes,
) -> Result<RawSchema, SchemaLoaderError> {
    let raw: RawSchema = FsReader::parse_structured_from_str(path, content)
        .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

    let name = filename_from_path(path)?;
    let raw = raw
        .with_name(name.into())
        .with_file_times(times.clone())
        .validated(path.to_string_lossy().as_ref())
        .map_err(SchemaLoaderError::Ingestion)?;

    Ok(raw)
}

fn filename_from_path(path: &Path) -> Result<&str, SchemaLoaderError> {
    path.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
            SchemaFileError::InvalidFilename {
                path: path.to_path_buf(),
                reason: "missing filename".into(),
            },
        ))
    })
}

fn build_view_from_raw(
    raw: &RawSchema,
    path: &Path,
    content_hash: [u8; 32],
) -> Result<RawSchemaView, SchemaLoaderError> {
    let file_times = FileTimesMetadata::new(
        raw.file_times().created_at,
        raw.file_times().modified_at,
    );
    let hashes = HashMetadata::new(
        content_hash,
        HashMetadata::compute_property_hashes(raw.properties()),
    );
    let version = SchemaVersion::new(file_times, hashes, raw)
        .map_err(SchemaLoaderError::Ingestion)?;
    let filename = filename_from_path(path)?;

    Ok(RawSchemaView::new(Filename::new(filename.into()), version))
}

fn collect_parent_properties(
    child_id: SchemaId,
    parent_ids: &[SchemaId],
    resolved_cache: &HashMap<SchemaId, Schema>,
    fetched_by_id: &HashMap<SchemaId, Schema>,
) -> HashMap<PropertyName, Property> {
    let mut merged = HashMap::new();

    for parent_id in parent_ids {
        let parent = resolved_cache
            .get(parent_id)
            .or_else(|| fetched_by_id.get(parent_id));

        if let Some(parent) = parent {
            #[expect(
                clippy::iter_over_hash_type,
                reason = "HashMap iteration is intentional for property \
                          inheritance"
            )]
            for (name, prop) in parent.properties() {
                merged.insert(name.clone(), prop.clone());
            }
        } else {
            tracing::warn!(
                schema_id = %child_id,
                parent_id = %parent_id,
                "Parent schema not found in resolved_cache or fetched_by_id, \
                 using empty properties. This may indicate a missing parent in \
                 database."
            );
        }
    }

    merged
}

fn compute_deltas_from_cached(
    raw: &RawSchema,
    view: &RawSchemaView,
) -> (ExcludesDelta, SchemaPropertyDelta) {
    let current = view.current();

    let excludes_delta = diff_excludes(
        current.map_or(&[][..], |v| v.excludes()),
        raw.excludes(),
    );

    let property_delta = current.map_or_else(
        || property_delta_from_new(raw),
        |version| {
            property_delta_from_cached(raw, version.hashes().properties())
        },
    );

    (excludes_delta, property_delta)
}

fn diff_excludes(old: &[PropertyName], new: &[PropertyName]) -> ExcludesDelta {
    let old_set: HashSet<&PropertyName> = old.iter().collect();
    let new_set: HashSet<&PropertyName> = new.iter().collect();

    let mut added: Vec<PropertyName> =
        new.iter().filter(|p| !old_set.contains(p)).cloned().collect();
    let mut removed: Vec<PropertyName> =
        old.iter().filter(|p| !new_set.contains(p)).cloned().collect();
    added.sort();
    removed.sort();
    ExcludesDelta {
        added,
        removed,
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "match ergonomics on RawProperty references"
)]
fn property_delta_from_new(raw: &RawSchema) -> SchemaPropertyDelta {
    let mut inline = HashMap::new();
    let mut refs = HashMap::new();

    for (name, entry) in raw.properties().iter() {
        match entry {
            RawProperty::Inline(value) => {
                inline.insert(name.clone(), value.clone());
            }
            RawProperty::Ref(value) => {
                refs.insert(name.clone(), value.clone());
            }
        }
    }

    SchemaPropertyDelta {
        upserts: SchemaPropertyUpserts {
            inline,
            refs,
        },
        removed: Vec::new(),
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "match ergonomics on RawProperty references"
)]
fn property_delta_from_cached(
    raw: &RawSchema,
    prev_hashes: &HashMap<PropertyName, [u8; 32]>,
) -> SchemaPropertyDelta {
    let mut inline = HashMap::new();
    let mut refs = HashMap::new();
    let mut seen = HashSet::with_capacity(raw.properties().len());

    for (name, entry) in raw.properties().iter() {
        let new_hash = HashMetadata::hash_entry(entry);
        if prev_hashes.get(name) != Some(&new_hash) {
            match entry {
                RawProperty::Inline(value) => {
                    inline.insert(name.clone(), value.clone());
                }
                RawProperty::Ref(value) => {
                    refs.insert(name.clone(), value.clone());
                }
            }
        }
        seen.insert(name.clone());
    }

    let mut removed: Vec<PropertyName> = prev_hashes
        .keys()
        .filter(|name| !seen.contains(*name))
        .cloned()
        .collect();
    removed.sort();

    SchemaPropertyDelta {
        upserts: SchemaPropertyUpserts {
            inline,
            refs,
        },
        removed,
    }
}

fn merge_properties(
    parent: Option<&HashMap<PropertyName, Property>>,
    child: &HashMap<PropertyName, Property>,
    excludes: &[PropertyName],
) -> HashMap<PropertyName, Property> {
    let mut result = child.clone();
    let excluded_names: HashSet<PropertyName> =
        excludes.iter().cloned().collect();
    if let Some(parent) = parent {
        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap iteration is intentional for property \
                      inheritance"
        )]
        for (name, prop) in parent {
            if excluded_names.contains(name) || result.contains_key(name) {
                continue;
            }
            result.insert(name.clone(), prop.clone());
        }
    }
    result
}
