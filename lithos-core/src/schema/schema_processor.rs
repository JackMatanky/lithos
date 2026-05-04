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
//! - `AllMissing` owns the initial scan data for new schemas only.
//! - `Present` owns the discovery graph plus payloads for schemas with views.
//! - `ContentBranch` variants carry the data needed for parsing or refresh.
//! - `FileParsedBranch::StaleParsed` guarantees parsed raw content.
//! - `InheritanceBranch::New` guarantees parsed raw content for new schemas.
//! - `AnalysisBranch` determines rebuild vs refresh with required payloads.
//!
//! # Usage
//!
//! ```ignore
//! let discovery = discover(&context, &repo, &source)?;
//! let schemas = match discovery {
//!     DiscoveryBranch::AllMissing(all_missing) => {
//!         let parsed_new = all_missing.parse(&source)?;
//!         parsed_new
//!             .build_new_graph()?
//!             .construct_new_schemas(&repo, &bank)?
//!     }
//!     DiscoveryBranch::HasPresent(present) => {
//!         let compared = present.compare(&source, None)?;
//!         let parsed = compared.parse(&source)?;
//!         parsed
//!             .build_graph()?
//!             .analyze_properties(&source, None)?
//!             .refresh_metadata(&repo)?
//!             .construct_schemas(&repo, &bank)?
//!             .complete(&repo)?
//!             .into_schemas()
//!     }
//! };
//! ```

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::Arc,
};

pub(crate) use crate::schema::delta::{ExcludesDelta, PropertyDelta};
use crate::{
    fs::{FileInfo, FsReader, RelativePath},
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        delta::PropertyDeltaEngine,
        discovery::{DiscoveredFile, DiscoveredView, SchemaFileKind},
        error::{
            SchemaError, SchemaFileError, SchemaIngestionError,
            SchemaLoaderError, SchemaRepositoryError,
        },
        expander::RefExpander,
        identifier::{SchemaId, SchemaName},
        index::{NameIdPairs, SchemaIndex},
        inheritance::{InheritanceGraph, ProcessingGraph, SchemaGraphBuilder},
        merger::Merger,
        property::{PropertyMap, PropertyName},
        raw::{RawPropertyMap, RawSchema},
        storage::Repository,
        views::{
            HashRecord, RawPropertyMapHash, RawSchemaView, SchemaVersion,
            contracts::{RawView, RawViewRead, Version, VersionRead},
        },
    },
    support::hash::Blake3Hash,
};

/// Type-safe state for the schema processor pipeline.
#[derive(Debug)]
pub(crate) struct SchemaProcessor<Stage, Status> {
    status: Status,
    _stage: PhantomData<Stage>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  DISCOVERY STAGE PAYLOADS
// ─────────────────────────────────────────────────────────────────────────────

/// Discovery phase: scanning filesystem and comparing with database.
#[derive(Debug)]
pub(crate) struct Discovery;

/// Carries initial scan data for a schema not found in the database.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialScan {
    pub(crate) path: RelativePath,
    pub(crate) info: FileInfo,
}

/// Carries scan data and cached view for a schema found in the database.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FoundPayload {
    pub(crate) path: RelativePath,
    pub(crate) info: FileInfo,
    pub(crate) view: RawSchemaView,
}

/// Carries data for all schemas found in discovery.
#[derive(Debug)]
pub(crate) struct AllMissing {
    pub(crate) new_schemas: NewBatch,
}

/// Map of stable ID to initial scan data for a batch of new schemas.
pub(crate) type NewBatch = HashMap<SchemaId, InitialScan>;

/// Carries data for a mix of existing and new schemas found in discovery.
#[derive(Debug)]
pub(crate) struct Present {
    pub(crate) graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    pub(crate) new_schemas: NewBatch,
    pub(crate) deleted_ids: Vec<SchemaId>,
}

/// Status and payload for a single schema within the processing graph.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessorNode<P> {
    depth: crate::graph::NodeDepth,
    status: NodeStatus,
    relation: ExtendsChangeKind,
    payload: P,
}

impl<P> ProcessorNode<P> {
    #[inline]
    #[must_use]
    pub(crate) fn new(
        status: NodeStatus,
        relation: ExtendsChangeKind,
        payload: P,
    ) -> Self {
        Self {
            depth: crate::graph::NodeDepth::ROOT,
            status,
            relation,
            payload,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn status(&self) -> NodeStatus {
        self.status
    }

    #[inline]
    #[must_use]
    pub(crate) fn relation(&self) -> ExtendsChangeKind {
        self.relation
    }

    #[inline]
    #[must_use]
    pub(crate) fn payload(&self) -> &P {
        &self.payload
    }

    #[inline]
    #[must_use]
    pub(crate) fn payload_mut(&mut self) -> &mut P {
        &mut self.payload
    }
}

// Trait implementations for graph node traits
impl<P> crate::graph::GraphNode for ProcessorNode<P> {
    type Payload = P;

    #[inline]
    fn payload(&self) -> &Self::Payload {
        &self.payload
    }
}

impl<P> crate::graph::GraphNodeMut for ProcessorNode<P> {
    #[inline]
    fn payload_mut(&mut self) -> &mut Self::Payload {
        &mut self.payload
    }
}

impl<P> crate::graph::DiGraphNode for ProcessorNode<P> {
    #[inline]
    fn depth(&self) -> crate::graph::NodeDepth {
        self.depth
    }

    #[inline]
    fn into_parts(self) -> (Self::Payload, crate::graph::NodeDepth) {
        (self.payload, self.depth)
    }

    #[inline]
    fn from_parts(
        payload: Self::Payload,
        depth: crate::graph::NodeDepth,
    ) -> Self {
        Self {
            depth,
            status: NodeStatus::Fresh,
            relation: ExtendsChangeKind::Unchanged,
            payload,
        }
    }
}

impl<P> crate::graph::DiGraphNodeMut for ProcessorNode<P> {
    #[inline]
    fn set_depth(&mut self, depth: crate::graph::NodeDepth) {
        self.depth = depth;
    }
}

/// Processing status of a schema node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeStatus {
    /// Schema is new and needs full parsing and construction.
    New,
    /// Schema is existing and its file content is fresh (matches DB).
    Fresh,
    /// Schema is existing but its file timestamps have changed.
    StaleTimestamps,
    /// Schema is existing but its content hash has changed.
    Stale,
    /// Schema is existing, content is fresh, but its parent/bank has changed.
    StaleBankReferences,
    /// Schema has been deleted from filesystem.
    Deleted,
    /// Node is in an invalid state for the current stage.
    Corrupt,
}

/// Type of change in inheritance relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtendsChangeKind {
    /// inheritance path is identical to the cached version.
    Unchanged,
    /// `extends` field changed from None to Some, or parent name changed.
    Rewired,
    /// `extends` field changed from Some to None (now a root schema).
    Promoted,
}

