//! Schema typestate pipeline scaffold.
//!
//! This module defines the schema pipeline stages and statuses without
//! implementing full logic yet. It mirrors the `PropertyBank` typestate pattern
//! and is orchestrated by `builder.rs`.

#![expect(dead_code, reason = "typestate scaffold for schema pipeline")]
#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "typestate pattern requires pub(crate) fields for pipeline \
              transitions"
)]
#![expect(
    clippy::todo,
    reason = "scaffold code with incomplete implementations"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "len() addition is safe for HashMap sizes"
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "reference patterns are intentional for Option matching"
)]
#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "grouping related status types together for clarity"
)]

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
            SchemaStorageError,
        },
        property::PropertyName,
        raw::{
            RawFileTimes, RawSchema,
            property::{RawProperty, RawPropertyMap},
        },
        storage::Repository,
        views::{
            FileTimesMetadata, HashMetadata, SchemaVersion, raw::RawSchemaView,
        },
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Pipeline Core
// ─────────────────────────────────────────────────────────────────────────────

/// Core typestate pipeline for schema processing.
#[derive(Debug)]
#[must_use]
pub(crate) struct SchemaProcessor<P, S> {
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> SchemaProcessor<P, S> {
    #[inline]
    fn transition<NP, NS>(_stage: NP, status: NS) -> SchemaProcessor<NP, NS> {
        SchemaProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage Markers
// ─────────────────────────────────────────────────────────────────────────────
//
//  7-Stage Pipeline:
//  1. Discovery        - Batch start, per-schema branch (Missing | Present)
//  2. Comparison       - Per-schema timestamp/hash checks (Fresh | Suspect)
//  3. TreeGraphed      - Batch graph building + cycle detection (GraphFresh |
//     GraphPatched)
//  4. PropertyAnalysis - Batch property/excludes delta (AllUnchanged |
//     HasChanges)
//  5. Construction     - Batch level-by-level expand + merge (Fresh | Changed |
//     New)
//  6. Completed        - Batch persistence (Ready)
//  7. Refresh          - Early exit for metadata-only changes (StaleTimestamps
//     | StaleContent)

/// Stage 1: Discovery - Batch start, per-schema branch.
#[derive(Debug)]
pub(crate) struct Discovery;

/// Stage 2: Comparison - Per-schema timestamp/hash checks.
#[derive(Debug)]
pub(crate) struct Comparison;

/// Stage 3: `TreeGraphed` - Batch graph building with fail-fast validation.
#[derive(Debug)]
pub(crate) struct TreeGraphed;

/// Stage 4: `PropertyAnalysis` - Batch property/excludes delta computation.
#[derive(Debug)]
pub(crate) struct PropertyAnalysis;

/// Stage 5: Construction - Batch level-by-level expand + merge.
#[derive(Debug)]
pub(crate) struct Construction;

/// Stage 6: Completed - Batch persistence.
#[derive(Debug)]
pub(crate) struct Completed;

/// Stage 7: Refresh - Early exit for metadata-only changes.
#[derive(Debug)]
pub(crate) struct Refresh;

// ─────────────────────────────────────────────────────────────────────────────
//  Status Types
// ─────────────────────────────────────────────────────────────────────────────

/// Status types for `PropertyAnalysis` stage (batch operation).
pub(crate) mod property_analysis_status {
    use super::{ExcludesDelta, SchemaId, SchemaPropertyDelta};

    /// All schemas have unchanged properties.
    #[derive(Debug)]
    pub(crate) struct AllUnchanged;

    /// Some schemas have property or excludes changes.
    #[derive(Debug)]
    pub(crate) struct HasChanges {
        /// Per-schema property deltas.
        pub(crate) property_deltas:
            std::collections::HashMap<SchemaId, SchemaPropertyDelta>,

        /// Per-schema excludes deltas.
        pub(crate) excludes_deltas:
            std::collections::HashMap<SchemaId, ExcludesDelta>,
    }
}

#[derive(Debug)]
pub(crate) struct Unknown;

#[derive(Debug)]
pub(crate) struct Missing;

#[derive(Debug)]
pub(crate) struct Present {
    pub(crate) id: SchemaId,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

#[derive(Debug)]
pub(crate) struct Suspect {
    pub(crate) id: SchemaId,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
    pub(crate) content: String,
}

#[derive(Debug)]
pub(crate) struct StaleTimestamps {
    pub(crate) id: SchemaId,
    pub(crate) view: RawSchemaView,
    pub(crate) times: RawFileTimes,
}

#[derive(Debug)]
pub(crate) struct StaleContent {
    pub(crate) id: SchemaId,
    pub(crate) view: RawSchemaView,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
}

/// Construction stage status: Schema retrieved from DB (no processing needed).
#[derive(Debug)]
pub(crate) struct Fresh {
    pub(crate) id: SchemaId,
}

/// Construction stage status: Schema re-expanded and merged (properties or
/// parent changed).
#[derive(Debug)]
pub(crate) struct Changed {
    pub(crate) schema: Schema,
}

/// Construction stage status: Schema built from scratch (first time seen).
#[derive(Debug)]
pub(crate) struct New {
    pub(crate) schema: Schema,
}

/// Completed stage status: All schemas ready for delivery.
#[derive(Debug)]
pub(crate) struct Ready {
    pub(crate) schemas: Vec<Schema>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Branching Enums
// ─────────────────────────────────────────────────────────────────────────────

/// Returned from Discovery stage: Schema exists in DB or not.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum DiscoveryBranch {
    Missing(SchemaProcessor<Comparison, Missing>),
    Present(SchemaProcessor<Comparison, Present>),
}

/// Returned from Comparison stage: Timestamps match or need content check.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ComparisonBranch {
    Fresh(SchemaProcessor<Construction, Fresh>),
    Suspect(SchemaProcessor<Comparison, Suspect>),
}

/// Returned from content hash check: Only timestamps stale or content changed.
///
/// Note: `StaleContent` schemas are collected for batch `TreeGraphed` stage.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ContentBranch {
    StaleTimestamps(SchemaProcessor<Refresh, StaleTimestamps>),
    StaleContent(SchemaProcessor<Refresh, StaleContent>),
}

/// Returned from `TreeGraphed` stage (batch): Graph reused or patched.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum TreeGraphedBranch {
    /// All schemas unchanged, graph reused from cache.
    GraphFresh {
        graph: TopologicalGraph,
        fresh_schema_ids: Vec<SchemaId>,
    },

    /// Graph patched with new/changed schemas.
    GraphPatched {
        graph: TopologicalGraph,
        raw_schemas: HashMap<SchemaId, RawSchema>,
        extends_deltas: HashMap<SchemaId, ExtendsDelta>,
        affected_subtrees: HashSet<SchemaId>,
    },
}

/// Returned from `PropertyAnalysis` stage (batch): All unchanged or has
/// changes.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum PropertyAnalysisBranch {
    AllUnchanged(
        SchemaProcessor<
            PropertyAnalysis,
            property_analysis_status::AllUnchanged,
        >,
    ),
    HasChanges(
        SchemaProcessor<PropertyAnalysis, property_analysis_status::HasChanges>,
    ),
}

