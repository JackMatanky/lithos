//! Schema processor pipeline with batch processing and incremental
//! construction.
//!
//! This module implements a type-safe, stage-based pipeline for processing
//! schema files with support for:
//! - Batch processing (group schemas by status)
//! - Granular stages with specific payloads
//! - Incremental construction (optimize based on change type)
//! - Type-safe transitions via branching enums
//!
//! # Architecture
//!
//! The pipeline progresses through distinct stages, each with its own payload
//! type:
//!
//! ```text
//! Discovery → TimeComparison → ContentComparison → FileParsed
//!     ↓             ↓                ↓                 ↓
//! InheritanceGraphed → PropertyAnalysis → Refresh → Construction → Completion
//! ```

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::PathBuf,
    sync::Arc,
};

use crate::schema::{
    aggregate::{Schema, SchemaId, SchemaName},
    bank::PropertyBank,
    error::{
        SchemaError, SchemaIngestionError, SchemaLoaderError,
        SchemaRepositoryError, SchemaStorageError,
    },
    graph::{
        DagValidator, GraphNode, InheritanceNode, NodeDepth, TopologicalGraph,
    },
    property::{Property, PropertyName},
    raw::{
        RawFileTimes, RawSchema,
        property::{RawPropertyInline, RawPropertyRef},
    },
    storage::Repository,
    views::RawSchemaView,
};

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE MARKERS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct Discovery;

#[derive(Debug)]
pub(crate) struct TimeComparison;

#[derive(Debug)]
pub(crate) struct ContentComparison;

#[derive(Debug)]
pub(crate) struct FileParsed;

#[derive(Debug)]
pub(crate) struct InheritanceGraphed;

#[derive(Debug)]
pub(crate) struct PropertyAnalysis;

#[derive(Debug)]
pub(crate) struct Refresh;

#[derive(Debug)]
pub(crate) struct Construction;

#[derive(Debug)]
pub(crate) struct Completion;

// ═════════════════════════════════════════════════════════════════════════════
//  CORE ENUMS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtendsChangeKind {
    Unchanged,
    RootToChild,
    ChildToRoot,
    Rewired,
}

impl ExtendsChangeKind {
    #[inline]
    #[must_use]
    pub(crate) const fn requires_merge(&self) -> bool {
        matches!(self, Self::Rewired | Self::RootToChild)
    }

