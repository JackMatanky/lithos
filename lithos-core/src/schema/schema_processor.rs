#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pipeline structs expose fields for stage transitions"
)]
#![expect(
    clippy::iter_over_hash_type,
    reason = "ordering is irrelevant for schema graph processing"
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "match ergonomics on borrowed payloads keep code readable"
)]
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
//! Discovery (builder) → Comparison → FileParsed
//!         ↓                 ↓                ↓                 ↓
//! InheritanceGraphed → PropertyAnalysis → Refresh → Construction → Completion
//! ```
//!
//! # Invariants
//! - `Missing` owns the initial scan data for new schemas only.
//! - `Present` owns the discovery graph plus payloads for schemas with views.
//! - `ComparisonBranch` variants carry the data needed for parsing or refresh.
//! - `FileParsedBranch::StaleParsed` guarantees parsed raw content.
//! - `InheritanceBranch::New` guarantees parsed raw content for new schemas.
//! - `AnalysisBranch` determines rebuild vs refresh with required payloads.
//!
//! # Usage
//! ```ignore
//! let discovery = discover(&context, &repo, &source)?;
//! let schemas = match discovery {
//!     DiscoveryBranch::AllMissing(missing) => {
//!         let parsed_new = missing.parse_new_schemas(&source)?;
//!         let parsed = SchemaProcessor::transition(FileParsed, Parsed {
//!             graph: InheritanceGraph { order: vec![], nodes: HashMap::new(), roots: vec![] },
//!             new_schemas: NewBatch::new(),
//!             deleted_ids: vec![],
//!         });
//!         SchemaProcessor::<InheritanceGraphed, Parsed>
//!             ::build_graph(parsed, &parsed_new)?
//!             .analyze_properties(&source, None)?
//!             .refresh_metadata(&repo)?
//!             .construct_schemas(&repo, &bank)?
//!             .complete(&repo)?
//!     }
//!     DiscoveryBranch::HasPresent(present) => {
//!         let compared = present.compare(&source, None)?;
//!         let parsed = compared.parse_stale_schemas(&source)?;
//!         SchemaProcessor::<InheritanceGraphed, Parsed>
//!             ::build_graph(parsed, &NewBatch::new())?
//!             .analyze_properties(&source, None)?
//!             .refresh_metadata(&repo)?
//!             .construct_schemas(&repo, &bank)?
//!             .complete(&repo)?
//!     }
//! };
//! ```

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::PathBuf,
    sync::Arc,
};

use crate::{
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        bank::PropertyBank,
        builder::DiscoveryContext,
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        graph::{
            DagBuilder, InheritanceAccess, InheritanceGraph, InheritanceNode,
        },
        merger::Merger,
        property::{PropertyMap, PropertyName},
        raw::{
            RawFileTimes, RawSchema,
            property::{RawPropertyInline, RawPropertyRef},
        },
        storage::Repository,
        views::RawSchemaView,
    },
};

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE MARKERS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct Comparison;

#[derive(Debug)]
pub(crate) struct Discovery;

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
    pub(crate) const fn requires_merge(self) -> bool {
        matches!(self, Self::Rewired | Self::RootToChild)
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn can_update(self) -> bool {
        matches!(self, Self::Unchanged | Self::ChildToRoot)
    }
}

/// Status of an individual schema node during processing.
///
/// Each status represents what we know about a schema and what operations
/// it needs. Statuses are independent of processor stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(
    dead_code,
    reason = "reserved statuses for incremental pipeline branches"
)]
pub(crate) enum NodeStatus {
    // Discovery / Comparison
    Deleted,
    Fresh,
    StaleTimestamps,
    StaleBankReferences,
    Stale,

    // Parsing / Graphing
    New,
    StaleParsed,

    // Analysis
    ExcludesChanged,
    PropertiesChanged,
    StaleContent,
    Corrupt,
}

// ═════════════════════════════════════════════════════════════════════════════
//  GRAPH NODES
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub(crate) struct SchemaGraphNode<T> {
    pub(crate) id: SchemaId,
    pub(crate) parents: Vec<SchemaId>,
    pub(crate) children: Vec<SchemaId>,
    pub(crate) depth: crate::schema::graph::NodeDepth,
    pub(crate) status: NodeStatus,
    pub(crate) payload: T,
}

impl<T> InheritanceAccess for SchemaGraphNode<T> {
    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn depth(&self) -> crate::schema::graph::NodeDepth {
        self.depth
    }