/// Returned from Construction stage: Schema processing outcome.
#[derive(Debug)]
#[must_use = "branch outcomes must be handled"]
pub(crate) enum ConstructionBranch {
    Fresh(SchemaProcessor<Completed, Ready>),
    Changed(SchemaProcessor<Completed, Ready>),
    New(SchemaProcessor<Completed, Ready>),
}

// ─────────────────────────────────────────────────────────────────────────────
//  Delta Structures (Outcomes)
// ─────────────────────────────────────────────────────────────────────────────

/// Delta for extends (parent) relationship changes.
///
/// Tracks transition from old parent to new parent for a single schema.
/// Used in `TreeGraphed` stage to determine graph rebuild strategy.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExtendsDelta {
    /// Previous parent schema name (None if was root).
    pub(crate) old_parent: Option<SchemaName>,

    /// New parent schema name (None if now root).
    pub(crate) new_parent: Option<SchemaName>,
}

impl ExtendsDelta {
    /// Returns `true` if the parent relationship changed.
    #[inline]
    pub(crate) fn changed(&self) -> bool {
        self.old_parent != self.new_parent
    }

    /// Returns the kind of extends change.
    #[inline]
    pub(crate) fn kind(&self) -> ExtendsChangeKind {
        match (&self.old_parent, &self.new_parent) {
            (None, None) => ExtendsChangeKind::Unchanged,
            (None, Some(_)) => ExtendsChangeKind::RootToChild,
            (Some(_), None) => ExtendsChangeKind::ChildToRoot,
            (Some(old), Some(new)) if old == new => {
                ExtendsChangeKind::Unchanged
            }
            (Some(_), Some(_)) => ExtendsChangeKind::Rewired,
        }
    }
}

