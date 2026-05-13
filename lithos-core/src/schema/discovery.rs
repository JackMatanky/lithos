//! Consolidated discovery logic for schema files.
//!
//! Provides the [`DiscoveryEngine`] which performs a single atomic scan of
//! both the filesystem and database, consolidating all data needed for
//! schema processing.

use std::collections::{HashMap, HashSet};

use crate::{
    config::paths::SchemaConfigSpec,
    fs::{DirScanInput, DirScanner, FileEntry, RelativePath},
    schema::{
        error::{SchemaIngestionError, SchemaLoaderError},
        identifier::SchemaId,
        inheritance::InheritanceGraph,
        repository::ReadRepository,
        views::{RawPropertyBankView, RawSchemaView},
    },
};

// ═════════════════════════════════════════════════════════════════════════════
//  Discovery Result Types
// ═════════════════════════════════════════════════════════════════════════════

/// Cached state for an existing schema file.
///
/// This type only exists for schemas that have been previously ingested.
#[derive(Debug, Clone)]
pub(crate) struct SchemaCachedState {
    /// Schema ID from previous ingestion.
    id: SchemaId,
    /// Cached view for staleness detection.
    view: RawSchemaView,
}

impl SchemaCachedState {
    /// Returns the schema ID.
    #[inline]
    pub(crate) fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the cached view.
    #[inline]
    pub(crate) fn view(&self) -> &RawSchemaView {
        &self.view
    }
}

/// Discovery data for a single schema file.
///
/// Combines filesystem metadata with optional cached state from the database.
#[derive(Debug, Clone)]
pub(crate) struct SchemaDiscovery {
    /// File entry from filesystem scan (path, filename, info).
    entry: FileEntry,
    /// Cached state from database (None for new files).
    cached: Option<SchemaCachedState>,
}

impl SchemaDiscovery {
    /// Returns the file entry.
    #[inline]
    pub(crate) fn entry(&self) -> &FileEntry {
        &self.entry
    }

    /// Returns the cached state, if any.
    #[inline]
    pub(crate) fn cached(&self) -> Option<&SchemaCachedState> {
        self.cached.as_ref()
    }
}

/// Discovery data for the property bank file.
///
/// Combines filesystem metadata with optional cached view from the database.
#[derive(Debug, Clone)]
pub(crate) struct PropertyBankDiscovery {
    /// File entry from filesystem scan (path, filename, info).
    entry: FileEntry,
    /// Cached view from database (None if never ingested).
    view: Option<RawPropertyBankView>,
}

impl PropertyBankDiscovery {
    /// Returns the file entry.
    #[inline]
    pub(crate) fn entry(&self) -> &FileEntry {
        &self.entry
    }

    /// Returns the cached view, if any.
    #[inline]
    pub(crate) fn view(&self) -> Option<&RawPropertyBankView> {
        self.view.as_ref()
    }
}

/// Result of atomic discovery combining filesystem scan and database state.
///
/// This type replaces the previous `DiscoveryOutcome` with a clearer structure
/// that separates schemas from property bank and makes new vs existing files
/// explicit via the `cached` field.
#[derive(Debug)]
pub(crate) struct DiscoveryResult {
    /// Discovered schema files (path → discovery data).
    schemas: HashMap<RelativePath, SchemaDiscovery>,
    /// Discovered property bank file (if present).
    property_bank: Option<PropertyBankDiscovery>,
    /// Inheritance graph from database (if exists).
    graph: Option<InheritanceGraph<()>>,
    /// Schema IDs that exist in DB but not on filesystem (deleted files).
    deleted_ids: Vec<SchemaId>,
}

impl DiscoveryResult {
    /// Returns the discovered schemas.
    #[inline]
    pub(crate) fn schemas(&self) -> &HashMap<RelativePath, SchemaDiscovery> {
        &self.schemas
    }

    /// Returns the discovered property bank, if any.
    #[inline]
    pub(crate) fn property_bank(&self) -> Option<&PropertyBankDiscovery> {
        self.property_bank.as_ref()
    }

