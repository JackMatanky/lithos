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

#[cfg(test)]
pub(crate) use crate::schema::delta::SchemaPropertyUpserts;
pub(crate) use crate::schema::delta::{ExcludesDelta, SchemaPropertyDelta};
use crate::{
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        bank::PropertyBank,
        builder::FilesContext,
        delta::PropertyDiffer,
        error::{
            SchemaError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError,
        },
        index::SchemaIndex,
        inheritance::{InheritanceGraph, ProcessingGraph, SchemaGraphBuilder},
        merger::Merger,
        property::{PropertyMap, PropertyName},
        raw::{RawFileTimes, RawSchema},
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
//  PROCESSOR NODES
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub(crate) struct ProcessorNode<T> {
    status: NodeStatus,
    relation: ExtendsChangeKind,
    payload: T,
}

#[expect(dead_code, reason = "API reserved for future use")]
impl<T> ProcessorNode<T> {
    #[inline]
    #[must_use]
    pub(crate) fn new(
        status: NodeStatus,
        relation: ExtendsChangeKind,
        payload: T,
    ) -> Self {
        Self {
            status,
            relation,
            payload,
        }
    }

    pub(crate) fn payload(&self) -> &T {
        &self.payload
    }

    /// Returns mutable access to the payload.
    pub(crate) fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Returns the node status.
    #[inline]
    #[must_use]
    pub(crate) fn status(&self) -> NodeStatus {
        self.status
    }

    /// Sets the node status.
    pub(crate) fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
    }

    /// Returns the extends relationship change kind.
    #[inline]
    #[must_use]
    pub(crate) fn relation(&self) -> ExtendsChangeKind {
        self.relation
    }

    /// Sets the extends relationship change kind.
    pub(crate) fn set_relation(&mut self, relation: ExtendsChangeKind) {
        self.relation = relation;
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  PAYLOAD STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PresentPayload {
    Found(FoundPayload),
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

// PipelinePayload migration rules:
// - The graph payload type stays stable across stages.
// - Stage transitions switch payload variants, not graph generic types.
// - Deleted nodes may intentionally pass through selected stages.
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "pipeline stages intentionally carry rich branch payloads"
)]
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
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "const match ergonomics keep variant mapping concise"
    )]
    pub(crate) const fn variant_name(&self) -> &'static str {
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

    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "const match ergonomics keep payload access concise"
    )]
    pub(crate) const fn as_analysis_mut(
        &mut self,
    ) -> Option<&mut AnalysisBranch> {
        match self {
            Self::Analysis(payload) => Some(payload),
            Self::Present(_)
            | Self::Compared(_)
            | Self::FileParsed(_)
            | Self::Inheritance(_)
            | Self::NewParsed(_)
            | Self::Deleted(_) => None,
        }
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "const match ergonomics keep payload access concise"
    )]
    pub(crate) const fn as_present(&self) -> Option<&PresentPayload> {
        match self {
            Self::Present(payload) => Some(payload),
            Self::Compared(_)
            | Self::FileParsed(_)
            | Self::Inheritance(_)
            | Self::Analysis(_)
            | Self::NewParsed(_)
            | Self::Deleted(_) => None,
        }
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "const match ergonomics keep payload access concise"
    )]
    pub(crate) const fn as_analysis(&self) -> Option<&AnalysisBranch> {
        match self {
            Self::Analysis(payload) => Some(payload),
            Self::Present(_)
            | Self::Compared(_)
            | Self::FileParsed(_)
            | Self::Inheritance(_)
            | Self::NewParsed(_)
            | Self::Deleted(_) => None,
        }
    }
}

impl AnalysisBranch {
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on &self keep accessors concise"
    )]
    fn as_rebuild(&self) -> Option<&RebuildNodePayload> {
        match self {
            Self::Rebuild(payload) => Some(payload),
            Self::Refresh(_) | Self::Update(_) => None,
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics on &self keep accessors concise"
    )]
    fn as_update(&self) -> Option<&UpdateNodePayload> {
        match self {
            Self::Update(payload) => Some(payload),
            Self::Refresh(_) | Self::Rebuild(_) => None,
        }
    }

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

    /// Consumes the batch and returns an iterator over entries sorted by ID.
    pub(crate) fn into_sorted_iter(
        self,
    ) -> impl Iterator<Item = (SchemaId, T)> {
        let mut entries: Vec<_> = self.0.into_iter().collect();
        entries.sort_by_key(|entry| entry.0);
        entries.into_iter()
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
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_schemas: NewBatch<InitialScan>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "ID vectors for incremental pipeline optimization"
)]
pub(crate) struct Compared {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_schemas: NewBatch<InitialRead>,
    fresh: Vec<SchemaId>,
    stale_timestamps: Vec<SchemaId>,
    stale_refs: Vec<SchemaId>,
    stale: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Parsed {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    new_schemas: NewBatch<InitialParsed>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Graphed {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Analyzed {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    refresh_ids: Vec<SchemaId>,
    stale_timestamp_ids: Vec<SchemaId>,
    rebuild_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct Constructed {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
    schemas: Vec<Arc<Schema>>,
    deleted_ids: Vec<SchemaId>,
}

#[derive(Debug)]
pub(crate) struct NewBuild {
    graph: ProcessingGraph<ProcessorNode<PipelinePayload>>,
}

#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
#[expect(
    clippy::large_enum_variant,
    reason = "branch payloads are large by design for pipeline staging"
)]
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
        graph: &InheritanceGraph<()>,
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
        graph: Option<&InheritanceGraph<()>>,
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
            for id in graph.topo_order() {
                if !file_ids.contains(id) && !missing.contains_key(id) {
                    deleted_ids.push(*id);
                }
            }
        }