/// Classification of extends (parent) relationship changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtendsChangeKind {
    /// Parent unchanged (including both None).
    Unchanged,

    /// Schema gained a parent (was root, now child).
    RootToChild,

    /// Schema became root (was child, now root).
    ChildToRoot,

    /// Schema changed parents (both Some, different values).
    Rewired,
}

/// Delta for excludes list changes.
///
/// Tracks properties added to or removed from the excludes list.
/// Computed in `PropertyAnalysis` stage before property checks.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ExcludesDelta {
    /// Properties added to excludes list.
    pub(crate) added: Vec<PropertyName>,

    /// Properties removed from excludes list.
    pub(crate) removed: Vec<PropertyName>,
}

impl ExcludesDelta {
    /// Returns `true` if there are no changes to the excludes list.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    /// Returns `true` if the excludes list changed.
    #[inline]
    pub(crate) fn changed(&self) -> bool {
        !self.is_empty()
    }
}

/// Unified property delta combining schema-local changes and bank reference
/// changes.
///
/// Follows the `PropertyBank` pattern: upserts contain new, modified, and
/// bank-affected properties (no distinction needed - all get re-expanded).
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyDelta {
    /// Properties that need upsert (new, modified, or affected by bank
    /// changes).
    pub(crate) upserts: SchemaPropertyUpserts,

    /// Properties that were removed.
    pub(crate) removed: Vec<PropertyName>,
}

impl SchemaPropertyDelta {
    /// Returns `true` if there are no property changes.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.removed.is_empty()
    }

    /// Returns all affected property names (upserts + removals).
    #[inline]
    pub(crate) fn affected_properties(&self) -> HashSet<PropertyName> {
        let mut affected = HashSet::new();
        affected.extend(self.upserts.inline.keys().cloned());
        affected.extend(self.upserts.refs.keys().cloned());
        affected.extend(self.removed.iter().cloned());
        affected
    }
}

/// Categorized upserts for different processing needs.
///
/// Separates inline property definitions from `PropertyBank` references
/// because they require different expansion logic.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyUpserts {
    /// Inline property definitions that changed or are new.
    /// Value: `RawPropertyMap` for re-expansion.
    pub(crate) inline: HashMap<PropertyName, RawPropertyMap<RawProperty>>,

    /// Properties referencing `PropertyBank` that need re-expansion.
    /// Key: Schema property name, Value: Bank property name.
    pub(crate) refs: HashMap<PropertyName, PropertyName>,
}

impl SchemaPropertyUpserts {
    /// Returns `true` if there are no upserts.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.inline.is_empty() && self.refs.is_empty()
    }

    /// Returns the total number of properties to upsert.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.inline.len() + self.refs.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Graph Structures (New for TreeGraphed stage)
// ─────────────────────────────────────────────────────────────────────────────