    /// Returns the inheritance graph, if any.
    #[inline]
    pub(crate) fn graph(&self) -> Option<&InheritanceGraph<()>> {
        self.graph.as_ref()
    }

    /// Returns the deleted schema IDs.
    #[inline]
    pub(crate) fn deleted_ids(&self) -> &[SchemaId] {
        &self.deleted_ids
    }

    /// Returns `true` if any schema files were discovered.
    #[inline]
    #[must_use]
    pub(crate) fn has_schemas(&self) -> bool {
        !self.schemas.is_empty()
    }

    /// Returns `true` if this is a cold-start discovery (no cached data).
    #[must_use]
    #[expect(dead_code, reason = "may be useful for future optimization")]
    pub(crate) fn is_cold_start(&self) -> bool {
        self.graph.is_none()
            && self.schemas.values().all(|s| s.cached.is_none())
            && self.property_bank.as_ref().is_none_or(|pb| pb.view.is_none())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  Cached State Helper
// ═════════════════════════════════════════════════════════════════════════════

/// Cached state from database (result of batch query).
///
/// This is a temporary struct used to pass DB query results from
/// `query_cached_state()` to `build_result()`.
struct CachedState {
    graph: Option<InheritanceGraph<()>>,
    property_bank_view: Option<RawPropertyBankView>,
    schema_views: HashMap<RelativePath, RawSchemaView>,
    schema_ids: HashMap<RelativePath, SchemaId>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  Discovery Engine
// ═════════════════════════════════════════════════════════════════════════════

/// Orchestrates atomic discovery of schemas and property bank.
///
/// The engine consolidates fragmented I/O and database operations into a
/// single-pass pipeline, ensuring consistency and performance.
pub(crate) struct DiscoveryEngine;

impl DiscoveryEngine {
    /// Performs an atomic discovery run.
    ///
    /// This method orchestrates the discovery pipeline:
    /// 1. Scan filesystem for schema files
    /// 2. Separate property bank from schemas
    /// 3. Query DB for all cached state (single transaction)
    /// 4. Combine filesystem + DB data into result
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if I/O or repository operations fail.
    pub(crate) fn run<R>(
        spec: &SchemaConfigSpec,
        repo: &R,
        vault_root: &std::path::Path,
    ) -> Result<DiscoveryResult, SchemaLoaderError>
    where
        R: ReadRepository,
    {
        // Step 1: Scan filesystem
        let entries = Self::scan_filesystem(spec, vault_root)?;

        // Step 2: Separate property bank from schemas (O(n) single pass)
        let (property_bank_entry, schema_entries) =
            Self::separate_property_bank(entries, spec.property_bank());

        // Step 3: Query DB for all cached state (single transaction)
        let cached_state = Self::query_cached_state(
            repo,
            property_bank_entry.as_ref(),
            &schema_entries,
            spec.property_bank(),
        )?;

        // Step 4: Combine filesystem + DB state into result
        Ok(Self::build_result(
            property_bank_entry,
            schema_entries,
            cached_state,
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Filesystem Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Scans filesystem for schema files.
    ///
    /// Returns `Vec<FileEntry>` for efficient processing (no `HashMap`
    /// overhead).
    ///
    /// # Errors
    ///
    /// Returns error if filesystem scanning fails.
    fn scan_filesystem(
        spec: &SchemaConfigSpec,
        vault_root: &std::path::Path,
    ) -> Result<Vec<FileEntry>, SchemaLoaderError> {
        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        let schema_dir = spec.directory();
        let pattern = format!("{}/**/*", schema_dir.as_path().display());

        DirScanner::new(vault_root)
            .entries(
                DirScanInput::new()
                    .with_pattern(&pattern)
                    .with_extensions(&SCHEMA_EXTENSIONS),
            )
            .map_err(|e| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::Io {
                        path: schema_dir.as_path().to_path_buf(),
                        source: std::io::Error::other(e),
                    },
                ))
            })
    }

    /// Separates property bank from schema files (O(n) single pass).
    ///
    /// Returns owned `FileEntry` values for efficient processing.
    #[expect(
        clippy::type_complexity,
        reason = "Functional return type for discovery separation"
    )]
    fn separate_property_bank(
        entries: Vec<FileEntry>,
        property_bank_path: &RelativePath,
    ) -> (Option<FileEntry>, Vec<(RelativePath, FileEntry)>) {
        let mut property_bank = None;
        let mut schemas = Vec::with_capacity(entries.len());

        for entry in entries {
            let Ok(path) = RelativePath::try_from(entry.path.clone()) else {
                continue;
            };

            if path == *property_bank_path {
                property_bank = Some(entry);
            } else {
                schemas.push((path, entry));
            }
        }

        if schemas.is_empty() && property_bank.is_none() {
            tracing::info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }

        (property_bank, schemas)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Database Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Queries cached state from repository read seam in one coherent pass.
    ///
    /// Note: each repository method controls its own storage transaction
    /// boundary. Discovery maintains a single orchestration pass over those
    /// read methods and skips schema batch lookups when no schema files exist.
    ///
    /// # Errors
    ///
    /// Returns error if database queries fail.
    fn query_cached_state<R>(
        repo: &R,
        property_bank_entry: Option<&FileEntry>,
        schema_entries: &[(RelativePath, FileEntry)],
        property_bank_path: &RelativePath,
    ) -> Result<CachedState, SchemaLoaderError>
    where
        R: ReadRepository,
    {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "iter over &[(RelativePath, FileEntry)] yields \
                      &&(RelativePath, FileEntry)"
        )]
        let schema_paths: Vec<_> =
            schema_entries.iter().map(|(path, _)| path).cloned().collect();

        let graph = repo
            .get_topological_graph()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let property_bank_view = match property_bank_entry {
            Some(_) => repo
                .get_raw_property_bank_view(property_bank_path)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?,
            None => None,
        };

        let (schema_views, schema_ids) = if schema_paths.is_empty() {
            (HashMap::new(), HashMap::new())
        } else {
            let schema_views = repo
                .find_raw_schema_views_by_paths(&schema_paths)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            let schema_views = schema_paths
                .iter()
                .cloned()
                .zip(schema_views)
                .filter_map(|(path, view)| view.map(|v| (path, v)))
                .collect();

            let schema_ids = repo
                .find_schema_ids_by_paths(&schema_paths)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            let schema_ids = schema_paths
                .iter()
                .cloned()
                .zip(schema_ids)
                .filter_map(|(path, id)| id.map(|v| (path, v)))
                .collect();

            (schema_views, schema_ids)
        };

        Ok(CachedState {
            graph,
            property_bank_view,
            schema_views,
            schema_ids,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Result Construction
    // ─────────────────────────────────────────────────────────────────────────

    /// Builds final `DiscoveryResult` from filesystem + DB data.
    ///
    /// Performance: Single pass over `schema_entries`; no intermediate
    /// allocations.
    fn build_result(
        property_bank_entry: Option<FileEntry>,
        schema_entries: Vec<(RelativePath, FileEntry)>,
        mut cached: CachedState,
    ) -> DiscoveryResult {
        // Build property bank discovery
        let property_bank =
            property_bank_entry.map(|entry| PropertyBankDiscovery {
                entry,
                view: cached.property_bank_view,
            });

        // Build schema discoveries with cached state lookup
        let mut schemas = HashMap::with_capacity(schema_entries.len());
        let mut filesystem_ids = HashSet::with_capacity(schema_entries.len());

        for (path, entry) in schema_entries {
            let cached_state = if let Some(view) =
                cached.schema_views.remove(&path)
            {
                let id = cached.schema_ids.remove(&path).unwrap_or_default();
                filesystem_ids.insert(id);
                Some(SchemaCachedState {
                    id,
                    view,
                })
            } else {
                None
            };

            schemas.insert(path, SchemaDiscovery {
                entry,
                cached: cached_state,
            });
        }

        // Detect deleted schemas
        let deleted_ids =
            cached.graph.as_ref().map_or_else(Vec::new, |graph| {
                Self::detect_deleted_schemas(graph, &filesystem_ids)
            });

        DiscoveryResult {
            schemas,
            property_bank,
            graph: cached.graph,
            deleted_ids,
        }
    }

    /// Detects schemas deleted from filesystem but still in DB.
    ///
    /// Performance: O(n) where n = graph size.
    fn detect_deleted_schemas(
        graph: &InheritanceGraph<()>,
        filesystem_ids: &HashSet<SchemaId>,
    ) -> Vec<SchemaId> {
        graph
            .topo_order()
            .iter()
            .filter(|id| !filesystem_ids.contains(id))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::schema::{
        aggregate::Schema,
        bank::PropertyBank,
        identifier::{SchemaId, SchemaName},
        index::{NameIdPairs, PathIdPairs, SchemaIndex},
        inheritance::InheritanceGraph,
        property::PropertyName,
        repository::ReadRepository,
        storage::testing::InMemoryRepository,
        views::{RawPropertyBankView, RawSchemaView},
    };

    struct CountingReadRepo {
        inner: InMemoryRepository,
        raw_views_by_paths_calls: AtomicUsize,
        ids_by_paths_calls: AtomicUsize,
    }

    impl CountingReadRepo {
        fn new() -> Self {
            Self {
                inner: InMemoryRepository::new(),
                raw_views_by_paths_calls: AtomicUsize::new(0),
                ids_by_paths_calls: AtomicUsize::new(0),
            }
        }
    }

    impl ReadRepository for CountingReadRepo {
        fn find_schema_by_id(
            &self,
            id: SchemaId,
        ) -> Result<Option<Schema>, crate::schema::error::SchemaStorageError>
        {
            self.inner.find_schema_by_id(id)
        }

        fn find_many_schemas_by_id(
            &self,
            ids: &[SchemaId],
        ) -> Result<Vec<Option<Schema>>, crate::schema::error::SchemaStorageError>
        {
            self.inner.find_many_schemas_by_id(ids)
        }

        fn find_schemas_by_ids(
            &self,
            ids: &[SchemaId],
        ) -> Result<Vec<Schema>, crate::schema::error::SchemaStorageError>
        {
            self.inner.find_schemas_by_ids(ids)
        }

        fn list_schemas(
            &self,
        ) -> Result<Vec<Schema>, crate::schema::error::SchemaStorageError>
        {
            self.inner.list_schemas()
        }

        fn find_schemas_using_properties(
            &self,
            property_names: &[PropertyName],
        ) -> Result<
            HashMap<SchemaId, Vec<PropertyName>>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.find_schemas_using_properties(property_names)
        }

        fn get_raw_schema_view(
            &self,
            id: SchemaId,
        ) -> Result<
            Option<RawSchemaView>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.get_raw_schema_view(id)
        }

        fn find_raw_schema_view_by_path(
            &self,
            path: &RelativePath,
        ) -> Result<
            Option<RawSchemaView>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.find_raw_schema_view_by_path(path)
        }

        fn find_raw_schema_views_by_paths(
            &self,
            paths: &[RelativePath],
        ) -> Result<
            Vec<Option<RawSchemaView>>,
            crate::schema::error::SchemaStorageError,
        > {
            self.raw_views_by_paths_calls.fetch_add(1, Ordering::Relaxed);
            self.inner.find_raw_schema_views_by_paths(paths)
        }

        fn get_property_bank(
            &self,
        ) -> Result<
            Option<PropertyBank>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.get_property_bank()
        }

        fn get_topological_graph(
            &self,
        ) -> Result<
            Option<InheritanceGraph<()>>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.get_topological_graph()
        }