    #[inline]
    #[must_use]
    pub(crate) const fn can_update(&self) -> bool {
        matches!(self, Self::Unchanged | Self::ChildToRoot)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DELTA STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ExcludesDelta {
    pub(crate) added: Vec<PropertyName>,
    pub(crate) removed: Vec<PropertyName>,
}

impl ExcludesDelta {
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyDelta {
    pub(crate) upserts: SchemaPropertyUpserts,
    pub(crate) removed: Vec<PropertyName>,
}

impl SchemaPropertyDelta {
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.inline.is_empty()
            && self.upserts.refs.is_empty()
            && self.removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyUpserts {
    pub(crate) inline: HashMap<PropertyName, RawPropertyInline>,
    pub(crate) refs: HashMap<PropertyName, RawPropertyRef>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  PAYLOAD STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MissingPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PresentPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleSuspectPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_str: Box<str>,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeletedPayload {
    pub(crate) is_deleted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FreshPayload {
    pub(crate) path: PathBuf,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleTimestampPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleContentSuspectPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GraphedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: Option<RawSchemaView>,
    pub(crate) extends_change: ExtendsChangeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnalyzedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
    pub(crate) extends_change: ExtendsChangeKind,
    pub(crate) excludes_delta: Option<ExcludesDelta>,
    pub(crate) property_delta: Option<SchemaPropertyDelta>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY CONTEXT
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub(crate) struct DiscoveryContext {
    pub(crate) graph: Option<TopologicalGraph<InheritanceNode>>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) has_property_bank: bool,
}

// ═════════════════════════════════════════════════════════════════════════════
//  SCHEMA PROCESSOR
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct SchemaProcessor<Stage, State> {
    pub(crate) status: State,
    _stage: PhantomData<Stage>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  STATE STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct DiscoveryState {
    pub(crate) context: DiscoveryContext,
}

#[derive(Debug)]
pub(crate) struct MissingBatch {
    pub(crate) batch: HashMap<SchemaId, MissingPayload>,
}

#[derive(Debug)]
pub(crate) struct PresentBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<PresentPayload>>,
    pub(crate) batch: HashMap<SchemaId, PresentPayload>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct FreshBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<FreshPayload>>,
    pub(crate) batch: HashMap<SchemaId, FreshPayload>,
}

#[derive(Debug)]
pub(crate) struct StaleSuspectBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<StaleSuspectPayload>>,
    pub(crate) batch: HashMap<SchemaId, StaleSuspectPayload>,
}

#[derive(Debug)]
pub(crate) struct StaleTimestampBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<StaleTimestampPayload>>,
    pub(crate) batch: HashMap<SchemaId, StaleTimestampPayload>,
}

#[derive(Debug)]
pub(crate) struct StaleContentBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<StaleContentSuspectPayload>>,
    pub(crate) batch: HashMap<SchemaId, StaleContentSuspectPayload>,
}

#[derive(Debug)]
pub(crate) struct ParsedBatch {
    pub(crate) new_schemas: HashMap<SchemaId, NewPayload>,
    pub(crate) stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
}

#[derive(Debug)]
pub(crate) struct GraphedBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<GraphedPayload>>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct AnalyzedBatch {
    pub(crate) graph: TopologicalGraph<GraphNode<AnalyzedPayload>>,
    pub(crate) refresh_ids: HashSet<SchemaId>,
    pub(crate) rebuild_ids: HashSet<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct ConstructionState {
    pub(crate) graph: TopologicalGraph<GraphNode<AnalyzedPayload>>,
    pub(crate) refresh_ids: HashSet<SchemaId>,
    pub(crate) rebuild_ids: HashSet<SchemaId>,
    pub(crate) schemas: Vec<Arc<Schema>>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  BRANCHING ENUMS
// ═════════════════════════════════════════════════════════════════════════════

pub(crate) enum DiscoveryBranch {
    AllMissing(SchemaProcessor<FileParsed, MissingBatch>),
    SomePresent {
        missing: Option<SchemaProcessor<FileParsed, MissingBatch>>,
        present: SchemaProcessor<TimeComparison, PresentBatch>,
    },
}

pub(crate) enum TimestampBranch {
    AllFresh(SchemaProcessor<InheritanceGraphed, FreshBatch>),
    SomeSuspect {
        fresh: Option<SchemaProcessor<InheritanceGraphed, FreshBatch>>,
        suspect: SchemaProcessor<ContentComparison, StaleSuspectBatch>,
    },
}

pub(crate) enum ContentBranch {
    AllStaleTimestamps(SchemaProcessor<Refresh, StaleTimestampBatch>),
    SomeStaleContent {
        timestamps: Option<SchemaProcessor<Refresh, StaleTimestampBatch>>,
        content: SchemaProcessor<FileParsed, StaleContentBatch>,
    },
}

// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Discovery, DiscoveryState> {
    pub(crate) fn from_context(context: DiscoveryContext) -> Self {
        Self {
            status: DiscoveryState {
                context,
            },
            _stage: PhantomData,
        }
    }

    pub(crate) fn discover<R>(
        self,
        repository: &R,
        source: &crate::fs::reader::Reader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaStorageError>,
    {
        let DiscoveryState {
            context,
        } = self.status;
        let DiscoveryContext {
            graph,
            files,
            has_property_bank: _,
        } = context;

        let views_by_path =
            repository.find_raw_schema_views_by_paths(&files).map_err(|e| {
                let storage_err: SchemaStorageError = e.into();
                SchemaLoaderError::Repository(SchemaRepositoryError::Storage(
                    storage_err,
                ))
            })?;

        let ids_by_path =
            repository.find_schema_ids_by_paths(&files).map_err(|e| {
                let storage_err: SchemaStorageError = e.into();
                SchemaLoaderError::Repository(SchemaRepositoryError::Storage(
                    storage_err,
                ))
            })?;

        let (missing_batch, present_batch, deleted_ids) =
            Self::classify_file_state(
                &files,
                &views_by_path,
                &ids_by_path,
                graph.as_ref(),
                source,
            );

        match (missing_batch.is_empty(), present_batch.is_empty()) {
            (false, true) => Ok(DiscoveryBranch::AllMissing(SchemaProcessor {
                status: MissingBatch {
                    batch: missing_batch,
                },
                _stage: PhantomData,
            })),
            (_, false) => {
                let missing_processor = if missing_batch.is_empty() {
                    None
                } else {
                    Some(SchemaProcessor {
                        status: MissingBatch {
                            batch: missing_batch,
                        },
                        _stage: PhantomData,
                    })
                };

                let graph = graph.unwrap_or_else(|| TopologicalGraph {
                    order: Vec::new(),
                    nodes: HashMap::new(),
                    roots: Vec::new(),
                });

                let graph = hydrate_graph_with_present(graph, &present_batch);

                Ok(DiscoveryBranch::SomePresent {
                    missing: missing_processor,
                    present: SchemaProcessor {
                        status: PresentBatch {
                            graph,
                            batch: present_batch,
                            deleted_ids,
                        },
                        _stage: PhantomData,
                    },
                })
            }
            (true, true) => {
                use crate::schema::error::SchemaFileError;
                Err(SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    SchemaFileError::FileSystem {
                        reason: "no schema files found in directory".into(),
                    },
                )))
            }
        }
    }

    fn classify_file_state(
        files: &[PathBuf],
        views_by_path: &HashMap<PathBuf, RawSchemaView>,
        ids_by_path: &HashMap<PathBuf, SchemaId>,
        graph: Option<&TopologicalGraph<InheritanceNode>>,
        source: &crate::fs::reader::Reader,
    ) -> (
        HashMap<SchemaId, MissingPayload>,
        HashMap<SchemaId, PresentPayload>,
        Vec<SchemaId>,
    ) {
        let mut missing_batch: HashMap<SchemaId, MissingPayload> =
            HashMap::new();
        let mut present_batch: HashMap<SchemaId, PresentPayload> =
            HashMap::new();

        for path in files {
            let times = RawFileTimes {
                created_at: source.created_at(path),
                modified_at: source.modified_at(path),
            };

            if let (Some(view), Some(id)) =
                (views_by_path.get(path), ids_by_path.get(path))
            {
                present_batch.insert(*id, PresentPayload {
                    path: path.clone(),
                    times,
                    view: view.clone(),
                });
            } else {
                let id = SchemaId::new();
                missing_batch.insert(id, MissingPayload {
                    path: path.clone(),
                    times,
                });
            }
        }

        let mut deleted_ids = Vec::new();
        if let Some(graph) = graph {
            let file_ids: HashSet<SchemaId> =
                present_batch.keys().copied().collect();
            for id in graph.nodes.keys() {
                if !file_ids.contains(id) && !missing_batch.contains_key(id) {
                    deleted_ids.push(*id);
                }
            }
        }

        (missing_batch, present_batch, deleted_ids)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TIMECOMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<TimeComparison, PresentBatch> {
    pub(crate) fn compare_timestamps(
        self,
        source: &crate::fs::reader::Reader,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let PresentBatch {
            graph,
            batch,
            ..
        } = self.status;

        let mut fresh_batch: HashMap<SchemaId, FreshPayload> = HashMap::new();
        let mut suspect_batch: HashMap<SchemaId, StaleSuspectPayload> =
            HashMap::new();

        for (id, present) in batch {
            let PresentPayload {
                path,
                times,
                view,
            } = present;

            let timestamps_match = view
                .current()
                .and_then(|v| {
                    Some(v.file_times().is_timestamp_match(
                        times.created_at,
                        times.modified_at,
                    ))
                })
                .unwrap_or(false);

            if timestamps_match {
                fresh_batch.insert(id, FreshPayload {
                    path,
                    view,
                });
            } else {
                let content_str = source
                    .read_to_string(&path)
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?;
                suspect_batch.insert(id, StaleSuspectPayload {
                    path,
                    times,
                    content_str: content_str.into(),
                    view,
                });
            }
        }

        match (fresh_batch.is_empty(), suspect_batch.is_empty()) {
            (false, true) => {
                let graph = hydrate_graph_with_fresh(graph, &fresh_batch);
                Ok(TimestampBranch::AllFresh(SchemaProcessor {
                    status: FreshBatch {
                        graph,
                        batch: fresh_batch,
                    },
                    _stage: PhantomData,
                }))
            }
            (_, false) => {
                let fresh_processor = if fresh_batch.is_empty() {
                    None
                } else {
                    let graph =
                        hydrate_graph_with_fresh(graph.clone(), &fresh_batch);
                    Some(SchemaProcessor {
                        status: FreshBatch {
                            graph,
                            batch: fresh_batch,
                        },
                        _stage: PhantomData,
                    })
                };

                let graph = hydrate_graph_with_suspect(graph, &suspect_batch);
                Ok(TimestampBranch::SomeSuspect {
                    fresh: fresh_processor,
                    suspect: SchemaProcessor {
                        status: StaleSuspectBatch {
                            graph,
                            batch: suspect_batch,
                        },
                        _stage: PhantomData,
                    },
                })
            }
            (true, true) => unreachable!("empty batch"),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CONTENTCOMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<ContentComparison, StaleSuspectBatch> {
    pub(crate) fn compare_content(
        self,
        source: &crate::fs::reader::Reader,
    ) -> Result<ContentBranch, SchemaLoaderError> {
        let StaleSuspectBatch {
            graph,
            batch,
        } = self.status;

        let mut timestamp_batch: HashMap<SchemaId, StaleTimestampPayload> =
            HashMap::new();
        let mut content_batch: HashMap<SchemaId, StaleContentSuspectPayload> =
            HashMap::new();

        for (id, suspect) in batch {
            let StaleSuspectPayload {
                path,
                times,
                content_str,
                view,
            } = suspect;

            let content_hash = *blake3::hash(content_str.as_bytes()).as_bytes();

            let content_match = view
                .current()
                .map(|v| v.hashes().is_content_match(&content_hash))
                .unwrap_or(false);

            if content_match {
                timestamp_batch.insert(id, StaleTimestampPayload {
                    path,
                    times,
                    view,
                });
            } else {
                let raw = crate::fs::reader::Reader::parse_structured_from_str(
                    &path,
                    &content_str,
                )
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
                content_batch.insert(id, StaleContentSuspectPayload {
                    path,
                    times,
                    content_hash,
                    raw,
                    view,
                });
            }
        }

        match (timestamp_batch.is_empty(), content_batch.is_empty()) {
            (false, true) => {
                let graph =
                    hydrate_graph_with_stale_timestamp(graph, &timestamp_batch);
                Ok(ContentBranch::AllStaleTimestamps(SchemaProcessor {
                    status: StaleTimestampBatch {
                        graph,
                        batch: timestamp_batch,
                    },
                    _stage: PhantomData,
                }))
            }
            (_, false) => {
                let timestamp_processor = if timestamp_batch.is_empty() {
                    None
                } else {
                    let graph = hydrate_graph_with_stale_timestamp(
                        graph.clone(),
                        &timestamp_batch,
                    );
                    Some(SchemaProcessor {
                        status: StaleTimestampBatch {
                            graph,
                            batch: timestamp_batch,
                        },
                        _stage: PhantomData,
                    })
                };

                let graph =
                    hydrate_graph_with_stale_content(graph, &content_batch);
                Ok(ContentBranch::SomeStaleContent {
                    timestamps: timestamp_processor,
                    content: SchemaProcessor {
                        status: StaleContentBatch {
                            graph,
                            batch: content_batch,
                        },
                        _stage: PhantomData,
                    },
                })
            }
            (true, true) => unreachable!("empty batch"),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  FILEPARSED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<FileParsed, MissingBatch> {
    pub(crate) fn parse_new_schemas(
        self,
        source: &crate::fs::reader::Reader,
    ) -> Result<ParsedBatch, SchemaLoaderError> {
        let MissingBatch {
            batch,
        } = self.status;
        let mut new_schemas: HashMap<SchemaId, NewPayload> = HashMap::new();

        for (id, missing) in batch {
            let MissingPayload {
                path,
                times,
            } = missing;

            let content = source
                .read_to_string(&path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            let content_hash = *blake3::hash(content.as_bytes()).as_bytes();

            let raw = crate::fs::reader::Reader::parse_structured_from_str(
                &path, &content,
            )
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?;

            new_schemas.insert(id, NewPayload {
                path,
                times,
                content_hash,
                raw,
            });
        }

        Ok(ParsedBatch {
            new_schemas,
            stale_schemas: HashMap::new(),
        })
    }
}

impl SchemaProcessor<FileParsed, StaleContentBatch> {
    pub(crate) fn pass_through(self) -> ParsedBatch {
        let StaleContentBatch {
            batch,
            ..
        } = self.status;

        ParsedBatch {
            new_schemas: HashMap::new(),
            stale_schemas: batch,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  INHERITANCEGRAPHED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl ParsedBatch {
    pub(crate) fn build_graph(
        self,
        existing_graph: Option<TopologicalGraph<InheritanceNode>>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        let ParsedBatch {
            new_schemas,
            stale_schemas,
        } = self;

        match existing_graph {
            None => Self::build_new_graph_inner(new_schemas, stale_schemas),
            Some(graph) => {
                Self::patch_graph_inner(graph, new_schemas, stale_schemas)
            }
        }
    }

    fn build_new_graph_inner(
        new_schemas: HashMap<SchemaId, NewPayload>,
        stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        let mut name_index: HashMap<SchemaName, SchemaId> = HashMap::new();
        for (id, new) in &new_schemas {
            let name = SchemaName::try_new(new.raw.name())
                .expect("raw schema name should be valid");
            name_index.insert(name, *id);
        }
        for (id, stale) in &stale_schemas {
            let name = SchemaName::try_new(stale.raw.name())
                .expect("raw schema name should be valid");
            name_index.insert(name, *id);
        }

        let mut nodes: HashMap<SchemaId, InheritanceNode> = HashMap::new();
        let mut payloads: HashMap<SchemaId, GraphedPayload> = HashMap::new();

        for (id, new) in &new_schemas {
            let parent_id = new
                .raw
                .extends()
                .and_then(|name| name_index.get(name).copied());

            nodes.insert(*id, InheritanceNode {
                id: *id,
                parents: parent_id.map(|p| vec![p]).unwrap_or_default(),
                children: Vec::new(),
                depth: NodeDepth::ROOT,
            });

            payloads.insert(*id, GraphedPayload {
                path: new.path.clone(),
                times: new.times.clone(),
                content_hash: new.content_hash,
                raw: new.raw.clone(),
                view: None,
                extends_change: ExtendsChangeKind::Unchanged,
            });
        }

        for (id, stale) in &stale_schemas {
            let parent_id = stale
                .raw
                .extends()
                .and_then(|name| name_index.get(name).copied());

            nodes.insert(*id, InheritanceNode {
                id: *id,
                parents: parent_id.map(|p| vec![p]).unwrap_or_default(),
                children: Vec::new(),
                depth: NodeDepth::ROOT,
            });

            payloads.insert(*id, GraphedPayload {
                path: stale.path.clone(),
                times: stale.times.clone(),
                content_hash: stale.content_hash,
                raw: stale.raw.clone(),
                view: Some(stale.view.clone()),
                extends_change: ExtendsChangeKind::Unchanged,
            });
        }

        build_children(&mut nodes);

        DagValidator::new(&nodes).detect_cycles().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;

        let mut graph = TopologicalGraph {
            nodes,
            order: Vec::new(),
            roots: Vec::new(),
        };
        graph.compute_depths();
        let (order, roots) = graph.topological_sort().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;
        graph.order = order;
        graph.roots = roots;

        let mut hydrated_nodes: HashMap<SchemaId, GraphNode<GraphedPayload>> =
            HashMap::new();
        for (id, node) in &graph.nodes {
            if let Some(payload) = payloads.remove(id) {
                hydrated_nodes.insert(*id, GraphNode {
                    id: node.id,
                    parents: node.parents.clone(),
                    children: node.children.clone(),
                    depth: node.depth,
                    payload,
                });
            }
        }

        Ok(GraphedBatch {
            graph: TopologicalGraph {
                order: graph.order,
                nodes: hydrated_nodes,
                roots: graph.roots,
            },
            deleted_ids: Vec::new(),
        })
    }

    fn patch_graph_inner(
        mut graph: TopologicalGraph<InheritanceNode>,
        new_schemas: HashMap<SchemaId, NewPayload>,
        stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        let mut name_index: HashMap<SchemaName, SchemaId> = HashMap::new();
        for (id, new) in &new_schemas {
            let name = SchemaName::try_new(new.raw.name())
                .expect("raw schema name should be valid");
            name_index.insert(name, *id);
        }
        for (id, stale) in &stale_schemas {
            let name = SchemaName::try_new(stale.raw.name())
                .expect("raw schema name should be valid");
            name_index.insert(name, *id);
        }

        for (id, new) in &new_schemas {
            let parent_id = new
                .raw
                .extends()
                .and_then(|name| name_index.get(name).copied());

            let node = InheritanceNode {
                id: *id,
                parents: parent_id.map(|p| vec![p]).unwrap_or_default(),
                children: Vec::new(),
                depth: NodeDepth::ROOT,
            };
            graph.nodes.insert(*id, node);
        }

        let mut extends_changes: HashMap<SchemaId, ExtendsChangeKind> =
            HashMap::new();
        for (id, stale) in &stale_schemas {
            let old_parent = graph
                .nodes
                .get(id)
                .and_then(|node| node.parents.first().copied());

            let new_parent = stale
                .raw
                .extends()
                .and_then(|name| name_index.get(name).copied());

            let change_kind = match (old_parent, new_parent) {
                (None, None) => ExtendsChangeKind::Unchanged,
                (None, Some(_)) => ExtendsChangeKind::RootToChild,
                (Some(_), None) => ExtendsChangeKind::ChildToRoot,
                (Some(old), Some(new)) if old == new => {
                    ExtendsChangeKind::Unchanged
                }
                (Some(_), Some(_)) => ExtendsChangeKind::Rewired,
            };

            extends_changes.insert(*id, change_kind);

            if change_kind != ExtendsChangeKind::Unchanged {
                if let Some(node) = graph.nodes.get_mut(id) {
                    node.parents =
                        new_parent.map(|p| vec![p]).unwrap_or_default();
                }
            }
        }

        build_children(&mut graph.nodes);

        graph.compute_depths();
        let (order, roots) = graph.topological_sort().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;
        graph.order = order;
        graph.roots = roots;

        DagValidator::new(&graph.nodes).detect_cycles().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;

        let mut nodes: HashMap<SchemaId, GraphNode<GraphedPayload>> =
            HashMap::new();
        for (id, node) in graph.nodes {
            let payload = if let Some(new) = new_schemas.get(&id) {
                GraphedPayload {
                    path: new.path.clone(),
                    times: new.times.clone(),
                    content_hash: new.content_hash,
                    raw: new.raw.clone(),
                    view: None,
                    extends_change: ExtendsChangeKind::Unchanged,
                }
            } else if let Some(stale) = stale_schemas.get(&id) {
                let change_kind = extends_changes
                    .get(&id)
                    .copied()
                    .unwrap_or(ExtendsChangeKind::Unchanged);

                GraphedPayload {
                    path: stale.path.clone(),
                    times: stale.times.clone(),
                    content_hash: stale.content_hash,
                    raw: stale.raw.clone(),
                    view: Some(stale.view.clone()),
                    extends_change: change_kind,
                }
            } else {
                continue;
            };

            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload,
            });
        }

        Ok(GraphedBatch {
            graph: TopologicalGraph {
                order: graph.order,
                nodes,
                roots: graph.roots,
            },
            deleted_ids: Vec::new(),
        })
    }
}

fn build_children(nodes: &mut HashMap<SchemaId, InheritanceNode>) {
    let parent_refs: Vec<(SchemaId, Vec<SchemaId>)> =
        nodes.iter().map(|(id, node)| (*id, node.parents.clone())).collect();

    for (child_id, parents) in parent_refs {
        for parent_id in parents {
            if let Some(parent) = nodes.get_mut(&parent_id) {
                if !parent.children.contains(&child_id) {
                    parent.children.push(child_id);
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  PROPERTYANALYSIS STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<InheritanceGraphed, GraphedBatch> {
    pub(crate) fn from_graphed_batch(batch: GraphedBatch) -> Self {
        Self {
            status: batch,
            _stage: PhantomData,
        }
    }

    pub(crate) fn analyze_properties(
        self,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<AnalyzedBatch, SchemaLoaderError> {
        let GraphedBatch {
            graph,
            deleted_ids: _,
        } = self.status;

        let mut refresh_ids: HashSet<SchemaId> = HashSet::new();
        let mut rebuild_ids: HashSet<SchemaId> = HashSet::new();
        let mut analyzed_nodes: HashMap<SchemaId, GraphNode<AnalyzedPayload>> =
            HashMap::new();

        for (id, node) in graph.nodes {
            let GraphedPayload {
                path,
                times,
                content_hash,
                raw,
                view,
                extends_change,
            } = node.payload;

            if view.is_none() {
                rebuild_ids.insert(id);
                continue;
            }

            let view = view.unwrap();

            let old_excludes =
                view.current().map(|v| v.excludes()).unwrap_or(&[]);
            let excludes_delta = diff_excludes(old_excludes, raw.excludes());

            let empty_hashes = HashMap::new();
            let old_property_hashes = view
                .current()
                .map(|v| v.hashes().properties())
                .unwrap_or(&empty_hashes);
            let property_delta = diff_properties(&raw, old_property_hashes);

            let bank_changed = if let Some(pb_delta) = property_bank_delta {
                view.current()
                    .map(|v| {
                        v.bank_references()
                            .values()
                            .any(|bank_prop| pb_delta.contains(bank_prop))
                    })
                    .unwrap_or(false)
            } else {
                false
            };

            let needs_rebuild = extends_change != ExtendsChangeKind::Unchanged
                || !excludes_delta.is_empty()
                || !property_delta.is_empty()
                || bank_changed;

            if needs_rebuild {
                rebuild_ids.insert(id);
            } else {
                refresh_ids.insert(id);
            }

            analyzed_nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: AnalyzedPayload {
                    path,
                    times,
                    content_hash,
                    raw,
                    view,
                    extends_change,
                    excludes_delta: if excludes_delta.is_empty() {
                        None
                    } else {
                        Some(excludes_delta)
                    },
                    property_delta: if property_delta.is_empty() {
                        None
                    } else {
                        Some(property_delta)
                    },
                },
            });
        }

        Ok(AnalyzedBatch {
            graph: TopologicalGraph {
                order: graph.order,
                nodes: analyzed_nodes,
                roots: graph.roots,
            },
            refresh_ids,
            rebuild_ids,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  REFRESH STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Refresh, AnalyzedBatch> {
    pub(crate) fn refresh_metadata<R>(
        self,
        repository: &R,
    ) -> Result<AnalyzedBatch, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaStorageError>,
    {
        use crate::schema::views::{
            metadata::{FileTimesMetadata, HashMetadata},
            version::SchemaVersion,
        };

        let AnalyzedBatch {
            mut graph,
            refresh_ids,
            rebuild_ids,
        } = self.status;

        for id in &refresh_ids {
            if let Some(node) = graph.nodes.get_mut(id) {
                let payload = &node.payload;

                let property_hashes = HashMetadata::compute_property_hashes(
                    payload.raw.properties(),
                );

                let file_times = FileTimesMetadata::new(
                    payload.times.created_at,
                    payload.times.modified_at,
                );
                let hashes =
                    HashMetadata::new(payload.content_hash, property_hashes);
                let version =
                    SchemaVersion::new(file_times, hashes, &payload.raw)
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?;

                let view = &mut node.payload.view;
                view.add_version(version);

                repository.save_raw_schema_view(*id, view).map_err(|e| {
                    let storage_err: SchemaStorageError = e.into();
                    SchemaLoaderError::Repository(
                        SchemaRepositoryError::Storage(storage_err),
                    )
                })?;
            }
        }

        Ok(AnalyzedBatch {
            graph,
            refresh_ids,
            rebuild_ids,
        })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CONSTRUCTION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Construction, AnalyzedBatch> {
    pub(crate) fn construct_schemas(
        self,
        repository: &impl Repository<Error = impl Into<SchemaStorageError>>,
        property_bank: &PropertyBank,
    ) -> Result<ConstructionState, SchemaLoaderError> {
        use crate::schema::expander::RefExpander;

        let AnalyzedBatch {
            graph,
            refresh_ids,
            rebuild_ids,
        } = self.status;

        let fetch_ids: Vec<SchemaId> = refresh_ids.iter().copied().collect();
        let mut fetched_by_id: HashMap<SchemaId, Schema> = HashMap::new();
        if !fetch_ids.is_empty() {
            let fetched =
                repository.find_schemas_by_ids(&fetch_ids).map_err(|e| {
                    let storage_err: SchemaStorageError = e.into();
                    SchemaLoaderError::Repository(
                        SchemaRepositoryError::Storage(storage_err),
                    )
                })?;
            fetched_by_id = fetched.into_iter().map(|s| (*s.id(), s)).collect();
        }

        let expand_pairs: Vec<(SchemaId, RawSchema)> = rebuild_ids
            .iter()
            .filter_map(|id| {
                graph.nodes.get(id).map(|node| (*id, node.payload.raw.clone()))
            })
            .collect();

        let expanded_by_id: HashMap<SchemaId, HashMap<PropertyName, Property>> =
            if expand_pairs.is_empty() {
                HashMap::new()
            } else {
                let expanded_vec = RefExpander::new(property_bank)
                    .expand_all(expand_pairs)
                    .map_err(SchemaLoaderError::Resolution)?;
                expanded_vec
                    .into_iter()
                    .map(|(id, expanded)| (id, expanded.properties))
                    .collect()
            };

        let mut schemas = Vec::new();
        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in &graph.order {
            let node = graph.nodes.get(id).unwrap();
            let payload = &node.payload;

            let schema = if refresh_ids.contains(id) {
                fetched_by_id.remove(id).unwrap()
            } else if rebuild_ids.contains(id) {
                Self::construct_schema_incremental(
                    *id,
                    node,
                    &expanded_by_id,
                    &fetched_by_id,
                    &constructed_cache,
                )?
            } else {
                repository
                    .find_schema_by_id(*id)
                    .map_err(|e| {
                        let storage_err: SchemaStorageError = e.into();
                        SchemaLoaderError::Repository(
                            SchemaRepositoryError::Storage(storage_err),
                        )
                    })?
                    .ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::File(
                                crate::schema::error::SchemaFileError::FileSystem {
                                    reason: format!(
                                        "schema {id} not found in repository"
                                    ).into(),
                                },
                            ),
                        )
                    })?
            };

            constructed_cache.insert(*id, schema.clone());
            schemas.push(Arc::new(schema));
        }

        Ok(ConstructionState {
            graph,
            refresh_ids,
            rebuild_ids,
            schemas,
        })
    }

    fn construct_schema_incremental(
        id: SchemaId,
        node: &GraphNode<AnalyzedPayload>,
        expanded_by_id: &HashMap<SchemaId, HashMap<PropertyName, Property>>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
        constructed_cache: &HashMap<SchemaId, Schema>,
    ) -> Result<Schema, SchemaLoaderError> {
        let payload = &node.payload;
        let extends_change = payload.extends_change;
        let property_delta = &payload.property_delta;

        match (extends_change, property_delta) {
            (ExtendsChangeKind::Unchanged, Some(delta)) => {
                let schema = fetched_by_id
                    .get(&id)
                    .or_else(|| constructed_cache.get(&id))
                    .cloned()
                    .ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::File(
                                crate::schema::error::SchemaFileError::FileSystem {
                                    reason: format!(
                                        "schema {id} not found for update"
                                    ).into(),
                                },
                            ),
                        )
                    })?;

                let expanded = expanded_by_id.get(&id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "expanded properties not found for {id}"
                            )
                            .into(),
                        },
                    ))
                })?;

                let mut properties = schema.properties().clone();
                for (name, prop) in expanded {
                    if delta.upserts.inline.contains_key(name)
                        || delta.upserts.refs.contains_key(name)
                    {
                        properties.insert(name.clone(), prop.clone());
                    }
                }
                for name in &delta.removed {
                    properties.remove(name);
                }

                let name = SchemaName::try_new(payload.raw.name())
                    .expect("raw schema name should be valid");

                Ok(Schema::new(
                    id,
                    name,
                    node.parents.first().copied(),
                    node.children.clone(),
                    properties,
                ))
            }

            (
                ExtendsChangeKind::Rewired | ExtendsChangeKind::RootToChild,
                _,
            ) => {
                let expanded = expanded_by_id.get(&id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "expanded properties not found for {id}"
                            )
                            .into(),
                        },
                    ))
                })?;

                let parent_props = if node.parents.is_empty() {
                    HashMap::new()
                } else {
                    Self::collect_parent_properties(
                        &node.parents,
                        &constructed_cache,
                        &fetched_by_id,
                    )
                };

                let mut merged = parent_props;
                for (name, prop) in expanded {
                    merged.insert(name.clone(), prop.clone());
                }
                for excluded in payload.raw.excludes() {
                    merged.remove(excluded);
                }

                let name = SchemaName::try_new(payload.raw.name())
                    .expect("raw schema name should be valid");

                Ok(Schema::new(
                    id,
                    name,
                    node.parents.first().copied(),
                    node.children.clone(),
                    merged,
                ))
            }

            (ExtendsChangeKind::ChildToRoot, _) => {
                let expanded = expanded_by_id.get(&id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "expanded properties not found for {id}"
                            )
                            .into(),
                        },
                    ))
                })?;

                let name = SchemaName::try_new(payload.raw.name())
                    .expect("raw schema name should be valid");

                Ok(Schema::new(
                    id,
                    name,
                    None,
                    node.children.clone(),
                    expanded.clone(),
                ))
            }

            (ExtendsChangeKind::Unchanged, None) => fetched_by_id
                .get(&id)
                .or_else(|| constructed_cache.get(&id))
                .cloned()
                .ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!("schema {id} not found for fetch")
                                .into(),
                        },
                    ))
                }),
        }
    }

    fn collect_parent_properties(
        parent_ids: &[SchemaId],
        constructed_cache: &HashMap<SchemaId, Schema>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
    ) -> HashMap<PropertyName, Property> {
        let mut merged = HashMap::new();
        for parent_id in parent_ids {
            if let Some(schema) = constructed_cache
                .get(parent_id)
                .or_else(|| fetched_by_id.get(parent_id))
            {
                for (name, prop) in schema.properties() {
                    merged.insert(name.clone(), prop.clone());
                }
            }
        }
        merged
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPLETION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Completion, ConstructionState> {
    pub(crate) fn from_construction_state(state: ConstructionState) -> Self {
        Self {
            status: state,
            _stage: PhantomData,
        }
    }

    pub(crate) fn complete(
        self,
        repository: &impl Repository<Error = impl Into<SchemaStorageError>>,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        let ConstructionState {
            graph,
            schemas,
            refresh_ids: _,
            rebuild_ids: _,
        } = self.status;

        let owned_schemas: Vec<Schema> =
            schemas.iter().map(|s| (**s).clone()).collect();
        if !owned_schemas.is_empty() {
            repository.save_schemas(&owned_schemas).map_err(|e| {
                let storage_err: SchemaStorageError = e.into();
                SchemaLoaderError::Repository(SchemaRepositoryError::Storage(
                    storage_err,
                ))
            })?;
        }

        let inheritance_graph = dehydrate_graph_to_inheritance(&graph);

        repository.save_topological_graph(&inheritance_graph).map_err(|e| {
            let storage_err: SchemaStorageError = e.into();
            SchemaLoaderError::Repository(SchemaRepositoryError::Storage(
                storage_err,
            ))
        })?;

        Ok(schemas)
    }
}

fn dehydrate_graph_to_inheritance(
    graph: &TopologicalGraph<GraphNode<AnalyzedPayload>>,
) -> TopologicalGraph<InheritanceNode> {
    let mut nodes = HashMap::new();

    for (id, node) in &graph.nodes {
        nodes.insert(*id, InheritanceNode {
            id: node.id,
            parents: node.parents.clone(),
            children: node.children.clone(),
            depth: node.depth,
        });
    }

    TopologicalGraph {
        order: graph.order.clone(),
        nodes,
        roots: graph.roots.clone(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HELPER FUNCTIONS
// ═════════════════════════════════════════════════════════════════════════════

fn hydrate_graph_with_present(
    graph: TopologicalGraph<InheritanceNode>,
    batch: &HashMap<SchemaId, PresentPayload>,
) -> TopologicalGraph<GraphNode<PresentPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: payload.clone(),
            });
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

fn hydrate_graph_with_fresh(
    graph: TopologicalGraph<GraphNode<PresentPayload>>,
    batch: &HashMap<SchemaId, FreshPayload>,
) -> TopologicalGraph<GraphNode<FreshPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: payload.clone(),
            });
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

