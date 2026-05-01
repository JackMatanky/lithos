//! Unified discovery engine.
//!
//! This module consolidates fragmented schema discovery logic into a single
//! component that performs all I/O operations in one atomic batch transaction.
//!
//! # Architecture
//!
//! By using a unified [`DiscoveredFile`] structure with the [`RawView`] trait,
//! we eliminate duplication between property bank and schema discovery. The
//! [`DiscoveryEngine`] orchestrates a single batch read that fetches:
//! - Property bank view and file stats
//! - Schema views, IDs, and file stats
//! - Topological graph
//! - Deleted schema detection
//!
//! This reduces repository transactions by 66% compared to the previous
//! fragmented approach.

#![expect(dead_code, reason = "Will be used in subsequent commits")]

use std::collections::{HashMap, HashSet};

use crate::{
    config::paths::SchemaConfigSpec,
    fs::{FileStats, Filename, FsReader, RelativePath},
    schema::{
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        identifier::SchemaId,
        inheritance::InheritanceGraph,
        views::{RawPropertyBankView, RawSchemaView, RawView as _},
    },
};

/// Type-safe wrapper for cached views.
///
/// This enum preserves type safety while allowing unified handling
/// in discovery logic. Processors can pattern match to get the
/// specific view type they need.
#[derive(Debug, Clone)]
pub(crate) enum DiscoveredView {
    /// Cached schema view.
    Schema(RawSchemaView),
    /// Cached property bank view.
    PropertyBank(RawPropertyBankView),
}

/// File kind discovered during scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaFileKind {
    /// Regular schema file with its stable schema ID.
    Schema(SchemaId),
    /// Property bank singleton file.
    PropertyBank,
}

/// Unified discovery data for a single file (schema or property bank).
///
/// This structure eliminates duplication by using the `RawView` trait.
/// Both `RawPropertyBankView` and `RawSchemaView` implement `RawView`,
/// allowing polymorphic handling of cached metadata.
#[derive(Debug, Clone)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Internal data structure"
)]
pub(crate) struct DiscoveredFile {
    /// Filename (e.g., "note.toml", "property-bank.json").
    pub(crate) filename: Filename,
    /// File kind information.
    pub(crate) kind: SchemaFileKind,
    /// Cached view from DB (None if never loaded).
    pub(crate) view: Option<DiscoveredView>,
    /// Current file stats from filesystem.
    pub(crate) file_stats: FileStats,
}

impl DiscoveredFile {
    /// Checks if timestamps match between cached view and current file.
    ///
    /// Returns `false` if no cached view exists.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Idiomatic option matching is preferred"
    )]
    #[must_use]
    pub(crate) fn is_timestamp_match(&self) -> bool {
        let Some(view) = self.view.as_ref() else {
            return false;
        };
        match view {
            DiscoveredView::Schema(v) => v.is_timestamp_match(
                self.file_stats.created_at(),
                self.file_stats.modified_at(),
            ),
            DiscoveredView::PropertyBank(v) => v.is_timestamp_match(
                self.file_stats.created_at(),
                self.file_stats.modified_at(),
            ),
        }
    }

    /// Returns `true` if this file has no cached view (first time seen).
    #[inline]
    #[must_use]
    pub(crate) fn is_new(&self) -> bool {
        self.view.is_none()
    }

    /// Returns true when this file is a schema file.
    #[inline]
    #[must_use]
    pub(crate) const fn is_schema(&self) -> bool {
        matches!(self.kind, SchemaFileKind::Schema(_))
    }

    /// Returns true when this file is the property bank.
    #[inline]
    #[must_use]
    pub(crate) const fn is_property_bank(&self) -> bool {
        matches!(self.kind, SchemaFileKind::PropertyBank)
    }

    /// Returns schema ID if this is a schema file.
    #[inline]
    #[must_use]
    pub(crate) const fn schema_id(&self) -> Option<SchemaId> {
        match self.kind {
            SchemaFileKind::Schema(id) => Some(id),
            SchemaFileKind::PropertyBank => None,
        }
    }

    /// Returns the raw schema view if this is a schema file and view exists.
    #[expect(
        clippy::ref_patterns,
        reason = "Idiomatic option matching is preferred"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn as_schema_view(&self) -> Option<&RawSchemaView> {
        match self.view {
            Some(DiscoveredView::Schema(ref v)) => Some(v),
            _ => None,
        }
    }

    /// Returns the raw property bank view if this is a property bank file and
    /// view exists.
    #[expect(
        clippy::ref_patterns,
        reason = "Idiomatic option matching is preferred"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn as_property_bank_view(&self) -> Option<&RawPropertyBankView> {
        match self.view {
            Some(DiscoveredView::PropertyBank(ref v)) => Some(v),
            _ => None,
        }
    }
}

