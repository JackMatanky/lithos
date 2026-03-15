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
    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // ── Step 1: read existing DB state ──────────────────────────────────
        let name_to_id = self.load_name_to_id_map()?;

        // ── Step 2: PropertyBank - handle loading with staleness internally
        let bank = self.load_property_bank()?;

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas = self.ingestor.all_schemas()?;

        // ── Step 4: check staleness for each schema ─────────────────────────
        let mut stale = Vec::new();
        let mut fresh_ids = Vec::new();

        for raw_schema in &raw_schemas {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            let existing_id = name_to_id.get(&schema_name).copied();

            let is_stale = self.is_schema_stale(raw_schema, existing_id)?;

            match (is_stale, existing_id) {
                (true, _) => {
                    let id = existing_id.unwrap_or_else(SchemaId::new);
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

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        let fresh_schemas = self
            .repository
            .find_schemas_by_ids(&fresh_ids)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        let known_parents: HashMap<SchemaId, Schema> = fresh_schemas
            .into_iter()
            .map(|schema| (*schema.id(), schema))
            .collect();

        // ── Step 6: pipeline (RefExpander → Extender → Resolver) ───────────
        let resolved: Vec<Schema> = if stale.is_empty() {
            Vec::new()
        } else {
            let expanded = RefExpander::new(&bank).expand_all(stale.clone())?;
            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built"
            );
            Resolver::resolve(&tree, &known_parents)?
        };

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_resolved_schemas(&resolved)?;
            self.persist_raw_views(&stale)?;
        }

        Ok(resolved)
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
    fn load_property_bank(&self) -> Result<PropertyBank, SchemaLoaderError> {
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
                // Case 3: Fresh - load from DB
                return self
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
                    });
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
            );

            self.repository
                .save_raw_property_bank_view(&view)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

            Ok(bank)
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

            Ok(bank)
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
}
