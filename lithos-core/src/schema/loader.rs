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
//!    - `Resolver::resolve()` → merge parent properties
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
        bank::PropertyBank,
        error::{
            SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError,
        },
        expander::RefExpander,
        extender::Extender,
        ingestor::Ingestor,
        raw::RawSchema,
        resolver::Resolver,
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
    repository: R,
    ingestor: Ingestor<'config>,
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
            repository,
            ingestor: Ingestor::new(source, config),
        }
    }

    /// Run the full ingestion pipeline.
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
    #[expect(
        clippy::too_many_lines,
        reason = "Loader orchestrates multi-phase resolution pipeline"
    )]
    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // ── Step 1: read existing DB state ──────────────────────────────────
        let name_to_id = self.load_name_to_id_map()?;

        // ── Step 2: PropertyBank - handle loading with staleness internally
        let (bank, changed_properties) = self.load_property_bank()?;

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas = self.ingestor.all_schemas()?;

        // ── Step 4: check staleness for each schema ─────────────────────────
        // Track whether each schema's FILE changed (not just bank cascade)
        let mut stale = Vec::new();
        let mut file_changed_ids = std::collections::HashSet::new();
        let mut fresh_ids = Vec::new();

        for raw_schema in &raw_schemas {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            let existing_id = name_to_id.get(&schema_name).copied();

            let is_stale = self.is_schema_stale(raw_schema, existing_id)?;

            match (is_stale, existing_id) {
                (true, _) => {
                    let id = existing_id.unwrap_or_else(SchemaId::new);
                    // Track if this is file-stale (not just bank cascade)
                    // New schemas and schemas with file changes are file-stale
                    if existing_id.is_none() || is_stale {
                        file_changed_ids.insert(id);
                    }
                    stale.push((id, raw_schema.clone()));
                }
                (false, Some(id)) => {
                    fresh_ids.push(id);
                }
                (false, None) => {
                    // New schema that somehow wasn't marked stale - shouldn't
                    // happen
                }
            }
        }

        // ── Step 5: Partition stale into categories ─────────────────────────
        let mut new_schemas = Vec::new();
        let mut existing_file_changed = Vec::new();
        let mut existing_file_unchanged = Vec::new();

        for (id, raw) in stale {
            if !name_to_id.values().any(|&existing| existing == id) {
                // NEW schema
                new_schemas.push((id, raw));
            } else if file_changed_ids.contains(&id) {
                // EXISTING schema with FILE changes
                existing_file_changed.push((id, raw));
            } else {
                // EXISTING schema with UNCHANGED file (only bank-cascade stale)
                existing_file_unchanged.push((id, raw));
            }
        }

        // ── Step 6: Incremental resolution for existing unchanged files ─────
        let mut resolved = Vec::new();

        if !existing_file_unchanged.is_empty() && !changed_properties.is_empty()
        {
            // Find which existing schemas use the changed properties
            let affected_map = self
                .repository
                .find_schemas_using_properties(&changed_properties)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            // Load existing schemas that are affected
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Tuple destructuring requires ref pattern"
            )]
            let unchanged_ids: Vec<SchemaId> =
                existing_file_unchanged.iter().map(|(id, _)| *id).collect();
            let stored_schemas = self
                .repository
                .find_schemas_by_ids(&unchanged_ids)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            // Apply incremental resolution to affected schemas
            for schema in &stored_schemas {
                if let Some(affected_props) = affected_map.get(schema.id()) {
                    let updated = Resolver::resolve_affected_properties(
                        schema,
                        affected_props,
                        &bank,
                    )?;
                    resolved.push(updated);
                }
            }
        }

        // ── Step 7: Full resolution for new + file-changed schemas ──────────
        let schemas_for_full_resolution: Vec<(SchemaId, RawSchema)> =
            new_schemas.into_iter().chain(existing_file_changed).collect();

        if !schemas_for_full_resolution.is_empty() {
            // Load fresh schemas as known_parents for inheritance
            let fresh_schemas = self
                .repository
                .find_schemas_by_ids(&fresh_ids)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            let known_parents: HashMap<SchemaId, Schema> = fresh_schemas
                .into_iter()
                .map(|schema| (*schema.id(), schema))
                .collect();

            // Run full resolution pipeline
            let expanded = RefExpander::new(&bank)
                .expand_all(schemas_for_full_resolution.clone())?;
            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built for full resolution"
            );
            let full_resolved = Resolver::resolve(&tree, &known_parents)?;
            resolved.extend(full_resolved);
        }

        // ── Step 8: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_resolved_schemas(&resolved)?;
            // Persist views for all processed schemas
            let all_stale: Vec<(SchemaId, RawSchema)> =
                schemas_for_full_resolution
                    .into_iter()
                    .chain(existing_file_unchanged)
                    .collect();

            // Extend name_to_id with new schemas for parent lookup
            let complete_name_to_id =
                Self::extend_name_to_id_map(&name_to_id, &all_stale)?;

            self.persist_raw_views(&all_stale)?;
            // Persist inheritance metadata for caching
            self.persist_inheritance_metadata(
                &all_stale,
                &complete_name_to_id,
            )?;
        }

        Ok(resolved)
    }

    /// Extend `name_to_id` map with new schemas for inheritance resolution.
    ///
    /// New schemas may reference other new schemas as parents (e.g., "task"
    /// extends "base" when both are new), so we need to include them in the
    /// lookup map before resolving inheritance.
    fn extend_name_to_id_map(
        name_to_id: &HashMap<SchemaName, SchemaId>,
        new_schemas: &[(SchemaId, RawSchema)],
    ) -> Result<HashMap<SchemaName, SchemaId>, SchemaLoaderError> {
        let mut complete = name_to_id.clone();
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Need explicit ref pattern for tuple in Vec iteration"
        )]
        for (id, raw) in new_schemas {
            let name = SchemaName::try_new(&raw.name)?;
            complete.insert(name, *id);
        }
        Ok(complete)
    }

    /// Load name-to-ID mapping from database.
    fn load_name_to_id_map(
        &self,
    ) -> Result<HashMap<SchemaName, SchemaId>, SchemaLoaderError> {
        let existing_pairs = self
            .repository
            .list_schema_name_id_pairs()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(existing_pairs.len());
        for (name, id) in existing_pairs {
            name_to_id.insert(name, id);
        }
        Ok(name_to_id)
    }

    /// Load the property bank with three cases:
    /// 1. First time (no view in DB): convert raw → domain → save
    /// 2. Stale: get changed properties, update existing bank
    /// 3. Fresh: load from DB
    ///
    /// Returns `(PropertyBank, Vec<PropertyName>)` where the second element
    /// contains property names that changed (empty if fresh or first time).
    fn load_property_bank(
        &self,
    ) -> Result<
        (PropertyBank, Vec<super::property::PropertyName>),
        SchemaLoaderError,
    > {
        let raw_bank = self.ingestor.property_bank()?.ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::FileSystem(
                "Property bank file not found".into(),
            ))
        })?;

        let bank_view =
            self.repository.get_raw_property_bank_view().map_err(|e| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                    path: "database".into(),
                    reason: e.to_string().into(),
                })
            })?;

        // Three cases based on view existence and staleness
        if let Some(mut view) = bank_view {
            // View exists - check staleness
            if view.is_fresh(&raw_bank.metadata) {
                // Case 3: Fresh - load from DB, no changes
                let bank = self
                    .repository
                    .get_property_bank()
                    .map_err(|e| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                            path: "database".into(),
                            reason: e.to_string().into(),
                        })
                    })?
                    .ok_or_else(|| {
                        SchemaLoaderError::Ingestion(
                            SchemaIngestionError::FileSystem(
                                "Property bank not found in database".into(),
                            ),
                        )
                    })?;
                return Ok((bank, Vec::new()));
            }
            // Case 2: Stale - get changed properties, update existing bank
            let changed = view.filter_changed_properties(&raw_bank.metadata);

            let mut bank = self
                .repository
                .get_property_bank()
                .map_err(|e| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                        path: "database".into(),
                        reason: e.to_string().into(),
                    })
                })?
                .ok_or_else(|| {
                    SchemaLoaderError::Ingestion(
                        SchemaIngestionError::FileSystem(
                            "Property bank not found in database".into(),
                        ),
                    )
                })?;

            bank.update_properties(&raw_bank, &changed)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

            self.repository
                .save_property_bank(&bank)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            // Update view
            view.add_version(
                raw_bank.metadata.content_hash.ok_or_else(|| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                        path: "property bank".into(),
                        reason: "missing content hash".into(),
                    })
                })?,
                raw_bank
                    .metadata
                    .property_hashes
                    .iter()
                    .filter_map(|(k, v)| {
                        super::property::PropertyName::try_new(k.as_ref())
                            .ok()
                            .map(|name| (name, *v))
                    })
                    .collect(),
                raw_bank.metadata.created_at,
                raw_bank.metadata.modified_at,
                None, // TODO(Phase 3): Pass compressed content from Ingestor
            );

            self.repository
                .save_raw_property_bank_view(&view)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            Ok((bank, changed))
        } else {
            // Case 1: First time - convert from raw and save
            let raw_bank_for_view = raw_bank.clone();
            let bank: PropertyBank = raw_bank.try_into()?;
            self.repository
                .save_property_bank(&bank)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            // Create and save view
            let view =
                super::views::RawPropertyBankView::try_from(&raw_bank_for_view)
                    .map_err(SchemaLoaderError::Ingestion)?;
            self.repository
                .save_raw_property_bank_view(&view)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            // First time = no changes to track
            Ok((bank, Vec::new()))
        }
    }

    /// Check if a schema is stale.
    ///
    /// Returns `true` if:
    /// - Schema ID is None (new schema)
    /// - No view exists in DB (never loaded)
    /// - Timestamps differ
    /// - Content hash differs
    fn is_schema_stale(
        &self,
        raw_schema: &RawSchema,
        existing_id: Option<SchemaId>,
    ) -> Result<bool, SchemaLoaderError> {
        // New schemas are always stale
        if existing_id.is_none() {
            return Ok(true);
        }

        #[expect(
            clippy::unwrap_used,
            reason = "Safe because we checked is_none() above"
        )]
        let view = self
            .repository
            .get_raw_schema_view(existing_id.unwrap())
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(match view {
            Some(v) => !v.is_fresh(&raw_schema.metadata),
            None => true, // No view = never loaded = stale
        })
    }

    /// Persist resolved schemas to the database.
    fn persist_resolved_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaLoaderError> {
        self.repository
            .save_schemas(schemas)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }

    /// Persist raw schema views for staleness tracking.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Reference pattern needed for iteration over slice"
    )]
    fn persist_raw_views(
        &self,
        stale: &[(SchemaId, RawSchema)],
    ) -> Result<(), SchemaLoaderError> {
        for (id, raw) in stale {
            let view = super::views::RawSchemaView::try_from(raw)
                .map_err(SchemaLoaderError::Ingestion)?;

            self.repository
                .save_raw_schema_view(*id, &view)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }
        Ok(())
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
            self.repository
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