fn hydrate_graph_with_suspect(
    graph: TopologicalGraph<GraphNode<PresentPayload>>,
    batch: &HashMap<SchemaId, StaleSuspectPayload>,
) -> TopologicalGraph<GraphNode<StaleSuspectPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: payload.clone(),
            });
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

fn hydrate_graph_with_stale_timestamp(
    graph: TopologicalGraph<GraphNode<StaleSuspectPayload>>,
    batch: &HashMap<SchemaId, StaleTimestampPayload>,
) -> TopologicalGraph<GraphNode<StaleTimestampPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: payload.clone(),
            });
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

fn hydrate_graph_with_stale_content(
    graph: TopologicalGraph<GraphNode<StaleSuspectPayload>>,
    batch: &HashMap<SchemaId, StaleContentSuspectPayload>,
) -> TopologicalGraph<GraphNode<StaleContentSuspectPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, GraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                payload: payload.clone(),
            });
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

fn diff_excludes(
    old_excludes: &[PropertyName],
    new_excludes: &[PropertyName],
) -> ExcludesDelta {
    let old_set: HashSet<&PropertyName> = old_excludes.iter().collect();
    let new_set: HashSet<&PropertyName> = new_excludes.iter().collect();

    let added: Vec<PropertyName> =
        new_set.difference(&old_set).map(|p| (*p).clone()).collect();
    let removed: Vec<PropertyName> =
        old_set.difference(&new_set).map(|p| (*p).clone()).collect();

    ExcludesDelta {
        added,
        removed,
    }
}