        (missing, found, deleted_ids)
    }

    fn build_present_graph(
        graph: &InheritanceGraph<()>,
        found: &HashMap<SchemaId, FoundPayload>,
        deleted_ids: &[SchemaId],
    ) -> ProcessingGraph<ProcessorNode<PipelinePayload>> {
        let deleted_set: HashSet<SchemaId> =
            deleted_ids.iter().copied().collect();

        let mut builder = SchemaGraphBuilder::new();

        for (id, _node) in graph.iter() {
            let payload = if let Some(found) = found.get(&id) {
                PipelinePayload::Present(PresentPayload::Found(found.clone()))
            } else if deleted_set.contains(&id) {
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

            builder.add_node(
                id,
                ProcessorNode::new(
                    status,
                    ExtendsChangeKind::Unchanged,
                    payload,
                ),
            );
        }

        for (child_id, &()) in graph.iter() {
            for &parent_id in graph.parents_of(child_id) {
                builder.add_parent(child_id, parent_id);
            }
        }

        builder.build()
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
        let mut stale_ts_ids = Vec::new();
        let mut stale_ref_ids = Vec::new();
        let mut stale_ids = Vec::new();

        // First pass: collect bank-affected IDs.
        let mut bank_affected_ids = HashSet::new();
        if let Some(delta) = property_bank_delta {
            for (id, node) in graph.graph().iter() {
                #[expect(
                    clippy::pattern_type_mismatch,
                    reason = "matching borrowed payload keeps extraction \
                              concise"
                )]
                let Some(PresentPayload::Found(payload)) =
                    node.payload().payload.as_present()
                else {
                    continue;
                };
                if payload.view.current().is_some_and(|v| {
                    v.bank_references().values().any(|p| delta.contains(p))
                }) {
                    bank_affected_ids.insert(id);
                }
            }
        }

        let next_graph = graph.map_payload(
            |id,
             node|
             -> Result<ProcessorNode<PipelinePayload>, SchemaLoaderError> {
                let relation = node.relation();
                match node.payload {
                    PipelinePayload::Present(PresentPayload::Found(
                        found_payload,
                    )) => {
                        let is_bank_affected = bank_affected_ids.contains(&id);
                        let comparison_payload =
                            match Self::check_timestamps(found_payload, source)? {
                                TimestampBranch::Match(matched_payload) => {
                                    if is_bank_affected {
                                        let content_str = source
                                            .read_to_string(&matched_payload.path)
                                            .map_err(SchemaIngestionError::from)
                                            .map_err(SchemaLoaderError::Ingestion)?;
                                        let content_hash = *blake3::hash(
                                            content_str.as_bytes(),
                                        )
                                        .as_bytes();
                                        ComparedPayload::StaleBankReferences(
                                            StalePayload {
                                                path: matched_payload.path,
                                                times: matched_payload.times,
                                                content_str: content_str.into(),
                                                content_hash,
                                                view: matched_payload.view,
                                            },
                                        )
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
                                            ComparedPayload::StaleTimestamps(
                                                FoundPayload {
                                                    path: content_payload.path,
                                                    times: content_payload.times,
                                                    view: content_payload.view,
                                                },
                                            )
                                        }
                                        ContentBranch::Mismatch(stale_payload) => {
                                            ComparedPayload::Stale(stale_payload)
                                        }
                                    }
                                }
                            };

                        #[expect(
                            clippy::pattern_type_mismatch,
                            reason = "match on enum reference for ID tracking"
                        )]
                        match &comparison_payload {
                            ComparedPayload::Fresh(_) => fresh_ids.push(id),
                            ComparedPayload::StaleTimestamps(_) => {
                                stale_ts_ids.push(id);
                            }
                            ComparedPayload::StaleBankReferences(_) => {
                                stale_ref_ids.push(id);
                            }
                            ComparedPayload::Stale(_) => stale_ids.push(id),
                        }

                        let status =
                            Self::status_for_payload(&comparison_payload);
                        Ok(ProcessorNode::new(
                            status,
                            relation,
                            PipelinePayload::Compared(comparison_payload),
                        ))
                    }
                    PipelinePayload::Deleted(payload) => Ok(ProcessorNode::new(
                        NodeStatus::Deleted,
                        relation,
                        PipelinePayload::Deleted(payload),
                    )),
                    unexpected => Err(stage_variant_error(
                        "compare",
                        id,
                        "Present or Deleted",
                        unexpected.variant_name(),
                    )),
                }
            },
        )?;

        let mut new_reads = NewBatch::new();
        for (id, scan) in new_schemas.into_sorted_iter() {
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
            graph: next_graph,
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
        reason = "parse transition keeps branch conversion logic co-located"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
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

        let parsed_new = Self::parse_new(new_schemas, source)?;

        let next_graph =
            graph.map_payload(
                |id,
                 node|
                 -> Result<
                    ProcessorNode<PipelinePayload>,
                    SchemaLoaderError,
                > {
                    let relation = node.relation();
                    let next = match node.payload {
                        PipelinePayload::Compared(ComparedPayload::Stale(
                            payload,
                        )) => {
                            let schema_name = source
                                .basename(&payload.path)
                                .map_err(SchemaIngestionError::from)
                                .map_err(SchemaLoaderError::Ingestion)?;
                            let times_for_raw = payload.times.clone();
                            let raw = FsReader::parse_structured_from_str::<
                                RawSchema,
                            >(
                                &payload.path, &payload.content_str
                            )
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?
                            .with_file_times(times_for_raw)
                            .with_name(schema_name.into());

                            ProcessorNode::new(
                                NodeStatus::StaleParsed,
                                relation,
                                PipelinePayload::FileParsed(
                                    FileParsedBranch::StaleParsed(
                                        StaleParsedPayload {
                                            path: payload.path,
                                            times: payload.times,
                                            content_hash: payload.content_hash,
                                            raw,
                                            view: payload.view,
                                        },
                                    ),
                                ),
                            )
                        }
                        PipelinePayload::Compared(
                            ComparedPayload::StaleBankReferences(payload),
                        ) => {
                            let schema_name = source
                                .basename(&payload.path)
                                .map_err(SchemaIngestionError::from)
                                .map_err(SchemaLoaderError::Ingestion)?;
                            let times_for_raw = payload.times.clone();
                            let raw = FsReader::parse_structured_from_str::<
                                RawSchema,
                            >(
                                &payload.path, &payload.content_str
                            )
                            .map_err(SchemaIngestionError::from)
                            .map_err(SchemaLoaderError::Ingestion)?
                            .with_file_times(times_for_raw)
                            .with_name(schema_name.into());
                            let content_hash = payload.content_hash;

                            ProcessorNode::new(
                                NodeStatus::StaleBankReferences,
                                relation,
                                PipelinePayload::FileParsed(
                                    FileParsedBranch::StaleParsed(
                                        StaleParsedPayload {
                                            path: payload.path,
                                            times: payload.times,
                                            content_hash,
                                            raw,
                                            view: payload.view,
                                        },
                                    ),
                                ),
                            )
                        }
                        PipelinePayload::Compared(ComparedPayload::Fresh(
                            payload,
                        )) => ProcessorNode::new(
                            NodeStatus::Fresh,
                            relation,
                            PipelinePayload::FileParsed(
                                FileParsedBranch::Fresh(payload),
                            ),
                        ),
                        PipelinePayload::Compared(
                            ComparedPayload::StaleTimestamps(payload),
                        ) => ProcessorNode::new(
                            NodeStatus::StaleTimestamps,
                            relation,
                            PipelinePayload::FileParsed(
                                FileParsedBranch::StaleTimestamps(payload),
                            ),
                        ),
                        PipelinePayload::Deleted(payload) => {
                            ProcessorNode::new(
                                NodeStatus::Deleted,
                                relation,
                                PipelinePayload::Deleted(payload),
                            )
                        }
                        unexpected => {
                            return Err(stage_variant_error(
                                "parse",
                                id,
                                "Compared or Deleted",
                                unexpected.variant_name(),
                            ));
                        }
                    };

                    Ok(next)
                },
            )?;

        Ok(Self::transition(InheritanceGraphed, Parsed {
            graph: next_graph,
            new_schemas: parsed_new,
            deleted_ids,
        }))
    }

    fn parse_new(
        new_schemas: NewBatch<InitialRead>,
        source: &FsReader,
    ) -> Result<NewBatch<InitialParsed>, SchemaLoaderError> {
        let mut parsed_new = NewBatch::new();

        for (id, read) in new_schemas.into_sorted_iter() {
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

        Ok(parsed_new)
    }
}

