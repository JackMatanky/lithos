//! Schema loader — orchestrates the full schema ingestion pipeline.
//!
//! ## Orchestration Pattern
//!
//! The loader coordinates the file → raw → resolved → database pipeline.
//! It is the **only** place where orchestration logic lives (following the
//! single responsibility principle).
//!
//! - **No behavior in StoredSchema**: Schema is a read model (no methods)
//! - **Event emission**: Emits pipeline events for observability
//! - **Staleness detection**: Two-tier (timestamp fast path, hash slow path)
//!
//! ## Pipeline Flow
//!
//! The loader uses **staleness detection** to decide whether to load from
//! files or reuse cached data from the database:
//!
//! 1. **Load existing state from DB**:
//!    - `Query::list_name_id_pairs()` → name→id map
//!    - `Query::get_property_bank()` → cached `PropertyBank` (if exists)
//!
//! 2. **`PropertyBank` staleness check**:
//!    - If no bank in DB → **load from file** (`load_raw_property_bank()`)
//!    - If bank exists:
//!      - `Query::is_bank_stale()` checks file timestamp vs DB version
//!      - Stale → **reload from file**
//!      - Fresh → **reuse cached bank**
//!
//! 3. **Scan all schema files** (always from filesystem):
//!    - `Ingestor::all_schemas()` → `Vec<(RawSchema, timestamps)>`
//!    - Schema names derived from filenames
//!
//! 4. **Schema staleness partitioning**:
//!    - `Query::are_many_stale()` → O(1) check for all schemas
//!    - Compares file timestamps vs DB metadata
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
//!    - `Command::save_many()` → save only changed schemas
//!    - `Command::save_inheritance_many()` → track parent-child relationships
//!    - `Command::save_property_bank()` → save bank if stale
//!
//! **Key optimization**: Lightweight staleness checks (filename + timestamp)
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
        error::{SchemaError, SchemaIngestionError},
        expander::RefExpander,
        extender::Extender,
        ingestor::Ingestor,
        raw::RawSchema,
        resolver::Resolver,
        storage::Repository,
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  LoaderError
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during schema loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoaderError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] SchemaIngestionError),

    /// Domain validation failed.
    #[error("domain error: {0}")]
    Domain(#[from] SchemaError),

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),
}

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

// Type aliases for loader operations
// Metadata is now embedded in RawSchema.metadata, so we only track (ID,
// RawSchema)
type SchemaWithId = (SchemaId, RawSchema);
type PartitionResult = (Vec<SchemaWithId>, Vec<SchemaId>);

