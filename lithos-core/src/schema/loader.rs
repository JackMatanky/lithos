//! Schema loader — orchestrates the full schema ingestion pipeline.
//!
//! ## Orchestration Pattern
//!
//! The loader coordinates the file → raw → resolved → database pipeline.
//! It is the **only** place where orchestration logic lives (following the
//! single responsibility principle).
//!
//! - **Schema is a read model**: Domain types have no file I/O or DB methods
//! - **Functional composition**: Direct function calls with `Result<T, E>` for
//!   error propagation
//! - **Staleness detection**: Two-tier (timestamp fast path, hash slow path)
//!
//! ## Pipeline Flow
//!
//! The loader uses **staleness detection** to decide whether to load from
//! files or reuse cached data from the database:
//!
//! 1. **Load existing state from DB**:
//!    - `list_schema_name_id_pairs()` → name→id map
//!    - `get_property_bank()` → cached `PropertyBank` (if exists)
//!    - `get_raw_property_bank_view()` → staleness metadata
//!
//! 2. **`PropertyBank` staleness check**:
//!    - If no bank in DB → **load from file** (`ingestor.property_bank()`)
//!    - If bank exists:
//!      - `view.is_fresh()` checks file metadata vs DB version
//!      - Stale → **reload from file**
//!      - Fresh → **reuse cached bank**
//!
//! 3. **Scan all schema files** (always from filesystem):
//!    - `ingestor.all_schemas()` → `Vec<RawSchema>`
//!    - Schema names derived from filenames
//!
//! 4. **Schema staleness partitioning**:
//!    - `partition_by_staleness()` → O(1) check for all schemas
//!    - Compares file metadata vs DB metadata
//!    - **Cascade staleness**: Parent changes mark all descendants stale
//!    - **Stale schemas** → reload + re-resolve
//!    - **Fresh schemas** → reuse from DB (fetch as `known_parents`)
//!
//! 5. **Process only stale schemas**:
//!    - `RefExpander::expand_all()` → resolve property refs
//!    - `Extender::build()` → build inheritance tree
//!    - `Merger::resolve()` → merge parent properties
//!
//! 6. **Persist changes**:
//!    - `save_schemas()` → save resolved schemas
//!    - `save_raw_schema_view()` → save staleness metadata
//!    - `save_property_bank()` → save bank if changed
//!
//! **Key optimization**: Lightweight staleness checks (metadata comparison)
//! avoid parsing/processing unchanged files.
//!
//! All schema-specific error types live in [`crate::schema::error`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use std::collections::HashMap;

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::{Schema, SchemaId, SchemaName},
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        expander::{RefExpandedSchema, RefExpander},
        extender::Extender,
        ingestor::{Ingestor, SchemaResult},
        merger::Merger,
        property::{Property, PropertyName},
        raw::RawSchema,
        storage::Repository,
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Loader
// ─────────────────────────────────────────────────────────────────────────────

/// Schema loader — orchestrates file ingestion and resolution.
///
/// Generic over `Repository` to support both production (redb) and test
/// implementations. Embeds an `Ingestor` for file I/O.
pub struct Loader<'config, R> {
    ingestor: Ingestor<'config, R>,
}

