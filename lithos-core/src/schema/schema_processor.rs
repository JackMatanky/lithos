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
    path::PathBuf,
    sync::Arc,
};

use crate::{
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        bank::PropertyBank,
        builder::FilesContext,
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        graph::GraphBuilder,
        index::SchemaIndex,
        inheritance::{
            GraphNode, InheritanceGraph, InheritanceNode, NodeAccessor,
            NodeDepth,
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
pub(crate) struct Discovery;

#[derive(Debug)]
pub(crate) struct Comparison;

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
pub(crate) struct Completed;

// ═════════════════════════════════════════════════════════════════════════════
//  DELTA STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ExcludesDelta {
    added: Vec<PropertyName>,
    removed: Vec<PropertyName>,
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
    upserts: SchemaPropertyUpserts,
    removed: Vec<PropertyName>,
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
    inline: HashMap<PropertyName, RawPropertyInline>,
    refs: HashMap<PropertyName, RawPropertyRef>,
}

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

// ═════════════════════════════════════════════════════════════════════════════
//  CORE ENUMS
// ═════════════════════════════════════════════════════════════════════════════

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
pub(crate) struct PreProcessNode<T> {
    id: SchemaId,
    parents: Vec<SchemaId>,
    children: Vec<SchemaId>,
    depth: NodeDepth,
    status: NodeStatus,
    payload: T,
}

impl<T> NodeAccessor for PreProcessNode<T> {
    fn id(&self) -> SchemaId {
        self.id
    }

    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }

    fn depth(&self) -> NodeDepth {
        self.depth
    }
}

impl<T> GraphNode for PreProcessNode<T> {
    fn set_edges(&mut self, parents: Vec<SchemaId>, children: Vec<SchemaId>) {
        self.parents = parents;
        self.children = children;
    }

    fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PostProcessNode<T> {
    id: SchemaId,
    parents: Vec<SchemaId>,
    children: Vec<SchemaId>,
    depth: NodeDepth,
    status: NodeStatus,
    extends_change: ExtendsChangeKind,
    payload: T,
}

impl<T> NodeAccessor for PostProcessNode<T> {
    fn id(&self) -> SchemaId {
        self.id
    }

    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }

    fn depth(&self) -> NodeDepth {
        self.depth
    }
}

impl<T> GraphNode for PostProcessNode<T> {
    fn set_edges(&mut self, parents: Vec<SchemaId>, children: Vec<SchemaId>) {
        self.parents = parents;
        self.children = children;
    }

    fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }
}