/// Lightweight topologically sorted inheritance graph.
///
/// Contains ONLY `SchemaId` and `SchemaName` for each node, with precomputed
/// topological order and depth information. No properties or excludes are
/// stored here - those belong in Schema aggregate and edge metadata.
///
/// This structure is persisted to DB as a singleton and rebuilt/patched when
/// inheritance relationships change (extends delta).
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub(crate) struct TopologicalGraph {
    /// Schemas in topological order (parents before children).
    /// Used for level-by-level processing in Construction stage.
    pub(crate) order: Vec<SchemaId>,

    /// Per-node metadata indexed by `SchemaId`.
    pub(crate) nodes: HashMap<SchemaId, GraphNode>,

    /// Root schemas (no parent in batch).
    pub(crate) roots: Vec<SchemaId>,

    /// Version hash for staleness detection.
    /// Computed from extends relationships: `hash(parent_id` ||
    /// `parent.structure_hash`).
    pub(crate) structure_hash: u64,
}

impl TopologicalGraph {
    /// Returns the topological order (parents before children).
    #[inline]
    pub(crate) fn order(&self) -> &[SchemaId] {
        &self.order
    }

    /// Returns a graph node by ID.
    #[inline]
    pub(crate) fn get(&self, id: SchemaId) -> Option<&GraphNode> {
        self.nodes.get(&id)
    }

    /// Returns all root schemas (no parent).
    #[inline]
    pub(crate) fn roots(&self) -> &[SchemaId] {
        &self.roots
    }

    /// Returns the structure hash for staleness detection.
    #[inline]
    pub(crate) fn structure_hash(&self) -> u64 {
        self.structure_hash
    }
}

/// Minimal node in the topological graph.
///
/// Contains only structural information needed for graph validation and
/// topological ordering. Properties and excludes are stored elsewhere.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub(crate) struct GraphNode {
    /// Schema identifier.
    pub(crate) id: SchemaId,

    /// Schema name.
    pub(crate) name: SchemaName,

    /// Parent schema ID (if not a root).
    pub(crate) parent_id: Option<SchemaId>,

    /// Inheritance depth (1 for roots, `parent_depth` + 1 for children).
    /// Used to enforce `MAX_DEPTH` limit (10 levels).
    pub(crate) depth: usize,
}

impl GraphNode {
    /// Returns the schema ID.
    #[inline]
    pub(crate) fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the schema name.
    #[inline]
    pub(crate) fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the parent ID if this is not a root.
    #[inline]
    pub(crate) fn parent_id(&self) -> Option<SchemaId> {
        self.parent_id
    }

    /// Returns the inheritance depth.
    #[inline]
    pub(crate) fn depth(&self) -> usize {
        self.depth
    }
}

/// Inheritance edge metadata stored per parent-child pair.
///
/// Key format: "{`parent_uuid}:{child_uuid`}".
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub(crate) struct InheritanceEdgeMetadata {
    /// Parent schema ID.
    pub(crate) parent_id: SchemaId,

    /// Child schema ID.
    pub(crate) child_id: SchemaId,

    /// Properties excluded from parent inheritance.
    /// Applied during Construction stage merging.
    pub(crate) excludes: Vec<PropertyName>,
}