        fn get_raw_property_bank_view(
            &self,
            path: &RelativePath,
        ) -> Result<
            Option<RawPropertyBankView>,
            crate::schema::error::SchemaStorageError,
        > {
            self.inner.get_raw_property_bank_view(path)
        }

        fn find_schema_id_by_name(
            &self,
            name: &SchemaName,
        ) -> Result<Option<SchemaId>, crate::schema::error::SchemaStorageError>
        {
            self.inner.find_schema_id_by_name(name)
        }

        fn find_schema_id_by_path(
            &self,
            path: &RelativePath,
        ) -> Result<Option<SchemaId>, crate::schema::error::SchemaStorageError>
        {
            self.inner.find_schema_id_by_path(path)
        }

        fn find_schema_ids_by_paths(
            &self,
            paths: &[RelativePath],
        ) -> Result<
            Vec<Option<SchemaId>>,
            crate::schema::error::SchemaStorageError,
        > {
            self.ids_by_paths_calls.fetch_add(1, Ordering::Relaxed);
            self.inner.find_schema_ids_by_paths(paths)
        }

        fn list_schema_name_id_pairs(
            &self,
        ) -> Result<NameIdPairs, crate::schema::error::SchemaStorageError>
        {
            self.inner.list_schema_name_id_pairs()
        }