fn diff_properties(
    raw: &RawSchema,
    old_hashes: &HashMap<PropertyName, [u8; 32]>,
) -> SchemaPropertyDelta {
    use blake3;

    let mut upserts = SchemaPropertyUpserts::default();
    let mut removed: Vec<PropertyName> = Vec::new();

    let mut current_hashes: HashMap<PropertyName, [u8; 32]> = HashMap::new();
    for (name, prop) in raw.properties() {
        let hash = *blake3::hash(
            serde_json::to_string(prop).unwrap_or_default().as_bytes(),
        )
        .as_bytes();
        current_hashes.insert(name.clone(), hash);
    }

    for (name, hash) in &current_hashes {
        if let Some(old_hash) = old_hashes.get(name) {
            if hash != old_hash {
                if let Some(prop) = raw.properties().get(name) {
                    match prop {
                        crate::schema::raw::property::RawProperty::Inline(
                            inline,
                        ) => {
                            upserts.inline.insert(name.clone(), inline.clone());
                        }
                        crate::schema::raw::property::RawProperty::Ref(
                            r#ref,
                        ) => {
                            upserts.refs.insert(name.clone(), r#ref.clone());
                        }
                    }
                }
            }
        } else {
            if let Some(prop) = raw.properties().get(name) {
                match prop {
                    crate::schema::raw::property::RawProperty::Inline(
                        inline,
                    ) => {
                        upserts.inline.insert(name.clone(), inline.clone());
                    }
                    crate::schema::raw::property::RawProperty::Ref(r#ref) => {
                        upserts.refs.insert(name.clone(), r#ref.clone());
                    }
                }
            }
        }
    }

    for name in old_hashes.keys() {
        if !current_hashes.contains_key(name) {
            removed.push(name.clone());
        }
    }

    SchemaPropertyDelta {
        upserts,
        removed,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_change_kind_unchanged_can_update() {
        assert!(ExtendsChangeKind::Unchanged.can_update());
        assert!(!ExtendsChangeKind::Unchanged.requires_merge());
    }

    #[test]
    fn extends_change_kind_root_to_child_requires_merge() {
        assert!(ExtendsChangeKind::RootToChild.requires_merge());
        assert!(!ExtendsChangeKind::RootToChild.can_update());
    }

    #[test]
    fn extends_change_kind_child_to_root_can_update() {
        assert!(ExtendsChangeKind::ChildToRoot.can_update());
        assert!(!ExtendsChangeKind::ChildToRoot.requires_merge());
    }

    #[test]
    fn extends_change_kind_rewired_requires_merge() {
        assert!(ExtendsChangeKind::Rewired.requires_merge());
        assert!(!ExtendsChangeKind::Rewired.can_update());
    }

    #[test]
    fn excludes_delta_empty_when_no_changes() {
        let delta = ExcludesDelta::default();
        assert!(delta.is_empty());
    }

    #[test]
    fn excludes_delta_not_empty_when_added() {
        let delta = ExcludesDelta {
            added: vec![PropertyName::try_new("test").unwrap()],
            removed: vec![],
        };
        assert!(!delta.is_empty());
    }

    #[test]
    fn property_delta_empty_when_no_changes() {
        let delta = SchemaPropertyDelta::default();
        assert!(delta.is_empty());
    }

    #[test]
    fn property_delta_not_empty_when_removed() {
        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts::default(),
            removed: vec![PropertyName::try_new("test").unwrap()],
        };
        assert!(!delta.is_empty());
    }
}