impl SchemaProcessor<InheritanceGraphed, Parsed> {
    #[expect(
        clippy::too_many_lines,
        reason = "graph construction keeps related steps co-located"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "match ergonomics keep structural checks concise"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
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
        for (id, node) in graph.graph().iter() {
            status_by_id.insert(id, node.payload().status());
        }

        let old_parents = Self::collect_old_parents(&graph);

        let mut new_ids: Vec<_> =
            new_schemas.iter().map(|(id, _)| *id).collect();
        new_ids.sort();
        let index =
            Self::build_resolution_index(&graph, &new_schemas, &deleted_ids)?;

        let mut builder = SchemaGraphBuilder::new();

        for id in graph.node_ids_sorted() {
            if deleted_ids.contains(&id) {
                continue;
            }

            let node = graph.graph().get(id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing from graph")
                            .into(),
                    },
                ))
            })?;
            let payload = match node.payload().payload.clone() {
                PipelinePayload::FileParsed(payload) => payload,
                PipelinePayload::Deleted(_) => continue,
                unexpected => {
                    return Err(stage_variant_error(
                        "build_graph",
                        id,
                        "FileParsed or Deleted",
                        unexpected.variant_name(),
                    ));
                }
            };
            let status = match &payload {
                FileParsedBranch::StaleParsed(_) => status_by_id
                    .get(&id)
                    .copied()
                    .unwrap_or(NodeStatus::StaleParsed),
                FileParsedBranch::Fresh(_) => NodeStatus::Fresh,
                FileParsedBranch::StaleTimestamps(_) => {
                    NodeStatus::StaleTimestamps
                }
            };

            let new_parent = match &payload {
                FileParsedBranch::StaleParsed(stale) => stale
                    .raw
                    .extends()
                    .and_then(|name| index.get_id_by_name(name)),
                FileParsedBranch::Fresh(_)
                | FileParsedBranch::StaleTimestamps(_) => {
                    old_parents.get(&id).and_then(|p| p.first().copied())
                }
            };

            let old_parent =
                old_parents.get(&id).and_then(|p| p.first().copied());
            let change_kind = match (old_parent, new_parent) {
                (None, None) => ExtendsChangeKind::Unchanged,
                (None, Some(_)) => ExtendsChangeKind::RootToChild,
                (Some(_), None) => ExtendsChangeKind::ChildToRoot,
                (Some(old), Some(new)) if old == new => {
                    ExtendsChangeKind::Unchanged
                }
                (Some(_), Some(_)) => ExtendsChangeKind::Rewired,
            };

            let branch_payload = match payload {
                FileParsedBranch::Fresh(p) => InheritanceBranch::Fresh(p),
                FileParsedBranch::StaleTimestamps(p) => {
                    InheritanceBranch::StaleTimestamps(p)
                }
                FileParsedBranch::StaleParsed(p) => {
                    InheritanceBranch::StaleParsed(p)
                }
            };

            builder.add_node(
                id,
                ProcessorNode::new(
                    status,
                    change_kind,
                    PipelinePayload::Inheritance(branch_payload),
                ),
            );

            if let Some(parent_id) = new_parent {
                builder.add_parent(id, parent_id);
            }
        }

        for id in &new_ids {
            let Some(new) = new_schemas.get(id) else {
                continue;
            };

            let new_parent =
                new.raw.extends().and_then(|name| index.get_id_by_name(name));

            builder.add_node(
                *id,
                ProcessorNode::new(
                    NodeStatus::New,
                    ExtendsChangeKind::Unchanged,
                    PipelinePayload::Inheritance(InheritanceBranch::New(
                        NewParsedPayload {
                            path: new.path.clone(),
                            times: new.times.clone(),
                            content_hash: new.content_hash,
                            raw: new.raw.clone(),
                        },
                    )),
                ),
            );

            if let Some(parent_id) = new_parent {
                builder.add_parent(*id, parent_id);
            }
        }

        let processed_graph = builder.build();

        Ok(SchemaProcessor::<PropertyAnalysis, Graphed>::transition(
            PropertyAnalysis,
            Graphed {
                graph: processed_graph,
                deleted_ids,
            },
        ))
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
    )]
    fn build_resolution_index(
        graph: &ProcessingGraph<ProcessorNode<PipelinePayload>>,
        new_schemas: &NewBatch<InitialParsed>,
        deleted_ids: &[SchemaId],
    ) -> Result<SchemaIndex, SchemaLoaderError> {
        let mut name_index = HashMap::new();

        for (id, node) in graph.graph().iter() {
            if deleted_ids.contains(&id) {
                continue;
            }
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "matching borrowed branch keeps expression concise"
            )]
            let name = match &node.payload().payload {
                PipelinePayload::FileParsed(FileParsedBranch::Fresh(
                    payload,
                )) => payload.view.name(),
                PipelinePayload::FileParsed(
                    FileParsedBranch::StaleTimestamps(payload),
                ) => payload.view.name(),
                PipelinePayload::FileParsed(FileParsedBranch::StaleParsed(
                    payload,
                )) => payload.raw.name(),
                PipelinePayload::Deleted(_) => continue,
                unexpected => {
                    return Err(stage_variant_error(
                        "build_resolution_index",
                        id,
                        "FileParsed or Deleted",
                        unexpected.variant_name(),
                    ));
                }
            };
            let name = SchemaName::try_new(name)
                .map_err(SchemaLoaderError::Resolution)?;
            name_index.insert(name, id);
        }

        for (id, new) in new_schemas.iter() {
            let name = SchemaName::try_new(new.raw.name())
                .map_err(SchemaLoaderError::Resolution)?;
            name_index.insert(name, *id);
        }

        Ok(SchemaIndex::from_name_id_pairs(name_index))
    }

    fn collect_old_parents(
        graph: &ProcessingGraph<ProcessorNode<PipelinePayload>>,
    ) -> HashMap<SchemaId, Vec<SchemaId>> {
        let mut old_parents = HashMap::new();
        for (id, _) in graph.graph().iter() {
            let parents: Vec<SchemaId> = graph.graph().parents_of(id).to_vec();
            old_parents.insert(id, parents);
        }
        old_parents
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

        let mut builder = SchemaGraphBuilder::new();

        // First pass: add all nodes
        let mut node_data: HashMap<SchemaId, NewParsedPayload> = HashMap::new();
        for (id, parsed) in new_schemas {
            let payload = NewParsedPayload {
                path: parsed.path,
                times: parsed.times,
                content_hash: parsed.content_hash,
                raw: parsed.raw,
            };
            node_data.insert(id, payload.clone());
            builder.add_node(
                id,
                ProcessorNode::new(
                    NodeStatus::New,
                    ExtendsChangeKind::Unchanged,
                    PipelinePayload::NewParsed(payload),
                ),
            );
        }

        // Second pass: add edges based on extends relationships
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Order not relevant for edge construction"
        )]
        for (id, payload) in &node_data {
            if let Some(parent_id) = payload
                .raw
                .extends()
                .and_then(|extends| index.get_id_by_name(extends))
            {
                builder.add_parent(*id, parent_id);
            }
        }

        let processing_graph = builder.build();

        Ok(SchemaProcessor::<Construction, NewBuild>::transition(
            Construction,
            NewBuild {
                graph: processing_graph,
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
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
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
        for (id, node) in graph.graph().iter() {
            if node.payload().relation().requires_merge() {
                merge_roots.insert(id);
            }
        }
        let affected: HashSet<SchemaId> = if merge_roots.is_empty() {
            HashSet::new()
        } else {
            crate::schema::inheritance::affected_subtree(
                graph.graph(),
                &merge_roots,
            )
        };

        let mut refresh_ids = Vec::new();
        let mut stale_timestamp_ids = Vec::new();
        let mut rebuild_ids = Vec::new();

        let next_graph = graph.map_payload(
            |id, node| -> Result<ProcessorNode<PipelinePayload>, SchemaLoaderError> {
                let relation = node.relation();
                let node_status = node.status();
                let (status, payload) = match node.payload {
                    PipelinePayload::Inheritance(InheritanceBranch::Fresh(
                        payload,
                    )) => {
                        let bank_changed =
                            Self::bank_changed(&payload.view, property_bank_delta);

                        if bank_changed {
                            let times_for_raw = RawFileTimes {
                                created_at: source.created_at(&payload.path),
                                modified_at: source.modified_at(&payload.path),
                            };
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
                            return Ok(ProcessorNode::new(
                                NodeStatus::Fresh,
                                relation,
                                PipelinePayload::Inheritance(
                                    InheritanceBranch::Fresh(payload),
                                ),
                            ));
                        }
                    }
                    PipelinePayload::Inheritance(
                        InheritanceBranch::StaleTimestamps(payload),
                    ) => {
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
                            stale_timestamp_ids.push(id);
                            return Ok(ProcessorNode::new(
                                NodeStatus::StaleTimestamps,
                                relation,
                                PipelinePayload::Inheritance(
                                    InheritanceBranch::StaleTimestamps(payload),
                                ),
                            ));
                        }
                    }
                    PipelinePayload::Inheritance(InheritanceBranch::New(
                        payload,
                    )) => {
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
                    PipelinePayload::Inheritance(
                        InheritanceBranch::StaleParsed(payload),
                    ) => {
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
                            let excludes_delta = ExcludesDelta::from_slices(
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
                                PropertyDiffer::for_schema(&payload.raw, old_property_hashes)
                                    .diff_schema();

                            let needs_rebuild =
                                !excludes_delta.is_empty()
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
                    PipelinePayload::Deleted(payload) => {
                        return Ok(ProcessorNode::new(
                            NodeStatus::Deleted,
                            relation,
                            PipelinePayload::Deleted(payload),
                        ));
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
                    status,
                    relation,
                    PipelinePayload::Analysis(payload),
                ))
            },
        )?;

        let topo_order = next_graph.topo_order().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Inheritance(e))
        })?;

        for id in topo_order {
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
        use crate::schema::views::metadata::{FileTimesMetadata, HashMetadata};

        for id in &status.refresh_ids {
            let Some(node) = status.graph.graph_mut().get_mut(*id) else {
                continue;
            };
            let Some(payload) = node
                .payload_mut()
                .payload
                .as_analysis_mut()
                .and_then(AnalysisBranch::as_refresh_mut)
            else {
                continue;
            };
            let current = payload.view.current().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "missing schema metadata in cached view".into(),
                    },
                ))
            })?;
            let file_times = FileTimesMetadata::new(
                payload.times.created_at,
                payload.times.modified_at,
            );
            let hashes = HashMetadata::new(
                payload.content_hash,
                current.hashes().properties().clone(),
            );
            payload.view.add_version(current.with_metadata(file_times, hashes));
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
        use crate::schema::views::metadata::{FileTimesMetadata, HashMetadata};

        for id in &status.stale_timestamp_ids {
            let Some(node) = status.graph.graph_mut().get_mut(*id) else {
                continue;
            };

            match node.payload_mut().payload {
                PipelinePayload::Inheritance(
                    InheritanceBranch::StaleTimestamps(ref mut payload),
                ) => {
                    let current = payload.view.current().ok_or_else(|| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: "missing schema metadata in cached view".into(),
                            },
                        ))
                    })?;
                    let file_times = FileTimesMetadata::new(
                        payload.times.created_at,
                        payload.times.modified_at,
                    );
                    let hashes = HashMetadata::new(
                        *current.hashes().content(),
                        current.hashes().properties().clone(),
                    );
                    payload
                        .view
                        .add_version(current.with_metadata(file_times, hashes));
                    repository
                        .save_raw_schema_view(*id, &payload.view)
                        .map_err(|e| {
                            let repo_err: SchemaRepositoryError = e.into();
                            SchemaLoaderError::Repository(repo_err)
                        })?;
                }
                PipelinePayload::Analysis(AnalysisBranch::Refresh(
                    ref mut payload,
                )) => {
                    let current = payload.view.current().ok_or_else(|| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: "missing schema metadata in cached view".into(),
                            },
                        ))
                    })?;
                    let file_times = FileTimesMetadata::new(
                        payload.times.created_at,
                        payload.times.modified_at,
                    );
                    let hashes = HashMetadata::new(
                        payload.content_hash,
                        current.hashes().properties().clone(),
                    );
                    payload
                        .view
                        .add_version(current.with_metadata(file_times, hashes));
                    repository
                        .save_raw_schema_view(*id, &payload.view)
                        .map_err(|e| {
                            let repo_err: SchemaRepositoryError = e.into();
                            SchemaLoaderError::Repository(repo_err)
                        })?;
                }
                PipelinePayload::Present(_)
                | PipelinePayload::Compared(_)
                | PipelinePayload::FileParsed(_)
                | PipelinePayload::Inheritance(_)
                | PipelinePayload::Analysis(_)
                | PipelinePayload::NewParsed(_)
                | PipelinePayload::Deleted(_) => {}
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
                .payload
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
        property_bank: &PropertyBank,
    ) -> Result<SchemaProcessor<Construction, Constructed>, SchemaLoaderError>
    {
        use crate::schema::expander::RefExpander;

        let Analyzed {
            graph,
            refresh_ids,
            stale_timestamp_ids,
            rebuild_ids,
            deleted_ids,
        } = self.status;

        let mut fetch_ids = refresh_ids.clone();
        for id in &stale_timestamp_ids {
            if !fetch_ids.contains(id) {
                fetch_ids.push(*id);
            }
        }
        let update_ids: Vec<SchemaId> = rebuild_ids
            .iter()
            .filter_map(|id| {
                let node = graph.graph().get(*id)?;
                let extends_change = node.payload().relation();
                let payload = node.payload().payload();
                if let Some(payload) =
                    payload.as_analysis().and_then(AnalysisBranch::as_rebuild)
                    && payload.property_delta.is_some()
                    && extends_change == ExtendsChangeKind::Unchanged
                {
                    Some(*id)
                } else if payload
                    .as_analysis()
                    .and_then(AnalysisBranch::as_update)
                    .is_some()
                {
                    Some(*id)
                } else {
                    None
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

        let expanded_by_id: HashMap<SchemaId, PropertyMap> = if rebuild_ids
            .is_empty()
        {
            HashMap::new()
        } else {
            let expander = RefExpander::new(property_bank);
            let mut expanded_by_id = HashMap::new();
            for id in &rebuild_ids {
                let Some(node) = graph.graph().get(*id) else {
                    continue;
                };
                let payload = node.payload().payload();
                let raw = if let Some(payload) =
                    payload.as_analysis().and_then(AnalysisBranch::as_rebuild)
                {
                    &payload.raw
                } else if let Some(payload) =
                    payload.as_analysis().and_then(AnalysisBranch::as_update)
                {
                    &payload.raw
                } else {
                    continue;
                };

                let refs = raw.properties().ref_entries();
                let mut expanded_props = expander
                    .expand_properties(&refs)
                    .map_err(SchemaLoaderError::Resolution)?;

                let inline_entries = raw.properties().inline_entries();
                if !inline_entries.is_empty() {
                    let inline_props = PropertyMap::try_from(inline_entries)
                        .map_err(SchemaLoaderError::Resolution)?;
                    expanded_props.extend(inline_props);
                }

                expanded_by_id.insert(*id, expanded_props);
            }
            expanded_by_id
        };

        let mut changed_schemas = Vec::new();
        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();

        let topo_order = graph.topo_order().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Inheritance(e))
        })?;

        for id in &topo_order {
            let node = graph.graph().get(*id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing from graph")
                            .into(),
                    },
                ))
            })?;

            let schema_id = *id;
            let schema = if refresh_ids.contains(&schema_id)
                || stale_timestamp_ids.contains(&schema_id)
            {
                fetched_by_id.remove(&schema_id).ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "schema {id} not found in refresh cache"
                            )
                            .into(),
                        },
                    ))
                })?
            } else if rebuild_ids.contains(&schema_id) {
                let parents = graph.graph().parents_of(schema_id);
                let children = graph.graph().children_of(schema_id);
                Self::construct_schema_incremental(
                    schema_id,
                    node.payload(),
                    node.payload().relation(),
                    parents,
                    children,
                    &expanded_by_id,
                    &fetched_by_id,
                    &constructed_cache,
                )?
            } else {
                repository
                    .find_schema_by_id(schema_id)
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

            let is_changed = rebuild_ids.contains(&schema_id);
            let schema = Arc::new(schema);

            constructed_cache.insert(schema_id, (*schema).clone());
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
    #[expect(
        clippy::too_many_arguments,
        reason = "incremental construction keeps inputs explicit"
    )]
    fn construct_schema_incremental(
        id: SchemaId,
        node: &ProcessorNode<PipelinePayload>,
        extends_change: ExtendsChangeKind,
        parents: &[SchemaId],
        children: &[SchemaId],
        expanded_by_id: &HashMap<SchemaId, PropertyMap>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
        constructed_cache: &HashMap<SchemaId, Schema>,
    ) -> Result<Schema, SchemaLoaderError> {
        let payload = &node.payload;
        let (raw, property_delta) = if let Some(payload) =
            payload.as_analysis().and_then(AnalysisBranch::as_rebuild)
        {
            (payload.raw.clone(), payload.property_delta.clone())
        } else if let Some(payload) =
            payload.as_analysis().and_then(AnalysisBranch::as_update)
        {
            (payload.raw.clone(), Some(payload.property_delta.clone()))
        } else if matches!(
            payload,
            PipelinePayload::Analysis(AnalysisBranch::Refresh(_))
        ) {
            return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "unexpected refresh node in rebuild path"
                            .into(),
                    },
                ),
            ));
        } else if matches!(payload, PipelinePayload::Deleted(_)) {
            return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "unexpected deleted node in rebuild path"
                            .into(),
                    },
                ),
            ));
        } else {
            return Err(stage_variant_error(
                "construct_schema_incremental",
                id,
                "Analysis",
                node.payload.variant_name(),
            ));
        };

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
                    if delta.contains_upsert(name) {
                        properties.insert(name.clone(), prop.clone());
                    }
                }
                for name in delta.removals() {
                    properties.remove(name);
                }

                let name = SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?;

                Ok(Schema::new(
                    id,
                    name,
                    parents.to_vec(),
                    children.to_vec(),
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

                let parent_props = if parents.is_empty() {
                    PropertyMap::new()
                } else {
                    Self::collect_parent_properties(
                        parents,
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
                    parents.to_vec(),
                    children.to_vec(),
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
                    children.to_vec(),
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

                let parent_props = if parents.is_empty() {
                    PropertyMap::new()
                } else {
                    Self::collect_parent_properties(
                        parents,
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
                    parents.to_vec(),
                    children.to_vec(),
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
    #[expect(
        clippy::too_many_lines,
        reason = "new-schema construction keeps fetch/build flow in one place"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "stage invariant failures intentionally collapse to one error"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "matching borrowed payload keeps parse extraction concise"
    )]
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

        let topo_order = graph.topo_order().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Inheritance(e))
        })?;

        for id in &topo_order {
            let node = graph.graph().get(*id).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("schema {id} missing from graph")
                            .into(),
                    },
                ))
            })?;
            let parsed = match &node.payload().payload {
                PipelinePayload::NewParsed(parsed) => parsed,
                PipelinePayload::Deleted(_) => continue,
                unexpected => {
                    return Err(stage_variant_error(
                        "construct_new_schemas",
                        *id,
                        "NewParsed or Deleted",
                        unexpected.variant_name(),
                    ));
                }
            };

            let refs = parsed.raw.properties().ref_entries();
            let mut expanded_props = expander
                .expand_properties(&refs)
                .map_err(SchemaLoaderError::Resolution)?;
            let inline_entries = parsed.raw.properties().inline_entries();
            if !inline_entries.is_empty() {
                let inline_props = PropertyMap::try_from(inline_entries)
                    .map_err(SchemaLoaderError::Resolution)?;
                expanded_props.extend(inline_props);
            }

            let schema_id = *id;
            let parents = graph.graph().parents_of(schema_id);
            let children = graph.graph().children_of(schema_id);
            let parent_props = if parents.is_empty() {
                PropertyMap::new()
            } else {
                SchemaProcessor::<Construction, Analyzed>::collect_parent_properties(
                    parents,
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
                schema_id,
                name,
                parents.to_vec(),
                children.to_vec(),
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
            repository.save_raw_schema_view(schema_id, &view).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;

            constructed_cache.insert(schema_id, schema.clone());
            built.push(schema);
        }

        if !built.is_empty() {
            repository.save_schemas(&built).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;
        }

        // Build unit-payload graph for persistence (structure only)
        let mut persist_builder = SchemaGraphBuilder::<()>::new();
        for (id, _node) in graph.graph().iter() {
            persist_builder.add_node(id, ()); // Unit payload
        }
        for (child_id, _) in graph.graph().iter() {
            for &parent_id in graph.graph().parents_of(child_id) {
                persist_builder.add_parent(child_id, parent_id);
            }
        }

        let inheritance_graph =
            InheritanceGraph::try_from(persist_builder.build()).map_err(
                |e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)),
            )?;
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

        for id in &deleted_ids {
            repository.delete_schema(*id).map_err(|e| {
                let repo_err: SchemaRepositoryError = e.into();
                SchemaLoaderError::Repository(repo_err)
            })?;
        }

        // Build unit-payload graph for persistence (structure only)
        let mut persist_builder = SchemaGraphBuilder::<()>::new();
        for (id, _node) in graph.graph().iter() {
            persist_builder.add_node(id, ());
        }
        for (child_id, _) in graph.graph().iter() {
            for &parent_id in graph.graph().parents_of(child_id) {
                persist_builder.add_parent(child_id, parent_id);
            }
        }

        let inheritance_graph =
            InheritanceGraph::try_from(persist_builder.build()).map_err(
                |e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)),
            )?;

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