        fn list_schema_path_id_pairs(
            &self,
        ) -> Result<PathIdPairs, crate::schema::error::SchemaStorageError>
        {
            self.inner.list_schema_path_id_pairs()
        }

        fn get_schema_index(
            &self,
        ) -> Result<SchemaIndex, crate::schema::error::SchemaStorageError>
        {
            self.inner.get_schema_index()
        }
    }

    fn accepts_read_repository_only<R: ReadRepository>(
        spec: &SchemaConfigSpec,
        repo: &R,
        root: &std::path::Path,
    ) -> Result<DiscoveryResult, SchemaLoaderError> {
        DiscoveryEngine::run(spec, repo, root)
    }

    #[test]
    fn run_finds_all_files() {
        let root = tempfile::tempdir().unwrap();
        let schema_dir = root.path().join("schemas");
        std::fs::create_dir_all(&schema_dir).unwrap();

        let bank_path = schema_dir.join("property_bank.json");
        std::fs::write(&bank_path, "{}").unwrap();

        let schema1_path = schema_dir.join("schema1.json");
        std::fs::write(&schema1_path, "{}").unwrap();

        let spec = SchemaConfigSpec::new(
            RelativePath::try_from(PathBuf::from("schemas")).unwrap(),
            RelativePath::try_from(PathBuf::from("schemas/property_bank.json"))
                .unwrap(),
        );

        let repo = InMemoryRepository::new();
        let result =
            accepts_read_repository_only(&spec, &repo, root.path()).unwrap();

        assert_eq!(result.schemas.len(), 1);
        assert!(result.property_bank.is_some());
        assert!(result.has_schemas());
    }

    #[test]
    fn run_skips_schema_batch_lookups_when_no_schema_files_exist() {
        let root = tempfile::tempdir().unwrap();
        let schema_dir = root.path().join("schemas");
        std::fs::create_dir_all(&schema_dir).unwrap();

        let bank_path = schema_dir.join("property_bank.json");
        std::fs::write(&bank_path, "{}").unwrap();

        let spec = SchemaConfigSpec::new(
            RelativePath::try_from(PathBuf::from("schemas")).unwrap(),
            RelativePath::try_from(PathBuf::from("schemas/property_bank.json"))
                .unwrap(),
        );

        let repo = CountingReadRepo::new();
        let result = DiscoveryEngine::run(&spec, &repo, root.path()).unwrap();

        assert!(!result.has_schemas());
        assert_eq!(repo.raw_views_by_paths_calls.load(Ordering::Relaxed), 0);
        assert_eq!(repo.ids_by_paths_calls.load(Ordering::Relaxed), 0);
    }
}
