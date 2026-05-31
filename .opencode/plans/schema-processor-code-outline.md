# Schema Processor - Complete Code Outline

**Date**: 2026-04-05
**Status**: IMPLEMENTATION GUIDE
**File**: `lithos-core/src/schema/schema_processor.rs`

---

## Table of Contents

1. [Phase 3: Discovery Stage](#phase-3-discovery-stage)
2. [Phase 4: TimeComparison & ContentComparison](#phase-4-timecomparison--contentcomparison)
3. [Phase 5: FileParsed Stage](#phase-5-fileparsed-stage)
4. [Phase 6: InheritanceGraphed Stage](#phase-6-inheritancegraphed-stage)
5. [Phase 7: PropertyAnalysis Stage](#phase-7-propertyanalysis-stage)
6. [Phase 8: Refresh Stage](#phase-8-refresh-stage)
7. [Phase 9: Construction Stage](#phase-9-construction-stage)
8. [Phase 10: Completion Stage](#phase-10-completion-stage)
9. [Helper Functions](#helper-functions)
10. [Integration Points](#integration-points)

---

## Phase 3: Discovery Stage

### State Structs

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for Discovery stage - Unknown status.
#[derive(Debug)]
pub(crate) struct DiscoveryState {
    /// Discovery context from Builder.
    pub(crate) context: DiscoveryContext,
}

/// State for Missing branch - new schemas to parse.
#[derive(Debug)]
pub(crate) struct MissingBatch {
    /// Batch of missing schemas (no view in DB).
    pub(crate) batch: HashMap<SchemaId, MissingPayload>,
}

/// State for Present branch - existing schemas to compare.
#[derive(Debug)]
pub(crate) struct PresentBatch {
    /// Persisted graph from DB.
    pub(crate) graph: TopologicalGraph<GraphNode<PresentPayload>>,
    /// Batch of present schemas (have view in DB).
    pub(crate) batch: HashMap<SchemaId, PresentPayload>,
    /// Deleted schema IDs (in graph but no file).
    pub(crate) deleted_ids: Vec<SchemaId>,
}
```

### Branching Enum

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  BRANCHING ENUMS
// ═════════════════════════════════════════════════════════════════════════════

/// Branch from Discovery stage.
pub(crate) enum DiscoveryBranch {
    /// No cached views exist; all files are new.
    AllMissing(SchemaProcessor<FileParsed, MissingBatch>),

    /// Some/all files have cached views; proceed to timestamp comparison.
    SomePresent {
        missing: Option<SchemaProcessor<FileParsed, MissingBatch>>,
        present: SchemaProcessor<TimeComparison, PresentBatch>,
    },
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  DISCOVERY STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Discovery, DiscoveryState> {
    /// Create processor from discovery context.
    pub(crate) fn from_context(context: DiscoveryContext) -> Self {
        Self {
            status: DiscoveryState { context },
            _stage: PhantomData,
        }
    }

    /// Discover and classify schemas.
    ///
    /// # Process
    ///
    /// 1. Query DB for views by file paths
    /// 2. Classify each file:
    ///    - Missing: No view in DB (new schema)
    ///    - Present: View exists in DB (check staleness)
    /// 3. Check graph for deleted schemas (ID in graph, no file)
    /// 4. Build batches and embed in graph nodes
    ///
    /// # Returns
    ///
    /// Branching enum based on classification results.
    pub(crate) fn discover<R: Repository>(
        self,
        repository: &R,
    ) -> Result<DiscoveryBranch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let DiscoveryState { context } = self.status;
        let DiscoveryContext { graph, files, .. } = context;

        // 1. Query DB for views by paths
        let views_by_path = repository
            .find_raw_schema_views_by_paths(&files)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // Build path -> view map for efficient lookups
        let mut view_map: HashMap<PathBuf, (SchemaId, RawSchemaView)> = HashMap::new();
        for (id, view) in views_by_path {
            if let Some(path) = files.iter().find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name == view.filename().as_str())
                    .unwrap_or(false)
            }) {
                view_map.insert(path.clone(), (id, view));
            }
        }

        // 2. Classify files as Missing or Present
        let mut missing_batch: HashMap<SchemaId, MissingPayload> = HashMap::new();
        let mut present_batch: HashMap<SchemaId, PresentPayload> = HashMap::new();

        for path in &files {
            let times = RawFileTimes {
                created_at: /* get from filesystem */,
                modified_at: /* get from filesystem */,
            };

            if let Some((id, view)) = view_map.get(path) {
                // Present: view exists
                present_batch.insert(
                    *id,
                    PresentPayload {
                        path: path.clone(),
                        times,
                        view: view.clone(),
                    },
                );
            } else {
                // Missing: no view (new schema)
                let id = SchemaId::new();
                missing_batch.insert(
                    id,
                    MissingPayload {
                        path: path.clone(),
                        times,
                    },
                );
            }
        }

        // 3. Check for deleted schemas
        let mut deleted_ids = Vec::new();
        if let Some(graph) = &graph {
            let file_ids: HashSet<SchemaId> = present_batch.keys().copied().collect();
            for id in graph.nodes.keys() {
                if !file_ids.contains(id) && !missing_batch.contains_key(id) {
                    deleted_ids.push(*id);
                }
            }
        }

        // 4. Branch based on classification
        match (missing_batch.is_empty(), present_batch.is_empty()) {
            (false, true) => {
                // All missing (first run or all files deleted)
                Ok(DiscoveryBranch::AllMissing(SchemaProcessor {
                    status: MissingBatch {
                        batch: missing_batch,
                    },
                    _stage: PhantomData,
                }))
            }
            (_, false) => {
                // Some present (normal case)
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

                // Hydrate graph with present payloads
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
                // No files at all - return error
                Err(SchemaLoaderError::Ingestion(/* no schemas error */))
            }
        }
    }
}
```

---

## Phase 4: TimeComparison & ContentComparison

### State Structs

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  TIMECOMPARISON STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for Fresh batch - timestamps match.
#[derive(Debug)]
pub(crate) struct FreshBatch {
    /// Graph with fresh payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<FreshPayload>>,
    /// Batch of fresh schemas.
    pub(crate) batch: HashMap<SchemaId, FreshPayload>,
}

/// State for StaleSuspect batch - timestamps don't match.
#[derive(Debug)]
pub(crate) struct StaleSuspectBatch {
    /// Graph with stale suspect payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<StaleSuspectPayload>>,
    /// Batch of stale suspect schemas.
    pub(crate) batch: HashMap<SchemaId, StaleSuspectPayload>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  CONTENTCOMPARISON STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for StaleTimestamp batch - content hash matches.
#[derive(Debug)]
pub(crate) struct StaleTimestampBatch {
    /// Graph with stale timestamp payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<StaleTimestampPayload>>,
    /// Batch of stale timestamp schemas.
    pub(crate) batch: HashMap<SchemaId, StaleTimestampPayload>,
}

/// State for StaleContent batch - content hash doesn't match.
#[derive(Debug)]
pub(crate) struct StaleContentBatch {
    /// Graph with stale content payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<StaleContentSuspectPayload>>,
    /// Batch of stale content schemas.
    pub(crate) batch: HashMap<SchemaId, StaleContentSuspectPayload>,
}
```

### Branching Enums

```rust
/// Branch from TimeComparison stage.
pub(crate) enum TimestampBranch {
    /// All timestamps match; schemas are fresh.
    AllFresh(SchemaProcessor<InheritanceGraphed, FreshBatch>),

    /// Some/all timestamps don't match; need content comparison.
    SomeSuspect {
        fresh: Option<SchemaProcessor<InheritanceGraphed, FreshBatch>>,
        suspect: SchemaProcessor<ContentComparison, StaleSuspectBatch>,
    },
}

/// Branch from ContentComparison stage.
pub(crate) enum ContentBranch {
    /// All content hashes match; only timestamps stale.
    AllStaleTimestamps(SchemaProcessor<Refresh, StaleTimestampBatch>),

    /// Some/all content changed; need parsing.
    SomeStaleContent {
        timestamps: Option<SchemaProcessor<Refresh, StaleTimestampBatch>>,
        content: SchemaProcessor<FileParsed, StaleContentBatch>,
    },
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  TIMECOMPARISON STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<TimeComparison, PresentBatch> {
    /// Compare file timestamps against cached views.
    ///
    /// # Process
    ///
    /// For each schema in batch:
    /// 1. Get file times from view
    /// 2. Compare with current file times
    /// 3. If match → Fresh (no processing needed)
    /// 4. If mismatch → StaleSuspect (read file content)
    pub(crate) fn compare_timestamps<FsSource>(
        self,
        source: &FsSource,
    ) -> Result<TimestampBranch, SchemaLoaderError>
    where
        FsSource: /* FileSource trait */,
    {
        let PresentBatch { graph, batch, .. } = self.status;

        let mut fresh_batch: HashMap<SchemaId, FreshPayload> = HashMap::new();
        let mut suspect_batch: HashMap<SchemaId, StaleSuspectPayload> = HashMap::new();

        for (id, present) in batch {
            let PresentPayload { path, times, view } = present;

            // Check if timestamps match
            let timestamps_match = view
                .current()
                .map(|v| v.file_times().is_timestamp_match(
                    times.created_at,
                    times.modified_at,
                ))
                .unwrap_or(false);

            if timestamps_match {
                // Fresh: no changes
                fresh_batch.insert(
                    id,
                    FreshPayload { path, view },
                );
            } else {
                // Suspect: need to read content
                let content_str = source.read_to_string(&path)?;
                suspect_batch.insert(
                    id,
                    StaleSuspectPayload {
                        path,
                        times,
                        content_str: content_str.into(),
                        view,
                    },
                );
            }
        }

        // Branch based on results
        match (fresh_batch.is_empty(), suspect_batch.is_empty()) {
            (false, true) => {
                // All fresh
                let graph = hydrate_graph_with_fresh(graph, &fresh_batch);
                Ok(TimestampBranch::AllFresh(SchemaProcessor {
                    status: FreshBatch { graph, batch: fresh_batch },
                    _stage: PhantomData,
                }))
            }
            (_, false) => {
                // Some suspect
                let fresh_processor = if fresh_batch.is_empty() {
                    None
                } else {
                    let graph = hydrate_graph_with_fresh(graph.clone(), &fresh_batch);
                    Some(SchemaProcessor {
                        status: FreshBatch { graph, batch: fresh_batch },
                        _stage: PhantomData,
                    })
                };

                let graph = hydrate_graph_with_suspect(graph, &suspect_batch);
                Ok(TimestampBranch::SomeSuspect {
                    fresh: fresh_processor,
                    suspect: SchemaProcessor {
                        status: StaleSuspectBatch { graph, batch: suspect_batch },
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
    /// Compare content hashes against cached views.
    ///
    /// # Process
    ///
    /// For each schema in batch:
    /// 1. Hash file content with blake3
    /// 2. Compare with view content hash
    /// 3. If match → StaleTimestamp (only times changed)
    /// 4. If mismatch → StaleContent (parse required)
    pub(crate) fn compare_content(
        self,
    ) -> Result<ContentBranch, SchemaLoaderError> {
        let StaleSuspectBatch { graph, batch } = self.status;

        let mut timestamp_batch: HashMap<SchemaId, StaleTimestampPayload> = HashMap::new();
        let mut content_batch: HashMap<SchemaId, StaleContentSuspectPayload> = HashMap::new();

        for (id, suspect) in batch {
            let StaleSuspectPayload { path, times, content_str, view } = suspect;

            // Hash content
            let content_hash = *blake3::hash(content_str.as_bytes()).as_bytes();

            // Check if content hash matches
            let content_match = view
                .current()
                .map(|v| v.hashes().is_content_match(&content_hash))
                .unwrap_or(false);

            if content_match {
                // StaleTimestamp: only times changed
                timestamp_batch.insert(
                    id,
                    StaleTimestampPayload { path, times, view },
                );
            } else {
                // StaleContent: need to parse
                let raw = parse_raw_schema_from_str(&path, &content_str, &times)?;
                content_batch.insert(
                    id,
                    StaleContentSuspectPayload {
                        path,
                        times,
                        content_hash,
                        raw,
                        view,
                    },
                );
            }
        }

        // Branch based on results
        match (timestamp_batch.is_empty(), content_batch.is_empty()) {
            (false, true) => {
                // All stale timestamps
                let graph = hydrate_graph_with_stale_timestamp(graph, &timestamp_batch);
                Ok(ContentBranch::AllStaleTimestamps(SchemaProcessor {
                    status: StaleTimestampBatch { graph, batch: timestamp_batch },
                    _stage: PhantomData,
                }))
            }
            (_, false) => {
                // Some stale content
                let timestamp_processor = if timestamp_batch.is_empty() {
                    None
                } else {
                    let graph = hydrate_graph_with_stale_timestamp(graph.clone(), &timestamp_batch);
                    Some(SchemaProcessor {
                        status: StaleTimestampBatch { graph, batch: timestamp_batch },
                        _stage: PhantomData,
                    })
                };

                let graph = hydrate_graph_with_stale_content(graph, &content_batch);
                Ok(ContentBranch::SomeStaleContent {
                    timestamps: timestamp_processor,
                    content: SchemaProcessor {
                        status: StaleContentBatch { graph, batch: content_batch },
                        _stage: PhantomData,
                    },
                })
            }
            (true, true) => unreachable!("empty batch"),
        }
    }
}
```

---

## Phase 5: FileParsed Stage

### State Struct

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  FILEPARSED STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for parsed schemas (new or stale content).
#[derive(Debug)]
pub(crate) struct ParsedBatch {
    /// Batch of new schemas.
    pub(crate) new_schemas: HashMap<SchemaId, NewPayload>,
    /// Batch of stale content schemas.
    pub(crate) stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  FILEPARSED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<FileParsed, MissingBatch> {
    /// Parse new schema files.
    ///
    /// # Process
    ///
    /// For each missing schema:
    /// 1. Read file content
    /// 2. Parse to RawSchema
    /// 3. Hash content
    /// 4. Optionally: Expand properties early and save view
    pub(crate) fn parse_new_schemas<FsSource>(
        self,
        source: &FsSource,
    ) -> Result<ParsedBatch, SchemaLoaderError>
    where
        FsSource: /* FileSource trait */,
    {
        let MissingBatch { batch } = self.status;
        let mut new_schemas: HashMap<SchemaId, NewPayload> = HashMap::new();

        for (id, missing) in batch {
            let MissingPayload { path, times } = missing;

            // Read and parse
            let content = source.read_to_string(&path)?;
            let content_hash = *blake3::hash(content.as_bytes()).as_bytes();
            let raw = parse_raw_schema_from_str(&path, &content, &times)?;

            new_schemas.insert(
                id,
                NewPayload {
                    path,
                    times,
                    content_hash,
                    raw,
                },
            );
        }

        Ok(ParsedBatch {
            new_schemas,
            stale_schemas: HashMap::new(),
        })
    }
}

impl SchemaProcessor<FileParsed, StaleContentBatch> {
    /// Schemas already parsed in ContentComparison.
    ///
    /// Just pass through to InheritanceGraphed.
    pub(crate) fn pass_through(self) -> ParsedBatch {
        let StaleContentBatch { batch, .. } = self.status;

        ParsedBatch {
            new_schemas: HashMap::new(),
            stale_schemas: batch,
        }
    }
}
```

---

## Phase 6: InheritanceGraphed Stage

### State Struct

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  INHERITANCEGRAPHED STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for schemas in inheritance graph.
#[derive(Debug)]
pub(crate) struct GraphedBatch {
    /// Unified graph with all schemas.
    pub(crate) graph: TopologicalGraph<GraphNode<GraphedPayload>>,
    /// Deleted schema IDs.
    pub(crate) deleted_ids: Vec<SchemaId>,
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  INHERITANCEGRAPHED STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl ParsedBatch {
    /// Build or patch inheritance graph.
    ///
    /// # Process
    ///
    /// 1. If no graph exists (first run):
    ///    - Build graph from scratch
    ///    - All schemas: ExtendsChangeKind::Unchanged (new)
    ///
    /// 2. If graph exists:
    ///    - Insert new schemas
    ///    - Update stale schemas
    ///    - Detect extends changes
    ///    - Update graph structure
    ///
    /// 3. Validate topological sort
    /// 4. Detect cycles
    pub(crate) fn build_graph(
        self,
        existing_graph: Option<TopologicalGraph<InheritanceNode>>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        let ParsedBatch { new_schemas, stale_schemas } = self;

        match existing_graph {
            None => {
                // NewGraph: Build from scratch
                self.build_new_graph(new_schemas, stale_schemas)
            }
            Some(graph) => {
                // PatchGraph: Update existing
                self.patch_graph(graph, new_schemas, stale_schemas)
            }
        }
    }

    fn build_new_graph(
        self,
        new_schemas: HashMap<SchemaId, NewPayload>,
        stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        // Build graph using DagBuilder
        let mut all_schemas: HashMap<SchemaId, &RawSchema> = HashMap::new();
        for (id, new) in &new_schemas {
            all_schemas.insert(*id, &new.raw);
        }
        for (id, stale) in &stale_schemas {
            all_schemas.insert(*id, &stale.raw);
        }

        let graph = DagBuilder::new(&all_schemas).build()?;

        // Hydrate with GraphedPayload
        let mut nodes: HashMap<SchemaId, GraphNode<GraphedPayload>> = HashMap::new();
        for (id, node) in graph.nodes {
            let payload = if let Some(new) = new_schemas.get(&id) {
                GraphedPayload {
                    path: new.path.clone(),
                    times: new.times,
                    content_hash: new.content_hash,
                    raw: new.raw.clone(),
                    view: None,
                    extends_change: ExtendsChangeKind::Unchanged,
                }
            } else if let Some(stale) = stale_schemas.get(&id) {
                GraphedPayload {
                    path: stale.path.clone(),
                    times: stale.times,
                    content_hash: stale.content_hash,
                    raw: stale.raw.clone(),
                    view: Some(stale.view.clone()),
                    extends_change: ExtendsChangeKind::Unchanged,
                }
            } else {
                unreachable!("schema in graph but not in batch")
            };

            nodes.insert(id, node.with_payload(payload));
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

    fn patch_graph(
        self,
        mut graph: TopologicalGraph<InheritanceNode>,
        new_schemas: HashMap<SchemaId, NewPayload>,
        stale_schemas: HashMap<SchemaId, StaleContentSuspectPayload>,
    ) -> Result<GraphedBatch, SchemaLoaderError> {
        // 1. Insert new schemas
        for (id, new) in &new_schemas {
            let parent_id = new.raw.extends()
                .and_then(|name| /* resolve SchemaName to SchemaId */);

            let node = InheritanceNode {
                id: *id,
                parents: parent_id.map(|p| vec![p]).unwrap_or_default(),
                children: Vec::new(),
                depth: NodeDepth::ROOT, // Will be recomputed
            };
            graph.nodes.insert(*id, node);
        }

        // 2. Detect extends changes for stale schemas
        let mut extends_changes: HashMap<SchemaId, ExtendsChangeKind> = HashMap::new();
        for (id, stale) in &stale_schemas {
            let old_parent = graph.nodes.get(id)
                .and_then(|node| node.parents.first().copied());

            let new_parent = stale.raw.extends()
                .and_then(|name| /* resolve SchemaName to SchemaId */);

            let change_kind = match (old_parent, new_parent) {
                (None, None) => ExtendsChangeKind::Unchanged,
                (None, Some(_)) => ExtendsChangeKind::RootToChild,
                (Some(_), None) => ExtendsChangeKind::ChildToRoot,
                (Some(old), Some(new)) if old == new => ExtendsChangeKind::Unchanged,
                (Some(_), Some(_)) => ExtendsChangeKind::Rewired,
            };

            extends_changes.insert(*id, change_kind);

            // Update graph if changed
            if change_kind != ExtendsChangeKind::Unchanged {
                if let Some(node) = graph.nodes.get_mut(id) {
                    node.parents = new_parent.map(|p| vec![p]).unwrap_or_default();
                }
            }
        }

        // 3. Recompute depths and topological order
        graph.compute_depths();
        let (order, roots) = graph.topological_sort()?;
        graph.order = order;
        graph.roots = roots;

        // 4. Detect cycles
        DagValidator::new(&graph).validate()?;

        // 5. Hydrate with GraphedPayload
        let mut nodes: HashMap<SchemaId, GraphNode<GraphedPayload>> = HashMap::new();
        for (id, node) in graph.nodes {
            let payload = if let Some(new) = new_schemas.get(&id) {
                GraphedPayload {
                    path: new.path.clone(),
                    times: new.times,
                    content_hash: new.content_hash,
                    raw: new.raw.clone(),
                    view: None,
                    extends_change: ExtendsChangeKind::Unchanged,
                }
            } else if let Some(stale) = stale_schemas.get(&id) {
                let change_kind = extends_changes.get(&id)
                    .copied()
                    .unwrap_or(ExtendsChangeKind::Unchanged);

                GraphedPayload {
                    path: stale.path.clone(),
                    times: stale.times,
                    content_hash: stale.content_hash,
                    raw: stale.raw.clone(),
                    view: Some(stale.view.clone()),
                    extends_change: change_kind,
                }
            } else {
                // Existing fresh schema - fetch from somewhere
                continue;
            };

            nodes.insert(id, node.with_payload(payload));
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
```

---

## Phase 7: PropertyAnalysis Stage

### State Struct

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  PROPERTYANALYSIS STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for analyzed schemas.
#[derive(Debug)]
pub(crate) struct AnalyzedBatch {
    /// Graph with analyzed payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<AnalyzedPayload>>,
    /// IDs needing refresh (unchanged semantics).
    pub(crate) refresh_ids: HashSet<SchemaId>,
    /// IDs needing construction (changed semantics).
    pub(crate) rebuild_ids: HashSet<SchemaId>,
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  PROPERTYANALYSIS STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<InheritanceGraphed, GraphedBatch> {
    /// Analyze schemas for property and excludes changes.
    ///
    /// # Process
    ///
    /// For each stale schema:
    /// 1. Compare excludes: raw.excludes vs view.excludes → ExcludesDelta
    /// 2. Hash properties: compare hashes → SchemaPropertyDelta
    /// 3. Check PropertyBankDelta against view.bank_references
    /// 4. Classify:
    ///    - Unchanged → Refresh
    ///    - Changed → Construction
    pub(crate) fn analyze_properties(
        self,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> Result<AnalyzedBatch, SchemaLoaderError> {
        let GraphedBatch { graph, .. } = self.status;

        let mut refresh_ids: HashSet<SchemaId> = HashSet::new();
        let mut rebuild_ids: HashSet<SchemaId> = HashSet::new();
        let mut analyzed_nodes: HashMap<SchemaId, GraphNode<AnalyzedPayload>> = HashMap::new();

        for (id, node) in graph.nodes {
            let GraphedPayload {
                path,
                times,
                content_hash,
                raw,
                view,
                extends_change,
            } = node.payload;

            // New schemas always need construction
            if view.is_none() {
                rebuild_ids.insert(id);
                continue;
            }

            let view = view.unwrap();

            // 1. Compare excludes
            let excludes_delta = diff_excludes(
                view.current().map(|v| v.excludes()).unwrap_or(&[]),
                raw.excludes(),
            );

            // 2. Hash and compare properties
            let property_delta = diff_properties(
                &raw,
                view.current().map(|v| v.hashes().properties()).unwrap_or(&HashMap::new()),
            );

            // 3. Check PropertyBankDelta
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

            // 4. Classify
            let needs_rebuild = extends_change != ExtendsChangeKind::Unchanged
                || !excludes_delta.is_empty()
                || !property_delta.is_empty()
                || bank_changed;

            if needs_rebuild {
                rebuild_ids.insert(id);
            } else {
                refresh_ids.insert(id);
            }

            analyzed_nodes.insert(
                id,
                node.with_payload(AnalyzedPayload {
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
                }),
            );
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
```

---

## Phase 8: Refresh Stage

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  REFRESH STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Refresh, AnalyzedBatch> {
    /// Refresh views for schemas with unchanged semantics.
    ///
    /// # Process
    ///
    /// For each schema in refresh_ids:
    /// 1. Build new SchemaVersion with updated times/hashes
    /// 2. Add version to view
    /// 3. Save view to DB
    /// 4. Transition to Fresh status
    pub(crate) fn refresh_metadata<R: Repository>(
        self,
        repository: &R,
    ) -> Result<AnalyzedBatch, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let AnalyzedBatch {
            mut graph,
            refresh_ids,
            rebuild_ids,
        } = self.status;

        for id in &refresh_ids {
            if let Some(node) = graph.nodes.get_mut(id) {
                let payload = &node.payload;

                // Compute property hashes
                let property_hashes = HashMetadata::compute_property_hashes(
                    payload.raw.properties(),
                );

                // Build new version
                let file_times = FileTimesMetadata::new(
                    payload.times.created_at,
                    payload.times.modified_at,
                );
                let hashes = HashMetadata::new(
                    payload.content_hash,
                    property_hashes,
                );
                let version = SchemaVersion::new(file_times, hashes, &payload.raw)?;

                // Update view
                let mut view = payload.view.clone();
                view.add_version(version);

                // Save to DB
                repository
                    .save_raw_schema_view(*id, &view)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

                // Update payload in node
                // (view is now updated)
            }
        }

        Ok(AnalyzedBatch {
            graph,
            refresh_ids,
            rebuild_ids,
        })
    }
}
```

---

## Phase 9: Construction Stage

### State Struct

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  CONSTRUCTION STAGE STATE
// ═════════════════════════════════════════════════════════════════════════════

/// State for construction stage.
#[derive(Debug)]
pub(crate) struct ConstructionState {
    /// Graph with analyzed payloads.
    pub(crate) graph: TopologicalGraph<GraphNode<AnalyzedPayload>>,
    /// IDs that were refreshed (fetch from DB).
    pub(crate) refresh_ids: HashSet<SchemaId>,
    /// IDs that need rebuild.
    pub(crate) rebuild_ids: HashSet<SchemaId>,
    /// Constructed schemas.
    pub(crate) schemas: Vec<Arc<Schema>>,
}
```

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  CONSTRUCTION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Construction, AnalyzedBatch> {
    /// Build schemas with incremental optimizations.
    ///
    /// # Strategy
    ///
    /// Based on ExtendsChangeKind:
    /// - Unchanged + no property delta → Fetch from DB
    /// - Unchanged + property delta → Update (incremental)
    /// - RootToChild / Rewired → Merge (full)
    /// - ChildToRoot → Construct (no merge)
    /// - New → Full (root or child)
    pub(crate) fn construct_schemas<R: Repository>(
        self,
        repository: &R,
        property_bank: &PropertyBank,
    ) -> Result<ConstructionState, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let AnalyzedBatch { graph, refresh_ids, rebuild_ids } = self.status;

        // 1. Fetch all refresh schemas from DB
        let fetch_ids: Vec<SchemaId> = refresh_ids.iter().copied().collect();
        let fetched = repository
            .find_schemas_by_ids(&fetch_ids)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let mut fetched_by_id: HashMap<SchemaId, Schema> = fetched
            .into_iter()
            .map(|s| (*s.id(), s))
            .collect();

        // 2. Expand properties for rebuild schemas
        let expand_ids: Vec<(SchemaId, RawSchema)> = rebuild_ids
            .iter()
            .filter_map(|id| {
                graph.nodes.get(id).map(|node| (*id, node.payload.raw.clone()))
            })
            .collect();

        let expanded = RefExpander::new(property_bank)
            .expand_all(expand_ids)
            .map_err(SchemaLoaderError::Resolution)?;

        let expanded_by_id: HashMap<SchemaId, ExpandedRaw> = expanded
            .into_iter()
            .collect();

        // 3. Walk graph in topological order and construct
        let mut schemas = Vec::new();
        let mut constructed_cache: HashMap<SchemaId, Schema> = HashMap::new();

        for id in &graph.order {
            let node = graph.nodes.get(id).unwrap();
            let payload = &node.payload;

            let schema = if refresh_ids.contains(id) {
                // Fetch: Already in fetched_by_id
                fetched_by_id.remove(id).unwrap()
            } else if rebuild_ids.contains(id) {
                // Rebuild: Use incremental strategy
                self.construct_schema_incremental(
                    *id,
                    node,
                    &expanded_by_id,
                    &fetched_by_id,
                    &constructed_cache,
                    repository,
                )?
            } else {
                // Fresh: Fetch from DB
                repository
                    .find_schema_by_id(*id)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?
                    .ok_or_else(|| SchemaLoaderError::/* not found */)?
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

    fn construct_schema_incremental<R: Repository>(
        &self,
        id: SchemaId,
        node: &GraphNode<AnalyzedPayload>,
        expanded_by_id: &HashMap<SchemaId, ExpandedRaw>,
        fetched_by_id: &HashMap<SchemaId, Schema>,
        constructed_cache: &HashMap<SchemaId, Schema>,
        repository: &R,
    ) -> Result<Schema, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let payload = &node.payload;
        let extends_change = payload.extends_change;
        let property_delta = &payload.property_delta;

        match (extends_change, property_delta) {
            // Strategy 1: Fetch + Update properties
            (ExtendsChangeKind::Unchanged, Some(delta)) => {
                // Fetch existing schema
                let mut schema = fetched_by_id.get(&id)
                    .or_else(|| constructed_cache.get(&id))
                    .cloned()
                    .ok_or_else(|| SchemaLoaderError::/* not found */)?;

                // Get expanded properties
                let expanded = expanded_by_id.get(&id)
                    .ok_or_else(|| SchemaLoaderError::/* not found */)?;

                // Update properties
                let mut properties = schema.properties().clone();
                for (name, prop) in &expanded.properties {
                    if delta.upserts.inline.contains_key(name)
                        || delta.upserts.refs.contains_key(name)
                    {
                        properties.insert(name.clone(), prop.clone());
                    }
                }
                for name in &delta.removed {
                    properties.remove(name);
                }

                // Build updated schema
                Ok(Schema::new(
                    id,
                    expanded.name.clone(),
                    node.parents.first().copied(),
                    node.children.clone(),
                    properties,
                ))
            }

            // Strategy 2: Full merge (extends changed)
            (ExtendsChangeKind::Rewired | ExtendsChangeKind::RootToChild, _) => {
                let expanded = expanded_by_id.get(&id)
                    .ok_or_else(|| SchemaLoaderError::/* not found */)?;

                // Collect parent properties
                let parent_props = if node.parents.is_empty() {
                    HashMap::new()
                } else {
                    collect_parent_properties(
                        id,
                        &node.parents,
                        constructed_cache,
                        fetched_by_id,
                    )
                };

                // Merge
                let merged = merge_properties(
                    Some(&parent_props),
                    &expanded.properties,
                    &expanded.excludes,
                );

                Ok(Schema::new(
                    id,
                    expanded.name.clone(),
                    node.parents.first().copied(),
                    node.children.clone(),
                    merged,
                ))
            }

            // Strategy 3: Simple construction (became root)
            (ExtendsChangeKind::ChildToRoot, _) => {
                let expanded = expanded_by_id.get(&id)
                    .ok_or_else(|| SchemaLoaderError::/* not found */)?;

                Ok(Schema::new(
                    id,
                    expanded.name.clone(),
                    None,
                    node.children.clone(),
                    expanded.properties.clone(),
                ))
            }

            // Strategy 4: Unchanged + no property delta → Fetch
            (ExtendsChangeKind::Unchanged, None) => {
                fetched_by_id.get(&id)
                    .or_else(|| constructed_cache.get(&id))
                    .cloned()
                    .ok_or_else(|| SchemaLoaderError::/* not found */)
            }
        }
    }
}
```

---

## Phase 10: Completion Stage

### Implementation

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  COMPLETION STAGE IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════

impl SchemaProcessor<Construction, ConstructionState> {
    /// Complete pipeline and persist results.
    ///
    /// # Process
    ///
    /// 1. Save all schemas to DB
    /// 2. Save all views to DB
    /// 3. Dehydrate graph (strip payloads)
    /// 4. Save graph to DB
    /// 5. Return schemas
    pub(crate) fn complete<R: Repository>(
        self,
        repository: &R,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        let ConstructionState { graph, schemas, .. } = self.status;

        // 1. Save schemas
        for schema in &schemas {
            repository
                .save_schema(schema.as_ref())
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }

        // 2. Save views (already done in Refresh, but verify)
        for (id, node) in &graph.nodes {
            let payload = &node.payload;
            repository
                .save_raw_schema_view(*id, &payload.view)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }

        // 3. Dehydrate graph
        let inheritance_graph = dehydrate_graph_to_inheritance(&graph);

        // 4. Save graph
        repository
            .save_topological_graph(&inheritance_graph)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // 5. Return schemas
        Ok(schemas)
    }
}

/// Dehydrate graph by stripping AnalyzedPayload to InheritanceNode.
fn dehydrate_graph_to_inheritance(
    graph: &TopologicalGraph<GraphNode<AnalyzedPayload>>,
) -> TopologicalGraph<InheritanceNode> {
    let mut nodes = HashMap::new();

    for (id, node) in &graph.nodes {
        nodes.insert(
            *id,
            InheritanceNode {
                id: node.id,
                parents: node.parents.clone(),
                children: node.children.clone(),
                depth: node.depth,
            },
        );
    }

    TopologicalGraph {
        order: graph.order.clone(),
        nodes,
        roots: graph.roots.clone(),
    }
}
```

---

## Helper Functions

```rust
// ═════════════════════════════════════════════════════════════════════════════
//  HELPER FUNCTIONS
// ═════════════════════════════════════════════════════════════════════════════

/// Hydrate graph with present payloads.
fn hydrate_graph_with_present(
    graph: TopologicalGraph<InheritanceNode>,
    batch: &HashMap<SchemaId, PresentPayload>,
) -> TopologicalGraph<GraphNode<PresentPayload>> {
    let mut nodes = HashMap::new();

    for (id, node) in graph.nodes {
        if let Some(payload) = batch.get(&id) {
            nodes.insert(id, node.with_payload(payload.clone()));
        }
    }

    TopologicalGraph {
        order: graph.order,
        nodes,
        roots: graph.roots,
    }
}

/// Diff excludes lists.
fn diff_excludes(
    old: &[PropertyName],
    new: &[PropertyName],
) -> ExcludesDelta {
    let old_set: HashSet<&PropertyName> = old.iter().collect();
    let new_set: HashSet<&PropertyName> = new.iter().collect();

    let added: Vec<PropertyName> = new_set
        .difference(&old_set)
        .map(|&name| name.clone())
        .collect();

    let removed: Vec<PropertyName> = old_set
        .difference(&new_set)
        .map(|&name| name.clone())
        .collect();

    ExcludesDelta { added, removed }
}

/// Diff properties by hashing.
fn diff_properties(
    raw: &RawSchema,
    prev_hashes: &HashMap<PropertyName, [u8; 32]>,
) -> SchemaPropertyDelta {
    let mut upserts = SchemaPropertyUpserts::default();
    let mut removed = Vec::new();

    // Compute current hashes
    let current_hashes = HashMetadata::compute_property_hashes(raw.properties());

    // Find upserts (new or changed)
    for (name, current_hash) in &current_hashes {
        let is_changed = prev_hashes
            .get(name)
            .map(|prev| prev != current_hash)
            .unwrap_or(true); // New property

        if is_changed {
            if let Some(inline) = raw.properties().inline.get(name) {
                upserts.inline.insert(name.clone(), inline.clone());
            } else if let Some(ref_prop) = raw.properties().refs.get(name) {
                upserts.refs.insert(name.clone(), ref_prop.clone());
            }
        }
    }

    // Find removed
    for name in prev_hashes.keys() {
        if !current_hashes.contains_key(name) {
            removed.push(name.clone());
        }
    }

    SchemaPropertyDelta { upserts, removed }
}

/// Collect parent properties for merging.
fn collect_parent_properties(
    _child_id: SchemaId,
    parent_ids: &[SchemaId],
    constructed_cache: &HashMap<SchemaId, Schema>,
    fetched_cache: &HashMap<SchemaId, Schema>,
) -> HashMap<PropertyName, Property> {
    let mut properties = HashMap::new();

    for parent_id in parent_ids {
        if let Some(parent) = constructed_cache
            .get(parent_id)
            .or_else(|| fetched_cache.get(parent_id))
        {
            properties.extend(parent.properties().clone());
        }
    }

    properties
}

/// Parse raw schema from file content.
fn parse_raw_schema_from_str(
    path: &Path,
    content: &str,
    times: &RawFileTimes,
) -> Result<RawSchema, SchemaLoaderError> {
    // Use existing ingestor logic
    // This is a placeholder - actual implementation would use SchemaIngestor
    todo!("parse raw schema")
}
```

---

## Integration Points

### Builder Integration

```rust
// In lithos-core/src/schema/builder.rs

impl<'config, R: Repository> Builder<'config, R>
where
    R::Error: Into<SchemaRepositoryError>,
{
    /// Load schemas using v2 batch processor.
    pub fn load_schemas_v2(
        &self,
        property_bank: &PropertyBank,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        // 1. Discovery
        let context = self.discovery_v2()?;

        // 2. Create processor
        let processor = SchemaProcessor::from_context(context);

        // 3. Run discovery stage
        let discovery_branch = processor.discover(&self.repository)?;

        // 4. Process branches
        match discovery_branch {
            DiscoveryBranch::AllMissing(missing) => {
                // Parse → Build Graph → Construct → Complete
                let parsed = missing.parse_new_schemas(&self.source)?;
                let graphed = parsed.build_graph(None)?;
                let analyzed = graphed.analyze_properties(
                    self.property_bank_delta.as_ref()
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed = refreshed.construct_schemas(
                    &self.repository,
                    property_bank,
                )?;
                constructed.complete(&self.repository)
            }

            DiscoveryBranch::SomePresent { missing, present } => {
                // Process present branch
                let timestamp_branch = present.compare_timestamps(&self.source)?;

                // ... (continue processing all branches)

                todo!("process all branches and merge results")
            }
        }
    }
}
```

### Testing Integration

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn first_run_all_new() {
        // No graph in DB, all schemas new
        let temp = TempDir::new().unwrap();
        let repo = InMemoryRepository::new();
        create_test_schemas(&temp);

        let config = setup_config(&temp);
        let source = FileReader::new(temp.path().to_path_buf());
        let builder = Builder::new(repo, source, &config);

        let schemas = builder.load_schemas_v2(&PropertyBank::default()).unwrap();

        assert_eq!(schemas.len(), 3);
        // Verify graph saved to DB
    }

    #[test]
    fn second_run_all_fresh() {
        // Graph in DB, all schemas fresh
        // ... test implementation
    }

    #[test]
    fn property_change_only() {
        // One schema property changed
        // Should use Update construction
        // ... test implementation
    }

    #[test]
    fn extends_change() {
        // Schema inheritance changed
        // Should use Merge construction
        // ... test implementation
    }
}
```

---

## Implementation Notes

### Key Patterns

1. **Branching Pattern**: Each stage returns an enum for type-safe transitions
2. **Batch Processing**: Group schemas by status, process in batches
3. **Graph Hydration**: Embed payloads in graph nodes for unified state
4. **Incremental Construction**: Use `ExtendsChangeKind` to optimize
5. **Zero-Copy where possible**: Reuse existing `Schema` objects when unchanged

### Missing Pieces

1. **FileSource trait**: Abstraction for reading files (FileReader)
2. **Schema name resolution**: Convert `SchemaName` to `SchemaId`
3. **Error types**: Specific errors for each stage
4. **DagBuilder integration**: Use existing builder from graph.rs
5. **RefExpander integration**: Use existing expander
6. **Repository methods**: Ensure all needed methods exist

### Performance Optimizations

1. **Parallel parsing**: Can parse multiple files concurrently
2. **Batch DB queries**: Single query for all views/schemas
3. **Graph caching**: Reuse graph structure across runs
4. **Property hash caching**: Store hashes in view for fast comparison

### Edge Cases to Handle

1. **Empty file list**: No schemas found
2. **Corrupted graph**: Graph exists but is invalid
3. **Deleted all schemas**: All IDs in graph, no files
4. **Circular inheritance**: Detect and error
5. **PropertyBank missing**: Handle gracefully

---

## Estimated Implementation Time

| Phase | Description | Time |
|-------|-------------|------|
| 3 | Discovery stage | 6h |
| 4 | TimeComparison + ContentComparison | 4h |
| 5 | FileParsed | 4h |
| 6 | InheritanceGraphed | 8h |
| 7 | PropertyAnalysis | 6h |
| 8 | Refresh | 3h |
| 9 | Construction | 10h |
| 10 | Completion | 2h |
| Integration | Builder + tests | 6h |
| **Total** | | **49h** |

---

## Next Steps

1. Review this outline with user
2. Implement phases sequentially
3. Test after each phase
4. Integrate when all complete
5. Cutover from old pipeline