impl<'config, R> Loader<'config, R>
where
    R: Repository,
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
    /// Returns [`LoaderError`] on any I/O, parsing, domain, or storage
    /// failure.
    #[inline]
    pub fn load(&self) -> Result<Vec<Schema>, LoaderError> {
        // ── Step 1: read existing DB state ──────────────────────────────────
        let name_to_id = self.load_name_to_id_map()?;

        // ── Step 2: PropertyBank staleness check ────────────────────────────
        let (bank, bank_stale, changed_properties) =
            self.load_property_bank()?;
        let current_bank_version = bank.version();

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas = self.ingestor.all_schemas()?;

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let (stale, fresh_ids) = self.partition_by_staleness(
            &raw_schemas,
            &name_to_id,
            current_bank_version,
            bank_stale,
        )?;

        // ── Step 4b: Incremental resolution for bank-only changes ──────────
        let (stale_for_full_resolution, incrementally_resolved) = self
            .apply_incremental_resolution(
                stale,
                bank_stale,
                &changed_properties,
                &bank,
            )?;

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        // Batch load: O(1) transaction for all fresh schemas
        let fresh_schemas = self
            .repository
            .find_schemas_by_ids(&fresh_ids)
            .map_err(|e| LoaderError::Storage(e.to_string()))?;

        // Convert to HashMap for Extender/Resolver
        let known_parents: HashMap<SchemaId, Schema> = fresh_schemas
            .into_iter()
            .map(|schema| (*schema.id(), schema))
            .collect();

        // ── Step 6: pipeline (RefExpander → Extender → Resolver) ───────────
        let stale_with_raw = stale_for_full_resolution;
        let stale_for_expand: Vec<(SchemaId, RawSchema)> =
            stale_with_raw.clone();

        let mut resolved = if stale_for_expand.is_empty() {
            Vec::new()
        } else {
            let expanded =
                RefExpander::new(&bank).expand_all(stale_for_expand)?;
            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built"
            );
            Resolver::resolve(&tree, &known_parents)?
        };

        // Combine full resolution and incremental resolution results
        resolved.extend(incrementally_resolved);

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_schemas(&resolved, stale_with_raw)?;
        }

        self.repository
            .save_property_bank(&bank)
            .map_err(|e| LoaderError::Storage(e.to_string()))?;

        Ok(resolved)
    }

    /// Load name-to-ID mapping from database.
    fn load_name_to_id_map(
        &self,
    ) -> Result<HashMap<SchemaName, SchemaId>, LoaderError> {
        let existing_pairs = self
            .repository
            .list_schema_name_id_pairs()
            .map_err(|e| LoaderError::Storage(e.to_string()))?;
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(existing_pairs.len());
        for (name, id) in existing_pairs {
            name_to_id.insert(name, id);
        }
        Ok(name_to_id)
    }

    /// Load property bank, checking staleness and rebuilding if needed.
    ///
    /// Returns tuple of (bank, `was_stale`, `changed_property_names`).
    #[expect(
        clippy::type_complexity,
        reason = "Return tuple is clearest for internal pipeline orchestration"
    )]
    fn load_property_bank(
        &self,
    ) -> Result<
        (PropertyBank, bool, Vec<super::property::PropertyName>),
        LoaderError,
    > {
        // Step 1: Load stored bank and view from repository
        let stored_bank = self
            .repository
            .get_property_bank()
            .map_err(|e| LoaderError::Storage(e.to_string()))?;
        let bank_view = self
            .repository
            .get_raw_property_bank_view()
            .map_err(|e| LoaderError::Storage(e.to_string()))?;

        // Step 2: Ingest raw bank from filesystem
        let raw_bank_opt = self.ingestor.property_bank()?;

        // Step 3: Determine staleness based on view.is_fresh()
        let (bank, bank_stale, changed_properties) =
            match (raw_bank_opt, stored_bank, bank_view) {
                (Some(raw_bank), Some(stored), Some(view)) => {
                    // Check if bank is fresh
                    if view.is_fresh(&raw_bank.metadata) {
                        // Fresh - reuse stored bank
                        (stored, false, Vec::new())
                    } else {
                        // Stale - update changed properties incrementally
                        // Compute changed properties FIRST using view helper
                        let changed_props =
                            view.filter_changed_properties(&raw_bank.metadata);

                        // Update only the changed properties
                        let mut updated_bank = stored;
                        updated_bank
                            .update_properties(&raw_bank, &changed_props)?;

                        (updated_bank, true, changed_props)
                    }
                }
                (Some(raw_bank), None, _) | (Some(raw_bank), _, None) => {
                    // New bank (no stored version or no view) - build from
                    // scratch
                    let new_bank = PropertyBank::try_from(raw_bank.clone())?;

                    // All properties are "changed"
                    let all_props: Vec<_> =
                        new_bank.all().map(|p| p.name().clone()).collect();

                    (new_bank, true, all_props)
                }
                (None, Some(stored), _) => {
                    // File disappeared but bank exists in DB - treat as fresh
                    // (could be temporary file system issue)
                    (stored, false, Vec::new())
                }
                (None, None, _) => {
                    // No file and no stored bank - error
                    return Err(LoaderError::Ingestion(
                        SchemaIngestionError::FileSystem(
                            "Property bank file not found".into(),
                        ),
                    ));
                }
            };

        Ok((bank, bank_stale, changed_properties))
    }

    /// Partition schemas into stale and fresh based on view staleness checks.
    ///
    /// Uses the view-based `is_fresh()` pattern for hybrid staleness detection
    /// (timestamps + content hash). Bank staleness forces all schemas stale.
    fn partition_by_staleness(
        &self,
        raw_schemas: &[RawSchema],
        name_to_id: &HashMap<SchemaName, SchemaId>,
        _current_bank_version: crate::schema::bank::BankVersion,
        bank_stale: bool,
    ) -> Result<PartitionResult, LoaderError> {
        let mut stale = Vec::new();
        let mut fresh_ids = Vec::new();

        for raw_schema in raw_schemas {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);

            // Check if schema is new (ID not in name_to_id means newly created)
            let is_new =
                !name_to_id.values().any(|&existing_id| existing_id == id);

            // Load view for staleness check (only if not new)
            let schema_stale = if is_new {
                true // New schemas are always stale
            } else {
                // Get view and check freshness
                let view = self
                    .repository
                    .get_raw_schema_view(id)
                    .map_err(|e| LoaderError::Storage(e.to_string()))?;

                match view {
                    Some(v) => !v.is_fresh(&raw_schema.metadata),
                    None => true, // No view = never loaded = stale
                }
            };

            let is_stale = bank_stale || schema_stale;

            if is_stale {
                stale.push((id, raw_schema.clone()));
            } else {
                fresh_ids.push(id);
            }
        }

        Ok((stale, fresh_ids))
    }

    /// Persist resolved schemas with raw views for staleness tracking.
    fn persist_schemas(
        &self,
        resolved: &[Schema],
        stale_with_raw: Vec<SchemaWithId>,
    ) -> Result<(), LoaderError> {
        // Save resolved schemas
        self.repository
            .save_schemas(resolved)
            .map_err(|e| LoaderError::Storage(e.to_string()))?;

        // Save raw views for staleness tracking
        for (id, raw) in stale_with_raw {
            // Create or update raw schema view
            if let Some(mut view) = self
                .repository
                .get_raw_schema_view(id)
                .map_err(|e| LoaderError::Storage(e.to_string()))?
            {
                // View exists - add new version
                // TODO: Get raw content from ingestor for compression
                let content = String::new(); // Placeholder
                view.add_version(
                    &content,
                    raw.metadata
                        .property_hashes
                        .clone()
                        .into_iter()
                        .filter_map(|(k, v)| {
                            super::property::PropertyName::try_new(k.as_ref())
                                .ok()
                                .map(|name| (name, v))
                        })
                        .collect(),
                    raw.metadata.created_at,
                    raw.metadata.modified_at,
                )
                .map_err(|e| LoaderError::Storage(e.to_string()))?;

                self.repository
                    .save_raw_schema_view(id, &view)
                    .map_err(|e| LoaderError::Storage(e.to_string()))?;
            } else {
                // New view - create it
                // TODO: Get raw content from ingestor
                let content = String::new(); // Placeholder
                let view = super::views::RawSchemaView::new(
                    format!("schemas/{}.toml", raw.name).into_boxed_str(),
                    raw.extends.clone().and_then(|name| {
                        super::aggregate::SchemaName::try_new(&name).ok()
                    }),
                    raw.excludes
                        .iter()
                        .filter_map(|name| {
                            super::property::PropertyName::try_new(
                                name.as_ref(),
                            )
                            .ok()
                        })
                        .collect(),
                    &content,
                    raw.metadata
                        .property_hashes
                        .clone()
                        .into_iter()
                        .filter_map(|(k, v)| {
                            super::property::PropertyName::try_new(k.as_ref())
                                .ok()
                                .map(|prop| (prop, v))
                        })
                        .collect(),
                    raw.metadata.created_at,
                    raw.metadata.modified_at,
                )
                .map_err(|e| LoaderError::Storage(e.to_string()))?;

                self.repository
                    .save_raw_schema_view(id, &view)
                    .map_err(|e| LoaderError::Storage(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Apply incremental resolution for schemas affected by `PropertyBank`
    /// changes.
    ///
    /// This method partitions stale schemas into two groups:
    /// 1. Schemas with file changes → need full resolution
    /// 2. Schemas with only bank changes → need incremental resolution
    ///
    /// For the second group, it applies `resolve_affected_properties()` to
    /// update only the properties that reference changed bank properties.
    #[expect(
        clippy::type_complexity,
        reason = "Return tuple is clearest for pipeline result partitioning"
    )]
    fn apply_incremental_resolution(
        &self,
        stale: Vec<SchemaWithId>,
        bank_stale: bool,
        changed_properties: &[super::property::PropertyName],
        bank: &PropertyBank,
    ) -> Result<(Vec<SchemaWithId>, Vec<Schema>), LoaderError> {
        // Early return if bank is not stale or no properties changed
        if !bank_stale || changed_properties.is_empty() {
            return Ok((stale, Vec::new()));
        }

        // Find schemas that reference changed properties
        let affected_map = self
            .repository
            .find_schemas_using_properties(changed_properties)
            .map_err(|e| LoaderError::Storage(e.to_string()))?;

        // Partition stale schemas: file-changed vs. bank-only-changed
        let mut full_resolution = Vec::new();
        let mut incremental_ids = Vec::new();

        for (id, raw) in stale {
            // Check if schema file itself is stale (has view and is fresh)
            let view = self
                .repository
                .get_raw_schema_view(id)
                .map_err(|e| LoaderError::Storage(e.to_string()))?;

            let file_fresh = view.is_some_and(|v| v.is_fresh(&raw.metadata));

            if file_fresh && affected_map.contains_key(&id) {
                // Schema file is fresh but references changed bank properties
                incremental_ids.push(id);
            } else {
                // Schema file changed OR doesn't reference changed properties
                full_resolution.push((id, raw));
            }
        }

        // Apply incremental resolution to bank-only-changed schemas
        let mut resolved_incremental = Vec::new();
        if !incremental_ids.is_empty() {
            let stored_schemas = self
                .repository
                .find_schemas_by_ids(&incremental_ids)
                .map_err(|e| LoaderError::Storage(e.to_string()))?;

            for schema in &stored_schemas {
                if let Some(affected_props) = affected_map.get(schema.id()) {
                    let updated = Resolver::resolve_affected_properties(
                        schema,
                        affected_props,
                        bank,
                    )?;
                    resolved_incremental.push(updated);
                }
            }
        }

        Ok((full_resolution, resolved_incremental))
    }
}