impl<T> PreProcessNode<T> {
    #[inline]
    #[expect(dead_code, reason = "reserved for future graph rehydration")]
    pub(crate) fn with_extends_change(
        self,
        extends_change: ExtendsChangeKind,
    ) -> PostProcessNode<T> {
        PostProcessNode {
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
//  PAYLOAD STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PresentPayload {
    Found(FoundPayload),
    Deleted(DeletedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FoundPayload {
    path: PathBuf,
    times: RawFileTimes,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeletedPayload;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FreshPayload {
    path: PathBuf,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SuspectPayload {
    path: PathBuf,
    times: RawFileTimes,
    content_str: Box<str>,
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StalePayload {
    path: PathBuf,
    times: RawFileTimes,
    content_str: Box<str>,
    content_hash: [u8; 32],
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewParsedPayload {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
    raw: RawSchema,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleParsedPayload {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
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
pub(crate) enum AnalysisBranch {
    Refresh(RefreshNodePayload),
    Rebuild(RebuildNodePayload),
    #[expect(dead_code, reason = "reserved for incremental property updates")]
    Update(UpdateNodePayload),
}

impl AnalysisBranch {
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on &mut self keep accessors concise"
    )]
    fn as_refresh_mut(&mut self) -> Option<&mut RefreshNodePayload> {
        match self {
            Self::Refresh(payload) => Some(payload),
            Self::Rebuild(_) | Self::Update(_) => None,
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on &mut self keep accessors concise"
    )]
    fn as_rebuild_mut(&mut self) -> Option<&mut RebuildNodePayload> {
        match self {
            Self::Rebuild(payload) => Some(payload),
            Self::Refresh(_) | Self::Update(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RefreshNodePayload {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
    view: RawSchemaView,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RebuildNodePayload {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
    raw: RawSchema,
    view: RawSchemaView,
    excludes_delta: Option<ExcludesDelta>,
    property_delta: Option<SchemaPropertyDelta>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UpdateNodePayload {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
    raw: RawSchema,
    view: RawSchemaView,
    property_delta: SchemaPropertyDelta,
}

// ═════════════════════════════════════════════════════════════════════════════
//  BATCH TYPES
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct NewBatch<T>(HashMap<SchemaId, T>);

impl<T> NewBatch<T> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn insert(&mut self, id: SchemaId, value: T) -> Option<T> {
        self.0.insert(id, value)
    }

    pub(crate) fn get(&self, id: &SchemaId) -> Option<&T> {
        self.0.get(id)
    }

    pub(crate) fn contains_key(&self, id: &SchemaId) -> bool {
        self.0.contains_key(id)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&SchemaId, &T)> {
        self.0.iter()
    }
}

impl<T> From<HashMap<SchemaId, T>> for NewBatch<T> {
    fn from(inner: HashMap<SchemaId, T>) -> Self {
        Self(inner)
    }
}

impl<'batch, T> IntoIterator for &'batch NewBatch<T> {
    type IntoIter = std::collections::hash_map::Iter<'batch, SchemaId, T>;
    type Item = (&'batch SchemaId, &'batch T);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> IntoIterator for NewBatch<T> {
    type IntoIter = std::collections::hash_map::IntoIter<SchemaId, T>;
    type Item = (SchemaId, T);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialScan {
    path: PathBuf,
    times: RawFileTimes,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialRead {
    path: PathBuf,
    times: RawFileTimes,
    content_str: Box<str>,
    content_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialParsed {
    path: PathBuf,
    times: RawFileTimes,
    content_hash: [u8; 32],
    raw: RawSchema,
}

// ═════════════════════════════════════════════════════════════════════════════
//  SCHEMA PROCESSOR
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) struct SchemaProcessor<Stage, Status> {
    status: Status,
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
pub(crate) struct NeverSeen;

#[derive(Debug)]
pub(crate) struct Review;

#[derive(Debug)]
pub(crate) struct AllMissing {
    new_schemas: NewBatch<InitialScan>,
}

#[derive(Debug)]
pub(crate) struct NewParsed {
    new_schemas: NewBatch<InitialParsed>,
}

#[derive(Debug)]
pub(crate) struct Present {
    graph: InheritanceGraph<PreProcessNode<PresentPayload>>,
    new_schemas: NewBatch<InitialScan>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "ID vectors for incremental pipeline optimization"
)]
pub(crate) struct Compared {
    graph: InheritanceGraph<PreProcessNode<ComparedPayload>>,
    new_schemas: NewBatch<InitialRead>,
    fresh: Vec<SchemaId>,
    stale_timestamps: Vec<SchemaId>,
    stale_refs: Vec<SchemaId>,
    stale: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Parsed {
    graph: InheritanceGraph<PreProcessNode<FileParsedBranch>>,
    new_schemas: NewBatch<InitialParsed>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Graphed {
    graph: InheritanceGraph<PostProcessNode<InheritanceBranch>>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Analyzed {
    graph: InheritanceGraph<PostProcessNode<AnalysisBranch>>,
    refresh_ids: Vec<SchemaId>,
    rebuild_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Constructed {
    graph: InheritanceGraph<PostProcessNode<AnalysisBranch>>,
    schemas: Vec<Arc<Schema>>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct NewBuild {
    graph: InheritanceGraph<PreProcessNode<NewParsedPayload>>,
}

#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum DiscoveryBranch {
    AllMissing(SchemaProcessor<FileParsed, AllMissing>),
    HasPresent(SchemaProcessor<Comparison, Present>),
}

// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY HELPERS
// ═════════════════════════════════════════════════════════════════════════════

type FileState =
    (NewBatch<InitialScan>, HashMap<SchemaId, FoundPayload>, Vec<SchemaId>);
type ViewMaps = (HashMap<PathBuf, RawSchemaView>, HashMap<PathBuf, SchemaId>);

impl SchemaProcessor<Discovery, NeverSeen> {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "signature matches discovery variants with fallible paths"
    )]
    pub(crate) fn discover(
        context: &FilesContext,
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let files = &context.files;
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

        Ok(DiscoveryBranch::AllMissing(Self::transition(
            FileParsed,
            AllMissing {
                new_schemas: missing,
            },
        )))
    }
}

impl SchemaProcessor<Discovery, Review> {
    pub(crate) fn discover<R>(
        context: &FilesContext,
        graph: &InheritanceGraph<InheritanceNode>,
        repository: &R,
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        let files = &context.files;

        let (views_by_path, ids_by_path) =
            Self::fetch_view_maps(repository, files)?;

        let (missing, found, deleted_ids) = Self::classify_file_state(
            files,
            &views_by_path,
            &ids_by_path,
            Some(graph),
            source,
        );

        if found.is_empty() {
            Ok(DiscoveryBranch::AllMissing(Self::transition(
                FileParsed,
                AllMissing {
                    new_schemas: missing,
                },
            )))
        } else {
            let present_graph =
                Self::build_present_graph(graph, &found, &deleted_ids);
            Ok(DiscoveryBranch::HasPresent(Self::transition(
                Comparison,
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
    fn fetch_view_maps<R: Repository>(
        repository: &R,
        files: &[PathBuf],
    ) -> Result<ViewMaps, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
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

        Ok((views_by_path, ids_by_path))
    }

    fn classify_file_state(
        files: &[PathBuf],
        views_by_path: &HashMap<PathBuf, RawSchemaView>,
        ids_by_path: &HashMap<PathBuf, SchemaId>,
        graph: Option<&InheritanceGraph<InheritanceNode>>,
        source: &FsReader,
    ) -> FileState {
        let mut missing = NewBatch::new();
        let mut found: HashMap<SchemaId, FoundPayload> = HashMap::new();

        for path in files {
            let times = RawFileTimes {
                created_at: source.created_at(path),
                modified_at: source.modified_at(path),
            };

            if let (Some(view), Some(id)) =
                (views_by_path.get(path), ids_by_path.get(path))
            {
                found.insert(*id, FoundPayload {
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
            let file_ids: HashSet<SchemaId> = found.keys().copied().collect();
            for id in graph.order() {
                if !file_ids.contains(id) && !missing.contains_key(id) {
                    deleted_ids.push(*id);
                }
            }
        }

        (missing, found, deleted_ids)
    }

    fn build_present_graph(
        graph: &InheritanceGraph<InheritanceNode>,
        found: &HashMap<SchemaId, FoundPayload>,
        deleted_ids: &[SchemaId],
    ) -> InheritanceGraph<PreProcessNode<PresentPayload>> {
        let mut nodes = HashMap::new();
        let deleted_set: HashSet<SchemaId> =
            deleted_ids.iter().copied().collect();

        for id in graph.order() {
            let Some(node) = graph.nodes().get(id) else {
                continue;
            };

            let payload = if let Some(found) = found.get(id) {
                PresentPayload::Found(found.clone())
            } else if deleted_set.contains(id) {
                PresentPayload::Deleted(DeletedPayload)
            } else {
                continue;
            };

            let status = match payload {
                PresentPayload::Found(_) => NodeStatus::Fresh,
                PresentPayload::Deleted(_) => NodeStatus::Deleted,
            };

            nodes.insert(*id, PreProcessNode {
                id: node.id(),
                parents: node.parents().to_vec(),
                children: node.children().to_vec(),
                depth: node.depth(),
                status,
                payload,
            });
        }

        InheritanceGraph::from_parts(
            nodes,
            graph.order().to_vec(),
            graph.roots().to_vec(),
        )
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Comparison, Present> {
    #[expect(
        clippy::too_many_lines,
        reason = "comparison stage keeps pipeline steps linear"
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

        let (mut graph_nodes, graph_order, graph_roots) = graph.into_parts();
        let mut nodes: HashMap<SchemaId, PreProcessNode<ComparedPayload>> =
            HashMap::new();

        let mut fresh_ids = Vec::new();
        let mut stale_ts_ids = Vec::new();
        let mut stale_ref_ids = Vec::new();
        let mut stale_ids = Vec::new();

        let bank_affected =
            Self::collect_bank_affected_ids(&graph_nodes, property_bank_delta);

        for graph_id in &graph_order {
            let Some(node) = graph_nodes.remove(graph_id) else {
                continue;
            };

            let PreProcessNode {
                id: node_id,
                parents,
                children,
                depth,
                payload,
                ..
            } = node;

            let PresentPayload::Found(found_payload) = payload else {
                continue;
            };

            let is_bank_affected = bank_affected
                .as_ref()
                .is_some_and(|ids| ids.contains(&node_id));
            let comparison_payload =
                match Self::check_timestamps(found_payload, source)? {
                    TimestampBranch::Match(matched_payload) => {
                        if is_bank_affected {
                            let content_str = source
                                .read_to_string(&matched_payload.path)
                                .map_err(SchemaIngestionError::from)
                                .map_err(SchemaLoaderError::Ingestion)?;
                            let content_hash =
                                *blake3::hash(content_str.as_bytes())
                                    .as_bytes();
                            ComparedPayload::StaleBankReferences(StalePayload {
                                path: matched_payload.path,
                                times: matched_payload.times,
                                content_str: content_str.into(),
                                content_hash,
                                view: matched_payload.view,
                            })
                        } else {
                            ComparedPayload::Fresh(FreshPayload {
                                path: matched_payload.path,
                                view: matched_payload.view,
                            })
                        }
                    }
                    TimestampBranch::Mismatch(suspect_payload) => {
                        let content_branch =
                            Self::check_content(suspect_payload);
                        match content_branch {
                            ContentBranch::Match(content_payload)
                                if is_bank_affected =>
                            {
                                let content_hash = *blake3::hash(
                                    content_payload.content_str.as_bytes(),
                                )
                                .as_bytes();
                                ComparedPayload::StaleBankReferences(
                                    StalePayload {
                                        path: content_payload.path,
                                        times: content_payload.times,
                                        content_str: content_payload
                                            .content_str,
                                        content_hash,
                                        view: content_payload.view,
                                    },
                                )
                            }
                            ContentBranch::Match(content_payload) => {
                                ComparedPayload::StaleTimestamps(FoundPayload {
                                    path: content_payload.path,
                                    times: content_payload.times,
                                    view: content_payload.view,
                                })
                            }
                            ContentBranch::Mismatch(stale_payload) => {
                                ComparedPayload::Stale(stale_payload)
                            }
                        }
                    }
                };
            let status = Self::status_for_payload(&comparison_payload);

            #[expect(
                clippy::pattern_type_mismatch,
                reason = "match on enum reference for ID tracking"
            )]
            match &comparison_payload {
                ComparedPayload::Fresh(_) => fresh_ids.push(node_id),
                ComparedPayload::StaleTimestamps(_) => {
                    stale_ts_ids.push(node_id);
                }
                ComparedPayload::StaleBankReferences(_) => {
                    stale_ref_ids.push(node_id);
                }
                ComparedPayload::Stale(_) => stale_ids.push(node_id),
            }

            nodes.insert(node_id, PreProcessNode {
                id: node_id,
                parents,
                children,
                depth,
                status,
                payload: comparison_payload,
            });
        }

        let mut new_reads = NewBatch::new();
        let mut new_entries: Vec<_> = new_schemas.into_iter().collect();
        new_entries.sort_by_key(|entry| entry.0);
        for (id, scan) in new_entries {
            let content = source
                .read_to_string(&scan.path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            let content_hash = *blake3::hash(content.as_bytes()).as_bytes();

            new_reads.insert(id, InitialRead {
                path: scan.path,
                times: scan.times,
                content_hash,
                content_str: content.into_boxed_str(),
            });
        }

        Ok(Self::transition(FileParsed, Compared {
            graph: InheritanceGraph::from_parts(
                nodes,
                graph_order,
                graph_roots,
            ),
            new_schemas: new_reads,
            fresh: fresh_ids,
            stale_timestamps: stale_ts_ids,
            stale_refs: stale_ref_ids,
            stale: stale_ids,
            deleted_ids,
        }))
    }

    fn check_timestamps(
        payload: FoundPayload,
        source: &FsReader,
    ) -> Result<TimestampBranch, SchemaLoaderError> {
        let timestamps_match = payload.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                payload.times.created_at,
                payload.times.modified_at,
            )
        });

        if timestamps_match {
            Ok(TimestampBranch::Match(payload))
        } else {
            let content_str = source
                .read_to_string(&payload.path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            Ok(TimestampBranch::Mismatch(SuspectPayload {
                path: payload.path,
                times: payload.times,
                content_str: content_str.into(),
                view: payload.view,
            }))
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    //  Comparison Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn check_content(payload: SuspectPayload) -> ContentBranch {
        let content_hash =
            *blake3::hash(payload.content_str.as_bytes()).as_bytes();
        let content_match = payload
            .view
            .current()
            .is_some_and(|v| v.hashes().is_content_match(&content_hash));

        if content_match {
            ContentBranch::Match(SuspectPayload {
                path: payload.path,
                times: payload.times,
                content_str: payload.content_str,
                view: payload.view,
            })
        } else {
            ContentBranch::Mismatch(StalePayload {
                path: payload.path,
                times: payload.times,
                content_str: payload.content_str,
                content_hash,
                view: payload.view,
            })
        }
    }

    #[expect(
        clippy::iter_over_hash_type,
        reason = "order irrelevant for bank delta scan"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on borrowed payload"
    )]
    fn collect_bank_affected_ids(
        nodes: &HashMap<SchemaId, PreProcessNode<PresentPayload>>,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Option<HashSet<SchemaId>> {
        let delta = property_bank_delta?;
        let mut affected = HashSet::new();
        for (id, node) in nodes {
            let PresentPayload::Found(payload) = &node.payload else {
                continue;
            };
            if payload.view.current().is_some_and(|v| {
                v.bank_references().values().any(|p| delta.contains(p))
            }) {
                affected.insert(*id);
            }
        }
        Some(affected)
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on borrowed branch"
    )]
    fn status_for_payload(payload: &ComparedPayload) -> NodeStatus {
        match payload {
            ComparedPayload::Fresh(_) => NodeStatus::Fresh,
            ComparedPayload::StaleTimestamps(_) => NodeStatus::StaleTimestamps,
            ComparedPayload::StaleBankReferences(_) => {
                NodeStatus::StaleBankReferences
            }
            ComparedPayload::Stale(_) => NodeStatus::Stale,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  FILEPARSED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<FileParsed, AllMissing> {
    pub(crate) fn parse(
        self,
        source: &FsReader,
    ) -> Result<SchemaProcessor<InheritanceGraphed, NewParsed>, SchemaLoaderError>
    {
        let AllMissing {
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

            let schema_name = source
                .basename(&path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
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

        Ok(Self::transition(InheritanceGraphed, NewParsed {
            new_schemas: parsed,
        }))
    }
}

impl SchemaProcessor<FileParsed, Compared> {
    #[expect(
        clippy::too_many_lines,
        reason = "kept linear to mirror staged parsing behavior"
    )]
    pub(crate) fn parse(
        self,
        source: &FsReader,
    ) -> Result<SchemaProcessor<InheritanceGraphed, Parsed>, SchemaLoaderError>
    {
        let Compared {
            graph,
            new_schemas,
            deleted_ids,
            ..
        } = self.status;
        let mut nodes = HashMap::new();

        let (graph_nodes, graph_order, graph_roots) = graph.into_parts();

        let mut parsed_new = NewBatch::new();
        let mut new_entries: Vec<_> = new_schemas.into_iter().collect();
        new_entries.sort_by_key(|entry| entry.0);
        for (id, read) in new_entries {
            let schema_name = source
                .basename(&read.path)
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            let times_for_raw = read.times.clone();
            let raw = FsReader::parse_structured_from_str::<RawSchema>(
                &read.path,
                &read.content_str,
            )
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?
            .with_file_times(times_for_raw)
            .with_name(schema_name.into());

            parsed_new.insert(id, InitialParsed {
                path: read.path,
                times: read.times,
                content_hash: read.content_hash,
                raw,
            });
        }

        let mut node_entries: Vec<_> = graph_nodes.into_iter().collect();
        node_entries.sort_by_key(|entry| entry.0);
        for (id, node) in node_entries {
            let next = match node.payload {
                ComparedPayload::Stale(payload) => {
                    let schema_name = source
                        .basename(&payload.path)
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?;
                    let times_for_raw = payload.times.clone();
                    let raw = FsReader::parse_structured_from_str::<RawSchema>(
                        &payload.path,
                        &payload.content_str,
                    )
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?
                    .with_file_times(times_for_raw)
                    .with_name(schema_name.into());

                    PreProcessNode {
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
                ComparedPayload::StaleBankReferences(payload) => {
                    let schema_name = source
                        .basename(&payload.path)
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?;
                    let times_for_raw = payload.times.clone();
                    let raw = FsReader::parse_structured_from_str::<RawSchema>(
                        &payload.path,
                        &payload.content_str,
                    )
                    .map_err(SchemaIngestionError::from)
                    .map_err(SchemaLoaderError::Ingestion)?
                    .with_file_times(times_for_raw)
                    .with_name(schema_name.into());
                    let content_hash = payload.content_hash;

                    PreProcessNode {
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
                ComparedPayload::Fresh(payload) => PreProcessNode {
                    id: node.id,
                    parents: node.parents,
                    children: node.children,
                    depth: node.depth,
                    status: NodeStatus::Fresh,
                    payload: FileParsedBranch::Fresh(payload),
                },
                ComparedPayload::StaleTimestamps(payload) => PreProcessNode {
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

        Ok(Self::transition(InheritanceGraphed, Parsed {
            graph: InheritanceGraph::from_parts(
                nodes,
                graph_order,
                graph_roots,
            ),
            new_schemas: parsed_new,
            deleted_ids,
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
        self,
    ) -> Result<SchemaProcessor<PropertyAnalysis, Graphed>, SchemaLoaderError>
    {
        let Parsed {
            graph,
            new_schemas,
            deleted_ids,
        } = self.status;

        let mut status_by_id: HashMap<SchemaId, NodeStatus> = HashMap::new();
        for id in graph.order() {
            let Some(node) = graph.nodes().get(id) else {
                continue;
            };
            status_by_id.insert(*id, node.status);
        }

        let base_graph = graph.map_payload(|node| PreProcessNode {
            id: node.id(),
            parents: node.parents().to_vec(),
            children: node.children().to_vec(),
            depth: node.depth(),
            status: node.status,
            payload: (),
        });

        let mut name_index: HashMap<SchemaName, SchemaId> = HashMap::new();
        let mut parsed_payloads: HashMap<SchemaId, FileParsedBranch> =
            HashMap::new();

        for id in graph.order() {
            let Some(node) = graph.nodes().get(id) else {
                continue;
            };
            match node.payload.clone() {
                FileParsedBranch::Fresh(payload) => {
                    let name = SchemaName::try_new(payload.view.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    name_index.insert(name, *id);
                    parsed_payloads
                        .insert(*id, FileParsedBranch::Fresh(payload));
                }
                FileParsedBranch::StaleTimestamps(payload) => {
                    let name = SchemaName::try_new(payload.view.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    name_index.insert(name, *id);
                    parsed_payloads.insert(
                        *id,
                        FileParsedBranch::StaleTimestamps(payload),
                    );
                }
                FileParsedBranch::StaleParsed(payload) => {
                    let name = SchemaName::try_new(payload.raw.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    name_index.insert(name, *id);
                    parsed_payloads
                        .insert(*id, FileParsedBranch::StaleParsed(payload));
                }
            }
        }

        let mut new_ids: Vec<_> =
            new_schemas.iter().map(|(id, _)| *id).collect();
        new_ids.sort();
        for id in &new_ids {
            let Some(new) = new_schemas.get(id) else {
                continue;
            };
            let name = SchemaName::try_new(new.raw.name())
                .map_err(SchemaLoaderError::Resolution)?;
            name_index.insert(name, *id);
        }

        let index = SchemaIndex::from_name_id_pairs(name_index.clone());

        let (base_nodes, base_order, base_roots) = base_graph.into_parts();
        let mut editor = crate::schema::graph::GraphEditor::from_parts(
            base_nodes, base_order, base_roots,
        );

        for id in &deleted_ids {
            editor.delete_node(*id);
        }

        for id in &new_ids {
            let Some(new) = new_schemas.get(id) else {
                continue;
            };
            editor.insert_node(PreProcessNode {
                id: *id,
                parents: Vec::new(),
                children: Vec::new(),
                depth: NodeDepth::ROOT,
                status: NodeStatus::New,
                payload: (),
            });
            let mut parents = Vec::new();
            if let Some(parent_id) =
                new.raw.extends().and_then(|name| index.get_id_by_name(name))
            {
                parents.push(parent_id);
            }
            editor.apply_change(*id, parents);
        }

        let mut extends_changes: HashMap<SchemaId, ExtendsChangeKind> =
            HashMap::new();
        for id in graph.order() {
            let Some(payload) = parsed_payloads.get(id) else {
                continue;
            };

            let old_parent = graph
                .nodes()
                .get(id)
                .and_then(|node| node.parents().first().copied());

            #[expect(
                clippy::pattern_type_mismatch,
                reason = "match ergonomics keep structural checks concise"
            )]
            let new_parent = match payload {
                FileParsedBranch::StaleParsed(stale) => stale
                    .raw
                    .extends()
                    .and_then(|name| index.get_id_by_name(name)),
                FileParsedBranch::Fresh(_)
                | FileParsedBranch::StaleTimestamps(_) => old_parent,
            };

            let change_kind = match (old_parent, new_parent) {
                (None, None) => ExtendsChangeKind::Unchanged,
                (None, Some(_)) => ExtendsChangeKind::RootToChild,
                (Some(_), None) => ExtendsChangeKind::ChildToRoot,
                (Some(old), Some(new)) if old == new => {
                    ExtendsChangeKind::Unchanged
                }
                (Some(_), Some(_)) => ExtendsChangeKind::Rewired,
            };

            if change_kind != ExtendsChangeKind::Unchanged {
                let mut parents = Vec::new();
                if let Some(p) = new_parent {
                    parents.push(p);
                }
                editor.apply_change(*id, parents);
            }

            extends_changes.insert(*id, change_kind);
        }

        let finalized_graph = editor.patch()?;
        let (final_nodes, final_order, final_roots) =
            finalized_graph.into_parts();

        let mut nodes = HashMap::new();
        let mut finalized_entries: Vec<_> = final_nodes.into_iter().collect();
        finalized_entries.sort_by_key(|entry| entry.0);
        for (id, node) in finalized_entries {
            let payload = if let Some(new) = new_schemas.get(&id) {
                InheritanceBranch::New(NewParsedPayload {
                    path: new.path.clone(),
                    times: new.times.clone(),
                    content_hash: new.content_hash,
                    raw: new.raw.clone(),
                })
            } else if let Some(existing) = parsed_payloads.get(&id) {
                match existing.clone() {
                    FileParsedBranch::Fresh(payload) => {
                        InheritanceBranch::Fresh(payload)
                    }
                    FileParsedBranch::StaleTimestamps(payload) => {
                        InheritanceBranch::StaleTimestamps(payload)
                    }
                    FileParsedBranch::StaleParsed(payload) => {
                        InheritanceBranch::StaleParsed(payload)
                    }
                }
            } else {
                continue;
            };

            let extends_change = extends_changes
                .get(&id)
                .copied()
                .unwrap_or(ExtendsChangeKind::Unchanged);

            let (status, payload) = match payload {
                InheritanceBranch::New(payload) => {
                    (NodeStatus::New, InheritanceBranch::New(payload))
                }
                InheritanceBranch::StaleParsed(payload) => (
                    NodeStatus::StaleParsed,
                    InheritanceBranch::StaleParsed(payload),
                ),
                InheritanceBranch::Fresh(payload) => {
                    (NodeStatus::Fresh, InheritanceBranch::Fresh(payload))
                }
                InheritanceBranch::StaleTimestamps(payload) => (
                    NodeStatus::StaleTimestamps,
                    InheritanceBranch::StaleTimestamps(payload),
                ),
            };
            let status = if status == NodeStatus::StaleParsed {
                status_by_id
                    .get(&id)
                    .copied()
                    .unwrap_or(NodeStatus::StaleParsed)
            } else {
                status
            };

            nodes.insert(id, PostProcessNode {
                id: node.id(),
                parents: node.parents().to_vec(),
                children: node.children().to_vec(),
                depth: node.depth(),
                status,
                extends_change,
                payload,
            });
        }

        Ok(SchemaProcessor::<PropertyAnalysis, Graphed>::transition(
            PropertyAnalysis,
            Graphed {
                graph: InheritanceGraph::from_parts(
                    nodes,
                    final_order,
                    final_roots,
                ),
                deleted_ids,
            },
        ))
    }
}

impl SchemaProcessor<InheritanceGraphed, NewParsed> {
    pub(crate) fn build_new_graph(
        self,
    ) -> Result<SchemaProcessor<Construction, NewBuild>, SchemaLoaderError>
    {
        let NewParsed {
            new_schemas,
        } = self.status;

        let mut raw_by_id: HashMap<SchemaId, &RawSchema> = HashMap::new();
        let mut payloads: HashMap<SchemaId, NewParsedPayload> = HashMap::new();

        let mut ids: Vec<_> = new_schemas.iter().map(|(id, _)| *id).collect();
        ids.sort();
        for id in ids {
            let Some(parsed_schema) = new_schemas.get(&id) else {
                continue;
            };
            raw_by_id.insert(id, &parsed_schema.raw);
        }

        let index = SchemaIndex::from_name_id_pairs(
            new_schemas
                .iter()
                .map(|(id, parsed)| {
                    let name = SchemaName::try_new(parsed.raw.name())
                        .map_err(SchemaLoaderError::Resolution)?;
                    Ok((name, *id))
                })
                .collect::<Result<Vec<_>, SchemaLoaderError>>()?,
        );

        let mut builder = GraphBuilder::new();
        #[expect(
            clippy::iter_over_hash_type,
            reason = "node insertion order is irrelevant for build"
        )]
        for (id, raw) in &raw_by_id {
            let mut parents = Vec::new();
            if let Some(parent_id) =
                raw.extends().and_then(|extends| index.get_id_by_name(extends))
            {
                parents.push(parent_id);
            }
            builder.insert_node(*id, parents);
        }
        let graph = builder.build(|id, parents| {
            InheritanceNode::new_child(id, parents, NodeDepth::ROOT)
        })?;
        let (graph_nodes, graph_order, graph_roots) = graph.into_parts();

        for (id, parsed) in new_schemas {
            payloads.insert(id, NewParsedPayload {
                path: parsed.path,
                times: parsed.times,
                content_hash: parsed.content_hash,
                raw: parsed.raw,
            });
        }

        let mut nodes = HashMap::new();
        let mut graph_entries: Vec<_> = graph_nodes.into_iter().collect();
        graph_entries.sort_by_key(|entry| entry.0);
        for (id, node) in graph_entries {
            let payload = payloads.remove(&id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing parsed payload")
                            .into(),
                    },
                ))
            })?;
            nodes.insert(id, PreProcessNode {
                id: node.id(),
                parents: node.parents().to_vec(),
                children: node.children().to_vec(),
                depth: node.depth(),
                status: NodeStatus::New,
                payload,
            });
        }

        Ok(SchemaProcessor::<Construction, NewBuild>::transition(
            Construction,
            NewBuild {
                graph: InheritanceGraph::from_parts(
                    nodes,
                    graph_order,
                    graph_roots,
                ),
            },
        ))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  PROPERTYANALYSIS STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<PropertyAnalysis, Graphed> {
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
    ) -> Result<SchemaProcessor<Refresh, Analyzed>, SchemaLoaderError> {
        let Graphed {
            graph,
            deleted_ids,
        } = self.status;

        let mut merge_roots: HashSet<SchemaId> = HashSet::new();
        for id in graph.order() {
            let Some(node) = graph.nodes().get(id) else {
                continue;
            };
            if node.extends_change.requires_merge() {
                merge_roots.insert(*id);
            }
        }
        let affected: HashSet<SchemaId> = if merge_roots.is_empty() {
            HashSet::new()
        } else {
            graph.affected_subtree(&merge_roots)
        };

        let mut refresh_ids = Vec::new();
        let mut rebuild_ids = Vec::new();

        let mut analyzed_nodes = HashMap::new();

        let (nodes, order, roots) = graph.into_parts();
        let mut node_entries: Vec<_> = nodes.into_iter().collect();
        node_entries.sort_by_key(|entry| entry.0);
        for (id, node) in node_entries {
            let node_status = node.status;
            let (status, payload) = match node.payload {
                InheritanceBranch::Fresh(payload) => {
                    let times_for_raw = RawFileTimes {
                        created_at: source.created_at(&payload.path),
                        modified_at: source.modified_at(&payload.path),
                    };
                    let bank_changed =
                        Self::bank_changed(&payload.view, property_bank_delta);

                    if bank_changed {
                        let content = source
                            .read_to_string(&payload.path)
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?;
                        let content_hash =
                            *blake3::hash(content.as_bytes()).as_bytes();
                        let schema_name =
                            Self::schema_stem(source, &payload.path)?;
                        let raw = FsReader::parse_structured_from_str::<
                            RawSchema,
                        >(
                            &payload.path, &content
                        )
                        .map_err(SchemaIngestionError::from)
                        .map_err(SchemaLoaderError::Ingestion)?
                        .with_file_times(times_for_raw.clone())
                        .with_name(schema_name);

                        let mut view = payload.view;
                        let version = Self::build_version(&raw, content_hash)?;
                        view.add_version(version);

                        let rebuild = RebuildNodePayload {
                            path: payload.path,
                            times: times_for_raw,
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
                        let times = RawFileTimes {
                            created_at: source.created_at(&payload.path),
                            modified_at: source.modified_at(&payload.path),
                        };
                        refresh_ids.push(id);
                        (
                            NodeStatus::Fresh,
                            AnalysisBranch::Refresh(RefreshNodePayload {
                                path: payload.path,
                                times,
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
                        let schema_name =
                            Self::schema_stem(source, &payload.path)?;
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

            analyzed_nodes.insert(id, PostProcessNode {
                id: node.id,
                parents: node.parents,
                children: node.children,
                depth: node.depth,
                status,
                extends_change: node.extends_change,
                payload,
            });
        }

        for id in &order {
            if affected.contains(id) && !rebuild_ids.contains(id) {
                rebuild_ids.push(*id);
            }
        }

        Ok(Self::transition(Refresh, Analyzed {
            graph: InheritanceGraph::from_parts(analyzed_nodes, order, roots),
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
        source: &FsReader,
        path: &std::path::Path,
    ) -> Result<Box<str>, SchemaLoaderError> {
        source
            .basename(path)
            .map(Into::into)
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)
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

impl SchemaProcessor<Refresh, Analyzed> {
    pub(crate) fn refresh_metadata<R>(
        mut self,
        repository: &R,
    ) -> Result<SchemaProcessor<Construction, Analyzed>, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        use crate::schema::views::metadata::{FileTimesMetadata, HashMetadata};

        let (mut nodes, order, roots) = self.status.graph.into_parts();

        for id in &self.status.refresh_ids {
            let Some(node) = nodes.get_mut(id) else {
                continue;
            };

            let Some(payload) = node.payload.as_refresh_mut() else {
                continue;
            };

            let current = payload.view.current().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "missing schema metadata in cached view".into(),
                    },
                ))
            })?;

            let property_hashes = current.hashes().properties().clone();

            let file_times = FileTimesMetadata::new(
                payload.times.created_at,
                payload.times.modified_at,
            );
            let hashes =
                HashMetadata::new(payload.content_hash, property_hashes);
            let version = current.with_metadata(file_times, hashes);

            payload.view.add_version(version);

            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }

        for id in &self.status.rebuild_ids {
            let Some(node) = nodes.get_mut(id) else {
                continue;
            };

            let Some(payload) = node.payload.as_rebuild_mut() else {
                continue;
            };

            repository.save_raw_schema_view(*id, &payload.view).map_err(
                |e| {
                    let repo_err: SchemaRepositoryError = e.into();
                    SchemaLoaderError::Repository(repo_err)
                },
            )?;
        }

        self.status.graph = InheritanceGraph::from_parts(nodes, order, roots);
        Ok(Self::transition(Construction, self.status))
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
                let node = graph.nodes().get(id)?;
                match node.payload.clone() {
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
                let node = graph.nodes().get(id)?;
                match node.payload.clone() {
                    AnalysisBranch::Rebuild(payload) => {
                        Some((*id, payload.raw))
                    }
                    AnalysisBranch::Update(payload) => Some((*id, payload.raw)),
                    AnalysisBranch::Refresh(_) => None,
                }
            })
            .collect();

        let expanded_by_id: HashMap<SchemaId, PropertyMap> =
            if expand_pairs.is_empty() {
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

                        let inline_entries = collect_inline_entries(&raw);
                        if !inline_entries.is_empty() {
                            let inline_props =
                                PropertyMap::try_from(inline_entries)
                                    .map_err(SchemaLoaderError::Resolution)?;
                            expanded_props.extend(inline_props);
                        }

                        Ok((id, expanded_props))
                    })
                    .collect::<Result<_, SchemaLoaderError>>()?
            };

        let mut changed_schemas = Vec::new();
        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in graph.order() {
            let node = graph.nodes().get(id).ok_or_else(|| {
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
        node: &PostProcessNode<AnalysisBranch>,
        expanded_by_id: &HashMap<SchemaId, PropertyMap>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
        constructed_cache: &HashMap<SchemaId, Schema>,
    ) -> Result<Schema, SchemaLoaderError> {
        let (raw, property_delta) = match node.payload.clone() {
            AnalysisBranch::Rebuild(payload) => {
                (payload.raw, payload.property_delta)
            }
            AnalysisBranch::Update(payload) => {
                (payload.raw, Some(payload.property_delta))
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
                for (name, prop) in schema.properties() {
                    merged.insert(name.clone(), prop.clone());
                }
            }
        }
        merged
    }
}

impl SchemaProcessor<Construction, NewBuild> {
    pub(crate) fn construct_new_schemas(
        self,
        repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
        property_bank: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        use crate::schema::expander::RefExpander;

        let NewBuild {
            graph,
        } = self.status;

        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();
        let mut built = Vec::new();
        let expander = RefExpander::new(property_bank);
        let empty_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in graph.order() {
            let node = graph.nodes().get(id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing from graph")
                            .into(),
                    },
                ))
            })?;
            let parsed = &node.payload;

            let refs = parsed.raw.properties().ref_entries();
            let mut expanded_props = expander
                .expand_properties(&refs)
                .map_err(SchemaLoaderError::Resolution)?;
            let inline_entries = collect_inline_entries(&parsed.raw);
            if !inline_entries.is_empty() {
                let inline_props = PropertyMap::try_from(inline_entries)
                    .map_err(SchemaLoaderError::Resolution)?;
                expanded_props.extend(inline_props);
            }

            let parent_props = if node.parents.is_empty() {
                PropertyMap::new()
            } else {
                SchemaProcessor::<Construction, Analyzed>::collect_parent_properties(
                    &node.parents,
                    &constructed_cache,
                    &empty_cache,
                )
            };

            let merged = Merger::inherit_properties(
                &parent_props,
                &expanded_props,
                parsed.raw.excludes(),
            );
            let name = SchemaName::try_new(parsed.raw.name())
                .map_err(SchemaLoaderError::Resolution)?;
            let schema = Schema::new(
                *id,
                name,
                node.parents.clone(),
                node.children.clone(),
                merged,
            );

            let filename =
                parsed.path.to_string_lossy().into_owned().into_boxed_str();
            let version =
                SchemaProcessor::<PropertyAnalysis, Graphed>::build_version(
                    &parsed.raw,
                    parsed.content_hash,
                )?;
            let view = RawSchemaView::new(
                crate::schema::views::Filename::new(filename),
                version,
            );
            repository.save_raw_schema_view(*id, &view).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;

            constructed_cache.insert(*id, schema.clone());
            built.push(schema);
        }

        if !built.is_empty() {
            repository.save_schemas(&built).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;
        }

        let inheritance_graph = graph.map_payload(|node| {
            let mut new_node = InheritanceNode::new_child(
                node.id(),
                node.parents().to_vec(),
                node.depth(),
            );
            new_node
                .set_edges(node.parents().to_vec(), node.children().to_vec());
            new_node
        });
        repository.save_topological_graph(&inheritance_graph).map_err(|e| {
            let repo_err: SchemaRepositoryError = e.into();
            SchemaLoaderError::Repository(repo_err)
        })?;

        Ok(built)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMPLETION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Construction, Constructed> {
    pub(crate) fn complete(
        self,
        repository: &impl Repository<Error = impl Into<SchemaRepositoryError>>,
    ) -> Result<SchemaProcessor<Completed, Constructed>, SchemaLoaderError>
    {
        let Constructed {
            graph,
            schemas,
            deleted_ids,
        } = self.status;

        let owned_schemas: Vec<Schema> =
            schemas.iter().map(|s| (**s).clone()).collect();
        if !owned_schemas.is_empty() {
            repository.save_schemas(&owned_schemas).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;
        }

        let inheritance_graph = graph.map_payload(|node| {
            let mut new_node = InheritanceNode::new_child(
                node.id(),
                node.parents().to_vec(),
                node.depth(),
            );
            new_node
                .set_edges(node.parents().to_vec(), node.children().to_vec());
            new_node
        });

        repository.save_topological_graph(&inheritance_graph).map_err(|e| {
            let repo_err: SchemaRepositoryError = e.into();
            SchemaLoaderError::Repository(repo_err)
        })?;

        Ok(Self::transition(Completed, Constructed {
            graph,
            schemas,
            deleted_ids,
        }))
    }
}

impl SchemaProcessor<Completed, Constructed> {
    #[inline]
    pub(crate) fn into_schemas(self) -> Vec<Arc<Schema>> {
        self.status.schemas
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HELPER FUNCTIONS
// ═════════════════════════════════════════════════════════════════════════════

fn collect_inline_entries(
    raw: &RawSchema,
) -> HashMap<PropertyName, RawPropertyInline> {
    let mut inline_entries = HashMap::new();
    for (name, entry) in raw.properties() {
        match entry.clone() {
            crate::schema::raw::property::RawProperty::Inline(inline) => {
                inline_entries.insert(name.clone(), inline);
            }
            crate::schema::raw::property::RawProperty::Ref(_) => {}
        }
    }
    inline_entries
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

#[expect(
    clippy::iter_over_hash_type,
    reason = "hash diff order does not impact output"
)]
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

        match prop.clone() {
            crate::schema::raw::property::RawProperty::Inline(inline) => {
                upserts.inline.insert(name.clone(), inline);
            }
            crate::schema::raw::property::RawProperty::Ref(r#ref) => {
                upserts.refs.insert(name.clone(), r#ref);
            }
        }
    }

    for name in old_hashes.keys() {
        if !current_hashes.contains_key(name) {
            removed.push(name.clone());
        }
    }
    removed.sort();

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