fn stage_variant_error(
    stage: &'static str,
    id: SchemaId,
    expected: &'static str,
    actual: &'static str,
) -> SchemaLoaderError {
    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
        crate::schema::error::SchemaFileError::FileSystem {
            reason: format!(
                "stage {stage}: schema {id} expected {expected} payload, got \
                 {actual}"
            )
            .into(),
        },
    ))
}

// ═════════════════════════════════════════════════════════════════════════════
//  TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
        let old = Vec::<PropertyName>::new();
        let new = vec![PropertyName::try_new("test").unwrap()];
        let delta = ExcludesDelta::from_slices(&old, &new);
        assert!(!delta.is_empty());
    }

    #[test]
    fn property_delta_empty_when_no_changes() {
        let delta = SchemaPropertyDelta::default();
        assert!(delta.is_empty());
    }

    #[test]
    fn property_delta_not_empty_when_removed() {
        let delta =
            SchemaPropertyDelta::new(SchemaPropertyUpserts::default(), vec![
                PropertyName::try_new("test").unwrap(),
            ]);
        assert!(!delta.is_empty());
    }

    #[test]
    fn pipeline_payload_variant_name_reports_expected() {
        let payload = PipelinePayload::Deleted(DeletedPayload);
        assert_eq!(payload.variant_name(), "Deleted");
    }

    #[test]
    fn pipeline_payload_analysis_accessor_none_for_non_analysis() {
        let mut payload = PipelinePayload::Deleted(DeletedPayload);
        assert!(payload.as_analysis_mut().is_none());
    }

    #[test]
    fn stage_variant_error_contains_stage_and_variant() {
        let id = SchemaId::new();
        let error = stage_variant_error("parse", id, "Compared", "Present");
        let message = error.to_string();
        assert!(message.contains("stage parse"));
        assert!(message.contains("expected Compared payload"));
        assert!(message.contains("got Present"));
    }

    fn make_raw_schema(name: &str) -> RawSchema {
        serde_json::from_value::<RawSchema>(serde_json::json!({
            "$version": "1.0",
            "properties": {}
        }))
        .expect("valid raw schema fixture")
        .with_name(name.into())
        .with_file_times(RawFileTimes {
            created_at: None,
            modified_at: None,
        })
    }

    fn make_view(name: &str, content_hash: [u8; 32]) -> RawSchemaView {
        let raw = make_raw_schema(name);
        let file_times =
            crate::schema::views::FileTimesMetadata::new(None, None);
        let hashes = crate::schema::views::HashMetadata::new(
            content_hash,
            HashMap::new(),
        );
        let version =
            crate::schema::views::SchemaVersion::new(file_times, hashes, &raw)
                .expect("valid schema view fixture");
        RawSchemaView::new(
            crate::schema::views::Filename::new(format!("{name}.toml").into()),
            version,
        )
    }

    #[test]
    fn compare_transitions_present_to_compared_fresh_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = FsReader::new(temp.path());
        let id = SchemaId::new();
        let path = PathBuf::from("schema.toml");

        let view = make_view("schema", [7u8; 32]);

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(
            id,
            ProcessorNode::new(
                NodeStatus::Fresh,
                ExtendsChangeKind::Unchanged,
                PipelinePayload::Present(PresentPayload::Found(FoundPayload {
                    path,
                    times: RawFileTimes {
                        created_at: None,
                        modified_at: None,
                    },
                    view,
                })),
            ),
        );

        let processor = SchemaProcessor::<Comparison, Present> {
            status: Present {
                graph: builder.build(),
                new_schemas: NewBatch::new(),
                deleted_ids: Vec::new(),
            },
            _stage: PhantomData,
        };

        let compared =
            processor.compare(&source, None).expect("compare succeeds");
        let compared_node = compared
            .status
            .graph
            .graph()
            .get(id)
            .expect("node present after compare");

        assert!(matches!(
            &compared_node.payload().payload,
            PipelinePayload::Compared(ComparedPayload::Fresh(_))
        ));
    }

    #[test]
    fn deleted_nodes_pass_through_compare_and_parse() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = FsReader::new(temp.path());
        let deleted_id = SchemaId::new();

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(
            deleted_id,
            ProcessorNode::new(
                NodeStatus::Deleted,
                ExtendsChangeKind::Unchanged,
                PipelinePayload::Deleted(DeletedPayload),
            ),
        );

        let processor = SchemaProcessor::<Comparison, Present> {
            status: Present {
                graph: builder.build(),
                new_schemas: NewBatch::new(),
                deleted_ids: vec![deleted_id],
            },
            _stage: PhantomData,
        };

        let compared =
            processor.compare(&source, None).expect("compare succeeds");
        let compared_deleted = compared
            .status
            .graph
            .graph()
            .get(deleted_id)
            .expect("deleted node present after compare");
        assert!(matches!(
            compared_deleted.payload().payload,
            PipelinePayload::Deleted(_)
        ));

        let parsed = compared.parse(&source).expect("parse succeeds");
        let parsed_deleted = parsed
            .status
            .graph
            .graph()
            .get(deleted_id)
            .expect("deleted node present after parse");
        assert!(matches!(
            parsed_deleted.payload().payload,
            PipelinePayload::Deleted(_)
        ));
    }
}