    fn id(&self) -> SchemaId {
        self.id
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InheritanceGraphNode<T> {
    pub(crate) id: SchemaId,
    pub(crate) parents: Vec<SchemaId>,
    pub(crate) children: Vec<SchemaId>,
    pub(crate) depth: crate::schema::graph::NodeDepth,
    pub(crate) status: NodeStatus,
    pub(crate) extends_change: ExtendsChangeKind,
    pub(crate) payload: T,
}

impl<T> InheritanceAccess for InheritanceGraphNode<T> {
    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn depth(&self) -> crate::schema::graph::NodeDepth {
        self.depth
    }

    fn id(&self) -> SchemaId {
        self.id
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }
}

impl<T> SchemaGraphNode<T> {
    #[inline]
    #[expect(dead_code, reason = "reserved for future graph rehydration")]
    pub(crate) fn with_extends_change(
        self,
        extends_change: ExtendsChangeKind,
    ) -> InheritanceGraphNode<T> {
        InheritanceGraphNode {
            id: self.id,
            parents: self.parents,
            children: self.children,
            depth: self.depth,
            status: self.status,
            extends_change,
            payload: self.payload,
        }
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
pub(crate) struct InitialScan {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialParsed {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PresentPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

pub(crate) type FreshPayload = PresentPayload;
pub(crate) type StaleTimestampsPayload = PresentPayload;
pub(crate) type StaleBankReferencesPayload = PresentPayload;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StalePayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) content_str: Box<str>,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewParsedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleParsedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ComparisonBranch {
    Fresh(FreshPayload),
    StaleTimestamps(StaleTimestampsPayload),
    StaleBankReferences(StaleBankReferencesPayload),
    Stale(StalePayload),
}

#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "branch payloads are large by design; boxing adds indirection"
)]
pub(crate) enum FileParsedBranch {
    Fresh(FreshPayload),
    StaleTimestamps(StaleTimestampsPayload),
    StaleParsed(StaleParsedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InheritanceBranch {
    New(NewParsedPayload),
    Fresh(FreshPayload),
    StaleTimestamps(StaleTimestampsPayload),
    StaleParsed(StaleParsedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnalysisBranch {
    Refresh(RefreshNodePayload),
    Rebuild(RebuildNodePayload),
    #[expect(dead_code, reason = "reserved for incremental property updates")]
    Update(UpdateNodePayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RefreshNodePayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RebuildNodePayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
    pub(crate) excludes_delta: Option<ExcludesDelta>,
    pub(crate) property_delta: Option<SchemaPropertyDelta>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UpdateNodePayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
    pub(crate) property_delta: SchemaPropertyDelta,
}

// ═════════════════════════════════════════════════════════════════════════════
//  SCHEMA PROCESSOR
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct SchemaProcessor<Stage, Status> {
    pub(crate) status: Status,
    _stage: PhantomData<Stage>,
}

impl<Stage, Status> SchemaProcessor<Stage, Status> {
    #[inline]
    pub(crate) fn transition<NextStage, NextStatus>(
        _stage: NextStage,
        status: NextStatus,
    ) -> SchemaProcessor<NextStage, NextStatus> {
        SchemaProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STATE STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct Missing {
    pub(crate) new_schemas: NewBatch<InitialScan>,
}

#[derive(Debug)]
pub(crate) struct GraphMissing {
    pub(crate) new_schemas: NewBatch<InitialScan>,
}

#[derive(Debug)]
#[expect(
    clippy::struct_field_names,
    reason = "field name clarifies the present-schemas map"
)]
pub(crate) struct Present {
    pub(crate) graph: InheritanceGraph<InheritanceNode>,
    pub(crate) present: HashMap<SchemaId, PresentPayload>,
    pub(crate) new_schemas: NewBatch<InitialScan>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct GraphPresent {
    pub(crate) graph: InheritanceGraph<InheritanceNode>,
    pub(crate) present: HashMap<SchemaId, PresentPayload>,
    pub(crate) new_schemas: NewBatch<InitialScan>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Compared {
    pub(crate) graph: InheritanceGraph<SchemaGraphNode<ComparisonBranch>>,
    #[expect(dead_code, reason = "reserved for future batch reconciliation")]
    pub(crate) new_schemas: NewBatch<InitialScan>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Parsed {
    pub(crate) graph: InheritanceGraph<SchemaGraphNode<FileParsedBranch>>,
    #[expect(dead_code, reason = "reserved for future batch reconciliation")]
    pub(crate) new_schemas: NewBatch<InitialParsed>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Graphed {
    pub(crate) graph: InheritanceGraph<InheritanceGraphNode<InheritanceBranch>>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Analyzed {
    pub(crate) graph: InheritanceGraph<InheritanceGraphNode<AnalysisBranch>>,
    pub(crate) refresh_ids: Vec<SchemaId>,
    pub(crate) rebuild_ids: Vec<SchemaId>,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Constructed {
    pub(crate) graph: InheritanceGraph<InheritanceGraphNode<AnalysisBranch>>,
    pub(crate) schemas: Vec<Arc<Schema>>,
    #[expect(dead_code, reason = "retained for future delete handling")]
    pub(crate) deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
#[expect(
    clippy::large_enum_variant,
    reason = "branch payloads carry full pipeline context"
)]
pub(crate) enum DiscoveryBranch {
    AllMissing(SchemaProcessor<Discovery, GraphMissing>),
    HasPresent(SchemaProcessor<Discovery, GraphPresent>),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct NewBatch<T> {
    inner: HashMap<SchemaId, T>,
}

impl<T> NewBatch<T> {
    pub(crate) fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[expect(dead_code, reason = "retained for metrics in future stages")]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    pub(crate) fn insert(&mut self, id: SchemaId, value: T) -> Option<T> {
        self.inner.insert(id, value)
    }

    pub(crate) fn get(&self, id: &SchemaId) -> Option<&T> {
        self.inner.get(id)
    }

    pub(crate) fn contains_key(&self, id: &SchemaId) -> bool {
        self.inner.contains_key(id)
    }

    #[expect(dead_code, reason = "retained for future batch inspection")]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&SchemaId, &T)> {
        self.inner.iter()
    }

    #[expect(dead_code, reason = "retained for future batch extraction")]
    pub(crate) fn into_inner(self) -> HashMap<SchemaId, T> {
        self.inner
    }
}

impl<T> From<HashMap<SchemaId, T>> for NewBatch<T> {
    fn from(inner: HashMap<SchemaId, T>) -> Self {
        Self {
            inner,
        }
    }
}

impl<'batch, T> IntoIterator for &'batch NewBatch<T> {
    type IntoIter = std::collections::hash_map::Iter<'batch, SchemaId, T>;
    type Item = (&'batch SchemaId, &'batch T);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<T> IntoIterator for NewBatch<T> {
    type IntoIter = std::collections::hash_map::IntoIter<SchemaId, T>;
    type Item = (SchemaId, T);

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY HELPERS
// ═════════════════════════════════════════════════════════════════════════════

type FileState =
    (NewBatch<InitialScan>, HashMap<SchemaId, PresentPayload>, Vec<SchemaId>);

impl SchemaProcessor<Discovery, GraphMissing> {
    pub(crate) fn discover(
        context: &DiscoveryContext,
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let files = &context.files;
        let missing = Self::scan_new_files(files, source);

        if missing.is_empty() {
            use crate::schema::error::SchemaFileError;
            return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(SchemaFileError::FileSystem {
                    reason: "no schema files found in directory".into(),
                }),
            ));
        }

        let processor: SchemaProcessor<Discovery, GraphMissing> =
            SchemaProcessor::<Discovery, GraphMissing>::transition(
                Discovery,
                GraphMissing {
                    new_schemas: missing,
                },
            );
        Ok(DiscoveryBranch::AllMissing(processor))
    }

    pub(crate) fn into_file_parsed(
        self,
    ) -> SchemaProcessor<FileParsed, Missing> {
        SchemaProcessor::<FileParsed, Missing>::transition(
            FileParsed,
            Missing {
                new_schemas: self.status.new_schemas,
            },
        )
    }
}

impl SchemaProcessor<Discovery, GraphPresent> {
    pub(crate) fn discover<R>(
        context: &DiscoveryContext,
        repository: &R,
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        let graph = &context.graph;
        let files = &context.files;

        let views_by_path =
            repository.find_raw_schema_views_by_paths(files).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;

        let ids_by_path =
            repository.find_schema_ids_by_paths(files).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;

        let (missing, present, deleted_ids) = Self::classify_file_state(
            files,
            &views_by_path,
            &ids_by_path,
            graph.as_ref(),
            source,
        );

        if missing.is_empty() && present.is_empty() {
            use crate::schema::error::SchemaFileError;
            return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(SchemaFileError::FileSystem {
                    reason: "no schema files found in directory".into(),
                }),
            ));
        }

        if present.is_empty() {
            let processor: SchemaProcessor<Discovery, GraphMissing> =
                SchemaProcessor::<Discovery, GraphMissing>::transition(
                    Discovery,
                    GraphMissing {
                        new_schemas: missing,
                    },
                );
            Ok(DiscoveryBranch::AllMissing(processor))
        } else {
            let Some(graph) = graph.clone() else {
                return Err(SchemaLoaderError::Ingestion(
                    SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: "missing inheritance graph for present \
                                     schemas"
                                .into(),
                        },
                    ),
                ));
            };

            let processor: SchemaProcessor<Discovery, GraphPresent> =
                SchemaProcessor::<Discovery, GraphPresent>::transition(
                    Discovery,
                    GraphPresent {
                        graph,
                        present,
                        new_schemas: missing,
                        deleted_ids,
                    },
                );
            Ok(DiscoveryBranch::HasPresent(processor))
        }
    }

    pub(crate) fn into_comparison(
        self,
    ) -> SchemaProcessor<Comparison, Present> {
        SchemaProcessor::<Comparison, Present>::transition(
            Comparison,
            Present {
                graph: self.status.graph,
                present: self.status.present,
                new_schemas: self.status.new_schemas,
                deleted_ids: self.status.deleted_ids,
            },
        )
    }
}

impl<Status> SchemaProcessor<Discovery, Status> {
    fn scan_new_files(
        files: &[PathBuf],
        source: &FsReader,
    ) -> NewBatch<InitialScan> {
        let mut missing = NewBatch::new();

        for path in files {
            let times = RawFileTimes {
                created_at: source.created_at(path),
                modified_at: source.modified_at(path),
            };
            let id = SchemaId::new();
            missing.insert(id, InitialScan {
                path: path.clone(),
                times,
            });
        }

        missing
    }

    fn classify_file_state(
        files: &[PathBuf],
        views_by_path: &HashMap<PathBuf, RawSchemaView>,
        ids_by_path: &HashMap<PathBuf, SchemaId>,
        graph: Option<&InheritanceGraph<InheritanceNode>>,
        source: &FsReader,
    ) -> FileState {
        let mut missing = NewBatch::new();
        let mut present: HashMap<SchemaId, PresentPayload> = HashMap::new();

        for path in files {
            let times = RawFileTimes {
                created_at: source.created_at(path),
                modified_at: source.modified_at(path),
            };

            if let (Some(view), Some(id)) =
                (views_by_path.get(path), ids_by_path.get(path))
            {
                present.insert(*id, PresentPayload {
                    path: path.clone(),
                    times,
                    view: view.clone(),
                });
            } else {
                let id = SchemaId::new();
                missing.insert(id, InitialScan {
                    path: path.clone(),
                    times,
                });
            }
        }

        let mut deleted_ids = Vec::new();
        if let Some(graph) = graph {
            let file_ids: HashSet<SchemaId> = present.keys().copied().collect();
            for id in graph.nodes.keys() {
                if !file_ids.contains(id) && !missing.contains_key(id) {
                    deleted_ids.push(*id);
                }
            }
        }

        (missing, present, deleted_ids)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Comparison, Present> {
    pub(crate) fn compare(
        self,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<SchemaProcessor<FileParsed, Compared>, SchemaLoaderError> {
        let Present {
            graph,
            present,
            deleted_ids,
            ..
        } = self.status;

        let mut nodes: HashMap<SchemaId, SchemaGraphNode<ComparisonBranch>> =
            HashMap::new();

        for (id, payload) in present {
            let PresentPayload {
                path,
                times,
                view,
            } = payload;

            let node = graph.nodes.get(&id);
            let Some(node) = node else {
                continue;
            };

            let branch = Self::classify_present_payload(
                PresentPayload {
                    path,
                    times,
                    view,
                },
                source,
                property_bank_delta,
            )?;
            let status = Self::status_for_branch(&branch);

            nodes.insert(id, Self::to_graph_node(node, status, branch));
        }

        Ok(Self::transition(FileParsed, Compared {
            graph: InheritanceGraph {
                order: graph.order,
                nodes,
                roots: graph.roots,
            },
            new_schemas: NewBatch::new(),
            deleted_ids,
        }))
    }

    fn classify_present_payload(
        payload: PresentPayload,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<ComparisonBranch, SchemaLoaderError> {
        if Self::timestamps_match(&payload) {
            Ok(Self::branch_for_match(payload, property_bank_delta))
        } else {
            let content_str = source
                .read_to_string(&payload.path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            Ok(Self::check_content(payload, content_str, property_bank_delta))
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    //  Comparison Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn branch_for_match(
        payload: PresentPayload,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> ComparisonBranch {
        if Self::bank_changed(&payload.view, property_bank_delta) {
            ComparisonBranch::StaleBankReferences(payload)
        } else {
            ComparisonBranch::Fresh(payload)
        }
    }

    fn check_content(
        payload: PresentPayload,
        content_str: String,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> ComparisonBranch {
        let PresentPayload {
            path,
            times,
            view,
        } = payload;
        let content_hash = *blake3::hash(content_str.as_bytes()).as_bytes();
        let content_match = view
            .current()
            .is_some_and(|v| v.hashes().is_content_match(&content_hash));

        if content_match {
            let refreshed_payload = PresentPayload {
                path,
                times,
                view,
            };
            if Self::bank_changed(&refreshed_payload.view, property_bank_delta)
            {
                ComparisonBranch::StaleBankReferences(refreshed_payload)
            } else {
                ComparisonBranch::StaleTimestamps(refreshed_payload)
            }
        } else {
            ComparisonBranch::Stale(StalePayload {
                path,
                times,
                content_hash,
                content_str: content_str.into(),
                view,
            })
        }
    }

    fn bank_changed(
        view: &RawSchemaView,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> bool {
        property_bank_delta.is_some_and(|delta| {
            view.current().is_some_and(|v| {
                v.bank_references().values().any(|p| delta.contains(p))
            })
        })
    }

    fn timestamps_match(payload: &PresentPayload) -> bool {
        payload.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                payload.times.created_at,
                payload.times.modified_at,
            )
        })
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on borrowed branch"
    )]
    fn status_for_branch(branch: &ComparisonBranch) -> NodeStatus {
        match branch {
            ComparisonBranch::Fresh(_) => NodeStatus::Fresh,
            ComparisonBranch::StaleTimestamps(_) => NodeStatus::StaleTimestamps,
            ComparisonBranch::StaleBankReferences(_) => {
                NodeStatus::StaleBankReferences
            }
            ComparisonBranch::Stale(_) => NodeStatus::Stale,
        }
    }

    fn to_graph_node(
        node: &InheritanceNode,
        status: NodeStatus,
        payload: ComparisonBranch,
    ) -> SchemaGraphNode<ComparisonBranch> {
        SchemaGraphNode {
            id: node.id,
            parents: node.parents.clone(),
            children: node.children.clone(),
            depth: node.depth,
            status,
            payload,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  FILEPARSED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<FileParsed, Missing> {
    pub(crate) fn parse_new_schemas(
        self,
        source: &FsReader,
    ) -> Result<NewBatch<InitialParsed>, SchemaLoaderError> {
        let Missing {
            new_schemas,
        } = self.status;
        let mut parsed = NewBatch::new();

        for (id, missing) in new_schemas {
            let InitialScan {
                path,
                times,
            } = missing;
            let times_for_raw = times.clone();

            let content = source
                .read_to_string(&path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            let content_hash = *blake3::hash(content.as_bytes()).as_bytes();

            let schema_name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "schema filename missing stem".into(),
                    },
                ))
            })?;
            let raw = FsReader::parse_structured_from_str::<RawSchema>(
                &path, &content,
            )
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?
            .with_file_times(times_for_raw)
            .with_name(schema_name.into());

            parsed.insert(id, InitialParsed {
                path,
                times,
                content_hash,
                raw,
            });
        }

        Ok(parsed)
    }
}

impl SchemaProcessor<FileParsed, Compared> {
    #[expect(
        clippy::too_many_lines,
        reason = "kept linear to mirror staged parsing behavior"
    )]
    pub(crate) fn parse_stale_schemas(
        self,
        source: &FsReader,
    ) -> Result<SchemaProcessor<FileParsed, Parsed>, SchemaLoaderError> {
        let mut nodes = HashMap::new();

        for (id, node) in self.status.graph.nodes {
            let next = match node.payload {
                ComparisonBranch::Stale(payload) => {
                    let schema_name = payload
                        .path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            SchemaLoaderError::Ingestion(
                                SchemaIngestionError::File(
                                    crate::schema::error::SchemaFileError::FileSystem {
                                        reason: "schema filename missing stem"
                                            .into(),
                                    },
                                ),
                            )
                        })?;
                    let times_for_raw = payload.times.clone();
                    let raw = FsReader::parse_structured_from_str::<RawSchema>(
                        &payload.path,
                        &payload.content_str,
                    )
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?
                    .with_file_times(times_for_raw)
                    .with_name(schema_name.into());

                    SchemaGraphNode {
                        id: node.id,
                        parents: node.parents,
                        children: node.children,
                        depth: node.depth,
                        status: NodeStatus::StaleParsed,
                        payload: FileParsedBranch::StaleParsed(
                            StaleParsedPayload {
                                path: payload.path,
                                times: payload.times,
                                content_hash: payload.content_hash,
                                raw,
                                view: payload.view,
                            },
                        ),
                    }
                }
                ComparisonBranch::StaleBankReferences(payload) => {
                    let content = source
                        .read_to_string(&payload.path)
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?;
                    let content_hash =
                        *blake3::hash(content.as_bytes()).as_bytes();
                    let schema_name = payload
                        .path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            SchemaLoaderError::Ingestion(
                                SchemaIngestionError::File(
                                    crate::schema::error::SchemaFileError::FileSystem {
                                        reason: "schema filename missing stem"
                                            .into(),
                                    },
                                ),
                            )
                        })?;
                    let times_for_raw = payload.times.clone();
                    let raw = FsReader::parse_structured_from_str::<RawSchema>(
                        &payload.path,
                        &content,
                    )
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?
                    .with_file_times(times_for_raw)
                    .with_name(schema_name.into());

                    SchemaGraphNode {
                        id: node.id,
                        parents: node.parents,
                        children: node.children,
                        depth: node.depth,
                        status: NodeStatus::StaleBankReferences,
                        payload: FileParsedBranch::StaleParsed(
                            StaleParsedPayload {
                                path: payload.path,
                                times: payload.times,
                                content_hash,
                                raw,
                                view: payload.view,
                            },
                        ),
                    }
                }
                ComparisonBranch::Fresh(payload) => SchemaGraphNode {
                    id: node.id,
                    parents: node.parents,
                    children: node.children,
                    depth: node.depth,
                    status: NodeStatus::Fresh,
                    payload: FileParsedBranch::Fresh(payload),
                },
                ComparisonBranch::StaleTimestamps(payload) => SchemaGraphNode {
                    id: node.id,
                    parents: node.parents,
                    children: node.children,
                    depth: node.depth,
                    status: NodeStatus::StaleTimestamps,
                    payload: FileParsedBranch::StaleTimestamps(payload),
                },
            };

            nodes.insert(id, next);
        }

        Ok(Self::transition(FileParsed, Parsed {
            graph: InheritanceGraph {
                order: self.status.graph.order,
                nodes,
                roots: self.status.graph.roots,
            },
            new_schemas: NewBatch::new(),
            deleted_ids: self.status.deleted_ids,
        }))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  INHERITANCEGRAPHED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<InheritanceGraphed, Parsed> {
    #[expect(
        clippy::too_many_lines,
        reason = "graph construction keeps related steps co-located"
    )]
    #[expect(
        clippy::cognitive_complexity,
        reason = "graph construction mirrors pipeline decisions"
    )]
    pub(crate) fn build_graph(
        parsed: SchemaProcessor<FileParsed, Parsed>,
        new_parsed: &NewBatch<InitialParsed>,
    ) -> Result<SchemaProcessor<InheritanceGraphed, Graphed>, SchemaLoaderError>
    {
        let Parsed {
            graph,
            deleted_ids,
            ..
        } = parsed.status;

        let status_by_id: HashMap<SchemaId, NodeStatus> =
            graph.nodes.iter().map(|(id, node)| (*id, node.status)).collect();

        let base_graph = dehydrate_parsed_graph(&graph);

        let mut name_index: HashMap<SchemaName, SchemaId> = HashMap::new();
        let mut parsed_payloads: HashMap<SchemaId, FileParsedBranch> =
            HashMap::new();

        for (id, node) in &graph.nodes {
            match &node.payload {
                FileParsedBranch::Fresh(payload)
                | FileParsedBranch::StaleTimestamps(payload) => {
                    let name = SchemaName::try_new(payload.view.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    name_index.insert(name, *id);
                    parsed_payloads.insert(*id, node.payload.clone());
                }
                FileParsedBranch::StaleParsed(payload) => {
                    let name = SchemaName::try_new(payload.raw.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    name_index.insert(name, *id);
                    parsed_payloads.insert(*id, node.payload.clone());
                }
            }
        }

        for (id, new) in new_parsed {
            let name = SchemaName::try_new(new.raw.name())
                .map_err(SchemaLoaderError::Resolution)?;
            name_index.insert(name, *id);
        }

        let mut builder =
            DagBuilder::from_existing_graph(&base_graph, name_index.clone());

        for (id, new) in new_parsed {
            builder.add_schema(*id, &new.raw)?;
        }

        for (id, payload) in &parsed_payloads {
            if let FileParsedBranch::StaleParsed(stale) = payload {
                builder.add_schema(*id, &stale.raw)?;
            }
        }

        let mut extends_changes: HashMap<SchemaId, ExtendsChangeKind> =
            HashMap::new();
        for (id, payload) in &parsed_payloads {
            let FileParsedBranch::StaleParsed(stale) = payload else {
                continue;
            };

            let old_parent = base_graph
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
        }

        let finalized_graph = builder.finalize()?;

        let mut nodes = HashMap::new();
        for (id, node) in finalized_graph.nodes {
            let payload = if let Some(new) = new_parsed.get(&id) {
                InheritanceBranch::New(NewParsedPayload {
                    path: new.path.clone(),
                    times: new.times.clone(),
                    content_hash: new.content_hash,
                    raw: new.raw.clone(),
                })
            } else if let Some(existing) = parsed_payloads.get(&id) {
                match existing {
                    FileParsedBranch::Fresh(payload) => {
                        InheritanceBranch::Fresh(payload.clone())
                    }
                    FileParsedBranch::StaleTimestamps(payload) => {
                        InheritanceBranch::StaleTimestamps(payload.clone())
                    }
                    FileParsedBranch::StaleParsed(payload) => {
                        InheritanceBranch::StaleParsed(payload.clone())
                    }
                }
            } else {
                continue;
            };

            let extends_change = extends_changes
                .get(&id)
                .copied()
                .unwrap_or(ExtendsChangeKind::Unchanged);

            let status = match &payload {
                InheritanceBranch::New(_) => NodeStatus::New,
                InheritanceBranch::StaleParsed(_) => NodeStatus::StaleParsed,
                InheritanceBranch::Fresh(_) => NodeStatus::Fresh,
                InheritanceBranch::StaleTimestamps(_) => {
                    NodeStatus::StaleTimestamps
                }
            };
            let status = if status == NodeStatus::StaleParsed {
                status_by_id
                    .get(&id)
                    .copied()
                    .unwrap_or(NodeStatus::StaleParsed)
            } else {
                status
            };

            nodes.insert(id, InheritanceGraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                status,
                extends_change,
                payload,
            });
        }

        Ok(SchemaProcessor::<InheritanceGraphed, Graphed>::transition(
            InheritanceGraphed,
            Graphed {
                graph: InheritanceGraph {
                    order: finalized_graph.order,
                    nodes,
                    roots: finalized_graph.roots,
                },
                deleted_ids,
            },
        ))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  PROPERTYANALYSIS STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<InheritanceGraphed, Graphed> {
    #[expect(
        clippy::excessive_nesting,
        reason = "stage analysis keeps related logic co-located"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "analysis keeps branch logic in one place"
    )]
    pub(crate) fn analyze_properties(
        self,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<SchemaProcessor<PropertyAnalysis, Analyzed>, SchemaLoaderError>
    {
        let Graphed {
            mut graph,
            deleted_ids,
        } = self.status;

        let merge_roots: HashSet<SchemaId> = graph
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                node.extends_change.requires_merge().then_some(*id)
            })
            .collect();
        let affected: HashSet<SchemaId> = if merge_roots.is_empty() {
            HashSet::new()
        } else {
            graph.affected_subtree(&merge_roots)
        };

        let mut refresh_ids = Vec::new();
        let mut rebuild_ids = Vec::new();

        let mut analyzed_nodes = HashMap::new();

        let nodes = std::mem::take(&mut graph.nodes);
        for (id, node) in nodes {
            let node_status = node.status;
            let (status, payload) = match node.payload {
                InheritanceBranch::Fresh(payload) => {
                    let times_for_raw = payload.times.clone();
                    let bank_changed =
                        Self::bank_changed(&payload.view, property_bank_delta);

                    if bank_changed {
                        let content = source
                            .read_to_string(&payload.path)
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        let content_hash =
                            *blake3::hash(content.as_bytes()).as_bytes();
                        let schema_name = Self::schema_stem(&payload.path)?;
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            &payload.path, &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_file_times(times_for_raw)
                        .with_name(schema_name);

                        let mut view = payload.view;
                        let version = Self::build_version(&raw, content_hash)?;
                        view.add_version(version);

                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            times: payload.times,
                            content_hash,
                            raw,
                            view,
                            excludes_delta: None,
                            property_delta: None,
                        };
                        rebuild_ids.push(id);
                        (
                            NodeStatus::StaleBankReferences,
                            AnalysisBranch::Rebuild(rebuild),
                        )
                    } else {
                        let content_hash = payload.view.current().map_or_else(
                            || [0u8; 32],
                            |v| *v.hashes().content(),
                        );
                        refresh_ids.push(id);
                        (
                            NodeStatus::Fresh,
                            AnalysisBranch::Refresh(RefreshNodePayload {
                                path: payload.path,
                                times: payload.times,
                                content_hash,
                                view: payload.view,
                            }),
                        )
                    }
                }
                InheritanceBranch::StaleTimestamps(payload) => {
                    let times_for_raw = payload.times.clone();
                    let bank_changed =
                        Self::bank_changed(&payload.view, property_bank_delta);

                    if bank_changed {
                        let content = source
                            .read_to_string(&payload.path)
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        let content_hash =
                            *blake3::hash(content.as_bytes()).as_bytes();
                        let schema_name = Self::schema_stem(&payload.path)?;
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            &payload.path, &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_file_times(times_for_raw)
                        .with_name(schema_name);

                        let mut view = payload.view;
                        let version = Self::build_version(&raw, content_hash)?;
                        view.add_version(version);

                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            times: payload.times,
                            content_hash,
                            raw,
                            view,
                            excludes_delta: None,
                            property_delta: None,
                        };
                        rebuild_ids.push(id);
                        (
                            NodeStatus::StaleBankReferences,
                            AnalysisBranch::Rebuild(rebuild),
                        )
                    } else {
                        let content_hash = payload.view.current().map_or_else(
                            || [0u8; 32],
                            |v| *v.hashes().content(),
                        );
                        refresh_ids.push(id);
                        (
                            NodeStatus::StaleTimestamps,
                            AnalysisBranch::Refresh(RefreshNodePayload {
                                path: payload.path,
                                times: payload.times,
                                content_hash,
                                view: payload.view,
                            }),
                        )
                    }
                }
                InheritanceBranch::New(payload) => {
                    let filename = payload
                        .path
                        .to_string_lossy()
                        .into_owned()
                        .into_boxed_str();
                    let property_hashes =
                        crate::schema::views::HashMetadata::compute_property_hashes(
                            payload.raw.properties(),
                        );
                    let file_times =
                        crate::schema::views::FileTimesMetadata::new(
                            payload.raw.file_times().created_at,
                            payload.raw.file_times().modified_at,
                        );
                    let hashes = crate::schema::views::HashMetadata::new(
                        payload.content_hash,
                        property_hashes,
                    );
                    let version = crate::schema::views::SchemaVersion::new(
                        file_times,
                        hashes,
                        &payload.raw,
                    )
                    .map_err(SchemaLoaderError::Ingestion)?;
                    let view = RawSchemaView::new(
                        crate::schema::views::Filename::new(filename),
                        version,
                    );
                    let rebuild = RebuildNodePayload {
                        path: payload.path,
                        times: payload.times,
                        content_hash: payload.content_hash,
                        raw: payload.raw,
                        view,
                        excludes_delta: None,
                        property_delta: None,
                    };
                    rebuild_ids.push(id);
                    (NodeStatus::New, AnalysisBranch::Rebuild(rebuild))
                }
                InheritanceBranch::StaleParsed(payload) => {
                    if node_status == NodeStatus::StaleBankReferences {
                        let mut view = payload.view;
                        let version = Self::build_version(
                            &payload.raw,
                            payload.content_hash,
                        )?;
                        view.add_version(version);
                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            times: payload.times,
                            content_hash: payload.content_hash,
                            raw: payload.raw,
                            view,
                            excludes_delta: None,
                            property_delta: None,
                        };
                        rebuild_ids.push(id);
                        (
                            NodeStatus::StaleBankReferences,
                            AnalysisBranch::Rebuild(rebuild),
                        )
                    } else {
                        let excludes_delta = diff_excludes(
                            payload
                                .view
                                .current()
                                .map_or(&[], crate::schema::views::version::SchemaVersion::excludes),
                            payload.raw.excludes(),
                        );

                        let empty_hashes = HashMap::new();
                        let old_property_hashes = payload
                            .view
                            .current()
                            .map_or(&empty_hashes, |v| v.hashes().properties());
                        let property_delta =
                            diff_properties(&payload.raw, old_property_hashes);

                        let needs_rebuild = !excludes_delta.is_empty()
                            || !property_delta.is_empty();

                        if needs_rebuild {
                            let mut view = payload.view;
                            let version = Self::build_version(
                                &payload.raw,
                                payload.content_hash,
                            )?;
                            view.add_version(version);
                            let rebuild = RebuildNodePayload {
                                path: payload.path,
                                times: payload.times,
                                content_hash: payload.content_hash,
                                raw: payload.raw,
                                view,
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
                            };
                            rebuild_ids.push(id);
                            (
                                NodeStatus::Stale,
                                AnalysisBranch::Rebuild(rebuild),
                            )
                        } else {
                            refresh_ids.push(id);
                            (
                                NodeStatus::StaleContent,
                                AnalysisBranch::Refresh(RefreshNodePayload {
                                    path: payload.path,
                                    times: payload.times,
                                    content_hash: payload.content_hash,
                                    view: payload.view,
                                }),
                            )
                        }
                    }
                }
            };

            analyzed_nodes.insert(id, InheritanceGraphNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                status,
                extends_change: node.extends_change,
                payload,
            });
        }

        for id in affected {
            if !rebuild_ids.contains(&id) {
                rebuild_ids.push(id);
            }
        }

        Ok(Self::transition(PropertyAnalysis, Analyzed {
            graph: InheritanceGraph {
                order: graph.order,
                nodes: analyzed_nodes,
                roots: graph.roots,
            },
            refresh_ids,
            rebuild_ids,
            deleted_ids,
        }))
    }

    fn bank_changed(
        view: &RawSchemaView,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> bool {
        property_bank_delta.is_some_and(|delta| {
            view.current().is_some_and(|v| {
                v.bank_references().values().any(|p| delta.contains(p))
            })
        })
    }

    fn schema_stem(
        path: &std::path::Path,
    ) -> Result<Box<str>, SchemaLoaderError> {
        path.file_stem()
            .and_then(|name| name.to_str())
            .map(Into::into)
            .ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "schema filename missing stem".into(),
                    },
                ))
            })
    }

    fn build_version(
        raw: &RawSchema,
        content_hash: [u8; 32],
    ) -> Result<crate::schema::views::SchemaVersion, SchemaLoaderError> {
        let property_hashes =
            crate::schema::views::HashMetadata::compute_property_hashes(
                raw.properties(),
            );
        let file_times = crate::schema::views::FileTimesMetadata::new(
            raw.file_times().created_at,
            raw.file_times().modified_at,
        );
        let hashes = crate::schema::views::HashMetadata::new(
            content_hash,
            property_hashes,
        );
        crate::schema::views::SchemaVersion::new(file_times, hashes, raw)
            .map_err(SchemaLoaderError::Ingestion)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  REFRESH STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<PropertyAnalysis, Analyzed> {
    pub(crate) fn refresh_metadata<R>(
        mut self,
        repository: &R,
    ) -> Result<SchemaProcessor<Refresh, Analyzed>, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        use crate::schema::views::{
            metadata::{FileTimesMetadata, HashMetadata},
            version::SchemaVersion,
        };

        for id in &self.status.refresh_ids {
            let Some(node) = self.status.graph.nodes.get_mut(id) else {
                continue;
            };

            let AnalysisBranch::Refresh(payload) = &mut node.payload else {
                continue;
            };

            let raw = payload
                .view
                .to_raw()
                .map_err(SchemaLoaderError::Ingestion)?
                .ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: "missing raw schema in cached view".into(),
                        },
                    ))
                })?;

            let property_hashes =
                HashMetadata::compute_property_hashes(raw.properties());

            let file_times = FileTimesMetadata::new(
                payload.times.created_at,
                payload.times.modified_at,
            );
            let hashes =
                HashMetadata::new(payload.content_hash, property_hashes);
            let version = SchemaVersion::new(file_times, hashes, &raw)
                .map_err(SchemaLoaderError::Ingestion)?;

            payload.view.add_version(version);

            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }

        for id in &self.status.rebuild_ids {
            let Some(node) = self.status.graph.nodes.get_mut(id) else {
                continue;
            };

            let AnalysisBranch::Rebuild(payload) = &mut node.payload else {
                continue;
            };

            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }

        Ok(Self::transition(Refresh, self.status))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CONSTRUCTION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Refresh, Analyzed> {
    #[expect(
        clippy::too_many_lines,
        reason = "construction keeps fetch/rebuild logic together"
    )]
    pub(crate) fn construct_schemas(
        self,
        repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
        property_bank: &PropertyBank,
    ) -> Result<SchemaProcessor<Construction, Constructed>, SchemaLoaderError>
    {
        use crate::schema::expander::RefExpander;

        let Analyzed {
            graph,
            refresh_ids,
            rebuild_ids,
            deleted_ids,
        } = self.status;

        let mut fetch_ids = refresh_ids.clone();
        let update_ids: Vec<SchemaId> = rebuild_ids
            .iter()
            .filter_map(|id| {
                let node = graph.nodes.get(id)?;
                match &node.payload {
                    AnalysisBranch::Rebuild(payload)
                        if payload.property_delta.is_some()
                            && node.extends_change
                                == ExtendsChangeKind::Unchanged =>
                    {
                        Some(*id)
                    }
                    AnalysisBranch::Update(_) => Some(*id),
                    AnalysisBranch::Rebuild(_) | AnalysisBranch::Refresh(_) => {
                        None
                    }
                }
            })
            .collect();
        for id in update_ids {
            if !fetch_ids.contains(&id) {
                fetch_ids.push(id);
            }
        }
        let mut fetched_by_id: HashMap<SchemaId, Schema> = HashMap::new();
        if !fetch_ids.is_empty() {
            let fetched =
                repository.find_schemas_by_ids(&fetch_ids).map_err(|e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                })?;
            fetched_by_id = fetched.into_iter().map(|s| (*s.id(), s)).collect();
        }

        let expand_pairs: Vec<(SchemaId, RawSchema)> = rebuild_ids
            .iter()
            .filter_map(|id| {
                let node = graph.nodes.get(id)?;
                match &node.payload {
                    AnalysisBranch::Rebuild(payload) => {
                        Some((*id, payload.raw.clone()))
                    }
                    AnalysisBranch::Update(payload) => {
                        Some((*id, payload.raw.clone()))
                    }
                    AnalysisBranch::Refresh(_) => None,
                }
            })
            .collect();

        let expanded_by_id: HashMap<SchemaId, PropertyMap> = if expand_pairs
            .is_empty()
        {
            HashMap::new()
        } else {
            let expander = RefExpander::new(property_bank);
            expand_pairs
                    .into_iter()
                    .map(|(id, raw)| {
                        let refs = raw.properties().ref_entries();
                        let mut expanded_props = expander
                            .expand_properties(&refs)
                            .map_err(SchemaLoaderError::Resolution)?;

                        let mut inline_entries = HashMap::new();
                        for (name, entry) in raw.properties() {
                            let inline = match entry {
                                crate::schema::raw::property::RawProperty::Inline(inline) => inline,
                                crate::schema::raw::property::RawProperty::Ref(_) => continue,
                            };
                            inline_entries.insert(name.clone(), inline.clone());
                        }
                        if !inline_entries.is_empty() {
                            let inline_props = PropertyMap::try_from(inline_entries)
                                .map_err(SchemaLoaderError::Resolution)?;
                            expanded_props.extend(inline_props);
                        }

                        Ok((id, expanded_props))
                    })
                    .collect::<Result<_, SchemaLoaderError>>()?
        };

        let mut changed_schemas = Vec::new();
        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in &graph.order {
            let node = graph.nodes.get(id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing from graph")
                            .into(),
                    },
                ))
            })?;

            let schema = if refresh_ids.contains(id) {
                fetched_by_id.remove(id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "schema {id} not found in refresh cache"
                            )
                            .into(),
                        },
                    ))
                })?
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
                        let repo_err: SchemaRepositoryError = e.into();
                        SchemaLoaderError::Repository(repo_err)
                    })?
                    .ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::File(
                                crate::schema::error::SchemaFileError::FileSystem {
                                    reason: format!(
                                        "schema {id} not found in repository"
                                    )
                                    .into(),
                                },
                            ),
                        )
                    })?
            };

            let is_changed = rebuild_ids.contains(id);
            let schema = Arc::new(schema);

            constructed_cache.insert(*id, (*schema).clone());
            if is_changed {
                changed_schemas.push(schema);
            }
        }

        Ok(Self::transition(Construction, Constructed {
            graph,
            schemas: changed_schemas,
            deleted_ids,
        }))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "incremental construction keeps branching in one place"
    )]
    fn construct_schema_incremental(
        id: SchemaId,
        node: &InheritanceGraphNode<AnalysisBranch>,
        expanded_by_id: &HashMap<SchemaId, PropertyMap>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
        constructed_cache: &HashMap<SchemaId, Schema>,
    ) -> Result<Schema, SchemaLoaderError> {
        let (raw, property_delta) = match &node.payload {
            AnalysisBranch::Rebuild(payload) => {
                (payload.raw.clone(), payload.property_delta.clone())
            }
            AnalysisBranch::Update(payload) => {
                (payload.raw.clone(), Some(payload.property_delta.clone()))
            }
            AnalysisBranch::Refresh(_) => {
                return Err(SchemaLoaderError::Ingestion(
                    SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: "unexpected refresh node in rebuild path"
                                .into(),
                        },
                    ),
                ));
            }
        };

        match (node.extends_change, property_delta) {
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
                                    )
                                    .into(),
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

                let name = SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?;

                Ok(Schema::new(
                    id,
                    name,
                    node.parents.clone(),
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
                    PropertyMap::new()
                } else {
                    Self::collect_parent_properties(
                        &node.parents,
                        constructed_cache,
                        fetched_by_id,
                    )
                };

                let merged = Merger::inherit_properties(
                    &parent_props,
                    expanded,
                    raw.excludes(),
                );

                let name = SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?;

                Ok(Schema::new(
                    id,
                    name,
                    node.parents.clone(),
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

                let name = SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?;

                Ok(Schema::new(
                    id,
                    name,
                    Vec::new(),
                    node.children.clone(),
                    expanded.clone(),
                ))
            }

            (ExtendsChangeKind::Unchanged, None) => {
                if let Some(schema) = fetched_by_id
                    .get(&id)
                    .or_else(|| constructed_cache.get(&id))
                    .cloned()
                {
                    return Ok(schema);
                }

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
                    PropertyMap::new()
                } else {
                    Self::collect_parent_properties(
                        &node.parents,
                        constructed_cache,
                        fetched_by_id,
                    )
                };

                let merged = Merger::inherit_properties(
                    &parent_props,
                    expanded,
                    raw.excludes(),
                );

                let name = SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?;

                Ok(Schema::new(
                    id,
                    name,
                    node.parents.clone(),
                    node.children.clone(),
                    merged,
                ))
            }
        }
    }

    fn collect_parent_properties(
        parent_ids: &[SchemaId],
        constructed_cache: &HashMap<SchemaId, Schema>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
    ) -> PropertyMap {
        let mut merged = PropertyMap::new();
        for parent_id in parent_ids {
            if let Some(schema) = constructed_cache
                .get(parent_id)
                .or_else(|| fetched_by_id.get(parent_id))
            {
                for (name, prop) in schema.properties().iter_named() {
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

impl SchemaProcessor<Construction, Constructed> {
    pub(crate) fn complete(
        self,
        repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        let Constructed {
            graph,
            schemas,
            ..
        } = self.status;

        let owned_schemas: Vec<Schema> =
            schemas.iter().map(|s| (**s).clone()).collect();
        if !owned_schemas.is_empty() {
            repository.save_schemas(&owned_schemas).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;
        }

        let inheritance_graph = dehydrate_graph_to_inheritance(&graph);

        repository.save_topological_graph(&inheritance_graph).map_err(|e| {
            let repo_err: SchemaRepositoryError = e.into();
            SchemaLoaderError::Repository(repo_err)
        })?;

        Ok(schemas)
    }
}

fn dehydrate_graph_to_inheritance(
    graph: &InheritanceGraph<InheritanceGraphNode<AnalysisBranch>>,
) -> InheritanceGraph<InheritanceNode> {
    let mut nodes = HashMap::new();

    for (id, node) in &graph.nodes {
        nodes.insert(*id, InheritanceNode {
            id: node.id,
            parents: node.parents.clone(),
            children: node.children.clone(),
            depth: node.depth,
        });
    }

    InheritanceGraph {
        order: graph.order.clone(),
        nodes,
        roots: graph.roots.clone(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HELPER FUNCTIONS
// ═════════════════════════════════════════════════════════════════════════════

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
        let Some(prop) = raw.properties().get(name) else {
            continue;
        };

        let is_new = old_hashes.get(name).is_none();
        let is_changed = old_hashes.get(name).is_some_and(|old| old != hash);
        if !(is_new || is_changed) {
            continue;
        }

        match prop {
            crate::schema::raw::property::RawProperty::Inline(inline) => {
                upserts.inline.insert(name.clone(), inline.clone());
            }
            crate::schema::raw::property::RawProperty::Ref(r#ref) => {
                upserts.refs.insert(name.clone(), r#ref.clone());
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

fn dehydrate_parsed_graph(
    graph: &InheritanceGraph<SchemaGraphNode<FileParsedBranch>>,
) -> InheritanceGraph<InheritanceNode> {
    let mut nodes = HashMap::new();
    for (id, node) in &graph.nodes {
        nodes.insert(*id, InheritanceNode {
            id: node.id,
            parents: node.parents.clone(),
            children: node.children.clone(),
            depth: node.depth,
        });
    }

    InheritanceGraph {
        order: graph.order.clone(),
        nodes,
        roots: graph.roots.clone(),
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
