//! Consolidated discovery logic for schema files.
//!
//! Provides the [`DiscoveryEngine`] which performs a single atomic scan of
//! both the filesystem and database, consolidating all data needed for
//! schema processing.

use std::collections::{HashMap, HashSet};

use crate::{
    fs::{DirScanInput, DirScanner, FileEntry, FileInfo, RelativePath},
    prelude::W,
    schema::{
        error::{
            SchemaFileError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError,
        },
        identifier::SchemaId,
        inheritance::InheritanceGraph,
        storage::BatchSchemaReader,
        views::contracts::RawViewRead,
    },
};

/// Discovered files from filesystem scan.
///
/// Newtype wrapper around `HashMap<RelativePath, FileEntry>` for type safety
/// and to provide domain-specific query methods.
#[derive(Debug)]
struct FileDiscovery(HashMap<RelativePath, FileEntry>);

impl FileDiscovery {
    /// Returns an iterator over all entries.
    fn iter(&self) -> impl Iterator<Item = (&RelativePath, &FileEntry)> {
        self.0.iter()
    }

    /// Extracts the property bank entry if present.
    fn extract_property_bank(
        &mut self,
        expected_path: &RelativePath,
    ) -> Option<FileEntry> {
        self.0.remove(expected_path)
    }

    /// Returns the number of remaining files (after property bank extraction).
    fn len(&self) -> usize {
        self.0.len()
    }

    /// Consumes self and returns the inner HashMap.
    fn into_inner(self) -> HashMap<RelativePath, FileEntry> {
        self.0
    }
}

/// File kind for discovery results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaFileKind {
    /// Regular schema file with its stable schema ID.
    Schema(SchemaId),
    /// Property bank singleton file.
    PropertyBank,
}

/// A cached view of a schema file or property bank.
///
/// This enum allows polymorphic handling of cached metadata from the database
/// during discovery.
#[derive(Debug, Clone)]
pub(crate) enum DiscoveredView {
    /// Cached view of a schema file.
    Schema(crate::schema::views::raw::RawSchemaView),
    /// Cached view of a property bank file.
    PropertyBank(crate::schema::views::raw::RawPropertyBankView),
}

impl DiscoveredView {
    /// Checks if timestamps match between this cached view and provided times.
    #[must_use]
    pub(crate) fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        match self {
            Self::Schema(v) => v.is_timestamp_match(created_at, modified_at),
            Self::PropertyBank(v) => {
                v.is_timestamp_match(created_at, modified_at)
            }
        }
    }
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
    /// File kind information.
    pub(crate) kind: SchemaFileKind,
    /// Cached view from DB (None if never loaded).
    pub(crate) view: Option<DiscoveredView>,
    /// Current file information from filesystem.
    pub(crate) info: FileInfo,
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
        view.is_timestamp_match(self.info.created_at(), self.info.modified_at())
    }

    /// Returns `true` if this file has no cached view (first time seen).
    #[inline]
    #[must_use]
    pub(crate) fn is_new(&self) -> bool {
        self.view.is_none()
    }
}

/// Configuration specification for schema discovery.
pub use crate::config::paths::SchemaConfigSpec;

/// Unified discovery outcome containing all metadata from filesystem and DB.
#[derive(Debug)]
pub(crate) struct DiscoveryOutcome {
    /// Map of all discovered files (schemas and property bank).
    pub(crate) files: HashMap<RelativePath, DiscoveredFile>,
    /// The current inheritance graph from the database.
    pub(crate) graph: Option<InheritanceGraph<()>>,
    /// List of schema IDs that exist in the database but not on filesystem.
    pub(crate) deleted_schemas: Vec<SchemaId>,
}

impl DiscoveryOutcome {
    /// Returns an iterator over all discovered schema files.
    pub(crate) fn schema_files(
        &self,
    ) -> impl Iterator<Item = (&RelativePath, &DiscoveredFile)> {
        self.files
            .iter()
            .filter(|(_, f)| matches!(f.kind, SchemaFileKind::Schema(_)))
    }

    /// Returns the discovered property bank file, if any.
    pub(crate) fn property_bank(
        &self,
    ) -> Option<(&RelativePath, &DiscoveredFile)> {
        self.files
            .iter()
            .find(|(_, f)| matches!(f.kind, SchemaFileKind::PropertyBank))
    }

    /// Returns `true` if any schema files were discovered.
    pub(crate) fn has_schemas(&self) -> bool {
        self.schema_files().next().is_some()
    }

    /// Returns `true` if this is a cold-start discovery (no cached views).
    pub(crate) fn is_cold_start(&self) -> bool {
        self.graph.is_none() || self.files.values().all(|f| f.view.is_none())
    }
}