impl InheritanceEdgeMetadata {
    /// Creates a composite key for database storage.
    pub(crate) fn key(parent_id: SchemaId, child_id: SchemaId) -> String {
        format!("{parent_id}:{child_id}")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage Implementations (Partial)
// ─────────────────────────────────────────────────────────────────────────────

/// Entry-state operations that identify missing/present views.
impl SchemaProcessor<Discovery, Unknown> {
    #[inline]
    pub(crate) fn new() -> Self {
        SchemaProcessor {
            status: Unknown,
            _stage: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn discover<R: Repository>(
        self,
        _source: &FsReader,
        _repository: &R,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        todo!("schema discovery pipeline")
    }
}

impl Default for SchemaProcessor<Discovery, Unknown> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Operations valid when a cached view is present.
impl SchemaProcessor<Comparison, Present> {
    #[inline]
    pub(crate) fn check_timestamps(self, content: &str) -> ComparisonBranch {
        let timestamps_match = self.status.view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(
                self.status.times.created_at,
                self.status.times.modified_at,
            )
        });

        if timestamps_match {
            ComparisonBranch::Fresh(Self::transition(Construction, Fresh {
                id: self.status.id,
            }))
        } else {
            ComparisonBranch::Suspect(Self::transition(Comparison, Suspect {
                id: self.status.id,
                times: self.status.times,
                view: self.status.view,
                content: content.into(),
            }))
        }
    }
}

/// Operations valid when timestamp drift requires content hashing.
impl SchemaProcessor<Comparison, Suspect> {
    #[inline]
    pub(crate) fn check_content(self) -> ContentBranch {
        let content_hash = blake3::hash(self.status.content.as_bytes());
        let content_match = self.status.view.current().is_some_and(|v| {
            v.hashes().is_content_match(content_hash.as_bytes())
        });

        if content_match {
            ContentBranch::StaleTimestamps(Self::transition(
                Refresh,
                StaleTimestamps {
                    id: self.status.id,
                    times: self.status.times,
                    view: self.status.view,
                },
            ))
        } else {
            ContentBranch::StaleContent(Self::transition(
                Refresh,
                StaleContent {
                    id: self.status.id,
                    times: self.status.times,
                    view: self.status.view,
                    content_hash: *content_hash.as_bytes(),
                },
            ))
        }
    }
}

/// Operations valid when only timestamps must be refreshed.
impl SchemaProcessor<Refresh, StaleTimestamps> {
    #[inline]
    pub(crate) fn sync_metadata<R: Repository>(
        self,
        _repository: &R,
    ) -> Result<SchemaProcessor<Construction, Fresh>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        // TODO: Implement metadata sync for stale timestamps
        // This is scaffold code - will be implemented in later phases
        todo!("sync metadata for stale timestamps")
    }
}

/// Operations valid when content hash must be refreshed.
impl SchemaProcessor<Refresh, StaleContent> {
    #[inline]
    pub(crate) fn sync_metadata<R: Repository>(
        self,
        repository: &R,
    ) -> Result<SchemaProcessor<Construction, Fresh>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let new_file_times = FileTimesMetadata::new(
            self.status.times.created_at,
            self.status.times.modified_at,
        );

        let mut view = self.status.view;
        let raw: RawSchema = view
            .to_raw()
            .map_err(SchemaLoaderError::Ingestion)?
            .ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                    SchemaStorageError::NotFound {
                        name: "raw schema view".into(),
                    },
                ))
            })?;

        let property_hashes =
            HashMetadata::compute_property_hashes(raw.properties());
        let hashes =
            HashMetadata::new(self.status.content_hash, property_hashes);
        let version = SchemaVersion::new(new_file_times, hashes, &raw)
            .map_err(SchemaLoaderError::Ingestion)?;

        view.add_version(version);

        repository
            .save_raw_schema_view(self.status.id, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(Self::transition(Construction, Fresh {
            id: self.status.id,
        }))
    }
}

/// Operations valid when constructing changed schemas.
impl SchemaProcessor<Construction, Changed> {
    #[inline]
    pub(crate) fn build(self) -> Result<ConstructionBranch, SchemaLoaderError> {
        todo!("schema construction for changed schemas")
    }
}

/// Operations valid when constructing new schemas.
impl SchemaProcessor<Construction, New> {
    #[inline]
    pub(crate) fn build(self) -> Result<ConstructionBranch, SchemaLoaderError> {
        todo!("schema construction for new schemas")
    }
}

/// Operations valid once processing is complete.
impl SchemaProcessor<Completed, Ready> {
    #[inline]
    #[must_use]
    pub(crate) fn into_schemas(self) -> Vec<Schema> {
        self.status.schemas
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    //  ExtendsDelta Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn extends_delta_unchanged_both_none() {
        let delta = ExtendsDelta {
            old_parent: None,
            new_parent: None,
        };

        assert!(!delta.changed());
        assert_eq!(delta.kind(), ExtendsChangeKind::Unchanged);
    }

    #[test]
    fn extends_delta_unchanged_same_parent() {
        let parent_name: SchemaName = "parent".try_into().unwrap();
        let delta = ExtendsDelta {
            old_parent: Some(parent_name.clone()),
            new_parent: Some(parent_name),
        };

        assert!(!delta.changed());
        assert_eq!(delta.kind(), ExtendsChangeKind::Unchanged);
    }