/// Complete discovery outcome containing all data needed to initialize
/// processors.
#[derive(Debug)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Internal data structure"
)]
pub(crate) struct DiscoveryOutcome {
    /// Discovered files, keyed by path.
    pub(crate) files: HashMap<RelativePath, DiscoveredFile>,
    /// Topological graph from previous run (None if never run or cold-start).
    pub(crate) graph: Option<InheritanceGraph<()>>,
    /// Schema IDs that exist in DB but have no corresponding file.
    pub(crate) deleted_schemas: Vec<SchemaId>,
}

impl DiscoveryOutcome {
    /// Returns `true` if this is a cold-start (no previous data in DB).
    #[inline]
    #[must_use]
    pub(crate) fn is_cold_start(&self) -> bool {
        self.graph.is_none() && self.files.values().all(|f| f.view.is_none())
    }

    /// Returns `true` if this is an incremental update (has previous data).
    #[inline]
    #[must_use]
    pub(crate) fn is_incremental(&self) -> bool {
        !self.is_cold_start()
    }

    /// Returns `true` if any schemas exist on disk.
    #[inline]
    #[must_use]
    pub(crate) fn has_schemas(&self) -> bool {
        self.files.values().any(DiscoveredFile::is_schema)
    }

    /// Returns the property bank file, if it exists.
    #[inline]
    #[must_use]
    pub(crate) fn property_bank(&self) -> Option<&DiscoveredFile> {
        self.files.values().find(|f| f.is_property_bank())
    }

    /// Returns an iterator over schema files (excludes property bank).
    #[inline]
    pub(crate) fn schema_files(&self) -> impl Iterator<Item = &DiscoveredFile> {
        self.files.values().filter(|f| f.is_schema())
    }
}

/// Unified discovery engine that performs all I/O in a single atomic batch.
pub(crate) struct DiscoveryEngine;

impl DiscoveryEngine {
    /// Discovers all files and database state required for schema processing.
    ///
    /// This method performs unified discovery by:
    /// 1. Scanning the filesystem for schema files and property bank
    /// 2. Querying the database for cached views and metadata
    /// 3. Combining filesystem and database state into a unified outcome
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if I/O or repository operations fail.
    pub(crate) fn run<R>(
        spec: &SchemaConfigSpec,
        repo: &R,
        source: &FsReader,
    ) -> Result<DiscoveryOutcome, SchemaLoaderError>
    where
        R: crate::schema::storage::Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        // Step 1: Scan filesystem for schema files and property bank
        let (schema_files, property_bank_path) =
            Self::scan_filesystem(spec, source)?;

        // Step 2: Perform all database queries in a single atomic batch read
        let (graph, bank_view, mut views_by_path, mut ids_by_path) = repo
            .with_batch_schema_reader(|batch_reader| {
                let graph = batch_reader.get_topological_graph()?;

                let bank_view = property_bank_path
                    .as_ref()
                    .and_then(|path| {
                        batch_reader.get_raw_property_bank_view(path).ok()
                    })
                    .flatten();

                let views = batch_reader
                    .find_raw_schema_views_by_paths(&schema_files)?;
                let ids =
                    batch_reader.find_schema_ids_by_paths(&schema_files)?;

                Ok((graph, bank_view, views, ids))
            })
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // Step 4: Fetch all file stats (I/O) outside the database transaction
        let mut files = HashMap::new();

