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
        raw::{RawPropertyBank, RawSchema},
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

// Type aliases for loader operations
// Metadata is now embedded in RawSchema.metadata, so we only track (ID,
// RawSchema)
type SchemaWithId = (SchemaId, RawSchema);
type PartitionResult = (Vec<SchemaWithId>, Vec<SchemaId>);

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

        // ── Step 2: PropertyBank - ingest raw, check staleness, load if fresh
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

        let bank_stale = match bank_view {
            Some(view) => !view.is_fresh(&raw_bank.metadata),
            None => true,
        };

        let bank = if bank_stale {
            PropertyBank::try_from(raw_bank)?
        } else {
            self.repository
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
                            "Property bank file not found".into(),
                        ),
                    )
                })?
        };
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
            self.persist_schemas(&resolved, &stale)?;
        }

        self.repository
            .save_property_bank(&bank)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

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

    /// Check if the `PropertyBank` is stale.
    ///
    /// Returns `true` if:
    /// - No view exists in DB (never loaded)
    /// - Property bank file appeared/disappeared
    /// - Timestamps differ
    /// - Content hash differs
    #[expect(dead_code, reason = "Will be used when simplifying load() method")]
    fn is_property_bank_stale(
        &self,
        raw_bank: Option<&RawPropertyBank>,
    ) -> Result<bool, SchemaLoaderError> {
        let bank_view =
            self.repository.get_raw_property_bank_view().map_err(|e| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                    path: "database".into(),
                    reason: e.to_string().into(),
                })
            })?;

        Ok(match (raw_bank, bank_view) {
            (Some(raw), Some(view)) => !view.is_fresh(&raw.metadata),
            (Some(_), None) | (None, Some(_)) => true, /* File appeared/ */
            // disappeared
            (None, None) => false, // No bank file (consistent)
        })
    }

    /// Check if a schema is stale.
    ///
    /// Returns `true` if:
    /// - Schema ID is None (new schema)
    /// - No view exists in DB (never loaded)
    /// - Timestamps differ
    /// - Content hash differs
    #[expect(dead_code, reason = "Will be used when simplifying load() method")]
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
    ) -> Result<PartitionResult, SchemaLoaderError> {
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
                let view =
                    self.repository.get_raw_schema_view(id).map_err(|e| {
                        SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                            path: "database".into(),
                            reason: e.to_string().into(),
                        })
                    })?;

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
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Reference pattern needed for iteration over slice"
    )]
    fn persist_schemas(
        &self,
        resolved: &[Schema],
        stale: &[(SchemaId, RawSchema)],
    ) -> Result<(), SchemaLoaderError> {
        // Save resolved schemas
        self.repository
            .save_schemas(resolved)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // Save raw views for staleness tracking
        for (id, raw) in stale {
            // Create or update raw schema view
            if let Some(mut view) =
                self.repository.get_raw_schema_view(*id).map_err(|e| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                        path: "database".into(),
                        reason: e.to_string().into(),
                    })
                })?
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
                .map_err(|e| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                        path: "schema view".into(),
                        reason: e.to_string().into(),
                    })
                })?;

                self.repository
                    .save_raw_schema_view(*id, &view)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
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
                .map_err(|e| {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::Io {
                        path: "schema view".into(),
                        reason: e.to_string().into(),
                    })
                })?;

                self.repository
                    .save_raw_schema_view(*id, &view)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
            }
        }

        Ok(())
    }
}