impl ExtendsChangeKind {
    #[inline]
    #[must_use]
    pub(crate) const fn requires_merge(self) -> bool {
        matches!(self, Self::Rewired)
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn can_update(self) -> bool {
        matches!(self, Self::Unchanged | Self::Promoted)
    }
}

/// Result of a discovery run, branching based on whether existing schemas
/// were found.
#[derive(Debug)]
pub(crate) enum DiscoveryBranch {
    /// Cold-start: No existing schemas found in database.
    AllMissing(SchemaProcessor<Discovery, AllMissing>),
    /// Incremental: Some existing schemas were found.
    HasPresent(SchemaProcessor<Discovery, Present>),
}

// ─────────────────────────────────────────────────────────────────────────────
//  TRANSITION PAYLOADS
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeletedPayload;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FreshPayload {
    path: RelativePath,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SuspectPayload {
    path: RelativePath,
    info: FileInfo,
    content_str: Box<str>,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StalePayload {
    path: RelativePath,
    info: FileInfo,
    content_str: Box<str>,
    content_hash: Blake3Hash,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewParsedPayload {
    path: RelativePath,
    info: FileInfo,
    content_hash: Blake3Hash,
    raw: RawSchema,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleParsedPayload {
    path: RelativePath,
    info: FileInfo,
    content_hash: Blake3Hash,
    raw: RawSchema,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TimestampBranch {
    Match(FoundPayload),
    Mismatch(SuspectPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContentBranch {
    Match(SuspectPayload),
    Mismatch(StalePayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ComparedPayload {
    Fresh(FreshPayload),
    StaleTimestamps(FoundPayload),
    StaleBankReferences(StalePayload),
    Stale(StalePayload),
}

impl ComparedPayload {
    #[inline]
    #[must_use]
    pub(crate) fn to_node_status(&self) -> NodeStatus {
        match self {
            Self::Fresh(_) => NodeStatus::Fresh,
            Self::StaleTimestamps(_) => NodeStatus::StaleTimestamps,
            Self::StaleBankReferences(_) => NodeStatus::StaleBankReferences,
            Self::Stale(_) => NodeStatus::Stale,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "branch payloads are large by design; boxing adds indirection"
)]
pub(crate) enum FileParsedBranch {
    Fresh(FreshPayload),
    StaleTimestamps(FoundPayload),
    StaleParsed(StaleParsedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InheritanceBranch {
    New(NewParsedPayload),
    Fresh(FreshPayload),
    StaleTimestamps(FoundPayload),
    StaleParsed(StaleParsedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RefreshNodePayload {
    path: RelativePath,
    info: FileInfo,
    content_hash: Blake3Hash,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RebuildNodePayload {
    path: RelativePath,
    info: FileInfo,
    content_hash: Blake3Hash,
    raw: RawSchema,
    view: RawSchemaView,
    excludes_delta: Option<ExcludesDelta>,
    property_delta: Option<PropertyDelta>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UpdateNodePayload {
    path: RelativePath,
    info: FileInfo,
    content_hash: Blake3Hash,
    raw: RawSchema,
    view: RawSchemaView,
    property_delta: PropertyDelta,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnalysisBranch {
    Refresh(RefreshNodePayload),
    Rebuild(RebuildNodePayload),
    #[expect(dead_code, reason = "reserved for incremental property updates")]
    Update(UpdateNodePayload),
}

impl AnalysisBranch {
    pub(crate) fn as_refresh_mut(&mut self) -> Option<&mut RefreshNodePayload> {
        match self {
            Self::Refresh(p) => Some(p),
            _ => None,
        }
    }

    pub(crate) fn as_rebuild_mut(&mut self) -> Option<&mut RebuildNodePayload> {
        match self {
            Self::Rebuild(p) => Some(p),
            _ => None,
        }
    }

    pub(crate) fn as_rebuild(&self) -> Option<&RebuildNodePayload> {
        match self {
            Self::Rebuild(p) => Some(p),
            _ => None,
        }
    }

    pub(crate) fn as_update(&self) -> Option<&UpdateNodePayload> {
        match self {
            Self::Update(p) => Some(p),
            _ => None,
        }
    }
}

// PipelinePayload migration rules:
// - The graph payload type stays stable across stages.
// - Stage transitions switch payload variants, not graph generic types.
// - Deleted nodes may intentionally pass through selected stages.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PipelinePayload {
    Present(PresentPayload),
    Compared(ComparedPayload),
    FileParsed(FileParsedBranch),
    Inheritance(InheritanceBranch),
    Analysis(AnalysisBranch),
    NewParsed(NewParsedPayload),
    Deleted(DeletedPayload),
}

impl PipelinePayload {
    pub(crate) fn variant_name(&self) -> &'static str {
        match self {
            Self::Present(_) => "Present",
            Self::Compared(_) => "Compared",
            Self::FileParsed(_) => "FileParsed",
            Self::Inheritance(_) => "Inheritance",
            Self::Analysis(_) => "Analysis",
            Self::NewParsed(_) => "NewParsed",
            Self::Deleted(_) => "Deleted",
        }
    }

    pub(crate) fn as_analysis_mut(&mut self) -> Option<&mut AnalysisBranch> {
        match self {
            Self::Analysis(p) => Some(p),
            _ => None,
        }
    }

    pub(crate) fn as_analysis(&self) -> Option<&AnalysisBranch> {
        match self {
            Self::Analysis(p) => Some(p),
            _ => None,
        }
    }

    pub(crate) fn as_inheritance(&self) -> Option<&InheritanceBranch> {
        match self {
            Self::Inheritance(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PresentPayload {
    Found(FoundPayload),
    Deleted(DeletedPayload),
}

// ─────────────────────────────────────────────────────────────────────────────
//  STAGE DEFINITIONS
// ─────────────────────────────────────────────────────────────────────────────

/// Identity phase: comparing file attributes with cached metadata.
#[derive(Debug)]
pub(crate) struct Comparison;

/// Semantic phase: file content has been parsed into raw schemas.
#[derive(Debug)]
pub(crate) struct FileParsed;

/// Graph phase: inheritance hierarchy established and validated.
#[derive(Debug)]
pub(crate) struct InheritanceGraphed;

/// Semantic phase: computing property deltas between file and view.
#[derive(Debug)]
pub(crate) struct PropertyAnalysis;

/// Maintenance phase: early commitment of proven metadata.
#[derive(Debug)]
pub(crate) struct Refresh;

/// Domain phase: schemas are reconstructed from resolved property maps.
#[derive(Debug)]
pub(crate) struct Construction;

/// Persistence phase: updated metadata and schemas committed to database.
#[derive(Debug)]
pub(crate) struct Completion;

/// Final stage: returns the collection of processed schemas.
pub(crate) struct Completed {
    schemas: Vec<Arc<Schema>>,
}

impl Completed {
    #[inline]
    #[must_use]
    pub(crate) fn into_schemas(self) -> Vec<Arc<Schema>> {
        self.schemas
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  SHARED DISCOVERY OPERATIONS
// ─────────────────────────────────────────────────────────────────────────────

impl<Stage, Status> SchemaProcessor<Stage, Status> {
    #[inline]
    #[must_use]
    fn transition<NextStage, NextStatus>(
        _stage: NextStage,
        status: NextStatus,
    ) -> SchemaProcessor<NextStage, NextStatus> {
        SchemaProcessor {
            status,
            _stage: PhantomData,
        }
    }

    fn bank_changed(
        view: &RawSchemaView,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> bool {
        property_bank_delta.is_some_and(|delta| {
            RawView::current(view).is_some_and(
                |v: &crate::schema::views::SchemaVersion| {
                    !v.changed_bank_references(delta).is_empty()
                },
            )
        })
    }

    fn schema_stem(
        source: &FsReader,
        path: &std::path::Path,
    ) -> Result<Box<str>, SchemaLoaderError> {
        source
            .filename(path)
            .map(|f| f.basename().to_owned().into_boxed_str())
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)
    }

    fn build_version(
        raw: &RawSchema,
        content_hash: Blake3Hash,
    ) -> Result<crate::schema::views::SchemaVersion, SchemaLoaderError> {
        let property_hashes = raw.properties().compute_hashes();
        let info = *raw.info();
        let hashes = crate::schema::views::HashRecord::new(
            content_hash,
            property_hashes,
        );
        crate::schema::views::SchemaVersion::new(info, hashes, raw)
            .map_err(SchemaLoaderError::Ingestion)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  DISCOVERY STAGE IMPLEMENTATION
// ─────────────────────────────────────────────────────────────────────────────

/// Placeholder for a never-seen state.
#[derive(Debug)]
pub(crate) struct NeverSeen;

/// Placeholder for an incremental state.
#[derive(Debug)]
pub(crate) struct Review;

impl SchemaProcessor<Discovery, NeverSeen> {
    /// Creates discovery branch from discovered files, bypassing redundant I/O.
    ///
    /// This method accepts discovered file data directly from the
    /// [`DiscoveryEngine`](crate::schema::DiscoveryEngine), eliminating
    /// the need to re-read file stats.
    ///
    /// # Design Note
    ///
    /// This method processes a **batch** of schema files because:
    /// - Multiple schemas can exist in a directory
    /// - Batch processing enables efficient bulk operations
    /// - All new schemas need the same initial processing (assign IDs, track
    ///   stats)
    ///
    /// In contrast, the property bank uses a **single-file** API because:
    /// - Only one property bank exists per schema directory (singleton)
    /// - Processing requires a full validation/parsing/comparison pipeline
    /// - The caller already has the specific file reference from discovery
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Matches existing API signature for consistency"
    )]
    pub(crate) fn from_discovery(
        discovered_files: Vec<(&RelativePath, &DiscoveredFile)>,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let mut missing = NewBatch::new();

        for (path, file) in discovered_files {
            if matches!(file.kind, SchemaFileKind::PropertyBank) {
                continue;
            }

            let id = SchemaId::new();
            missing.insert(id, InitialScan {
                path: path.clone(),
                info: file.info,
            });
        }

        Ok(DiscoveryBranch::AllMissing(Self::transition(
            Discovery,
            AllMissing {
                new_schemas: missing,
            },
        )))
    }
}

impl SchemaProcessor<Discovery, AllMissing> {
    pub(crate) fn parse(
        self,
        source: &FsReader,
    ) -> Result<
        SchemaProcessor<InheritanceGraphed, NewParsedBatch>,
        SchemaLoaderError,
    > {
        let AllMissing {
            new_schemas: status_new_schemas,
        } = self.status;
        let transitional = Self::transition(FileParsed, AllMissing {
            new_schemas: status_new_schemas,
        });
        SchemaProcessor::<FileParsed, AllMissing>::parse(transitional, source)
    }
}

impl SchemaProcessor<Discovery, Present> {
    pub(crate) fn compare(
        self,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<SchemaProcessor<FileParsed, Compared>, SchemaLoaderError> {
        let Present {
            graph: status_graph,
            new_schemas: status_new_schemas,
            deleted_ids: status_deleted_ids,
        } = self.status;
        let transitional = Self::transition(Comparison, Present {
            graph: status_graph,
            new_schemas: status_new_schemas,
            deleted_ids: status_deleted_ids,
        });
        SchemaProcessor::<Comparison, Present>::compare(
            transitional,
            source,
            property_bank_delta,
        )
    }
}

impl SchemaProcessor<Discovery, Review> {
    /// Creates discovery branch from discovered files, bypassing redundant I/O.
    ///
    /// This method accepts discovered file data directly from the
    /// [`DiscoveryEngine`](crate::schema::DiscoveryEngine), eliminating
    /// the need to re-query the repository.
    ///
    /// # Design Note
    ///
    /// This method processes a **batch** of schema files because:
    /// - Multiple schemas can exist in a directory
    /// - Batch processing enables efficient bulk operations
    /// - Schema discovery requires checking against the existing graph
    ///
    /// In contrast, the property bank uses a **single-file** API because:
    /// - Only one property bank exists per schema directory (singleton)
    /// - Processing requires a full validation/parsing/comparison pipeline
    /// - The caller already has the specific file reference from discovery
    #[inline]
    #[must_use = "state transitions must be used to continue the pipeline"]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Idiomatic option matching"
    )]
    pub(crate) fn from_discovery(
        discovered_files: Vec<(&RelativePath, &DiscoveredFile)>,
        graph: &InheritanceGraph<()>,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let mut missing = NewBatch::new();
        let mut found: HashMap<SchemaId, FoundPayload> = HashMap::new();

        for (path, file) in discovered_files {
            if matches!(file.kind, SchemaFileKind::PropertyBank) {
                continue;
            }

            match (&file.kind, &file.view) {
                (
                    SchemaFileKind::Schema(id),
                    Some(DiscoveredView::Schema(view)),
                ) => {
                    found.insert(*id, FoundPayload {
                        path: path.clone(),
                        info: file.info,
                        view: view.clone(),
                    });
                }
                (
                    SchemaFileKind::Schema(_),
                    Some(DiscoveredView::PropertyBank(_)),
                ) => {
                    return Err(SchemaLoaderError::Ingestion(
                        SchemaIngestionError::File(
                            SchemaFileError::FileSystem {
                                reason: format!(
                                    "schema file at '{path}' has a property \
                                     bank view (kind/view mismatch)"
                                )
                                .into(),
                            },
                        ),
                    ));
                }
                _ => {
                    let id = SchemaId::new();
                    missing.insert(id, InitialScan {
                        path: path.clone(),
                        info: file.info,
                    });
                }
            }
        }

        let mut deleted_ids = Vec::new();
        for id in graph.topo_order() {
            if !found.contains_key(id) && !missing.contains_key(id) {
                deleted_ids.push(*id);
            }
        }

        if found.is_empty() {
            Ok(DiscoveryBranch::AllMissing(Self::transition(
                Discovery,
                AllMissing {
                    new_schemas: missing,
                },
            )))
        } else {
            let present_graph =
                Self::build_present_graph(graph, &found, &deleted_ids);
            Ok(DiscoveryBranch::HasPresent(Self::transition(
                Discovery,
                Present {
                    graph: present_graph,
                    new_schemas: missing,
                    deleted_ids,
                },
            )))
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    //  Discovery Helpers
    // ═════════════════════════════════════════════════════════════════════
    fn build_present_graph(
        graph: &InheritanceGraph<()>,
        found: &HashMap<SchemaId, FoundPayload>,
        deleted_ids: &[SchemaId],
    ) -> ProcessingGraph<ProcessorNode<PipelinePayload>> {
        let mut builder = SchemaGraphBuilder::new();
        let mut statuses = HashMap::new();

        for (id, _node) in graph.iter() {
            let payload = if let Some(found) = found.get(&id) {
                PipelinePayload::Present(PresentPayload::Found(found.clone()))
            } else if deleted_ids.contains(&id) {
                PipelinePayload::Deleted(DeletedPayload)
            } else {
                continue;
            };

            let status = match payload {
                PipelinePayload::Present(_) => NodeStatus::Fresh,
                PipelinePayload::Deleted(_) => NodeStatus::Deleted,
                PipelinePayload::Compared(_)
                | PipelinePayload::FileParsed(_)
                | PipelinePayload::Inheritance(_)
                | PipelinePayload::Analysis(_)
                | PipelinePayload::NewParsed(_) => NodeStatus::Corrupt,
            };

            statuses.insert(id, status);
            builder.add_node(id, payload);
        }

        for (child_id, &()) in graph.iter() {
            for &parent_id in graph.parents_of(child_id) {
                builder.add_parent(child_id, parent_id);
            }
        }

        builder
            .build::<crate::graph::Node<PipelinePayload>>()
            .map_payload(|id, node| {
                let status =
                    statuses.get(&id).copied().unwrap_or(NodeStatus::Corrupt);
                let (payload, _depth) = node.into_parts();
                Ok::<_, ()>(ProcessorNode::new(
                    status,
                    ExtendsChangeKind::Unchanged,
                    payload,
                ))
            })
            .unwrap()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Comparison, Present> {
    #[expect(
        clippy::excessive_nesting,
        reason = "comparison branch matrix is intentionally explicit"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "comparison stage keeps pipeline steps linear"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
    )]
    pub(crate) fn compare(
        self,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<SchemaProcessor<FileParsed, Compared>, SchemaLoaderError> {
        let Present {
            graph,
            new_schemas,
            deleted_ids,
        } = self.status;

        let mut fresh_ids = Vec::new();
        let mut stale_timestamp_ids = Vec::new();
        let mut stale_bank_ids = Vec::new();
        let mut suspect_ids = Vec::new();

        let next_graph = graph.map_payload(|id, node| {
            let relation = node.relation();
            let payload = match node.payload {
                PipelinePayload::Present(PresentPayload::Found(found)) => {
                    // Check if property bank changes affect this schema
                    let bank_changed =
                        Self::bank_changed(&found.view, property_bank_delta);

                    // Check if timestamps match
                    let timestamps_match = RawViewRead::is_timestamp_match(
                        &found.view,
                        found.info.created_at(),
                        found.info.modified_at(),
                    );

                    if timestamps_match && !bank_changed {
                        fresh_ids.push(id);
                        (
                            NodeStatus::Fresh,
                            PipelinePayload::Compared(ComparedPayload::Fresh(
                                FreshPayload {
                                    path: found.path,
                                    view: found.view,
                                },
                            )),
                        )
                    } else if timestamps_match && bank_changed {
                        // Properties changed in bank but file is same
                        // Transition to SUSPECT to read content and re-hash
                        let content_str = source
                            .read_to_string(found.path.as_path())
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        suspect_ids.push(id);
                        (
                            NodeStatus::StaleBankReferences,
                            PipelinePayload::Compared(
                                ComparedPayload::StaleBankReferences(
                                    StalePayload {
                                        path: found.path,
                                        info: found.info,
                                        content_str: content_str.into(),
                                        content_hash: Blake3Hash::new([0; 32]),
                                        view: found.view,
                                    },
                                ),
                            ),
                        )
                    } else {
                        // Timestamps changed - must read content to verify
                        let content_str = source
                            .read_to_string(found.path.as_path())
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        suspect_ids.push(id);
                        (
                            NodeStatus::StaleTimestamps,
                            PipelinePayload::Compared(
                                ComparedPayload::StaleTimestamps(found),
                            ),
                        )
                    }
                }
                PipelinePayload::Deleted(p) => {
                    (NodeStatus::Deleted, PipelinePayload::Deleted(p))
                }
                _ => (NodeStatus::Corrupt, node.payload),
            };

            Ok::<_, SchemaLoaderError>(ProcessorNode::new(
                payload.0, relation, payload.1,
            ))
        })?;

        let compared = Compared {
            graph: next_graph,
            new_schemas,
            fresh_ids,
            stale_timestamp_ids,
            stale_bank_ids,
            suspect_ids,
            deleted_ids,
        };

        Ok(Self::transition(FileParsed, compared))
    }
}

/// Carries results of the Comparison stage.
#[derive(Debug)]
pub(crate) struct Compared {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_schemas: NewBatch,
    fresh_ids: Vec<SchemaId>,
    stale_timestamp_ids: Vec<SchemaId>,
    stale_bank_ids: Vec<SchemaId>,
    suspect_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  PARSED STAGE IMPLEMENTATION
// ─────────────────────────────────────────────────────────────────────────────

impl SchemaProcessor<FileParsed, Compared> {
    #[expect(
        clippy::too_many_lines,
        reason = "parsing stage keeps I/O and parse logic together"
    )]
    pub(crate) fn parse(
        self,
        source: &FsReader,
    ) -> Result<
        SchemaProcessor<InheritanceGraphed, ParsedBatch>,
        SchemaLoaderError,
    > {
        let mut stale_parsed_ids = Vec::new();
        let mut suspect_ids = self.status.suspect_ids;

        let next_graph = self.status.graph.map_payload(|id, node| {
            let relation = node.relation();
            let payload = match node.payload {
                PipelinePayload::Compared(ComparedPayload::Fresh(p)) => {
                    PipelinePayload::FileParsed(FileParsedBranch::Fresh(p))
                }
                PipelinePayload::Compared(
                    ComparedPayload::StaleTimestamps(payload),
                ) => {
                    let content = source
                        .read_to_string(payload.path.as_path())
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?;
                    let content_hash = Blake3Hash::compute(content.as_bytes());

                    let content_match = RawViewRead::is_content_match(
                        &payload.view,
                        &content_hash,
                    );

                    if content_match {
                        PipelinePayload::FileParsed(
                            FileParsedBranch::StaleTimestamps(payload),
                        )
                    } else {
                        let schema_name =
                            Self::schema_stem(source, payload.path.as_path())?;
                        let info_for_raw = payload.info;
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            payload.path.as_path(), &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_info(info_for_raw)
                        .with_name(schema_name);

                        stale_parsed_ids.push(id);
                        PipelinePayload::FileParsed(
                            FileParsedBranch::StaleParsed(StaleParsedPayload {
                                path: payload.path,
                                info: payload.info,
                                content_hash,
                                raw,
                                view: payload.view,
                            }),
                        )
                    }
                }
                PipelinePayload::Compared(
                    ComparedPayload::StaleBankReferences(payload),
                ) => {
                    let schema_name =
                        Self::schema_stem(source, payload.path.as_path())?;
                    let info_for_raw = payload.info;
                    let raw = FsReader::parse_structured_from_str::<RawSchema>(
                        payload.path.as_path(),
                        &payload.content_str,
                    )
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?
                    .with_info(info_for_raw)
                    .with_name(schema_name);

                    stale_parsed_ids.push(id);
                    PipelinePayload::FileParsed(FileParsedBranch::StaleParsed(
                        StaleParsedPayload {
                            path: payload.path,
                            info: payload.info,
                            content_hash: payload.content_hash,
                            raw,
                            view: payload.view,
                        },
                    ))
                }
                p => p,
            };

            Ok::<_, SchemaLoaderError>(ProcessorNode::new(
                node.status,
                relation,
                payload,
            ))
        })?;

        let mut parsed_new = HashMap::new();
        for (id, payload) in self.status.new_schemas {
            let content = source
                .read_to_string(payload.path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            let content_hash = Blake3Hash::compute(content.as_bytes());
            let schema_name =
                Self::schema_stem(source, payload.path.as_path())?;
            let info_for_raw = payload.info;
            let raw = FsReader::parse_structured_from_str::<RawSchema>(
                payload.path.as_path(),
                &content,
            )
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?
            .with_info(info_for_raw)
            .with_name(schema_name);

            parsed_new.insert(id, NewParsedPayload {
                path: payload.path,
                info: payload.info,
                content_hash,
                raw,
            });
        }

        Ok(Self::transition(InheritanceGraphed, ParsedBatch {
            graph: next_graph,
            new_schemas: parsed_new,
            fresh_ids: self.status.fresh_ids,
            stale_timestamp_ids: self.status.stale_timestamp_ids,
            stale_parsed_ids,
            deleted_ids: self.status.deleted_ids,
        }))
    }
}

/// Carries results of the Parsing stage.
#[derive(Debug)]
pub(crate) struct ParsedBatch {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_schemas: HashMap<SchemaId, NewParsedPayload>,
    fresh_ids: Vec<SchemaId>,
    stale_timestamp_ids: Vec<SchemaId>,
    stale_parsed_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

impl SchemaProcessor<FileParsed, AllMissing> {
    pub(crate) fn parse(
        self,
        source: &FsReader,
    ) -> Result<
        SchemaProcessor<InheritanceGraphed, NewParsedBatch>,
        SchemaLoaderError,
    > {
        let mut parsed_new = HashMap::new();
        for (id, payload) in self.status.new_schemas {
            let content = source
                .read_to_string(payload.path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            let content_hash = Blake3Hash::compute(content.as_bytes());
            let schema_name =
                Self::schema_stem(source, payload.path.as_path())?;
            let info_for_raw = payload.info;
            let raw = FsReader::parse_structured_from_str::<RawSchema>(
                payload.path.as_path(),
                &content,
            )
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?
            .with_info(info_for_raw)
            .with_name(schema_name);

            parsed_new.insert(id, NewParsedPayload {
                path: payload.path,
                info: payload.info,
                content_hash,
                raw,
            });
        }

        Ok(Self::transition(InheritanceGraphed, NewParsedBatch {
            new_schemas: parsed_new,
        }))
    }
}

/// Carries results of the Cold Start Parsing stage.
#[derive(Debug)]
pub(crate) struct NewParsedBatch {
    new_schemas: HashMap<SchemaId, NewParsedPayload>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  INHERITANCE STAGE IMPLEMENTATION
// ─────────────────────────────────────────────────────────────────────────────

impl SchemaProcessor<InheritanceGraphed, NewParsedBatch> {
    pub(crate) fn build_new_graph(
        self,
    ) -> Result<SchemaProcessor<Construction, NewParsedBatch>, SchemaLoaderError>
    {
        // Simple case: all new, no existing graph to compare with
        // Validation of existence happens in Construction
        Ok(Self::transition(Construction, self.status))
    }
}

impl SchemaProcessor<InheritanceGraphed, ParsedBatch> {
    #[expect(
        clippy::too_many_lines,
        reason = "graph rebuild logic is complex but linear"
    )]
    pub(crate) fn build_graph(
        self,
    ) -> Result<
        SchemaProcessor<PropertyAnalysis, GraphedBatch>,
        SchemaLoaderError,
    > {
        let mut builder = SchemaGraphBuilder::new();
        let mut name_to_id = HashMap::new();

        // Register existing nodes from graph
        for (id, node) in self.status.graph.graph().iter() {
            match node.payload() {
                PipelinePayload::FileParsed(FileParsedBranch::Fresh(p)) => {
                    builder.add_node(id, ());
                    name_to_id.insert(p.view.name().to_owned(), id);
                }
                PipelinePayload::FileParsed(
                    FileParsedBranch::StaleTimestamps(p),
                ) => {
                    builder.add_node(id, ());
                    name_to_id.insert(p.view.name().to_owned(), id);
                }
                PipelinePayload::FileParsed(FileParsedBranch::StaleParsed(
                    p,
                )) => {
                    builder.add_node(id, ());
                    name_to_id.insert(p.raw.name().to_owned(), id);
                }
                PipelinePayload::Deleted(_) => {
                    // Deleted nodes don't participate in new graph construction
                }
                _ => {}
            }
        }

        // Register new nodes
        for (id, payload) in &self.status.new_schemas {
            builder.add_node(*id, ());
            name_to_id.insert(payload.raw.name().to_owned(), *id);
        }

        // Build relations
        for (id, node) in self.status.graph.graph().iter() {
            let parent_name = match node.payload() {
                PipelinePayload::FileParsed(FileParsedBranch::Fresh(p)) => {
                    RawView::current(&p.view).and_then(|v| v.extends().cloned())
                }
                PipelinePayload::FileParsed(
                    FileParsedBranch::StaleTimestamps(p),
                ) => {
                    RawView::current(&p.view).and_then(|v| v.extends().cloned())
                }
                PipelinePayload::FileParsed(FileParsedBranch::StaleParsed(
                    p,
                )) => p.raw.extends().cloned(),
                _ => None,
            };

            if let Some(name) = parent_name {
                if let Some(parent_id) = name_to_id.get(name.as_str()) {
                    builder.add_parent(id, *parent_id);
                }
            }
        }

        // New node relations
        for (id, payload) in &self.status.new_schemas {
            if let Some(name) = payload.raw.extends() {
                if let Some(parent_id) = name_to_id.get(name.as_str()) {
                    builder.add_parent(*id, *parent_id);
                }
            }
        }

        let next_graph_base = InheritanceGraph::try_from(builder.build::<()>())
            .map_err(|e| {
                SchemaLoaderError::Resolution(SchemaError::Inheritance(e))
            })?;

        // Detect inheritance changes (Rewired/Promoted)
        let graphed = GraphedBatch {
            graph: self.status.graph,
            new_graph_base: next_graph_base,
            new_schemas: self.status.new_schemas,
            fresh_ids: self.status.fresh_ids,
            stale_timestamp_ids: self.status.stale_timestamp_ids,
            stale_parsed_ids: self.status.stale_parsed_ids,
            deleted_ids: self.status.deleted_ids,
        };

        Ok(Self::transition(PropertyAnalysis, graphed))
    }
}

/// Carries results of the Inheritance stage.
#[derive(Debug)]
pub(crate) struct GraphedBatch {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_graph_base: InheritanceGraph<()>,
    new_schemas: HashMap<SchemaId, NewParsedPayload>,
    fresh_ids: Vec<SchemaId>,
    stale_timestamp_ids: Vec<SchemaId>,
    stale_parsed_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  PROPERTY ANALYSIS STAGE IMPLEMENTATION
// ─────────────────────────────────────────────────────────────────────────────

impl SchemaProcessor<PropertyAnalysis, GraphedBatch> {
    #[expect(
        clippy::too_many_lines,
        reason = "analysis stage orchestrates complex graph transitions"
    )]
    pub(crate) fn analyze_properties(
        self,
        source: &FsReader,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<SchemaProcessor<Refresh, Analyzed>, SchemaLoaderError> {
        let GraphedBatch {
            graph,
            new_graph_base,
            new_schemas,
            fresh_ids,
            stale_timestamp_ids,
            stale_parsed_ids,
            deleted_ids,
        } = self.status;

        let mut rebuild_ids = Vec::new();
        let mut refresh_ids = Vec::new();
        let mut stale_timestamp_ids = stale_timestamp_ids;

        // Perform semantic analysis to find which schemas need full rebuild
        // vs simple metadata refresh.
        let mut affected = HashSet::new();

        // 1. Identify which schemas are directly affected by bank changes
        if let Some(delta) = property_bank_delta {
            for (id, node) in graph.graph().iter() {
                let bank_changed = match node.payload() {
                    PipelinePayload::FileParsed(FileParsedBranch::Fresh(p)) => {
                        Self::bank_changed(&p.view, Some(delta))
                    }
                    PipelinePayload::FileParsed(
                        FileParsedBranch::StaleTimestamps(p),
                    ) => Self::bank_changed(&p.view, Some(delta)),
                    _ => false,
                };

                if bank_changed {
                    affected.insert(id);
                    // Also all children of affected nodes are affected
                    let mut changed_ids = HashSet::new();
                    changed_ids.insert(id);
                    let dag = new_graph_base.to_dag_graph().map_err(|e| {
                        SchemaLoaderError::Resolution(SchemaError::Inheritance(
                            e,
                        ))
                    })?;
                    let subtree = crate::schema::inheritance::affected_subtree(
                        dag.graph(),
                        &changed_ids,
                    );
                    affected.extend(subtree);
                }
            }
        }

        // 2. Identify which schemas have changed inheritance (Rewired/Promoted)
        // (Currently simplified: any rewired/promoted triggers rebuild)
        // ... (inheritance change detection logic would go here)

        // 3. Perform property-level diff for parsed schemas
        let property_bank = PropertyBank::new(); // TODO: Inject real bank

        let next_graph = graph.map_payload(|id, node| {
            let relation = node.relation();
            let node_status = node.status;
            let payload = match node.payload {
                PipelinePayload::FileParsed(FileParsedBranch::Fresh(
                    payload,
                )) => {
                    if affected.contains(&id) {
                        let content = source
                            .read_to_string(payload.path.as_path())
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        let content_hash =
                            Blake3Hash::compute(content.as_bytes());
                        let schema_name =
                            Self::schema_stem(source, payload.path.as_path())?;

                        // Ensure view's current version info is preserved
                        let current_info =
                            *RawView::current(&payload.view).unwrap().info();
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            payload.path.as_path(), &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_info(current_info)
                        .with_name(schema_name);

                        let mut view = payload.view;
                        let version = Self::build_version(&raw, content_hash)?;
                        view.add_version(version);

                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            info: current_info,
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
                        refresh_ids.push(id);
                        (
                            NodeStatus::Fresh,
                            AnalysisBranch::Refresh(RefreshNodePayload {
                                path: payload.path,
                                info: *RawView::current(&payload.view)
                                    .unwrap()
                                    .info(),
                                content_hash: *RawView::current(&payload.view)
                                    .unwrap()
                                    .hashes()
                                    .content(),
                                view: payload.view,
                            }),
                        )
                    }
                }
                PipelinePayload::FileParsed(
                    FileParsedBranch::StaleTimestamps(payload),
                ) => {
                    let info_for_raw =
                        *RawView::current(&payload.view).unwrap().info();
                    let bank_changed =
                        Self::bank_changed(&payload.view, property_bank_delta);

                    if bank_changed {
                        let content = source
                            .read_to_string(payload.path.as_path())
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        let content_hash =
                            Blake3Hash::compute(content.as_bytes());
                        let schema_name =
                            Self::schema_stem(source, payload.path.as_path())?;
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            payload.path.as_path(), &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_info(info_for_raw)
                        .with_name(schema_name);

                        let mut view = payload.view;
                        let version = Self::build_version(&raw, content_hash)?;
                        view.add_version(version);

                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            info: payload.info,
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
                        stale_timestamp_ids.push(id);
                        (
                            NodeStatus::StaleTimestamps,
                            AnalysisBranch::Refresh(RefreshNodePayload {
                                path: payload.path,
                                info: payload.info,
                                content_hash: *RawView::current(&payload.view)
                                    .unwrap()
                                    .hashes()
                                    .content(),
                                view: payload.view,
                            }),
                        )
                    }
                }
                PipelinePayload::FileParsed(FileParsedBranch::StaleParsed(
                    payload,
                )) => {
                    if node_status == NodeStatus::StaleBankReferences {
                        let mut view = payload.view;
                        let version = Self::build_version(
                            &payload.raw,
                            payload.content_hash,
                        )?;
                        view.add_version(version);
                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            info: payload.info,
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
                        let excludes_delta = ExcludesDelta::from_slices(
                            RawView::current(&payload.view).map_or(
                                &[],
                                crate::schema::views::SchemaVersion::excludes,
                            ),
                            payload.raw.excludes(),
                        );

                        let empty_hashes = RawPropertyMapHash::default();
                        let old_property_hashes =
                            RawView::current(&payload.view)
                                .map_or(&empty_hashes, |v| {
                                    v.hashes().properties()
                                });

                        // Eagerly resolve refs during delta computation
                        let expander = RefExpander::new(&property_bank);
                        let property_delta = PropertyDeltaEngine::for_schema(
                            &payload.raw,
                            old_property_hashes,
                        )
                        .diff_schema(&expander)?;

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
                                info: payload.info,
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
                                NodeStatus::Stale,
                                AnalysisBranch::Refresh(RefreshNodePayload {
                                    path: payload.path,
                                    info: payload.info,
                                    content_hash: payload.content_hash,
                                    view: payload.view,
                                }),
                            )
                        }
                    }
                }
                PipelinePayload::Deleted(payload) => {
                    (
                        NodeStatus::Deleted,
                        AnalysisBranch::Refresh(RefreshNodePayload {
                            path: RelativePath::try_from("deleted").unwrap(),
                            info: FileInfo::new(None, None, 0),
                            content_hash: Blake3Hash::new([0; 32]),
                            view: RawSchemaView::new(
                                RelativePath::try_from("deleted").unwrap(),
                                SchemaVersion::new(
                                    FileInfo::new(None, None, 0),
                                    HashRecord::new(
                                        Blake3Hash::new([0; 32]),
                                        Default::default(),
                                    ),
                                    &RawSchema {
                                        version: Default::default(),
                                        name: "deleted".into(),
                                        extends: None,
                                        excludes: vec![],
                                        properties: RawPropertyMap::from_map(
                                            HashMap::new(),
                                        ),
                                        info: FileInfo::new(None, None, 0),
                                    },
                                )
                                .unwrap(),
                            ),
                        }),
                    ) // This path is never actually used for deleted nodes
                }
                unexpected => {
                    return Err(stage_variant_error(
                        "analyze_properties",
                        id,
                        "Inheritance or Deleted",
                        unexpected.variant_name(),
                    ));
                }
            };

            Ok(ProcessorNode::new(
                payload.0,
                relation,
                PipelinePayload::Analysis(payload.1),
            ))
        })?;

        let topo_order = new_graph_base.topo_order();

        for &id in topo_order {
            if affected.contains(&id) && !rebuild_ids.contains(&id) {
                rebuild_ids.push(id);
            }
        }

        Ok(Self::transition(Refresh, Analyzed {
            graph: next_graph,
            refresh_ids,
            stale_timestamp_ids,
            rebuild_ids,
            deleted_ids,
        }))
    }
}

/// Carries results of the Property Analysis stage.
#[derive(Debug)]
pub(crate) struct Analyzed {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    refresh_ids: Vec<SchemaId>,
    stale_timestamp_ids: Vec<SchemaId>,
    rebuild_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  REFRESH STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Refresh, Analyzed> {
    pub(crate) fn refresh_metadata<R>(
        mut self,
        repository: &R,
    ) -> Result<SchemaProcessor<Construction, Analyzed>, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        Self::refresh_cached_views(&mut self.status, repository)?;
        Self::refresh_stale_timestamp_views(&mut self.status, repository)?;
        Self::refresh_rebuild_views(&mut self.status, repository)?;

        // Graph structure unchanged, only payloads mutated in-place
        Ok(Self::transition(Construction, self.status))
    }

    fn refresh_cached_views<R>(
        status: &mut Analyzed,
        repository: &R,
    ) -> Result<(), SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        use crate::schema::views::HashRecord;

        for id in &status.refresh_ids {
            let Some(node) = status.graph.graph_mut().get_mut(*id) else {
                continue;
            };
            let Some(payload) = node
                .payload_mut()
                .as_analysis_mut()
                .and_then(AnalysisBranch::as_refresh_mut)
            else {
                continue;
            };
            let current = RawView::current(&payload.view).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "missing schema metadata in cached view".into(),
                    },
                ))
            })?;
            let info = payload.info;
            let hashes = HashRecord::new(
                payload.content_hash,
                current.hashes().properties().clone(),
            );
            payload.view.add_version(current.with_metadata(info, hashes));
            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }
        Ok(())
    }

    fn refresh_stale_timestamp_views<R>(
        status: &mut Analyzed,
        repository: &R,
    ) -> Result<(), SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        use crate::schema::views::HashRecord;

        for id in &status.stale_timestamp_ids {
            let Some(node) = status.graph.graph_mut().get_mut(*id) else {
                continue;
            };
            match node.payload_mut() {
                PipelinePayload::Analysis(AnalysisBranch::Refresh(payload)) => {
                    let current = RawView::current(&payload.view).ok_or_else(|| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: "missing schema metadata in cached view".into(),
                            },
                        ))
                    })?;
                    let info = payload.info;
                    let hashes = HashRecord::new(
                        *current.hashes().content(),
                        current.hashes().properties().clone(),
                    );
                    payload
                        .view
                        .add_version(current.with_metadata(info, hashes));
                    repository
                        .save_raw_schema_view(*id, &payload.view)
                        .map_err(|e| {
                            let repo_err: SchemaRepositoryError = e.into();
                            SchemaLoaderError::Repository(repo_err)
                        })?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn refresh_rebuild_views<R>(
        status: &mut Analyzed,
        repository: &R,
    ) -> Result<(), SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        for id in &status.rebuild_ids {
            let Some(node) = status.graph.graph_mut().get_mut(*id) else {
                continue;
            };
            let Some(payload) = node
                .payload_mut()
                .as_analysis_mut()
                .and_then(AnalysisBranch::as_rebuild_mut)
            else {
                continue;
            };
            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }
        Ok(())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CONSTRUCTION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Construction, Analyzed> {
    #[expect(
        clippy::too_many_lines,
        reason = "construction keeps fetch/rebuild logic together"
    )]
    pub(crate) fn construct_schemas(
        self,
        repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
        _property_bank: &PropertyBank,
    ) -> Result<SchemaProcessor<Completion, Completed>, SchemaLoaderError> {
        let mut schemas = Vec::new();
        let mut fetched_by_id: HashMap<SchemaId, Schema> = HashMap::new();

        // 1. Fetch schemas that don't need rebuild
        let mut fetch_ids = Vec::new();
        for (id, node) in self.status.graph.graph().iter() {
            if matches!(
                node.status(),
                NodeStatus::Fresh | NodeStatus::StaleTimestamps
            ) {
                fetch_ids.push(id);
            }
        }

        if !fetch_ids.is_empty() {
            // Placeholder: repository would fetch schemas here
            // let fetched = repository.find_schemas_by_ids(&fetch_ids)...
            // fetched_by_id = fetched.into_iter().map(|s| (*s.id(),
            // s)).collect();
        }

        // 2. Rebuild schemas that changed (and their children)
        // ... (complex reconstruction logic)

        Ok(Self::transition(Completion, Completed {
            schemas,
        }))
    }
}

impl SchemaProcessor<Construction, NewParsedBatch> {
    pub(crate) fn construct_new_schemas(
        self,
        _repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
        _property_bank: &PropertyBank,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        // Simple case: all new, just build them
        Ok(Vec::new())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPLETION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Completion, Completed> {
    pub(crate) fn complete(
        self,
        _repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
    ) -> Result<Completed, SchemaLoaderError> {
        // Finalize metadata, commitment of graph, etc.
        Ok(self.status)
    }
}

fn stage_variant_error(
    stage: &str,
    id: SchemaId,
    expected: &str,
    actual: &str,
) -> SchemaLoaderError {
    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
        SchemaFileError::FileSystem {
            reason: format!(
                "Invalid stage payload in {stage} for {id}: expected \
                 {expected}, got {actual}"
            )
            .into(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::raw::property::RawPropertyMapHash;

    fn make_raw_schema(name: &str) -> RawSchema {
        serde_json::from_value(serde_json::json!({
            "$version": "1.0",
            "properties": {}
        }))
        .expect("valid raw schema fixture")
        .with_name(name.into())
        .with_info(FileInfo::new(None, None, 0))
    }

    fn make_view(name: &str, content_hash: Blake3Hash) -> RawSchemaView {
        let raw = make_raw_schema(name);
        let info = crate::fs::FileInfo::new(None, None, 0);
        let hashes = crate::schema::views::HashRecord::new(
            content_hash,
            RawPropertyMapHash::default(),
        );
        let version =
            crate::schema::views::SchemaVersion::new(info, hashes, &raw)
                .expect("valid schema view fixture");
        let path = format!("schemas/{name}.toml");
        RawSchemaView::new(
            crate::fs::RelativePath::try_from(path.as_str())
                .expect("valid relative schema path"),
            version,
        )
    }

    #[test]
    fn compare_transitions_present_to_compared_fresh_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = FsReader::new(temp.path());
        let id = SchemaId::new();
        let path = crate::fs::RelativePath::try_from("schema.toml")
            .expect("valid relative path");

        let view = make_view("schema", Blake3Hash::new([7u8; 32]));

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(
            id,
            ProcessorNode::new(
                NodeStatus::Fresh,
                ExtendsChangeKind::Unchanged,
                PipelinePayload::Present(PresentPayload::Found(FoundPayload {
                    path,
                    info: FileInfo::new(None, None, 0),
                    view,
                })),
            ),
        );

        // ... test implementation
    }
}
