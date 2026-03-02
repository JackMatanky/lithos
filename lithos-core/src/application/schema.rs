//! Schema application service — orchestrates the full schema ingestion
//! pipeline.
//!
//! # Pipeline
//!
//! ```text
//! Ingestor
//!   load_raw_property_bank()       → RawPropertyBank
//!   scan_raw_schemas()             → Vec<(RawSchema, Option<Timestamp>)>
//! Query
//!   list_name_id_pairs()           → existing name → id map
//!   get_property_bank()           → Option<PropertyBank>
//! PropertyBank::from_raw()         → PropertyBank
//! Staleness partitioning
//!   is_bank_stale() / is_schema_stale()
//! Query
//!   find_by_id() for each fresh id → known_parents map
//! Dereferencer::deref()            → Vec<(SchemaId, DereferencedSchema)>
//! Extender::build()                → SchemaTree
//! Resolver::resolve()              → Vec<Schema>
//! Command
//!   save_batch()
//!   save_property_bank()
//! ```
//!
//! All schema-specific error types live in [`crate::schema::error`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use std::collections::HashMap;

use crate::schema::{
    adapter::{ingestor::Ingestor, stored::StoredMetadata},
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
    bank::PropertyBank,
    command::Command,
    dereferencer::Dereferencer,
    error::{
        SchemaCommandError, SchemaError, SchemaIngestionError, SchemaQueryError,
    },
    extender::Extender,
    query::Query,
    raw::RawSchema,
    resolver::Resolver,
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaServiceError
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during schema service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaServiceError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] SchemaIngestionError),

    /// Domain validation failed.
    #[error("domain error: {0}")]
    Domain(#[from] SchemaError),

    /// Storage query failed.
    #[error("query error: {0}")]
    Query(#[from] SchemaQueryError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] SchemaCommandError),
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaService
// ─────────────────────────────────────────────────────────────────────────────

/// Thin orchestration service for schema ingestion.
///
/// Uses concrete redb adapters for production use. If testing with mocks
/// is needed in the future, this can be made generic again.
pub struct SchemaService<'db> {
    query: Query<crate::schema::adapter::query::QueryAdapter<'db>>,
    command: Command<crate::schema::adapter::command::CommandAdapter<'db>>,
}

impl<'db> SchemaService<'db> {
    /// Create a new `SchemaService` with query and command adapters.
    #[inline]
    #[must_use]
    pub fn new(
        query: Query<crate::schema::adapter::query::QueryAdapter<'db>>,
        command: Command<crate::schema::adapter::command::CommandAdapter<'db>>,
    ) -> Self {
        Self {
            query,
            command,
        }
    }

    /// Run the full ingestion pipeline.
    ///
    /// Reads raw files via `ingestor`, resolves schemas through the
    /// `Dereferencer → Extender → Resolver` pipeline, and persists the
    /// results.  Only stale schemas are re-resolved; fresh schemas are used
    /// as `known_parents` for the inheritance tree.
    ///
    /// Returns the set of freshly resolved schemas.  Schemas that were
    /// already fresh in the DB are not included (they were not re-resolved).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaServiceError`] on any I/O, parsing, domain, query, or
    /// command failure.
    #[inline]
    pub fn load(
        &self,
        ingestor: &Ingestor<'_>,
    ) -> Result<Vec<Schema>, SchemaServiceError> {
        type SchemaWithTimes =
            (SchemaId, RawSchema, Option<Timestamp>, Option<Timestamp>);
        type TimestampPair = (Option<Timestamp>, Option<Timestamp>);
        type StalenessCheck = (SchemaId, Option<Timestamp>, Option<Timestamp>);

        // ── Step 1: file ingestion ──────────────────────────────────────────
        let raw_bank = ingestor.load_raw_property_bank()?;
        let raw_schemas_with_times = ingestor.scan_raw_schemas()?;

        // ── Step 2: read existing DB state ──────────────────────────────────
        let existing_pairs = self.query.list_name_id_pairs()?;
        let stored_bank = self.query.get_property_bank()?;

        // Build name → id lookup from DB pairs.
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(existing_pairs.len());
        for (name, id) in existing_pairs {
            name_to_id.insert(name, id);
        }

        // ── Step 3: build PropertyBank ──────────────────────────────────────
        let bank = PropertyBank::from_raw(raw_bank, stored_bank.as_ref())?;
        let current_bank_version = bank.version();

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let bank_stale = self.query.is_bank_stale(current_bank_version)?;

        // Build schema ID and staleness check data in one pass
        let mut schema_ids: Vec<SchemaId> =
            Vec::with_capacity(raw_schemas_with_times.len());
        let mut staleness_checks: Vec<StalenessCheck> =
            Vec::with_capacity(raw_schemas_with_times.len());

        #[expect(
            clippy::ref_patterns,
            reason = "Required for borrowing RawSchema while destructuring \
                      tuple"
        )]
        for &(ref raw_schema, modified, created) in &raw_schemas_with_times {
            let schema_name = SchemaName::new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);
            schema_ids.push(id);
            staleness_checks.push((id, created, modified));
        }

        // Batch staleness check: O(1) transaction for all schemas
        let staleness_map = self
            .query
            .batch_is_stale(&staleness_checks, current_bank_version)?;

        let mut stale: Vec<SchemaWithTimes> = Vec::new();
        let mut fresh_ids: Vec<SchemaId> = Vec::new();

        for (id, (raw_schema, modified, created)) in
            schema_ids.into_iter().zip(raw_schemas_with_times.into_iter())
        {
            let schema_stale = staleness_map.get(&id).copied().unwrap_or(true);
            let is_stale = bank_stale || schema_stale;

            if is_stale {
                stale.push((id, raw_schema, modified, created));
            } else {
                fresh_ids.push(id);
            }
        }

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        // Batch load: O(1) transaction for all fresh schemas
        let known_parents = self.query.batch_find_by_ids(&fresh_ids)?;

        // ── Step 6: pipeline (Dereferencer → Extender → Resolver) ──────────
        // Extract just (id, raw_schema) for dereferencer, keep timestamps
        // separate
        let stale_with_times = stale;
        let stale_for_deref: Vec<(SchemaId, RawSchema)> = stale_with_times
            .iter()
            .map(|entry| (entry.0, entry.1.clone()))
            .collect();

        let derefed = Dereferencer::new(&bank).deref(stale_for_deref)?;
        let tree = Extender::build(derefed, &known_parents)?;
        tracing::debug!(
            root_count = tree.roots().len(),
            total_count = tree.nodes().len(),
            "schema tree built"
        );
        let resolved = Resolver::resolve(&tree, &known_parents)?;

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            // Build metadata map: schema_id → (modified, created)
            let mut time_map: HashMap<SchemaId, TimestampPair> =
                HashMap::with_capacity(stale_with_times.len());
            for (id, _, modified, created) in stale_with_times {
                time_map.insert(id, (modified, created));
            }

            // Build metadata vector in same order as schemas
            let metadata: Vec<StoredMetadata> = resolved
                .iter()
                .map(|schema| {
                    let (modified, created) = time_map
                        .get(&schema.id())
                        .copied()
                        .unwrap_or((None, None));
                    StoredMetadata::new(current_bank_version, created, modified)
                })
                .collect();

            // Save with metadata (method only available when C is
            // CommandAdapter) For generic ports, call save_batch
            // directly without metadata
            self.command.save_batch_with_metadata(&resolved, &metadata)?;
        }

        self.command.save_property_bank(&bank)?;

        Ok(resolved)
    }
}