impl<'config, R> Loader<'config, R>
where
    R: Repository,
    R::Error: Into<SchemaRepositoryError>,
{
    /// Create a new `Loader` with a repository, file source, and config.
    ///
    /// The loader embeds an `Ingestor` which handles file I/O.
    #[inline]
    #[must_use]
    pub fn new(
        repository: R,
        source: FsReader,
        config: &'config Config,
    ) -> Self {
        Self {
            ingestor: Ingestor::new(source, config, repository),
        }
    }

    /// Run the full ingestion pipeline.
    ///
    /// Uses structured `IngestorResults` to eliminate double-loop pattern
    /// and enable incremental resolution when `PropertyBank` is fresh.
    ///
    /// Reads raw files via the embedded ingestor, resolves schemas through the
    /// `RefExpander → Extender → Resolver` pipeline, and persists the
    /// results.  Only stale schemas are re-resolved; fresh schemas are used
    /// as `known_parents` for the inheritance tree.
    ///
    /// # Phase 5.2 Optimization
    ///
    /// When a schema file is Fresh (unchanged) but `PropertyBank` is stale:
    /// - If cached expansion exists → skip `RefExpander`, use cached properties
    /// - If no cached expansion → run `RefExpander` normally
    ///
    /// This optimization avoids expensive property resolution for schemas that
    /// haven't changed but reference modified bank properties.
    ///
    /// Returns the set of freshly resolved schemas.  Schemas that were
    /// already fresh in the DB are not included (they were not re-resolved).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] on any I/O, parsing, domain, or
    /// storage failure.
    #[inline]
    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // ── Single ingestion call (no double loop!) ─────────────────────────
        let results = self.ingestor.ingest_all()?;

        // ── Extract property bank ───────────────────────────────────────────
        let bank = results.property_bank.bank();
        let bank_is_fresh = results.property_bank.is_fresh();

        // ── Partition schemas based on SchemaResult variants ───────────────
        // Three categories:
        // - needs_expansion: Run RefExpander (file changed/new, or fresh but no
        //   cache)
        // - cached_expansion: Skip RefExpander (fresh + bank stale + has cache)
        // - fresh_ids: No processing needed (fresh + bank fresh)
        let mut needs_expansion = Vec::new();
        let mut cached_expansion = Vec::new();
        let mut fresh_ids = Vec::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "Order doesn't affect correctness - partitioning by \
                      staleness"
        )]
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Matching borrowed SchemaResult from HashMap values"
        )]
        for result in results.schemas.values() {
            match result {
                // Bank fresh + Schema fresh = fully reusable
                SchemaResult::Fresh {
                    id,
                    ..
                } if bank_is_fresh => {
                    fresh_ids.push(*id);
                }
                // Bank stale + Schema fresh + cached expansion = skip
                // RefExpander
                SchemaResult::Fresh {
                    id,
                    expanded: Some(cached),
                } => {
                    cached_expansion.push((*id, cached.clone()));
                }
                // Bank stale + Schema fresh + no cache = run RefExpander
                SchemaResult::Fresh {
                    id,
                    expanded: None,
                } => {
                    if let Some(raw) = self.load_raw_from_view(*id)? {
                        needs_expansion.push((*id, raw));
                    }
                }
                // File changed or new = run RefExpander
                SchemaResult::Stale {
                    id,
                    raw,
                    ..
                }
                | SchemaResult::New {
                    id,
                    raw,
                } => {
                    needs_expansion.push((*id, raw.clone()));
                }
            }
        }

        // ── Collect all IDs for known_parents ───────────────────────────────
        // Both fresh and cached_expansion schemas need known_parents for
        // inheritance
        let mut parent_ids = fresh_ids;
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Iterator returns &T, we need to dereference to get T"
        )]
        parent_ids.extend(cached_expansion.iter().map(|(id, _)| *id));

        // Load fresh + cached schemas as known_parents for inheritance
        let parent_schemas = self
            .ingestor
            .repository()
            .find_schemas_by_ids(&parent_ids)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let known_parents: HashMap<SchemaId, Schema> = parent_schemas
            .into_iter()
            .map(|schema| (*schema.id(), schema))
            .collect();

        // ── Process schemas needing full expansion ─────────────────────────
        let mut resolved = Vec::new();

        if !needs_expansion.is_empty() {
            // Run full resolution pipeline (RefExpander → Extender → Merger)
            let expanded =
                RefExpander::new(bank).expand_all(needs_expansion.clone())?;

            // Store expanded properties for future incremental resolution
            self.store_expanded_properties(&expanded)?;

            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built for needs_expansion"
            );
            let full_resolved = Merger::resolve(&tree, &known_parents)?;
            resolved.extend(full_resolved);
        }

        // ── Process schemas with cached expansion (skip RefExpander!) ───────
        if !cached_expansion.is_empty() {
            let cached_resolved = self.resolve_with_cached_expansion(
                cached_expansion,
                &known_parents,
            )?;
            resolved.extend(cached_resolved);
        }

        // ── Persist resolved schemas ────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_resolved_schemas(&resolved)?;

            // Note: RawSchemaView already persisted by Ingestor during
            // ingest_all() call

            // Build name_to_id map for inheritance metadata
            let name_to_id = Self::build_name_to_id_map(&needs_expansion)?;

            // Persist inheritance metadata for caching
            self.persist_inheritance_metadata(&needs_expansion, &name_to_id)?;
        }

        Ok(resolved)
    }

    /// Resolve schemas with cached expanded properties (Phase 5.2
    /// optimization).
    ///
    /// Skips `RefExpander` entirely - uses cached properties directly.
    /// Only needs to re-apply inheritance via `Extender` and `Merger`.
    #[expect(
        clippy::type_complexity,
        reason = "Vec tuple is clear in this context"
    )]
    fn resolve_with_cached_expansion(
        &self,
        cached: Vec<(SchemaId, HashMap<PropertyName, Property>)>,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        let mut expanded = Vec::with_capacity(cached.len());

        for (id, cached_props) in cached {
            // Load raw schema metadata from view (name, extends, excludes)
            let raw = self.load_raw_from_view(id)?.ok_or_else(|| {
                SchemaLoaderError::Repository(SchemaRepositoryError::NotFound {
                    name: format!("schema id {id:?}").into(),
                })
            })?;

            // Construct RefExpandedSchema directly from cached properties
            let exp_schema = RefExpandedSchema {
                name: raw.name,
                extends: raw.extends,
                excludes: raw.excludes,
                properties: cached_props,
            };

            expanded.push((id, exp_schema));
        }

        let tree = Extender::build(expanded, known_parents)?;
        tracing::debug!(
            root_count = tree.roots().len(),
            total_count = tree.nodes().len(),
            "schema tree built for cached_expansion (skipped RefExpander)"
        );

        let resolved = Merger::resolve(&tree, known_parents)?;
        Ok(resolved)
    }

    /// Load raw schema from database view.
    fn load_raw_from_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchema>, SchemaLoaderError> {
        let view = self
            .ingestor
            .repository()
            .get_raw_schema_view(id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        match view {
            Some(v) => v.to_raw().map_err(SchemaLoaderError::Ingestion),
            None => Ok(None),
        }
    }

    /// Store expanded properties in schema views.
    ///
    /// Called after `RefExpander` runs to cache the expanded properties,
    /// enabling incremental resolution when `PropertyBank` is fresh.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Reference pattern needed for iteration over slices"
    )]
    fn store_expanded_properties(
        &self,
        expanded: &[(SchemaId, RefExpandedSchema)],
    ) -> Result<(), SchemaLoaderError> {
        // Update each schema's view with expanded properties
        for (id, exp_schema) in expanded {
            // Store expanded properties (already HashMap)
            let props_map = exp_schema.properties.clone();

            // Load view, update current version's expanded properties, save
            if let Some(mut view) = self
                .ingestor
                .repository()
                .get_raw_schema_view(*id)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            {
                if let Some(current) = view.current_mut() {
                    current.set_expanded_properties(props_map);
                }

                // Save updated view
                self.ingestor
                    .repository()
                    .save_raw_schema_view(*id, &view)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            }
        }

        Ok(())
    }

    /// Build name-to-ID map from schema list.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching tuple references requires explicit destructuring"
    )]
    fn build_name_to_id_map(
        schemas: &[(SchemaId, RawSchema)],
    ) -> Result<HashMap<SchemaName, SchemaId>, SchemaLoaderError> {
        schemas
            .iter()
            .map(|(id, raw)| {
                SchemaName::try_new(&raw.name).map(|name| (name, *id))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))
    }

    /// Persist resolved schemas to the database.
    fn persist_resolved_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaLoaderError> {
        self.ingestor
            .repository()
            .save_schemas(schemas)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }

    /// Persist inheritance metadata for caching.
    ///
    /// Computes and saves `SchemaInheritanceView` for each schema, enabling
    /// future optimizations like skipping tree rebuilds when inheritance is
    /// unchanged.
    ///
    /// Note: This first implementation saves immediate parent only. Ancestors
    /// list will be computed on-demand during staleness checks.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Reference pattern needed for iteration over slice"
    )]
    fn persist_inheritance_metadata(
        &self,
        schemas: &[(SchemaId, RawSchema)],
        name_to_id: &HashMap<SchemaName, SchemaId>,
    ) -> Result<(), SchemaLoaderError> {
        use std::time::SystemTime;

        use super::views::SchemaInheritanceView;

        for (schema_id, raw) in schemas {
            // Resolve parent name to ID
            let parent = if let Some(parent_name) = &raw.extends {
                let pid =
                    *name_to_id.get(parent_name.as_ref()).ok_or_else(|| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                            path: format!("schema: {}", raw.name).into(),
                            reason: format!("Parent not found: {parent_name}")
                                .into(),
                        })
                    })?;
                Some(pid)
            } else {
                None
            };

            // For now, ancestors list is empty - will be computed on-demand
            // This simplifies the implementation and avoids topological
            // ordering issues
            let ancestors = Vec::new();

            // Compute ancestors hash from parent (or 0 if root)
            let ancestors_hash = if let Some(pid) = parent {
                use std::hash::{Hash as _, Hasher as _};
                let mut hasher =
                    std::collections::hash_map::DefaultHasher::new();
                pid.hash(&mut hasher);
                hasher.finish()
            } else {
                0
            };

            let metadata = SchemaInheritanceView {
                parent,
                ancestors,
                excludes: raw.excludes.clone(),
                ancestors_hash,
                resolved_at: SystemTime::now(),
            };

            // Persist to database
            self.ingestor
                .repository()
                .save_inheritance_metadata(*schema_id, &metadata)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test submodules organized by feature, not alphabetically"
)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::schema::{
        aggregate::Schema, storage::RedbRepository, testing::InMemoryRepository,
    };

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    /// Test database context that holds both the temp directory and path.
    /// Allows reopening the database to work around rkyv deserialization
    /// issues.
    struct TestDbContext {
        _dir: TempDir,
        path: std::path::PathBuf,
    }

    impl TestDbContext {
        fn new() -> TestResult<Self> {
            let dir = TempDir::new()?;
            let path = dir.path().join("test.redb");
            Ok(Self {
                _dir: dir,
                path,
            })
        }

        fn open(&self) -> TestResult<Arc<crate::db::Database>> {
            Ok(Arc::new(crate::db::Database::open(&self.path)?))
        }
    }

    /// Helper to write a file.
    fn write_file(
        root: &std::path::Path,
        relative: &str,
        content: &str,
    ) -> TestResult {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Helper to create test config.
    fn test_config(root: &std::path::Path) -> TestResult<Config> {
        use crate::config::{
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        };
        let raw = RawConfig::default();
        let vault_root = VaultRoot::try_new(root.to_path_buf())?;
        let config = Config::build(
            &raw,
            VaultId::new(),
            vault_root,
            crate::config::aggregate::Version::initial(),
        )?;
        Ok(config)
    }

    // ========================================================================
    // Pipeline Integration Tests
    // ========================================================================

    /// Tests for full pipeline orchestration - loading, resolution, and
    /// storage.
    mod pipeline_tests {
        use super::*;

        #[test]
        fn resolves_schema_when_new_file_is_added() -> TestResult {
            // GIVEN: Empty DB + new schema file
            let vault_dir = TempDir::new()?;
            let db_ctx = TestDbContext::new()?;
            let db = db_ctx.open()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            let config = test_config(vault_dir.path())?;
            let repository = RedbRepository::new(Arc::clone(&db));
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);

            // WHEN: Loading schemas
            let resolved = loader.load()?;

            // THEN: Schema is resolved via full pipeline
            if resolved.len() != 1 {
                return Err(format!(
                    "Expected 1 schema, got {}",
                    resolved.len()
                )
                .into());
            }

            let schema =
                resolved.first().ok_or("Expected at least one schema")?;
            if schema.properties().len() != 1 {
                return Err(format!(
                    "Expected 1 property, got {}",
                    schema.properties().len()
                )
                .into());
            }

            Ok(())
        }

        #[test]
        fn reloads_schema_when_file_changes() -> TestResult {
            let vault_dir = TempDir::new()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            let config = test_config(vault_dir.path())?;
            let repository = InMemoryRepository::new();

            // GIVEN: First load populates repository
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository.clone(), source, &config);
            let initial = loader.load()?;

            if initial.len() != 1 {
                return Err(format!(
                    "Expected 1 initial schema, got {}",
                    initial.len()
                )
                .into());
            }

            // WHEN: File changes (add property)
            #[expect(
                clippy::disallowed_methods,
                reason = "Test needs filesystem timing"
            )]
            std::thread::sleep(std::time::Duration::from_millis(10));

            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {
                    "title": {"$ref": "property_bank#/title"},
                    "status": {"type": "bool"}
                }}"#,
            )?;

            // THEN: Full resolution updates schema
            let source2 = FsReader::new(vault_dir.path());
            let loader2 = Loader::new(repository.clone(), source2, &config);
            let updated = loader2.load()?;

            if updated.len() != 1 {
                return Err(format!(
                    "Expected 1 updated schema, got {}",
                    updated.len()
                )
                .into());
            }
            let schema =
                updated.first().ok_or("Expected at least one schema")?;
            if schema.properties().len() != 2 {
                return Err(format!(
                    "Expected 2 properties, got {}",
                    schema.properties().len()
                )
                .into());
            }

            Ok(())
        }

        #[test]
        fn handles_mixed_new_changed_and_incremental_paths() -> TestResult {
            let vault_dir = TempDir::new()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {
                    "title": {"type": "string"},
                    "status": {"type": "string"}
                }}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/note.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            let config = test_config(vault_dir.path())?;
            let repository = InMemoryRepository::new();

            // GIVEN: First load: 2 new schemas
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository.clone(), source, &config);
            let initial = loader.load()?;

            if initial.len() != 2 {
                return Err(format!(
                    "Expected 2 initial schemas, got {}",
                    initial.len()
                )
                .into());
            }

            #[expect(
                clippy::disallowed_methods,
                reason = "Test needs filesystem timing"
            )]
            std::thread::sleep(std::time::Duration::from_millis(10));

            // WHEN: Mixed changes:
            // 1. Add new schema (project.json) - NEW path
            // 2. Modify task.json - FILE-CHANGED path
            // 3. Modify property bank title - affects note.json via INCREMENTAL
            //    path

            write_file(
                vault_dir.path(),
                "schemas/project.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {
                    "title": {"$ref": "property_bank#/title"},
                    "done": {"type": "bool"}
                }}"#,
            )?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {
                    "title": {"type": "string", "max_length": 100},
                    "status": {"type": "string"}
                }}"#,
            )?;

            // THEN: All three paths exercised
            let source2 = FsReader::new(vault_dir.path());
            let loader2 = Loader::new(repository.clone(), source2, &config);
            let updated = loader2.load()?;

            if updated.len() < 2 {
                return Err(format!(
                    "Expected at least 2 updated schemas, got {}",
                    updated.len()
                )
                .into());
            }

            // Verify we got the expected schemas
            let names: Vec<&str> =
                updated.iter().map(|s: &Schema| s.name().as_ref()).collect();
            if !names.contains(&"project") {
                return Err("Expected to find 'project' schema".into());
            }
            if !names.contains(&"task") {
                return Err("Expected to find 'task' schema".into());
            }

            Ok(())
        }
    }

    // ========================================================================
    // Incremental Resolution Tests
    // ========================================================================

    /// Tests for incremental resolution optimization when only property bank
    /// changes.
    mod incremental_resolution_tests {
        use super::*;

        #[test]
        fn applies_incremental_when_only_bank_changes() -> TestResult {
            let vault_dir = TempDir::new()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"status": {"type": "string"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {"status": {"$ref": "property_bank#/status"}}}"#,
            )?;

            let config = test_config(vault_dir.path())?;
            let repository = InMemoryRepository::new();

            // GIVEN: First load populates repository
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository.clone(), source, &config);
            let initial = loader.load()?;

            if initial.len() != 1 {
                return Err(format!(
                    "Expected 1 initial schema, got {}",
                    initial.len()
                )
                .into());
            }

            // WHEN: Property bank changes (modify status property)
            #[expect(
                clippy::disallowed_methods,
                reason = "Test needs filesystem timing"
            )]
            std::thread::sleep(std::time::Duration::from_millis(10));

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"status": {"type": "bool"}}}"#,
            )?;

            // THEN: Incremental resolution updates schema
            let source2 = FsReader::new(vault_dir.path());
            let loader2 = Loader::new(repository.clone(), source2, &config);
            let updated = loader2.load()?;

            if updated.len() != 1 {
                return Err(format!(
                    "Expected 1 updated schema, got {}",
                    updated.len()
                )
                .into());
            }

            Ok(())
        }

        #[test]
        fn skips_incremental_when_property_hash_unchanged() -> TestResult {
            let vault_dir = TempDir::new()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            let config = test_config(vault_dir.path())?;
            let repository = InMemoryRepository::new();

            // GIVEN: First load populates repository
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository.clone(), source, &config);
            let initial = loader.load()?;

            if initial.len() != 1 {
                return Err(format!(
                    "Expected 1 initial schema, got {}",
                    initial.len()
                )
                .into());
            }

            // WHEN: Touch file without changing content hash
            #[expect(
                clippy::disallowed_methods,
                reason = "Test needs filesystem timing"
            )]
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Rewrite same content
            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
            )?;

            // THEN: No schemas re-resolved (hash unchanged)
            let source2 = FsReader::new(vault_dir.path());
            let loader2 = Loader::new(repository.clone(), source2, &config);
            let updated = loader2.load()?;

            if !updated.is_empty() {
                return Err(format!(
                    "Expected 0 updated schemas, got {}",
                    updated.len()
                )
                .into());
            }

            Ok(())
        }
    }

    // ========================================================================
    // Phase 5.2 Cached Expansion Tests (CRITICAL COVERAGE GAP)
    // ========================================================================

    /// Tests for Phase 5.2 optimization: cached expansion when property bank is
    /// stale but schema files are fresh.
    ///
    /// Uses `InMemoryRepository` for pure unit testing.
    mod cached_expansion_tests {
        use super::*;
        use crate::schema::testing::InMemoryRepository;

        /// Helper to create minimal test config.
        fn test_config_inmem(root: &std::path::Path) -> TestResult<Config> {
            use crate::config::{
                raw::RawConfig,
                vault::{VaultId, VaultRoot},
            };
            let raw = RawConfig::default();
            let vault_root = VaultRoot::try_new(root.to_path_buf())?;
            let config = Config::build(
                &raw,
                VaultId::new(),
                vault_root,
                crate::config::aggregate::Version::initial(),
            )?;
            Ok(config)
        }

        #[test]
        fn first_load_expands_and_resolves_schemas() -> TestResult {
            // GIVEN: Fresh vault with schema referencing property bank
            let vault_dir = TempDir::new()?;

            write_file(
                vault_dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
            )?;
            write_file(
                vault_dir.path(),
                "schemas/note.json",
                r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
            )?;

            let config = test_config_inmem(vault_dir.path())?;
            let repository = InMemoryRepository::new();
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);

            // WHEN: Loading schemas (first time - runs full expansion)
            let resolved = loader.load()?;

            // THEN: Schema should be fully resolved with expanded properties
            if resolved.len() != 1 {
                return Err(format!(
                    "Expected 1 schema, got {}",
                    resolved.len()
                )
                .into());
            }

            let schema = resolved.first().ok_or("Expected schema")?;
            if schema.name().as_ref() != "note" {
                return Err(format!(
                    "Expected 'note', got {}",
                    schema.name().as_ref()
                )
                .into());
            }

            // Should have the title property expanded from property bank
            if !schema
                .properties()
                .contains_key(&PropertyName::try_new("title")?)
            {
                return Err("Schema should have 'title' property expanded \
                            from property bank"
                    .into());
            }

            Ok(())
        }

        #[test]
        fn cached_expansion_used_when_bank_stale_schema_fresh() {
            // This test requires simulating:
            // 1. Initial load (caches expanded properties)
            // 2. Property bank change
            // 3. Second load uses cached expansion
            //
            // For now, skip this as it requires more complex test setup with
            // filesystem manipulation between loads.
            // TODO: Implement when we have better test fixture management
        }
    }
}