/// Orchestrates atomic discovery of schemas and property bank.
///
/// The engine consolidates fragmented I/O and database operations into a
/// single-pass pipeline, ensuring consistency and performance.
pub(crate) struct DiscoveryEngine;

impl DiscoveryEngine {
    /// Performs an atomic discovery run.
    ///
    /// This method executes the following pipeline:
    /// 1. Scanning filesystem for schemas and property bank
    /// 2. Fetching all matching cached views from database in one transaction
    /// 3. Combining filesystem and database state into a unified outcome
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if I/O or repository operations fail.
    pub(crate) fn run<R>(
        spec: &SchemaConfigSpec,
        repo: &R,
        vault_root: &std::path::Path,
    ) -> Result<DiscoveryOutcome, SchemaLoaderError>
    where
        R: crate::schema::storage::Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        // Step 1: Scan filesystem for all schema files and property bank
        let mut discovered = Self::scan_filesystem(spec, vault_root)?;

        // Step 2: Extract property bank with O(1) lookup
        let property_bank_path = spec.property_bank();
        let property_bank_entry =
            discovered.extract_property_bank(property_bank_path);

        // Step 3: Extract paths for database queries (only schemas remain)
        let schema_paths: Vec<RelativePath> =
            discovered.iter().map(|(path, _)| path.clone()).collect();

        // Step 4: Perform all database queries in a single atomic batch read
        let (graph, bank_view, mut views_by_path, mut ids_by_path) = repo
            .with_batch_schema_reader(|batch_reader| {
                let graph = batch_reader.get_topological_graph()?;

                let bank_view = property_bank_entry.as_ref().and_then(|_| {
                    batch_reader
                        .get_raw_property_bank_view(property_bank_path)
                        .ok()
                });

                let views = batch_reader
                    .find_raw_schema_views_by_paths(&schema_paths)?;
                let ids =
                    batch_reader.find_schema_ids_by_paths(&schema_paths)?;

                Ok((graph, bank_view, views, ids))
            })
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // Step 5: Build discovered files map
        let mut files = HashMap::new();

        // Add property bank if found
        if let Some(entry) = property_bank_entry {
            files.insert(property_bank_path.clone(), DiscoveredFile {
                kind: SchemaFileKind::PropertyBank,
                view: bank_view.map(DiscoveredView::PropertyBank),
                info: entry.info,
            });
        }

        let mut filesystem_ids = HashSet::new();

        // Add schema files
        for (path, entry) in discovered.into_inner() {
            let id = ids_by_path.remove(&path).unwrap_or_else(SchemaId::new);
            let view = views_by_path.remove(&path).map(DiscoveredView::Schema);

            filesystem_ids.insert(id);

            files.insert(path, DiscoveredFile {
                kind: SchemaFileKind::Schema(id),
                view,
                info: entry.info,
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
    /// Returns `FileDiscovery` containing all discovered files (schemas and
    /// property bank).
    ///
    /// # Errors
    ///
    /// Returns error if filesystem scanning fails.
    fn scan_filesystem(
        spec: &SchemaConfigSpec,
        vault_root: &std::path::Path,
    ) -> Result<FileDiscovery, SchemaLoaderError> {
        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        let schema_dir = spec.directory();

        // Scan directory with extension filter (DirScanner handles filtering)
        let pattern = format!("{}/**/*", schema_dir.as_path().display());
        let scanner = DirScanner::new(vault_root);
        let entries = scanner
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
            })?;

        // Convert to HashMap for O(1) lookups
        let mut files = HashMap::new();
        for entry in entries {
            let Ok(relative_path) = RelativePath::try_from(entry.path.clone())
            else {
                continue;
            };
            files.insert(relative_path, entry);
        }

        if files.is_empty() {
            tracing::info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }

        Ok(FileDiscovery(files))
    }

    fn detect_deleted_schemas(
        graph: Option<&InheritanceGraph<()>>,
        filesystem_ids: &HashSet<SchemaId>,
    ) -> Vec<SchemaId> {
        let Some(graph) = graph else {
            return Vec::new();
        };

        let mut deleted_ids = Vec::new();
        for id in graph.topo_order() {
            if !filesystem_ids.contains(id) {
                deleted_ids.push(*id);
            }
        }

        deleted_ids
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::schema::testing::InMemoryRepository;

    #[test]
    fn run_finds_all_files() {
        let root = tempfile::tempdir().unwrap();
        let schema_dir = root.path().join("schemas");
        std::fs::create_dir(&schema_dir).unwrap();

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
        let outcome = DiscoveryEngine::run(&spec, &repo, root.path()).unwrap();

        assert_eq!(outcome.files.len(), 2);
        assert!(outcome.property_bank().is_some());
        assert_eq!(outcome.schema_files().count(), 1);
    }
}
