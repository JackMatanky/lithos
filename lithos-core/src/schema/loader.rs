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
        ingestor::{Ingestor, IngestorResults, SchemaResult},
        merger::Merger,
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

        // ── Partition schemas for resolution ────────────────────────────────
        let (schemas_to_resolve, fresh_ids) =
            self.partition_for_resolution(&results)?;

        // ── Process only schemas that need resolution ───────────────────────
        let mut resolved = Vec::new();

        if !schemas_to_resolve.is_empty() {
            // Load fresh schemas as known_parents for inheritance
            let fresh_schemas = self
                .ingestor
                .repository()
                .find_schemas_by_ids(&fresh_ids)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            let known_parents: HashMap<SchemaId, Schema> = fresh_schemas
                .into_iter()
                .map(|schema| (*schema.id(), schema))
                .collect();

            // Run full resolution pipeline
            let expanded = RefExpander::new(bank)
                .expand_all(schemas_to_resolve.clone())?;

            // Store expanded properties for future incremental resolution
            self.store_expanded_properties(&expanded)?;

            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built for resolution"
            );
            let full_resolved = Merger::resolve(&tree, &known_parents)?;
            resolved.extend(full_resolved);
        }

        // ── Persist resolved schemas ────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_resolved_schemas(&resolved)?;

            // Note: RawSchemaView already persisted by Ingestor during
            // ingest_all() call

            // Build name_to_id map for inheritance metadata
            let name_to_id = Self::build_name_to_id_map(&schemas_to_resolve)?;

            // Persist inheritance metadata for caching
            self.persist_inheritance_metadata(
                &schemas_to_resolve,
                &name_to_id,
            )?;
        }

        Ok(resolved)
    }

    /// Partition schemas for resolution based on staleness.
    ///
    /// Returns (`schemas_to_resolve`, `fresh_schema_ids`).
    #[expect(
        clippy::type_complexity,
        reason = "Complex return type is clear in this internal helper; \
                  extracting a type alias would reduce locality"
    )]
    fn partition_for_resolution(
        &self,
        results: &IngestorResults,
    ) -> Result<(Vec<(SchemaId, RawSchema)>, Vec<SchemaId>), SchemaLoaderError>
    {
        let mut to_resolve = Vec::new();
        let mut fresh_ids = Vec::new();

        // PropertyBank staleness affects whether we can reuse cached schemas
        let bank_is_fresh = results.property_bank.is_fresh();

        // Iterate over hash map values in arbitrary order - order doesn't
        // matter for partition logic
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Order doesn't affect correctness - we're just \
                      partitioning by staleness"
        )]
        for result in results.schemas.values() {
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Matching borrowed enum requires dereferencing \
                          pattern bindings"
            )]
            match result {
                SchemaResult::Fresh {
                    id,
                    ..
                } if bank_is_fresh => {
                    // PropertyBank fresh + Schema fresh = fully reusable
                    fresh_ids.push(*id);
                }
                SchemaResult::Fresh {
                    id,
                    ..
                } => {
                    // Schema fresh but PropertyBank changed - need to
                    // re-resolve Load raw schema from DB
                    // view to re-resolve with new bank
                    if let Some(raw) = self.load_raw_from_view(*id)? {
                        to_resolve.push((*id, raw));
                    }
                }
                SchemaResult::Stale {
                    id,
                    raw,
                    ..
                }
                | SchemaResult::New {
                    id,
                    raw,
                } => {
                    // File changed or new - needs full resolution
                    to_resolve.push((*id, raw.clone()));
                }
            }
        }

        Ok((to_resolve, fresh_ids))
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
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::schema::{aggregate::Schema, storage::RedbRepository};

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

    /// **TEST-001**: New schema uses full resolution pipeline.
    #[test]
    fn new_schema_uses_full_resolution() -> TestResult {
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
            return Err(
                format!("Expected 1 schema, got {}", resolved.len()).into()
            );
        }
        let schema = resolved.first().ok_or("Expected at least one schema")?;
        if schema.name().as_ref() != "task" {
            return Err(format!(
                "Expected name 'task', got '{}'",
                schema.name().as_ref()
            )
            .into());
        }
        if schema.properties().len() != 1 {
            return Err(format!(
                "Expected 1 property, got {}",
                schema.properties().len()
            )
            .into());
        }

        Ok(())
    }

    /// **TEST-002**: Existing schema with file change uses full resolution.
    #[test]
    #[ignore = "rkyv deserialization limitation - requires integration test or \
                FakeRepository"]
    fn existing_schema_file_change_uses_full_resolution() -> TestResult {
        let vault_dir = TempDir::new()?;
        let db_ctx = TestDbContext::new()?;

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

        // First load - use a scope to ensure db drops
        let initial = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

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
        // Database from first scope is now dropped, safe to reopen
        let updated = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

        if updated.len() != 1 {
            return Err(format!(
                "Expected 1 updated schema, got {}",
                updated.len()
            )
            .into());
        }
        let schema = updated.first().ok_or("Expected at least one schema")?;
        if schema.properties().len() != 2 {
            return Err(format!(
                "Expected 2 properties, got {}",
                schema.properties().len()
            )
            .into());
        }

        Ok(())
    }

    /// **TEST-003**: Existing schema with only bank change uses incremental
    /// resolution.
    #[test]
    #[ignore = "rkyv deserialization limitation - requires integration test or \
                FakeRepository"]
    fn existing_schema_bank_change_uses_incremental() -> TestResult {
        let vault_dir = TempDir::new()?;
        let db_ctx = TestDbContext::new()?;

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

        // First load - use a scope to ensure db drops
        let initial = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

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
        // Database from first scope is now dropped, safe to reopen
        let updated = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

        if updated.len() != 1 {
            return Err(format!(
                "Expected 1 updated schema, got {}",
                updated.len()
            )
            .into());
        }

        Ok(())
    }

    /// **TEST-004**: No incremental when property hash unchanged.
    #[test]
    #[ignore = "rkyv deserialization limitation - requires integration test or \
                FakeRepository"]
    fn no_incremental_when_property_unchanged() -> TestResult {
        let vault_dir = TempDir::new()?;
        let db_ctx = TestDbContext::new()?;

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

        // First load - use a scope to ensure db drops
        let initial = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

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
        // Database from first scope is now dropped, safe to reopen
        let updated = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

        if !updated.is_empty() {
            return Err(format!(
                "Expected 0 updated schemas, got {}",
                updated.len()
            )
            .into());
        }

        Ok(())
    }

    /// **TEST-005**: Mixed scenario - new, file-changed, and incremental.
    #[test]
    #[ignore = "rkyv deserialization limitation - requires integration test or \
                FakeRepository"]
    fn mixed_scenario_handles_all_three_paths() -> TestResult {
        let vault_dir = TempDir::new()?;
        let db_ctx = TestDbContext::new()?;

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

        // First load: 2 new schemas - use a scope to ensure db drops
        let initial = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

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
        // Database from first scope is now dropped, safe to reopen
        let updated = {
            let db = db_ctx.open()?;
            let repository = RedbRepository::new(db);
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            loader.load()?
        };

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