    #[test]
    fn extends_delta_root_to_child() {
        let parent_name: SchemaName = "parent".try_into().unwrap();
        let delta = ExtendsDelta {
            old_parent: None,
            new_parent: Some(parent_name),
        };

        assert!(delta.changed());
        assert_eq!(delta.kind(), ExtendsChangeKind::RootToChild);
    }

    #[test]
    fn extends_delta_child_to_root() {
        let parent_name: SchemaName = "parent".try_into().unwrap();
        let delta = ExtendsDelta {
            old_parent: Some(parent_name),
            new_parent: None,
        };

        assert!(delta.changed());
        assert_eq!(delta.kind(), ExtendsChangeKind::ChildToRoot);
    }

    #[test]
    fn extends_delta_rewired() {
        let old_parent: SchemaName = "old_parent".try_into().unwrap();
        let new_parent: SchemaName = "new_parent".try_into().unwrap();
        let delta = ExtendsDelta {
            old_parent: Some(old_parent),
            new_parent: Some(new_parent),
        };

        assert!(delta.changed());
        assert_eq!(delta.kind(), ExtendsChangeKind::Rewired);
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  ExcludesDelta Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn excludes_delta_empty() {
        let delta = ExcludesDelta {
            added: vec![],
            removed: vec![],
        };

        assert!(delta.is_empty());
        assert!(!delta.changed());
    }

    #[test]
    fn excludes_delta_with_additions() {
        let prop_a: PropertyName = "prop_a".try_into().unwrap();
        let delta = ExcludesDelta {
            added: vec![prop_a],
            removed: vec![],
        };

        assert!(!delta.is_empty());
        assert!(delta.changed());
    }

    #[test]
    fn excludes_delta_with_removals() {
        let prop_a: PropertyName = "prop_a".try_into().unwrap();
        let delta = ExcludesDelta {
            added: vec![],
            removed: vec![prop_a],
        };

        assert!(!delta.is_empty());
        assert!(delta.changed());
    }

    #[test]
    fn excludes_delta_with_both() {
        let prop_a: PropertyName = "prop_a".try_into().unwrap();
        let prop_b: PropertyName = "prop_b".try_into().unwrap();
        let delta = ExcludesDelta {
            added: vec![prop_a],
            removed: vec![prop_b],
        };

        assert!(!delta.is_empty());
        assert!(delta.changed());
    }

    #[test]
    fn excludes_delta_default_is_empty() {
        let delta = ExcludesDelta::default();
        assert!(delta.is_empty());
        assert!(!delta.changed());
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  SchemaPropertyDelta Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn schema_property_delta_empty() {
        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts::default(),
            removed: vec![],
        };

        assert!(delta.is_empty());
        assert!(delta.affected_properties().is_empty());
    }

    #[test]
    fn schema_property_delta_default_is_empty() {
        let delta = SchemaPropertyDelta::default();
        assert!(delta.is_empty());
    }

    #[test]
    fn schema_property_delta_with_inline_upserts() {
        let prop_name: PropertyName = "test_prop".try_into().unwrap();
        let mut inline = HashMap::new();

        // Create a minimal RawPropertyMap for testing
        let raw_json = serde_json::json!({
            "nested": { "type": "string" }
        });
        let raw_map: RawPropertyMap<RawProperty> =
            serde_json::from_value(raw_json).unwrap();

        inline.insert(prop_name.clone(), raw_map);

        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts {
                inline,
                refs: HashMap::new(),
            },
            removed: vec![],
        };

        assert!(!delta.is_empty());

        let affected = delta.affected_properties();
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&prop_name));
    }

    #[test]
    fn schema_property_delta_with_ref_upserts() {
        let prop_name: PropertyName = "test_prop".try_into().unwrap();
        let bank_ref: PropertyName = "bank_prop".try_into().unwrap();

        let mut refs = HashMap::new();
        refs.insert(prop_name.clone(), bank_ref);

        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts {
                inline: HashMap::new(),
                refs,
            },
            removed: vec![],
        };

        assert!(!delta.is_empty());

        let affected = delta.affected_properties();
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&prop_name));
    }

    #[test]
    fn schema_property_delta_with_removals() {
        let prop_name: PropertyName = "removed_prop".try_into().unwrap();

        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts::default(),
            removed: vec![prop_name.clone()],
        };

        assert!(!delta.is_empty());

        let affected = delta.affected_properties();
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&prop_name));
    }

    #[test]
    fn schema_property_delta_with_all_types() {
        let inline_prop: PropertyName = "inline_prop".try_into().unwrap();
        let ref_prop: PropertyName = "ref_prop".try_into().unwrap();
        let removed_prop: PropertyName = "removed_prop".try_into().unwrap();
        let bank_ref: PropertyName = "bank_prop".try_into().unwrap();

        let mut inline = HashMap::new();
        let raw_json = serde_json::json!({
            "field": { "type": "bool" }
        });
        let raw_map: RawPropertyMap<RawProperty> =
            serde_json::from_value(raw_json).unwrap();
        inline.insert(inline_prop.clone(), raw_map);

        let mut refs = HashMap::new();
        refs.insert(ref_prop.clone(), bank_ref);

        let delta = SchemaPropertyDelta {
            upserts: SchemaPropertyUpserts {
                inline,
                refs,
            },
            removed: vec![removed_prop.clone()],
        };

        assert!(!delta.is_empty());

        let affected = delta.affected_properties();
        assert_eq!(affected.len(), 3);
        assert!(affected.contains(&inline_prop));
        assert!(affected.contains(&ref_prop));
        assert!(affected.contains(&removed_prop));
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  SchemaPropertyUpserts Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn schema_property_upserts_empty() {
        let upserts = SchemaPropertyUpserts {
            inline: HashMap::new(),
            refs: HashMap::new(),
        };

        assert!(upserts.is_empty());
        assert_eq!(upserts.len(), 0);
    }

    #[test]
    fn schema_property_upserts_default_is_empty() {
        let upserts = SchemaPropertyUpserts::default();
        assert!(upserts.is_empty());
        assert_eq!(upserts.len(), 0);
    }

    #[test]
    fn schema_property_upserts_len_counts_both() {
        let inline_prop: PropertyName = "inline".try_into().unwrap();
        let ref_prop: PropertyName = "ref".try_into().unwrap();
        let bank_ref: PropertyName = "bank".try_into().unwrap();

        let mut inline = HashMap::new();
        let raw_json = serde_json::json!({ "x": { "type": "number" } });
        let raw_map: RawPropertyMap<RawProperty> =
            serde_json::from_value(raw_json).unwrap();
        inline.insert(inline_prop, raw_map);

        let mut refs = HashMap::new();
        refs.insert(ref_prop, bank_ref);

        let upserts = SchemaPropertyUpserts {
            inline,
            refs,
        };

        assert!(!upserts.is_empty());
        assert_eq!(upserts.len(), 2);
    }

    #[test]
    fn schema_property_upserts_with_only_inline() {
        let prop: PropertyName = "prop".try_into().unwrap();

        let mut inline = HashMap::new();
        let raw_json = serde_json::json!({ "field": { "type": "string" } });
        let raw_map: RawPropertyMap<RawProperty> =
            serde_json::from_value(raw_json).unwrap();
        inline.insert(prop, raw_map);

        let upserts = SchemaPropertyUpserts {
            inline,
            refs: HashMap::new(),
        };

        assert!(!upserts.is_empty());
        assert_eq!(upserts.len(), 1);
    }

    #[test]
    fn schema_property_upserts_with_only_refs() {
        let prop: PropertyName = "prop".try_into().unwrap();
        let bank: PropertyName = "bank".try_into().unwrap();

        let mut refs = HashMap::new();
        refs.insert(prop, bank);

        let upserts = SchemaPropertyUpserts {
            inline: HashMap::new(),
            refs,
        };

        assert!(!upserts.is_empty());
        assert_eq!(upserts.len(), 1);
    }
}