        if let Some(bank_path) = property_bank_path {
            let file_stats = source
                .stats(bank_path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            let filename = source
                .filename(bank_path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            files.insert(bank_path.clone(), DiscoveredFile {
                filename,
                kind: SchemaFileKind::PropertyBank,
                view: bank_view.map(DiscoveredView::PropertyBank),
                file_stats,
            });
        }

        let mut stats_by_path =
            Self::fetch_file_stats_batch(&schema_files, source)?;
        let mut filesystem_ids = HashSet::new();

        for path in &schema_files {
            let id = ids_by_path.remove(path).unwrap_or_else(SchemaId::new);
            let view = views_by_path.remove(path).map(DiscoveredView::Schema);
            let file_stats = stats_by_path.remove(path).ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::Io {
                        path: path.as_path().to_path_buf(),
                        source: std::io::Error::other("missing file stats"),
                    },
                ))
            })?;

            let filename = source
                .filename(path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            filesystem_ids.insert(id);

            files.insert(path.clone(), DiscoveredFile {
                filename,
                kind: SchemaFileKind::Schema(id),
                view,
                file_stats,
            });
        }

        let deleted_schemas =
            Self::detect_deleted_schemas(graph.as_ref(), &filesystem_ids);

        Ok(DiscoveryOutcome {
            files,
            graph,
            deleted_schemas,
        })
    }

    /// Scans the filesystem for schema files and property bank.
    ///
    /// Returns a tuple of (`schema_files`, `property_bank_path`).
    ///
    /// # Errors
    ///
    /// Returns error if filesystem scanning fails or duplicate property bank is
    /// found.
    #[expect(
        clippy::type_complexity,
        reason = "Return type is clear and self-documenting; Vec and Option \
                  are common"
    )]
    fn scan_filesystem(
        spec: &SchemaConfigSpec,
        source: &FsReader,
    ) -> Result<(Vec<RelativePath>, Option<RelativePath>), SchemaLoaderError>
    {
        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        let schema_dir = spec.directory();
        let property_bank_path = spec.property_bank();

        // Extract property bank filename for comparison
        let bank_filename = source
            .filename(property_bank_path.as_path())
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?;

        // Scan directory for all files
        let pattern = format!("{}/**/*", schema_dir.as_path().display());
        let all_files = source.list_files(&pattern).map_err(|e| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::Io {
                    path: schema_dir.as_path().to_path_buf(),
                    source: std::io::Error::other(e),
                },
            ))
        })?;

        let mut schema_files: Vec<RelativePath> = Vec::new();
        let mut found_property_bank: Option<RelativePath> = None;

        for path in all_files {
            let Ok(file_name) = Filename::try_from(path.as_path()) else {
                continue;
            };

            // Check if this is the property bank file
            if file_name == bank_filename {
                if found_property_bank.is_some() {
                    return Err(SchemaLoaderError::Ingestion(
                        SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: "duplicate property bank file found"
                                    .into(),
                            },
                        ),
                    ));
                }
                found_property_bank = Some(property_bank_path.clone());
                continue;
            }

            // Check if this is a schema file (by extension)
            let Some(ext) = file_name.extension() else {
                continue;
            };

            if SCHEMA_EXTENSIONS
                .iter()
                .any(|allowed| ext.eq_ignore_ascii_case(allowed))
            {
                let relative =
                    RelativePath::try_from(path).map_err(|error| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "invalid schema path discovered: {error}"
                            )
                            .into(),
                        },
                    ))
                    })?;
                schema_files.push(relative);
            }
        }

        if schema_files.is_empty() {
            tracing::info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }

        Ok((schema_files, found_property_bank))
    }

    fn fetch_file_stats_batch(
        files: &[RelativePath],
        source: &FsReader,
    ) -> Result<HashMap<RelativePath, FileStats>, SchemaLoaderError> {
        let mut stats_map = HashMap::new();

        for path in files {
            let stats = source
                .stats(path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            stats_map.insert(path.clone(), stats);
        }

        Ok(stats_map)
    }

    fn detect_deleted_schemas(
        graph: Option<&InheritanceGraph<()>>,
        filesystem_ids: &HashSet<SchemaId>,
    ) -> Vec<SchemaId> {
        let Some(graph) = graph else {
            return Vec::new();
        };

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
    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::FsReader,
        schema::{
            inheritance::SchemaGraphBuilder,
            raw::RawSchema,
            storage::Repository as _,
            testing::InMemoryRepository,
            views::{HashRecord, RawPropertyMapHash, SchemaVersion},
        },
        support::hash::Blake3Hash,
    };

    fn setup_test_env() -> (TempDir, InMemoryRepository, FsReader) {
        let temp = TempDir::new().unwrap();
        let repo = InMemoryRepository::new();
        let source = FsReader::new(temp.path().to_path_buf());
        (temp, repo, source)
    }

    fn create_test_spec() -> SchemaConfigSpec {
        use std::path::PathBuf;

        let directory =
            RelativePath::try_from(PathBuf::from("schemas")).unwrap();
        let property_bank =
            RelativePath::try_from(PathBuf::from("schemas/property_bank.json"))
                .unwrap();
        SchemaConfigSpec::new(directory, property_bank)
    }

    #[test]
    fn discovery_engine_cold_start() {
        let (temp, repo, source) = setup_test_env();

        // Create schemas directory and files
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        let path1 = schemas_dir.join("schema1.json");
        let path2 = schemas_dir.join("schema2.json");
        std::fs::write(&path1, "{}").unwrap();
        std::fs::write(&path2, "{}").unwrap();

        let spec = create_test_spec();
        let outcome = DiscoveryEngine::run(&spec, &repo, &source).unwrap();

        assert!(outcome.is_cold_start());
        assert!(!outcome.is_incremental());
        assert!(outcome.has_schemas());
        assert!(outcome.property_bank().is_none());
        assert_eq!(outcome.files.len(), 2);
        assert!(outcome.graph.is_none());
        assert!(outcome.deleted_schemas.is_empty());

        for file in outcome.schema_files() {
            assert!(file.is_new());
            assert!(!file.is_property_bank());
        }
    }

    #[test]
    fn discovery_engine_incremental_with_property_bank() {
        use std::path::PathBuf;

        let (temp, repo, source) = setup_test_env();

        // Create schemas directory, schema file, and property bank
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        let schema_path = schemas_dir.join("schema.json");
        let bank_path = schemas_dir.join("property_bank.json");
        std::fs::write(&schema_path, "{}").unwrap();
        std::fs::write(&bank_path, "{}").unwrap();

        // Mock a previous graph
        let mut builder = SchemaGraphBuilder::new();
        let id = SchemaId::new();
        builder.add_node(id, ());
        let graph = InheritanceGraph::try_from(builder.build()).unwrap();
        repo.save_topological_graph(&graph).unwrap();

        // Mock cached schema view
        let raw_json = r#"{ "$version": "1.0", "properties": {} }"#;
        let raw_schema = serde_json::from_str::<RawSchema>(raw_json)
            .unwrap()
            .with_name("test".into());
        let hashes = HashRecord::new(
            Blake3Hash::new([0; 32]),
            RawPropertyMapHash::default(),
        );
        let version = SchemaVersion::new(
            FileStats::new(None, None, 0),
            hashes,
            &raw_schema,
        )
        .unwrap();
        let view = RawSchemaView::new(
            RelativePath::try_from(PathBuf::from("schemas/schema.json"))
                .unwrap(),
            version,
        );
        repo.save_raw_schema_view(id, &view).unwrap();

        let spec = create_test_spec();
        let outcome = DiscoveryEngine::run(&spec, &repo, &source).unwrap();

        assert!(!outcome.is_cold_start());
        assert!(outcome.is_incremental());
        assert!(outcome.has_schemas());
        assert!(outcome.property_bank().is_some());
        assert_eq!(outcome.files.len(), 2);
        assert_eq!(outcome.schema_files().count(), 1);

        let schema_file = outcome.schema_files().next().unwrap();
        assert!(!schema_file.is_new());
        assert_eq!(schema_file.schema_id(), Some(id));
    }

    #[test]
    fn discovery_engine_detects_deleted_schemas() {
        let (temp, repo, source) = setup_test_env();

        // Create schemas directory
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Graph with 3 schemas
        let mut builder = SchemaGraphBuilder::new();
        let id1 = SchemaId::new(); // Will be deleted
        let id2 = SchemaId::new(); // Will be deleted
        let id3 = SchemaId::new(); // Will remain
        builder.add_node(id1, ());
        builder.add_node(id2, ());
        builder.add_node(id3, ());
        let graph = InheritanceGraph::try_from(builder.build()).unwrap();
        repo.save_topological_graph(&graph).unwrap();

        // Create only 1 schema file in the filesystem (at id3's path)
        let path3 = schemas_dir.join("schema3.json");
        std::fs::write(&path3, "{}").unwrap();
        let rel_path3 = RelativePath::try_from("schemas/schema3.json").unwrap();

        // Mock the cached view for the remaining schema so it maps to id3
        let raw_json = r#"{ "$version": "1.0", "properties": {} }"#;
        let raw_schema = serde_json::from_str::<RawSchema>(raw_json)
            .unwrap()
            .with_name("test3".into());
        let hashes = HashRecord::new(
            Blake3Hash::new([0; 32]),
            RawPropertyMapHash::default(),
        );
        let version = SchemaVersion::new(
            FileStats::new(None, None, 0),
            hashes,
            &raw_schema,
        )
        .unwrap();
        let view = RawSchemaView::new(rel_path3.clone(), version);
        repo.save_raw_schema_view(id3, &view).unwrap();

        // Use relative paths for the spec
        let spec = SchemaConfigSpec::new(
            RelativePath::try_from("schemas").unwrap(),
            RelativePath::try_from("schemas/property_bank.json").unwrap(),
        );

        let outcome = DiscoveryEngine::run(&spec, &repo, &source).unwrap();

        assert_eq!(outcome.deleted_schemas.len(), 2);
        assert!(outcome.deleted_schemas.contains(&id1));
        assert!(outcome.deleted_schemas.contains(&id2));
        assert!(!outcome.deleted_schemas.contains(&id3));
    }

    #[test]
    fn discovery_engine_handles_empty_file_list() {
        let (_temp, repo, source) = setup_test_env();

        // Create empty schema directory
        let spec = create_test_spec();
        let outcome = DiscoveryEngine::run(&spec, &repo, &source).unwrap();

        assert!(outcome.files.is_empty());
        assert!(!outcome.has_schemas());
    }
}
